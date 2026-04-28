use std::collections::HashMap;

use crate::FlowMapDescriptor;
use nalgebra_sparse::CooMatrix;
use ndarray::Array2;

macro_rules! almost_equal {
    ($i:expr, $base:expr, $eps:expr) => {
        (($i - $base).abs() < $eps)
    };
}

// macro_rules! non_zero {
//     ($i:expr, $eps:expr) => {
//         ($i.abs() > $eps)
//     };
// }

// macro_rules! near_one {
//     ($i:expr, $eps:expr) => {
//         almost_equal!($i, 1.0, $eps)
//     };
// }

macro_rules! round_if_needed {
    ($val:expr, $base:expr, $tol:expr) => {{
        if almost_equal!($val, $base, $tol) {
            $base
        } else {
            $val
        }
    }};
}
fn get_transition_from_fm(fm: Array2<f64>) -> (CooMatrix<f64>, Vec<f64>) {
    let n_compartments: usize = fm.nrows();

    let mut transition = CooMatrix::new(n_compartments, n_compartments);
    let mut row_sum = vec![0.; n_compartments];

    (0..n_compartments).for_each(|i_row| {
        for i_col in 0..n_compartments {
            if i_row != i_col {
                let val = *fm.get((i_row, i_col)).expect("Bad formated flowmap");
                transition.push(i_row, i_col, val);
                row_sum[i_row] += val;
            }
        }
    });

    (0..n_compartments).for_each(|i_row| {
        let val = row_sum[i_row];
        transition.push(i_row, i_row, -val);
    });

    (transition, row_sum)
}

#[cfg(feature = "probability")]
fn get_probability(liquid_neighors: &Array2<usize>, transition: &CooMatrix<f64>) -> Array2<f64> {
    use nalgebra_sparse::CscMatrix;

    //TODO: this method is called even though there's no flow (0D)
    //To allow correct behaviour, assert has the condition outflow==0
    //Find a way cleaner way to : skip test and do not trigger assert

    //TODO change assert to real real and return Result<>
    let shape = liquid_neighors.dim();
    //Start with filled with one array to ensure that probability will be increasing
    let mut proba = Array2::<f64>::ones(shape);
    let transition_csc = CscMatrix::from(transition);
    let ghost_neighor = shape.0 + 1;
    (0..shape.0).for_each(|i_compartment| {
        let mut cumsum = 0.;
        let out_flow = round_if_needed!(
            transition_csc
                .index_entry(i_compartment, i_compartment)
                .into_value()
                .abs(),
            0.,
            1e-12
        );

        //TODO PFR lead to cumsum==-1 how to handle assertion ?

        let mut count_neighbor = 0;
        liquid_neighors.row(i_compartment).for_each(|&i_neighbor| {
            if i_neighbor != ghost_neighor {
                let proba_out: f64 = if out_flow != 0. {
                    transition_csc
                        .index_entry(i_compartment, i_neighbor)
                        .into_value()
                        / out_flow
                } else {
                    0.
                };
                debug_assert!(proba_out >= 0.);
                //round to one if close enough
                let p_cp = round_if_needed!(proba_out + cumsum, 1., 1e-8);

                *proba
                    .get_mut((i_compartment, count_neighbor))
                    .expect("Probability out of bound") = p_cp;

                cumsum += proba_out;
            }
            count_neighbor += 1;
        });
        //TODO PFR lead to cumsum==-1 how to handle assertion ?
        // Idea:
        // let is_pfr = i_compartment == 0 || i_compartment == liquid_neighors.len() - 1;
        //For PFR NEED TO REMOVE THISASSERT FIXME
        // assert!(
        //     (cumsum - 1.0).abs() < 1e-10 || out_flow == 0.,
        //     "compartment {} cumulative probability = {} < 1 (not conservative)",
        //     i_compartment,
        //     cumsum
        // );
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

    #[inline(always)]
    pub fn total_volume(&self) -> f64 {
        self.volumes.iter().sum()
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

        //TODO
        // call this only if liq has flow (transition and neighbors), 0D maps do not have
        //proba
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
        let _flow_cma = std::env::var("CUVE_SLDMSH_FLOW_PATH");
        let _volume_cma = std::env::var("CUVE_SLDMSH_VOLUME_PATH");

        if let (Ok(flow_cma), Ok(volume_cma)) = (_flow_cma, _volume_cma) {
            let descriptor = FlowMapDescriptor::from_path(flow_cma, volume_cma).unwrap();

            let vol_ref = descriptor.volumes.clone();

            let state = IterationState::new(descriptor, None, HashMap::new());

            assert!(state.liquid.volumes == vol_ref);
        }
    }
}
