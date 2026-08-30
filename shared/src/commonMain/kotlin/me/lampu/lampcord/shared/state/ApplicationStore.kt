package me.lampu.lampcord.shared.state

import androidx.compose.runtime.mutableStateMapOf
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import me.lampu.lampcord.shared.api.ApplicationApi
import me.lampu.lampcord.shared.model.Application

class ApplicationStore(
    private val applicationApi: ApplicationApi,
    private val scope: CoroutineScope
) {
    val applications = mutableStateMapOf<String, Application>()
    private val loadingIds = mutableSetOf<String>()
    private val mutex = Mutex()

    fun fetchIfNonExisting(applicationId: String) {
        scope.launch {
            mutex.withLock {
                if (applications.containsKey(applicationId) || loadingIds.contains(applicationId)) {
                    return@launch
                }
                loadingIds.add(applicationId)
            }

            try {
                val results = applicationApi.getApplications(listOf(applicationId))
                results.forEach { app ->
                    applications[app.id] = app
                }
            } catch (e: Exception) {
                // Ignore
            } finally {
                mutex.withLock {
                    loadingIds.remove(applicationId)
                }
            }
        }
    }
    
    fun getApplication(applicationId: String): Application? {
        val app = applications[applicationId]
        if (app == null) {
            fetchIfNonExisting(applicationId)
        }
        return app
    }
}
