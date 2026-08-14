@file:Suppress("ktlint:standard:max-line-length")

package me.lampu.lampcord.shared.ui.icons

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.PathFillType
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.unit.dp

// Status glyphs ported verbatim from Discord's Android XML vectors
// (discord-jadx res/drawable-anydpi-v24/ic_status_*_16dp.xml and res/drawable/ic_mobile.xml).
// The EvenOdd fill keeps the inner shapes (crescent, DND dash, streaming play, phone cutout,
// offline ring) as real cutouts so the status background shows through.
object StatusIcons {
    val Online: ImageVector by lazy {
        statusVector(
            name = "Status.Online",
            pathData = "M4,4m-4,0a4,4 0,1 1,8 0a4,4 0,1 1,-8 0",
            fill = Color(0xFF3BA55C),
        )
    }

    val Idle: ImageVector by lazy {
        statusVector(
            name = "Status.Idle",
            pathData = "M2.00009,5C3.65695,5 5.00009,3.65685 5.00009,2C5.00009,1.23515 4.71386,0.53714 4.24268,0.00726C6.33882,0.1327 8.00001,1.87237 8.00001,4.00002C8.00001,6.20916 6.20914,8.00002 4.00001,8.00002C1.87228,8.00002 0.13256,6.33872 0.00723,4.24248C0.53713,4.71373 1.23518,5 2.00009,5Z",
            fill = Color(0xFFFAA61A),
        )
    }

    val Dnd: ImageVector by lazy {
        statusVector(
            name = "Status.Dnd",
            pathData = "M4,8C6.20914,8 8,6.20914 8,4C8,1.79086 6.20914,0 4,0C1.79086,0 0,1.79086 0,4C0,6.20914 1.79086,8 4,8ZM2,3C1.44772,3 1,3.44772 1,4C1,4.55228 1.44772,5 2,5H6C6.55228,5 7,4.55228 7,4C7,3.44772 6.55228,3 6,3H2Z",
            fill = Color(0xFFED4245),
        )
    }

    val Streaming: ImageVector by lazy {
        statusVector(
            name = "Status.Streaming",
            pathData = "M4,8C6.20914,8 8,6.20914 8,4C8,1.79086 6.20914,0 4,0C1.79086,0 0,1.79086 0,4C0,6.20914 1.79086,8 4,8ZM3,5.73205L6,4L3,2.26795V5.73205Z",
            fill = Color(0xFF593A93),
        )
    }

    val Mobile: ImageVector by lazy {
        statusVector(
            name = "Status.Mobile",
            pathData = "M0,1.5C0,0.6716 0.6716,0 1.5,0H6.5C7.3284,0 8,0.6716 8,1.5V10.5C8,11.3284 7.3284,12 6.5,12H1.5C0.6716,12 0,11.3284 0,10.5V1.5ZM5,10C5,10.5523 4.5523,11 4,11C3.4477,11 3,10.5523 3,10C3,9.4477 3.4477,9 4,9C4.5523,9 5,9.4477 5,10ZM7,2H1V8H7V2Z",
            fill = Color(0xFF3BA55C),
            viewportWidth = 8f,
            viewportHeight = 12f,
        )
    }

    val Invisible: ImageVector by lazy {
        statusVector(
            name = "Status.Invisible",
            pathData = "M4,4m-3,0a3,3 0,1 1,6 0a3,3 0,1 1,-6 0",
            fill = null,
            stroke = Color(0xFF747F8D),
            strokeWidth = 2f,
        )
    }

    fun fromStatus(status: String): ImageVector = when (status) {
        "online" -> Online
        "idle" -> Idle
        "dnd" -> Dnd
        "streaming" -> Streaming
        "mobile" -> Mobile
        else -> Invisible
    }
}

private fun statusVector(
    name: String,
    pathData: String,
    fill: Color?,
    stroke: Color? = null,
    strokeWidth: Float = 0f,
    viewportWidth: Float = 8f,
    viewportHeight: Float = 8f,
): ImageVector =
    ImageVector.Builder(
        name = name,
        defaultWidth = 16.dp,
        defaultHeight = 16.dp * (viewportHeight / viewportWidth),
        viewportWidth = viewportWidth,
        viewportHeight = viewportHeight,
    ).addPath(
        pathData = PathParser().parsePathString(pathData).toNodes(),
        pathFillType = PathFillType.EvenOdd,
        fill = fill?.let { SolidColor(it) },
        stroke = stroke?.let { SolidColor(it) },
        strokeLineWidth = strokeWidth,
    ).build()
