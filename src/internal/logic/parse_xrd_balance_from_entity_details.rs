use crate::prelude::*;

const XRD: &str = "resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd";

pub(crate) fn parse_xrd_balance_from(entity_state: EntityState) -> Result<String, NetworkError> {
    assert_eq!(entity_state.items.len(), 1);
    let item: &EntityStateItem = entity_state.items.first().unwrap();
    let fungible_resources = item.fungible_resources.clone();

    fungible_resources
        .items
        .into_iter()
        .filter(|x| x.resource_address == XRD)
        .map(|x| x.amount.clone())
        .next()
        .ok_or(NetworkError::NoXRDBalanceFound)
}
