use crate::prelude::*;

/// Rust constructs one or several dispatchers, which are being passed a
/// handler from FFI side (Swift side) of type [`FFIOperationHandler`],
/// it can e.g. be `URLSession` in Swift which supports making network
/// calls.
pub struct FFIOperationDispatcher<L: IsOutcomeListener> {
    /// Handler FFI side, receiving operations from us (Rust side),
    /// and passes result of the operation back to us (Rust side).
    pub handler: Arc<dyn FFIOperationHandler<L>>,
}

impl<L: IsOutcomeListener> FFIOperationDispatcher<L> {
    /// Create a new dispatcher with a handler originally passed to Rust
    /// from FFI side (Swift side), e.g. a `URLSession` which implements
    /// the [`FFIOperationHandler`] trait (Swift: conforms to the `FFIOperationHandler`
    /// protocol), with supported [`FFIOperationKind::Networking`] to be able
    /// to make networks calls in Swift - e.g. on iOS, but invoked from Rust.
    pub fn new(handler: Arc<dyn FFIOperationHandler<L>>) -> Self {
        Self { handler }
    }

    pub(crate) async fn dispatch(
        &self,
        operation: L::Request,
    ) -> Result<L::Response, FFIBridgeError> {
        // Underlying tokio channel used to get result from Swift back to Rust.
        let (sender, receiver) = channel::<L::Outcome>();

        // Our callback we pass to Swift
        let result_listener = FFIOperationOutcomeListener::new(sender);

        // Make request
        self.handler
            .execute_operation(
                // Pass operation to Swift to make
                operation,
                // Pass callback, Swift will call `result_listener.notify_outcome`
                result_listener.into(),
            )
            .map_err(|e| FFIBridgeError::from(e))?;

        // Await response from Swift
        let response = receiver.await.map_err(|_| FFIBridgeError::FromRust {
            error: RustSideError::FailedToReceiveResponseFromSwift,
        })?;

        response.into().map_err(|e| e.into().into())
    }
}
