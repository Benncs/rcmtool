use crate::{CMCase, DataError, FlowMapDescriptor, RawData, rawdata, states::IterationState};
use std::{collections::HashMap, ops::Index, sync::Arc};

pub struct FlowMapBuffer(
    Vec<FlowMapDescriptor>,
    Option<Vec<FlowMapDescriptor>>,
    Vec<HashMap<String, Vec<f64>>>,
);

impl FlowMapBuffer {
    pub(crate) fn new_unique(
        liq: FlowMapDescriptor,
        gas: Option<FlowMapDescriptor>,
        misc: HashMap<String, Vec<f64>>,
    ) -> Self {
        Self(vec![liq], gas.map(|g| vec![g]), vec![misc])
    }

    pub(crate) fn new(
        liq: Vec<FlowMapDescriptor>,
        gas: Option<Vec<FlowMapDescriptor>>,
        misc: Vec<HashMap<String, Vec<f64>>>,
    ) -> Option<Self> {
        if gas.is_some() && liq.len() != gas.as_ref().unwrap().len() {
            None
        } else {
            Some(Self(liq, gas, misc))
        }
    }
}

impl Index<usize> for FlowMapBuffer {
    type Output = FlowMapDescriptor;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

fn get_descriptor(
    root: &str,
    case: &CMCase,
) -> Result<
    (
        FlowMapDescriptor,
        Option<FlowMapDescriptor>,
        HashMap<String, Vec<f64>>,
    ),
    DataError,
> {
    let mut misc = HashMap::new();

    //TODO Improve
    if let Some(eps_path) = case.resolve(root, crate::CMAExportType::EnergyDissipation) {
        let scalar_field = rawdata::RawDataScalar::read_raw(eps_path).ok_or(DataError::BadData)?;
        let value: Vec<f64> = scalar_field.values.iter().map(|v| v.value).collect();
        misc.insert(String::from("energy_dissipation"), value);
    }

    let liq_flow_path = case
        .resolve(root, crate::CMAExportType::LiquidFlow)
        .ok_or(DataError::BadData)?;
    let liq_vol_path = case
        .resolve(root, crate::CMAExportType::LiquidVolume)
        .ok_or(DataError::BadData)?;

    let liquid_descriptor = FlowMapDescriptor::from_path(liq_flow_path, liq_vol_path)?;

    let gas_descriptor = match (
        case.resolve(root, crate::CMAExportType::GasFlow),
        case.resolve(root, crate::CMAExportType::GasVolume),
    ) {
        (Some(gas_flow), Some(gas_volume)) => {
            Some(FlowMapDescriptor::from_path(gas_flow, gas_volume)?)
        }
        _ => None,
    };

    Ok((liquid_descriptor, gas_descriptor, misc))
}

pub trait FlowMapTransitioner {
    fn advance(&mut self, current_time: f64, time_step: f64) -> &IterationState;
    fn advance_arc(&mut self, current_time: f64, time_step: f64) -> Arc<IterationState>;
    fn need_advance(&self, current_time: f64, time_step: f64) -> bool;
    fn get_at(&self, idx: usize) -> Option<Arc<IterationState>>;

    fn get_current_arc(&self) -> Arc<IterationState>;
    fn get_current(&self) -> &IterationState;
    fn size(&self) -> usize;
    //Start with one dt per flowmap, maybe be improve by using different dt per flowmap if needed
    fn new(time_per_flomap: f64, buffer: FlowMapBuffer) -> Self;

    fn from_case(root: &str, case: &CMCase) -> Result<Self, DataError>
    where
        Self: Sized,
    {
        let buffer = if case.is_reursive {
            let mut folders: Vec<String> = std::fs::read_dir(root)
                .unwrap()
                .filter_map(|entry| {
                    if let Ok(dir) = entry {
                        let file_name = dir.file_name();
                        let file_name_str = file_name.to_string_lossy();
                        if let Some(index_str) = file_name_str.strip_prefix("i_") {
                            if index_str.parse::<usize>().is_ok() {
                                Some(file_name_str.to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            folders.sort_by(|a, b| {
                let index_a: usize = a.trim_start_matches("i_").parse().unwrap_or(0);
                let index_b: usize = b.trim_start_matches("i_").parse().unwrap_or(0);
                index_a.cmp(&index_b)
            });

            let (liquid, gas, miscs) = {
                let mut liquids = Vec::new();
                let mut gases = Vec::new();
                let mut miscs = Vec::new();

                for folder in folders.iter() {
                    let (l, g, misc) =
                        get_descriptor(&format!("{}/{}", root, folder), case).unwrap();
                    liquids.push(l);
                    if let Some(g_val) = g {
                        gases.push(g_val);
                    }
                    miscs.push(misc);
                }

                let gas_vec = if gases.is_empty() { None } else { Some(gases) };

                (liquids, gas_vec, miscs)
            };
            FlowMapBuffer::new(liquid, gas, miscs).unwrap()
        } else {
            let (liquid_descriptor, gas_descriptor, misc) = get_descriptor(root, case)?;

            FlowMapBuffer::new_unique(liquid_descriptor, gas_descriptor, misc)
        };

        Ok(Self::new(case.time_per_flow_map, buffer))
    }
}
fn get_state_buffer(buffer: FlowMapBuffer) -> Vec<Arc<IterationState>> {
    match buffer.1 {
        Some(gas) => buffer
            .0
            .into_iter()
            .zip(gas)
            .zip(buffer.2)
            .map(|((liq, _gas), _misc)| Arc::new(IterationState::new(liq, Some(_gas), _misc)))
            .collect(),
        None => buffer
            .0
            .into_iter()
            .zip(buffer.2)
            .map(|(fd, misc)| Arc::new(IterationState::new(fd, None, misc)))
            .collect(),
    }
}
pub enum TransitionerType {
    Discontinuous,
    Simple,
    None,
}
pub struct DiscontinuousTransitioner {
    state_buffer: Vec<Arc<IterationState>>,
    time_per_flomap: f64,
    current_index: usize,
}

impl DiscontinuousTransitioner {
    #[inline(always)]
    fn index_for_time(&self, t: f64) -> usize {
        (t / self.time_per_flomap).floor() as usize % self.state_buffer.len()
    }

    fn get_index_and_state(&mut self, current_time: f64) -> (usize, &Arc<IterationState>) {
        let index_map = self.index_for_time(current_time);
        self.current_index = index_map;
        (index_map, &self.state_buffer[index_map])
    }

    pub fn advance_mut(
        &mut self,
        state: &mut Arc<IterationState>,
        current_time: f64,
        _time_step: f64,
    ) -> bool {
        let (_, new_arc) = self.get_index_and_state(current_time);
        let old_ptr = Arc::as_ptr(state);
        let new_ptr = Arc::as_ptr(new_arc);

        if old_ptr != new_ptr {
            *state = new_arc.clone();
            true
        } else {
            false
        }
    }
}

impl FlowMapTransitioner for DiscontinuousTransitioner {
    fn advance(&mut self, current_time: f64, _time_step: f64) -> &IterationState {
        let (_, state) = self.get_index_and_state(current_time);
        state // Arc -> &IterationState
    }

    fn advance_arc(&mut self, current_time: f64, _time_step: f64) -> Arc<IterationState> {
        let (_, state) = self.get_index_and_state(current_time);
        state.clone() // cheap clone
    }

    fn need_advance(&self, current_time: f64, _time_step: f64) -> bool {
        self.state_buffer.len() > 1 && self.index_for_time(current_time) != self.current_index
    }

    #[inline(always)]
    fn size(&self) -> usize {
        self.state_buffer.len()
    }

    #[inline(always)]
    fn get_current_arc(&self) -> Arc<IterationState> {
        self.state_buffer[self.current_index].clone()
    }

    #[inline(always)]
    fn get_current(&self) -> &IterationState {
        &self.state_buffer[self.current_index]
    }

    fn get_at(&self, idx: usize) -> Option<Arc<IterationState>> {
        self.state_buffer.get(idx).cloned()
    }

    fn new(time_per_flomap: f64, buffer: FlowMapBuffer) -> Self {
        let state_buffer = get_state_buffer(buffer);
        Self {
            time_per_flomap,
            state_buffer,
            current_index: 0,
        }
    }
}

pub struct SimpleTransitioner {
    state_buffer: Vec<Arc<IterationState>>,
    time_per_flomap: f64,
    remaining_time: f64,
    current_index: usize,
}

impl FlowMapTransitioner for SimpleTransitioner {
    fn advance_arc(&mut self, _current_time: f64, time_step: f64) -> Arc<IterationState> {
        if self.remaining_time >= self.time_per_flomap {
            self.current_index = (self.current_index + 1) % self.state_buffer.len();
            self.remaining_time = 0.;
        }
        self.remaining_time += time_step;
        self.state_buffer[self.current_index].clone()
    }

    fn get_at(&self, idx: usize) -> Option<Arc<IterationState>> {
        self.state_buffer.get(idx).cloned()
    }

    fn advance(&mut self, _current_time: f64, time_step: f64) -> &IterationState {
        self.remaining_time += time_step;

        if self.remaining_time >= self.time_per_flomap {
            self.remaining_time -= self.time_per_flomap; // more stable than =0
            self.current_index = (self.current_index + 1) % self.state_buffer.len();
        }

        &self.state_buffer[self.current_index]
    }

    fn need_advance(&self, _current_time: f64, time_step: f64) -> bool {
        true //Actually needs to be alsways updated because of remaining_time 
    }

    fn get_current(&self) -> &IterationState {
        &self.state_buffer[self.current_index]
    }

    fn get_current_arc(&self) -> Arc<IterationState> {
        self.state_buffer[self.current_index].clone()
    }

    fn size(&self) -> usize {
        self.state_buffer.len()
    }

    fn new(time_per_flomap: f64, buffer: FlowMapBuffer) -> Self {
        let state_buffer = get_state_buffer(buffer);

        Self {
            state_buffer,
            time_per_flomap,
            remaining_time: 0.,
            current_index: 0,
        }
    }
}

// pub fn get_transionner(
//     ttype: TransitionerType,
//     root: &str,
//     case: &CMCase,
// ) -> Result<impl FlowMapTransitionner, DataError> {
//     match ttype {
//         TransitionerType::Simple => SimpleTransitioner::from_case(root, case),
//         TransitionerType::Discontinuous => DiscontinuousTransitioner::from_case(root, case),
//         _ => unimplemented!(),
//     }
// }
