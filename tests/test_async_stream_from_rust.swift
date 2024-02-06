import Foundation
import ffibre

/*
  In this file we test building a Swift Async Sequence using a thread spawned
  inside of Rust with tokio.

  TL;DR This is a bad idea - at least in its current form - because it is very 
  complex and requires DOUBLE sided cancellation listeners. Rust must listen
  to cancellation from Swift and Swift must listen to cancellation from Rust.

  See the `test_async_stream_from_swift` example which is much more simple and
  the recommended way to go.
*/

/* Some bug in UniFFI not marking the `uniffi::Error` as `Swift.Error`... */
extension FfiNetworkingError: Swift.Error {}

extension FfiNetworkingResponse {
  init(data: Data, urlResponse: URLResponse) {
    guard let httpUrlResponse = urlResponse as? HTTPURLResponse else {
      fatalError("Expected URLResponse to always be HTTPURLResponse")
    }
    self.init(statusCode: UInt16(httpUrlResponse.statusCode), body: data)
  }
}

extension Transaction: CustomStringConvertible {
  public var description: String {
    """
    Transaction(
      epoch: \(self.epoch),
      round: \(self.round),
      txID: \(self.txId),
      fee: \(self.feePaid)
    )
    """
  }
}

extension FfiNetworkingOutcome {

  static func fail(error: Swift.Error, data: Data? = nil, urlResponse: URLResponse? = nil) -> Self {
    func message() -> String? {
      data.map { String(data: $0, encoding: .utf8) } ?? nil
    }
    func statusCode() -> UInt16? {
      urlResponse.map { $0 as? HTTPURLResponse ?? nil }?.map { UInt16($0.statusCode) } ?? nil
    }

    return .failure(
      error: .requestFailed(
        statusCode: statusCode(),
        urlSessionUnderlyingError: String(describing: error),
        errorMessageFromGateway: message()
      )
    )

  }

  static func with(
    data: Data?,
    urlResponse: URLResponse?,
    error: Swift.Error?
  ) -> Self {
    if let error {
      return .fail(error: error, data: data, urlResponse: urlResponse)
    }
    guard let data else { fatalError("If error is nil data SHOULD be present if error is nil.") }
    guard let urlResponse else {
      fatalError("Expected URLResponse to always be present if error is nil and data is some.")
    }
    return .success(value: FfiNetworkingResponse(data: data, urlResponse: urlResponse))
  }
}

extension FfiNetworkingRequest {
  // Convert `[Rust]FfiNetworkingRequest` to `[Swift]URLRequest`
  func urlRequest(url: URL) -> URLRequest {
    var request = URLRequest(url: url)
    request.httpMethod = self.method
    request.httpBody = self.body
    request.allHTTPHeaderFields = self.headers
    return request
  }

  func urlRequest() throws -> URLRequest {
    guard let url = URL(string: self.url) else {
      throw FfiNetworkingError.failedToCreateUrlFrom(string: self.url)
    }
    return self.urlRequest(url: url)
  }
}

// Conform `[Swift]URLSession` to `[Rust]FfiNetworkingExecutor`
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

final class AsyncSubject<T> {
  private let continuation: AsyncStream<T>.Continuation
  private let stream: AsyncStream<T>
  private var rustSideCancellationListener: CancellationListener?
  private init() {
    let (stream, continuation) = AsyncStream<T>.makeStream()
    self.stream = stream
    self.continuation = continuation
  }
  static func start(
    operation: @escaping (AsyncSubject<T>) async throws -> Void
  ) -> (stream: AsyncStream<T>, cancel: () -> Void) {
    let subject = AsyncSubject<T>()
    let task = Task {
      // Non blocking, non returning loop
      try await operation(subject)
    }
    subject.continuation.onTermination = { termination in
      print("❌ SWIFT subject.continuation.onTermination: \(termination)")
      task.cancel()
      subject.rustSideCancellationListener?.notifyCancelled()
    }
    return (subject.stream, subject.continuation.finish)
  }
}

extension GatewayClient {
  func txStream() -> (stream: AsyncStream<Transaction>, cancel: () -> Void) {
    AsyncSubject<Transaction>.start {
      await self.subscribeStreamOfLatestTransactions(
        publisher: $0 as IsTransactionPublisher
      )
    }
  }
}

extension AsyncSubject<Transaction>: IsTransactionPublisher {
  func onValue(value: Transaction) {
    self.continuation.yield(value)
  }
  func finishedFromRustSide() {
    print("❌ SWIFT received finishedFromRustSide")
    self.continuation.finish()
  }
  func rustIsSubscribedNotifyCancellationOn(listener: CancellationListener) {
    print("🌱 SWIFT rustIsSubscribedNotifyCancellationOn got listener")
    self.rustSideCancellationListener = listener
  }
}

func test_async_stream() async throws {

  let gatewayClient = GatewayClient(
    networkAntenna: URLSession.shared
  )

  let t = Task {
    let (stream, cancel) = gatewayClient.txStream()
    for await tx in stream.prefix(3) {
      print("🚀🛜  ❤️ SWIFT FOO async value from stream: \(tx)")

      print("SWIFT ✨ cancelling FOO task")
      cancel()
    }
  }
  let u = Task {
    let (stream, _) = gatewayClient.txStream()
    for await tx in stream.prefix(3) {
      print("🚀🛜  💚SWIFT BAR async value from stream: \(tx)")
    }
  }
  let _ = await [t.value, u.value]
}

func test() async throws {
  print("🚀🛜  SWIFT 'test_test_async_stream' start")
  defer { print("🏁🛜  SWIFT 'test_test_async_stream' done") }

  do {
    try await test_async_stream()
  } catch {
    print("🛜 ❌ SWIFT 'test_async_stream' error: \(String(describing: error))")
  }

}

try! await test()
