package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data class CustomProfile(
    @SerialName("u") val user_id: String? = null,
    @SerialName("c") val theme_colors: List<Int>? = null,
    @SerialName("b") val banner: String? = null,
    @SerialName("a") val accent_color: Int? = null,
    @SerialName("p") val avatar: String? = null
)

/**
 * UserBG lists every user with a banner (hundreds of thousands of IDs), so banner owners are kept
 * as a sorted array of snowflakes and [CustomProfile]s are only built for the user being looked up.
 */
class ClientProfileMapping(
    private val bannerUserIds: LongArray = LongArray(0),
    private val avatars: Map<String, String> = emptyMap()
) {
    operator fun get(userId: String): CustomProfile? {
        val hasBanner = userId.toLongOrNull()?.let { bannerUserIds.binarySearch(it) >= 0 } == true
        val avatar = avatars[userId]
        if (!hasBanner && avatar == null) return null
        return CustomProfile(
            user_id = userId,
            banner = if (hasBanner) "$USRBG_BANNER_BASE_URL$userId" else null,
            avatar = avatar
        )
    }

    companion object {
        const val USRBG_BANNER_BASE_URL = "https://usrbg.is-hardly.online/usrbg/v2/"
    }
}
