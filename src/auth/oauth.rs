use std::path::Path;
use yup_oauth2::{
    authenticator::Authenticator,
    ApplicationSecret,
    InstalledFlowAuthenticator,
    InstalledFlowReturnMethod,
    ServiceAccountAuthenticator,
    hyper_rustls::HttpsConnector,
};

/// All scopes needed for Google Workspace APIs
pub const SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/gmail.modify",
    "https://www.googleapis.com/auth/drive",
    "https://www.googleapis.com/auth/calendar",
    "https://www.googleapis.com/auth/documents",
    "https://www.googleapis.com/auth/spreadsheets",
    "https://www.googleapis.com/auth/presentations",
    "https://www.googleapis.com/auth/tasks",
    "https://www.googleapis.com/auth/chat.spaces",
    "https://www.googleapis.com/auth/chat.messages",
    "https://www.googleapis.com/auth/chat.memberships",
    "https://www.googleapis.com/auth/contacts",
    "https://www.googleapis.com/auth/directory.readonly",
    "https://www.googleapis.com/auth/cloud-identity.groups.readonly",
    "https://www.googleapis.com/auth/admin.directory.group.readonly",
];

pub type WorkspaceAuthenticator = Authenticator<HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>;

/// Create an authenticator using OAuth2 installed application flow (interactive)
pub async fn create_installed_flow_auth(
    credentials_path: &Path,
    token_cache_path: &Path,
) -> Result<WorkspaceAuthenticator, AuthError> {
    let secret = read_application_secret(credentials_path).await?;

    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk(token_cache_path)
        .build()
        .await
        .map_err(|e| AuthError::FlowFailed(e.to_string()))?;

    Ok(auth)
}

/// Create an authenticator using service account (headless)
pub async fn create_service_account_auth(
    service_account_path: &Path,
) -> Result<WorkspaceAuthenticator, AuthError> {
    let sa_key = yup_oauth2::read_service_account_key(service_account_path)
        .await
        .map_err(|e| AuthError::InvalidCredentials(e.to_string()))?;

    let auth = ServiceAccountAuthenticator::builder(sa_key)
        .build()
        .await
        .map_err(|e| AuthError::FlowFailed(e.to_string()))?;

    Ok(auth)
}

/// Read OAuth2 application secret from credentials.json
async fn read_application_secret(path: &Path) -> Result<ApplicationSecret, AuthError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| AuthError::InvalidCredentials(format!("Failed to read credentials: {}", e)))?;

    let secret: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| AuthError::InvalidCredentials(format!("Invalid JSON: {}", e)))?;

    // Handle both "installed" and "web" application types
    let app_secret = if let Some(installed) = secret.get("installed") {
        parse_secret_object(installed)?
    } else if let Some(web) = secret.get("web") {
        parse_secret_object(web)?
    } else {
        return Err(AuthError::InvalidCredentials(
            "credentials.json must contain 'installed' or 'web' key".to_string()
        ));
    };

    Ok(app_secret)
}

fn parse_secret_object(obj: &serde_json::Value) -> Result<ApplicationSecret, AuthError> {
    let get_str = |key: &str| -> Result<String, AuthError> {
        obj.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AuthError::InvalidCredentials(format!("Missing field: {}", key)))
    };

    let get_str_opt = |key: &str| -> Option<String> {
        obj.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    };

    let get_vec = |key: &str| -> Vec<String> {
        obj.get(key)
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default()
    };

    let client_id = get_str("client_id")?;
    let client_secret = get_str("client_secret")?;

    // Validate client_id and client_secret are not empty
    if client_id.trim().is_empty() {
        return Err(AuthError::InvalidCredentials(
            "client_id cannot be empty".to_string()
        ));
    }
    if client_secret.trim().is_empty() {
        return Err(AuthError::InvalidCredentials(
            "client_secret cannot be empty".to_string()
        ));
    }

    Ok(ApplicationSecret {
        client_id,
        client_secret,
        auth_uri: get_str_opt("auth_uri").unwrap_or_else(|| "https://accounts.google.com/o/oauth2/auth".to_string()),
        token_uri: get_str_opt("token_uri").unwrap_or_else(|| "https://oauth2.googleapis.com/token".to_string()),
        redirect_uris: get_vec("redirect_uris"),
        project_id: get_str_opt("project_id"),
        client_email: get_str_opt("client_email"),
        auth_provider_x509_cert_url: get_str_opt("auth_provider_x509_cert_url"),
        client_x509_cert_url: get_str_opt("client_x509_cert_url"),
    })
}

/// Get an access token for the given scopes
pub async fn get_token(
    auth: &WorkspaceAuthenticator,
    scopes: &[&str],
) -> Result<String, AuthError> {
    let token = auth
        .token(scopes)
        .await
        .map_err(|e| AuthError::TokenFailed(e.to_string()))?;

    token
        .token()
        .map(|t| t.to_string())
        .ok_or_else(|| AuthError::TokenFailed("No access token in response".to_string()))
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    #[error("Authentication flow failed: {0}")]
    FlowFailed(String),

    #[error("Failed to get token: {0}")]
    TokenFailed(String),

    #[error("Token storage error: {0}")]
    StorageError(String),
}
