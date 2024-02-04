use crate::prelude::*;

/// An object representing that Rust is listening on the result of an operation
/// carried out by FFI (Swift-side). When FFI side has finished the operation,
/// either successfully or with failure, it passes back this result to Rust
/// side by calling `notify_outcome`. This is effectively a callback pattern.
pub struct FFIOperationOutcomeListener<R> {
    sender: Mutex<Option<Sender<R>>>,
}

impl<R> FFIOperationOutcomeListener<R> {
    pub(crate) fn new(sender: Sender<R>) -> Self {
        Self {
            sender: Mutex::new(Some(sender)),
        }
    }

    /// This is called from FFI Side (Swift side), inside the implementation of
    /// an `execute_operation:operation:listener_rust_side` method on a [`FFIOperationHandler`],
    /// when the operation has finished, with the `result` of type Self::R
    pub(crate) fn notify_outcome(&self, result: R) {
        self.sender
            .lock()
            .expect("Should only have access sender Mutex once.")
            .take()
            .expect("You MUST NOT call `notifyOutcome` twice in Swift.")
            .send(result)
            .map_err(|_| RustSideError::FailedToPropagateResultFromFFIOperationBackToDispatcher)
            .expect("Must never fail, since some context's in FFI side cannot be throwing.")
    }
}
