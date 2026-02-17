// SPDX-License-Identifier: GPL-3.0-or-later

use std::ptr::null;

use cmtool_data::{
    DiscontinuousTransitioner, FlowMapTransitioner, HydroState, IterationState, get_transitioner,
};
use nalgebra_sparse::CooMatrix;
struct TransitionerWrapper(DiscontinuousTransitioner);

struct IterationStateWrapper(*const IterationState);

struct HydroStateWrapper(*const HydroState);

struct CooMatrixWrap(*const CooMatrix<f64>);

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        type TransitionerWrapper;
        type IterationStateWrapper;
        type HydroStateWrapper;
        type CooMatrixWrap;

        fn get_dtransitioner(root: &str) -> Result<Box<TransitionerWrapper>>;

        fn advance(
            self: &mut TransitionerWrapper,
            _current_time: f64,
            time_step: f64,
        ) -> Box<IterationStateWrapper>;

        // fn advance_mut(
        //     self: &mut TransitionerWrapper,
        //     boxed_wrapper: &mut Box<IterationStateWrapper>,
        //     current_time: f64,
        //     _time_step: f64,
        // ) -> bool;

        fn get_current(self: &TransitionerWrapper) -> Box<IterationStateWrapper>;

        fn need_advance(self: &TransitionerWrapper, current_time: f64, time_step: f64) -> bool;

        fn get_at(self: &TransitionerWrapper, index: usize) -> Box<IterationStateWrapper>;

        fn size(self: &TransitionerWrapper) -> usize;

        fn get_liquid(self: &IterationStateWrapper) -> Box<HydroStateWrapper>;

        fn get_gas(self: &IterationStateWrapper) -> Box<HydroStateWrapper>;

        fn has_gas(self: &IterationStateWrapper) -> bool;

        #[allow(clippy::needless_lifetimes)]
        unsafe fn get_misc<'a>(self: &'a IterationStateWrapper, key: &str) -> &'a [f64];

        fn has_misc(self: &IterationStateWrapper, key: &str) -> bool;

        fn flat_neighobrs(self: &IterationStateWrapper) -> &[usize];

        fn n_compartments(self: &IterationStateWrapper) -> usize;

        fn flat_probability_leaving(self: &IterationStateWrapper) -> &[f64];

        //HydroState
        fn n_compartments(self: &HydroStateWrapper) -> usize;
        fn is_valid(self: &HydroStateWrapper) -> bool;

        fn volume(self: &HydroStateWrapper) -> &[f64];
        fn inverse_volume(self: &HydroStateWrapper) -> &[f64];

        fn transition(self: &HydroStateWrapper) -> Box<CooMatrixWrap>;

        fn out_flows(self: &HydroStateWrapper) -> &[f64];

        //COO Matrix

        pub fn nrows(self: &CooMatrixWrap) -> usize;

        pub fn ncols(self: &CooMatrixWrap) -> usize;

        pub fn row_indices(self: &CooMatrixWrap) -> &[usize];

        pub fn col_indices(self: &CooMatrixWrap) -> &[usize];

        pub fn values(self: &CooMatrixWrap) -> &[f64];
    }
}

impl CooMatrixWrap {
    fn nrows(&self) -> usize {
        unsafe { (*self.0).nrows() }
    }

    fn ncols(&self) -> usize {
        unsafe { (*self.0).ncols() }
    }

    fn row_indices(&self) -> &[usize] {
        unsafe { (*self.0).row_indices() }
    }

    fn col_indices(&self) -> &[usize] {
        unsafe { (*self.0).col_indices() }
    }

    fn values(&self) -> &[f64] {
        unsafe { (*self.0).values() }
    }
}

impl TransitionerWrapper {
    #[inline]
    fn get_current(&self) -> Box<IterationStateWrapper> {
        Box::new(IterationStateWrapper(self.0.get_current()))
    }

    #[inline]
    fn need_advance(&self, current_time: f64, time_step: f64) -> bool {
        self.0.need_advance(current_time, time_step)
    }

    // fn advance_mut(
    //     &mut self,
    //     boxed_wrapper: &mut Box<IterationStateWrapper>,
    //     current_time: f64,
    //     time_step: f64,
    // ) -> bool {
    //     let IterationStateWrapper(ref mut state) = **boxed_wrapper;

    //     let new_state = self.0.advance(current_time, time_step);

    //     let old_ptr = *state;
    //     let new_ptr = new_state as *const IterationState;

    //     if old_ptr != new_ptr {
    //         *state = new_state;
    //         true
    //     } else {
    //         false
    //     }
    // }

    fn advance(&mut self, current_time: f64, time_step: f64) -> Box<IterationStateWrapper> {
        let state = self.0.advance(current_time, time_step);
        // Returning a reference wrapped in a raw pointer
        Box::new(IterationStateWrapper(state))
    }
    #[inline]
    fn size(&self) -> usize {
        self.0.size()
    }
    #[inline]
    fn get_at(&self, index: usize) -> Box<IterationStateWrapper> {
        Box::new(IterationStateWrapper(&*self.0.get_at(index).unwrap()))
    }
}

fn get_dtransitioner(root: &str) -> Result<Box<TransitionerWrapper>, String> {
    match get_transitioner(root) {
        Ok(t) => Ok(Box::new(TransitionerWrapper(t))),
        Err(d) => Err(format!("Error while reading {}: {}", root, d)),
    }
}

impl IterationStateWrapper {
    fn get_liquid(&self) -> Box<HydroStateWrapper> {
        Box::new(HydroStateWrapper(&unsafe { &*self.0 }.liquid))
    }

    fn n_compartments(&self) -> usize {
        unsafe { &*self.0 }.n_compartments()
    }

    fn get_gas(&self) -> Box<HydroStateWrapper> {
        match &unsafe { &*self.0 }.gas {
            Some(t) => Box::new(HydroStateWrapper(t)),
            None => Box::new(HydroStateWrapper(null())),
        }
    }
    fn has_misc(self: &IterationStateWrapper, key: &str) -> bool {
        unsafe { &*self.0 }.get(key).is_some()
    }

    fn get_misc(self: &IterationStateWrapper, key: &str) -> &[f64] {
        unsafe { &*self.0 }.get(key).unwrap()
    }

    fn has_gas(&self) -> bool {
        unsafe { &*self.0 }.gas.is_some()
    }

    fn flat_neighobrs(&self) -> &[usize] {
        unsafe { &*self.0 }
            .liquid_neighors
            .as_slice_memory_order()
            .unwrap()
    }

    fn flat_probability_leaving(&self) -> &[f64] {
        unsafe { &*self.0 }
            .liquid_cumulative_probability
            .as_slice_memory_order()
            .unwrap()
    }
}

impl HydroStateWrapper {
    fn volume(&self) -> &[f64] {
        let deref = unsafe { &*self.0 };
        deref.get_volume()
    }

    fn inverse_volume(&self) -> &[f64] {
        let deref = unsafe { &*self.0 };
        &deref.inverse_volume
    }

    fn transition(&self) -> Box<CooMatrixWrap> {
        let coo = unsafe { (*self.0).get_transition() };
        // let csc = CscMatrix::from(coo);
        Box::new(CooMatrixWrap(coo))
    }

    fn out_flows(&self) -> &[f64] {
        unsafe { &(*self.0).out_flows }
    }

    fn is_valid(&self) -> bool {
        !self.0.is_null()
    }

    pub fn n_compartments(&self) -> usize {
        unsafe { &*self.0 }.volumes.len()
    }
}
