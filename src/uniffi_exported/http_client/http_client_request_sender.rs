use crate::prelude::*;

#[export]
pub trait NotifyRustFromSwift: Send + Sync {
    fn response(&self, result: NetworkResult);
}

#[uniffi::export(with_foreign)]
pub trait HTTPClientRequestSender: Send + Sync {
    fn send(
        &self,
        request: NetworkRequest,
        response_back: Arc<dyn NotifyRustFromSwift>,
    ) -> Result<(), SwiftSideError>;
}
