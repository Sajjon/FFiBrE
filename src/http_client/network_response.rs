use crate::prelude::*;

#[derive(Record, Clone, Debug)]
pub struct NetworkResponse {
    pub(crate) response_code: u16,
    pub(crate) body: Vec<u8>,
}
