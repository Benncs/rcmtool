// SPDX-License-Identifier: GPL-3.0-or-later

//! In-repo fixtures, so tests do not depend on external CFD data.

use crate::{FluxFileHeader, RawDataFlux, RawDataScalar, RawFlux};

/// A three-compartment chain `0 -> 1 -> 2` with a small back-flow on each
/// interface, and unit volumes.
///
/// ```text
///        1.0        2.0
///   [0] -----> [1] -----> [2]
///       <-----     <-----
///        0.5        0.25
/// ```
pub(crate) fn chain_of_three() -> (RawDataFlux, RawDataScalar) {
    let fluxes = vec![
        RawFlux {
            id_source: 0,
            id_target: 1,
            flux_source_target: 1.0,
            flux_target_source: 0.5,
        },
        RawFlux {
            id_source: 1,
            id_target: 2,
            flux_source_target: 2.0,
            flux_target_source: 0.25,
        },
    ];

    let flow = RawDataFlux {
        header: FluxFileHeader {
            n_zone: 3,
            n_fluxes: fluxes.len() as u32,
        },
        fluxes,
    };

    (flow, vec![1.0, 2.0, 4.0].into())
}
