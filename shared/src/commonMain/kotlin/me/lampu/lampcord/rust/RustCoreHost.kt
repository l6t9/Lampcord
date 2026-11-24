package me.lampu.lampcord.rust

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.withContext
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.EntityStore
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.MemberListStore
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.PresenceStore
import me.lampu.lampcord.shared.state.ReadStateStore
import me.lampu.lampcord.shared.state.RelationshipStore
import me.lampu.lampcord.shared.state.RustCoreMode
import me.lampu.lampcord.shared.state.SessionManager
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.utils.Logging
import me.lampu.lampcord.shared.utils.getAppStoragePath

@Composable
fun RustCoreHost() {
    val sessionManager: SessionManager = koinInject()
    val entityStore: EntityStore = koinInject()
    val guildStore: GuildStore = koinInject()
    val messageStore: MessageStore = koinInject()
    val navigationStore: NavigationStore = koinInject()
    val userStore: UserStore = koinInject()
    val presenceStore: PresenceStore = koinInject()
    val readStateStore: ReadStateStore = koinInject()
    val memberListStore: MemberListStore = koinInject()
    val relationshipStore: RelationshipStore = koinInject()

    LaunchedEffect(sessionManager) {
        val engine = withContext(Dispatchers.Default) {
            try {
                RustCoreEngine(RustClientConfig(storagePath = "${getAppStoragePath()}/lampcord-accounts.json"))
            } catch (error: Throwable) {
                Logging.e("RustCore", "RustCoreEngine init failed", error)
                null
            }
        }
        val token = Settings.shared.discordToken.trim().takeIf { it.isNotBlank() }
        if (engine == null) {
            Logging.w("RustCore", "Failed to start RustCoreEngine; falling back to Kotlin gateway")
            RustCoreMode.enabled = false
            if (token != null) sessionManager.connect(token)
            return@LaunchedEffect
        }
        Logging.i("RustCore", "RustCoreEngine started")
        val integration = RustCoreIntegration(
            engine,
            entityStore,
            guildStore,
            messageStore,
            navigationStore,
            userStore,
            presenceStore,
            readStateStore,
            memberListStore,
            relationshipStore,
        )
        try {
            integration.start()
            if (token != null) engine.connectToken(token) else engine.connectStoredToken()
            awaitCancellation()
        } finally {
            integration.close()
            engine.close()
            Logging.i("RustCore", "RustCoreEngine stopped")
        }
    }
}
