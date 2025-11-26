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
use nalgebra::DMatrix;

// We use 2 species to demonstrates that pycmtool can handle different dissolved species
const N_SPECIES: usize = 2;

// fn get_transitionner<T: FlowMapTransitioner>() -> Option<T> {
//     let root = std::env::var("EXAMPLE_ROOT").unwrap();
//     let case_path = format!("{}/cma_case", root);
//     let p = std::path::Path::new(&case_path);
//     let case = CCMCaseInfo::read_case(p).unwrap();
//     //Load all the case information into the iterator
//     T::from_case(&root, &case).ok()
// }

fn integration<T: FlowMapTransitioner>(
    fm_t: &mut T,
    mass_0: &DMatrix<f64>,
    n_step: usize,
    duration: f64,
) -> DMatrix<f64> {
    let mut mass_i = mass_0.clone();
    let time_step = duration / ((n_step - 1) as f64);
    let mut current_time = 0.;
    for _ in 0..n_step {
        //Advance iterator to the corresponding flowmap (according to current_time or time_step)
        let it = fm_t.advance(current_time, time_step);
        //Get precalculated volume inverse
        let inverse_volume_j = &it.liquid.inverse_volume;
        //Convert sparse into dense matrix
        let transition = nalgebra::DMatrix::<f64>::from(it.liquid.get_transition());

        //Get concentration for mass and volume as C=M/V
        let concentration_i = {
            let mut tmp = mass_i.clone();
            (0..tmp.ncols()).for_each(|j| {
                tmp.column_mut(j).scale_mut(inverse_volume_j[j]);
            });
            tmp
        };
        //dm/dt = c*m
        mass_i += time_step * (concentration_i * transition);
        current_time += time_step;
    }

    mass_i
}

fn check_mixing<T: FlowMapTransitioner>(mut fm_t: T, final_time: f64, n_step: usize) -> Option<()> {
    let it = fm_t.get_current();

    let n_c = {
        let t = &it.liquid.transition;
        t.nrows()
    };

    //Get initial mass from volume and concentration
    let mass_0 = {
        let volume_0 = it.liquid.get_volume();
        let mut concentrations: nalgebra::DMatrix<f64> = nalgebra::DMatrix::zeros(N_SPECIES, n_c);
        concentrations[(0, 0)] = 1.;
        (0..concentrations.ncols()).for_each(|j| {
            concentrations.column_mut(j).scale_mut(volume_0[j]);
        });
        concentrations
    };

    let mass_i = integration(&mut fm_t, &mass_0, n_step, final_time);
    let mass_0m = mass_0.column_sum()[0];
    let mass_im = mass_i.column_sum()[0];

    let it = fm_t.get_current();
    let inverse_volume = &it.liquid.inverse_volume;

    let concentration_i = {
        let mut tmp = mass_i.clone();
        (0..tmp.ncols()).for_each(|j| {
            tmp.column_mut(j).scale_mut(inverse_volume[j]);
        });
        tmp
    };

    let mean_c = concentration_i.column_mean();
    let concentration_i_i_n = concentration_i.map(|x| x / mean_c[0]);

    println!("Inital mass: {}", mass_0m);
    println!("Final mass:  {}", mass_im);
    println!("Variance {}", concentration_i_i_n.column_variance()[0]);
    print!("Final normalized C: [");
    for i in 0..5 {
        print!("{}, ", concentration_i_i_n[(0, i)]);
    }
    println!("]");
    assert!((mass_0m - mass_im).abs() < 1e-8);
    Some(())
}

fn main() {
    let final_time: f64 = 50.;
    let n_step: usize = 5000;

    let root = std::env::var("EXAMPLE_ROOT").unwrap();
    //All fonctions use generic, specify iterator type here
    let t: DiscontinuousTransitioner = get_transitioner(&root).unwrap();
    check_mixing(t, final_time, n_step);
}
