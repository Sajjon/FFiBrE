import Foundation
import network

public final class AsyncOperation<T> {
  typealias Operation = (FfiOperation) async throws -> T
  typealias MapToData = (T) async throws -> Data

  private var task: Task<Void, Never>?

  let supportedOperationKinds: [FfiOperationKind]
  let operation: Operation
  let mapToData: MapToData

  init(
    supportedOperationKinds: [FfiOperationKind] = [.networking],
    operation: @escaping Operation,
    mapToData: @escaping MapToData
  ) {
    self.supportedOperationKinds = supportedOperationKinds
    self.operation = operation
    self.mapToData = mapToData
  }
}

extension AsyncOperation where T == Data {
  convenience init(
    supportedOperationKinds: [FfiOperationKind] = [.networking],
    operation: @escaping Operation
  ) {
    self.init(supportedOperationKinds: supportedOperationKinds, operation: operation) { $0 }
  }

}

extension AsyncOperation: FfiOperationHandler {
  public func supportedOperations() -> [FfiOperationKind] {
    supportedOperationKinds
  }

  public func executeOperation(
    operation rustOperation: FfiOperation,
    listenerRustSide: FfiDataResultListener
  ) throws {
    self.task = Task {
      do {
        let result = try await self.operation(rustOperation)
        let data = try await self.mapToData(result)
        listenerRustSide.notifyResult(result: .success(value: data))
      } catch {
        // wrong error kind....
        listenerRustSide.notifyResult(
          result: .failure(
            error: .UnableToCastUrlResponseToHttpUrlResponse
          ))
      }
    }
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

extension HTTPURLResponse {
  var ok: Bool {
    (200...299).contains(statusCode)
  }
}

// Conform `[Swift]URLSession` to `[Rust]FfiOperationHandler`
extension URLSession: FfiOperationHandler {
  public func supportedOperations() -> [FfiOperationKind] {
    [.networking]
  }

  public func executeOperation(
    operation rustOperation: FfiOperation,
    listenerRustSide: FfiDataResultListener
  ) throws {
    guard case let .networking(rustNetworkRequest) = rustOperation else {
      fatalError(
        """
        Should never happen - Rust side should have queried `supportedOperations()` which states we only support `.networking` request, so no other operation kind should have been sent.
        """
      )
    }
    return try makeNetworkRequest(
      request: rustNetworkRequest,
      listenerRustSide: listenerRustSide
    )
  }
}

extension URLSession {

  // Make a network call using this URLSession, pass back result to Rust via
  // callback.
  func makeNetworkRequest(
    request rustRequest: NetworkRequest,
    listenerRustSide: FfiDataResultListener
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

      // Build result of operation, by inspecting passed triple.
      let networkResult: FfiOperationResult = {
        guard let httpResponse = urlResponse as? HTTPURLResponse else {
          return .failure(
            error: .UnableToCastUrlResponseToHttpUrlResponse
          )
        }
        let statusCode = UInt16(httpResponse.statusCode)
        guard httpResponse.ok else {
          let urlSessionUnderlyingError = error.map { String(describing: $0) }
          let errorMessageFromGateway = data.map { String(data: $0, encoding: .utf8) ?? nil } ?? nil
          return .failure(
            error: .RequestFailed(
              statusCode: statusCode,
              urlSessionUnderlyingError: urlSessionUnderlyingError,
              errorMessageFromGateway: errorMessageFromGateway
            )
          )
        }

        return .success(
          value: data
        )

      }()

      // Notify Rust side that network request has finished by passing `[Rust]FfiOperationResult`
      listenerRustSide.notifyResult(result: networkResult)
    }

    // Start `[Swift]URLSessionDataTask`
    task.resume()
  }
}

func test() async throws {
  // Init `[Rust]GatewayClient` by passing `[Swift]URLSession` as `[Rust]FfiOperationHandler`
  // let gatewayClient = GatewayClient(networkAntenna: URLSession.shared)
  let gatewayClient = GatewayClient(
    networkAntenna: AsyncOperation(
      operation: { rustOperation in
        guard case let .networking(rustNetworkRequest) = rustOperation else {
          fatalError(
            """
            Should never happen - Rust side should have queried `supportedOperations()` which states we only support `.networking` request, so no other operation kind should have been sent.
            """
          )
        }

        return try await URLSession.shared.data(for: rustNetworkRequest.urlRequest()).0
      })
  )

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
