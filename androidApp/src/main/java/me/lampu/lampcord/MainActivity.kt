package me.lampu.lampcord

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.graphics.Color
import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.SystemBarStyle
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.ContextCompat
import me.lampu.lampcord.shared.state.ChannelNavigator
import me.lampu.lampcord.shared.state.DiscordLinkHandler
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.ui.App
import org.koin.core.context.GlobalContext

class MainActivity : ComponentActivity() {

    private val requestNotificationsPermission =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) { }

    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge(
            statusBarStyle = SystemBarStyle.auto(Color.TRANSPARENT, Color.TRANSPARENT),
            navigationBarStyle = SystemBarStyle.auto(Color.TRANSPARENT, Color.TRANSPARENT),
        )
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            window.isNavigationBarContrastEnforced = false
        }
        super.onCreate(savedInstanceState)
        requestNotificationsPermissionIfNeeded()
        setContent {
            App()
        }
        handleNotificationIntent(intent)
        handleDeepLink(intent)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        handleNotificationIntent(intent)
        handleDeepLink(intent)
    }

    private fun requestNotificationsPermissionIfNeeded() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            val granted = ContextCompat.checkSelfPermission(this, Manifest.permission.POST_NOTIFICATIONS)
            if (granted != PackageManager.PERMISSION_GRANTED) {
                requestNotificationsPermission.launch(Manifest.permission.POST_NOTIFICATIONS)
            }
        }
    }

    private fun handleNotificationIntent(intent: Intent?) {
        if (intent == null) return
        val navigationStore = GlobalContext.get().get<NavigationStore>()

        navigationStore.isBubble = intent.getBooleanExtra(NotificationHelper.EXTRA_IS_BUBBLE, false)
        intent.removeExtra(NotificationHelper.EXTRA_IS_BUBBLE)

        val channelId = intent.getStringExtra(NotificationHelper.EXTRA_CHANNEL_ID) ?: return
        val guildId = intent.getStringExtra(NotificationHelper.EXTRA_GUILD_ID)
        intent.removeExtra(NotificationHelper.EXTRA_CHANNEL_ID)
        intent.removeExtra(NotificationHelper.EXTRA_GUILD_ID)

        val navigator = GlobalContext.get().get<ChannelNavigator>()
        navigator.navigateToChannel(channelId, guildId)
    }
    private fun handleDeepLink(intent: Intent?) {
        if (intent?.action != Intent.ACTION_VIEW) return
        val data = intent.data ?: return
        if (GlobalContext.get().get<DiscordLinkHandler>().open(data.toString())) {
            intent.data = null
        }
    }
}
