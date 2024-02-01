import Foundation

extension NetworkRequest {
    func urlRequest() -> URL
}

final class RequestSender: HTTPClientRequestSender {
  unowned var httpClient: HttpClient?
  func send(request: NetworkRequest) throws {
    fatalError("Impl URLRequestMaker")
    // httpClient.gotResultOfNetworkRequest() // ResultOfNetworkRequest
  }
}

func test() throws {
  print("HELLO WORLD from swift")
  var counter = DispatchGroup()
  counter.enter()

  let requestSender = RequestSender()
  let httpClient = HttpClient(requestSender: requestSender)
  requestSender.httpClient = httpClient  // install callback
  let gatewayClient = GatewayClient(httpClient: httpClient)
  Task {
    defer { counter.leave() }
    let balance = try await gatewayClient.getXrdBalanceOfAccount(
      address: "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease")
    print("✅ successfully got balance")
  }

}

try! test()
