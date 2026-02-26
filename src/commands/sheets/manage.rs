use crate::client::ApiClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchUpdateRequest {
    requests: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchUpdateResponse {
    pub spreadsheet_id: String,
    #[serde(default)]
    pub replies: Vec<serde_json::Value>,
}

/// Add a new sheet tab to an existing spreadsheet
pub async fn add_sheet(client: &ApiClient, spreadsheet_id: &str, title: &str) -> Result<BatchUpdateResponse> {
    let path = format!("/spreadsheets/{}:batchUpdate", spreadsheet_id);
    let request = BatchUpdateRequest {
        requests: vec![serde_json::json!({
            "addSheet": {
                "properties": {
                    "title": title
                }
            }
        })],
    };
    client.post(&path, &request).await
}

/// Delete a sheet tab by its sheet ID (not title)
pub async fn delete_sheet(client: &ApiClient, spreadsheet_id: &str, sheet_id: i64) -> Result<BatchUpdateResponse> {
    let path = format!("/spreadsheets/{}:batchUpdate", spreadsheet_id);
    let request = BatchUpdateRequest {
        requests: vec![serde_json::json!({
            "deleteSheet": {
                "sheetId": sheet_id
            }
        })],
    };
    client.post(&path, &request).await
}
