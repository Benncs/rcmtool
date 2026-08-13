// SPDX-License-Identifier: GPL-3.0-or-later

///Knobs of the balancing pass, `Default` is what the CLI uses unless it is told otherwise
#[derive(Debug, Clone, Copy)]
pub struct BalanceSettings {
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
