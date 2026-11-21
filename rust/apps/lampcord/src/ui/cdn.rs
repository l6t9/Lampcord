use lampcord_model::snowflake::Snowflake;

pub fn normalize_size(size: u32) -> u32 {
    match size {
        0..=16 => 16,
        17..=32 => 32,
        33..=64 => 64,
        65..=128 => 128,
        129..=256 => 256,
        257..=512 => 512,
        _ => 1024,
    }
}

pub fn guild_icon_url(guild_id: Snowflake, icon_hash: Option<&str>, size: u32) -> Option<String> {
    let hash = icon_hash?;
    if hash.is_empty() {
        return None;
    }
    let ext = if hash.starts_with("a_") {
        "webp"
    } else {
        "webp"
    };
    let size = normalize_size(size);
    Some(format!(
        "https://cdn.discordapp.com/icons/{guild_id}/{hash}.{ext}?size={size}"
    ))
}

pub fn user_avatar_url(user_id: Snowflake, avatar_hash: Option<&str>, size: u32) -> String {
    let size = normalize_size(size);
    match avatar_hash {
        Some(hash) if !hash.is_empty() => {
            let ext = if hash.starts_with("a_") {
                "webp"
            } else {
                "webp"
            };
            format!("https://cdn.discordapp.com/avatars/{user_id}/{hash}.{ext}?size={size}")
        }
        _ => default_avatar_url(user_id),
    }
}

pub fn default_avatar_url(user_id: Snowflake) -> String {
    // Discord default avatar index: (user_id >> 22) % 6
    let raw: u64 = user_id.into();
    let index = (raw >> 22) % 6;
    format!("https://cdn.discordapp.com/embed/avatars/{index}.png")
}

pub fn channel_icon_url(
    channel_id: Snowflake,
    icon_hash: Option<&str>,
    size: u32,
) -> Option<String> {
    let hash = icon_hash?;
    if hash.is_empty() {
        return None;
    }
    let ext = if hash.starts_with("a_") {
        "webp"
    } else {
        "webp"
    };
    let size = normalize_size(size);
    Some(format!(
        "https://cdn.discordapp.com/channel-icons/{channel_id}/{hash}.{ext}?size={size}"
    ))
}
