use crate::prelude::*;

/// A handler on the FFI side, which receives request
/// from Rust, executes them and notifies Rust with
/// the result of the FFI operation.
#[uniffi::export(with_foreign)]
pub trait FFIOperationHandler: Send + Sync {
    fn supported_operations(&self) -> Vec<FFIOperationKind>;
    fn execute_operation(
        &self,
        operation: FFIOperation,
        listener_rust_side: Arc<FFIDataResultListener>,
    ) -> Result<(), SwiftSideError>;
}
