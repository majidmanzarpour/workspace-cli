use std::path::Path;
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::error::{WorkspaceError, ApiError};
use super::types::{File as DriveFile, FileMetadata};

const RESUMABLE_THRESHOLD: u64 = 5 * 1024 * 1024; // 5MB

pub struct UploadParams {
    pub file_path: String,
    pub name: Option<String>,
    pub parent_id: Option<String>,
    pub mime_type: Option<String>,
}

pub async fn upload_file(
    access_token: &str,
    params: UploadParams,
) -> Result<DriveFile, WorkspaceError> {
    let path = Path::new(&params.file_path);
    let file_name = params.name.unwrap_or_else(|| {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string()
    });

    let metadata = std::fs::metadata(path)
        .map_err(|e| WorkspaceError::Io(e))?;
    let file_size = metadata.len();

    let mime_type = params.mime_type.unwrap_or_else(|| {
        mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string()
    });

    if file_size > RESUMABLE_THRESHOLD {
        resumable_upload(access_token, path, &file_name, &mime_type, params.parent_id).await
    } else {
        simple_upload(access_token, path, &file_name, &mime_type, params.parent_id).await
    }
}

async fn simple_upload(
    access_token: &str,
    path: &Path,
    name: &str,
    mime_type: &str,
    parent_id: Option<String>,
) -> Result<DriveFile, WorkspaceError> {
    let client = Client::new();

    let mut file = File::open(path).await?;
    let mut content = Vec::new();
    file.read_to_end(&mut content).await?;

    let metadata = FileMetadata {
        name: name.to_string(),
        mime_type: Some(mime_type.to_string()),
        parents: parent_id.map(|p| vec![p]),
    };

    let metadata_json = serde_json::to_string(&metadata)?;

    // Multipart upload
    let boundary = "workspace_cli_boundary";
    let mut body = Vec::new();

    // Metadata part
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Type: application/json; charset=UTF-8\r\n\r\n");
    body.extend_from_slice(metadata_json.as_bytes());
    body.extend_from_slice(b"\r\n");

    // Content part
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(format!("Content-Type: {}\r\n\r\n", mime_type).as_bytes());
    body.extend_from_slice(&content);
    body.extend_from_slice(format!("\r\n--{}--", boundary).as_bytes());

    let response = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", format!("multipart/related; boundary={}", boundary))
        .body(body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let text = response.text().await.unwrap_or_default();
        return Err(WorkspaceError::Api(ApiError {
            code: status,
            message: text,
            domain: "drive".to_string(),
            retry_after: None,
        }));
    }

    response.json().await.map_err(WorkspaceError::from)
}

async fn resumable_upload(
    access_token: &str,
    path: &Path,
    name: &str,
    mime_type: &str,
    parent_id: Option<String>,
) -> Result<DriveFile, WorkspaceError> {
    let client = Client::new();

    let file_size = std::fs::metadata(path)
        .map_err(|e| WorkspaceError::Io(e))?
        .len();

    let metadata = FileMetadata {
        name: name.to_string(),
        mime_type: Some(mime_type.to_string()),
        parents: parent_id.map(|p| vec![p]),
    };

    // Step 1: Initiate resumable upload
    let init_response = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=resumable")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("X-Upload-Content-Type", mime_type)
        .header("X-Upload-Content-Length", file_size.to_string())
        .json(&metadata)
        .send()
        .await?;

    if !init_response.status().is_success() {
        let status = init_response.status().as_u16();
        let text = init_response.text().await.unwrap_or_default();
        return Err(WorkspaceError::Api(ApiError {
            code: status,
            message: text,
            domain: "drive".to_string(),
            retry_after: None,
        }));
    }

    let upload_uri = init_response
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| WorkspaceError::Config("No upload URI in response".to_string()))?
        .to_string();

    // Step 2: Upload the file content in chunks
    const CHUNK_SIZE: usize = 256 * 1024; // 256KB chunks
    let mut file = File::open(path).await?;
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut uploaded = 0u64;

    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }

        let chunk_end = uploaded + bytes_read as u64 - 1;
        let content_range = format!("bytes {}-{}/{}", uploaded, chunk_end, file_size);

        let response = client
            .put(&upload_uri)
            .header("Content-Type", mime_type)
            .header("Content-Length", bytes_read.to_string())
            .header("Content-Range", content_range)
            .body(buffer[..bytes_read].to_vec())
            .send()
            .await?;

        uploaded += bytes_read as u64;

        // 308 Resume Incomplete means continue uploading
        if response.status().as_u16() == 308 {
            continue;
        }

        // Check for success (200 or 201)
        if response.status().is_success() {
            return response.json().await.map_err(WorkspaceError::from);
        }

        // Handle error
        let status = response.status().as_u16();
        let text = response.text().await.unwrap_or_default();
        return Err(WorkspaceError::Api(ApiError {
            code: status,
            message: text,
            domain: "drive".to_string(),
            retry_after: None,
        }));
    }

    // If we get here, the upload completed but didn't get a final response
    Err(WorkspaceError::Config("Upload completed but no response received".to_string()))
}
