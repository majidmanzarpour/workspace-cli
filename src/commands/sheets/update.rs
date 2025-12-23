use crate::client::ApiClient;
use crate::error::Result;
use super::types::{ValueRange, UpdateValuesResponse, AppendValuesResponse};

pub struct UpdateParams {
    pub spreadsheet_id: String,
    pub range: String,
    pub values: Vec<Vec<serde_json::Value>>,
    pub value_input_option: ValueInputOption,
}

#[derive(Debug, Clone, Copy)]
pub enum ValueInputOption {
    Raw,
    UserEntered,
}

impl ValueInputOption {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Raw => "RAW",
            Self::UserEntered => "USER_ENTERED",
        }
    }
}

impl Default for ValueInputOption {
    fn default() -> Self {
        Self::UserEntered
    }
}

pub async fn update_values(
    client: &ApiClient,
    params: UpdateParams,
) -> Result<UpdateValuesResponse> {
    let path = format!(
        "/spreadsheets/{}/values/{}?valueInputOption={}",
        params.spreadsheet_id,
        urlencoding::encode(&params.range),
        params.value_input_option.as_str()
    );

    let body = ValueRange {
        range: params.range,
        major_dimension: Some("ROWS".to_string()),
        values: params.values,
    };

    client.put(&path, &body).await
}

pub async fn append_values(
    client: &ApiClient,
    spreadsheet_id: &str,
    range: &str,
    values: Vec<Vec<serde_json::Value>>,
    value_input_option: ValueInputOption,
) -> Result<AppendValuesResponse> {
    let path = format!(
        "/spreadsheets/{}/values/{}:append?valueInputOption={}&insertDataOption=INSERT_ROWS",
        spreadsheet_id,
        urlencoding::encode(range),
        value_input_option.as_str()
    );

    let body = ValueRange {
        range: range.to_string(),
        major_dimension: Some("ROWS".to_string()),
        values,
    };

    client.post(&path, &body).await
}

pub async fn clear_values(
    client: &ApiClient,
    spreadsheet_id: &str,
    range: &str,
) -> Result<serde_json::Value> {
    let path = format!(
        "/spreadsheets/{}/values/{}:clear",
        spreadsheet_id,
        urlencoding::encode(range)
    );

    client.post(&path, &serde_json::json!({})).await
}

/// Parse values from JSON string input (for CLI usage)
pub fn parse_values_json(json: &str) -> Result<Vec<Vec<serde_json::Value>>> {
    serde_json::from_str(json)
        .map_err(|e| crate::error::WorkspaceError::Config(format!("Invalid JSON values: {}", e)))
}
