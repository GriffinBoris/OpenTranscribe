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

pub struct OpenAiCredentials;

impl OpenAiCredentials {
    pub fn status() -> AppResult<CredentialStatus> {
        match entry()?.get_password() {
            Ok(key) => Ok(CredentialStatus {
                configured: true,
                masked_key: Some(mask_key(&key)),
            }),
            Err(KeyringError::NoEntry) => Ok(CredentialStatus {
                configured: false,
                masked_key: None,
            }),
            Err(error) => Err(credential_error(error)),
        }
    }

    pub fn save(key: &str) -> AppResult<CredentialStatus> {
        let key = key.trim();

        if key.is_empty() {
            return Err(AppError::Credential("API key cannot be empty".to_owned()));
        }

        entry()?.set_password(key).map_err(credential_error)?;
        Ok(CredentialStatus {
            configured: true,
            masked_key: Some(mask_key(key)),
        })
    }

    pub fn remove() -> AppResult<CredentialStatus> {
        match entry()?.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(CredentialStatus {
                configured: false,
                masked_key: None,
            }),
            Err(error) => Err(credential_error(error)),
        }
    }

    pub fn read() -> AppResult<String> {
        entry()?.get_password().map_err(|error| match error {
            KeyringError::NoEntry => {
                AppError::Credential("add an OpenAI API key in Settings first".to_owned())
            }
            other => credential_error(other),
        })
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
    AppError::Credential(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::mask_key;

    #[test]
    fn masks_all_but_the_last_four_characters() {
        assert_eq!(mask_key("sk-project-123456"), "•••• 3456");
    }
}
