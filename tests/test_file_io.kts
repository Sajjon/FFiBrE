import uniffi.ffibre.*
import kotlinx.coroutines.*
import java.io.*
import java.nio.file.FileSystems
import java.security.AccessControlException

object FileWriter: FfiFileIoWriteExecutor {
    override fun executeFileIoWriteRequest(
        request: FfiFileIoWriteRequest,
        listenerRustSide: FfiFileIoWriteOutcomeListener
    ) {
        val response = runCatching {
            val file = File(request.absolutePath)
            val fileExisted = file.exists()
            if (fileExisted && request.existsStrategy == FileAlreadyExistsStrategy.ABORT) {
                FfiFileIoWriteResponse.OverwriteAborted
            } else {
                file.writeBytes(request.contents)
                FfiFileIoWriteResponse.DidWrite(alreadyExisted = fileExisted)
            }
        }.fold(
            onSuccess = { response ->
                FfiFileIoWriteOutcome.Success(response)
            },
            onFailure = { error ->
                val rustError = when (error) {
                    is AccessControlException -> FfiFileIoWriteError.FailedToCreateNewFile
                    else -> FfiFileIoWriteError.FailedToWriteToFileHandle(underlying = error.message.orEmpty())
                }
                FfiFileIoWriteOutcome.Failure(rustError)
            }
        )

        listenerRustSide.notifyOutcome(response)
    }
}

object FileReader: FfiFileIoReadExecutor {
    override fun executeFileIoReadRequest(
        request: FfiFileIoReadRequest,
        listenerRustSide: FfiFileIoReadOutcomeListener
    ) {
        val response = runCatching {
            val file = File(request.absolutePath)
            if (file.exists()) {
                FfiFileIoReadResponse.Exists(contents = file.readBytes())
            } else {
                FfiFileIoReadResponse.DoesNotExist
            }
        }.fold(
            onSuccess = { response ->
                FfiFileIoReadOutcome.Success(response)
            },
            onFailure = { error ->
                FfiFileIoReadOutcome.Failure(FfiFileIoReadError.Unknown(underlying = error.message.orEmpty()))
            }
        )

        listenerRustSide.notifyOutcome(response)
    }
}



suspend fun write(path: String, content: String, strategy: ExtendExistingFileStrategy) {
    val fileInterface = FileIoInterface(fileWriter = FileWriter, fileReader = FileReader)

    val outcome = fileInterface.writeToNewOrExtendExistingFile(
        fileAbsolutePath = path,
        extendStrategy = strategy,
        contents = content.toByteArray()
    )
    println("✅🗂️  writeToNewOrExtendExistingFile CB outcome: $outcome")
}

fun test() = runBlocking {
    println("🚀🗂️  Kotlin 'test_file_io' start")

    val fileInterface = FileIoInterface(fileWriter = FileWriter, fileReader = FileReader)
    val path = "${FileSystems.getDefault().getPath(".").toAbsolutePath().toString()}/safeToRemove.txt"

    fileInterface.write(
        fileAbsolutePath = path,
        contents = "Kotlin".toByteArray(),
        existsStrategy = FileAlreadyExistsStrategy.OVERWRITE,
    )

    fileInterface.writeToNewOrExtendExistingFile(
        fileAbsolutePath = path,
        contents = "Hello from".toByteArray(),
        extendStrategy = ExtendExistingFileStrategy.Prepend(separator = ": ")
    )

    val content = fileInterface.read(fileAbsolutePath = path)?.toString(charset = Charsets.UTF_8)
    println("Content of file: $path")
    println(content)
    assert("Hello from: Kotlin" == content)

    println("🚀🗂️  Kotlin 'test_file_io' done")
}

test()