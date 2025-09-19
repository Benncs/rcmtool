use crate::{CMCase, DataError, FlowMapDescriptor, states::IterationState};
use std::{ops::Index, sync::Arc};

pub struct FlowMapBuffer(Vec<FlowMapDescriptor>, Option<Vec<FlowMapDescriptor>>);

impl FlowMapBuffer {
    pub(crate) fn new_unique(liq: FlowMapDescriptor, gas: Option<FlowMapDescriptor>) -> Self {
        Self(vec![liq], gas.map(|g| vec![g]))
    }

    pub(crate) fn new(
        liq: Vec<FlowMapDescriptor>,
        gas: Option<Vec<FlowMapDescriptor>>,
    ) -> Option<Self> {
        if gas.is_some() && liq.len() != gas.as_ref().unwrap().len() {
            None
        } else {
            Some(Self(liq, gas))
        }
    }
}

impl Index<usize> for FlowMapBuffer {
    type Output = FlowMapDescriptor;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

//impl FlowMapBuffer {
//fn len(&self) -> usize {
//return self.0.len();
//}
//}

fn get_descriptor(
    root: &str,
    case: &CMCase,
) -> Result<(FlowMapDescriptor, Option<FlowMapDescriptor>), DataError> {
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

    Ok((liquid_descriptor, gas_descriptor))
}

pub trait FlowMapTransitionner {
    fn advance(&mut self, current_time: f64, time_step: f64) -> &IterationState;
    fn advance_arc(&mut self, current_time: f64, time_step: f64) -> Arc<IterationState>;
    fn need_advance(&self, current_time: f64, time_step: f64) -> bool;
    fn get_at(&self, idx: usize) -> Option<Arc<IterationState>>;

    fn get_current(&self) -> Arc<IterationState>;
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

            let (liquid, gas) = {
                let mut liquids = Vec::new();
                let mut gases = Vec::new();

                for folder in folders.iter() {
                    let (l, g) = get_descriptor(&format!("{}/{}", root, folder), case).unwrap();
                    liquids.push(l);
                    if let Some(g_val) = g {
                        gases.push(g_val);
                    }
                }

                let gas_vec = if gases.is_empty() { None } else { Some(gases) };

                (liquids, gas_vec)
            };
            FlowMapBuffer::new(liquid, gas).unwrap()
        } else {
            let (liquid_descriptor, gas_descriptor) = get_descriptor(root, case)?;

            FlowMapBuffer::new_unique(liquid_descriptor, gas_descriptor)
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
            .map(|(liq, _gas)| Arc::new(IterationState::new(liq, Some(_gas))))
            .collect(),
        None => buffer
            .0
            .into_iter()
            .map(|fd| Arc::new(IterationState::new(fd, None)))
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

impl FlowMapTransitionner for DiscontinuousTransitioner {
    fn advance(&mut self, current_time: f64, _time_step: f64) -> &IterationState {
        let index_map =
            (current_time / self.time_per_flomap).floor() as usize % self.state_buffer.len();
        self.current_index = index_map;
        &self.state_buffer[index_map]
    }
    fn size(&self) -> usize {
        self.state_buffer.len()
    }

    fn get_current(&self) -> Arc<IterationState> {
        self.state_buffer[self.current_index].clone()
    }

    fn advance_arc(&mut self, current_time: f64, _time_step: f64) -> Arc<IterationState> {
        let index_map =
            (current_time / self.time_per_flomap).floor() as usize % self.state_buffer.len();

        self.current_index = index_map;
        self.state_buffer[index_map].clone()
    }

    fn need_advance(&self, _current_time: f64, _time_step: f64) -> bool {
        true
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

impl FlowMapTransitionner for SimpleTransitioner {
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
        if self.remaining_time >= self.time_per_flomap {
            self.current_index = (self.current_index + 1) % self.state_buffer.len();
            self.remaining_time = 0.;
        }
        self.remaining_time += time_step;
        &self.state_buffer[self.current_index]
    }

    fn get_current(&self) -> Arc<IterationState> {
        self.state_buffer[self.current_index].clone()
    }

    fn need_advance(&self, _current_time: f64, time_step: f64) -> bool {
        (self.remaining_time + time_step) >= self.time_per_flomap
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
