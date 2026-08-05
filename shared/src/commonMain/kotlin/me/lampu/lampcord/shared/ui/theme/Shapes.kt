package me.lampu.lampcord.shared.ui.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Shapes
import androidx.compose.ui.unit.dp

val LampcordShapes = Shapes(
    extraSmall = RoundedCornerShape(16.dp),
    small = RoundedCornerShape(20.dp),
    medium = RoundedCornerShape(24.dp),
    large = RoundedCornerShape(28.dp),
    extraLarge = RoundedCornerShape(32.dp),
)

object ComponentShapes {
    val CardShape = RoundedCornerShape(20.dp)
    val DialogShape = RoundedCornerShape(28.dp)
    val BottomSheetShape = RoundedCornerShape(topStart = 32.dp, topEnd = 32.dp)
    val ButtonShape = RoundedCornerShape(20.dp)
}
