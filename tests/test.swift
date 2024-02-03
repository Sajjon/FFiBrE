import Foundation
import network

extension FfiOperation {
  var asNetworkRequest: NetworkRequest {
    guard case let .networking(rustNetworkRequest) = self else {
      fatalError(
        """
        Should never happen - Rust side should have queried `supportedOperations()` which states we only support `.networking` request, so no other operation kind should have been sent.
        """
      )
    }
    return rustNetworkRequest
  }
}

extension FfiNetworkResult {
  static func with(
    to request: NetworkRequest,
    data: Data?,
    urlResponse: URLResponse?,
    error: Swift.Error?
  ) -> Self {
    func reason() -> String? {
      error.map { String(describing: $0) } ?? nil
    }
    func message() -> String? {
      data.map { String(data: $0, encoding: .utf8) } ?? nil
    }
    func statusCode() -> UInt16? {
      urlResponse.map { $0 as? HTTPURLResponse ?? nil }?.map { UInt16($0.statusCode) } ?? nil
    }

    guard let urlResponse else {
      return .failure(
        error: .RequestFailed(
          statusCode: statusCode(),
          urlSessionUnderlyingError: reason(),
          errorMessageFromGateway: message()
        )
      )
    }

    guard let httpUrlResponse = urlResponse as? HTTPURLResponse else {
      return .failure(
        error: .UnableToCastUrlResponseToHttpUrlResponse
      )
    }

    if error != nil {
      return .failure(
        error: .RequestFailed(
          statusCode: statusCode(),
          urlSessionUnderlyingError: message(),
          errorMessageFromGateway: message()
        )
      )
    }

    guard let data else {
      fatalError("Should not happen, error was nil, so we should have data.")
    }

    return .success(
      value: NetworkResponse(
        statusCode: statusCode() ?? 200,
        url: httpUrlResponse.url.map { $0.absoluteString } ?? request.url,
        headers: (httpUrlResponse.allHeaderFields as? [String: String]) ?? [:],
        body: data
      )
    )
  }
}

/*
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

extension AsyncOperation: FFINetworkingHandler {
	public func supportedOperations() -> [FfiOperationKind] {
		supportedOperationKinds
	}

	public func executeNetworkRequest(
		request rustNetworkRequest: NetworkRequest,
		listenerRustSide: FfiDataResultListener
	) throws {
		self.task = Task {
			do {
				let result = try await self.operation(.Networking())
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
*/
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
        to: rustRequest,
        data: data,
        urlResponse: urlResponse,
        error: error
      )
      listenerRustSide.notifyResult(result: result)
    }
    task.resume()
  }
}

func test() async throws {
  let urlSession = URLSession.shared

  let clientCompletionCallbackBased = GatewayClient(
    networkAntenna: urlSession
  )

  // let clientAsyncBased = GatewayClient(
  //   networkAntenna: AsyncOperation {
  //     try await urlSession.data(for: $0.asNetworkRequest.urlRequest()).0
  //   }
  // )

  // Call async method in Rust land from Swift!
  let address = "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease"
  do {
    var balance = try await clientCompletionCallbackBased.getXrdBalanceOfAccount(address: address)
    print("SWIFT ✅ completionCallbackBased balance: \(balance) ✅")
    // balance = try await clientAsyncBased.getXrdBalanceOfAccount(address: address)
    // print("SWIFT ✅ clientAsyncBased balance: \(balance) ✅")
  } catch {
    print("SWIFT ❌ getXrdBalanceOfAccount failed, error: \(String(describing: error))")
  }

}

try! await test()
