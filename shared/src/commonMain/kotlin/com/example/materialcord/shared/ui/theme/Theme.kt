package com.example.materialcord.shared.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import com.materialkolor.DynamicMaterialTheme

@Composable
fun MaterialcordTheme(
    seedColor: Color = Color(0xFF5865F2), // Discord Blurple
    useDarkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit
) {
    DynamicMaterialTheme(
        primary = seedColor,
        isDark = useDarkTheme,
        content = content
    )
}
