import Foundation
import ffibre

/* Some bug in UniFFI not marking the `uniffi::Error` as `Swift.Error`... */
extension FfiNetworkingError: Swift.Error {}

extension NetworkResponse {
  init(data: Data, urlResponse: URLResponse) {
    guard let httpUrlResponse = urlResponse as? HTTPURLResponse else {
      fatalError("Expected URLResponse to always be HTTPURLResponse")
    }
    self.init(statusCode: UInt16(httpUrlResponse.statusCode), body: data)
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
    return .success(value: NetworkResponse(data: data, urlResponse: urlResponse))
  }
}

extension NetworkRequest {
  // Convert `[Rust]NetworkRequest` to `[Swift]URLRequest`
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

// Conform `[Swift]URLSession` to `[Rust]FfiNetworkingHandler`
extension URLSession: FfiNetworkingHandler {
  public func executeNetworkRequest(
    request rustRequest: NetworkRequest,
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

extension Async: FfiNetworkingHandler
where Request == NetworkRequest, Intermediary == (Data, URLResponse), Response == NetworkResponse {

  convenience init(
    call op: @escaping (URLRequest) async throws -> Intermediary
  ) {
    self.init(
      operation: { (rustRequest: NetworkRequest) in try await op(rustRequest.urlRequest()) },
      mapToResponse: { (data: Data, urlResponse: URLResponse) in
        NetworkResponse(data: data, urlResponse: urlResponse)
      }
    )
  }

  public func executeNetworkRequest(
    request rustRequest: NetworkRequest,
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

func test() async throws {
  print("🚀🛜  SWIFT 'test_networking' start")
  defer { print("🏁🛜  SWIFT 'test_networking' done") }

  let address = "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease"

  do {
    try await test_callback(address: address)
    try await test_async(address: address)
  } catch {
    print("🛜 ❌ SWIFT 'test_networking' error: \(String(describing: error))")
  }

}

try! await test()
