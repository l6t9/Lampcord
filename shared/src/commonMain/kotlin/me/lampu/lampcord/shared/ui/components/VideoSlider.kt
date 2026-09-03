package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.geometry.lerp
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun VideoSlider(
    value: Float,
    onValueChange: (Float) -> Unit,
    onValueChangeFinished: () -> Unit,
    bufferedFraction: Float,
    modifier: Modifier = Modifier,
) {
    Slider(
        value = value,
        onValueChange = onValueChange,
        onValueChangeFinished = onValueChangeFinished,
        modifier = modifier.height(32.dp),
        thumb = { Spacer(Modifier.size(0.dp)) },
        track = { sliderState ->
            PlayerSliderTrack(
                sliderState = sliderState,
                bufferedFraction = bufferedFraction,
                trackHeight = 16.dp, // Shorter as requested
                colors = SliderDefaults.colors(
                    activeTrackColor = MaterialTheme.colorScheme.primary,
                    inactiveTrackColor = Color.White.copy(alpha = 0.24f)
                )
            )
        }
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PlayerSliderTrack(
    sliderState: SliderState,
    modifier: Modifier = Modifier,
    colors: SliderColors = SliderDefaults.colors(),
    trackHeight: Dp = 16.dp,
    bufferedFraction: Float = 0f,
    bufferedColor: Color = colors.activeTrackColor.copy(alpha = 0.38f),
    enabled: Boolean = true,
) {
    val inactiveTrackColor = if (enabled) colors.inactiveTrackColor else colors.disabledInactiveTrackColor
    val activeTrackColor = if (enabled) colors.activeTrackColor else colors.disabledActiveTrackColor
    val inactiveTickColor = if (enabled) colors.inactiveTickColor else colors.disabledInactiveTickColor
    val activeTickColor = if (enabled) colors.activeTickColor else colors.disabledActiveTickColor
    val valueRange = sliderState.valueRange
    val hasValidRange =
        valueRange.start.isFinite() &&
            valueRange.endInclusive.isFinite() &&
            valueRange.endInclusive > valueRange.start
    Canvas(
        modifier
            .fillMaxWidth()
            .height(trackHeight),
    ) {
        drawTrack(
            stepsToTickFractions(sliderState.steps),
            0f,
            if (hasValidRange) {
                calcFraction(
                    valueRange.start,
                    valueRange.endInclusive,
                    sliderState.value.coerceIn(valueRange.start, valueRange.endInclusive),
                )
            } else {
                0f
            },
            bufferedFraction.coerceIn(0f, 1f),
            inactiveTrackColor,
            bufferedColor,
            activeTrackColor,
            inactiveTickColor,
            activeTickColor,
            trackHeight,
        )
    }
}

private fun DrawScope.drawTrack(
    tickFractions: FloatArray,
    activeRangeStart: Float,
    activeRangeEnd: Float,
    bufferedFraction: Float,
    inactiveTrackColor: Color,
    bufferedTrackColor: Color,
    activeTrackColor: Color,
    inactiveTickColor: Color,
    activeTickColor: Color,
    trackHeight: Dp = 16.dp,
) {
    val isRtl = layoutDirection == LayoutDirection.Rtl
    val trackWidth = size.width
    val trackHeightPx = trackHeight.toPx()
    val cornerRadius = CornerRadius(4.dp.toPx()) // Proportional rounding

    // Inactive track (background)
    drawRoundRect(
        color = inactiveTrackColor,
        topLeft = Offset(0f, center.y - (trackHeightPx / 2f)),
        size = Size(trackWidth, trackHeightPx),
        cornerRadius = cornerRadius
    )

    if (bufferedFraction > 0f) {
        val bufferedWidth = trackWidth * bufferedFraction
        drawRoundRect(
            color = bufferedTrackColor,
            topLeft = Offset(if (isRtl) trackWidth - bufferedWidth else 0f, center.y - (trackHeightPx / 2f)),
            size = Size(bufferedWidth, trackHeightPx),
            cornerRadius = cornerRadius
        )
    }

    val activeWidth = trackWidth * (activeRangeEnd - activeRangeStart)
    val activeStart = trackWidth * activeRangeStart
    drawRoundRect(
        color = activeTrackColor,
        topLeft = Offset(if (isRtl) trackWidth - activeStart - activeWidth else activeStart, center.y - (trackHeightPx / 2f)),
        size = Size(activeWidth, trackHeightPx),
        cornerRadius = cornerRadius
    )

    val tickSize = 2.0.dp.toPx()
    for (tick in tickFractions) {
        val outsideFraction = (tick !in activeRangeStart..activeRangeEnd)
        val sliderStart = if (isRtl) trackWidth else 0f
        val sliderEnd = if (isRtl) 0f else trackWidth
        drawCircle(
            color = if (outsideFraction) inactiveTickColor else activeTickColor,
            center = Offset(lerp(Offset(sliderStart, center.y), Offset(sliderEnd, center.y), tick).x, center.y),
            radius = tickSize / 2f,
        )
    }
}

private fun stepsToTickFractions(steps: Int): FloatArray =
    if (steps == 0) {
        floatArrayOf()
    } else {
        FloatArray(steps + 2) {
            it.toFloat() / (steps + 1)
        }
    }

private fun calcFraction(
    a: Float,
    b: Float,
    pos: Float,
) = (if (b - a == 0f) 0f else (pos - a) / (b - a)).coerceIn(0f, 1f)
