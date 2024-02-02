use crate::prelude::*;

#[uniffi::export(with_foreign)]
pub trait DeviceNetworkAntenna: Send + Sync {
    fn make_request(
        &self,
        request: NetworkRequest,
        listener_rust_side: Arc<NetworkResultListener>,
    ) -> Result<(), SwiftSideError>;
}
