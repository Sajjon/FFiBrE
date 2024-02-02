import Foundation
import network

extension NetworkRequest {
  func urlRequest() throws -> URLRequest {
    guard let url = URL(string: self.url) else {
      fatalError("invalid url")
    }
    var request = URLRequest(url: url)
    request.httpMethod = self.method
    request.httpBody = self.body
    request.allHTTPHeaderFields = self.headers
    print("Swift constructed request: \(request)\nfrom: \(self)")
    return request
  }
}

extension URLSession: HttpClientRequestSender {
  public func send(request: NetworkRequest, responseBack: NotifyRustFromSwift) throws {
    print("⚡️ START HttpClientRequestSender (URLSession) - send:request:responseBack method")
    defer { print("⚡️ END HttpClientRequestSender (URLSession) - send:request:responseBack method") }
    let dataTask = try self.dataTask(with: request.urlRequest()) {
      (data: Data?, resp: URLResponse?, err: Error?) in
      let res: NetworkResult = {
        if let data {
          guard let httpResp = resp as? HTTPURLResponse else { fatalError("not http resp") }
          let resResp = NetworkResponse(responseCode: UInt16(httpResp.statusCode), body: data)
          return NetworkResult.success(value: resResp)
        } else if let err {
          return NetworkResult.failure(
            error: NetworkError.UrlSessionDataTaskFailed(underlying: String(describing: err)))
        } else {
          fatalError("Bad state, data AND error was nil.")
        }
      }()
      responseBack.response(result: res)
    }
    print("Created dataTask, now resuming it")
    dataTask.resume()
  }
}

func test() async throws {
  print("HELLO WORLD from swift")
  defer { print("BY BYE from swift") }

  let httpClient = HttpClient(requestSender: URLSession.shared)
  let gatewayClient = GatewayClient(httpClient: httpClient)
  print("🧵🚀Task started")
  let balance = try await gatewayClient.getXrdBalanceOfAccount(
    address: "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease")
  print("✅ successfully got balance: \(balance)")
  print("🧵✅ Task end")

}

try! await test()
