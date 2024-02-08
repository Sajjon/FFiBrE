import uniffi.ffibre.*
import kotlinx.coroutines.*
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

suspend fun testBalance(address: String) = runCatching {
    println("🛜 ┌ Test Balance")
    println("🛜 ┝ Request for $address")
    val client = GatewayClient(networkAntenna = KotlinNetworkAntenna)
    client.getXrdBalanceOfAccount(address = address)
}.onSuccess { balance ->
    println("🛜 ┝ $balance ")
    println("🛜 └ ✅ ")
}.onFailure { error ->
    println("🛜 └ ❌  ${error}")
}

suspend fun testLatestTransactions() = runCatching {
    println("🛜 ┌ Test Latest Transactions")
    val client = GatewayClient(networkAntenna = KotlinNetworkAntenna)
    client.getLatestTransactions()
}.onSuccess { transactions ->
     println("${transactions.joinToString(prefix = "🛜 ┝ ", separator = "\n🛜 ┝ ")}")
     println("🛜 └ ✅ ")
}.onFailure { error ->
     println("🛜 └ ❌  ${error}")
}

suspend fun testLatestTransaction() = runCatching {
    println("🛜 ┌ Test Latest Transaction or Panic ")
    val client = GatewayClient(networkAntenna = KotlinNetworkAntenna)
    client.getLatestTransactionsOrPanic()
}.onSuccess { transaction ->
     println("🛜 ┝ $transaction")
     println("🛜 └ ✅ ")
}.onFailure { error ->
     println("🛜 └ ❌  ${error}")
}

fun test() = runBlocking {
    println("🛜 🚀 Kotlin 'test_networking' start")

    testBalance(address = "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease")
    testLatestTransactions()
    testLatestTransaction()

    println("🛜 🏁 Kotlin 'test_networking' done")
}

test()
