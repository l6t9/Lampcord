package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable

@Serializable
data class Sticker(
    val id: String,
    val pack_id: String? = null,
    val name: String,
    val description: String? = null,
    val tags: String? = null,
    val type: Int,
    val format_type: Int,
    val available: Boolean? = null,
    val guild_id: String? = null,
    val user: User? = null,
    val sort_value: Int? = null
)

@Serializable
data class StickerItem(
    val id: String,
    val name: String,
    val format_type: Int
)

@Serializable
data class StickerPack(
    val id: String,
    val stickers: List<Sticker>,
    val name: String,
    val sku_id: String,
    val cover_sticker_id: String? = null,
    val description: String? = null,
    val banner_asset_id: String? = null
)

@Serializable
data class StickerPackStoreListing(
    val id: String,
    val sku: Sku,
    val description: String,
    val unpublished_at: String? = null
)

@Serializable
data class StickerStoreDirectory(
    val sticker_packs: List<StickerPack>
)
