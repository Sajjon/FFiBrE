use crate::prelude::*;

#[derive(Object)]
pub struct HTTPClient {
    pub network_antenna: Arc<dyn DeviceNetworkAntenna>,
}

impl HTTPClient {
    pub fn new(network_antenna: Arc<dyn DeviceNetworkAntenna>) -> Self {
        Self { network_antenna }
    }
}
