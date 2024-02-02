use crate::prelude::*;

#[derive(Object)]
pub struct GatewayClient {
    pub(crate) http_client: Arc<HTTPClient>,
}

#[export]
impl GatewayClient {
    #[uniffi::constructor]
    pub fn new(http_client: Arc<HTTPClient>) -> Self {
        Self { http_client }
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
