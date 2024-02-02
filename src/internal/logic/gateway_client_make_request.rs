use crate::prelude::*;

impl GatewayClient {
    async fn make_request<T, U, V, F>(
        &self,
        path: impl AsRef<str>,
        method: impl AsRef<str>,
        request: T,
        map: F,
    ) -> Result<V, NetworkError>
    where
        T: Serialize,
        U: for<'a> Deserialize<'a>,
        F: Fn(U) -> Result<V, NetworkError>,
    {
        let body = to_vec(&request).unwrap();
        let url = format!("https://mainnet.radixdlt.com/{}", path.as_ref());
        let request = NetworkRequest {
            url,
            body,
            method: method.as_ref().to_owned(),
            headers: HashMap::<String, String>::from_iter([(
                "Content-Type".to_owned(),
                "application/json".to_owned(),
            )]),
        };
        let response = self.http_client.make_request(request).await;
        response
            .and_then(|r| {
                serde_json::from_slice::<U>(&r.body).map_err(|_| {
                    NetworkError::UnableJSONDeserializeHTTPResponseBodyIntoTypeName {
                        type_name: std::any::type_name::<U>().to_owned(),
                    }
                })
            })
            .and_then(|s| map(s))
    }
}

impl GatewayClient {
    pub(crate) async fn post<T, U, V, F>(
        &self,
        path: impl AsRef<str>,
        request: T,
        map: F,
    ) -> Result<V, NetworkError>
    where
        T: Serialize,
        U: for<'a> Deserialize<'a>,
        F: Fn(U) -> Result<V, NetworkError>,
    {
        self.make_request(path, "POST", request, map).await
    }
}
