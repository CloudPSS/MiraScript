use std::{rc::Rc, time::Duration};

use crate::{RuntimeProviders, default_runtime_providers};

/// Limits and injectable providers used for each Runtime execution.
#[derive(Clone)]
pub struct RunOptions {
    /// Maximum wall-clock time allowed for one execution.
    pub timeout: Duration,
    /// Number of interpreter checkpoints between timeout-provider checks.
    pub checkpoint_interval: u32,
    /// Maximum nested script and native call depth.
    pub max_call_depth: u32,
    /// Maximum number of elements created by bounded array operations.
    pub max_array_len: usize,
    /// Host implementation for random numbers, time, and debug output.
    pub providers: Rc<dyn RuntimeProviders>,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(100),
            checkpoint_interval: 100,
            max_call_depth: 128,
            max_array_len: 0x100_0000,
            providers: default_runtime_providers(),
        }
    }
}
