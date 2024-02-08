import uniffi.ffibre.*
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*
import okhttp3.*
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.Headers.Companion.toHeaders

object KotlinNetworkAntenna: FfiNetworkingExecutor {
    private val client = OkHttpClient()

    override fun executeNetworkingRequest(request: FfiNetworkingRequest, listenerRustSide: FfiNetworkingOutcomeListener) {
        val outcome = runCatching {
            val contentType = request.headers["Content-Type"] ?: "application/json; charset=utf-8"

            val requestBody = RequestBody.create(
                contentType.toMediaType(),
                request.body
            )
            val request = Request.Builder()
                .url(url = request.url)
                .headers(request.headers.toHeaders())
                .method(method = request.method, body = requestBody)
                .build()

            client.newCall(request).execute()
        }.fold(
            onSuccess = { response ->
                if (response.isSuccessful) {
                    FfiNetworkingOutcome.Success(
                        value = FfiNetworkingResponse(
                            statusCode = response.code.toUShort(),
                            body = response.body?.bytes() ?: byteArrayOf()
                        )
                    )
                } else {
                    FfiNetworkingOutcome.Failure(
                        error = FfiNetworkingError.RequestFailed(
                            statusCode = response.code.toUShort(),
                            urlSessionUnderlyingError = null,
                            errorMessageFromGateway = response.body?.string()
                        )
                    )
                }
            },
            onFailure = { error ->
                FfiNetworkingOutcome.Failure(
                    error = FfiNetworkingError.RequestFailed(
                        statusCode = null,
                        urlSessionUnderlyingError = error.toString(),
                        errorMessageFromGateway = null
                    )
                )
            }
        )

        listenerRustSide.notifyOutcome(result = outcome)
    }
}

fun testAsyncStream(): Flow<Transaction> = flow {
    val client = GatewayClient(networkAntenna = KotlinNetworkAntenna)

    while (true) {
        val transaction = client.getLatestTransactionsOrPanic()
        emit(transaction)
        delay(7000)
    }
}


fun test() {
    println("🛜 🚀 Kotlin 'test_test_async_stream' start")
    runBlocking {
        testAsyncStream()
            .onStart {
                println("🛜 ┌ Subscribed to transactions")
            }
            .distinctUntilChanged { old: Transaction, new: Transaction ->
                if (old == new) {
                    println("🛜 ┝ IGNORED: Latest transaction is still `${new.txId}`.")
                }

                old == new
            }
            .take(3)
            .catch { error: Throwable ->
                println("🛜 ┝ ❌ ${error}")
            }
            .collect { transaction ->
                println("🛜 ┝ $transaction")
            }
    }
    println("🛜 └ 🏁 Kotlin 'test_test_async_stream' done")
}

test()