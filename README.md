# UniFFI "async" operations in FFI invoked from Rust

Showcase of FFI side (Swift side) - executing a Network request using `URLSession`'s [`dataTask:with:completionHandler`](https://developer.apple.com/documentation/foundation/urlsession/1407613-datatask) invoked from Rust, and letting Rust side deserialize JSON of the HTTP body and "massage the data" into an REST call result - which is exposed to FFI side (Swift side) as an async fn inside of Rust.

# Try

Run test:

```sh
cargo test
```

Which should output something like:

```sh
SWIFT ✅ getXrdBalanceOfAccount success, got balance: 890.8 ✅
test uniffi_foreign_language_testcase_test_swift ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.07s
```

# Rust side

```rust,no_run
#[uniffi::export(with_foreign)]
    fn execute_operation(
        &self,
        operation: FFIOperation,
        listener_rust_side: Arc<FFIDataResultListener>,
    ) -> Result<(), SwiftSideError>;
}
```

Where `FFIDataResultListener` is:

```rust,no_run
#[derive(Object)]
pub struct FFIDataResultListener {
    sender: Mutex<Option<Sender<FFIOperationResult>>>,
}

#[export]
impl FFIDataResultListener {
    fn notify_result(&self, result: FFIOperationResult) {
       self.sender.send(result) // Pseudocode
    }
}
```

Which allows us to build an async method e.g. a REST API endpoint, where JSON deserialization happens inside of Rust, and parsing of models into a result.

```rust,no_run
#[derive(Object)]
pub struct GatewayClient {
    pub(crate) request_dispatcher: Arc<FFIOperationDispatcher>,
}

impl GatewayClient {
    func make_request<T: Serialize, U: Deserialize>(
        request: T,
        url: String,
        method: String
    ) -> Result<U, Error> {

        let body = serde_json::to_vec(request)?;
        let network_request = NetworkRequest {
            url,
            body,
            method: ..
            headers: ..
        };

        let ffi_operation = FFIOperation::Networking { request: request };

        // Let Swift side make network request and await response
        let response = self.request_dispatcher.dispatch(ffi_operation).await?;

        serde_json::from_slice<U>(response)?;
    }

    pub async fn get_xrd_balance_of_account(
        &self,
        address: String,
    ) -> Result<String, NetworkError> {
        self.make_request(
            GetEntityDetailsRequest::new(address),
            "https://mainnet.radixdlt.com/state/entity/details",
            "POST",
        )
        .await
    }
}
```

# Swift Side

## Impl trait `FfiOperationHandler`

```swift
import Foundation
import network

// Convert `[Rust]NetworkRequest` to `[Swift]URLRequest`
extension NetworkRequest {
	func urlRequest(url: URL) -> URLRequest {
		var request = URLRequest(url: url)
		request.httpMethod = self.method
		request.httpBody = self.body
		request.allHTTPHeaderFields = self.headers
		return request
	}
}

// Turn `URLSession` into a "network antenna" for Rust
extension URLSession: FfiOperationHandler {
	public func executeOperation(
		operation rustOperation: FfiOperation,
		listenerRustSide: FfiDataResultListener
	) throws {
		guard
			case let .networking(rustRequest) = rustOperation,
			let url = URL(string: rustRequest.url)
		else {
			throw .error ...
		}
		dataTask(with: rustRequest.urlRequest(url: url)) { body, urlResponse, error in
			// Notify Rust with result
			listenerRustSide.notifyResult(
				{
					guard
						let httpResponse = urlResponse as? HTTPURLResponse,
						httpResponse.ok
					else {
						return .failure(error: ...)
					}
					return .success(value: body)
				}()
			)
		}.resume()
	}
}
```

Now ready to be used!

## Usage

```swift
let gatewayClient = GatewayClient(networkAntenna: URLSession.shared)
// Call async method in Rust land from Swift!
let balance = try await gatewayClient.getXrdBalanceOfAccount(
	address: "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease"
)
// Print result, if successful
print("SWIFT ✅ getXrdBalanceOfAccount success, got balance: \(balance) ✅")
```
