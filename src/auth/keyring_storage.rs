use keyring::Entry;
use serde::{Deserialize, Serialize};

const SERVICE_NAME: &str = "workspace-cli";

/// Token data stored in keyring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>, // Unix timestamp
}

/// Keyring-based token storage
pub struct KeyringStorage {
    entry: Entry,
}

impl KeyringStorage {
    /// Create a new keyring storage for the given user/account
    pub fn new(account: &str) -> Result<Self, KeyringError> {
        let entry = Entry::new(SERVICE_NAME, account)
            .map_err(|e| KeyringError::InitFailed(e.to_string()))?;
        Ok(Self { entry })
    }

    /// Store token in keyring
    pub fn store(&self, token: &StoredToken) -> Result<(), KeyringError> {
        let json = serde_json::to_string(token)
            .map_err(|e| KeyringError::SerializationFailed(e.to_string()))?;

        self.entry
            .set_password(&json)
            .map_err(|e| KeyringError::StoreFailed(e.to_string()))
    }

    /// Retrieve token from keyring
    pub fn retrieve(&self) -> Result<StoredToken, KeyringError> {
        let json = self.entry
            .get_password()
            .map_err(|e| KeyringError::RetrieveFailed(e.to_string()))?;

        serde_json::from_str(&json)
            .map_err(|e| KeyringError::SerializationFailed(format!(
                "Failed to deserialize token from keyring (data may be corrupted): {}",
                e
            )))
    }

    /// Delete token from keyring
    pub fn delete(&self) -> Result<(), KeyringError> {
        self.entry
            .delete_credential()
            .map_err(|e| KeyringError::DeleteFailed(e.to_string()))
    }

    /// Check if a token exists
    pub fn exists(&self) -> bool {
        self.entry.get_password().is_ok()
    }
}

/// File-based fallback storage for environments without keyring
pub struct FileStorage {
    path: std::path::PathBuf,
}

impl FileStorage {
    /// Create a new file storage at the given path
    pub fn new(path: std::path::PathBuf) -> Self {
        Self { path }
    }

    /// Get default token file path
    pub fn default_path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|p| p.join("workspace-cli").join("tokens.json"))
    }

    /// Store token to file
    pub fn store(&self, token: &StoredToken) -> Result<(), KeyringError> {
        // Create parent directory if needed
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| KeyringError::StoreFailed(e.to_string()))?;
        }

        let json = serde_json::to_string_pretty(token)
            .map_err(|e| KeyringError::SerializationFailed(e.to_string()))?;

        // Write to file with restricted permissions (0600 = rw-------)
        std::fs::write(&self.path, json)
            .map_err(|e| KeyringError::StoreFailed(e.to_string()))?;

        // Set file permissions to user-only read/write (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&self.path, perms)
                .map_err(|e| KeyringError::StoreFailed(format!("Failed to set file permissions: {}", e)))?;
        }

        Ok(())
    }

    /// Retrieve token from file
    pub fn retrieve(&self) -> Result<StoredToken, KeyringError> {
        let json = std::fs::read_to_string(&self.path)
            .map_err(|e| KeyringError::RetrieveFailed(format!(
                "Failed to read token file at {}: {}",
                self.path.display(),
                e
            )))?;

        serde_json::from_str(&json)
            .map_err(|e| KeyringError::SerializationFailed(format!(
                "Failed to deserialize token from {} (file may be corrupted): {}",
                self.path.display(),
                e
            )))
    }

    /// Delete token file
    pub fn delete(&self) -> Result<(), KeyringError> {
        if self.path.exists() {
            std::fs::remove_file(&self.path)
                .map_err(|e| KeyringError::DeleteFailed(format!(
                    "Failed to delete {}: {}",
                    self.path.display(),
                    e
                )))?;
        }
        Ok(())
    }

    /// Check if token file exists
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

/// Combined storage that tries keyring first, falls back to file
pub struct TokenStorage {
    keyring: Option<KeyringStorage>,
    file: FileStorage,
}

impl TokenStorage {
    /// Create new token storage, trying keyring first
    pub fn new(account: &str) -> Self {
        let keyring = KeyringStorage::new(account).ok();

        // Use account-specific file path to support multiple accounts
        let file_path = if let Some(config_dir) = dirs::config_dir() {
            config_dir
                .join("workspace-cli")
                .join(format!("tokens_{}.json", account))
        } else {
            std::path::PathBuf::from(format!("tokens_{}.json", account))
        };

        let file = FileStorage::new(file_path);
        Self { keyring, file }
    }

    /// Store token (keyring preferred, file fallback)
    pub fn store(&self, token: &StoredToken) -> Result<(), KeyringError> {
        let mut keyring_success = false;
        let mut keyring_error = None;

        if let Some(ref kr) = self.keyring {
            match kr.store(token) {
                Ok(()) => {
                    keyring_success = true;
                    // Clean up file storage if keyring succeeded
                    let _ = self.file.delete();
                }
                Err(e) => {
                    keyring_error = Some(e);
                }
            }
        }

        if keyring_success {
            return Ok(());
        }

        // Fall back to file storage
        self.file.store(token).map_err(|file_err| {
            // If both keyring and file storage failed, provide detailed error
            if let Some(kr_err) = keyring_error {
                KeyringError::StoreFailed(format!(
                    "Keyring storage failed: {}. File storage also failed: {}",
                    kr_err, file_err
                ))
            } else {
                file_err
            }
        })
    }

    /// Retrieve token (keyring preferred, file fallback)
    pub fn retrieve(&self) -> Result<StoredToken, KeyringError> {
        if let Some(ref kr) = self.keyring {
            if let Ok(token) = kr.retrieve() {
                return Ok(token);
            }
        }
        self.file.retrieve()
    }

    /// Delete token from both storages
    pub fn delete(&self) -> Result<(), KeyringError> {
        let mut keyring_error = None;
        let mut file_error = None;

        if let Some(ref kr) = self.keyring {
            if let Err(e) = kr.delete() {
                keyring_error = Some(e);
            }
        }

        if let Err(e) = self.file.delete() {
            file_error = Some(e);
        }

        // Report errors if any occurred
        match (keyring_error, file_error) {
            (None, None) => Ok(()),
            (Some(kr_err), None) => Err(kr_err),
            (None, Some(f_err)) => Err(f_err),
            (Some(kr_err), Some(f_err)) => Err(KeyringError::DeleteFailed(format!(
                "Keyring delete failed: {}. File delete failed: {}",
                kr_err, f_err
            ))),
        }
    }

    /// Check if token exists in either storage
    pub fn exists(&self) -> bool {
        self.keyring.as_ref().map(|kr| kr.exists()).unwrap_or(false)
            || self.file.exists()
    }

    /// Check which storage is being used
    pub fn storage_type(&self) -> &'static str {
        if self.keyring.as_ref().map(|kr| kr.exists()).unwrap_or(false) {
            "keyring"
        } else if self.file.exists() {
            "file"
        } else {
            "none"
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeyringError {
    #[error("Failed to initialize keyring: {0}")]
    InitFailed(String),

    #[error("Failed to store token: {0}")]
    StoreFailed(String),

    #[error("Failed to retrieve token: {0}")]
    RetrieveFailed(String),

    #[error("Failed to delete token: {0}")]
    DeleteFailed(String),

    #[error("Serialization error: {0}")]
    SerializationFailed(String),
}
