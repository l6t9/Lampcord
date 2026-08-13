package me.lampu.lampcord

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.ContextCompat
import me.lampu.lampcord.shared.state.ChannelNavigator
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.ui.App
import org.koin.core.context.GlobalContext

class MainActivity : ComponentActivity() {

    private val requestNotificationsPermission =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) { }

    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
        requestNotificationsPermissionIfNeeded()
        setContent {
            App()
        }
        handleNotificationIntent(intent)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        handleNotificationIntent(intent)
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
        val navigationStore = GlobalContext.get().get<NavigationStore>()
        navigationStore.isBubble = intent?.getBooleanExtra(NotificationHelper.EXTRA_IS_BUBBLE, false) == true
        if (intent == null) return
        intent.removeExtra(NotificationHelper.EXTRA_IS_BUBBLE)

        val channelId = intent.getStringExtra(NotificationHelper.EXTRA_CHANNEL_ID) ?: return
        val guildId = intent.getStringExtra(NotificationHelper.EXTRA_GUILD_ID)
        intent.removeExtra(NotificationHelper.EXTRA_CHANNEL_ID)
        intent.removeExtra(NotificationHelper.EXTRA_GUILD_ID)

        val navigator = GlobalContext.get().get<ChannelNavigator>()
        navigator.navigateToChannel(channelId, guildId)
    }
}
