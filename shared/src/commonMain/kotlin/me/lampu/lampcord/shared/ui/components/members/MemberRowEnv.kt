package me.lampu.lampcord.shared.ui.components.members

import androidx.compose.runtime.Stable
import androidx.compose.runtime.State
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.PresenceUpdate

@Stable
class MemberRowEnv(
    val guild: Guild?,
    val currentUserId: String?,
    val currentUserStatus: String?,
    val developerMode: Boolean,
    val presences: State<Map<String, PresenceUpdate>>,
    val relationshipTypes: State<Map<String, Int>>,
    val animate: Boolean,
    val isTouch: Boolean,
    val loadImages: Boolean
)
