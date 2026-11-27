use crate::utils::voice_settings;

pub fn is_muted(user_id: u64) -> bool {
    voice_settings::get().local_mutes.contains(&user_id)
}

pub fn set_muted(user_id: u64, muted: bool) {
    voice_settings::update(|s| {
        s.local_mutes.retain(|u| *u != user_id);
        if muted {
            s.local_mutes.push(user_id);
        }
    });
}

pub fn volume_percent(user_id: u64) -> f32 {
    voice_settings::get()
        .local_volumes
        .iter()
        .find(|(u, _)| *u == user_id)
        .map_or(100.0, |(_, v)| *v)
}

pub fn volume(user_id: u64) -> f32 {
    volume_percent(user_id) / 100.0
}

pub fn set_volume_percent(user_id: u64, percent: f32) {
    voice_settings::update(|s| {
        s.local_volumes.retain(|(u, _)| *u != user_id);
        if (percent - 100.0).abs() > f32::EPSILON {
            s.local_volumes.push((user_id, percent.clamp(0.0, 200.0)));
        }
    });
}
