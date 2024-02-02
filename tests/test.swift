import Foundation
import network

extension NetworkRequest {
  /// Convert `[Rust]NetworkRequest` to `[Swift]URLRequest`
  func urlRequest(url: URL) -> URLRequest {
    var request = URLRequest(url: url)
    request.httpMethod = self.method
    request.httpBody = self.body
    request.allHTTPHeaderFields = self.headers
    return request
  }
}

extension HTTPURLResponse {
  var ok: Bool {
    (200...299).contains(statusCode)
  }
}

/// Conform `[Swift]URLSession` to `[Rust]HttpClientRequestSender`
extension URLSession: HttpClientRequestSender {

  public func send(
    request rustRequest: NetworkRequest,
    responseBack: NotifyRustFromSwift
  ) throws {
    let urlString = rustRequest.url
    guard let url = URL(string: urlString) else {
      throw SwiftSideError.FailedToCreateUrlFrom(string: urlString)
    }
    let swiftURLRequest = rustRequest.urlRequest(url: url)
    let task = dataTask(with: swiftURLRequest) { data, urlResponse, error in
      let networkResult: NetworkResult = {
        guard let httpResponse = urlResponse as? HTTPURLResponse else {
          return .failure(
            error: .UnableToCastUrlResponseToHttpUrlResponse
          )
        }
        let statusCode = UInt16(httpResponse.statusCode)
        guard httpResponse.ok else {
          let reason = error.map { String(describing: $0) } ?? "Unknown"
          return .failure(
            error: .RequestFailed(
              statusCode: statusCode, reason: reason
            )
          )
        }

        return .success(
          value: NetworkResponse(statusCode: statusCode, body: data)
        )

      }()
      responseBack.response(result: networkResult)
    }
    task.resume()
  }
}

func test() async throws {

  let httpClient = HttpClient(requestSender: URLSession.shared)
  let gatewayClient = GatewayClient(httpClient: httpClient)
  let balance = try await gatewayClient.getXrdBalanceOfAccount(
    address: "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease")
  print("SWIFT ✅ successfully got balance: \(balance) ✅")
}

try! await test()
