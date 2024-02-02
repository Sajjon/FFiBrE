import Foundation
import network

extension NetworkRequest {
  // Convert `[Rust]NetworkRequest` to `[Swift]URLRequest`
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

// Conform `[Swift]URLSession` to `[Rust]HttpClientRequestSender`
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

    // Construct `[Swift]URLSessionDataTask` with `[Swift]URLRequest`
    let task = dataTask(with: swiftURLRequest) { data, urlResponse, error in
      // Inside response callback, called by URLSession when URLSessionDataTask finished
      // translate triple `[Swift](data, urlResponse, error)` -> `[Rust]NetworkResult`
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
      // Notify Rust side that network request has finished by passing `[Rust]NetworkResult`
      responseBack.response(result: networkResult)
    }

    // Start `[Swift]URLSessionDataTask`
    task.resume()
  }
}

func test() async throws {
  // Init `[Rust]HttpClient` by passing `[Swift]URLSession` as `[Rust]HttpClientRequestSender`
  // which conforms thanks to impl above
  let httpClient = HttpClient(requestSender: URLSession.shared)
  // Init `[Rust]GatewayClient` with httpClient, now capable of doing network request
  let gatewayClient = GatewayClient(httpClient: httpClient)

  // Call async method in Rust land from Swift!
  do {

    let balance = try await gatewayClient.getXrdBalanceOfAccount(
      address: "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease"
    )
    // Print result, if successful
    print("SWIFT ✅ getXrdBalanceOfAccount success, got balance: \(balance) ✅")
  } catch {
    print("SWIFT ❌ getXrdBalanceOfAccount failed, error: \(String(describing: error))")
  }

}

try! await test()
