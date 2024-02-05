use crate::prelude::*;

#[derive(Record, Clone)]
pub struct Transaction {
    pub epoch: u32,
    pub round: u32,
    pub tx_id: String,
    pub fee_paid: String,
}

impl From<TransactionStreamItem> for Transaction {
    fn from(value: TransactionStreamItem) -> Self {
        Self {
            epoch: value.epoch,
            round: value.round,
            tx_id: value.intent_hash,
            fee_paid: value.fee_paid,
        }
    }
}

pub(crate) fn parse_transactions(
    response: GetTransactionStreamResponse,
) -> Result<Vec<Transaction>, RustSideError> {
    Ok(response.items.into_iter().map(|i| i.into()).collect())
}
