mod private {
    pub trait Sealed {}
}

/// Reserved marker for a future MiraScript external value implementation.
///
/// The trait is sealed intentionally: external values are not implemented by
/// this release and downstream crates cannot implement this marker.
pub trait MiraExtern: std::any::Any + private::Sealed + 'static {}
