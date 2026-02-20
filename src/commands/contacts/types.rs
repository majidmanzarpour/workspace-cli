use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub resource_name: Option<String>,
    pub etag: Option<String>,
    #[serde(default)]
    pub names: Vec<Name>,
    #[serde(default)]
    pub email_addresses: Vec<EmailAddress>,
    #[serde(default)]
    pub phone_numbers: Vec<PhoneNumber>,
    #[serde(default)]
    pub organizations: Vec<Organization>,
    #[serde(default)]
    pub urls: Vec<Url>,
    #[serde(default)]
    pub birthdays: Vec<Birthday>,
    #[serde(default)]
    pub biographies: Vec<Biography>,
    #[serde(default)]
    pub addresses: Vec<Address>,
    #[serde(default)]
    pub user_defined: Vec<UserDefined>,
    pub metadata: Option<PersonMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Name {
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub display_name: Option<String>,
    pub display_name_last_first: Option<String>,
    pub metadata: Option<FieldMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailAddress {
    pub value: Option<String>,
    #[serde(rename = "type")]
    pub email_type: Option<String>,
    pub formatted_type: Option<String>,
    pub metadata: Option<FieldMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhoneNumber {
    pub value: Option<String>,
    #[serde(rename = "type")]
    pub phone_type: Option<String>,
    pub formatted_type: Option<String>,
    pub metadata: Option<FieldMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub name: Option<String>,
    pub title: Option<String>,
    pub department: Option<String>,
    pub metadata: Option<FieldMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Url {
    pub value: Option<String>,
    #[serde(rename = "type")]
    pub url_type: Option<String>,
    pub metadata: Option<FieldMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Birthday {
    pub date: Option<DateValue>,
    pub text: Option<String>,
    pub metadata: Option<FieldMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateValue {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Biography {
    pub value: Option<String>,
    pub content_type: Option<String>,
    pub metadata: Option<FieldMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub formatted_value: Option<String>,
    #[serde(rename = "type")]
    pub address_type: Option<String>,
    pub street_address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub metadata: Option<FieldMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDefined {
    pub key: Option<String>,
    pub value: Option<String>,
    pub metadata: Option<FieldMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldMetadata {
    pub primary: Option<bool>,
    pub verified: Option<bool>,
    pub source: Option<Source>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    #[serde(rename = "type")]
    pub source_type: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonMetadata {
    #[serde(default)]
    pub sources: Vec<Source>,
}

// Response types

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionsResponse {
    #[serde(default)]
    pub connections: Vec<Person>,
    pub next_page_token: Option<String>,
    pub total_people: Option<i32>,
    pub total_items: Option<i32>,
    pub next_sync_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    #[serde(default)]
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub person: Option<Person>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryPeopleResponse {
    #[serde(default)]
    pub people: Vec<Person>,
    pub next_page_token: Option<String>,
    pub total_size: Option<i32>,
}

// Request body for creating/updating contacts

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContactRequest {
    #[serde(default)]
    pub names: Vec<Name>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub email_addresses: Vec<EmailAddress>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phone_numbers: Vec<PhoneNumber>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub organizations: Vec<Organization>,
}

pub const READ_MASK: &str = "names,emailAddresses,phoneNumbers,organizations,urls,birthdays,biographies,addresses,userDefined,metadata";
pub const DIRECTORY_READ_MASK: &str = "names,emailAddresses";
