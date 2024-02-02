use crate::prelude::*;

#[derive(Object)]
pub struct GatewayClient {
    pub(crate) http_client: Arc<HTTPClient>,
}

#[export]
impl GatewayClient {
    #[uniffi::constructor]
    pub fn new(network_antenna: Arc<dyn DeviceNetworkAntenna>) -> Self {
        Self {
            http_client: HTTPClient::new(network_antenna).into(),
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
