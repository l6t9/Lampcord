package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.ui.icons.Icons

@OptIn(ExperimentalMaterial3ExpressiveApi::class, ExperimentalMaterial3Api::class)
@Composable
fun ExpressiveTooltip(
    modifier: Modifier = Modifier,
    tooltipString: String? = null,
    anchorPosition: TooltipAnchorPosition = TooltipAnchorPosition.End,
    maxWidth: Int = 200,
    content: @Composable () -> Unit,
    interactionSource: MutableInteractionSource? = null,
    anchor: @Composable BoxScope.() -> Unit = {}
) {
    val hoverSource = interactionSource ?: remember { MutableInteractionSource() }
    val isHovered by hoverSource.collectIsHoveredAsState()
    val tooltipState = rememberTooltipState()

    TooltipBox(
        modifier = modifier,
        positionProvider = TooltipDefaults.rememberTooltipPositionProvider(
            positioning = anchorPosition
        ),
        tooltip = {
            RichTooltip(
                caretShape = TooltipDefaults.caretShape(),
                maxWidth = maxWidth.dp
            ) {
                content()
            }
        },
        state = tooltipState,
    ) {
        Box(
            modifier = Modifier.hoverable(hoverSource),
            content = anchor
        )
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class, ExperimentalMaterial3Api::class)
@Composable
fun ExpressiveHoverTooltip(
    modifier: Modifier = Modifier,
    tooltipContent: @Composable () -> Unit,
) {
    val interactionSource = remember { MutableInteractionSource() }
    val isHovered by interactionSource.collectIsHoveredAsState()
    val tooltipState = rememberTooltipState()

    Box(
        modifier = modifier.hoverable(interactionSource),
    ) {
        if (isHovered) {
            TooltipBox(
                positionProvider = TooltipDefaults.rememberTooltipPositionProvider(
                    positioning = TooltipAnchorPosition.End
                ),
                tooltip = {
                    RichTooltip(
                        caretShape = TooltipDefaults.caretShape()
                    ) {
                        tooltipContent()
                    }
                },
                state = tooltipState,
            ) {
                 Box(Modifier.matchParentSize())
            }
        }
        if (isHovered) {
            Icon(
                imageVector = Icons.Filled.Info,
                contentDescription = "tooltip",
                modifier = Modifier.align(Alignment.TopEnd),
                tint = MaterialTheme.colorScheme.primary,
            )
        }
    }
}

@Composable
fun tooltipText(
    text: String,
    modifier: Modifier = Modifier,
) : @Composable () -> Unit {
    return {
        Text(text, modifier = modifier)
    }
}
