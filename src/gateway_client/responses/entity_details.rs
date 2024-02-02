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
pub struct EntityStateItem {
    pub(crate) fungible_resources: FungibleResources,
}

#[derive(Deserialize, Clone)]
pub struct EntityState {
    pub(crate) items: Vec<EntityStateItem>,
}
