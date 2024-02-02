import Foundation
import network

extension NetworkRequest {
  func urlRequest() throws -> URLRequest {
    fatalError()
  }
}

extension URLSession: HttpClientRequestSender {
  public func send(request: NetworkRequest, responseBack: NotifyRustFromSwift) throws {
    try self.dataTask(with: request.urlRequest()) {
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
    }.resume()
  }
}

var counter = DispatchGroup()
counter.enter()

func test() throws {
  print("HELLO WORLD from swift")

  let httpClient = HttpClient(requestSender: URLSession.shared)
  let gatewayClient = GatewayClient(httpClient: httpClient)
  Task {
    let balance = try await gatewayClient.getXrdBalanceOfAccount(
      address: "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease")
    print("✅ successfully got balance")
    counter.leave()
  }

}

try! test()
