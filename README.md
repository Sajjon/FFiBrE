# UniFFI "async" operations in FFI invoked from Rust

Showcase of FFI side (Swift side) - executing a Network request called from Rust, implemented with [tokio::oneshot](https://docs.rs/tokio/latest/tokio/sync/oneshot/fn.channel.html).

Swift is using `URLSession`'s [`dataTask:with:completionHandler`](https://developer.apple.com/documentation/foundation/urlsession/1407613-datatask) invoked from Rust, and letting Rust side deserialize JSON of the HTTP body and "massage the data" into an REST call result - which is exposed to FFI side (Swift side) as an async fn inside of Rust.

# Try

Run test:

```sh
cargo test
```

Which should output something like:

```sh
running 1 test
    Finished dev [unoptimized + debuginfo] target(s) in 0.21s
SWIFT ✅ completionCallbackBased balance: 890.88637929049 ✅
SWIFT ✅ clientAsyncBased balance: 890.88637929049 ✅
test uniffi_foreign_language_testcase_test_swift ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.48s

```

# Rust side

```rust,no_run
/// A handler on the FFI side, which receives request from Rust, executes them
/// and notifies Rust with the result of the FFI operation.
#[uniffi::export(with_foreign)]
pub trait FFIOperationHandler: Send + Sync {
    fn execute_operation(
        &self,
        operation: FFIOperation,
        listener_rust_side: Arc<FFIOperationResultListener>,
    ) -> Result<(), SwiftSideError>;
}
```

Where `FFIOperationResultListener` is:

```rust,no_run
#[derive(Object)]
pub struct FFIOperationResultListener {
    sender: Mutex<Option<tokio::oneshot::Sender<FFIOperationResult>>>,
}

#[export]
impl FFIOperationResultListener {
    fn notify_result(&self, result: FFIOperationResult) {
       self.sender.send(result) // Pseudocode
    }
}
```

The `FFIOperationHandler` is used by a `FFIOperationDispatcher`.

```rust,no_run
#[derive(Object)]
pub struct FFIOperationDispatcher {
    /// Handler FFI side, receiving operations from us (Rust side),
    /// and passes result of the operation back to us (Rust side).
    pub handler: Arc<dyn FFIOperationHandler>,
}

impl FFIOperationDispatcher {
    pub(crate) async fn dispatch(
        &self,
        operation: FFIOperation,
    ) -> Result<Option<Vec<u8>>, NetworkError> {
        let (sender, receiver) = tokio::oneshot::channel::<FFIOperationResult>();
        let result_listener = FFIOperationResultListener::new(sender);

        // Make request
        self.handler
            .execute_operation(
                // Pass operation to Swift to make
                operation,
                // Pass callback, Swift will call `result_listener.notify_result`
                result_listener.into(),
            )
            .map_err(|e| NetworkError::from(e))?;

        // Await response from Swift
        let response: FFIOperationResult = receiver.await?;

        // Map response from Swift -> Result<Option<Vec<u8>>, NetworkError>,
        // keeping any errors happening in Swift intact.
        Result::<Option<Vec<u8>>, SwiftSideError>::from(response).map_err(|e| e.into())
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

Translate NetworkRequest -> `URLRequest`

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

```

## Completion Handler Callback based

```swift

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

### Usage

```swift
let gatewayClient = GatewayClient(networkAntenna: URLSession.shared)
// Call async method in Rust land from Swift!
let balance = try await gatewayClient.getXrdBalanceOfAccount(
	address: "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease"
)
// Print result, if successful
print("SWIFT ✅ getXrdBalanceOfAccount success, got balance: \(balance) ✅")
```

## Async based

But it gets better! We can perform an async call in a Swift `Task` and let a holder of it implement the `FfiOperationHandler` trait!

```swift
public final class AsyncOperation<T> {
	typealias Operation = (FfiOperation) async throws -> T
	typealias MapToData = (T) async throws -> Data

	private var task: Task<Void, Never>?

	let operation: Operation
	let mapToData: MapToData
}

extension AsyncOperation where T == Data {
	convenience init(
		operation: @escaping Operation
	) {
		self.init(operation: operation) { $0 }
	}

}

extension AsyncOperation: FfiOperationHandler {

	public func executeOperation(
		operation rustOperation: FfiOperation,
		listenerRustSide: FfiDataResultListener
	) throws {
		self.task = Task {
			do {
				let result = try await self.operation(rustOperation)
				let data = try await self.mapToData(result)
				listenerRustSide.notifyResult(result: .success(value: data))
			} catch {
				listenerRustSide.notifyResult(
					result: .failure(
						error: ...
					))
			}
		}
	}
}
```

Now ready to be used!

### Usage

```swift
let gatewayClient = GatewayClient(
    networkAntenna: AsyncOperation {
      try await urlSession.data(for: $0.asNetworkRequest.urlRequest()).0
    }
)
balance = try await gatewayClient.getXrdBalanceOfAccount(address: "...")
```

🎉
