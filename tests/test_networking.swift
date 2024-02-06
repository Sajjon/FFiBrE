import Foundation
import ffibre

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

public final class Async<Request, Intermediary, Response> {
  typealias Operation = (Request) async throws -> Intermediary
  typealias MapToResponse = (Intermediary) async throws -> Response

  private var task: Task<Void, Never>?

  let operation: Operation
  let mapToResponse: MapToResponse

  init(
    operation: @escaping Operation,
    mapToResponse: @escaping MapToResponse
  ) {
    self.operation = operation
    self.mapToResponse = mapToResponse
  }
}

extension Async: FfiNetworkingExecutor
where
  Request == FfiNetworkingRequest, Intermediary == (Data, URLResponse),
  Response == FfiNetworkingResponse
{

  convenience init(
    call op: @escaping (URLRequest) async throws -> Intermediary
  ) {
    self.init(
      operation: { (rustRequest: FfiNetworkingRequest) in try await op(rustRequest.urlRequest()) },
      mapToResponse: { (data: Data, urlResponse: URLResponse) in
        FfiNetworkingResponse(data: data, urlResponse: urlResponse)
      }
    )
  }

  public func executeNetworkingRequest(
    request rustRequest: FfiNetworkingRequest,
    listenerRustSide: FfiNetworkingOutcomeListener
  ) throws {
    self.task = Task {
      do {
        let intermediary = try await self.operation(rustRequest)
        let response = try await self.mapToResponse(intermediary)
        listenerRustSide.notifyOutcome(result: .success(value: response))
      } catch {
        listenerRustSide.notifyOutcome(result: .fail(error: error))
      }
    }
  }
}

func test_callback(address: String) async throws {
  let gatewayClient = GatewayClient(
    networkAntenna: URLSession.shared
  )

  let balance = try await gatewayClient.getXrdBalanceOfAccount(address: address)
  print("🛜 ✅ SWIFT CB balance: \(balance)")
}

func test_async(address: String) async throws {
  let gatewayClient = GatewayClient(
    networkAntenna: Async(call: URLSession.shared.data(for:))
  )

  let balance = try await gatewayClient.getXrdBalanceOfAccount(address: address)
  print("🛜 ✅ SWIFT ASYNC balance: \(balance)")
}

func test_balance() async throws {
  let address = "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease"
  try await test_callback(address: address)
  try await test_async(address: address)
}

func test_latest_tx() async throws {
  let gatewayClient = GatewayClient(
    networkAntenna: Async(call: URLSession.shared.data(for:))
  )
  let transactions = try await gatewayClient.getLatestTransactions()
  let transactionsDescription = transactions.map { String(describing: $0) }.joined(separator: ", ")
  print("🛜 ✅ SWIFT ASYNC latest transactions: \(transactionsDescription)")
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
      print("❌ SWIFT subject task operation returned")
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
      try await self.subscribeStreamOfLatestTransactions(
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
    print("❌ SWIFT finishedFromRustSide")
    self.continuation.finish()
  }
  func rustIsSubscribedNotifyCancellationOn(listener: CancellationListener) {
    print("🌱 SWIFT rustIsSubscribedNotifyCancellationOn got listener")
    self.rustSideCancellationListener = listener
  }
}

func test_async_stream() async throws {
  print("🚀🛜  SWIFT 'test_test_async_stream' start")
  defer { print("🏁🛜  SWIFT 'test_test_async_stream' done") }

  let gatewayClient = GatewayClient(
    networkAntenna: Async(call: URLSession.shared.data(for:))
  )

  let t = Task {
    let (stream, cancel) = gatewayClient.txStream()
    for await tx in stream.prefix(3) {
      print(
        "🚀🛜  ❤️ SWIFT PING async value from stream: \(tx) CANCELLING")
      cancel()
    }
  }
  let u = Task {
    let (stream, _) = gatewayClient.txStream()
    for await tx in stream.prefix(3) {
      print("🚀🛜  💚SWIFT PONG async value from stream: \(tx)")
    }
  }
  let txs = try await [t.value, u.value]
}

func test() async throws {
  print("🚀🛜  SWIFT 'test_networking' start")
  defer { print("🏁🛜  SWIFT 'test_networking' done") }

  do {
    try await test_balance()
  } catch {
    print("🛜 ❌ SWIFT 'test_networking - test_balance' error: \(String(describing: error))")
  }

  do {
    try await test_latest_tx()
  } catch {
    print("🛜 ❌ SWIFT 'test_networking - test_tx_stream' error: \(String(describing: error))")
  }

  do {
    try await test_async_stream()
  } catch {
    print("🛜 ❌ SWIFT 'test_networking - test_async_stream' error: \(String(describing: error))")
  }

}

try! await test()
