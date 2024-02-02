use crate::prelude::*;

#[derive(Record, Clone, Debug, PartialEq, Eq)]
pub struct NetworkRequest {
    pub url: String,
    pub body: Vec<u8>,
    pub method: String,
    pub headers: HashMap<String, String>,
}
