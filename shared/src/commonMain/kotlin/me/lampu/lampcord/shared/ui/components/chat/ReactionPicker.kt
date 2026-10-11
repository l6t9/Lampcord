package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.rememberBottomSheetState
import androidx.compose.material3.SheetValue
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Emoji
import me.lampu.lampcord.shared.ui.components.DiscordBottomSheet
import me.lampu.lampcord.shared.ui.components.EmojiPicker
import me.lampu.lampcord.shared.ui.components.rememberDiscordSheetState

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ReactionPickerSheet(
    onDismiss: () -> Unit,
    onEmojiSelected: (Emoji) -> Unit
) {
    val sheetState = rememberDiscordSheetState()
    
    DiscordBottomSheet(
        fillHeight = true,
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        modifier = Modifier.fillMaxWidth()
    ) {
        EmojiPicker(
            fillAvailableHeight = true,
            emojisOnly = true,
            onEmojiSelected = { emoji ->
                onEmojiSelected(emoji)
                onDismiss()
            }
        )
    }
}
