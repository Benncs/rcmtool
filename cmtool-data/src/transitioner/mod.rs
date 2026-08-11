// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{CMCase, DataError, states::IterationState};
use std::sync::Arc;
mod buffer;
use buffer::{FlowMapBuffer, read_descriptors};

/// A trait describing an abstraction over time-indexed flow-map states.
///
/// `FlowMapTransitioner` provides a unified interface to:
///
/// * step forward in time (`advance`, `advance_arc`)
/// * check whether a transition is required (`need_advance`)
/// * access specific stored states (`get_at`, `get_current`, `get_current_arc`)
/// * query the number of available flow-map entries (`size`)
/// * build the transitioner from a `FlowMapBuffer` or from input case data (`new`, `from_case`)
///
/// # Semantics
///
/// A `FlowMapTransitioner` maintains an time iteration
/// Calling `advance()` or `advance_arc()` advances this index if `need_advance()`
/// returns `true`
//
///
/// Implementations may optimize state caching to minimize `Arc` cloning or
/// recomputation.
///
/// # Errors
///
/// `from_case()` returns a `DataError` if the underlying descriptors cannot be
/// read or if the flow-map buffer cannot be constructed.
///
/// # Example
///
/// ```ignore
/// let transitioner = MyTransitioner::from_case("root/path", &case)?;
/// let state = transitioner.advance(12.0, 0.1);
/// println!("Current state: {:?}", state);
/// ```
pub trait FlowMapTransitioner {
    /// Advances the internal state according to `current_time` and
    /// `time_step`, and returns a **borrowed** reference to the active
    /// `IterationState`.
    ///
    /// Implementations must not clone or allocate here
    fn advance(&mut self, current_time: f64, time_step: f64) -> &IterationState;

    /// Same as [`advance`], but returns an owned `Arc<IterationState>`.
    ///
    /// Implementations should minimize `Arc` cloning and may cache the
    /// currently active state to avoid atomic operations inside tight loops.
    fn advance_arc(&mut self, current_time: f64, time_step: f64) -> Arc<IterationState>;

    /// Returns `true` if the flow-map index should advance when evaluating
    /// the given `current_time` and `time_step`.
    ///
    /// This does not mutate internal state.
    fn need_advance(&self, current_time: f64, time_step: f64) -> bool;

    /// Retrieves the state at the given buffer index as an owned `Arc`.
    ///
    /// Returns `None` if `idx` exceeds the buffer size.
    fn get_at(&self, idx: usize) -> Option<Arc<IterationState>>;

    /// Returns the current state as an owned `Arc<IterationState>`.
    fn get_current_arc(&self) -> Arc<IterationState>;

    /// Returns the current state as a borrowed reference.
    fn get_current(&self) -> &IterationState;

    /// Returns the number of flow-map states stored in this transitioner.
    fn size(&self) -> usize;

    /// Constructs a new transitioner from the duration of each flow-map (`time_per_flomap`)
    /// and a validated [`FlowMapBuffer`].
    ///
    /// The buffer must already satisfy the invariant that either all entries
    /// contain gas descriptors or none do.
    fn new(time_per_flomap: f64, buffer: FlowMapBuffer) -> Self;

    /// Constructs a transitioner by loading flow-map descriptors from disk
    /// using the provided case definition.
    ///
    /// For recursive cases, all flow-map descriptor folders are gathered and
    /// combined into a multi-entry [`FlowMapBuffer`].
    /// For non-recursive cases, only the root folder is read.
    ///
    /// # Errors
    /// Returns `DataError` if descriptor reading fails or if the produced
    /// `FlowMapBuffer` violates flow-map invariants.
    fn from_case(root: &str, case: &CMCase) -> Result<Self, DataError>
    where
        Self: Sized,
    {
        let buffer = if case.is_recursive {
            let folders = case.get_folders(root);

            let mut buffers = Vec::new();
            for folder in folders.iter() {
                buffers.push(read_descriptors(&format!("{}/{}", root, folder), case).unwrap());
            }
            FlowMapBuffer::new(buffers).unwrap()
        } else {
            let buffer = read_descriptors(root, case)?;
            FlowMapBuffer::new_unique(buffer)
        };

        Ok(Self::new(case.time_per_flow_map, buffer))
    }
}

///Type of available transitionner
pub enum TransitionerType {
    ///Time based discontinous transition, easier to manipulate
    Discontinuous,
    ///Index based discontinous transitionner, alway keep current state
    Simple,

    None,
}

/// A transitioner that handles **discontinuous transitions** between flow-map states.
///
/// Each `IterationState` in `state_buffer` represents a flow-map state, and
/// consecutive states are separated by a fixed duration `time_per_flomap`.
///
/// # Behavior
///
/// * The transitioner does **not** maintain an internal time counter;
///   calling `advance` or `advance_arc` with the same `current_time` multiple
///   times are idempotent and always returns the correct state corresponding to `current_time`.
/// * Delta time argument is therefore not used
/// * Transitions are **discontinuous**: as `current_time` crosses multiples of
///   `time_per_flomap`, `current_index` jumps directly to the correct state
///   in `state_buffer`.
/// * Iteration is **cyclic**: when the `current_index` reaches the last state,
///   it wraps around to index 0. For example, with 15 states, iteration goes:
///   `14 -> 0 -> 1 -> ...`.
/// * `current_index` always points to the active flow-map state based on `current_time`.
///
/// # Fields
///
/// * `state_buffer` — A `Vec<Arc<IterationState>>` containing all flow-map states.
/// * `time_per_flomap` — Duration of each flow-map in seconds (or any consistent time unit).
/// * `current_index` — The index of the currently active flow-map state in `state_buffer`.
///
/// # Example
///
/// ```ignore
/// let transitioner = DiscontinuousTransitioner {
///     state_buffer: vec![Arc::new(state1), Arc::new(state2), Arc::new(state3)],
///     time_per_flomap: 0.5,
///     current_index: 0,
/// };
///
/// // current_time = 1.3 -> index = 2 (3rd state)
/// let state = transitioner.advance(1.3, 0.1);
///
/// // current_time = 1.6 -> cycles back to index 0 (if 3 states)
/// let state = transitioner.advance(1.6, 0.1);
/// ```
pub struct DiscontinuousTransitioner {
    state_buffer: Vec<Arc<IterationState>>,
    time_per_flomap: f64,
    current_index: usize,
}

impl DiscontinuousTransitioner {
    #[inline(always)]
    fn index_for_time(&self, t: f64) -> usize {
        (t / self.time_per_flomap).floor() as usize % self.state_buffer.len()
    }

    fn get_index_and_state(&mut self, current_time: f64) -> (usize, &Arc<IterationState>) {
        let index_map = self.index_for_time(current_time);
        self.current_index = index_map;
        (index_map, &self.state_buffer[index_map])
    }
    fn get_state(&mut self, current_time: f64) -> &Arc<IterationState> {
        let index_map = self.index_for_time(current_time);
        self.current_index = index_map;
        &self.state_buffer[index_map]
    }

    pub fn advance_mut(
        &mut self,
        state: &mut Arc<IterationState>,
        current_time: f64,
        _time_step: f64,
    ) -> bool {
        let (_, new_arc) = self.get_index_and_state(current_time);
        let old_ptr = Arc::as_ptr(state);
        let new_ptr = Arc::as_ptr(new_arc);

        if old_ptr != new_ptr {
            *state = new_arc.clone();
            true
        } else {
            false
        }
    }
}

impl FlowMapTransitioner for DiscontinuousTransitioner {
    fn advance(&mut self, current_time: f64, _time_step: f64) -> &IterationState {
        self.get_state(current_time)
    }

    fn advance_arc(&mut self, current_time: f64, _time_step: f64) -> Arc<IterationState> {
        let state = self.get_state(current_time);
        state.clone() // cheap clone
    }

    fn need_advance(&self, current_time: f64, _time_step: f64) -> bool {
        self.state_buffer.len() > 1 && self.index_for_time(current_time) != self.current_index
    }

    fn size(&self) -> usize {
        self.state_buffer.len()
    }

    fn get_current_arc(&self) -> Arc<IterationState> {
        self.state_buffer[self.current_index].clone()
    }

    fn get_current(&self) -> &IterationState {
        &self.state_buffer[self.current_index]
    }

    fn get_at(&self, idx: usize) -> Option<Arc<IterationState>> {
        self.state_buffer.get(idx).cloned()
    }

    fn new(time_per_flomap: f64, buffer: FlowMapBuffer) -> Self {
        let state_buffer = buffer.into_state_buffer();
        Self {
            time_per_flomap,
            state_buffer,
            current_index: 0,
        }
    }
}

pub struct SimpleTransitioner {
    state_buffer: Vec<Arc<IterationState>>,
    time_per_flomap: f64,
    remaining_time: f64,
    current_index: usize,
}

impl FlowMapTransitioner for SimpleTransitioner {
    fn advance_arc(&mut self, _current_time: f64, time_step: f64) -> Arc<IterationState> {
        if self.remaining_time >= self.time_per_flomap {
            self.current_index = (self.current_index + 1) % self.state_buffer.len();
            self.remaining_time = 0.;
        }
        self.remaining_time += time_step;
        self.state_buffer[self.current_index].clone()
    }

    fn get_at(&self, idx: usize) -> Option<Arc<IterationState>> {
        self.state_buffer.get(idx).cloned()
    }

    fn advance(&mut self, _current_time: f64, time_step: f64) -> &IterationState {
        self.remaining_time += time_step;

        if self.remaining_time >= self.time_per_flomap {
            self.remaining_time -= self.time_per_flomap; // more stable than =0
            self.current_index = (self.current_index + 1) % self.state_buffer.len();
        }

        &self.state_buffer[self.current_index]
    }

    fn need_advance(&self, _current_time: f64, _time_step: f64) -> bool {
        true //Actually needs to be alsways updated because of remaining_time
    }

    fn get_current(&self) -> &IterationState {
        &self.state_buffer[self.current_index]
    }

    fn get_current_arc(&self) -> Arc<IterationState> {
        self.state_buffer[self.current_index].clone()
    }

    fn size(&self) -> usize {
        self.state_buffer.len()
    }

    fn new(time_per_flomap: f64, buffer: FlowMapBuffer) -> Self {
        let state_buffer = buffer.into_state_buffer();

        Self {
            state_buffer,
            time_per_flomap,
            remaining_time: 0.,
            current_index: 0,
        }
    }
}
