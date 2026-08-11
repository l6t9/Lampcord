package me.lampu.lampcord.shared.utils

import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.font.FontStyle

actual fun loadFont(
    path: String,
    weight: FontWeight,
    style: FontStyle
): Font = if (path.startsWith("/") || path.startsWith("content://")) {
    Font(java.io.File(path), weight, style)
} else {
    Font(path, AndroidContextProvider.applicationContext.assets, weight, style)
}
