use crate::client::ApiClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permission {
    pub id: Option<String>,
    pub r#type: String,
    pub role: String,
    pub email_address: Option<String>,
    pub domain: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionList {
    pub permissions: Vec<Permission>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreatePermissionRequest {
    r#type: String,
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    email_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<String>,
}

/// List permissions for a file
pub async fn list_permissions(client: &ApiClient, file_id: &str) -> Result<PermissionList> {
    let path = format!(
        "/files/{}/permissions?fields=permissions(id,type,role,emailAddress,domain,displayName)",
        urlencoding::encode(file_id)
    );
    client.get(&path).await
}

/// Share a file with a user
pub async fn share_with_user(
    client: &ApiClient,
    file_id: &str,
    email: &str,
    role: &str,
) -> Result<Permission> {
    let path = format!("/files/{}/permissions", urlencoding::encode(file_id));

    let request = CreatePermissionRequest {
        r#type: "user".to_string(),
        role: role.to_string(),
        email_address: Some(email.to_string()),
        domain: None,
    };

    client.post(&path, &request).await
}

/// Share a file with anyone who has the link
pub async fn share_with_anyone(
    client: &ApiClient,
    file_id: &str,
    role: &str,
) -> Result<Permission> {
    let path = format!("/files/{}/permissions", urlencoding::encode(file_id));

    let request = CreatePermissionRequest {
        r#type: "anyone".to_string(),
        role: role.to_string(),
        email_address: None,
        domain: None,
    };

    client.post(&path, &request).await
}

/// Share a file with a domain
pub async fn share_with_domain(
    client: &ApiClient,
    file_id: &str,
    domain: &str,
    role: &str,
) -> Result<Permission> {
    let path = format!("/files/{}/permissions", urlencoding::encode(file_id));

    let request = CreatePermissionRequest {
        r#type: "domain".to_string(),
        role: role.to_string(),
        email_address: None,
        domain: Some(domain.to_string()),
    };

    client.post(&path, &request).await
}

/// Remove a permission from a file
pub async fn remove_permission(
    client: &ApiClient,
    file_id: &str,
    permission_id: &str,
) -> Result<()> {
    let path = format!(
        "/files/{}/permissions/{}",
        urlencoding::encode(file_id),
        urlencoding::encode(permission_id)
    );
    client.delete(&path).await
}
