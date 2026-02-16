use serde::{Deserialize, Serialize};

// Admin Directory API groups list response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryGroupsResponse {
    #[serde(default)]
    pub groups: Vec<DirectoryGroup>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryGroup {
    pub id: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub direct_members_count: Option<String>,
    pub admin_created: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityKey {
    pub id: Option<String>,
    pub namespace: Option<String>,
}

// Group lookup response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LookupGroupResponse {
    pub name: Option<String>,
}

// Group members response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipsResponse {
    #[serde(default)]
    pub memberships: Vec<Membership>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Membership {
    pub name: Option<String>,
    pub preferred_member_key: Option<EntityKey>,
    pub r#type: Option<String>,
    #[serde(default)]
    pub roles: Vec<MembershipRole>,
    pub create_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipRole {
    pub name: Option<String>,
}

impl Membership {
    /// Get the highest role (OWNER > MANAGER > MEMBER)
    pub fn primary_role(&self) -> &str {
        for role in &self.roles {
            if let Some(ref name) = role.name {
                if name == "OWNER" {
                    return "OWNER";
                }
            }
        }
        for role in &self.roles {
            if let Some(ref name) = role.name {
                if name == "MANAGER" {
                    return "MANAGER";
                }
            }
        }
        "MEMBER"
    }
}
