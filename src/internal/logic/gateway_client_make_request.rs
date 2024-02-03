use crate::prelude::*;

impl GatewayClient {
    fn model_from_response<U>(&self, response: NetworkResponse) -> Result<U, RustSideError>
    where
        U: for<'a> Deserialize<'a>,
    {
        if let 200..=299 = response.status_code {
            // all good
        } else {
            return Err(RustSideError::BadResponseCode);
        }

        let body = response.body;
        if body.is_empty() {
            return Err(RustSideError::ResponseBodyWasNil.into());
        }

        serde_json::from_slice::<U>(&body).map_err(|_| {
            RustSideError::UnableJSONDeserializeHTTPResponseBodyIntoTypeName {
                type_name: std::any::type_name::<U>().to_owned(),
            }
        })
    }

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
        // JSON serialize request into body bytes
        let body = to_vec(&request).unwrap();

        // Append relative path to base url
        let url = format!("https://mainnet.radixdlt.com/{}", path.as_ref());

        // Create Network request object, which will be translated by
        // Swift side into a `[Swift]URLRequest`
        let request = NetworkRequest {
            url,
            body,
            method: method.as_ref().to_owned(),
            headers: HashMap::<String, String>::from_iter([(
                "Content-Type".to_owned(),
                "application/json".to_owned(),
            )]),
        };

        // Let Swift side make network request and await response
        let response = self
            .network_dispatcher
            .dispatch_network_request(request)
            .await?;

        // Read out HTTP body from response and JSON parse it into U
        let model = self
            .model_from_response(response)
            .map_err(|error| NetworkError::FromRust { error })?;

        // Map U -> V
        map(model).map_err(|e| e.into())
    }
}

impl GatewayClient {
    /// Makes a HTTP POST request using `http_client`, which in turn uses
    /// DeviceNetworkAntenna "installed" from Swift.
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
