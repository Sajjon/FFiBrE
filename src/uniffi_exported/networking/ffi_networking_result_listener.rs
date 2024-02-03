use crate::prelude::*;

#[derive(Object)]
pub struct FFINetworkingResultListener {
    result_listener: FFIOperationResultListener,
}

impl FFINetworkingResultListener {
    pub fn with_result_listener(result_listener: FFIOperationResultListener) -> Self {
        Self { result_listener }
    }

    pub fn new(sender: Sender<FFIOperationResult>) -> Self {
        Self::with_result_listener(FFIOperationResultListener::new(sender))
    }
}

#[export]
impl FFINetworkingResultListener {
    /// This is called from FFI Side (Swift side), inside the implementation of
    /// an `execute_network_request:request:listener_rust_side` method on a [`FFIOperationHandler`],
    /// when the operation has finished, with the [`FFIOperationResult`].
    fn notify_result(&self, result: FFINetworkResult) {
        self.result_listener.notify_result(result.into())
    }
}
