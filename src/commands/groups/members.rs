use crate::client::ApiClient;
use crate::error::Result;
use super::types::{LookupGroupResponse, MembershipsResponse};

/// Look up a group's resource name by email
pub async fn lookup_group(client: &ApiClient, group_email: &str) -> Result<LookupGroupResponse> {
    let query = vec![("groupKey.id", group_email.to_string())];
    client.get_with_query("/groups:lookup", &query).await
}

pub struct ListMembersParams {
    pub group_name: String,
    pub page_size: u32,
    pub page_token: Option<String>,
}

/// List members of a group by its resource name
pub async fn list_members(client: &ApiClient, params: ListMembersParams) -> Result<MembershipsResponse> {
    let mut query: Vec<(&str, String)> = vec![
        ("pageSize", params.page_size.to_string()),
    ];
    if let Some(ref token) = params.page_token {
        query.push(("pageToken", token.clone()));
    }
    let path = format!("/{}/memberships", params.group_name);
    client.get_with_query(&path, &query).await
}

/// List members of a group by email (two-step: lookup + list)
pub async fn list_members_by_email(client: &ApiClient, group_email: &str, page_size: u32) -> Result<MembershipsResponse> {
    let lookup = lookup_group(client, group_email).await?;
    let group_name = lookup.name.ok_or_else(|| {
        crate::error::WorkspaceError::NotFound(format!("Group not found: {}", group_email))
    })?;
    let params = ListMembersParams {
        group_name,
        page_size,
        page_token: None,
    };
    list_members(client, params).await
}
