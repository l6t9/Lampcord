package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.*
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.utils.ColorUtils

@Composable
fun HsvColorPicker(
    initialColor: Color,
    onColorSelected: (Color) -> Unit,
    onDismiss: () -> Unit
) {
    val hsv = remember(initialColor) { ColorUtils.colorToHsv(initialColor) }
    var hue by remember { mutableStateOf(hsv[0]) }
    var saturation by remember { mutableStateOf(hsv[1]) }
    var value by remember { mutableStateOf(hsv[2]) }

    val currentColor = remember(hue, saturation, value) {
        ColorUtils.hsvToColor(hue, saturation, value)
    }

    Column(
        modifier = Modifier
            .width(320.dp)
            .clip(RoundedCornerShape(16.dp))
            .background(MaterialTheme.colorScheme.surfaceVariant)
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .height(240.dp),
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Box(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxHeight()
                    .clip(RoundedCornerShape(4.dp))
                    .pointerInput(hue) {
                        detectDragGestures(
                            onDragStart = { offset ->
                                saturation = (offset.x / size.width).coerceIn(0f, 1f)
                                value = (1f - (offset.y / size.height)).coerceIn(0f, 1f)
                            }
                        ) { change, _ ->
                            saturation = (change.position.x / size.width).coerceIn(0f, 1f)
                            value = (1f - (change.position.y / size.height)).coerceIn(0f, 1f)
                        }
                    }
            ) {
                Canvas(modifier = Modifier.fillMaxSize()) {
                    drawRect(
                        brush = Brush.horizontalGradient(
                            colors = listOf(Color.White, ColorUtils.hsvToColor(hue, 1f, 1f))
                        )
                    )
                    drawRect(
                        brush = Brush.verticalGradient(
                            colors = listOf(Color.Transparent, Color.Black)
                        )
                    )

                    val selectorX = saturation * size.width
                    val selectorY = (1f - value) * size.height
                    drawCircle(
                        color = Color.White,
                        radius = 6.dp.toPx(),
                        center = Offset(selectorX, selectorY),
                        style = Stroke(width = 2.dp.toPx())
                    )
                    drawCircle(
                        color = Color.Black,
                        radius = 7.dp.toPx(),
                        center = Offset(selectorX, selectorY),
                        style = Stroke(width = 1.dp.toPx())
                    )
                }
            }

            Box(
                modifier = Modifier
                    .width(24.dp)
                    .fillMaxHeight()
                    .clip(RoundedCornerShape(4.dp))
                    .pointerInput(Unit) {
                        detectDragGestures(
                            onDragStart = { offset ->
                                hue = (1f - (offset.y / size.height)).coerceIn(0f, 1f) * 360f
                            }
                        ) { change, _ ->
                            hue = (1f - (change.position.y / size.height)).coerceIn(0f, 1f) * 360f
                        }
                    }
            ) {
                Canvas(modifier = Modifier.fillMaxSize()) {
                    val hueColors = listOf(
                        Color.Red, Color.Magenta, Color.Blue, Color.Cyan, Color.Green, Color.Yellow, Color.Red
                    )
                    drawRect(
                        brush = Brush.verticalGradient(
                            colors = hueColors.reversed()
                        )
                    )

                    val selectorY = (1f - (hue / 360f)) * size.height
                    drawRect(
                        color = Color.White,
                        topLeft = Offset(-2.dp.toPx(), selectorY - 1.dp.toPx()),
                        size = androidx.compose.ui.geometry.Size(size.width + 4.dp.toPx(), 2.dp.toPx()),
                        style = Stroke(width = 2.dp.toPx())
                    )
                    drawRect(
                        color = Color.Black,
                        topLeft = Offset(-2.dp.toPx(), selectorY - 1.dp.toPx()),
                        size = androidx.compose.ui.geometry.Size(size.width + 4.dp.toPx(), 2.dp.toPx())
                    )
                }
            }
        }

        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Box(
                modifier = Modifier
                    .size(48.dp)
                    .clip(CircleShape)
                    .background(currentColor)
                    .border(2.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.2f), CircleShape)
            )

            var textValue by remember(currentColor) {
                mutableStateOf("#" + (currentColor.value shr 32).toString(16).substring(2).uppercase())
            }
            
            // Wait, Color.value in Compose is ULong. Let's just manually format.
            val hexString = remember(currentColor) {
                val r = (currentColor.red * 255).toInt().toString(16).padStart(2, '0')
                val g = (currentColor.green * 255).toInt().toString(16).padStart(2, '0')
                val b = (currentColor.blue * 255).toInt().toString(16).padStart(2, '0')
                "#$r$g$b".uppercase()
            }

            Surface(
                modifier = Modifier.weight(1f).height(48.dp),
                shape = RoundedCornerShape(16.dp),
                color = MaterialTheme.colorScheme.surface,
                border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.2f))
            ) {
                Box(
                    modifier = Modifier.fillMaxSize().padding(horizontal = 12.dp),
                    contentAlignment = Alignment.CenterStart
                ) {
                    var localText by remember(hexString) { mutableStateOf(hexString) }
                    BasicTextField(
                        value = localText,
                        onValueChange = {
                            localText = it
                            if (it.matches(Regex("^#[0-9a-fA-F]{6}$"))) {
                                try {
                                    val newColor = Color(it.removePrefix("#").toLong(16) or 0xFF000000)
                                    val newHsv = ColorUtils.colorToHsv(newColor)
                                    hue = newHsv[0]
                                    saturation = newHsv[1]
                                    value = newHsv[2]
                                } catch (_: Exception) {}
                            }
                        },
                        modifier = Modifier.fillMaxWidth(),
                        textStyle = MaterialTheme.typography.bodyLarge.copy(
                            color = MaterialTheme.colorScheme.onSurface,
                            fontWeight = FontWeight.Bold,
                            fontFamily = FontFamily.Monospace
                        ),
                        cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                        singleLine = true
                    )
                }
            }
        }

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            TextButton(onClick = { /* Presets TODO */ }) {
                Text("Presets")
            }

            TextButton(onClick = {
                val resetHsv = ColorUtils.colorToHsv(initialColor)
                hue = resetHsv[0]
                saturation = resetHsv[1]
                value = resetHsv[2]
            }) {
                Text("Reset")
            }

            Button(
                onClick = { onColorSelected(currentColor) },
                shape = RoundedCornerShape(16.dp)
            ) {
                Text("Select")
            }
        }
    }
}
