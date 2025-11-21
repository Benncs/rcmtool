use std::collections::HashMap;

use crate::FlowMapDescriptor;
use nalgebra_sparse::CooMatrix;
use ndarray::{Array2, Axis};

macro_rules! non_zero {
    ($i:ident, $eps:expr) => {
        ($i.abs() > $eps)
    };
}

fn get_transition_from_fm(fm: Array2<f64>) -> (CooMatrix<f64>, Vec<f64>) {
    let n_compartments: usize = fm.nrows();
    const EPS: f64 = 1e-7;
    // let row_sum = fm.sum_axis(Axis(1));

    let mut transition = CooMatrix::new(n_compartments, n_compartments);
    let mut row_sum = vec![0.; n_compartments];

    (0..n_compartments).for_each(|i_row| {
        for i_col in 0..n_compartments {
            if i_row != i_col {
                let val = *fm.get((i_row, i_col)).expect("Bad formated flowmap");
                if non_zero!(val, EPS) {
                    transition.push(i_row, i_col, val);
                }
                row_sum[i_row] += val;
            }
        }
    });

    (0..n_compartments).for_each(|i_row| {
        let val = row_sum[i_row];
        if non_zero!(val, EPS) {
            transition.push(i_row, i_row, -val);
        }
    });
    (transition, row_sum)
}

#[cfg(feature = "probability")]
fn get_probability(liquid_neighors: &Array2<usize>, transition: &CooMatrix<f64>) -> Array2<f64> {
    use nalgebra_sparse::CscMatrix;

    let shape = liquid_neighors.dim();
    let mut proba = Array2::<f64>::zeros(shape);
    let transition_csc = CscMatrix::from(transition);

    (0..shape.0).for_each(|i_compartment| {
        let mut cumsum = 0.;
        let mut count_neighbor = 0;
        liquid_neighors.row(i_compartment).for_each(|i_neighbor| {
            if *i_neighbor != i_compartment {
                let out_flow = transition_csc
                    .index_entry(i_compartment, i_compartment)
                    .into_value();

                let proba_out: f64 = if out_flow != 0. {
                    transition_csc
                        .index_entry(i_compartment, *i_neighbor)
                        .into_value()
                        / out_flow.abs()
                } else {
                    0.
                };

                let p_cp = proba_out + cumsum;
                *proba.get_mut((i_compartment, count_neighbor)).unwrap() = p_cp;
                cumsum += proba_out;
            }
            count_neighbor += 1;
        });
    });

    proba
}

pub struct HydroState {
    pub transition: CooMatrix<f64>,
    pub out_flows: Vec<f64>,
    pub volumes: Vec<f64>,
    pub inverse_volume: Vec<f64>,
}

impl HydroState {
    #[inline(always)]
    pub fn get_volume(&self) -> &[f64] {
        &self.volumes
    }
    #[inline(always)]
    pub fn get_transition(&self) -> &CooMatrix<f64> {
        &self.transition
    }
    #[inline(always)]
    pub fn n_compartments(&self) -> usize {
        self.volumes.len()
    }
}

impl From<FlowMapDescriptor> for HydroState {
    fn from(value: FlowMapDescriptor) -> Self {
        let inverse = value.volumes.iter().map(|val| 1. / val).collect();
        let (transition, out_flows) = get_transition_from_fm(value.flowmap);
        HydroState {
            volumes: value.volumes,
            out_flows,
            inverse_volume: inverse,
            transition,
        }
    }
}

pub struct IterationState {
    pub liquid: HydroState,
    pub gas: Option<HydroState>,
    pub liquid_neighors: Array2<usize>,
    #[cfg(feature = "probability")]
    pub liquid_cumulative_probability: Array2<f64>,
    pub misc: HashMap<String, Vec<f64>>,
}

impl IterationState {
    pub fn new(
        liq: FlowMapDescriptor,
        gas: Option<FlowMapDescriptor>,
        misc: HashMap<String, Vec<f64>>,
    ) -> Self {
        let liquid_neighors = liq.neighbors.clone(); //Todo find way to remove clone

        let liq_state: HydroState = liq.into();
        #[cfg(feature = "probability")]
        let liquid_cumulative_probability =
            get_probability(&liquid_neighors, &liq_state.transition);

        let ret = Self {
            liquid: liq_state,
            gas: gas.map(|_gas| _gas.into()),
            liquid_neighors,
            #[cfg(feature = "probability")]
            liquid_cumulative_probability,
            misc,
        };

        if ret.gas.is_some() {
            let g = ret.gas.as_ref().unwrap();
            assert!(g.n_compartments() == ret.liquid.n_compartments());
        }

        ret
    }

    #[inline(always)]
    pub fn get(&self, info: &str) -> Option<&[f64]> {
        self.misc.get(info).map(|v| &**v)
    }

    #[inline(always)]
    pub fn n_compartments(&self) -> usize {
        self.liquid.n_compartments()
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{FlowMapDescriptor, IterationState};

    #[test]
    fn construct_itstate_liquid_only() {
        let flow_cma = std::env::var("CUVE_SLDMSH_FLOW_PATH").unwrap();

        let volume_cma = std::env::var("CUVE_SLDMSH_VOLUME_PATH").unwrap();

        let descriptor = FlowMapDescriptor::from_path(flow_cma, volume_cma).unwrap();

        let vol_ref = descriptor.volumes.clone();

        let state = IterationState::new(descriptor, None, HashMap::new());

        assert!(state.liquid.volumes == vol_ref);
    }
}
