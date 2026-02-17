use std::path::PathBuf;
use crate::config::Config;
use super::oauth::{self, AuthError, WorkspaceAuthenticator, SCOPES};
use super::keyring_storage::{KeyringError, TokenStorage, StoredToken};

/// Manages authentication and token lifecycle
pub struct TokenManager {
    authenticator: Option<WorkspaceAuthenticator>,
    storage: TokenStorage,
    config: Config,
    credentials_path: Option<PathBuf>,
    /// Email to impersonate via domain-wide delegation (service account only)
    subject: Option<String>,
}

impl TokenManager {
    /// Create a new token manager with the given config
    pub fn new(config: Config) -> Self {
        Self {
            authenticator: None,
            storage: TokenStorage::new("default"),
            credentials_path: None,
            subject: config.auth.impersonate_subject.clone(),
            config,
        }
    }

    /// Set the email to impersonate via domain-wide delegation
    pub fn set_subject(&mut self, subject: Option<String>) {
        self.subject = subject;
    }

    /// Try to restore authenticator from cached tokens
    /// Call this before making API requests
    /// When subject is set, automatically uses service account flow for domain-wide delegation
    pub async fn ensure_authenticated(&mut self) -> Result<(), TokenManagerError> {
        // Already have an authenticator
        if self.authenticator.is_some() {
            // Validate that the authenticator can still get tokens
            // This checks for expiry and refreshes if needed
            if let Ok(_) = self.get_access_token().await {
                return Ok(());
            }
            // If token fetch fails, clear the authenticator and retry
            self.authenticator = None;
        }

        // If impersonation is requested, use service account flow
        if self.subject.is_some() {
            return self.login_service_account(None).await;
        }

        let token_cache = self.token_cache_path();

        // Check if token cache exists
        if !token_cache.exists() {
            return Err(TokenManagerError::NotAuthenticated);
        }

        // Try to find credentials path
        let creds_path = self.credentials_path.clone()
            .or_else(|| self.config.auth.credentials_path.clone())
            .or_else(|| self.find_credentials_file());

        let creds_path = creds_path.ok_or(TokenManagerError::MissingCredentials(
            "No credentials file found. Run 'workspace-cli auth login --credentials <path>' first.".to_string()
        ))?;

        // Restore authenticator from cached tokens
        let auth = oauth::create_installed_flow_auth(&creds_path, &token_cache)
            .await
            .map_err(TokenManagerError::Auth)?;

        // Verify we can get a token before considering authentication successful
        oauth::get_token(&auth, SCOPES)
            .await
            .map_err(TokenManagerError::Auth)?;

        self.authenticator = Some(auth);
        self.credentials_path = Some(creds_path);
        Ok(())
    }

    /// Find credentials file in common locations
    fn find_credentials_file(&self) -> Option<PathBuf> {
        let candidates = [
            // Current directory
            PathBuf::from("credentials.json"),
            // Config directory
            Config::config_dir().map(|d| d.join("credentials.json")).unwrap_or_default(),
            // Home directory
            dirs::home_dir().map(|d| d.join("credentials.json")).unwrap_or_default(),
            dirs::home_dir().map(|d| d.join(".credentials.json")).unwrap_or_default(),
        ];

        candidates.into_iter().find(|p| p.exists())
    }

    /// Initialize with interactive OAuth2 flow
    pub async fn login_interactive(&mut self, credentials_path: Option<PathBuf>) -> Result<(), TokenManagerError> {
        let creds_path = credentials_path
            .or_else(|| self.config.auth.credentials_path.clone())
            .ok_or_else(|| TokenManagerError::MissingCredentials(
                "No credentials path provided. Use --credentials or set WORKSPACE_CREDENTIALS_PATH".to_string()
            ))?;

        // Validate that credentials file exists
        if !creds_path.exists() {
            return Err(TokenManagerError::MissingCredentials(
                format!("Credentials file not found at: {}", creds_path.display())
            ));
        }

        let token_cache = self.token_cache_path();

        // Ensure the config directory exists
        if let Some(parent) = token_cache.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                TokenManagerError::MissingCredentials(format!("Failed to create config directory: {}", e))
            })?;
        }

        let auth = oauth::create_installed_flow_auth(&creds_path, &token_cache)
            .await
            .map_err(TokenManagerError::Auth)?;

        // Test that we can get a token
        let token = oauth::get_token(&auth, SCOPES)
            .await
            .map_err(TokenManagerError::Auth)?;

        // Store token info with current timestamp + estimated expiry (3600 seconds is typical)
        let expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|d| (d.as_secs() + 3600) as i64);

        self.storage.store(&StoredToken {
            access_token: token.clone(),
            refresh_token: None, // yup-oauth2 handles refresh internally
            expires_at,
        }).map_err(TokenManagerError::Storage)?;

        self.authenticator = Some(auth);
        self.credentials_path = Some(creds_path);
        Ok(())
    }

    /// Initialize with service account
    pub async fn login_service_account(&mut self, sa_path: Option<PathBuf>) -> Result<(), TokenManagerError> {
        let sa_path = sa_path
            .or_else(|| self.config.auth.service_account_path.clone())
            .or_else(|| std::env::var("GOOGLE_APPLICATION_CREDENTIALS").ok().map(PathBuf::from))
            .ok_or_else(|| TokenManagerError::MissingCredentials(
                "No service account path provided. Set GOOGLE_APPLICATION_CREDENTIALS".to_string()
            ))?;

        // Validate that service account file exists
        if !sa_path.exists() {
            return Err(TokenManagerError::MissingCredentials(
                format!("Service account file not found at: {}", sa_path.display())
            ));
        }

        let auth = oauth::create_service_account_auth(&sa_path, self.subject.as_deref())
            .await
            .map_err(TokenManagerError::Auth)?;

        self.authenticator = Some(auth);
        Ok(())
    }

    /// Get an access token for API calls
    pub async fn get_access_token(&self) -> Result<String, TokenManagerError> {
        let auth = self.authenticator.as_ref()
            .ok_or(TokenManagerError::NotAuthenticated)?;

        oauth::get_token(auth, SCOPES)
            .await
            .map_err(TokenManagerError::Auth)
    }

    /// Get token for specific scopes
    pub async fn get_token_for_scopes(&self, scopes: &[&str]) -> Result<String, TokenManagerError> {
        let auth = self.authenticator.as_ref()
            .ok_or(TokenManagerError::NotAuthenticated)?;

        oauth::get_token(auth, scopes)
            .await
            .map_err(TokenManagerError::Auth)
    }

    /// Check if we have stored credentials
    pub fn is_authenticated(&self) -> bool {
        self.authenticator.is_some() || self.token_cache_path().exists()
    }

    /// Clear all stored tokens (logout)
    pub fn logout(&mut self) -> Result<(), TokenManagerError> {
        // Clear the authenticator to free resources
        self.authenticator = None;
        self.credentials_path = None;

        self.storage.delete().map_err(TokenManagerError::Storage)?;

        // Also try to remove the token cache file
        let cache_path = self.token_cache_path();
        if cache_path.exists() {
            std::fs::remove_file(cache_path)
                .map_err(|e| TokenManagerError::Storage(
                    KeyringError::DeleteFailed(format!("Failed to remove token cache: {}", e))
                ))?;
        }

        Ok(())
    }

    /// Get authentication status info
    pub fn status(&self) -> AuthStatus {
        AuthStatus {
            authenticated: self.is_authenticated(),
            storage_type: self.storage.storage_type().to_string(),
            token_cache_path: self.token_cache_path(),
        }
    }

    /// Get the token cache file path
    fn token_cache_path(&self) -> PathBuf {
        Config::config_dir()
            .map(|d| d.join("token_cache.json"))
            .unwrap_or_else(|| PathBuf::from("token_cache.json"))
    }
}

/// Authentication status information
#[derive(Debug, Clone, serde::Serialize)]
pub struct AuthStatus {
    pub authenticated: bool,
    pub storage_type: String,
    pub token_cache_path: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum TokenManagerError {
    #[error("Not authenticated. Run 'workspace-cli auth login' first.")]
    NotAuthenticated,

    #[error("Missing credentials: {0}")]
    MissingCredentials(String),

    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("Token storage error: {0}")]
    Storage(#[from] KeyringError),
}
