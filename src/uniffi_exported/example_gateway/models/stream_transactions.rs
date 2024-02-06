use crate::prelude::*;

#[derive(Serialize)]
pub struct GetTransactionStreamRequest {
    pub(crate) limit_per_page: u16,
}

impl Default for GetTransactionStreamRequest {
    fn default() -> Self {
        Self { limit_per_page: 5 }
    }
}

//
// RESPONSE
//

#[derive(Deserialize, Clone)]
pub struct TransactionStreamItem {
    pub epoch: u32,
    pub round: u32,
    pub intent_hash: String,
    pub fee_paid: String,
}

/// The response a call to the REST Endpoint:
/// `https://mainnet.radixdlt.com/state/entity/details`
///
/// Which contains token balances of an account.
#[derive(Deserialize, Clone)]
pub struct GetTransactionStreamResponse {
    pub(crate) items: Vec<TransactionStreamItem>,
}
