use chrono::{DateTime, Datelike, Local, TimeZone, Timelike, Utc};
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering;
use std::time::SystemTime;
use std::time;

pub use crate::models::id::DISCORD_EPOCH_MS;

pub fn month_long(month0: u32) -> String {
    requiem_i18n::text(&format!("time-month-long-{}", month0 + 1))
}

pub fn month_short(month0: u32) -> String {
    requiem_i18n::text(&format!("time-month-short-{}", month0 + 1))
}

pub fn months_long() -> Vec<String> {
    (0..12).map(month_long).collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeFormat {
    #[default]
    Auto,
    Hour12,
    Hour24,
}

impl TimeFormat {
    pub fn key(self) -> &'static str {
        match self {
            TimeFormat::Auto => "auto",
            TimeFormat::Hour12 => "12h",
            TimeFormat::Hour24 => "24h",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        [TimeFormat::Auto, TimeFormat::Hour12, TimeFormat::Hour24]
            .into_iter()
            .find(|f| f.key() == key)
    }
}

static TIME_FORMAT: AtomicU8 = AtomicU8::new(0);

pub fn set_time_format(format: TimeFormat) {
    TIME_FORMAT.store(format as u8, Ordering::Relaxed);
}

fn time_format() -> TimeFormat {
    match TIME_FORMAT.load(Ordering::Relaxed) {
        1 => TimeFormat::Hour12,
        2 => TimeFormat::Hour24,
        _ => TimeFormat::Auto,
    }
}

pub fn clock<T: Timelike>(time: &T) -> String {
    let (is_pm, hour12) = time.hour12();
    let key = match time_format() {
        TimeFormat::Auto => requiem_i18n::key!("time-clock"),
        TimeFormat::Hour12 => requiem_i18n::key!("time-clock-12h"),
        TimeFormat::Hour24 => requiem_i18n::key!("time-clock-24h"),
    };
    requiem_i18n::tr(
        key,
        &requiem_i18n::Args::new()
            .with("hour12", hour12.to_string())
            .with("hour24", format!("{:02}", time.hour()))
            .with("minute", format!("{:02}", time.minute()))
            .with(
                "meridiem",
                requiem_i18n::text(if is_pm {
                    requiem_i18n::key!("time-meridiem-pm")
                } else {
                    requiem_i18n::key!("time-meridiem-am")
                }),
            ),
    )
}

pub fn date_medium<D: Datelike>(date: &D) -> String {
    requiem_i18n::t!(
        "time-date-medium",
        month = month_short(date.month0()),
        day = date.day().to_string(),
        year = date.year().to_string()
    )
}

pub fn date_long<D: Datelike>(date: &D) -> String {
    requiem_i18n::t!(
        "time-date-long",
        month = month_long(date.month0()),
        day = date.day().to_string(),
        year = date.year().to_string()
    )
}

pub fn month_year_long<D: Datelike>(date: &D) -> String {
    requiem_i18n::t!(
        "time-month-year-long",
        month = month_long(date.month0()),
        year = date.year().to_string()
    )
}

pub fn date_numeric<D: Datelike>(date: &D) -> String {
    requiem_i18n::t!(
        "time-date-numeric",
        month = date.month().to_string(),
        day = date.day().to_string(),
        year = format!("{:02}", date.year().rem_euclid(100))
    )
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

pub fn time_of_day_ms() -> u64 {
    now_ms() % 86_400_000
}

pub fn current_year() -> u32 {
    Utc::now().year() as u32
}

pub fn snowflake_at(unix_ms: i64) -> u64 {
    ((unix_ms - DISCORD_EPOCH_MS).max(0) as u64) << 22
}

pub fn snowflake_days_ago(days: u64) -> u64 {
    snowflake_at(now_ms().saturating_sub(days * 86_400_000) as i64)
}

pub fn clock_label() -> String {
    clock(&Local::now())
}

pub fn date_label(unix_ms: i64) -> String {
    Local
        .timestamp_millis_opt(unix_ms)
        .single()
        .map(|d| date_medium(&d))
        .unwrap_or_default()
}

pub fn snowflake_month_year(id: u64) -> String {
    let ms = ((id >> 22) as i64) + DISCORD_EPOCH_MS;
    DateTime::<Utc>::from_timestamp_millis(ms)
        .map(|d| {
            requiem_i18n::t!(
                "time-month-year-short",
                month = month_short(d.month0()),
                year = d.year().to_string()
            )
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_follows_the_time_format() {
        let t = chrono::NaiveTime::from_hms_opt(17, 5, 0).unwrap();
        set_time_format(TimeFormat::Hour24);
        assert_eq!(clock(&t), "17:05");
        set_time_format(TimeFormat::Hour12);
        assert_eq!(clock(&t), "5:05 PM");
        set_time_format(TimeFormat::Auto);
        assert_eq!(clock(&t), "5:05 PM");
    }

    #[test]
    fn snowflake_round_trip() {
        let ms = 1_700_000_000_000;
        assert_eq!(((snowflake_at(ms) >> 22) as i64) + DISCORD_EPOCH_MS, ms);
    }

    #[test]
    fn month_year_of_a_known_snowflake() {
        assert_eq!(snowflake_month_year(0), "Jan 2015");
        assert_eq!(
            snowflake_month_year(snowflake_at(1_784_246_400_000)),
            "Jul 2026"
        );
    }
}
