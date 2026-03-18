// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt::Write;

///Symmetric mean absolute percentage error
///
/// Details : https://en.wikipedia.org/wiki/Symmetric_mean_absolute_percentage_error
fn smape(a: f64, b: f64) -> f64 {
    if a == 0. && b == 0. {
        return 0.;
    }
    (a - b).abs() / (a.abs() + b.abs())
}

pub fn check_flows(
    _grid: &dyn cmtool_core::grid::CompartmentMeshManip,
    raw_flows: &cmtool_data::RawDataFlux,
) -> Option<String> {
    let mut mass_balance: Vec<(f64, f64)> = vec![(0.0, 0.0); raw_flows.header.n_zone as usize];

    // let boundary = grid.get_boundary();

    for flow in raw_flows.fluxes.iter() {
        // mass_balance[flow.id_target as usize].0 += flow.flux_source_target;
        // mass_balance[flow.id_source as usize].1 += flow.flux_source_target;
        //
        //
        mass_balance[flow.id_target as usize].0 += flow.flux_source_target;
        mass_balance[flow.id_source as usize].1 += flow.flux_source_target;

        mass_balance[flow.id_source as usize].0 += flow.flux_target_source;
        mass_balance[flow.id_target as usize].1 += flow.flux_target_source;
    }

    let mut zone_relative_errors = Vec::with_capacity(raw_flows.header.n_zone as usize);
    let mut total_relative_error: f64 = 0.0;
    let mut max_relative_error: f64 = 0.0;

    let mut total_inflow = 0.0;
    let mut total_outflow = 0.0;
    let mut f = String::new();
    let mut n_zone = 0;

    writeln!(&mut f, "zone_id,int,out,relative_error_percent").unwrap();
    for (i, (inflow, outflow)) in mass_balance.iter().enumerate() {
        // let is_boundary = boundary.iter().find(|&&ci| ci == i);
        // if is_boundary.is_some() {
        //     continue;
        // }
        n_zone += 1;
        let relative_error = smape(*inflow, *outflow);

        zone_relative_errors.push(relative_error);
        total_relative_error += relative_error;
        max_relative_error = max_relative_error.max(relative_error);

        total_inflow += inflow;
        total_outflow += outflow;

        // writeln!(
        //     &mut f,
        //     "{},{},{},{},{}",
        //     i + 1,
        //     inflow,
        //     outflow,
        //     relative_error * 100.0,
        //     is_boundary.is_some()
        // )
        // .unwrap();
        writeln!(
            &mut f,
            "{},{},{},{}",
            i + 1,
            inflow,
            outflow,
            relative_error * 100.0,
        )
        .unwrap();
    }
    writeln!(&mut f).unwrap();

    let mean_relative_error = total_relative_error / n_zone as f64;

    writeln!(&mut f, "metric,value").unwrap();
    writeln!(&mut f, "total_inflow,{:.8}", total_inflow).unwrap();
    writeln!(&mut f, "total_outflow,{:.8}", total_outflow).unwrap();
    writeln!(
        &mut f,
        "global_net_error_percent,{:.8}",
        smape(total_inflow, total_outflow) * 100.0
    )
    .unwrap();
    writeln!(
        &mut f,
        "mean_relative_error_percent,{:.6}",
        mean_relative_error * 100.0
    )
    .unwrap();
    writeln!(
        &mut f,
        "max_relative_error_percent,{:.6}",
        max_relative_error * 100.0
    )
    .unwrap();

    Some(f)
}

pub fn divergence_free(raw_flows: &cmtool_data::RawDataFlux) -> cmtool_data::RawDataFlux {
    use nalgebra::{DMatrix, DVector};
    let n_zone = raw_flows.header.n_zone as usize;
    let n_flux = raw_flows.fluxes.len() * 2;
    let mut new_flows = raw_flows.clone();

    let mut f_vec = DVector::from_element(n_flux, 0.0);
    for (i, flow) in new_flows.fluxes.iter().enumerate() {
        f_vec[i * 2] = flow.flux_source_target;
        f_vec[i * 2 + 1] = flow.flux_target_source;
    }

    let mut a_mat = DMatrix::zeros(n_zone, n_flux);
    for (k, flow) in new_flows.fluxes.iter().enumerate() {
        let s = flow.id_source as usize;
        let t = flow.id_target as usize;

        a_mat[(s, k * 2)] = 1.0;
        a_mat[(t, k * 2)] = -1.0;

        a_mat[(t, k * 2 + 1)] = 1.0;
        a_mat[(s, k * 2 + 1)] = -1.0;
    }

    let div = &a_mat * &f_vec;

    let a_at = a_mat.transpose();
    let aat_inv = match (a_mat.clone() * a_at.clone()).try_inverse() {
        Some(inv) => inv,
        None => panic!("Cannot invert A*A^T"),
    };
    let delta_f = a_at * (aat_inv * (-div));

    for (i, flow) in new_flows.fluxes.iter_mut().enumerate() {
        flow.flux_source_target += delta_f[i * 2];
        flow.flux_target_source += delta_f[i * 2 + 1];

        if flow.flux_source_target < 0.0 || flow.flux_target_source < 0.0 {
            panic!("Negative flux encountered");
        }
    }

    new_flows
}
