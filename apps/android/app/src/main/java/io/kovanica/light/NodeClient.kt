package io.kovanica.light

import org.json.JSONArray
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL

/** Blocking JSON-RPC and faucet client. Call it off the UI thread. */
class NodeClient(base: String) {
    private val rpcUrl: String = if (base.trimEnd('/').endsWith("/rpc")) {
        base.trimEnd('/')
    } else {
        base.trimEnd('/') + "/rpc"
    }
    val faucetBase: String = base.trimEnd('/').removeSuffix("/rpc")

    fun call(method: String, params: JSONArray): Any? {
        val body = JSONObject()
            .put("jsonrpc", "2.0")
            .put("method", method)
            .put("params", params)
            .put("id", 1)
        val response = post(rpcUrl, body.toString())
        val parsed = JSONObject(response)
        if (parsed.has("error") && !parsed.isNull("error")) {
            throw IllegalStateException("RPC $method failed: ${parsed.get("error")}")
        }
        if (!parsed.has("result") || parsed.isNull("result")) {
            return null
        }
        return parsed.get("result")
    }

    fun postFaucet(address: String): String {
        val body = JSONObject().put("address", address)
        return post(faucetBase.trimEnd('/') + "/faucet", body.toString())
    }

    private fun post(url: String, body: String): String {
        val connection = (URL(url).openConnection() as HttpURLConnection).apply {
            requestMethod = "POST"
            connectTimeout = 15_000
            readTimeout = 15_000
            doOutput = true
            setRequestProperty("Content-Type", "application/json")
        }
        connection.outputStream.use { it.write(body.toByteArray()) }
        val stream = if (connection.responseCode in 200..299) {
            connection.inputStream
        } else {
            connection.errorStream
        }
        val text = stream.bufferedReader().readText()
        if (connection.responseCode !in 200..299) {
            throw IllegalStateException("HTTP ${connection.responseCode} from $url: $text")
        }
        return text
    }
}

fun parseQuantity(value: Any?): ULong {
    return when (value) {
        is Number -> value.toLong().toULong()
        is String -> {
            val raw = value.removePrefix("0x")
            if (raw.isEmpty()) 0uL else raw.toULong(16)
        }
        else -> throw IllegalArgumentException("expected a quantity, got $value")
    }
}
