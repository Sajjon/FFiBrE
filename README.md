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

For each FFI interface you need to declare:

- Request - some operation we want the FFI side to execute using the `executor` (see below).
- Response (`Ok` value) - some value produced by the `executor` as a response to the `request`.
- Failure (`Error` value) - a type representing all kinds of failures that can happen FFI side during the `executor`s running of the request.
- Outcome (`Result<Response, Failure>`) - a result type aggregating both responses and failures, which the `executor` pass back to the `outcomeListener` using `notifyOutcome`.
- OutcomeListener - which has a single function the FFI side's executor (see below) should invoke, named `notifyOutcome`
- Executor - a (request, outcomeListener) receiver FFI side which is responsible for executing the `request` and sends back the outcome using the provided `outcomeListener`.
- Some async fn Rust side which builds the request, creates an `outcomeListener` and dispatches the `request` to the `executor` and awaits the `notifyOutcome` call on the `outcomeListener`, e.g. `async fn login_user`

# Networking demo

## Rust side

```rust,no_run
#[derive(Record)]
pub struct FFINetworkingRequest {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,

    pub body: Vec<u8>,
}

#[derive(Record)]
pub struct FFINetworkingResponse {
    pub status_code: u16,

    /// Can be empty.
    pub body: Vec<u8>,
}


#[derive(uniffi::Error, thiserror::Error)]
pub enum FFINetworkingError {
    #[error("Fail to create Swift 'URL''")]
    FailedToCreateURL,

    ...
}

#[derive(Enum)]
pub enum FFINetworkingOutcome {
    Success { value: FFINetworkingResponse },
    Failure { error: FFINetworkingError },
}
```

```rust,no_run
#[uniffi::export(with_foreign)]
pub trait FFINetworkingExecutor: FFIOperationExecutor<FFINetworkingOutcomeListener> {
    fn execute_networking_request(
        &self,
        request: FFINetworkingRequest,
        listener_rust_side: Arc<FFINetworkingOutcomeListener>,
    ) -> Result<(), FFISideError>;
}
```

Where `FFINetworkingOutcomeListener` is:

```rust,no_run
#[derive(Object)]
pub struct FFINetworkingOutcomeListener {
    result_listener: FFIOperationOutcomeListener<FFINetworkingOutcome>,
}

impl IsOutcomeListener for FFINetworkingOutcomeListener {
    type Request = FFINetworkingRequest;
    type Response = FFINetworkingResponse;
    type Failure = FFINetworkingError;
    type Outcome = FFINetworkingOutcome;
}

#[export]
impl FFINetworkingOutcomeListener {
    fn notify_outcome(&self, result: FFINetworkingOutcome) {
        self.result_listener.notify_outcome(result.into())
    }
}
```

Which allows us to build an async method e.g. a REST API endpoint, where JSON deserialization happens inside of Rust, and parsing of models into a result.

```rust,no_run
#[derive(Object)]
pub struct GatewayClient {
    pub(crate) networking_dispatcher: FFIOperationDispatcher<FFINetworkingOutcomeListener>,
}


impl GatewayClient {
    func make_request<T: Serialize, U: Deserialize>(
        request: T,
        url: String,
        method: String
    ) -> Result<U, Error> {

        let body = serde_json::to_vec(request)?;
        let request = FFINetworkingRequest {
            url,
            body,
            method: ..
            headers: ..
        };


        // Let Swift side make network request and await response
        let response = self.networking_dispatcher.dispatch(request).await?;

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

## Swift Side

## Internals

The `FFINetworkingExecutor` is used by a `FFIOperationDispatcher`.

```rust,no_run
pub struct FFIOperationDispatcher<L: IsOutcomeListener> {
    pub executor: Arc<dyn FFIOperationExecutor<L>>,
}

impl<L: IsOutcomeListener> FFIOperationDispatcher<L> {

    pub(crate) async fn dispatch(
        &self,
        operation: L::Request,
    ) -> Result<L::Response, FFIBridgeError> {
        // Underlying tokio channel used to get result from Swift back to Rust.
        let (sender, receiver) = channel::<L::Outcome>();

        // Our callback we pass to Swift
        let outcome_listener = FFIOperationOutcomeListener::new(sender);

        // Make request
        self.executor
            .execute_request(
                // Pass operation to Swift to make
                operation,
                // Pass callback, Swift will call `outcome_listener.notify_outcome`
                outcome_listener.into(),
            )
            .map_err(|e| FFIBridgeError::from(e))?;

        // Await response from Swift
        let response = receiver.await.map_err(|_| FFIBridgeError::FromRust {
            error: RustSideError::FailedToReceiveResponseFromSwift,
        })?;

        response.into().map_err(|e| e.into().into())
    }
}

```

# Swift Side

Translate FFINetworkingRequest -> `URLRequest`

```swift
import Foundation
import ffibre

// Convert `[Rust]FFINetworkingRequest` to `[Swift]URLRequest`
extension FFINetworkingRequest {
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
extension URLSession: FfiNetworkingExecutor {
	public func executeNetworkingRequest(
		request rustRequest: FfiNetworkingRequest,
		listenerRustSide: FfiNetworkingOutcomeListener
	) throws {
		guard let url = URL(string: rustRequest.url) else {
			throw FfiNetworkingError.failedToCreateUrlFrom(string: rustRequest.url)
		}
		let task = dataTask(with: rustRequest.urlRequest(url: url)) { data, urlResponse, error in
			let result = FfiNetworkingOutcome.with(
				data: data,
				urlResponse: urlResponse,
				error: error
			)
			listenerRustSide.notifyOutcome(result: result)
		}
		task.resume()
	}
}

```

Now ready to be used!

## Usage

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
public final class Async<Request, Intermediary, Response> {
    typealias Operation = (Request) async throws -> Intermediary
    typealias MapToResponse = (Intermediary) async throws -> Response

	private var task: Task<Void, Never>?

	let operation: Operation
	let mapToData: MapToData
}

extension Async: FfiNetworkingExecutor
where
  Request == FfiNetworkingRequest, Intermediary == (Data, URLResponse),
  Response == FfiNetworkingResponse
{
	public func executeNetworkingRequest(
		request rustRequest: FfiNetworkingRequest,
		listenerRustSide: FfiNetworkingOutcomeListener
	) throws {
		self.task = Task {
			do {
				let result = try await self.operation(rustRequest)
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
    networkAntenna: Async {
      try await urlSession.data(for: $0.asFFINetworkingRequest.urlRequest()).0
    }
)
let balance = try await gatewayClient.getXrdBalanceOfAccount(address: "account_rdx...")
print("SWIFT ✅ getXrdBalanceOfAccount success, got balance: \(balance) ✅")
// 🎉
```
