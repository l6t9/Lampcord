package me.lampu.lampcord

import androidx.compose.animation.animateColorAsState
import androidx.compose.runtime.*
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.graphics.vector.rememberVectorPainter
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import me.lampu.lampcord.shared.di.appModule
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.App
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.ui.WaylandDensityProvider
import me.lampu.lampcord.utils.WaylandScale
import org.koin.compose.koinInject
import org.koin.core.context.startKoin

fun main() {
    WaylandScale.detectAndApply()
    
    startKoin {
        modules(appModule)
    }

    application {
        val chatState: ChatState = koinInject()
        
        val seedColorString = chatState.settingsStore.accentColor
        val targetSeedColor = remember(seedColorString) {
            try {
                Color(seedColorString.removePrefix("#").toLong(16) or 0xFF000000)
            } catch (_: Exception) {
                Color(0xFF6750A4)
            }
        }
        
        val seedColor by animateColorAsState(targetSeedColor)
        
        val discordPainter = rememberVectorPainter(Icons.Brand.Discord)
        val dynamicIcon = remember(seedColor) {
            object : Painter() {
                override val intrinsicSize: Size = Size(256f, 256f)
                override fun DrawScope.onDraw() {
                    val cornerRadius = size.width * 0.25f
                    drawRoundRect(
                        color = seedColor,
                        size = size,
                        cornerRadius = CornerRadius(cornerRadius, cornerRadius)
                    )
                    with(discordPainter) {
                        val logoSize = size * 0.62f
                        val offset = Offset((size.width - logoSize.width) / 2f, (size.height - logoSize.height) / 2f)
                        translate(offset.x, offset.y) {
                            draw(logoSize, colorFilter = ColorFilter.tint(Color.White))
                        }
                    }
                }
            }
        }

        Window(
            onCloseRequest = ::exitApplication, 
            title = "Lampcord",
            icon = dynamicIcon
        ) {
            WaylandDensityProvider {
                App()
            }
        }
    }
}
