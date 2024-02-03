import Foundation
import network

extension NetworkResponse {
  init(data: Data, urlResponse: URLResponse) {
    guard let httpUrlResponse = urlResponse as? HTTPURLResponse else {
      fatalError("Expected URLResponse to always be HTTPURLResponse")
    }
    self.init(statusCode: UInt16(httpUrlResponse.statusCode), body: data)
  }
}

extension FfiNetworkResult {

  static func fail(error: Swift.Error, data: Data? = nil, urlResponse: URLResponse? = nil) -> Self {
    func message() -> String? {
      data.map { String(data: $0, encoding: .utf8) } ?? nil
    }
    func statusCode() -> UInt16? {
      urlResponse.map { $0 as? HTTPURLResponse ?? nil }?.map { UInt16($0.statusCode) } ?? nil
    }

    return .failure(
      error: .RequestFailed(
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
      throw SwiftSideError.FailedToCreateUrlFrom(string: self.url)
    }
    return self.urlRequest(url: url)
  }
}

// Conform `[Swift]URLSession` to `[Rust]FfiNetworkingHandler`
extension URLSession: FfiNetworkingHandler {
  public func executeNetworkRequest(
    request rustRequest: NetworkRequest,
    listenerRustSide: FfiNetworkingResultListener
  ) throws {
    guard let url = URL(string: rustRequest.url) else {
      throw SwiftSideError.FailedToCreateUrlFrom(string: rustRequest.url)
    }
    let task = dataTask(with: rustRequest.urlRequest(url: url)) { data, urlResponse, error in
      let result = FfiNetworkResult.with(
        data: data,
        urlResponse: urlResponse,
        error: error
      )
      listenerRustSide.notifyResult(result: result)
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

extension Async
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

}

extension Async: FfiNetworkingHandler
where Request == NetworkRequest, Intermediary == (Data, URLResponse), Response == NetworkResponse {
  public func executeNetworkRequest(
    request rustRequest: NetworkRequest,
    listenerRustSide: FfiNetworkingResultListener
  ) throws {
    self.task = Task {
      do {
        let intermediary = try await self.operation(rustRequest)
        let response = try await self.mapToResponse(intermediary)
        listenerRustSide.notifyResult(result: .success(value: response))
      } catch {
        listenerRustSide.notifyResult(result: .fail(error: error))
      }
    }
  }
}

func test() async throws {
  let urlSession = URLSession.shared

  let clientCompletionCallbackBased = GatewayClient(
    networkAntenna: urlSession
  )

  let clientAsyncBased = GatewayClient(
    networkAntenna: Async(call: urlSession.data(for:))
  )

  // Call async method in Rust land from Swift!
  let address = "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease"
  do {
    var balance = try await clientCompletionCallbackBased.getXrdBalanceOfAccount(address: address)
    print("SWIFT ✅ completionCallbackBased balance: \(balance) ✅")
    balance = try await clientAsyncBased.getXrdBalanceOfAccount(address: address)
    print("SWIFT ✅ clientAsyncBased balance: \(balance) ✅")
  } catch {
    print("SWIFT ❌ getXrdBalanceOfAccount failed, error: \(String(describing: error))")
  }

}

try! await test()
