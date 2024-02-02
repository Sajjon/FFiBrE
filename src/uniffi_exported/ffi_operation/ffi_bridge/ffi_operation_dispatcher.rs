use crate::prelude::*;

/// Rust constructs one or several dispatchers, which are being passed a
/// handler from FFI side (Swift side) of type [`FFIOperationHandler`],
/// it can e.g. be `URLSession` in Swift which supports making network
/// calls.
#[derive(Object)]
pub struct FFIOperationDispatcher {
    /// Handler FFI side, receiving operations from us (Rust side),
    /// and passes result of the operation back to us (Rust side).
    pub handler: Arc<dyn FFIOperationHandler>,
}

impl FFIOperationDispatcher {
    /// Create a new dispatcher with a handler originally passed to Rust
    /// from FFI side (Swift side), e.g. a `URLSession` which implements
    /// the [`FFIOperationHandler`] trait (Swift: conforms to the `FFIOperationHandler`
    /// protocol), with supported [`FFIOperationKind::Networking`] to be able
    /// to make networks calls in Swift - e.g. on iOS, but invoked from Rust.
    pub fn new(handler: Arc<dyn FFIOperationHandler>) -> Self {
        Self { handler }
    }
}
