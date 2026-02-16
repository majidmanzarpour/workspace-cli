use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Space {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub space_type: Option<String>,
    pub space_uri: Option<String>,
    pub space_threading_state: Option<String>,
    pub single_user_bot_dm: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceListResponse {
    #[serde(default)]
    pub spaces: Vec<Space>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub name: Option<String>,
    pub sender: Option<User>,
    pub text: Option<String>,
    pub argument_text: Option<String>,
    pub create_time: Option<String>,
    pub thread: Option<Thread>,
    pub space: Option<SpaceRef>,
    pub formatted_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageListResponse {
    #[serde(default)]
    pub messages: Vec<Message>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub r#type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceRef {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Membership {
    pub name: Option<String>,
    pub member: Option<User>,
    pub role: Option<String>,
    pub state: Option<String>,
    pub create_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipListResponse {
    #[serde(default)]
    pub memberships: Vec<Membership>,
    pub next_page_token: Option<String>,
}

// Request types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMessageRequest {
    pub text: String,
    pub thread: Option<Thread>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupSpaceRequest {
    pub space: SpaceSetup,
    #[serde(default)]
    pub memberships: Vec<MembershipSetup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceSetup {
    pub display_name: Option<String>,
    pub space_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipSetup {
    pub member: MemberRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberRef {
    pub name: String,
    pub r#type: String,
}
