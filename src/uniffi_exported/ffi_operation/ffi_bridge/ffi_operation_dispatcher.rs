use crate::prelude::*;

#[derive(Object)]
pub struct FFIOperationDispatcher {
    /// Handler FFI side, receiving operations from us (Rust side),
    /// and passes result of the operation back to us (Rust side).
    pub handler: Arc<dyn FFIOperationHandler>,
}

impl FFIOperationDispatcher {
    pub fn new(handler: Arc<dyn FFIOperationHandler>) -> Self {
        Self { handler }
    }
}
