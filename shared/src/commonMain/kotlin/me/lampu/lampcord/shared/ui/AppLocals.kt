package me.lampu.lampcord.shared.ui

import androidx.compose.runtime.compositionLocalOf
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.User

val LocalCurrentUser = compositionLocalOf<User?> { null }
val LocalGuilds = compositionLocalOf<List<Guild>> { emptyList() }
