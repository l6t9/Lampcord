package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay
import me.lampu.lampcord.shared.api.MediaApi
import me.lampu.lampcord.shared.model.Gif
import me.lampu.lampcord.shared.model.GifCategory
import kotlin.time.Duration.Companion.milliseconds

@Composable
fun GifPicker(
    query: String,
    onQueryChange: (String) -> Unit,
    mediaApi: MediaApi,
    onGifSelected: (Gif) -> Unit
) {
    var gifResults by remember { mutableStateOf<List<Gif>>(emptyList()) }
    var categories by remember { mutableStateOf<List<GifCategory>>(emptyList()) }
    var isLoading by remember { mutableStateOf(false) }
    var isCategoryMode by remember { mutableStateOf(false) }

    LaunchedEffect(query) {
        isLoading = true
        if (query.isBlank()) {
            categories = mediaApi.getTrendingGifCategories()?.categories ?: emptyList()
            gifResults = emptyList()
            isCategoryMode = false
        } else if (isCategoryMode) {
            gifResults = mediaApi.getTrendingGifCategory(query)
            categories = emptyList()
        } else {
            delay(500.milliseconds)
            gifResults = mediaApi.searchGifs(query)
            categories = emptyList()
            isCategoryMode = false
        }
        isLoading = false
    }

    if (isLoading) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            ContainedLoadingIndicator()
        }
    } else {
        if (query.isBlank()) {
            LazyVerticalGrid(
                columns = GridCells.Fixed(2),
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(4.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                items(categories) { category ->
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .aspectRatio(1.8f)
                            .clip(RoundedCornerShape(8.dp))
                            .clickable { 
                                isCategoryMode = true
                                onQueryChange(category.name) 
                            }
                    ) {
                        AsyncImage(
                            model = category.src,
                            contentDescription = category.name,
                            modifier = Modifier.fillMaxSize(),
                            contentScale = ContentScale.Crop
                        )
                        Box(
                            modifier = Modifier
                                .fillMaxSize()
                                .background(Color.Black.copy(alpha = 0.3f)),
                            contentAlignment = Alignment.BottomStart
                        ) {
                            Text(
                                text = category.name,
                                color = Color.White,
                                fontWeight = FontWeight.Bold,
                                modifier = Modifier.padding(8.dp),
                                style = MaterialTheme.typography.labelLarge
                            )
                        }
                    }
                }
            }
        } else {
            LazyVerticalGrid(
                columns = GridCells.Fixed(2),
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(4.dp),
                horizontalArrangement = Arrangement.spacedBy(4.dp),
                verticalArrangement = Arrangement.spacedBy(4.dp)
            ) {
                items(gifResults) { gif ->
                    AsyncImage(
                        model = gif.url,
                        contentDescription = null,
                        modifier = Modifier
                            .fillMaxWidth()
                            .aspectRatio(1.5f)
                            .clip(RoundedCornerShape(8.dp))
                            .clickable { onGifSelected(gif) },
                        contentScale = ContentScale.Crop
                    )
                }
            }
        }
    }
}
