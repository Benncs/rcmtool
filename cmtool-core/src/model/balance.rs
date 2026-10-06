// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    coordinates::Coords3,
    model::interfaces::{AInterfacesInfo, InterfaceFlow},
};

pub const ABS_TOLERANCE_DIVERGENCE_CELL: f64 = 1e-7;

///Knobs of the balancing pass, `Default` is what the CLI uses unless it is told otherwise
#[derive(Debug, Clone, Copy)]
pub struct BalanceSettings {
    pub do_balance: bool,
    ///Iterations the balancing may spend before giving up
    pub max_iterations: usize,
    ///Divergence it aims for, convergence is linear so asking for much less costs iterations
    pub tolerance: f64,
    ///Divergence above which the resulting flow map is rejected
    pub max_divergence: f64,
}

impl Default for BalanceSettings {
    fn default() -> Self {
        Self {
            do_balance: true,
            max_iterations: 1000,
            tolerance: 1e-6,
            max_divergence: 1e-2,
        }
    }
}

///What the balancing pass achieved, so that a caller can tell a converged map from a giving up one
pub struct BalanceReport {
    pub iterations: usize,
    pub residual: f64,
}

pub enum InletPosition {
    Point(Coords3),
    Ring(Coords3, f64, f64),
    Circle(Coords3, f64, f64),
    Positions(Vec<usize>),
}

fn calculate_residuals(n_zone: usize, inflow: &[f64], outflow: &[f64]) -> f64 {
    (0..n_zone)
        .map(|zone| {
            let abs_diff = (inflow[zone] - outflow[zone]).abs();
            let total = inflow[zone].abs() + outflow[zone].abs();

            // use abs_diff to not reach numerical noise
            if abs_diff > ABS_TOLERANCE_DIVERGENCE_CELL && total > f64::EPSILON {
                2. * abs_diff / total
            } else {
                0.
            }
        })
        .fold(0.0f64, f64::max)
}

fn calculate_scale_factor(inflow: &[f64], outflow: &[f64], factor: &mut [f64]) {
    //A compartment with no flow either way has nothing to scale
    // (0..n_zone).map(|zone| {
    //     if inflow[zone] > f64::EPSILON && outflow[zone] > f64::EPSILON {
    //         (inflow[zone] / outflow[zone]).sqrt()
    //     } else {
    //         1.
    //     }
    // })
    factor.iter_mut().enumerate().for_each(|(zone, f)| {
        *f = if inflow[zone] > f64::EPSILON && outflow[zone] > f64::EPSILON {
            (inflow[zone] / outflow[zone]).sqrt()
        } else {
            1.
        }
    });
}

///Balance flow
///
///A flow is only ever multiplied by a positive factor. Scaling the
///flows leaving a compartment by `sqrt(in / out)` moves it halfway to its balance, and
///repeating it converges the same way Sinkhorn balancing does.
pub fn balance(
    settings: &BalanceSettings,
    n_zone: usize,
    interfaces: &AInterfacesInfo,
    flows: &mut [InterfaceFlow],
) -> BalanceReport {
    let mut inflow = vec![0.0f64; n_zone];
    let mut outflow = vec![0.0f64; n_zone];
    let mut factor = vec![0.0f64; n_zone];
    let mut report = BalanceReport {
        iterations: 0,
        residual: f64::INFINITY,
    };

    for iteration in 0..settings.max_iterations {
        inflow.fill(0.);
        outflow.fill(0.);
        //Calculate balance
        for (i_interface, flow) in flows.iter().enumerate() {
            let source_id = interfaces.ids[i_interface].source_id;
            let target_id = interfaces.ids[i_interface].target_id;
            outflow[source_id] += flow.source_flow;
            inflow[target_id] += flow.source_flow;
            outflow[target_id] += flow.target_flow;
            inflow[source_id] += flow.target_flow;
        }

        report.iterations = iteration;
        report.residual = calculate_residuals(n_zone, &inflow, &outflow);

        if report.residual < settings.tolerance {
            break;
        }
        calculate_scale_factor(&inflow, &outflow, &mut factor);

        //Apply scaling
        for (i_interface, flow) in flows.iter_mut().enumerate() {
            let source_id = interfaces.ids[i_interface].source_id;
            let target_id = interfaces.ids[i_interface].target_id;
            flow.source_flow *= factor[source_id];
            flow.target_flow *= factor[target_id];
        }
    }

    report
}

struct OpenPhase(InletPosition, f64);

fn fill_inlet_compartments(OpenPhase(pos, fr): &OpenPhase) -> Vec<usize> {
    todo!()
}

fn fill_outlet_compartments(OpenPhase(pos, fr): &OpenPhase) -> Vec<usize> {
    todo!()
}

///Balance flow
///
///A flow is only ever multiplied by a positive factor. Scaling the
///flows leaving a compartment by `sqrt(in / out)` moves it halfway to its balance, and
///repeating it converges the same way Sinkhorn balancing does.
pub fn balance_open(
    settings: &BalanceSettings,
    n_zone: usize,
    interfaces: &AInterfacesInfo,
    flows: &mut [InterfaceFlow],
) -> BalanceReport {
    let mut inflow = vec![0.0f64; n_zone];
    let mut outflow = vec![0.0f64; n_zone];
    let mut factor = vec![0.0f64; n_zone];
    let mut report = BalanceReport {
        iterations: 0,
        residual: f64::INFINITY,
    };

    let ph = OpenPhase(InletPosition::Point([0., 0., 0.]), 0.1);

    let inlet_1 = fill_inlet_compartments(&ph);
    let outlet_i = fill_outlet_compartments(&ph);
    let n_inlet = inlet_1.len();
    let n_outlet = outlet_i.len();
    for iteration in 0..settings.max_iterations {
        inflow.fill(0.);
        outflow.fill(0.);

        (0..n_inlet).for_each(|i_i| {
            inflow[inlet_1[i_i]] += ph.1;
        });

        (0..n_outlet).for_each(|i_i| {
            outflow[outlet_i[i_i]] += ph.1;
        });

        //Calculate balance
        for (i_interface, flow) in flows.iter().enumerate() {
            let source_id = interfaces.ids[i_interface].source_id;
            let target_id = interfaces.ids[i_interface].target_id;
            outflow[source_id] += flow.source_flow;
            inflow[target_id] += flow.source_flow;
            outflow[target_id] += flow.target_flow;
            inflow[source_id] += flow.target_flow;
        }

        report.iterations = iteration;
        report.residual = calculate_residuals(n_zone, &inflow, &outflow);

        if report.residual < settings.tolerance {
            break;
        }
        calculate_scale_factor(&inflow, &outflow, &mut factor);

        //Apply scaling
        for (i_interface, flow) in flows.iter_mut().enumerate() {
            let source_id = interfaces.ids[i_interface].source_id;
            let target_id = interfaces.ids[i_interface].target_id;
            flow.source_flow *= factor[source_id];
            flow.target_flow *= factor[target_id];
        }
    }

    report
}
