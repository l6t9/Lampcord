pub fn file_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = [
        requiem_i18n::key!("format-unit-kb"),
        requiem_i18n::key!("format-unit-mb"),
        requiem_i18n::key!("format-unit-gb"),
        requiem_i18n::key!("format-unit-tb"),
    ];
    if bytes < 1024 {
        return requiem_i18n::t!("format-bytes", count = bytes);
    }
    let mut value = bytes as f64 / 1024.0;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    requiem_i18n::t!(
        "format-size",
        value = format!("{value:.2}"),
        unit = requiem_i18n::text(UNITS[unit])
    )
}

pub fn media_time(secs: f64) -> String {
    let secs = secs.max(0.0) as u64;
    format!("{}:{:02}", secs / 60, secs % 60)
}

pub fn clock_with_millis(ms: u64) -> String {
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        ms / 3_600_000,
        ms / 60_000 % 60,
        ms / 1000 % 60,
        ms % 1000
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        assert_eq!(file_size(10), "10 bytes");
        assert_eq!(file_size(300_000), "292.97 KB");
    }

    #[test]
    fn times() {
        assert_eq!(media_time(75.9), "1:15");
        assert_eq!(clock_with_millis(3_723_004), "01:02:03.004");
    }
}
