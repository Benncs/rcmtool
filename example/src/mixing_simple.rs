//! # Simple Example: Mixing Simulation with `cmtool` Functionality
//!
//! **Purpose:**
//! This script demonstrates how to perform a simple mixing simulation in Rust,
//! analogous to the Python `pycmtool` library.
//! It integrates a mass balance over time using sparse matrix representations of the system,
//! with core functionality for chemical or compartmental mixing simulations.
//!
//! **Key Features:**
//! - Constructs flow sparse matrices from system states.
//! - Integrates mass balance equations
//!
//! **Example Workflow:**
//! 1. Define initial mass distribution.
//! 2. Advance the system state and construct sparse matrices.
//! 3. Integrate the system over a specified duration.
//!
//! Author:** CASALE Benjamin
//! Date:** 2025-09-18
//! Version:** 1.0

use cmtool_data::*;
use nalgebra::{DMatrix, DVector};

fn get_transitionner<T: FlowMapTransitionner>() -> Option<T> {
    let root = std::env::var("EXAMPLE_ROOT").unwrap();
    let case_path = format!("{}/cma_case", root);
    let p = std::path::Path::new(&case_path);
    let case = CCMCaseInfo::read_case(p).unwrap();
    T::from_case(&root, &case).ok()
}

fn integration<T: FlowMapTransitionner>(
    fm_t: &mut T,
    mass_0: &DMatrix<f64>,
    n_step: usize,
    duration: f64,
) -> DMatrix<f64> {
    let mut mass_i = mass_0.clone();
    let time_step = duration / (n_step as f64);
    let mut current_time = 0.;
    for _ in 0..n_step {
        let it = fm_t.advance(current_time, time_step);
        let volume_i = it.liquid.get_volume();
        let m = nalgebra::DMatrix::<f64>::from(it.liquid.get_transition());
        let mut concentration_i = mass_i.clone();
        for j in 0..concentration_i.ncols() {
            concentration_i.column_mut(j).scale_mut(1.0 / volume_i[j]);
        }
        let dm = concentration_i * m;
        mass_i += dm.map(|x| x * time_step);
        current_time += time_step;
    }
    mass_i
}

fn check_mixing<T: FlowMapTransitionner>(mut fm_t: T) -> Option<()> {
    let it = fm_t.get_at(0)?;

    let m = &it.liquid.transition;

    let n_c = m.nrows();
    let n_species = 2;

    let mass_0 = {
        let volume_0 = it.liquid.get_volume();
        let mut concentrations: nalgebra::DMatrix<f64> = nalgebra::DMatrix::zeros(n_species, n_c);
        concentrations[(0, 0)] = 1.;
        let volume_0 = nalgebra::DVector::from_row_slice(volume_0);
        for j in 0..concentrations.ncols() {
            concentrations.column_mut(j).scale_mut(volume_0[j]);
        }
        concentrations
    };

    let mass_i = integration(&mut fm_t, &mass_0, 5000, 100.);

    let it = fm_t.get_at(fm_t.size() - 1).unwrap();
    let volume_i = nalgebra::DVector::from_row_slice(it.liquid.get_volume());

    let mut concentration_i = mass_i.clone();
    for j in 0..concentration_i.ncols() {
        concentration_i.column_mut(j).scale_mut(1.0 / volume_i[j]);
    }

    let mean_c = concentration_i.column_mean();
    let concentration_i_i_n = concentration_i.map(|x| x / mean_c[0]);

    println!("{:?}", concentration_i_i_n);

    Some(())
}

fn main() {
    let t: DiscontinuousTransitioner = get_transitionner().unwrap();
    check_mixing(t);
}
