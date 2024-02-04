use crate::prelude::*;

pub trait FFIOperationExecutor<L: IsOutcomeListener>: Send + Sync {
    fn execute_request(
        &self,
        request: L::Request,
        listener_rust_side: L,
    ) -> Result<(), FFISideError>;
}
