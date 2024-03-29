use crate::prelude::*;

const XRD: &str = "resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd";

pub(crate) fn parse_xrd_balance_from(
    entity_state: GetEntityDetailsResponse,
) -> Result<String, RustSideError> {
    assert_eq!(entity_state.items.len(), 1);
    let item: &EntityDetailsItem = entity_state.items.first().unwrap();
    let fungible_resources = item.fungible_resources.clone();

    fungible_resources
        .items
        .into_iter()
        .filter(|x| x.resource_address == XRD)
        .map(|x| x.amount.clone())
        .next()
        .ok_or(RustSideError::NoXRDBalanceFound)
}
