use crate::prelude::*;

/// A [Radix][https://www.radixdlt.com/] Gateway REST client, that makes its
/// network request using a "network antenna" 'installed' from FFI Side (Swift side).
#[derive(Object)]
pub struct GatewayClient {
    pub(crate) networking_dispatcher: FFIOperationDispatcher<FFINetworkingResultListener>,
}

#[export]
impl GatewayClient {
    /// Constructs a new [`GatewayClient`] using a "network antenna" - a type
    /// implementing [`FFIOperationHandler`] on the FFI side (Swift side), e.g.
    /// `[Swift]URLSession` which wraps the execution of a network call.
    #[uniffi::constructor]
    pub fn new(network_antenna: Arc<dyn FFINetworkingHandler>) -> Self {
        Self {
            networking_dispatcher: FFIOperationDispatcher::<FFINetworkingResultListener>::new(
                network_antenna,
            ),
        }
    }

    /// Reads the XRD balance of a Radix account with `[address]`, the actual
    /// network call is being done FFI Side (Swift side), but the parsing of JSON
    /// into models, and mapping of models [`GetEntityDetailsResponse`] ->
    /// balance (String).
    pub async fn get_xrd_balance_of_account(
        &self,
        address: String,
    ) -> Result<String, FFIBridgeError> {
        self.post(
            "state/entity/details",
            GetEntityDetailsRequest::new(address),
            parse_xrd_balance_from,
        )
        .await
    }
}
