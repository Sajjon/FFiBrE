use crate::prelude::*;

#[derive(Record, Clone, Debug)]
pub struct NetworkRequest {
    pub url: String,
    pub body: Vec<u8>,
    pub method: String,
    pub headers: HashMap<String, String>,
}
