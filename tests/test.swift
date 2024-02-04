import Foundation
import ffibre

extension FfiNetworkingError: Swift.Error {} /* Some bug in UniFFI... */
extension FfiFileIoWriteError: Swift.Error {} /* Some bug in UniFFI... */
extension FfiFileIoReadError: Swift.Error {} /* Some bug in UniFFI... */

struct FileHandleForWritingOutcome {
  let fileAlreadyExists: Bool
  let result: Result<FileHandle, FfiFileIoWriteError>
}
func fileForWriting(to absolutePath: String) -> FileHandleForWritingOutcome {

  let alreadyExisted = FileManager.default.fileExists(atPath: absolutePath)
  if !alreadyExisted {
    guard FileManager.default.createFile(atPath: absolutePath, contents: nil) else {
      return FileHandleForWritingOutcome(
        fileAlreadyExists: false,
        result: .failure(.failedToCreateNewFile)
      )
    }
  }
  guard let fileHandle = FileHandle(forWritingAtPath: absolutePath) else {
    return FileHandleForWritingOutcome(
      fileAlreadyExists: alreadyExisted,
      result: .failure(.failedToGetHandleToFileForWriting)
    )
  }
  return FileHandleForWritingOutcome(
    fileAlreadyExists: alreadyExisted,
    result: .success(fileHandle)
  )

}

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

public final class AsyncFileManager {
  private init() {}
  static let shared = AsyncFileManager()
}
extension AsyncFileManager {
  public func read(absolutePath: String) async throws -> Data? {
    guard let fileHandle = FileHandle(forReadingAtPath: absolutePath) else {
      return nil
    }
    var iterator = fileHandle.bytes.makeAsyncIterator()

    do {
      var contents = Data()
      while let byte = try await iterator.next() {
        contents.append(byte)
      }
      return contents
    } catch {
      throw FfiFileIoReadError.unknown(underlying: String(describing: error))
    }
  }

  public func write(contents: Data, absolutePath: String, abortIfExists: Bool) async throws
    -> FfiFileIoWriteResponse
  {
    let fileHandleOutcome = fileForWriting(to: absolutePath)
    let alreadyExists = fileHandleOutcome.fileAlreadyExists
    if abortIfExists, alreadyExists {
      return .overwriteAborted
    }
    switch fileHandleOutcome.result {
    case let .failure(error): throw error
    case let .success(fileHandle):
      do {
        try fileHandle.write(contentsOf: contents)
        return .didWrite(alreadyExisted: alreadyExists)
      } catch {
        throw FfiFileIoWriteError.failedToWriteToFileHandle(
          underlying: String(describing: error)
        )
      }
    }
  }

}

extension Async: FfiFileIoReadHandler
where Request == FfiFileIoReadRequest, Intermediary == Data?, Response == FfiFileIoReadResponse {

  convenience init(
    call op: @escaping (String) async throws -> Intermediary
  ) {
    self.init(
      operation: { (rustRequest: Request) in try await op(rustRequest.absolutePath) },
      mapToResponse: { (data: Data?) in
        data.map { .exists(contents: $0) } ?? .doesNotExist
      }
    )
  }

  public func executeFileIoReadRequest(
    request rustRequest: FfiFileIoReadRequest,
    listenerRustSide: FfiFileIoReadOutcomeListener
  ) throws {
    self.task = Task {
      do {
        let intermediary = try await self.operation(rustRequest)
        let response = try await self.mapToResponse(intermediary)
        listenerRustSide.notifyOutcome(result: .success(value: response))
      } catch {
        listenerRustSide.notifyOutcome(
          result: .failure(error: .unknown(underlying: String(describing: error))))
      }
    }
  }
}

extension Async: FfiFileIoWriteHandler
where
  Request == FfiFileIoWriteRequest, Intermediary == FfiFileIoWriteResponse,
  Response == FfiFileIoWriteResponse
{

  convenience init(
    call op: @escaping (Data, String, Bool) async throws -> Intermediary
  ) {
    self.init(
      operation: { (rustRequest: Request) in
        try await op(
          rustRequest.contents, rustRequest.absolutePath, rustRequest.existsStrategy == .abort)
      },
      mapToResponse: { $0 }
    )
  }

  public func executeFileIoWriteRequest(
    request rustRequest: Request,
    listenerRustSide: FfiFileIoWriteOutcomeListener
  ) throws {
    self.task = Task {
      do {
        let intermediary = try await self.operation(rustRequest)
        let response = try await self.mapToResponse(intermediary)
        listenerRustSide.notifyOutcome(result: .success(value: response))
      } catch let writeError as FfiFileIoWriteError {
        listenerRustSide.notifyOutcome(
          result: .failure(error: writeError)
        )
      } catch {
        fatalError("Expected all errors to be of type FfiFileIoWriteError")
      }
    }
  }
}

public final class CallbackBasedFileManager {
  private let queue: DispatchQueue
  private init() {
    self.queue = DispatchQueue(
      label: "simpleFileManagerQueue",
      qos: .background,
      target: nil
    )
  }
  static let shared = CallbackBasedFileManager()
}
extension CallbackBasedFileManager: FfiFileIoReadHandler {
  func read(absolutePath: String, callback: @escaping (FfiFileIoReadOutcome) -> Void) {
    print("🪲 SWIFT SimpleFileManager read: '\(absolutePath)'")
    guard let fileHandle = FileHandle(forReadingAtPath: absolutePath) else {
      return callback(
        FfiFileIoReadOutcome.success(value: .doesNotExist))
    }
    queue.async {
      let result: FfiFileIoReadOutcome
      do {
        if let contents = try fileHandle.readToEnd() {
          result = .success(
            value: .exists(
              contents: contents
            )
          )
        } else {
          result = .success(value: .doesNotExist)
        }
      } catch {
        result = .failure(error: .unknown(underlying: String(describing: error)))
      }
      DispatchQueue.main.async {
        callback(result)
      }
    }
  }

  public func executeFileIoReadRequest(
    request: FfiFileIoReadRequest,
    listenerRustSide: FfiFileIoReadOutcomeListener
  ) throws {
    self.read(absolutePath: request.absolutePath) { result in
      listenerRustSide.notifyOutcome(result: result)
    }
  }
}

extension CallbackBasedFileManager: FfiFileIoWriteHandler {
  func write(
    contents: Data,
    to absolutePath: String,
    abortIfExists: Bool,
    callback: @escaping (FfiFileIoWriteOutcome) -> Void
  ) {
    print("🐌 SWIFT SimpleFileManager write: '\(absolutePath)'")

    let fileHandleOutcome = fileForWriting(to: absolutePath)
    let alreadyExists = fileHandleOutcome.fileAlreadyExists
    if abortIfExists, alreadyExists {
      return callback(.success(value: .overwriteAborted))
    }

    queue.async {
      let result: FfiFileIoWriteOutcome

      switch fileHandleOutcome.result {
      case let .failure(error): result = .failure(error: error)
      case let .success(fileHandle):
        do {
          try fileHandle.write(contentsOf: contents)
          result = .success(value: .didWrite(alreadyExisted: alreadyExists))
        } catch {
          result = .failure(
            error: .failedToWriteToFileHandle(
              underlying: String(describing: error)
            ))
        }
      }

      DispatchQueue.main.async {
        callback(result)
      }
    }

  }
  public func executeFileIoWriteRequest(
    request: FfiFileIoWriteRequest,
    listenerRustSide: FfiFileIoWriteOutcomeListener
  ) throws {
    self.write(
      contents: request.contents,
      to: request.absolutePath,
      abortIfExists: request.existsStrategy == .abort
    ) { result in
      listenerRustSide.notifyOutcome(result: result)
    }
  }
}

func test_file_io_callbackbased(fileAbsolutePath: String) async throws {
  print("🚀 SWIFT 'test_file_io_callbackbased' start")
  defer { print("🏁 SWIFT 'test_file_io_callbackbased' done") }
  let fileIoInterface = FileIoInterface(
    fileWriter: CallbackBasedFileManager.shared,
    fileReader: CallbackBasedFileManager.shared
  )

  let outcome = try await fileIoInterface.writeToNewOrExtendExistingFile(
    fileAbsolutePath: fileAbsolutePath,
    extendStrategy: .prepend(separator: "\n"),
    contents: "Callback".data(using: .utf8)!
  )
  print("✅ writeToNewOrExtendExistingFile CB outcome: \(outcome)")
}

func test_file_io_async(fileAbsolutePath: String) async throws {
  print("🚀 SWIFT 'test_file_io_callbackbased' start")
  defer { print("🏁 SWIFT 'test_file_io_callbackbased' done") }
  let fileIoInterface = FileIoInterface(
    fileWriter: Async(call: AsyncFileManager.shared.write(contents:absolutePath:abortIfExists:)),
    fileReader: Async(call: AsyncFileManager.shared.read(absolutePath:))
  )

  let outcome = try await fileIoInterface.writeToNewOrExtendExistingFile(
    fileAbsolutePath: fileAbsolutePath,
    extendStrategy: .append(separator: "\n"),
    contents: "Async".data(using: .utf8)!
  )
  print("✅ writeToNewOrExtendExistingFile ASYNC outcome: \(outcome)")
}

func test_file_io() async throws {
  print("🚀 SWIFT 'test_file_io' start")
  defer { print("🏁 SWIFT 'test_file_io' done") }
  let fileAbsolutePath = "\(FileManager.default.currentDirectoryPath)/safeToRemove.txt"
  try await test_file_io_callbackbased(fileAbsolutePath: fileAbsolutePath)
  try await test_file_io_async(fileAbsolutePath: fileAbsolutePath)
  try await test_file_io_callbackbased(fileAbsolutePath: fileAbsolutePath)
}

func test() async throws {
  print("🚀 SWIFT 'test' start")
  defer { print("🏁 SWIFT 'test' done") }
  try await test_file_io()
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
