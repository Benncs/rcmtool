// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    coordinates::Coords3,
    errors::ModelError,
    grid::CompartmentMesh,
    model::interfaces::{AInterfacesInfo, InterfaceFlow},
};
use std::collections::HashMap;

pub const ABS_TOLERANCE_DIVERGENCE_CELL: f64 = 1e-7;
const ABS_TOL: f64 = 1e-12;
const REL_TOL: f64 = f64::EPSILON;
const N_SAMPLE_RADIUS: usize = 64;
const N_SAMPLE_ANGLE: usize = 256;
const Z_AXIS: usize = 2;

///Balance Settings
#[derive(Debug, Clone, Copy)]
pub struct BalanceSettings {
    pub do_balance: bool,
    ///Max number of iterations
    pub max_iterations: usize,
    /// Divergence accepted
    pub tolerance: f64,
    ///Divergence reject
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

/// the balancing pass report
pub struct BalanceReport {
    pub iterations: usize,
    pub residual: f64,
    ///Open phase compartments with their flow, empty for a closed phase
    pub inlet: Vec<(usize, f64)>,
    pub outlet: Vec<(usize, f64)>,
}

pub enum InletPosition {
    Point(Coords3),
    //center, radius, height above the center
    Ring(Coords3, f64, f64),
    Circle(Coords3, f64, f64),
    //list of compartment indice (extractor from Eg reader or from elsewhere)
    Positions(Vec<usize>),
}

///Relative error between a and b, 0 under the abs or rel tolerance
fn err(a: f64, b: f64) -> f64 {
    let abs = (a - b).abs();
    let rel = 2. * abs / (a.abs() + b.abs());
    if abs > ABS_TOL && rel > REL_TOL {
        rel
    } else {
        0.
    }
}

fn calculate_residuals(n_zone: usize, inflow: &[f64], outflow: &[f64]) -> f64 {
    (0..n_zone)
        .map(|zone| err(inflow[zone], outflow[zone]))
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

fn calculate_balance(
    flows: &[InterfaceFlow],
    interfaces: &AInterfacesInfo,
    inflow: &mut [f64],
    outflow: &mut [f64],
) {
    //Calculate balance
    for (i_interface, flow) in flows.iter().enumerate() {
        let source_id = interfaces.ids[i_interface].source_id;
        let target_id = interfaces.ids[i_interface].target_id;
        outflow[source_id] += flow.source_flow;
        inflow[target_id] += flow.source_flow;
        outflow[target_id] += flow.target_flow;
        inflow[source_id] += flow.target_flow;
    }
}

fn apply_scaling(flows: &mut [InterfaceFlow], interfaces: &AInterfacesInfo, factor: &[f64]) {
    //Apply scaling
    for (i_interface, flow) in flows.iter_mut().enumerate() {
        let source_id = interfaces.ids[i_interface].source_id;
        let target_id = interfaces.ids[i_interface].target_id;
        flow.source_flow *= factor[source_id];
        flow.target_flow *= factor[target_id];
    }
}

pub struct OpenPhase(pub InletPosition, pub f64);

///Compartments receiving the injected flow and their share, summing up to the flow rate of the
///phase
fn fill_inlet_compartments(
    grid: &dyn CompartmentMesh,
    OpenPhase(pos, fr): &OpenPhase,
) -> Vec<(usize, f64)> {
    match pos {
        InletPosition::Point(point) => grid
            .cell_from_coordinates(point)
            .map(|cell| vec![(cell, *fr)])
            .unwrap_or_default(),
        InletPosition::Ring(center, radius, height) => {
            //Every theta sector crossed by the ring gets the same share
            let mut cells: Vec<usize> = (0..N_SAMPLE_ANGLE)
                .filter_map(|k| {
                    let angle = 2. * std::f64::consts::PI * (k as f64) / (N_SAMPLE_ANGLE as f64);
                    grid.cell_from_coordinates(&[
                        center[0] + radius * angle.cos(),
                        center[1] + radius * angle.sin(),
                        center[2] + height,
                    ])
                })
                .collect();
            cells.sort_unstable();
            cells.dedup();

            let n_cell = cells.len() as f64;
            cells.into_iter().map(|cell| (cell, fr / n_cell)).collect()
        }
        InletPosition::Circle(center, radius, height) => {
            //Midpoint sampling of the disk, a sample stands for the area r*dr*dphi, dr*dphi being
            //constant it cancels out once scaled to the flow rate
            let dr = radius / (N_SAMPLE_RADIUS as f64);
            let mut area: HashMap<usize, f64> = HashMap::new();
            for i in 0..N_SAMPLE_RADIUS {
                let r = (i as f64 + 0.5) * dr;
                for k in 0..N_SAMPLE_ANGLE {
                    let angle = 2. * std::f64::consts::PI * (k as f64) / (N_SAMPLE_ANGLE as f64);
                    if let Some(cell) = grid.cell_from_coordinates(&[
                        center[0] + r * angle.cos(),
                        center[1] + r * angle.sin(),
                        center[2] + height,
                    ]) {
                        *area.entry(cell).or_default() += r;
                    }
                }
            }

            let total: f64 = area.values().sum();
            area.into_iter()
                .map(|(cell, a)| (cell, fr * a / total))
                .collect()
        }
        InletPosition::Positions(cells) => {
            let n_cell = cells.len() as f64;
            cells.iter().map(|&cell| (cell, fr / n_cell)).collect()
        }
    }
}

///Compartments the phase leaves through, the top layer
fn fill_outlet_compartments(grid: &dyn CompartmentMesh, _: &OpenPhase) -> Vec<usize> {
    let top = grid.n_points_axis(Z_AXIS) - 1;
    (0..grid.number_cell())
        .filter(|&cell| grid.cell_points(cell)[Z_AXIS] == top)
        .collect()
}

///Correction of an open phase, called once the interfaces flows are summed: the injected flow
///enters the inlet compartments and an outlet compartment lets out what it gets in excess of what
///it sends to its neighbors so that in == out. Nothing goes out of one sending more than it gets,
///the scaling lowers what it sends. Returns the compartments really letting out with their flow
fn apply_open_phase(
    inlet: &[(usize, f64)],
    outlet: &[usize],
    inflow: &mut [f64],
    outflow: &mut [f64],
) -> Vec<(usize, f64)> {
    inlet.iter().for_each(|&(zone, f)| {
        inflow[zone] += f;
    });
    outlet
        .iter()
        .filter_map(|&zone| {
            let f = (inflow[zone] - outflow[zone]).max(0.);
            outflow[zone] += f;
            (f > 0.).then_some((zone, f))
        })
        .collect()
}

///Balance flow
///
///A flow is only ever multiplied by a positive factor. Scaling the
///flows leaving a compartment by `sqrt(in / out)` moves it halfway to its balance, and
///repeating it converges the same way Sinkhorn balancing does.
/// for open phase injected flow added to what enters the inlet compartments and the
///extracted flow to what leaves the outlet ones, neither is scaled.
pub fn balance_2(
    settings: &BalanceSettings,
    n_zone: usize,
    grid: &dyn CompartmentMesh,
    interfaces: &AInterfacesInfo,
    flows: &mut [InterfaceFlow],
    phase: Option<OpenPhase>,
) -> Result<BalanceReport, ModelError> {
    let mut inflow = vec![0.0f64; n_zone];
    let mut outflow = vec![0.0f64; n_zone];
    let mut factor = vec![0.0f64; n_zone];
    let mut report = BalanceReport {
        iterations: 0,
        residual: f64::INFINITY,
        inlet: Vec::new(),
        outlet: Vec::new(),
    };

    let (inlet, outlet) = match &phase {
        Some(p) => (
            fill_inlet_compartments(grid, p),
            fill_outlet_compartments(grid, p),
        ),
        None => (Vec::new(), Vec::new()),
    };

    //One more pass than scalings so that the residual checked is the one of the returned map
    for iteration in 0..=settings.max_iterations {
        inflow.fill(0.);
        outflow.fill(0.);

        calculate_balance(flows, interfaces, &mut inflow, &mut outflow);
        report.outlet = apply_open_phase(&inlet, &outlet, &mut inflow, &mut outflow);

        report.iterations = iteration;
        report.residual = calculate_residuals(n_zone, &inflow, &outflow);

        if report.residual < settings.tolerance || iteration == settings.max_iterations {
            break;
        }

        calculate_scale_factor(&inflow, &outflow, &mut factor);
        apply_scaling(flows, interfaces, &factor);
    }

    if report.residual > settings.max_divergence {
        return Err(ModelError::CellToCellDivergence(
            report.residual,
            settings.max_divergence,
        ));
    }

    //What leaves has to be what enters
    let q_in: f64 = inlet.iter().map(|(_, f)| f).sum();
    let q_out: f64 = report.outlet.iter().map(|(_, f)| f).sum();
    if !inlet.is_empty() && err(q_in, q_out) > settings.max_divergence {
        return Err(ModelError::InletOutletMismatch(q_in, q_out));
    }
    report.inlet = inlet;

    Ok(report)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::grid::{AxisDescriptor, MeshType, get_mesh};

    fn mesh() -> Box<dyn CompartmentMesh> {
        get_mesh(
            MeshType::Cylindrical,
            [
                AxisDescriptor::new(0., 4., 4),
                AxisDescriptor::new(-std::f64::consts::PI, std::f64::consts::PI, 4),
                AxisDescriptor::new(0., 4., 4),
            ],
        )
    }

    #[test]
    fn test_fill_point() {
        let pos = InletPosition::Point([0., 0., 0.]);
        let flowrate = 0.1;
        let inlet = fill_inlet_compartments(&*mesh(), &super::OpenPhase(pos, flowrate));
        assert!((inlet.iter().map(|(_, f)| f).sum::<f64>() - flowrate).abs() < 1e-12);
    }

    #[test]
    fn test_fill_ring() {
        let ring_center = [0., 0., 0.];
        let ring_radius = 1.;
        let ring_height = 1.;
        let pos = InletPosition::Ring(ring_center, ring_radius, ring_height);
        let flowrate = 0.1;
        let inlet = fill_inlet_compartments(&*mesh(), &super::OpenPhase(pos, flowrate));
        assert!((inlet.iter().map(|(_, f)| f).sum::<f64>() - flowrate).abs() < 1e-12);
    }

    #[test]
    fn test_fill_circle() {
        let circle_center = [0., 0., 0.];
        let circle_radius = 1.;
        let circle_height = 1.;
        let pos = InletPosition::Circle(circle_center, circle_radius, circle_height);
        let flowrate = 0.1;
        let inlet = fill_inlet_compartments(&*mesh(), &super::OpenPhase(pos, flowrate));
        assert!((inlet.iter().map(|(_, f)| f).sum::<f64>() - flowrate).abs() < 1e-12);
    }

    #[test]
    fn test_fill_positions() {
        let pos = InletPosition::Positions(vec![0, 5, 4, 3]);
        let flowrate = 0.1;
        let inlet = fill_inlet_compartments(&*mesh(), &super::OpenPhase(pos, flowrate));
        assert!((inlet.iter().map(|(_, f)| f).sum::<f64>() - flowrate).abs() < 1e-12);
    }
}
