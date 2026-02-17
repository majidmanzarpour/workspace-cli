use crate::client::ApiClient;
use crate::error::Result;
use super::types::{File, FileList};

pub struct ListParams {
    pub query: Option<String>,
    pub max_results: u32,
    pub page_token: Option<String>,
    pub order_by: Option<String>,
    pub fields: Option<String>,
    /// Corpora to search: user, domain, drive, allDrives
    pub corpora: Option<String>,
    /// Include permissions in response fields
    pub include_permissions: bool,
}

impl Default for ListParams {
    fn default() -> Self {
        Self {
            query: None,
            max_results: 20,
            page_token: None,
            order_by: None,
            fields: Some("files(id,name,mimeType,webViewLink,modifiedTime)".to_string()),
            corpora: None,
            include_permissions: false,
        }
    }
}

pub async fn list_files(client: &ApiClient, params: ListParams) -> Result<FileList> {
    let mut query_params: Vec<(&str, String)> = vec![
        ("pageSize", params.max_results.to_string()),
    ];

    if let Some(ref q) = params.query {
        query_params.push(("q", q.clone()));
    }
    if let Some(ref token) = params.page_token {
        query_params.push(("pageToken", token.clone()));
    }
    if let Some(ref order) = params.order_by {
        query_params.push(("orderBy", order.clone()));
    }
    if let Some(ref corpora) = params.corpora {
        query_params.push(("corpora", corpora.clone()));
        // allDrives requires includeItemsFromAllDrives and supportsAllDrives
        if corpora == "allDrives" || corpora == "drive" {
            query_params.push(("includeItemsFromAllDrives", "true".to_string()));
            query_params.push(("supportsAllDrives", "true".to_string()));
        }
    }
    if params.include_permissions {
        let fields_with_perms = params.fields.clone()
            .unwrap_or_else(|| "files(id,name,mimeType,webViewLink,modifiedTime,permissions(emailAddress,role,type))".to_string());
        query_params.push(("fields", fields_with_perms));
    } else if let Some(ref fields) = params.fields {
        query_params.push(("fields", fields.clone()));
    }

    client.get_with_query("/files", &query_params).await
}

pub async fn get_file(client: &ApiClient, file_id: &str, fields: Option<&str>) -> Result<File> {
    let default_fields = "id,name,mimeType,webViewLink,webContentLink,size,createdTime,modifiedTime,parents";
    let query = [("fields", fields.unwrap_or(default_fields))];
    client.get_with_query(&format!("/files/{}", file_id), &query).await
}
