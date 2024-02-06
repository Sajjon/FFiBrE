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

extension AsyncStream where Element: Equatable {

  static func new(
    label: String,
    nextElement: @escaping () async throws -> Element
  ) -> (
    stream: Self, cancel: () -> Void
  ) {
    var cancel: (() -> Void)!
    let stream = AsyncStream<Element> { (continuation: AsyncStream<Element>.Continuation) in
      let task = Task {
        var last: Element?
        while !Task.isCancelled {
          try Task.checkCancellation()
          let value = try await nextElement()
          if value != last {
            continuation.yield(value)
          } else {
            print("SWIFT 🐌 \(label) duplicate ignored")
          }
          last = value
          try await Task.sleep(for: .seconds(7))
        }
        continuation.finish()
      }
      cancel = {
        task.cancel()
      }
      continuation.onTermination = { termination in
        task.cancel()
      }
    }
    return (stream, cancel)
  }
}

extension GatewayClient {
  func txStream(label: String) -> (stream: AsyncStream<Transaction>, cancel: () -> Void) {
    AsyncStream.new(label: label) { [unowned self] in await self.getLatestTransactionsOrPanic() }
  }
}

func test_async_stream() async throws {

  let gatewayClient = GatewayClient(
    networkAntenna: URLSession.shared
  )

  await withDiscardingTaskGroup { taskGroup in
    taskGroup.addTask {
      let label = "FOO"
      let (stream, cancel) = gatewayClient.txStream(label: label)
      for await tx in stream.prefix(3) {
        print("🚀🛜  💜 SWIFT \(label) async value from stream: \(tx)")

        print("🚀🛜  💜 SWIFT ✨ cancelling \(label) task (and breaking...)")
        cancel();break
      }
    }

    taskGroup.addTask {
      let label = "BAR"
      let (stream, _) = gatewayClient.txStream(label: label)
      for await tx in stream.prefix(3) {
        print("🚀🛜  💚SWIFT \(label) async value from stream: \(tx)")
      }
    }
  }
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
