use std::{
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

struct SystemRuntimeProviders;
impl RuntimeProviders for SystemRuntimeProviders {}

/// Host capabilities used by non-deterministic standard-library functions.
pub trait RuntimeProviders {
    /// Return a uniformly distributed random number in `[0, 1)`.
    fn random(&self) -> f64 {
        rand::random()
    }

    /// Return the current Unix timestamp in milliseconds.
    fn now_millis(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }

    /// Emit one debug message from a script.
    fn debug(&self, message: &str) {
        eprintln!("{message}");
    }
}

std::thread_local! {
    static DEFAULT_RUNTIME_PROVIDERS: Rc<dyn RuntimeProviders> = Rc::new(SystemRuntimeProviders);
}

/// Return a reference-counted handle to the default runtime providers.
pub fn default_runtime_providers() -> Rc<dyn RuntimeProviders> {
    DEFAULT_RUNTIME_PROVIDERS.with(Clone::clone)
}
