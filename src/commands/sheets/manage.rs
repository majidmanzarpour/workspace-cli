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

/// Add a filter view to a spreadsheet
///
/// criteria_json: JSON array of filter criteria, e.g.:
/// [{"column": 5, "condition": {"type": "TEXT_EQ", "values": ["T1"]}}]
///
/// Condition types: TEXT_EQ, TEXT_NOT_EQ, TEXT_CONTAINS, TEXT_NOT_CONTAINS,
///                  NUMBER_GREATER, NUMBER_LESS, NUMBER_EQ, NOT_BLANK, BLANK
pub async fn add_filter_view(
    client: &ApiClient,
    spreadsheet_id: &str,
    title: &str,
    sheet_id: i64,
    criteria_json: &str,
) -> Result<BatchUpdateResponse> {
    let criteria: Vec<serde_json::Value> = serde_json::from_str(criteria_json)?;

    // Build filterSpecs from criteria
    let filter_specs: Vec<serde_json::Value> = criteria.iter().map(|c| {
        let col = c["column"].as_i64().unwrap_or(0);
        let condition = &c["condition"];
        let cond_type = condition["type"].as_str().unwrap_or("TEXT_EQ");
        let values: Vec<serde_json::Value> = condition["values"]
            .as_array()
            .map(|arr| arr.iter().map(|v| {
                serde_json::json!({"userEnteredValue": v.as_str().unwrap_or("")})
            }).collect())
            .unwrap_or_default();

        serde_json::json!({
            "columnIndex": col,
            "filterCriteria": {
                "condition": {
                    "type": cond_type,
                    "values": values
                }
            }
        })
    }).collect();

    let path = format!("/spreadsheets/{}:batchUpdate", spreadsheet_id);
    let request = BatchUpdateRequest {
        requests: vec![serde_json::json!({
            "addFilterView": {
                "filter": {
                    "title": title,
                    "range": {
                        "sheetId": sheet_id,
                    },
                    "filterSpecs": filter_specs
                }
            }
        })],
    };
    client.post(&path, &request).await
}

/// Insert rows or columns at a given position
pub async fn insert_dimension(client: &ApiClient, spreadsheet_id: &str, sheet_id: i64, dimension: &str, start_index: i64, count: i64) -> Result<BatchUpdateResponse> {
    let dim = if dimension.to_uppercase().starts_with('C') { "COLUMNS" } else { "ROWS" };
    let path = format!("/spreadsheets/{}:batchUpdate", spreadsheet_id);
    let request = BatchUpdateRequest {
        requests: vec![serde_json::json!({
            "insertDimension": {
                "range": {
                    "sheetId": sheet_id,
                    "dimension": dim,
                    "startIndex": start_index,
                    "endIndex": start_index + count
                },
                "inheritFromBefore": start_index > 0
            }
        })],
    };
    client.post(&path, &request).await
}

/// Delete rows or columns at a given range
pub async fn delete_dimension(client: &ApiClient, spreadsheet_id: &str, sheet_id: i64, dimension: &str, start_index: i64, count: i64) -> Result<BatchUpdateResponse> {
    let dim = if dimension.to_uppercase().starts_with('C') { "COLUMNS" } else { "ROWS" };
    let path = format!("/spreadsheets/{}:batchUpdate", spreadsheet_id);
    let request = BatchUpdateRequest {
        requests: vec![serde_json::json!({
            "deleteDimension": {
                "range": {
                    "sheetId": sheet_id,
                    "dimension": dim,
                    "startIndex": start_index,
                    "endIndex": start_index + count
                }
            }
        })],
    };
    client.post(&path, &request).await
}

/// Delete a filter view by its ID
pub async fn delete_filter_view(client: &ApiClient, spreadsheet_id: &str, filter_view_id: i64) -> Result<BatchUpdateResponse> {
    let path = format!("/spreadsheets/{}:batchUpdate", spreadsheet_id);
    let request = BatchUpdateRequest {
        requests: vec![serde_json::json!({
            "deleteFilterView": {
                "filterId": filter_view_id
            }
        })],
    };
    client.post(&path, &request).await
}

/// Update a filter view's title and/or criteria
pub async fn update_filter_view(
    client: &ApiClient,
    spreadsheet_id: &str,
    filter_view_id: i64,
    title: Option<&str>,
    criteria_json: Option<&str>,
) -> Result<BatchUpdateResponse> {
    let mut filter = serde_json::json!({
        "filterViewId": filter_view_id,
    });

    let mut fields = Vec::new();

    if let Some(t) = title {
        filter["title"] = serde_json::json!(t);
        fields.push("title");
    }

    if let Some(cj) = criteria_json {
        let criteria: Vec<serde_json::Value> = serde_json::from_str(cj)?;
        let filter_specs: Vec<serde_json::Value> = criteria.iter().map(|c| {
            let col = c["column"].as_i64().unwrap_or(0);
            let condition = &c["condition"];
            let cond_type = condition["type"].as_str().unwrap_or("TEXT_EQ");
            let values: Vec<serde_json::Value> = condition["values"]
                .as_array()
                .map(|arr| arr.iter().map(|v| {
                    serde_json::json!({"userEnteredValue": v.as_str().unwrap_or("")})
                }).collect())
                .unwrap_or_default();
            serde_json::json!({
                "columnIndex": col,
                "filterCriteria": {
                    "condition": {
                        "type": cond_type,
                        "values": values
                    }
                }
            })
        }).collect();
        filter["filterSpecs"] = serde_json::json!(filter_specs);
        fields.push("filterSpecs");
    }

    let path = format!("/spreadsheets/{}:batchUpdate", spreadsheet_id);
    let request = BatchUpdateRequest {
        requests: vec![serde_json::json!({
            "updateFilterView": {
                "filter": filter,
                "fields": {
                    "paths": fields
                }
            }
        })],
    };
    client.post(&path, &request).await
}

/// Rename a sheet tab
pub async fn rename_sheet(client: &ApiClient, spreadsheet_id: &str, sheet_id: i64, title: &str) -> Result<BatchUpdateResponse> {
    let path = format!("/spreadsheets/{}:batchUpdate", spreadsheet_id);
    let request = BatchUpdateRequest {
        requests: vec![serde_json::json!({
            "updateSheetProperties": {
                "properties": {
                    "sheetId": sheet_id,
                    "title": title
                },
                "fields": "title"
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
