package me.lampu.lampcord.shared.utils

import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.font.FontStyle

actual fun loadFont(
    path: String,
    weight: FontWeight,
    style: FontStyle
): Font {
    val file = java.io.File(path)
    return if (file.exists()) {
        androidx.compose.ui.text.platform.Font(file, weight, style)
    } else {
        androidx.compose.ui.text.platform.Font(path, weight, style)
    }
}
