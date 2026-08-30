package me.lampu.lampcord.shared.api

import io.ktor.client.call.body
import io.ktor.client.request.get
import io.ktor.client.request.parameter
import io.ktor.http.isSuccess
import me.lampu.lampcord.shared.model.Application
import me.lampu.lampcord.shared.utils.Logging

class ApplicationApi(private val rest: RestClient) {
    suspend fun getApplications(applicationIds: List<String>): List<Application> {
        if (applicationIds.isEmpty()) return emptyList()
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/applications/public") {
                standardHeaders(rest)
                parameter("application_ids", applicationIds.joinToString(","))
            }
            if (response.status.isSuccess()) {
                response.body()
            } else {
                emptyList()
            }
        } catch (e: Exception) {
            Logging.e("Application", "Error fetching applications: ${e.message}")
            emptyList()
        }
    }
}
