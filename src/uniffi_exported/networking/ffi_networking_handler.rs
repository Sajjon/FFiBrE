use crate::prelude::*;

#[uniffi::export(with_foreign)]
pub trait FFINetworkingHandler: FFIOperationHandler<FFINetworkingOutcomeListener> {
    /// Rust will tell the handler to execute `operation` by calling this
    /// function, which a concrete type FFI side (Swift side) has implemented.
    /// Once the operation has finished with a result (Success/Failure) it
    /// passes back the result using the `listener_rust_side` callback.
    fn execute_network_request(
        &self,
        request: NetworkRequest,
        listener_rust_side: Arc<FFINetworkingOutcomeListener>,
    ) -> Result<(), FFISideError>;
}

impl<U: FFINetworkingHandler> FFIOperationHandler<FFINetworkingOutcomeListener> for U {
    fn execute_operation(
        &self,
        operation: <FFINetworkingOutcomeListener as IsOutcomeListener>::Request,
        listener_rust_side: FFINetworkingOutcomeListener,
    ) -> Result<(), FFISideError> {
        self.execute_network_request(operation, listener_rust_side.into())
    }
}
