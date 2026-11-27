use chrono::Local;
use chrono::NaiveDate;
use chrono::TimeZone;
use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::anim::ToggleState;

use crate::models::Id;
use crate::models::id::marker::{ChannelMarker, MessageMarker};
use crate::rest::endpoints::search::{SearchParams, SearchScope, SortMode};

use crate::utils::time::snowflake_at;

pub const HISTORY_LIMIT: usize = 10;

pub const GROUP_LIMIT: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FilterKind {
    From,
    In,
    Mentions,
    Has,
    Before,
    During,
    After,
    Pinned,
    AuthorType,
}

impl FilterKind {
    pub const SUGGESTED: [FilterKind; 4] = [
        FilterKind::From,
        FilterKind::In,
        FilterKind::Has,
        FilterKind::Mentions,
    ];

    pub const ALL: [FilterKind; 9] = [
        FilterKind::From,
        FilterKind::In,
        FilterKind::Mentions,
        FilterKind::Has,
        FilterKind::Before,
        FilterKind::During,
        FilterKind::After,
        FilterKind::Pinned,
        FilterKind::AuthorType,
    ];

    pub fn keyword(self) -> &'static str {
        match self {
            FilterKind::From => "from",
            FilterKind::In => "in",
            FilterKind::Mentions => "mentions",
            FilterKind::Has => "has",
            FilterKind::Before => "before",
            FilterKind::During => "during",
            FilterKind::After => "after",
            FilterKind::Pinned => "pinned",
            FilterKind::AuthorType => "author_type",
        }
    }

    pub fn from_keyword(word: &str) -> Option<Self> {
        FilterKind::ALL
            .into_iter()
            .find(|k| k.keyword().eq_ignore_ascii_case(word))
    }

    pub fn title(self) -> String {
        match self {
            FilterKind::From => requiem_i18n::t!("search-state-from-a-specific-user"),
            FilterKind::In => requiem_i18n::t!("search-state-sent-in-a-specific-channel"),
            FilterKind::Has => requiem_i18n::t!("search-state-includes-a-specific-type-of-data"),
            FilterKind::Mentions => requiem_i18n::t!("search-state-mentions-a-specific-user"),
            FilterKind::Before => requiem_i18n::t!("search-state-before-a-specific-date"),
            FilterKind::During => requiem_i18n::t!("search-state-during-a-specific-date"),
            FilterKind::After => requiem_i18n::t!("search-state-after-a-specific-date"),
            FilterKind::Pinned => requiem_i18n::t!("search-state-pinned-messages"),
            FilterKind::AuthorType => requiem_i18n::t!("search-state-from-a-type-of-author"),
        }
    }

    pub fn hint(self) -> String {
        match self {
            FilterKind::From | FilterKind::Mentions => requiem_i18n::t!("search-state-hint-user"),
            FilterKind::In => requiem_i18n::t!("search-state-hint-channel"),
            FilterKind::Has => requiem_i18n::t!("search-state-link-embed-or-file"),
            FilterKind::Before | FilterKind::During | FilterKind::After => {
                requiem_i18n::t!("search-state-hint-date")
            }
            FilterKind::Pinned => requiem_i18n::t!("search-state-true-or-false"),
            FilterKind::AuthorType => requiem_i18n::t!("search-state-user-bot-or-webhook"),
        }
    }

    pub fn group_label(self) -> String {
        match self {
            FilterKind::From => requiem_i18n::t!("search-state-from-user"),
            FilterKind::In => requiem_i18n::t!("search-state-in-channel"),
            FilterKind::Mentions => requiem_i18n::t!("search-state-mentions-user"),
            FilterKind::Has => requiem_i18n::t!("search-state-message-contains"),
            FilterKind::Before => requiem_i18n::t!("search-state-before-date"),
            FilterKind::During => requiem_i18n::t!("search-state-during-date"),
            FilterKind::After => requiem_i18n::t!("search-state-after-date"),
            FilterKind::Pinned => requiem_i18n::t!("search-state-pinned"),
            FilterKind::AuthorType => requiem_i18n::t!("search-state-author-type"),
        }
    }

    pub fn is_date(self) -> bool {
        matches!(
            self,
            FilterKind::Before | FilterKind::During | FilterKind::After
        )
    }
}

pub const HAS_VALUES: [&str; 9] = [
    "image", "video", "link", "file", "embed", "sound", "poll", "sticker", "forward",
];
pub const AUTHOR_TYPES: [&str; 3] = ["user", "bot", "webhook"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub value: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: FilterKind,
    pub answer: Option<Answer>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
    Filter(Token),

    Text(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    pub segments: Vec<Segment>,

    pub text: String,
}

impl Query {
    pub fn terms(&self) -> Vec<String> {
        self.content()
            .split_whitespace()
            .map(str::to_string)
            .collect()
    }

    pub fn content(&self) -> String {
        let mut parts: Vec<&str> = self
            .segments
            .iter()
            .filter_map(|s| {
                if let Segment::Text(t) = s {
                    Some(t.as_str())
                } else {
                    None
                }
            })
            .collect();
        parts.push(self.text.trim());
        parts.retain(|p| !p.is_empty());
        parts.join(" ")
    }

    pub fn tokens(&self) -> impl Iterator<Item = &Token> {
        self.segments.iter().filter_map(|s| {
            if let Segment::Filter(t) = s {
                Some(t)
            } else {
                None
            }
        })
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty() && self.text.trim().is_empty()
    }

    pub fn params(&self) -> SearchParams {
        let mut params = SearchParams {
            content: self.content(),
            ..Default::default()
        };
        for token in self.tokens() {
            let Some(answer) = &token.answer else {
                continue;
            };
            match token.kind {
                FilterKind::From => params.author_ids.extend(answer.value.parse::<u64>()),
                FilterKind::In => params.channel_ids.extend(answer.value.parse::<u64>()),
                FilterKind::Mentions => params.mentions.extend(answer.value.parse::<u64>()),
                FilterKind::Has => params.has.push(answer.value.clone()),
                FilterKind::AuthorType => params.author_types.push(answer.value.clone()),
                FilterKind::Pinned => params.pinned = Some(answer.value == "true"),
                FilterKind::Before | FilterKind::During | FilterKind::After => {
                    let Some((start, end)) = day_bounds_ms(&answer.value) else {
                        continue;
                    };
                    match token.kind {
                        FilterKind::Before => params.max_id = Some(snowflake_at(start)),
                        FilterKind::After => params.min_id = Some(snowflake_at(end)),
                        _ => {
                            params.min_id = Some(snowflake_at(start));
                            params.max_id = Some(snowflake_at(end));
                        }
                    }
                }
            }
        }
        params
    }
}

fn day_bounds_ms(date: &str) -> Option<(i64, i64)> {
    let day = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    let start = Local
        .from_local_datetime(&day.and_hms_opt(0, 0, 0)?)
        .earliest()?;
    let end = Local
        .from_local_datetime(&day.succ_opt()?.and_hms_opt(0, 0, 0)?)
        .earliest()?;
    Some((start.timestamp_millis(), end.timestamp_millis()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Submitted {
    pub scope: SearchScope,
    pub query: Query,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ItemAction {
    AddFilter(FilterKind),

    Answer(Answer),

    Submit,

    History(usize),

    ShowFilters,

    MoreFilters,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowIcon {
    Search,
    Filter(FilterKind),
    More,
    Has(&'static str),
    Channel,
    User,
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoutItem {
    pub icon: RowIcon,
    pub title: String,

    pub subtitle: Option<(String, String)>,

    pub title_suffix: Option<String>,

    pub avatar: Option<(u64, String, Option<String>)>,

    pub query: Option<Query>,
    pub action: ItemAction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PopoutRow {
    Header { label: String, clear_history: bool },
    Item(PopoutItem),
    Divider,
}

#[derive(Clone, Default)]
pub struct Candidates {
    pub users: Vec<(u64, String, String, String, Option<String>)>,

    pub channels: Vec<(u64, String)>,
}

fn matches(haystack: &str, needle: &str) -> bool {
    needle.is_empty() || haystack.to_lowercase().contains(&needle.to_lowercase())
}

fn user_item(user: &(u64, String, String, String, Option<String>)) -> PopoutItem {
    let (id, display, username, discriminator, avatar) = user;
    PopoutItem {
        icon: RowIcon::User,
        title: display.clone(),
        title_suffix: Some(username.clone()),
        subtitle: None,
        avatar: Some((*id, discriminator.clone(), avatar.clone())),
        query: None,
        action: ItemAction::Answer(Answer {
            value: id.to_string(),
            label: display.clone(),
        }),
    }
}

fn channel_item(channel: &(u64, String)) -> PopoutItem {
    PopoutItem {
        icon: RowIcon::Channel,
        title: channel.1.clone(),
        title_suffix: None,
        subtitle: None,
        avatar: None,
        query: None,
        action: ItemAction::Answer(Answer {
            value: channel.0.to_string(),
            label: channel.1.clone(),
        }),
    }
}

fn has_item(value: &'static str) -> PopoutItem {
    PopoutItem {
        icon: RowIcon::Has(value),
        title: value.to_string(),
        title_suffix: None,
        subtitle: None,
        avatar: None,
        query: None,
        action: ItemAction::Answer(Answer {
            value: value.to_string(),
            label: value.to_string(),
        }),
    }
}

fn plain_item(value: &str) -> PopoutItem {
    PopoutItem {
        icon: RowIcon::None,
        title: value.to_string(),
        title_suffix: None,
        subtitle: None,
        avatar: None,
        query: None,
        action: ItemAction::Answer(Answer {
            value: value.to_string(),
            label: value.to_string(),
        }),
    }
}

pub fn popout_rows(
    text: &str,
    parsed: &Parsed,
    show_filters: bool,
    history: &[Query],
    candidates: &Candidates,
) -> Vec<PopoutRow> {
    if let Some((kind, _, partial)) = parsed.pending(text) {
        let q = partial.as_str();
        let items: Vec<PopoutItem> = match kind {
            FilterKind::From | FilterKind::Mentions => candidates
                .users
                .iter()
                .filter(|u| matches(&u.1, q) || matches(&u.2, q))
                .take(GROUP_LIMIT * 2)
                .map(user_item)
                .collect(),
            FilterKind::In => candidates
                .channels
                .iter()
                .filter(|c| matches(&c.1, q))
                .take(GROUP_LIMIT * 2)
                .map(channel_item)
                .collect(),
            FilterKind::Has => HAS_VALUES
                .iter()
                .filter(|v| matches(v, q))
                .map(|v| has_item(v))
                .collect(),
            FilterKind::AuthorType => AUTHOR_TYPES
                .iter()
                .filter(|v| matches(v, q))
                .map(|v| plain_item(v))
                .collect(),
            FilterKind::Pinned => ["true", "false"]
                .iter()
                .filter(|v| matches(v, q))
                .map(|v| plain_item(v))
                .collect(),
            FilterKind::Before | FilterKind::During | FilterKind::After => Vec::new(),
        };
        if items.is_empty() {
            return Vec::new();
        }

        let mut rows = if kind == FilterKind::Pinned {
            Vec::new()
        } else {
            vec![PopoutRow::Header {
                label: kind.group_label().to_string(),
                clear_history: false,
            }]
        };
        rows.extend(items.into_iter().map(PopoutRow::Item));
        return rows;
    }

    let typed = parsed.free_text(text);
    let typed = typed.as_str();
    let mut rows = Vec::new();
    if !typed.is_empty() && !show_filters {
        rows.push(PopoutRow::Item(PopoutItem {
            icon: RowIcon::Search,
            title: requiem_i18n::t!("search-state-search-for", query = typed),
            title_suffix: None,
            subtitle: None,
            avatar: None,
            query: None,
            action: ItemAction::Submit,
        }));
        rows.push(PopoutRow::Item(PopoutItem {
            icon: RowIcon::More,
            title: requiem_i18n::t!("search-state-add-search-filters").to_string(),
            title_suffix: None,
            subtitle: None,
            avatar: None,
            query: None,
            action: ItemAction::ShowFilters,
        }));
        let mut group = |kind: FilterKind, items: Vec<PopoutItem>| {
            if !items.is_empty() {
                rows.push(PopoutRow::Header {
                    label: kind.group_label().to_string(),
                    clear_history: false,
                });
                rows.extend(items.into_iter().map(PopoutRow::Item));
            }
        };
        let user_matches = |kind: FilterKind| -> Vec<PopoutItem> {
            candidates
                .users
                .iter()
                .filter(|u| matches(&u.1, typed) || matches(&u.2, typed))
                .take(GROUP_LIMIT)
                .map(|u| with_prefix(user_item(u), kind))
                .collect()
        };
        group(FilterKind::From, user_matches(FilterKind::From));
        group(FilterKind::Mentions, user_matches(FilterKind::Mentions));
        group(
            FilterKind::In,
            candidates
                .channels
                .iter()
                .filter(|c| matches(&c.1, typed))
                .take(GROUP_LIMIT)
                .map(|c| with_prefix(channel_item(c), FilterKind::In))
                .collect(),
        );
        group(
            FilterKind::Has,
            HAS_VALUES
                .iter()
                .filter(|v| matches(v, typed))
                .take(GROUP_LIMIT)
                .map(|v| with_prefix(has_item(v), FilterKind::Has))
                .collect(),
        );
        return rows;
    }

    rows.push(PopoutRow::Header {
        label: requiem_i18n::t!("search-state-filters").to_string(),
        clear_history: false,
    });
    for kind in FilterKind::SUGGESTED {
        rows.push(PopoutRow::Item(PopoutItem {
            icon: RowIcon::Filter(kind),
            title: kind.title().to_string(),
            title_suffix: None,
            subtitle: Some((format!("{}:", kind.keyword()), kind.hint().to_string())),
            avatar: None,
            query: None,
            action: ItemAction::AddFilter(kind),
        }));
    }
    rows.push(PopoutRow::Item(PopoutItem {
        icon: RowIcon::More,
        title: requiem_i18n::t!("search-state-more-filters").to_string(),
        title_suffix: None,
        subtitle: Some((
            String::new(),
            requiem_i18n::t!("search-state-dates-author-type-and-more").to_string(),
        )),
        avatar: None,
        query: None,
        action: ItemAction::MoreFilters,
    }));
    if !history.is_empty() {
        rows.push(PopoutRow::Divider);
        rows.push(PopoutRow::Header {
            label: requiem_i18n::t!("search-state-history").to_string(),
            clear_history: true,
        });
        for (i, query) in history.iter().enumerate() {
            rows.push(PopoutRow::Item(PopoutItem {
                icon: RowIcon::Search,
                title: String::new(),
                title_suffix: None,
                subtitle: None,
                avatar: None,
                query: Some(query.clone()),
                action: ItemAction::History(i),
            }));
        }
    }
    rows
}

fn with_prefix(mut item: PopoutItem, kind: FilterKind) -> PopoutItem {
    if let ItemAction::Answer(answer) = item.action.clone() {
        item.subtitle = Some((format!("{}:", kind.keyword()), answer.label.clone()));
        item.title_suffix = None;
        item.query = Some(Query {
            segments: vec![Segment::Filter(Token {
                kind,
                answer: Some(answer.clone()),
            })],
            text: String::new(),
        });
    }
    item
}

pub fn answer_kind(item: &PopoutItem) -> Option<FilterKind> {
    item.query.as_ref()?.tokens().next().map(|t| t.kind)
}

#[derive(Clone, Debug)]
pub struct ParsedToken {
    pub kind: FilterKind,

    pub keyword: Range<usize>,

    pub answer: Option<(Range<usize>, Answer)>,
}

#[derive(Clone, Debug, Default)]
pub struct Parsed {
    pub tokens: Vec<ParsedToken>,
}

fn at_boundary(rest: &str) -> bool {
    rest.is_empty() || rest.starts_with(char::is_whitespace)
}

fn starts_with_name(rest: &str, name: &str) -> bool {
    !name.is_empty()
        && rest
            .get(..name.len())
            .is_some_and(|head| head.to_lowercase() == name.to_lowercase())
        && at_boundary(&rest[name.len()..])
}

fn resolve(kind: FilterKind, rest: &str, candidates: &Candidates) -> Option<(Answer, usize)> {
    match kind {
        FilterKind::From | FilterKind::Mentions => {
            let mut best: Option<(Answer, usize)> = None;
            for (id, display, username, _, _) in &candidates.users {
                for name in [display, username] {
                    if starts_with_name(rest, name)
                        && best.as_ref().is_none_or(|(_, len)| name.len() > *len)
                    {
                        best = Some((
                            Answer {
                                value: id.to_string(),
                                label: display.clone(),
                            },
                            name.len(),
                        ));
                    }
                }
            }
            best
        }
        FilterKind::In => {
            let hashless = rest.strip_prefix('#').unwrap_or(rest);
            let skipped = rest.len() - hashless.len();
            candidates
                .channels
                .iter()
                .filter(|(_, name)| starts_with_name(hashless, name))
                .max_by_key(|(_, name)| name.len())
                .map(|(id, name)| {
                    (
                        Answer {
                            value: id.to_string(),
                            label: name.clone(),
                        },
                        skipped + name.len(),
                    )
                })
        }
        FilterKind::Has => HAS_VALUES
            .iter()
            .find(|v| starts_with_name(rest, v))
            .map(|v| {
                (
                    Answer {
                        value: (*v).to_string(),
                        label: (*v).to_string(),
                    },
                    v.len(),
                )
            }),
        FilterKind::AuthorType => {
            AUTHOR_TYPES
                .iter()
                .find(|v| starts_with_name(rest, v))
                .map(|v| {
                    (
                        Answer {
                            value: (*v).to_string(),
                            label: (*v).to_string(),
                        },
                        v.len(),
                    )
                })
        }
        FilterKind::Pinned => ["true", "false"]
            .iter()
            .find(|v| starts_with_name(rest, v))
            .map(|v| {
                (
                    Answer {
                        value: (*v).to_string(),
                        label: (*v).to_string(),
                    },
                    v.len(),
                )
            }),
        FilterKind::Before | FilterKind::During | FilterKind::After => {
            let date = rest.get(..10)?;
            (day_bounds_ms(date).is_some() && at_boundary(&rest[10..])).then(|| {
                (
                    Answer {
                        value: date.to_string(),
                        label: date.to_string(),
                    },
                    10,
                )
            })
        }
    }
}

pub fn parse(text: &str, candidates: &Candidates) -> Parsed {
    let mut tokens = Vec::new();
    let mut pos = 0;
    while pos < text.len() {
        let word_end = text[pos..]
            .find(char::is_whitespace)
            .map_or(text.len(), |n| pos + n);
        let word = &text[pos..word_end];
        let kind = word
            .find(':')
            .and_then(|colon| FilterKind::from_keyword(&word[..colon]).map(|kind| (kind, colon)));
        if let Some((kind, colon)) = kind {
            let after = pos + colon + 1;
            let skipped = text[after..].len() - text[after..].trim_start().len();
            let answer_start = after + skipped;
            match resolve(kind, &text[answer_start..], candidates) {
                Some((answer, len)) => {
                    tokens.push(ParsedToken {
                        kind,
                        keyword: pos..after,
                        answer: Some((after..answer_start + len, answer)),
                    });
                    pos = answer_start + len;
                }
                None => {
                    tokens.push(ParsedToken {
                        kind,
                        keyword: pos..after,
                        answer: None,
                    });
                    pos = after;
                }
            }
            continue;
        }
        pos = text[word_end..]
            .find(|c: char| !c.is_whitespace())
            .map_or(text.len(), |n| word_end + n);
    }
    Parsed { tokens }
}

impl Parsed {
    pub fn pending(&self, text: &str) -> Option<(FilterKind, usize, String)> {
        let token = self.tokens.last()?;
        match &token.answer {
            None => Some((
                token.kind,
                token.keyword.end,
                text[token.keyword.end..].trim().to_string(),
            )),
            Some((range, _)) if range.end == text.len() => Some((
                token.kind,
                token.keyword.end,
                text[range.clone()].trim().to_string(),
            )),
            Some(_) => None,
        }
    }

    pub fn free_text(&self, text: &str) -> String {
        let mut out = String::new();
        let mut pos = 0;
        for token in &self.tokens {
            out.push_str(&text[pos..token.keyword.start]);
            out.push(' ');
            pos = token
                .answer
                .as_ref()
                .map_or(token.keyword.end, |(range, _)| range.end);
        }
        out.push_str(&text[pos..]);
        out.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    pub fn query(&self, text: &str) -> Query {
        Query {
            segments: self
                .tokens
                .iter()
                .filter_map(|t| {
                    t.answer.as_ref().map(|(_, a)| {
                        Segment::Filter(Token {
                            kind: t.kind,
                            answer: Some(a.clone()),
                        })
                    })
                })
                .collect(),
            text: self.free_text(text),
        }
    }

    pub fn decorations(&self) -> Vec<(Range<usize>, bool)> {
        let mut out = Vec::new();
        for token in &self.tokens {
            out.push((token.keyword.clone(), true));
            if let Some((range, _)) = &token.answer {
                out.push((range.clone(), false));
            }
        }
        out
    }
}

pub fn query_to_text(query: &Query) -> String {
    let mut out = String::new();
    for token in query.tokens() {
        if let Some(answer) = &token.answer {
            out.push_str(&format!("{}: {} ", token.kind.keyword(), answer.label));
        }
    }
    out.push_str(&query.text);
    out
}

pub fn with_answer(
    text: &str,
    keyword_end: Option<usize>,
    kind: FilterKind,
    answer: &Answer,
) -> String {
    let head = match keyword_end {
        Some(end) => text[..end].to_string(),
        None => {
            let trimmed = text.trim_end();
            let head = trimmed
                .rsplit_once(char::is_whitespace)
                .map_or("", |(before, _)| before);
            let mut head = head.to_string();
            if !head.is_empty() {
                head.push(' ');
            }
            head.push_str(&format!("{}:", kind.keyword()));
            head
        }
    };
    format!("{head} {} ", answer.label)
}

#[derive(Clone)]
pub struct SearchState {
    pub text: Signal<String>,

    pub has_text: Signal<bool>,

    pub text_version: Signal<u32>,

    pub candidates: Rc<RefCell<Rc<Candidates>>>,

    pub popout_open: Signal<bool>,

    pub show_filters: Signal<bool>,

    pub selected: Signal<usize>,

    pub submitted: Signal<Option<Submitted>>,
    pub page: Signal<u32>,
    pub sort: Signal<SortMode>,
    pub history: Signal<Vec<Query>>,

    pub sort_menu_open: Signal<bool>,

    pub calendar_month: Signal<(i32, u32)>,

    pub filters_open: Signal<ToggleState>,

    pub filters_draft: Signal<FiltersDraft>,

    pub filters_dates: Rc<RefCell<Vec<(FilterKind, String)>>>,
    pub filters_tick: Signal<u32>,

    pub filters_baseline: Rc<RefCell<(FiltersDraft, Vec<(FilterKind, String)>)>>,

    pub filters_select: Signal<Option<String>>,

    pub actions: Rc<RefCell<Vec<(ItemAction, Option<FilterKind>)>>>,

    pub on_jump: Rc<RefCell<Rc<dyn Fn(Id<ChannelMarker>, Id<MessageMarker>)>>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FiltersDraft {
    pub from: Vec<Answer>,
    pub in_channels: Vec<Answer>,
    pub mentions: Vec<Answer>,
    pub has: Vec<String>,
    pub author_types: Vec<String>,
    pub pinned: Option<bool>,
}

impl FiltersDraft {
    pub fn from_query(query: &Query) -> (Self, Vec<(FilterKind, String)>) {
        let mut draft = FiltersDraft::default();
        let mut dates = Vec::new();
        for segment in &query.segments {
            let Segment::Filter(Token {
                kind,
                answer: Some(answer),
            }) = segment
            else {
                continue;
            };
            match kind {
                FilterKind::From => draft.from.push(answer.clone()),
                FilterKind::In => draft.in_channels.push(answer.clone()),
                FilterKind::Mentions => draft.mentions.push(answer.clone()),
                FilterKind::Has => draft.has.push(answer.value.clone()),
                FilterKind::AuthorType => draft.author_types.push(answer.value.clone()),
                FilterKind::Pinned => draft.pinned = Some(answer.value == "true"),
                FilterKind::Before | FilterKind::During | FilterKind::After => {
                    dates.push((*kind, answer.value.clone()))
                }
            }
        }
        (draft, dates)
    }

    pub fn apply(&self, dates: &[(FilterKind, String)], free_text: &str) -> Query {
        let mut out: Vec<Segment> = Vec::new();
        let mut push = |kind: FilterKind, answer: Answer| {
            out.push(Segment::Filter(Token {
                kind,
                answer: Some(answer),
            }))
        };
        for a in &self.from {
            push(FilterKind::From, a.clone());
        }
        for a in &self.in_channels {
            push(FilterKind::In, a.clone());
        }
        for v in &self.has {
            push(
                FilterKind::Has,
                Answer {
                    value: v.clone(),
                    label: v.clone(),
                },
            );
        }
        for a in &self.mentions {
            push(FilterKind::Mentions, a.clone());
        }
        for (kind, value) in dates {
            if day_bounds_ms(value).is_some() {
                push(
                    *kind,
                    Answer {
                        value: value.clone(),
                        label: value.clone(),
                    },
                );
            }
        }
        for v in &self.author_types {
            push(
                FilterKind::AuthorType,
                Answer {
                    value: v.clone(),
                    label: v.clone(),
                },
            );
        }
        if let Some(pinned) = self.pinned {
            let word = if pinned { "true" } else { "false" };
            push(
                FilterKind::Pinned,
                Answer {
                    value: word.to_string(),
                    label: word.to_string(),
                },
            );
        }
        Query {
            segments: out,
            text: free_text.to_string(),
        }
    }

    pub fn count(&self, dates: &[(FilterKind, String)]) -> usize {
        self.from.len()
            + self.in_channels.len()
            + self.mentions.len()
            + self.has.len()
            + self.author_types.len()
            + usize::from(self.pinned.is_some())
            + dates
                .iter()
                .filter(|(_, v)| day_bounds_ms(v).is_some())
                .count()
    }
}

pub const NO_ROW: usize = usize::MAX;

impl SearchState {
    pub fn new() -> Self {
        Self {
            text: signal(String::new()),
            has_text: signal(false),
            text_version: signal(0),
            candidates: Rc::new(RefCell::new(Rc::new(Candidates::default()))),
            popout_open: signal(false),
            show_filters: signal(false),
            selected: signal(NO_ROW),
            submitted: signal(None),
            page: signal(1),
            sort: signal(SortMode::Newest),
            history: signal(Vec::new()),
            sort_menu_open: signal(false),
            calendar_month: signal((0, 0)),
            filters_open: signal(ToggleState::new(false)),
            filters_draft: signal(FiltersDraft::default()),
            filters_dates: Rc::new(RefCell::new(Vec::new())),
            filters_baseline: Rc::new(RefCell::new((FiltersDraft::default(), Vec::new()))),
            filters_tick: signal(0),
            filters_select: signal(None),
            actions: Rc::new(RefCell::new(Vec::new())),
            on_jump: Rc::new(RefCell::new(Rc::new(|_, _| {}))),
        }
    }

    pub fn submit(&self, scope: SearchScope, query: Query) {
        if query.is_empty() {
            return;
        }
        self.history.update(|history| {
            history.retain(|q| q != &query);
            history.insert(0, query.clone());
            history.truncate(HISTORY_LIMIT);
        });
        self.page.set(1);
        self.popout_open.set(false);
        self.show_filters.set(false);
        self.submitted.set(Some(Submitted { scope, query }));
    }

    pub fn parsed(&self, text: &str) -> Parsed {
        parse(text, &self.candidates.borrow())
    }

    pub fn query(&self) -> Query {
        let text = self.text.peek();
        self.parsed(&text).query(&text)
    }

    pub fn typed(&self, text: &str) {
        if self.text.peek() != text {
            self.text.set(text.to_string());
        }
        let has_text = !text.is_empty();
        if self.has_text.peek() != has_text {
            self.has_text.set(has_text);
        }
    }

    pub fn set_text(&self, text: String) {
        self.typed(&text);
        self.text_version.update(|n| *n += 1);
    }

    pub fn open_filters(&self) {
        let (draft, dates) = FiltersDraft::from_query(&self.query());
        *self.filters_baseline.borrow_mut() = (draft.clone(), dates.clone());
        self.filters_draft.set(draft);
        *self.filters_dates.borrow_mut() = dates;
        self.filters_select.set(None);
        self.popout_open.set(false);
        self.filters_open.set(self.filters_open.peek().set(true));
    }

    pub fn close_filters(&self) {
        self.filters_select.set(None);
        self.filters_open.set(self.filters_open.peek().set(false));
    }

    pub fn clear(&self) {
        self.set_text(String::new());
        self.submitted.set(None);
        self.page.set(1);
        self.show_filters.set(false);
        self.selected.set(NO_ROW);
    }

    pub fn load(&self, query: &Query) {
        self.set_text(query_to_text(query));
    }
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageItem {
    Page(u32),
    Gap,
}

pub fn page_list(total: u32, selected: u32, max_visible: u32) -> Vec<PageItem> {
    let (a, e) = (max_visible, total);
    let s = max_visible.div_ceil(2);
    let n = max_visible / 2;
    let (mut lo, mut hi) = if selected <= s {
        (1, a)
    } else if selected > e.saturating_sub(n) {
        ((e + 1).saturating_sub(a), e)
    } else {
        (selected + 1 - s, selected + n)
    };
    lo = lo.max(1);
    hi = hi.min(e);
    let (mut head, mut tail) = (Vec::new(), Vec::new());
    if lo > 1 {
        head = vec![PageItem::Page(1), PageItem::Gap];
        lo += 2;
    }
    if hi < e {
        tail.push(PageItem::Gap);
        tail.push(PageItem::Page(e));
        hi = hi.saturating_sub(2);
    }
    let mut out = head;
    out.extend((lo..=hi).map(PageItem::Page));
    out.extend(tail);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn people() -> Candidates {
        Candidates {
            users: vec![
                (42, "Test User".into(), "testing".into(), "0".into(), None),
                (7, "bot".into(), "bot".into(), "0".into(), None),
            ],
            channels: vec![(5, "general".into())],
        }
    }

    #[test]
    fn filters_are_found_in_plain_text() {
        let text = "hello from: Test User in:#general has: image world";
        let parsed = parse(text, &people());
        assert_eq!(parsed.tokens.len(), 3);
        assert_eq!(parsed.tokens[0].answer.as_ref().unwrap().1.value, "42");
        assert_eq!(parsed.tokens[1].answer.as_ref().unwrap().1.value, "5");
        assert_eq!(parsed.tokens[2].answer.as_ref().unwrap().1.value, "image");
        assert_eq!(parsed.free_text(text), "hello world");
        let params = parsed.query(text).params();
        assert_eq!(params.author_ids, vec![42]);
        assert_eq!(params.channel_ids, vec![5]);
        assert_eq!(params.content, "hello world");
    }

    #[test]
    fn an_edited_answer_is_text_again() {
        let parsed = parse("from: Test Us", &people());
        assert!(parsed.tokens[0].answer.is_none());
        assert_eq!(
            parsed
                .pending("from: Test Us",)
                .map(|(k, _, partial)| (k, partial)),
            Some((FilterKind::From, "Test Us".to_string()))
        );

        let text = "from: testing";
        assert!(parse(text, &people()).pending(text).is_some());

        let text = "from: testing ";
        assert!(parse(text, &people()).pending(text).is_none());
    }

    #[test]
    fn dates_need_a_whole_date() {
        let text = "before: 2026-10-0";
        assert!(parse(text, &people()).tokens[0].answer.is_none());
        let text = "before: 2026-10-01 x";
        assert_eq!(
            parse(text, &people()).tokens[0]
                .answer
                .as_ref()
                .unwrap()
                .1
                .value,
            "2026-10-01"
        );
    }

    #[test]
    fn answers_replace_what_was_typed() {
        let answer = Answer {
            value: "42".into(),
            label: "Test User".into(),
        };
        assert_eq!(
            with_answer("hi from: te", Some(8), FilterKind::From, &answer),
            "hi from: Test User "
        );
        assert_eq!(
            with_answer("hi tes", None, FilterKind::From, &answer),
            "hi from: Test User "
        );
        assert_eq!(
            with_answer("tes", None, FilterKind::From, &answer),
            "from: Test User "
        );
    }

    #[test]
    fn a_query_round_trips_through_text() {
        let text = "from: Test User hello";
        let query = parse(text, &people()).query(text);
        assert_eq!(query_to_text(&query), "from: Test User hello");
    }

    #[test]
    fn page_list_matches_the_live_control() {
        assert_eq!(
            page_list(3, 1, 9),
            vec![PageItem::Page(1), PageItem::Page(2), PageItem::Page(3)]
        );

        let list = page_list(20, 1, 9);
        assert_eq!(list.last(), Some(&PageItem::Page(20)));
        assert_eq!(list[list.len() - 2], PageItem::Gap);
        assert_eq!(list.len(), 9);

        let list = page_list(20, 10, 9);
        assert_eq!(list.first(), Some(&PageItem::Page(1)));
        assert_eq!(list[1], PageItem::Gap);
        assert_eq!(list.last(), Some(&PageItem::Page(20)));
        assert_eq!(list.len(), 9);
    }

    #[test]
    fn day_bounds_make_snowflakes() {
        let (start, end) = day_bounds_ms("2026-10-01").unwrap();
        assert!(end > start);
        assert!(snowflake_at(start) < snowflake_at(end));
    }
}
