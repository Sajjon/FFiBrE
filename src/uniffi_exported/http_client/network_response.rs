use crate::prelude::*;

#[derive(Record, Clone, Debug)]
pub struct NetworkResponse {
    pub(crate) status_code: u16,
    pub(crate) body: Option<Vec<u8>>,
}
