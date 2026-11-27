use chrono::{
    DateTime, Datelike, Duration, Local, LocalResult, Months, NaiveDate, NaiveTime, TimeZone,
    Timelike, Weekday,
};
use std::iter;

fn local(unix: i64) -> Option<DateTime<Local>> {
    DateTime::from_timestamp(unix, 0).map(|utc| utc.with_timezone(&Local))
}

pub fn format(unix: i64, letter: Option<char>) -> String {
    format_at(unix, letter, Local::now())
}

fn format_at(unix: i64, letter: Option<char>, now: DateTime<Local>) -> String {
    let Some(time) = local(unix) else {
        return unix.to_string();
    };
    let pattern = match letter.unwrap_or('f') {
        't' => "%-I:%M %p",
        'T' => "%-I:%M:%S %p",
        'd' => "%m/%d/%Y",
        'D' => "%B %-d, %Y",
        'f' => "%B %-d, %Y at %-I:%M %p",
        'F' => "%A, %B %-d, %Y at %-I:%M %p",
        's' => "%-m/%-d/%y, %-I:%M %p",
        'S' => "%-m/%-d/%y, %-I:%M:%S %p",
        'R' => return relative(time.timestamp() - now.timestamp()),
        _ => "%B %-d, %Y at %-I:%M %p",
    };
    time.format(pattern).to_string()
}

fn relative(seconds_from_now: i64) -> String {
    let seconds_from_now = seconds_from_now - 1;
    let future = seconds_from_now > 0;
    let s = seconds_from_now.abs();
    let (n, unit) = if s < 60 {
        (s.max(1), "second")
    } else if s < 3600 {
        ((s as f64 / 60.0).round() as i64, "minute")
    } else if s < 22 * 3600 {
        ((s as f64 / 3600.0).round() as i64, "hour")
    } else if s < 26 * 86400 {
        ((s as f64 / 86400.0).round() as i64, "day")
    } else if s < 320 * 86400 {
        ((s as f64 / (30.4375 * 86400.0)).round() as i64, "month")
    } else {
        ((s as f64 / (365.25 * 86400.0)).round() as i64, "year")
    };
    let amount = match (n, unit) {
        (1, "hour") => "an hour".to_string(),
        (1, "minute") => "a minute".to_string(),
        (1, "day") => "a day".to_string(),
        (1, "month") => "a month".to_string(),
        (1, "year") => "a year".to_string(),
        (n, unit) => format!("{n} {unit}{}", if n == 1 { "" } else { "s" }),
    };
    if future {
        format!("in {amount}")
    } else {
        format!("{amount} ago")
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parsed {
    pub time: DateTime<Local>,
    pub hour: bool,
    pub weekday: bool,

    pub day: bool,
    pub year: bool,
    pub second: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Unit {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

fn unit_of(word: &str) -> Option<Unit> {
    if word == "s" {
        return Some(Unit::Second);
    }
    Some(match word.trim_end_matches('s') {
        "second" | "sec" => Unit::Second,
        "minute" | "min" | "m" => Unit::Minute,
        "hour" | "hr" | "h" => Unit::Hour,
        "day" | "d" => Unit::Day,
        "week" | "wk" | "w" => Unit::Week,
        "month" | "mo" => Unit::Month,
        "year" | "yr" | "y" => Unit::Year,
        _ => return None,
    })
}

fn number_word(word: &str) -> Option<f64> {
    Some(match word {
        "a" | "an" | "one" => 1.0,
        "two" | "couple" => 2.0,
        "three" | "few" => 3.0,
        "four" => 4.0,
        "five" => 5.0,
        "six" => 6.0,
        "seven" => 7.0,
        "eight" => 8.0,
        "nine" => 9.0,
        "ten" => 10.0,
        "eleven" => 11.0,
        "twelve" | "dozen" => 12.0,
        "fifteen" => 15.0,
        "twenty" => 20.0,
        "thirty" => 30.0,
        "forty" => 40.0,
        "fifty" => 50.0,
        "sixty" => 60.0,
        "ninety" => 90.0,
        "half" => 0.5,
        _ => return None,
    })
}

fn words(query: &str) -> Vec<String> {
    let cleaned: String = query
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if matches!(c, ',' | ';' | '!' | '?') {
                ' '
            } else {
                c
            }
        })
        .collect();
    cleaned
        .split_whitespace()
        .filter_map(|w| match w {
            "at" | "@" => Some("@".to_string()),
            "on" | "the" | "of" | "by" | "around" | "about" | "for" | "is" | "be"
            | "approximately" | "approx" | "please" | "sometime" | "like" | "um" | "uh" => None,
            _ => Some(w.trim_end_matches('.').to_string()),
        })
        .flat_map(|w| split_compound(&w))
        .collect()
}

fn split_compound(word: &str) -> Vec<String> {
    let plain = word.chars().all(|c| c.is_ascii_alphanumeric() || c == '.');
    let mut starts = Vec::new();
    let mut previous_digit = false;
    for (at, c) in word.char_indices() {
        let digit = c.is_ascii_digit();
        if digit && !previous_digit && at > 0 && word[..at].chars().any(|p| p.is_ascii_alphabetic())
        {
            starts.push(at);
        }
        previous_digit = digit || c == '.';
    }
    if !plain || starts.is_empty() {
        return vec![word.to_string()];
    }
    let mut parts = Vec::new();
    let mut from = 0;
    for at in starts.into_iter().chain(iter::once(word.len())) {
        parts.push(word[from..at].to_string());
        from = at;
    }
    parts
}

fn amount_and_unit(words: &[String], i: usize) -> Option<(f64, Unit, usize)> {
    let word = words.get(i)?;

    let digits = word
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .count();
    if digits > 0 {
        let amount: f64 = word[..digits].parse().ok()?;
        if digits < word.len() {
            return unit_of(&word[digits..]).map(|u| (amount, u, 1));
        }
        return unit_of(words.get(i + 1)?).map(|u| (amount, u, 2));
    }

    if matches!(word.as_str(), "a" | "an")
        && matches!(words.get(i + 1).map(String::as_str), Some("couple" | "few"))
    {
        return amount_and_unit(words, i + 1).map(|(amount, unit, used)| (amount, unit, used + 1));
    }
    let amount = number_word(word)?;
    let mut next = i + 1;

    if amount == 0.5 && matches!(words.get(next).map(String::as_str), Some("a" | "an")) {
        next += 1;
    }
    let unit = unit_of(words.get(next)?)?;
    Some((amount, unit, next + 1 - i))
}

fn duration(words: &[String]) -> Option<(Vec<(f64, Unit)>, i32)> {
    let mut i = 0;
    let mut sign = 1;
    if words.first().map(String::as_str) == Some("in") {
        i = 1;
    }
    let mut terms: Vec<(f64, Unit)> = Vec::new();
    while i < words.len() {
        match words[i].as_str() {
            "and" => i += 1,
            "ago" => {
                sign = -1;
                i += 1;
            }
            "later" | "hence" => i += 1,
            "from" if words.get(i + 1).map(String::as_str) == Some("now") => i += 2,

            "a" if words.get(i + 1).map(String::as_str) == Some("half") && !terms.is_empty() => {
                let unit = terms.last()?.1;
                terms.push((0.5, unit));
                i += 2;
            }
            _ => {
                let (amount, unit, used) = amount_and_unit(words, i)?;
                terms.push((amount, unit));
                i += used;
            }
        }
    }
    (!terms.is_empty()).then_some((terms, sign))
}

fn shift(base: DateTime<Local>, terms: &[(f64, Unit)], sign: i32) -> Option<DateTime<Local>> {
    let mut time = base;
    for (amount, unit) in terms {
        let seconds_per = match unit {
            Unit::Second => 1.0,
            Unit::Minute => 60.0,
            Unit::Hour => 3600.0,
            Unit::Day => 86400.0,
            Unit::Week => 7.0 * 86400.0,
            Unit::Month | Unit::Year => 0.0,
        };
        if seconds_per > 0.0 {
            time += Duration::milliseconds(
                (amount * seconds_per * 1000.0).round() as i64 * i64::from(sign),
            );
        } else {
            let months_per = if *unit == Unit::Year { 12.0 } else { 1.0 };
            let months = amount * months_per;
            let whole = Months::new(months.trunc() as u32);
            time = if sign >= 0 {
                time.checked_add_months(whole)?
            } else {
                time.checked_sub_months(whole)?
            };
            time += Duration::seconds((months.fract() * 30.0 * 86400.0) as i64 * i64::from(sign));
        }
    }
    Some(time)
}

fn weekday(word: &str) -> Option<Weekday> {
    Some(match word {
        "mon" | "monday" => Weekday::Mon,
        "tue" | "tues" | "tuesday" => Weekday::Tue,
        "wed" | "weds" | "wednesday" => Weekday::Wed,
        "thu" | "thur" | "thurs" | "thursday" => Weekday::Thu,
        "fri" | "friday" => Weekday::Fri,
        "sat" | "saturday" => Weekday::Sat,
        "sun" | "sunday" => Weekday::Sun,
        _ => return None,
    })
}

fn month(word: &str) -> Option<u32> {
    const FULL: [&str; 12] = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];
    FULL.iter()
        .position(|name| {
            word == *name || word == &name[..3] || (word == "sept" && *name == "september")
        })
        .map(|i| i as u32 + 1)
}

fn at(date: NaiveDate, time: NaiveTime) -> Option<DateTime<Local>> {
    match Local.from_local_datetime(&date.and_time(time)) {
        LocalResult::Single(t) => Some(t),
        LocalResult::Ambiguous(first, _) => Some(first),
        LocalResult::None => None,
    }
}

fn hms(h: u32, m: u32, s: u32) -> Option<NaiveTime> {
    NaiveTime::from_hms_opt(h, m, s)
}

fn plain_hour(hour: u32) -> u32 {
    match hour {
        1..=6 => hour + 12,
        _ => hour,
    }
}

fn clock(words: &[String], i: usize) -> Option<(NaiveTime, usize, bool)> {
    let word = words.get(i)?.as_str();
    match word {
        "noon" => return hms(12, 0, 0).map(|t| (t, 1, false)),
        "midnight" => return hms(0, 0, 0).map(|t| (t, 1, false)),
        _ => {}
    }
    let after_at = i > 0 && words[i - 1] == "@";
    let (body, mut meridiem) = match word
        .strip_suffix("pm")
        .map(|b| (b, true))
        .or_else(|| word.strip_suffix("am").map(|b| (b, false)))
    {
        Some((b, pm)) => (b.trim_end_matches('.'), Some(pm)),
        None => (word, None),
    };
    let mut used = 1;
    if meridiem.is_none() {
        if let Some(next) = words
            .get(i + 1)
            .map(String::as_str)
            .filter(|w| matches!(*w, "am" | "pm" | "a.m" | "p.m"))
        {
            meridiem = Some(next.starts_with('p'));
            used = 2;
        }
    }
    let mut parts = body.split(':');
    let hour: u32 = parts.next()?.parse().ok()?;
    let minute: Option<u32> = parts.next().map(|m| m.parse().ok()).unwrap_or(Some(0));
    let second: Option<u32> = parts.next().map(|s| s.parse().ok()).unwrap_or(Some(0));
    let had_colon = body.contains(':');
    if parts.next().is_some() || (!had_colon && meridiem.is_none() && !after_at) {
        return None;
    }
    let (minute, second) = (minute?, second?);
    let hour = match meridiem {
        Some(pm) => {
            if !(1..=12).contains(&hour) {
                return None;
            }
            hour % 12 + if pm { 12 } else { 0 }
        }
        None if !had_colon && after_at => plain_hour(hour),
        None if hour < 24 => hour,
        None => return None,
    };
    hms(hour, minute, second).map(|t| (t, used, second != 0))
}

fn spoken_clock(words: &[String], i: usize) -> Option<(NaiveTime, usize)> {
    let (minutes, to) = match (words.get(i)?.as_str(), words.get(i + 1)?.as_str()) {
        ("half", "past") => (30, false),
        ("quarter", "past") => (15, false),
        ("quarter", "to") => (15, true),
        _ => return None,
    };
    let hour: u32 = words
        .get(i + 2)?
        .parse()
        .ok()
        .filter(|h| (1..=12).contains(h))?;
    let hour = plain_hour(hour);
    let time = if to {
        hms((hour + 23) % 24, 60 - minutes, 0)?
    } else {
        hms(hour, minutes, 0)?
    };
    Some((time, 3))
}

fn day_number(word: &str) -> Option<u32> {
    let digits = word.trim_end_matches(|c: char| c.is_ascii_alphabetic());
    let suffix = &word[digits.len()..];
    if !suffix.is_empty() && !matches!(suffix, "st" | "nd" | "rd" | "th") {
        return None;
    }
    digits.parse::<u32>().ok().filter(|d| (1..=31).contains(d))
}

fn start_of_week(date: NaiveDate) -> NaiveDate {
    date - Duration::days(i64::from(date.weekday().num_days_from_sunday()))
}

fn last_day_of_month(year: i32, month: u32) -> Option<NaiveDate> {
    let first_next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)?
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)?
    };
    Some(first_next - Duration::days(1))
}

fn calendar_date(words: &[String], i: usize, today: NaiveDate) -> Option<(NaiveDate, usize, bool)> {
    let word = words.get(i)?.as_str();
    let next = words.get(i + 1).map(String::as_str);
    if let Some((y, rest)) = word.split_once('-') {
        let (m, d) = rest.split_once('-')?;
        return NaiveDate::from_ymd_opt(y.parse().ok()?, m.parse().ok()?, d.parse().ok()?)
            .map(|date| (date, 1, true));
    }
    if word.contains('/') {
        let parts: Vec<&str> = word.split('/').collect();
        let (m, d): (u32, u32) = (parts.first()?.parse().ok()?, parts.get(1)?.parse().ok()?);
        return match parts.get(2) {
            Some(y) => {
                let y: i32 = y.parse().ok()?;
                NaiveDate::from_ymd_opt(if y < 100 { 2000 + y } else { y }, m, d)
                    .map(|date| (date, 1, true))
            }
            None => NaiveDate::from_ymd_opt(today.year(), m, d).map(|date| (date, 1, false)),
        };
    }

    let holiday = |m: u32, d: u32, used: usize| {
        NaiveDate::from_ymd_opt(today.year(), m, d).map(|date| (date, used, false))
    };
    match (word, next) {
        ("christmas", Some("eve")) => return holiday(12, 24, 2),
        ("christmas", _) => return holiday(12, 25, 1),
        ("halloween", _) => return holiday(10, 31, 1),
        ("valentines" | "valentine's" | "valentine", Some("day")) => return holiday(2, 14, 2),
        ("valentines" | "valentine's", _) => return holiday(2, 14, 1),
        ("new", Some("year" | "years" | "year's")) => {
            return match words.get(i + 2).map(String::as_str) {
                Some("eve") => holiday(12, 31, 3),
                Some("day") => {
                    NaiveDate::from_ymd_opt(today.year() + 1, 1, 1).map(|date| (date, 3, false))
                }
                _ => NaiveDate::from_ymd_opt(today.year() + 1, 1, 1).map(|date| (date, 2, false)),
            };
        }
        _ => {}
    }

    if let Some(m) = month(word) {
        let d = day_number(next?)?;
        let year = words
            .get(i + 2)
            .and_then(|y| y.parse::<i32>().ok())
            .filter(|y| (1000..=9999).contains(y));
        return NaiveDate::from_ymd_opt(year.unwrap_or(today.year()), m, d)
            .map(|date| (date, if year.is_some() { 3 } else { 2 }, year.is_some()));
    }

    if let (Some(d), Some(m)) = (day_number(word), next.and_then(month)) {
        let year = words
            .get(i + 2)
            .and_then(|y| y.parse::<i32>().ok())
            .filter(|y| (1000..=9999).contains(y));
        return NaiveDate::from_ymd_opt(year.unwrap_or(today.year()), m, d)
            .map(|date| (date, if year.is_some() { 3 } else { 2 }, year.is_some()));
    }
    None
}

pub fn parse(query: &str, now: DateTime<Local>) -> Option<Parsed> {
    let words = words(query);
    if words.is_empty() || words == ["now"] {
        return Some(Parsed {
            time: now,
            hour: true,
            weekday: false,
            day: true,
            year: true,
            second: true,
        });
    }
    read(&words, now)
        .or_else(|| fallback(&words.join(" "), now))
        .map(|parsed| settle(parsed, now))
}

fn settle(mut parsed: Parsed, now: DateTime<Local>) -> Parsed {
    let _ = now;
    if !parsed.hour {
        let quarter = 900_000;
        let rounded =
            (parsed.time.timestamp_millis() as f64 / quarter as f64).round() as i64 * quarter;
        if let Some(time) = Local.timestamp_millis_opt(rounded).single() {
            parsed.time = time;
        }
    }
    parsed
}

fn read(words: &[String], now: DateTime<Local>) -> Option<Parsed> {
    if let Some((terms, sign)) = duration(words) {
        let precise = terms
            .iter()
            .all(|(_, unit)| matches!(unit, Unit::Second | Unit::Minute | Unit::Hour));
        return Some(Parsed {
            time: shift(now, &terms, sign)?,
            hour: precise,
            weekday: false,
            day: true,
            year: true,
            second: precise,
        });
    }

    if let Some(at_word) = words
        .iter()
        .position(|w| matches!(w.as_str(), "from" | "after" | "before"))
    {
        if let Some((terms, _)) = duration(&words[..at_word]) {
            let sign = if words[at_word] == "before" { -1 } else { 1 };
            let base = read_calendar(&words[at_word + 1..], now)?;
            return Some(Parsed {
                time: shift(base.time, &terms, sign)?,
                day: true,
                ..base
            });
        }
    }
    read_calendar(words, now)
}

fn read_calendar(words: &[String], now: DateTime<Local>) -> Option<Parsed> {
    let today = now.date_naive();
    let mut date: Option<NaiveDate> = None;
    let mut time: Option<NaiveTime> = None;
    let (mut weekday_said, mut day_said, mut year_said, mut second_said) =
        (false, false, false, false);
    let mut part_of_day = false;

    let mut relative = false;
    let mut i = 0;
    while i < words.len() {
        let word = words[i].as_str();
        let next = words.get(i + 1).map(String::as_str);
        match word {
            "@" | "and" | "in" => i += 1,
            "today" => {
                relative = true;
                year_said = true;
                date = Some(today);
                day_said = true;
                i += 1;
            }
            "tomorrow" | "tmr" | "tmrw" | "2moro" => {
                relative = true;
                year_said = true;
                date = Some(today + Duration::days(1));
                day_said = true;
                i += 1;
            }
            "yesterday" => {
                relative = true;
                year_said = true;
                date = Some(today - Duration::days(1));
                day_said = true;
                i += 1;
            }
            "day"
                if next == Some("after")
                    && words.get(i + 2).map(String::as_str) == Some("tomorrow") =>
            {
                date = Some(today + Duration::days(2));
                relative = true;
                year_said = true;
                day_said = true;
                i += 3;
            }
            "day"
                if next == Some("before")
                    && words.get(i + 2).map(String::as_str) == Some("yesterday") =>
            {
                date = Some(today - Duration::days(2));
                relative = true;
                year_said = true;
                day_said = true;
                i += 3;
            }
            "tonight" => {
                date.get_or_insert(today);
                day_said = true;
                time.get_or_insert(hms(20, 0, 0)?);
                part_of_day = true;
                i += 1;
            }
            "morning" | "afternoon" | "evening" | "night" | "eod" => {
                time.get_or_insert(match word {
                    "morning" => hms(9, 0, 0)?,
                    "afternoon" => hms(15, 0, 0)?,
                    "evening" => hms(18, 0, 0)?,
                    "night" => hms(21, 0, 0)?,
                    _ => hms(17, 0, 0)?,
                });
                part_of_day = true;
                i += 1;
            }
            "end" | "start" | "beginning"
                if matches!(next, Some("month" | "week" | "year" | "day")) =>
            {
                let end = word == "end";
                match (next, end) {
                    (Some("month"), true) => date = last_day_of_month(today.year(), today.month()),
                    (Some("month"), false) => {
                        date = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
                    }
                    (Some("week"), true) => date = Some(start_of_week(today) + Duration::days(5)),
                    (Some("week"), false) => date = Some(start_of_week(today) + Duration::days(1)),
                    (Some("year"), true) => date = NaiveDate::from_ymd_opt(today.year(), 12, 31),
                    (Some("year"), false) => date = NaiveDate::from_ymd_opt(today.year(), 1, 1),
                    _ => {
                        date.get_or_insert(today);
                        time.get_or_insert(if end { hms(17, 0, 0)? } else { hms(9, 0, 0)? });
                    }
                }
                day_said = true;
                i += 2;
            }
            "next" | "last" | "previous" | "coming" | "this" => {
                let sign: i64 = match word {
                    "next" | "coming" => 1,
                    "this" => 0,
                    _ => -1,
                };
                let target = next?;
                i += 2;
                if let Some(day) = weekday(target) {
                    let ahead = (i64::from(day.num_days_from_monday())
                        - i64::from(today.weekday().num_days_from_monday()))
                    .rem_euclid(7);
                    date = Some(match sign {
                        1 => today + Duration::days(if ahead == 0 { 7 } else { ahead }),
                        -1 => today - Duration::days(if ahead == 0 { 7 } else { 7 - ahead }),
                        _ => today + Duration::days(if ahead > 3 { ahead - 7 } else { ahead }),
                    });
                    weekday_said = true;
                } else if let Some(unit) = unit_of(target)
                    .filter(|u| matches!(u, Unit::Day | Unit::Week | Unit::Month | Unit::Year))
                {
                    let base = at(today, hms(12, 0, 0)?)?;
                    date = Some(shift(base, &[(1.0, unit)], sign as i32)?.date_naive());
                    relative = true;
                    year_said = true;
                    day_said = true;
                } else if matches!(target, "morning" | "afternoon" | "evening" | "night") {
                    i -= 1;
                    date.get_or_insert(today);
                    day_said = true;
                } else {
                    return None;
                }
            }
            _ => {
                if let Some((t, used)) = spoken_clock(words, i) {
                    time = Some(t);
                    i += used;
                } else if let Some((t, used, seconds)) = clock(words, i) {
                    time = Some(t);
                    second_said |= seconds;
                    i += used;
                } else if let Some((found, used, with_year)) = calendar_date(words, i, today) {
                    date = Some(found);
                    day_said = true;
                    year_said |= with_year;
                    i += used;
                } else if let Some(day) = weekday(word) {
                    let ahead = (i64::from(day.num_days_from_monday())
                        - i64::from(today.weekday().num_days_from_monday()))
                    .rem_euclid(7);

                    let offset = if ahead > 3 { ahead - 7 } else { ahead };
                    date = Some(today + Duration::days(offset));
                    weekday_said = true;
                    i += 1;
                } else {
                    let d = day_number(word).filter(|_| word.ends_with(|c: char| c.is_ascii_alphabetic()))?;
                    let this = NaiveDate::from_ymd_opt(today.year(), today.month(), d);
                    date = match this.filter(|day| *day >= today) {
                        Some(day) => Some(day),
                        None => today
                            .checked_add_months(Months::new(1))
                            .and_then(|next_month| {
                                NaiveDate::from_ymd_opt(next_month.year(), next_month.month(), d)
                            }),
                    };
                    day_said = true;
                    i += 1;
                }
            }
        }
    }
    if date.is_none() && time.is_none() {
        return None;
    }
    let hour_said = time.is_some();
    let date = date.unwrap_or(today);
    let time = time.unwrap_or_else(|| {
        if relative {
            now.time()
        } else {
            hms(12, 0, 0).expect("noon exists")
        }
    });
    let found = at(date, time)?;
    let _ = part_of_day;
    Some(Parsed {
        time: found,
        hour: hour_said,
        weekday: weekday_said && !day_said,
        day: day_said,
        year: year_said,
        second: second_said,
    })
}

fn fallback(text: &str, now: DateTime<Local>) -> Option<Parsed> {
    if text.rsplit(' ').next().is_some_and(|last| {
        !last.is_empty() && last.len() <= 2 && last.chars().all(|c| c.is_ascii_digit())
    }) {
        return None;
    }
    if let Ok(time) = interim::parse_date_string(text, now, interim::Dialect::Us) {
        return Some(Parsed {
            time,
            hour: true,
            weekday: false,
            day: true,
            year: false,
            second: false,
        });
    }
    match human_date_parser::from_human_time(text, now.naive_local()).ok()? {
        human_date_parser::ParseResult::DateTime(date_time) => Local
            .from_local_datetime(&date_time)
            .single()
            .map(|time| Parsed {
                time,
                hour: true,
                weekday: false,
                day: true,
                year: false,
                second: date_time.second() != 0,
            }),
        human_date_parser::ParseResult::Date(date) => at(date, hms(12, 0, 0)?).map(|time| Parsed {
            time,
            hour: false,
            weekday: false,
            day: true,
            year: false,
            second: false,
        }),
        human_date_parser::ParseResult::Time(clock) => {
            at(now.date_naive(), clock).map(|time| Parsed {
                time,
                hour: true,
                weekday: false,
                day: false,
                year: false,
                second: false,
            })
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Row {
    Format {
        unix: i64,
        letter: char,
        description: Option<&'static str>,
    },
    Divider,
}

#[derive(Clone, Copy, PartialEq)]
enum Period {
    Day,
    Week,
    Year,
}

impl Period {
    fn same(self, a: DateTime<Local>, b: DateTime<Local>) -> bool {
        let (a, b) = (a.date_naive(), b.date_naive());
        match self {
            Period::Day => a == b,
            Period::Week => start_of_week(a) == start_of_week(b),
            Period::Year => a.year() == b.year(),
        }
    }

    fn step(self, time: DateTime<Local>, by: i64) -> DateTime<Local> {
        match self {
            Period::Day => time + Duration::days(by),
            Period::Week => time + Duration::days(7 * by),
            Period::Year => {
                let months = Months::new(12);
                if by >= 0 {
                    time.checked_add_months(months).unwrap_or(time)
                } else {
                    time.checked_sub_months(months).unwrap_or(time)
                }
            }
        }
    }

    fn names(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Period::Day => (
                "time-panel-yesterday",
                "time-panel-today",
                "time-panel-tomorrow",
            ),
            Period::Week => (
                "time-panel-last-week",
                "time-panel-this-week",
                "time-panel-next-week",
            ),
            Period::Year => (
                "time-panel-last-year",
                "time-panel-this-year",
                "time-panel-next-year",
            ),
        }
    }
}

pub fn rows(parsed: &Parsed, now: DateTime<Local>) -> Vec<Row> {
    let mut rows = Vec::new();
    let short = if parsed.second { 'S' } else { 's' };
    let period = if !parsed.day && !parsed.weekday {
        rows.push(Row::Format {
            unix: parsed.time.timestamp(),
            letter: if parsed.second { 'T' } else { 't' },
            description: Some("time-panel-time-only"),
        });
        rows.push(Row::Divider);
        Some(Period::Day)
    } else if parsed.weekday && !parsed.day {
        Some(Period::Week)
    } else if !parsed.year {
        Some(Period::Year)
    } else {
        None
    };
    let (mut adjusted, mut adjusted_description, mut own_description): (
        Option<DateTime<Local>>,
        Option<&'static str>,
        Option<&'static str>,
    ) = (None, None, None);
    if let Some(period) = period {
        let (previous, current, next) = period.names();
        if period.same(parsed.time, now) {
            own_description = Some(current);
            if parsed.time <= now {
                adjusted_description = Some(next);
            }
        } else if period.same(parsed.time, period.step(now, -1)) {
            own_description = Some(previous);
            adjusted_description = Some(current);
        }
        if adjusted_description.is_some() {
            adjusted = Some(period.step(parsed.time, 1));
        }
    }
    let group = |rows: &mut Vec<Row>, time: DateTime<Local>, description: Option<&'static str>| {
        let unix = time.timestamp();
        rows.push(Row::Format {
            unix,
            letter: short,
            description,
        });
        for letter in ['f', 'F', 'R'] {
            rows.push(Row::Format {
                unix,
                letter,
                description: None,
            });
        }
    };
    if let Some(time) = adjusted {
        group(&mut rows, time, adjusted_description);
        rows.push(Row::Divider);
    }
    group(&mut rows, parsed.time, own_description);
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Local> {
        at(
            NaiveDate::from_ymd_opt(2026, 10, 16).unwrap(),
            NaiveTime::from_hms_opt(15, 44, 38).unwrap(),
        )
        .unwrap()
    }

    fn shown(query: &str) -> Option<String> {
        parse(query, now()).map(|p| p.time.format("%Y-%m-%d %H:%M:%S").to_string())
    }

    #[test]
    fn spans_of_time_are_read_whole() {
        assert_eq!(shown("in 4 hours").as_deref(), Some("2026-10-16 19:44:38"));
        assert_eq!(
            shown("in 4 hours and 30 minutes").as_deref(),
            Some("2026-10-16 20:14:38")
        );
        assert_eq!(
            shown("in 2 hours 15 minutes").as_deref(),
            Some("2026-10-16 17:59:38")
        );
        assert_eq!(shown("2h15m").as_deref(), Some("2026-10-16 17:59:38"));
        assert_eq!(
            shown("in half an hour").as_deref(),
            Some("2026-10-16 16:14:38")
        );
        assert_eq!(
            shown("in an hour and a half").as_deref(),
            Some("2026-10-16 17:14:38")
        );
        assert_eq!(
            shown("in 90 minutes").as_deref(),
            Some("2026-10-16 17:14:38")
        );
        assert_eq!(shown("3 hours ago").as_deref(), Some("2026-10-16 12:44:38"));
        assert_eq!(
            shown("in a couple of hours").as_deref(),
            Some("2026-10-16 17:44:38")
        );
        assert_eq!(
            shown("in 2 weeks").map(|s| s[..10].to_string()).as_deref(),
            Some("2026-10-30")
        );
    }

    #[test]
    fn days_and_clock_times_are_read_together() {
        assert_eq!(shown("tomorrow").as_deref(), Some("2026-10-17 15:45:00"));
        assert_eq!(shown("friday").as_deref(), Some("2026-10-16 12:00:00"));
        assert_eq!(shown("oct 5").as_deref(), Some("2026-10-05 12:00:00"));
        assert_eq!(
            shown("tomorrow 5pm").as_deref(),
            Some("2026-10-17 17:00:00")
        );
        assert_eq!(
            shown("tomorrow at 5pm").as_deref(),
            Some("2026-10-17 17:00:00")
        );
        assert_eq!(
            shown("noon tomorrow").as_deref(),
            Some("2026-10-17 12:00:00")
        );
        assert_eq!(
            shown("tomorrow morning").as_deref(),
            Some("2026-10-17 09:00:00")
        );
        assert_eq!(
            shown("the day after tomorrow at 8:30").as_deref(),
            Some("2026-10-18 08:30:00")
        );
        assert_eq!(
            shown("next friday at 9:30").as_deref(),
            Some("2026-10-23 09:30:00")
        );
        assert_eq!(shown("friday at 3").as_deref(), Some("2026-10-16 15:00:00"));
        assert_eq!(
            shown("half past 5 tomorrow").as_deref(),
            Some("2026-10-17 17:30:00")
        );
        assert_eq!(shown("oct 20 8pm").as_deref(), Some("2026-10-20 20:00:00"));
        assert_eq!(
            shown("2026-12-24 noon").as_deref(),
            Some("2026-12-24 12:00:00")
        );
        assert_eq!(
            shown("christmas eve 6pm").as_deref(),
            Some("2026-12-24 18:00:00")
        );
        assert_eq!(
            shown("a week from tomorrow")
                .map(|s| s[..10].to_string())
                .as_deref(),
            Some("2026-10-24")
        );
        assert_eq!(
            shown("end of the month")
                .map(|s| s[..10].to_string())
                .as_deref(),
            Some("2026-10-31")
        );
    }

    #[test]
    fn what_is_not_a_time_is_refused() {
        assert_eq!(shown("banana"), None);
        assert_eq!(shown("in x hours"), None);
        assert_eq!(shown("tomorrow 5"), None);
    }

    #[test]
    fn a_time_said_without_an_hour_is_rounded_to_a_quarter() {
        assert_eq!(shown("in 3 days").as_deref(), Some("2026-10-19 15:45:00"));
        assert_eq!(shown("next week").as_deref(), Some("2026-10-23 15:45:00"));
    }

    #[test]
    fn rows_follow_what_was_said() {
        let formats = |rows: &[Row]| {
            rows.iter()
                .filter_map(|r| {
                    if let Row::Format { letter, .. } = r {
                        Some(*letter)
                    } else {
                        None
                    }
                })
                .collect::<String>()
        };

        let parsed = parse("in 4 hours", now()).unwrap();
        assert_eq!(formats(&rows(&parsed, now())), "SfFR");

        let parsed = parse("3pm", now()).unwrap();
        let rows = rows(&parsed, now());
        assert_eq!(formats(&rows), "tsfFRsfFR");
        assert!(matches!(
            rows[0],
            Row::Format {
                description: Some("time-panel-time-only"),
                ..
            }
        ));
        assert!(matches!(
            rows[2],
            Row::Format {
                description: Some("time-panel-tomorrow"),
                ..
            }
        ));
        assert!(matches!(
            rows[7],
            Row::Format {
                description: Some("time-panel-today"),
                ..
            }
        ));

        let parsed = parse("oct 5", now()).unwrap();
        assert!(rows_have(&rows_of(&parsed), "time-panel-next-year"));
    }

    fn rows_of(parsed: &Parsed) -> Vec<Row> {
        rows(parsed, now())
    }

    fn rows_have(rows: &[Row], description: &str) -> bool {
        rows.iter()
            .any(|r| matches!(r, Row::Format { description: Some(d), .. } if *d == description))
    }

    #[test]
    fn relative_times_use_the_clients_thresholds() {
        assert_eq!(relative(5), "in 4 seconds");
        assert_eq!(relative(-3), "4 seconds ago");
        assert_eq!(relative(86401), "in a day");
        assert_eq!(relative(-2 * 3600 + 1), "2 hours ago");
        assert_eq!(relative(40 * 86400 + 1), "in a month");
    }

    #[test]
    fn formats_match_the_client() {
        let unix = at(
            NaiveDate::from_ymd_opt(2026, 10, 17).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .unwrap()
        .timestamp();
        assert_eq!(format_at(unix, Some('s'), now()), "10/17/26, 5:00 PM");
        assert_eq!(format_at(unix, None, now()), "October 17, 2026 at 5:00 PM");
        assert_eq!(
            format_at(unix, Some('F'), now()),
            "Saturday, October 17, 2026 at 5:00 PM"
        );
        assert_eq!(format_at(unix, Some('R'), now()), "in a day");
    }
}
