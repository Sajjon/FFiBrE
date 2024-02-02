use crate::prelude::*;

/// An object representing that Rust is listening on the result of an operation
/// carried out by FFI (Swift-side). When FFI side has finished the operation,
/// either successfully or with failure, it passes back this result to Rust
/// side by calling `notify_result`. This is effectively a callback pattern.
#[derive(Object)]
pub struct FFIDataResultListener {
    sender: Mutex<Option<Sender<FFIOperationResult>>>,
}

impl FFIDataResultListener {
    pub fn new(sender: Sender<FFIOperationResult>) -> Self {
        Self {
            sender: Mutex::new(Some(sender)),
        }
    }
}

#[export]
impl FFIDataResultListener {
    /// This is called from FFI Side (Swift side), inside the implementation of
    /// an `execute_operation:operation:listener_rust_side` method on a [`FFIOperationHandler`],
    /// when the operation has finished, with the [`FFIOperationResult`].
    fn notify_result(&self, result: FFIOperationResult) {
        let sender = self.sender.lock().unwrap().take().unwrap();
        sender.send(result).unwrap();
    }
}
