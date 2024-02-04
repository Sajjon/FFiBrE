use crate::prelude::*;

#[derive(Deserialize, Clone)]
pub struct FungibleResourceItem {
    pub(crate) amount: String,
    pub(crate) resource_address: String,
}

#[derive(Deserialize, Clone)]
pub struct FungibleResources {
    pub(crate) items: Vec<FungibleResourceItem>,
}

#[derive(Deserialize, Clone)]
pub struct EntityDetailsItem {
    pub(crate) fungible_resources: FungibleResources,
}

/// The response a call to the REST Endpoint:
/// `https://mainnet.radixdlt.com/state/entity/details`
///
/// Which contains token balances of an account.
#[derive(Deserialize, Clone)]
pub struct GetEntityDetailsResponse {
    pub(crate) items: Vec<EntityDetailsItem>,
}
