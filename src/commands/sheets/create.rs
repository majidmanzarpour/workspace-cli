use crate::client::ApiClient;
use crate::error::Result;
use super::types::Spreadsheet;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateSpreadsheetRequest {
    properties: SpreadsheetPropertiesCreate,
    #[serde(skip_serializing_if = "Option::is_none")]
    sheets: Option<Vec<SheetCreate>>,
}

#[derive(Debug, Serialize)]
struct SpreadsheetPropertiesCreate {
    title: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SheetCreate {
    properties: SheetPropertiesCreate,
}

#[derive(Debug, Serialize)]
struct SheetPropertiesCreate {
    title: String,
}

/// Create a new Google Spreadsheet
pub async fn create_spreadsheet(client: &ApiClient, title: &str) -> Result<Spreadsheet> {
    let request = CreateSpreadsheetRequest {
        properties: SpreadsheetPropertiesCreate {
            title: title.to_string(),
        },
        sheets: None,
    };

    client.post("/spreadsheets", &request).await
}

/// Create a new spreadsheet with initial sheet names
pub async fn create_spreadsheet_with_sheets(
    client: &ApiClient,
    title: &str,
    sheet_names: &[String],
) -> Result<Spreadsheet> {
    let sheets = sheet_names
        .iter()
        .map(|name| SheetCreate {
            properties: SheetPropertiesCreate {
                title: name.clone(),
            },
        })
        .collect();

    let request = CreateSpreadsheetRequest {
        properties: SpreadsheetPropertiesCreate {
            title: title.to_string(),
        },
        sheets: Some(sheets),
    };

    client.post("/spreadsheets", &request).await
}
