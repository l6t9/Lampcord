use std::cmp::Reverse;

pub const MAX_SAMPLES: usize = 10;

const DAY_MS: u64 = 86_400_000;

pub const DEFAULT_REACTIONS: [&str; 13] = [
    "100",
    "100",
    "thumbsup",
    "thumbsup",
    "thumbsdown",
    "thumbsdown",
    "heart",
    "point_up",
    "eyes",
    "weary",
    "laughing",
    "white_check_mark",
    "x",
];

pub const DEFAULT_USED: [&str; 10] = [
    "thumbsup",
    "eyes",
    "laughing",
    "watermelon",
    "fork_and_knife",
    "yum",
    "weary",
    "tired_face",
    "poop",
    "100",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub total_uses: u32,

    pub recent_uses: Vec<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Frecency {
    items: Vec<(String, Item)>,
}

fn weight(days: u64) -> u64 {
    match days {
        0..=3 => 100,
        4..=15 => 70,
        16..=30 => 50,
        31..=45 => 30,
        46..=80 => 10,
        _ => 1,
    }
}

impl Item {
    pub fn score(&self, now_ms: u64) -> u64 {
        self.recent_uses
            .iter()
            .take(MAX_SAMPLES)
            .map(|&at| weight(now_ms.saturating_sub(at) / DAY_MS))
            .sum()
    }

    pub fn frecency(&self, now_ms: u64) -> u64 {
        let score = self.score(now_ms);
        if score == 0 || self.recent_uses.is_empty() {
            return 0;
        }
        (u64::from(self.total_uses) * score).div_ceil(self.recent_uses.len() as u64)
    }
}

impl Frecency {
    pub fn from_items(items: Vec<(String, Item)>) -> Self {
        Self { items }
    }

    pub fn items(&self) -> &[(String, Item)] {
        &self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn default_reactions(now_ms: u64) -> Self {
        let mut frecency = Self::default();
        for key in DEFAULT_REACTIONS {
            frecency.track(key, now_ms);
        }
        frecency
    }

    pub fn default_used(now_ms: u64) -> Self {
        let mut frecency = Self::default();
        for (rank, key) in DEFAULT_USED.iter().enumerate() {
            for _ in 0..DEFAULT_USED.len() - rank {
                frecency.track(key, now_ms);
            }
        }
        frecency
    }

    pub fn track(&mut self, key: &str, now_ms: u64) {
        match self.items.iter_mut().find(|(k, _)| k == key) {
            Some((_, item)) => {
                item.total_uses += 1;
                item.recent_uses.push(now_ms);
                item.recent_uses.sort_unstable();
                while item.recent_uses.len() > MAX_SAMPLES {
                    item.recent_uses.remove(0);
                }
            }
            None => self.items.push((
                key.to_string(),
                Item {
                    total_uses: 1,
                    recent_uses: vec![now_ms],
                },
            )),
        }
    }

    pub fn frequently(&self, now_ms: u64, limit: usize) -> Vec<String> {
        let mut ranked: Vec<(&str, u64)> = self
            .items
            .iter()
            .map(|(k, item)| (k.as_str(), item.frecency(now_ms)))
            .collect();
        ranked.sort_by_key(|a| Reverse(a.1));
        ranked
            .into_iter()
            .take(limit)
            .map(|(k, _)| k.to_string())
            .collect()
    }

    pub fn top(&self, now_ms: u64, limit: usize) -> Vec<(String, Item)> {
        let keep = self.frequently(now_ms, limit);
        self.items
            .iter()
            .filter(|(k, _)| keep.contains(k))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: u64 = 1_800_000_000_000;

    #[test]
    fn a_fresh_account_gets_100_thumbsup_thumbsdown_heart() {
        let frecency = Frecency::default_reactions(NOW);
        assert_eq!(
            frecency.frequently(NOW, 4),
            ["100", "thumbsup", "thumbsdown", "heart"]
        );
    }

    #[test]
    fn using_an_emoji_again_moves_it_up() {
        let mut frecency = Frecency::default_reactions(NOW);
        frecency.track("eyes", NOW);
        frecency.track("eyes", NOW);
        frecency.track("eyes", NOW);
        assert_eq!(frecency.frequently(NOW, 1), ["eyes"]);
    }

    #[test]
    fn old_uses_count_for_less() {
        let mut old = Frecency::default();
        old.track("a", NOW - 100 * DAY_MS);
        old.track("a", NOW - 100 * DAY_MS);
        let mut recent = Frecency::default();
        recent.track("a", NOW);
        recent.track("a", NOW);
        assert!(recent.items()[0].1.frecency(NOW) > old.items()[0].1.frecency(NOW));
        assert_eq!(old.items()[0].1.frecency(NOW), 2);
        assert_eq!(recent.items()[0].1.frecency(NOW), 200);
    }

    #[test]
    fn only_the_last_ten_uses_are_kept() {
        let mut frecency = Frecency::default();
        for i in 0..15 {
            frecency.track("a", NOW + i);
        }
        let item = &frecency.items()[0].1;
        assert_eq!(item.total_uses, 15);
        assert_eq!(item.recent_uses.len(), MAX_SAMPLES);
        assert_eq!(item.recent_uses[0], NOW + 5);
    }

    #[test]
    fn age_buckets() {
        assert_eq!(
            [
                weight(0),
                weight(3),
                weight(4),
                weight(15),
                weight(16),
                weight(30),
                weight(45),
                weight(80),
                weight(81)
            ],
            [100, 100, 70, 70, 50, 50, 30, 10, 1]
        );
    }
}
