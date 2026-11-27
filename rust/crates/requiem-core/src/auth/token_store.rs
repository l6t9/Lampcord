use keyring::Entry;
use thiserror::Error;

const SERVICE: &str = "requiem";

#[derive(Debug, Error)]
pub enum TokenStoreError {
    #[error("keyring error: {0}")]
    Keyring(#[from] keyring::Error),
}

pub struct TokenStore {
    service: &'static str,
    account: String,
}

impl TokenStore {
    pub fn new(account: impl Into<String>) -> Self {
        Self::in_service(SERVICE, account)
    }

    pub(crate) fn in_service(service: &'static str, account: impl Into<String>) -> Self {
        Self {
            service,
            account: account.into(),
        }
    }

    fn entry(&self) -> Result<Entry, TokenStoreError> {
        Ok(Entry::new(self.service, &self.account)?)
    }

    pub fn save(&self, token: &str) -> Result<(), TokenStoreError> {
        self.entry()?.set_password(token)?;
        Ok(())
    }

    pub fn load(&self) -> Result<Option<String>, TokenStoreError> {
        match self.entry()?.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub fn clear(&self) -> Result<(), TokenStoreError> {
        match self.entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
}
