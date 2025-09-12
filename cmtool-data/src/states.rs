use crate::FlowMapDescriptor;
use nalgebra_sparse::CooMatrix;
use ndarray::{Array2, Axis};
use peroxide::structure::sparse::SPMatrix;

fn get_transition_from_fm(fm: Array2<f64>) -> CooMatrix<f64> {
    let n_compartments: usize = fm.nrows();

    // let row_sum = fm.sum_axis(Axis(1));

    let mut transition = CooMatrix::new(n_compartments, n_compartments);
    let mut row_sum = vec![0.; n_compartments];

    (0..n_compartments).for_each(|i_row| {
        for i_col in 0..n_compartments {
            if i_row != i_col {
                let val = *fm.get((i_row, i_col)).expect("Bad formated flowmap");
                if val != 0. {
                    transition.push(i_row, i_col, val);
                }
                row_sum[i_row] += val;
            }
        }
    });

    (0..n_compartments).for_each(|i_row| {
        let val = row_sum[i_row];
        if val != 0. {
            transition.push(i_row, i_row, -val);
        }
    });
    transition
}

#[cfg(probability)]
fn get_propability_from_fm(fm: &FlowMapDescriptor) -> Array2<f64> {
    todo!()
}

pub struct HydroState {
    pub transition: CooMatrix<f64>,
    pub volumes: Vec<f64>,
    pub inverse_volume: Vec<f64>,
}

impl From<FlowMapDescriptor> for HydroState {
    fn from(value: FlowMapDescriptor) -> Self {
        let inverse = value.volumes.iter().map(|val| 1. / val).collect();
        let transition = get_transition_from_fm(value.flowmap);
        HydroState {
            volumes: value.volumes,
            inverse_volume: inverse,
            transition,
        }
    }
}

pub struct IterationState {
    pub liquid: HydroState,
    pub gas: Option<HydroState>,
    pub liquid_neighors: Array2<usize>,
    #[cfg(probability)]
    pub liquid_cumulative_probability: Array2<f64>,
}

impl IterationState {
    pub fn new(liq: FlowMapDescriptor, gas: Option<FlowMapDescriptor>) -> Self {
        let liquid_neighors = liq.neighbors.clone(); //Todo find way to remove clone

        #[cfg(probability)]
        let liquid_cumulative_probability = get_propability_from_fm(&liq);

        Self {
            liquid: liq.into(),
            gas: gas.map(|_gas| _gas.into()),
            liquid_neighors,
            #[cfg(probability)]
            liquid_cumulative_probability,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{FlowMapDescriptor, IterationState};

    #[test]
    fn construct_itstate_liquid_only() {
        let flow_cma = std::env::var("CUVE_SLDMSH_FLOW_PATH").unwrap();

        let volume_cma = std::env::var("CUVE_SLDMSH_VOLUME_PATH").unwrap();

        let descriptor = FlowMapDescriptor::from_path(flow_cma, volume_cma).unwrap();

        let vol_ref = descriptor.volumes.clone();

        let state = IterationState::new(descriptor, None);

        assert!(state.liquid.volumes == vol_ref);
    }
}
