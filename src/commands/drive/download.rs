use std::path::Path;
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::error::{WorkspaceError, ApiError};

pub async fn download_file(
    access_token: &str,
    file_id: &str,
    output_path: &Path,
) -> Result<u64, WorkspaceError> {
    let client = Client::new();

    let url = format!(
        "https://www.googleapis.com/drive/v3/files/{}?alt=media",
        file_id
    );

    let mut response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", access_token))
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

    let mut file = File::create(output_path).await?;
    let mut total_bytes = 0u64;

    // Stream the response to avoid loading entire file into memory
    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk).await?;
        total_bytes += chunk.len() as u64;
    }

    file.flush().await?;

    Ok(total_bytes)
}

/// Export Google Docs/Sheets/Slides to a specific format
pub async fn export_file(
    access_token: &str,
    file_id: &str,
    mime_type: &str,
    output_path: &Path,
) -> Result<u64, WorkspaceError> {
    let client = Client::new();

    let url = format!(
        "https://www.googleapis.com/drive/v3/files/{}/export?mimeType={}",
        file_id,
        urlencoding::encode(mime_type)
    );

    let mut response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", access_token))
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

    let mut file = File::create(output_path).await?;
    let mut total_bytes = 0u64;

    // Stream the response to avoid loading entire file into memory
    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk).await?;
        total_bytes += chunk.len() as u64;
    }

    file.flush().await?;

    Ok(total_bytes)
}
