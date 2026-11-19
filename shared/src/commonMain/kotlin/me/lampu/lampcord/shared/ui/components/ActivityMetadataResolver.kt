package me.lampu.lampcord.shared.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import me.lampu.lampcord.shared.api.UserApi
import me.lampu.lampcord.shared.model.Activity
import me.lampu.lampcord.shared.model.ActivityMetadata
import me.lampu.lampcord.shared.model.resolvedButtons
import org.koin.compose.koinInject

@Composable
fun rememberActivitiesWithButtonUrls(
    activities: List<Activity>,
    userId: String,
    userApi: UserApi = koinInject()
): List<Activity> {
    val metadataCache = remember(userId) { mutableStateMapOf<String, ActivityMetadata>() }
    var resolved by remember(userId) { mutableStateOf(activities) }

    val activitiesKey = activities.joinToString("|") { activity ->
        listOf(
            activity.id.orEmpty(),
            activity.session_id.orEmpty(),
            activity.application_id.orEmpty(),
            activity.buttons?.size?.toString().orEmpty()
        ).joinToString("/")
    }

    LaunchedEffect(userId, activitiesKey) {
        resolved = activities.map { activity ->
            val sessionId = activity.session_id
            if (activity.resolvedButtons.isNotEmpty() ||
                activity.buttons.isNullOrEmpty() ||
                sessionId.isNullOrBlank()
            ) {
                activity
            } else {
                val applicationId = activity.application_id
                    ?: if (activity.type == Activity.TYPE_LISTENING) "0" else null
                if (applicationId == null) {
                    activity
                } else {
                    val cacheKey = "$userId/$sessionId/$applicationId"
                    val metadata = metadataCache[cacheKey]
                        ?: userApi.getActivityMetadata(userId, sessionId, applicationId)
                            ?.also { metadataCache[cacheKey] = it }
                    if (metadata != null) activity.copy(metadata = metadata) else activity
                }
            }
        }
    }

    return resolved
}
