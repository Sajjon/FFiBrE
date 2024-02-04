import Foundation
import ffibre

/* Some bug in UniFFI not marking the `uniffi::Error` as `Swift.Error`... */
extension FfiFileIoWriteError: Swift.Error {}
extension FfiFileIoReadError: Swift.Error {}

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

func test_callback(fileAbsolutePath: String) async throws {
  let fileIoInterface = FileIoInterface(
    fileWriter: CallbackBasedFileManager.shared,
    fileReader: CallbackBasedFileManager.shared
  )

  let outcome = try await fileIoInterface.writeToNewOrExtendExistingFile(
    fileAbsolutePath: fileAbsolutePath,
    extendStrategy: .prepend(separator: "\n"),
    contents: "Callback".data(using: .utf8)!
  )
  print("✅🗂️  writeToNewOrExtendExistingFile CB outcome: \(outcome)")
}

func test_async(fileAbsolutePath: String) async throws {
  let fileIoInterface = FileIoInterface(
    fileWriter: Async(call: AsyncFileManager.shared.write(contents:absolutePath:abortIfExists:)),
    fileReader: Async(call: AsyncFileManager.shared.read(absolutePath:))
  )

  let outcome = try await fileIoInterface.writeToNewOrExtendExistingFile(
    fileAbsolutePath: fileAbsolutePath,
    extendStrategy: .append(separator: "\n"),
    contents: "Async".data(using: .utf8)!
  )
  print("✅🗂️  writeToNewOrExtendExistingFile ASYNC outcome: \(outcome)")
}

func test() async throws {
  print("🚀🗂️  SWIFT 'test_file_io' start")
  defer { print("🏁🗂️  SWIFT 'test_file_io' done") }
  let fileAbsolutePath = "\(FileManager.default.currentDirectoryPath)/safeToRemove.txt"
  do {
    try await test_callback(fileAbsolutePath: fileAbsolutePath)
    try await test_async(fileAbsolutePath: fileAbsolutePath)
    try await test_callback(fileAbsolutePath: fileAbsolutePath)
  } catch {
    print("🚀 🗂️ SWIFT 'test_file_io' error: \(String(describing: error))")

  }
}

try! await test()
