package me.lampu.lampcord.shared.ui.components.members

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.ui.components.ShimmerBox

@Composable
fun MemberSkeleton() {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(44.dp)
            .padding(horizontal = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        ShimmerBox(
            modifier = Modifier.size(32.dp),
            shape = CircleShape
        )
        Spacer(modifier = Modifier.width(12.dp))
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            ShimmerBox(
                modifier = Modifier
                    .width(100.dp)
                    .height(14.dp),
                shape = RoundedCornerShape(7.dp)
            )
            ShimmerBox(
                modifier = Modifier
                    .width(60.dp)
                    .height(10.dp),
                shape = RoundedCornerShape(5.dp)
            )
        }
    }
}
