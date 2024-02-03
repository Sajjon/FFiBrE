use crate::prelude::*;

/// A handler on the FFI side, which receives request
/// from Rust, executes them and notifies Rust with
/// the result of the FFI operation.
///
/// This is a trait - a protocol - which should be implemented
/// FFI side (Swift side), and then Rust can ask it to execute certain
/// operations, e.g. Network calls.
pub trait FFIOperationHandler<L: ResultListener>: Send + Sync {

    /// Rust will tell the handler to execute `operation` by calling this
    /// function, which a concrete type FFI side (Swift side) has implemented.
    /// Once the operation has finished with a result (Success/Failure) it
    /// passes back the result using the `listener_rust_side` callback.
    fn execute_operation(
        &self,
        operation: L::Request,
        listener_rust_side: L,
    ) -> Result<(), SwiftSideError>;
}
