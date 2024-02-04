use crate::prelude::*;

#[derive(Serialize)]
pub struct GetEntityDetailsRequest {
    pub(crate) addresses: Vec<String>,
}

impl GetEntityDetailsRequest {
    pub(crate) fn new(address: impl AsRef<str>) -> Self {
        Self {
            addresses: vec![address.as_ref().to_owned()],
        }
    }
}
