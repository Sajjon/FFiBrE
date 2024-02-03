use crate::prelude::*;

/// A handler on the FFI side, which receives request
/// from Rust, executes them and notifies Rust with
/// the result of the FFI operation.
///
/// This is a trait - a protocol - which should be implemented
/// FFI side (Swift side), and then Rust can ask it to execute certain
/// operations, e.g. Network calls.
pub trait FFIOperationHandler: Send + Sync {
    /// A set of supported operations by an [`FFIOperationHandler`],
    /// Rust MUST NOT send any operation to the handler before,
    /// asking it if it supports the operation kind.
    fn supported_operations(&self) -> Vec<FFIOperationKind>;

    /// Rust will tell the handler to execute `operation` by calling this
    /// function, which a concrete type FFI side (Swift side) has implemented.
    /// Once the operation has finished with a result (Success/Failure) it
    /// passes back the result using the `listener_rust_side` callback.
    fn execute_operation(
        &self,
        operation: FFIOperation,
        listener_rust_side: Arc<FFIDataResultListener>,
    ) -> Result<(), SwiftSideError>;
}

#[uniffi::export(with_foreign)]
pub trait FFINetworkRequestHandler: FFIOperationHandler {
    /// Rust will tell the handler to execute `operation` by calling this
    /// function, which a concrete type FFI side (Swift side) has implemented.
    /// Once the operation has finished with a result (Success/Failure) it
    /// passes back the result using the `listener_rust_side` callback.
    fn execute_network_request(
        &self,
        operation: NetworkRequest,
        listener_rust_side: Arc<FFIDataResultListener>,
    ) -> Result<(), SwiftSideError>;
}
impl<U: FFINetworkRequestHandler> FFIOperationHandler for U {
    fn supported_operations(&self) -> Vec<FFIOperationKind> {
        vec![FFIOperationKind::Networking]
    }

    fn execute_operation(
        &self,
        operation: FFIOperation,
        listener_rust_side: Arc<FFIDataResultListener>,
    ) -> Result<(), SwiftSideError> {
        let request = operation.into_networking().expect("Network request");
        self.execute_network_request(request, listener_rust_side)
    }
}
