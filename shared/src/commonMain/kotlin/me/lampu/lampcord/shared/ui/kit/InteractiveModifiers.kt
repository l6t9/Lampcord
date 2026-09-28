package me.lampu.lampcord.shared.ui.kit

import androidx.compose.foundation.Indication
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.PointerIcon
import androidx.compose.ui.input.pointer.pointerHoverIcon
import androidx.compose.ui.semantics.Role

@OptIn(ExperimentalComposeUiApi::class)
fun Modifier.handCursor(): Modifier = pointerHoverIcon(PointerIcon.Hand)

// Compose Multiplatform's `clickable` never sets a pointer icon, so on desktop the cursor
@OptIn(ExperimentalComposeUiApi::class)
fun Modifier.clickableCursor(
    enabled: Boolean = true,
    onClickLabel: String? = null,
    role: Role? = null,
    onClick: () -> Unit
): Modifier = pointerHoverIcon(PointerIcon.Hand)
    .clickable(enabled = enabled, onClickLabel = onClickLabel, role = role, onClick = onClick)

@OptIn(ExperimentalComposeUiApi::class)
fun Modifier.clickableCursor(
    interactionSource: MutableInteractionSource,
    indication: Indication?,
    enabled: Boolean = true,
    onClickLabel: String? = null,
    role: Role? = null,
    onClick: () -> Unit
): Modifier = pointerHoverIcon(PointerIcon.Hand)
    .clickable(
        interactionSource = interactionSource,
        indication = indication,
        enabled = enabled,
        onClickLabel = onClickLabel,
        role = role,
        onClick = onClick
    )

@OptIn(ExperimentalComposeUiApi::class)
fun Modifier.combinedClickableCursor(
    interactionSource: MutableInteractionSource,
    indication: Indication?,
    enabled: Boolean = true,
    onClickLabel: String? = null,
    role: Role? = null,
    onLongClickLabel: String? = null,
    onLongClick: (() -> Unit)? = null,
    onDoubleClick: (() -> Unit)? = null,
    onClick: () -> Unit
): Modifier = pointerHoverIcon(PointerIcon.Hand)
    .combinedClickable(
        interactionSource = interactionSource,
        indication = indication,
        enabled = enabled,
        onClickLabel = onClickLabel,
        role = role,
        onLongClickLabel = onLongClickLabel,
        onLongClick = onLongClick,
        onDoubleClick = onDoubleClick,
        onClick = onClick
    )

@OptIn(ExperimentalComposeUiApi::class)
fun Modifier.combinedClickableCursor(
    enabled: Boolean = true,
    onClickLabel: String? = null,
    role: Role? = null,
    onLongClickLabel: String? = null,
    onLongClick: (() -> Unit)? = null,
    onDoubleClick: (() -> Unit)? = null,
    onClick: () -> Unit
): Modifier = pointerHoverIcon(PointerIcon.Hand)
    .combinedClickable(
        enabled = enabled,
        onClickLabel = onClickLabel,
        role = role,
        onLongClickLabel = onLongClickLabel,
        onLongClick = onLongClick,
        onDoubleClick = onDoubleClick,
        onClick = onClick
    )

// square behind them on desktop hover. Use this where the target is an image rather than
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun Modifier.pointerClickable(
    enabled: Boolean = true,
    onClick: () -> Unit
): Modifier = this
    .pointerHoverIcon(PointerIcon.Hand)
    .clickable(
        interactionSource = remember { MutableInteractionSource() },
        indication = null,
        enabled = enabled,
        onClick = onClick
    )
