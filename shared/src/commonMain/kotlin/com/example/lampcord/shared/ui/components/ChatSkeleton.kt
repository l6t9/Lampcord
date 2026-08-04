package com.example.lampcord.shared.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun ChatSkeleton() {
    Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
        repeat(10) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = 8.dp),
                verticalAlignment = Alignment.Top
            ) {
                ShimmerBox(
                    modifier = Modifier.size(40.dp),
                    shape = CircleShape
                )
                Spacer(modifier = Modifier.width(12.dp))
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        ShimmerBox(
                            modifier = Modifier.width(100.dp).height(14.dp),
                            shape = RoundedCornerShape(7.dp)
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        ShimmerBox(
                            modifier = Modifier.width(60.dp).height(10.dp),
                            shape = RoundedCornerShape(5.dp)
                        )
                    }
                    ShimmerBox(
                        modifier = Modifier.fillMaxWidth(0.8f).height(12.dp),
                        shape = RoundedCornerShape(6.dp)
                    )
                    ShimmerBox(
                        modifier = Modifier.fillMaxWidth(0.5f).height(12.dp),
                        shape = RoundedCornerShape(6.dp)
                    )
                }
            }
        }
    }
}
