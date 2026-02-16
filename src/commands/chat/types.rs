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
    pub last_active_time: Option<String>,
    pub membership_count: Option<serde_json::Value>,
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

// Read state types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceReadState {
    pub name: Option<String>,
    pub last_read_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadReadState {
    pub name: Option<String>,
    pub last_read_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnreadSpace {
    pub space_name: Option<String>,
    pub display_name: Option<String>,
    pub space_type: Option<String>,
    pub last_read_time: Option<String>,
    #[serde(default)]
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnreadResult {
    #[serde(default)]
    pub spaces: Vec<UnreadSpace>,
    pub total_unread_spaces: usize,
    pub total_unread_messages: usize,
}

// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceNotificationSetting {
    pub name: Option<String>,
    pub notification_setting: Option<String>,
    pub mute_setting: Option<String>,
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
