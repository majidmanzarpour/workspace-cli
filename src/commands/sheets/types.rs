use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spreadsheet {
    pub spreadsheet_id: String,
    pub properties: SpreadsheetProperties,
    #[serde(default)]
    pub sheets: Vec<Sheet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetProperties {
    pub title: String,
    pub locale: Option<String>,
    #[serde(rename = "timeZone")]
    pub time_zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sheet {
    pub properties: SheetProperties,
    #[serde(default)]
    pub filter_views: Vec<FilterView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterView {
    pub filter_view_id: i64,
    pub title: String,
    #[serde(default)]
    pub range: Option<GridRange>,
    #[serde(default)]
    pub filter_specs: Vec<serde_json::Value>,
    #[serde(default)]
    pub criteria: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridRange {
    #[serde(default)]
    pub sheet_id: i64,
    pub start_row_index: Option<i64>,
    pub end_row_index: Option<i64>,
    pub start_column_index: Option<i64>,
    pub end_column_index: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetProperties {
    pub sheet_id: i64,
    pub title: String,
    pub index: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueRange {
    pub range: String,
    pub major_dimension: Option<String>,
    #[serde(default)]
    pub values: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateValuesResponse {
    pub spreadsheet_id: String,
    pub updated_range: String,
    pub updated_rows: Option<i64>,
    pub updated_columns: Option<i64>,
    pub updated_cells: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendValuesResponse {
    pub spreadsheet_id: String,
    pub table_range: Option<String>,
    pub updates: Option<UpdateValuesResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchGetValuesResponse {
    pub spreadsheet_id: String,
    #[serde(default)]
    pub value_ranges: Vec<ValueRange>,
}
