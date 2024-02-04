# FFiBre - FFI BridgE - async bridge Swift<>Rust PoC

FFiBRe pronounced "fibre" is a **Proof-of-Concept** of bridging between Swift and Rust for async methods.

I showcase how we can bridge certain operation from Rust to FFI side (Swift side) and read the outcome of these operations in Rust - using callback pattern.

The implementation uses [tokio::oneshot::channel](https://docs.rs/tokio/latest/tokio/sync/oneshot/fn.channel.html) for the callback.

This repo contains three examples:

- Networking
- File IO Read
- File IO Write

All examples have two versions:

- Callback based
- Async wrapped (translated to callback)

# Test

Run test:

```sh
cargo test
```

Which should output something like:

```sh
🚀🗂️  SWIFT 'test_file_io' start
✅🗂️  writeToNewOrExtendExistingFile CB outcome: didWrite(alreadyExisted: false)
✅🗂️  writeToNewOrExtendExistingFile ASYNC outcome: didWrite(alreadyExisted: true)
✅🗂️  writeToNewOrExtendExistingFile CB outcome: didWrite(alreadyExisted: true)
🏁🗂️  SWIFT 'test_file_io' done

🚀🛜  SWIFT 'test_networking' start
🛜 ✅ SWIFT CB balance: 890.88637929049
🛜 ✅ SWIFT ASYNC balance: 890.88637929049
🏁🛜  SWIFT 'test_networking' done
```

# Design

For each FFI operation you need to declare a `Executor`

# Rust side

```rust,no_run
/// A handler on the FFI side, which receives request from Rust, executes them
/// and notifies Rust with the result of the FFI operation.
#[uniffi::export(with_foreign)]
pub trait FFIOperationExecutor: Send + Sync {
    fn execute_request(
        &self,
        operation: FFIOperation,
        listener_rust_side: Arc<FFIOperationOutcomeListener>,
    ) -> Result<(), FFISideError>;
}
```

Where `FFIOperationOutcomeListener` is:

```rust,no_run
#[derive(Object)]
pub struct FFIOperationOutcomeListener {
    sender: Mutex<Option<tokio::oneshot::Sender<FFIOperationResult>>>,
}

#[export]
impl FFIOperationOutcomeListener {
    fn notify_outcome(&self, result: FFIOperationResult) {
       self.sender.send(result) // Pseudocode
    }
}
```

The `FFIOperationExecutor` is used by a `FFIOperationDispatcher`.

```rust,no_run
#[derive(Object)]
pub struct FFIOperationDispatcher {
    /// Handler FFI side, receiving operations from us (Rust side),
    /// and passes result of the operation back to us (Rust side).
    pub handler: Arc<dyn FFIOperationExecutor>,
}

impl FFIOperationDispatcher {
    pub(crate) async fn dispatch(
        &self,
        operation: FFIOperation,
    ) -> Result<Option<Vec<u8>>, FFIBridgeError> {
        let (sender, receiver) = tokio::oneshot::channel::<FFIOperationResult>();
        let result_listener = FFIOperationOutcomeListener::new(sender);

        // Make request
        self.handler
            .execute_request(
                // Pass operation to Swift to make
                operation,
                // Pass callback, Swift will call `result_listener.notify_outcome`
                result_listener.into(),
            )
            .map_err(|e| FFIBridgeError::from(e))?;

        // Await response from Swift
        let response: FFIOperationResult = receiver.await?;

        // Map response from Swift -> Result<Option<Vec<u8>>, FFIBridgeError>,
        // keeping any errors happening in Swift intact.
        Result::<Option<Vec<u8>>, FFISideError>::from(response).map_err(|e| e.into())
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
    ) -> Result<String, FFIBridgeError> {
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
import ffibre

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
extension URLSession: FfiOperationExecutor {
	public func executeOperation(
		operation rustOperation: FfiOperation,
		listenerRustSide: FfiDataOutcomeListener
	) throws {
		guard
			case let .networking(rustRequest) = rustOperation,
			let url = URL(string: rustRequest.url)
		else {
			throw .error ...
		}
		dataTask(with: rustRequest.urlRequest(url: url)) { body, urlResponse, error in
			// Notify Rust with result
			listenerRustSide.notifyOutcome(
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
	address: "account_rdx..."
)
print("SWIFT ✅ getXrdBalanceOfAccount success, got balance: \(balance) ✅")
```

## Async based

But it gets better! We can perform an async call in a Swift `Task` and let a holder of it implement the `FfiOperationExecutor` trait!

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

extension AsyncOperation: FfiOperationExecutor {
	public func executeOperation(
		operation rustOperation: FfiOperation,
		listenerRustSide: FfiDataOutcomeListener
	) throws {
		self.task = Task {
			do {
				let result = try await self.operation(rustOperation)
				let data = try await self.mapToData(result)
				listenerRustSide.notifyOutcome(result: .success(value: data))
			} catch {
				listenerRustSide.notifyOutcome(result: .failure(error: ...))
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
let balance = try await gatewayClient.getXrdBalanceOfAccount(address: "account_rdx...")
print("SWIFT ✅ getXrdBalanceOfAccount success, got balance: \(balance) ✅")
// 🎉
```
