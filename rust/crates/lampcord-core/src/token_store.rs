use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedAccount {
    pub token: String,
    pub user_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TokenStore {
    accounts: Vec<SavedAccount>,
}

impl TokenStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_json(body: &str) -> Self {
        serde_json::from_str(body).unwrap_or_default()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "[]".to_owned())
    }

    pub fn accounts(&self) -> &[SavedAccount] {
        &self.accounts
    }

    pub fn add(&mut self, account: SavedAccount) {
        self.accounts
            .retain(|existing| existing.user_id != account.user_id);
        self.accounts.push(account);
    }

    pub fn remove(&mut self, token: &str) {
        self.accounts.retain(|account| account.token != token);
    }

    pub fn find(&self, token: &str) -> Option<&SavedAccount> {
        self.accounts.iter().find(|account| account.token == token)
    }

    pub fn contains(&self, token: &str) -> bool {
        self.find(token).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(token: &str, user_id: &str) -> SavedAccount {
        SavedAccount {
            token: token.to_owned(),
            user_id: user_id.to_owned(),
            username: None,
        }
    }

    #[test]
    fn add_replaces_the_same_user() {
        let mut store = TokenStore::new();
        store.add(account("old", "123"));
        store.add(account("new", "123"));
        assert_eq!(store.accounts().len(), 1);
        assert_eq!(
            store.find("new").map(|account| account.user_id.as_str()),
            Some("123")
        );
        assert!(!store.contains("old"));
    }

    #[test]
    fn remove_uses_exact_token() {
        let mut store = TokenStore::new();
        store.add(account("one", "123"));
        store.add(account("two", "456"));
        store.remove("one");
        assert_eq!(store.accounts().len(), 1);
        assert!(store.contains("two"));
    }

    #[test]
    fn json_round_trip_preserves_accounts() {
        let mut store = TokenStore::new();
        store.add(account("one", "123"));
        let restored = TokenStore::from_json(&store.to_json());
        assert_eq!(restored, store);
    }
}
