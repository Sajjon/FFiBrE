use crate::prelude::*;

impl GatewayClient {
    async fn make_request<T, U, V, F, E>(
        &self,
        path: impl AsRef<str>,
        method: impl AsRef<str>,
        request: T,
        map: F,
    ) -> Result<V, NetworkError>
    where
        T: Serialize,
        U: for<'a> Deserialize<'a>,
        F: Fn(U) -> Result<V, E>,
        E: Into<NetworkError>,
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
            .and_then(|r| r.body.ok_or(RustSideError::ResponseBodyWasNil.into()))
            .and_then(|b| {
                if b.is_empty() {
                    Err(RustSideError::ResponseBodyWasNil.into())
                } else {
                    Ok(b)
                }
            })
            .and_then(|b| {
                serde_json::from_slice::<U>(&b).map_err(|_| {
                    RustSideError::UnableJSONDeserializeHTTPResponseBodyIntoTypeName {
                        type_name: std::any::type_name::<U>().to_owned(),
                    }
                    .into()
                })
            })
            .and_then(|s| map(s).map_err(|e| e.into()))
    }
}

impl GatewayClient {
    pub(crate) async fn post<T, U, V, F, E>(
        &self,
        path: impl AsRef<str>,
        request: T,
        map: F,
    ) -> Result<V, NetworkError>
    where
        T: Serialize,
        U: for<'a> Deserialize<'a>,
        F: Fn(U) -> Result<V, E>,
        E: Into<NetworkError>,
    {
        self.make_request(path, "POST", request, map).await
    }
}
