use crate::client::ApiClient;
use crate::error::Result;
use super::types::{Space, SpaceListResponse, SetupSpaceRequest, SpaceSetup, MembershipSetup, MemberRef};

pub struct ListSpacesParams {
    pub page_size: u32,
    pub page_token: Option<String>,
}

impl Default for ListSpacesParams {
    fn default() -> Self {
        Self { page_size: 100, page_token: None }
    }
}

pub async fn list_spaces(client: &ApiClient, params: ListSpacesParams) -> Result<SpaceListResponse> {
    let mut query: Vec<(&str, String)> = vec![
        ("pageSize", params.page_size.to_string()),
    ];
    if let Some(ref token) = params.page_token {
        query.push(("pageToken", token.clone()));
    }
    client.get_with_query("/spaces", &query).await
}

pub async fn get_space(client: &ApiClient, space_name: &str) -> Result<Space> {
    let path = if space_name.starts_with("spaces/") {
        space_name.to_string()
    } else {
        format!("spaces/{}", space_name)
    };
    client.get(&format!("/{}", path)).await
}

pub async fn create_space(client: &ApiClient, display_name: &str, member_emails: &[String]) -> Result<Space> {
    let memberships: Vec<MembershipSetup> = member_emails.iter().map(|email| {
        MembershipSetup {
            member: MemberRef {
                name: format!("users/{}", email),
                r#type: "HUMAN".to_string(),
            },
        }
    }).collect();

    let body = SetupSpaceRequest {
        space: SpaceSetup {
            display_name: Some(display_name.to_string()),
            space_type: "SPACE".to_string(),
        },
        memberships,
    };

    client.post("/spaces:setup", &body).await
}

pub async fn find_space_by_name(client: &ApiClient, name: &str) -> Result<Vec<Space>> {
    let response: SpaceListResponse = list_spaces(client, ListSpacesParams { page_size: 200, page_token: None }).await?;
    let name_lower = name.to_lowercase();
    let matches: Vec<Space> = response.spaces.into_iter()
        .filter(|s| s.display_name.as_ref().map(|n| n.to_lowercase().contains(&name_lower)).unwrap_or(false))
        .collect();
    Ok(matches)
}
