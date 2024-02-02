use crate::prelude::*;

#[derive(Object)]
pub struct GatewayClient {
    pub(crate) request_dispatcher: Arc<FFIOperationDispatcher>,
}

#[export]
impl GatewayClient {
    #[uniffi::constructor]
    pub fn new(network_antenna: Arc<dyn FFIOperationHandler>) -> Self {
        Self {
            request_dispatcher: FFIOperationDispatcher::new(network_antenna).into(),
        }
    }

    pub async fn get_xrd_balance_of_account(
        &self,
        address: String,
    ) -> Result<String, NetworkError> {
        self.post(
            "state/entity/details",
            GetEntityDetailsRequest::new(address),
            parse_xrd_balance_from,
        )
        .await
    }
}
