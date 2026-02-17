use crate::client::ApiClient;
use crate::error::Result;
use super::types::{File, FileList, SharedDriveList};

pub struct ListParams {
    pub query: Option<String>,
    pub max_results: u32,
    pub page_token: Option<String>,
    pub order_by: Option<String>,
    pub fields: Option<String>,
    pub corpora: Option<String>,
    pub include_permissions: bool,
}

impl Default for ListParams {
    fn default() -> Self {
        Self {
            query: None,
            max_results: 20,
            page_token: None,
            order_by: None,
            fields: None,
            corpora: None,
            include_permissions: false,
        }
    }
}

/// Default fields for Drive file listing
const DEFAULT_FILE_FIELDS: &str = "id,name,mimeType,owners(emailAddress),createdTime,modifiedTime,viewedByMeTime,size,parents,shared,shortcutDetails(targetId,targetMimeType)";
const PERMISSION_FIELDS: &str = ",permissions(id,type,role,emailAddress,domain),driveId";

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

    // Build fields parameter with files() wrapper
    let file_fields = params.fields.as_deref().unwrap_or(DEFAULT_FILE_FIELDS);
    let fields_str = if params.include_permissions {
        format!("nextPageToken,incompleteSearch,files({}{})", file_fields, PERMISSION_FIELDS)
    } else {
        format!("nextPageToken,incompleteSearch,files({})", file_fields)
    };
    query_params.push(("fields", fields_str));

    if let Some(ref corpora) = params.corpora {
        query_params.push(("corpora", corpora.clone()));
        // domain/drive/allDrives corpora require these params
        if corpora != "user" {
            query_params.push(("supportsAllDrives", "true".to_string()));
            query_params.push(("includeItemsFromAllDrives", "true".to_string()));
        }
    }

    client.get_with_query("/files", &query_params).await
}

pub async fn get_file(client: &ApiClient, file_id: &str, fields: Option<&str>) -> Result<File> {
    let default_fields = "id,name,mimeType,webViewLink,webContentLink,size,createdTime,modifiedTime,parents";
    let query = [("fields", fields.unwrap_or(default_fields))];
    client.get_with_query(&format!("/files/{}", file_id), &query).await
}

/// List all Shared Drives in the domain (uses useDomainAdminAccess for org-wide visibility)
pub async fn list_drives(client: &ApiClient, limit: u32, page_token: Option<&str>) -> Result<SharedDriveList> {
    let mut query_params: Vec<(&str, String)> = vec![
        ("pageSize", limit.to_string()),
        ("useDomainAdminAccess", "true".to_string()),
    ];
    if let Some(token) = page_token {
        query_params.push(("pageToken", token.to_string()));
    }
    client.get_with_query("/drives", &query_params).await
}
