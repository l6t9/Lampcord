#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stack {
    Desktop,

    Web,

    Plain,
}

impl Stack {
    pub const ORDER: [Stack; 3] = [Stack::Desktop, Stack::Web, Stack::Plain];

    pub fn query(self) -> &'static str {
        match self {
            Stack::Desktop => "encoding=etf&v=9&compress=zstd-stream",
            Stack::Web => "encoding=json&v=9&compress=zlib-stream",
            Stack::Plain => "encoding=json&v=9",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Stack::Desktop => "desktop",
            Stack::Web => "web",
            Stack::Plain => "plain",
        }
    }

    fn from_key(key: &str) -> Option<Stack> {
        Stack::ORDER.into_iter().find(|s| s.key() == key)
    }
}

pub fn is_fixed(url: &str) -> bool {
    url.split_once('?')
        .is_some_and(|(_, query)| query.split('&').any(|pair| pair.starts_with("encoding=")))
}

pub fn url_for(base: &str, stack: Stack) -> String {
    format!("{}/?{}", base.trim_end_matches('/'), stack.query())
}

fn meta_key(base: &str) -> String {
    format!("gateway_stack:{base}")
}

pub fn remembered(base: &str) -> Option<Stack> {
    requiem_db::Db::shared()
        .meta(&meta_key(base))
        .and_then(|key| Stack::from_key(&key))
}

pub fn candidates(base: &str) -> Vec<Stack> {
    ordered(remembered(base))
}

fn ordered(remembered: Option<Stack>) -> Vec<Stack> {
    let mut list: Vec<Stack> = remembered.into_iter().collect();
    list.extend(Stack::ORDER.into_iter().filter(|s| Some(*s) != remembered));
    list
}

pub fn remember(base: &str, stack: Stack) {
    let db = requiem_db::Db::shared();
    if db.meta(&meta_key(base)).as_deref() != Some(stack.key()) {
        let _ = db.set_meta(&meta_key(base), stack.key());
    }
}

pub fn forget(base: &str) {
    let _ = requiem_db::Db::shared().with(|conn| {
        conn.execute("DELETE FROM meta WHERE key = ?1", [meta_key(base)])
            .map(drop)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_official_url_is_fixed_and_a_bare_address_is_not() {
        assert!(is_fixed(
            "wss://gateway.discord.gg/?encoding=etf&v=9&compress=zstd-stream"
        ));
        assert!(!is_fixed("wss://gateway.mee6isgoinggreat.com"));
        assert!(!is_fixed("wss://gateway.example.com/?v=9"));
    }

    #[test]
    fn urls_carry_the_stacks_query_and_the_remembered_one_goes_first() {
        assert_eq!(
            url_for("wss://gateway.example.com", Stack::Web),
            "wss://gateway.example.com/?encoding=json&v=9&compress=zlib-stream"
        );
        assert_eq!(
            url_for("wss://gateway.example.com/", Stack::Plain),
            "wss://gateway.example.com/?encoding=json&v=9"
        );
        assert_eq!(
            ordered(None),
            vec![Stack::Desktop, Stack::Web, Stack::Plain]
        );
        assert_eq!(
            ordered(Some(Stack::Web)),
            vec![Stack::Web, Stack::Desktop, Stack::Plain]
        );
    }
}
