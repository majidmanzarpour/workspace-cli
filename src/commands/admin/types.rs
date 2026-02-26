use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsersListResponse {
    #[serde(default)]
    pub users: Vec<User>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub primary_email: Option<String>,
    pub name: Option<UserName>,
    pub org_unit_path: Option<String>,
    pub is_admin: Option<bool>,
    pub suspended: Option<bool>,
    pub creation_time: Option<String>,
    pub last_login_time: Option<String>,
    pub is_delegated_admin: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserName {
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub full_name: Option<String>,
}
