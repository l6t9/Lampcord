package me.lampu.lampcord.shared.model

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector

data class AutocompleteItem(
    val id: String,
    val title: String,
    val subtitle: String? = null,
    val icon: String? = null,
    val iconType: ImageVector? = null,
    val replacement: String,
    val searchReplacement: String? = null,
    val isCommand: Boolean = false,
    val commandObj: ApplicationCommand? = null,
    val color: Color? = null
)
