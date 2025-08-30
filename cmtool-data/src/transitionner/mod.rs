use crate::{CMCase, DataError, FlowMapDescriptor, RawData, RawDataFlux, states::IterationState};
use enum_dispatch::enum_dispatch;
use ndarray::Array2;
use std::{iter::repeat, ops::Index, sync::Arc};

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

impl FlowMapBuffer {
    fn len(&self) -> usize {
        return self.0.len();
    }
}

pub trait FlowMapTransitionner {
    fn advance(&mut self, time_step: f64) -> &IterationState;
    fn advance_arc(&mut self, time_step: f64) -> Arc<IterationState>;
    fn need_advance(&self, time_step: f64) -> bool;
    fn get_at(&self, idx: usize) -> Option<Arc<IterationState>>;
    //Start with one dt per flowmap, maybe be improve by using different dt per flowmap if needed
    fn new(time_per_flomap: f64, buffer: FlowMapBuffer) -> Self;

    fn from_case(root: &str, case: &CMCase) -> Result<Self, DataError>
    where
        Self: Sized,
    {
        let buffer = if case.recur {
            todo!()
        } else {
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

            FlowMapBuffer::new_unique(liquid_descriptor, gas_descriptor)
        };

        Ok(Self::new(case.time_per_flow_map, buffer))
    }
}

pub struct DiscontinuousTransitioner {
    state_buffer: Vec<Arc<IterationState>>,
    time_per_flomap: f64,
    remaining_time: f64,
    current_index: usize,
}

pub enum TransitionerType {
    Discontinuous,
    None,
}

pub fn get_transionner(
    ttype: TransitionerType,
    root: &str,
    case: &CMCase,
) -> Result<impl FlowMapTransitionner, DataError> {
    match ttype {
        TransitionerType::Discontinuous => DiscontinuousTransitioner::from_case(root, case),
        _ => unimplemented!(),
    }
}

impl FlowMapTransitionner for DiscontinuousTransitioner {
    fn advance_arc(&mut self, time_step: f64) -> Arc<IterationState> {
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
    fn advance(&mut self, time_step: f64) -> &IterationState {
        if self.remaining_time >= self.time_per_flomap {
            self.current_index = (self.current_index + 1) % self.state_buffer.len();
            self.remaining_time = 0.;
        }
        self.remaining_time += time_step;
        &self.state_buffer[self.current_index]
    }

    fn need_advance(&self, time_step: f64) -> bool {
        (self.remaining_time + time_step) >= self.time_per_flomap
    }

    fn new(time_per_flomap: f64, buffer: FlowMapBuffer) -> Self {
        let state_buffer: Vec<Arc<IterationState>> = match buffer.1 {
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
        };

        Self {
            state_buffer,
            time_per_flomap,
            remaining_time: 0.,
            current_index: 0,
        }
    }
}
