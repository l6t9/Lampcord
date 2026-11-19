package me.lampu.lampcord.shared.api

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.client.request.forms.MultiPartFormDataContent
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import io.ktor.utils.io.ByteChannel
import io.ktor.utils.io.InternalAPI
import io.ktor.utils.io.readBuffer
import kotlinx.coroutines.runBlocking
import kotlinx.io.readByteArray
import kotlinx.serialization.json.Json
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

@OptIn(InternalAPI::class)
class MessageMultipartTest {

    @Test
    fun `file part has a single content disposition with name and filename`() = runBlocking {
        val body = captureBody("photo.png")

        assertEquals(
            "Content-Disposition: form-data; name=\"files[0]\"; filename=\"photo.png\"",
            body.lines().first { "filename=\"photo.png\"" in it }.trim(),
        )
        assertTrue("Content-Type: image/png" in body)
    }

    @Test
    fun `webm files use video webm content type`() = runBlocking {
        val body = captureBody("clip.webm")

        assertEquals(
            "Content-Disposition: form-data; name=\"files[0]\"; filename=\"clip.webm\"",
            body.lines().first { "filename=\"clip.webm\"" in it }.trim(),
        )
        assertTrue("Content-Type: video/webm" in body)
    }

    private suspend fun captureBody(filename: String): String {
        var bodyText: String? = null
        val engine = MockEngine { request ->
            val body = request.body as? MultiPartFormDataContent
            if (body != null) {
                val channel = ByteChannel(autoFlush = true)
                body.writeTo(channel)
                bodyText = channel.readBuffer().readByteArray().toString(Charsets.UTF_8)
            }
            respond("{}", HttpStatusCode.BadRequest, headersOf(HttpHeaders.ContentType, "application/json"))
        }
        val client = HttpClient(engine) { expectSuccess = false }
        val api = MessageApi(RestClient(client, Json { ignoreUnknownKeys = true }))

        api.sendMessage("channel-id", "hello", files = listOf(filename to byteArrayOf(1, 2, 3)))
        client.close()

        return bodyText ?: error("Expected a multipart body in the request")
    }
}
