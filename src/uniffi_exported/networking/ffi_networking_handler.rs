use crate::prelude::*;

#[uniffi::export(with_foreign)]
pub trait FFINetworkingHandler: FFIOperationHandler<FFINetworkingResultListener> {
    /// Rust will tell the handler to execute `operation` by calling this
    /// function, which a concrete type FFI side (Swift side) has implemented.
    /// Once the operation has finished with a result (Success/Failure) it
    /// passes back the result using the `listener_rust_side` callback.
    fn execute_network_request(
        &self,
        request: NetworkRequest,
        listener_rust_side: Arc<FFINetworkingResultListener>,
    ) -> Result<(), FFISideError>;
}

impl<U: FFINetworkingHandler> FFIOperationHandler<FFINetworkingResultListener> for U {
    fn execute_operation(
        &self,
        operation: <FFINetworkingResultListener as IsResultListener>::Request,
        listener_rust_side: FFINetworkingResultListener,
    ) -> Result<(), FFISideError> {
        self.execute_network_request(operation, listener_rust_side.into())
    }
}
