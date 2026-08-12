package me.lampu.lampcord

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import me.lampu.lampcord.shared.ui.BubbleScreen

class BubbleActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)

        val channelId = intent.getStringExtra(NotificationHelper.EXTRA_CHANNEL_ID)
        if (channelId == null) {
            finish()
            return
        }
        val guildId = intent.getStringExtra(NotificationHelper.EXTRA_GUILD_ID)

        setContent {
            BubbleScreen(channelId = channelId, guildId = guildId)
        }
    }
}
