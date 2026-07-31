use std::sync::Mutex;

use keyring::{Entry, Error as KeyringError};
use serde::Serialize;

use crate::error::{AppError, AppResult};

const OPENAI_SERVICE: &str = "com.griffinboris.opentranscribe.openai";
const OPENAI_ACCOUNT: &str = "default";

#[derive(Clone, Debug, Serialize)]
pub struct CredentialStatus {
    pub configured: bool,
    pub masked_key: Option<String>,
}

enum CachedCredential {
    Unknown,
    Missing,
    Present(String),
}

pub struct OpenAiCredentials {
    cached: Mutex<CachedCredential>,
}

impl Default for OpenAiCredentials {
    fn default() -> Self {
        Self {
            cached: Mutex::new(CachedCredential::Unknown),
        }
    }
}

impl OpenAiCredentials {
    pub fn status(&self) -> AppResult<CredentialStatus> {
        match self.load()? {
            Some(key) => Ok(CredentialStatus {
                configured: true,
                masked_key: Some(mask_key(&key)),
            }),
            None => Ok(CredentialStatus {
                configured: false,
                masked_key: None,
            }),
        }
    }

    pub fn save(&self, key: &str) -> AppResult<CredentialStatus> {
        let key = key.trim();

        if key.is_empty() {
            return Err(AppError::Credential("API key cannot be empty".to_owned()));
        }

        entry()?.set_password(key).map_err(credential_error)?;
        *self.cached.lock().expect("credential cache lock poisoned") =
            CachedCredential::Present(key.to_owned());
        Ok(CredentialStatus {
            configured: true,
            masked_key: Some(mask_key(key)),
        })
    }

    pub fn remove(&self) -> AppResult<CredentialStatus> {
        match entry()?.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => {
                *self.cached.lock().expect("credential cache lock poisoned") =
                    CachedCredential::Missing;
                Ok(CredentialStatus {
                    configured: false,
                    masked_key: None,
                })
            }
            Err(error) => Err(credential_error(error)),
        }
    }

    pub fn read(&self) -> AppResult<String> {
        self.load()?.ok_or_else(|| {
            AppError::Credential("add an OpenAI API key in Settings first".to_owned())
        })
    }

    fn load(&self) -> AppResult<Option<String>> {
        self.load_with(|| match entry()?.get_password() {
            Ok(key) => Ok(Some(key)),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(error) => Err(credential_error(error)),
        })
    }

    fn load_with(
        &self,
        load_from_vault: impl FnOnce() -> AppResult<Option<String>>,
    ) -> AppResult<Option<String>> {
        let mut cached = self.cached.lock().expect("credential cache lock poisoned");

        match &*cached {
            CachedCredential::Present(key) => return Ok(Some(key.clone())),
            CachedCredential::Missing => return Ok(None),
            CachedCredential::Unknown => {}
        }

        match load_from_vault()? {
            Some(key) => {
                *cached = CachedCredential::Present(key.clone());
                Ok(Some(key))
            }
            None => {
                *cached = CachedCredential::Missing;
                Ok(None)
            }
        }
    }
}

fn entry() -> AppResult<Entry> {
    Entry::new(OPENAI_SERVICE, OPENAI_ACCOUNT).map_err(credential_error)
}

fn mask_key(key: &str) -> String {
    let ending: String = key
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("•••• {ending}")
}

fn credential_error(error: KeyringError) -> AppError {
    let message = match error {
        KeyringError::NoStorageAccess(details) => {
            format!("unlock the operating system credential vault and try again: {details}")
        }
        KeyringError::NoDefaultStore => {
            "the operating system credential vault is unavailable; Linux requires a Secret Service-compatible keyring"
                .to_owned()
        }
        other => other.to_string(),
    };
    AppError::Credential(message)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use keyring::Error as KeyringError;

    use super::{OpenAiCredentials, credential_error, mask_key};

    #[test]
    fn masks_all_but_the_last_four_characters() {
        assert_eq!(mask_key("sk-project-123456"), "•••• 3456");
    }

    #[test]
    fn reads_the_platform_vault_once_per_process() {
        let credentials = OpenAiCredentials::default();
        let reads = AtomicUsize::new(0);

        assert_eq!(
            credentials
                .load_with(|| {
                    reads.fetch_add(1, Ordering::Relaxed);
                    Ok(Some("sk-project-123456".to_owned()))
                })
                .unwrap(),
            Some("sk-project-123456".to_owned())
        );
        assert_eq!(
            credentials
                .load_with(|| {
                    reads.fetch_add(1, Ordering::Relaxed);
                    Ok(Some("different".to_owned()))
                })
                .unwrap(),
            Some("sk-project-123456".to_owned())
        );
        assert_eq!(reads.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn caches_an_unconfigured_platform_vault() {
        let credentials = OpenAiCredentials::default();
        let reads = AtomicUsize::new(0);

        for _ in 0..2 {
            assert_eq!(
                credentials
                    .load_with(|| {
                        reads.fetch_add(1, Ordering::Relaxed);
                        Ok(None)
                    })
                    .unwrap(),
                None
            );
        }

        assert_eq!(reads.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn explains_the_linux_secret_service_requirement() {
        assert!(
            credential_error(KeyringError::NoDefaultStore)
                .to_string()
                .contains("Secret Service-compatible keyring")
        );
    }
}
