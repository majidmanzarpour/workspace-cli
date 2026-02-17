use crate::client::ApiClient;
use crate::error::Result;
use super::list::{list_files, ListParams};
use super::types::File;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeNode {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub depth: u32,
    pub parent_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    pub shared: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<TreePermission>,
    #[serde(rename = "driveId", skip_serializing_if = "Option::is_none")]
    pub shared_drive_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewed_by_me_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut_details: Option<TreeShortcutDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeShortcutDetails {
    pub target_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreePermission {
    #[serde(rename = "type")]
    pub perm_type: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TreeResult {
    pub root_id: String,
    pub total_items: usize,
    pub total_folders: usize,
    pub total_files: usize,
    pub max_depth: u32,
    pub nodes: Vec<TreeNode>,
}

impl TreeNode {
    fn from_file(file: &File, depth: u32, parent_id: &str, include_permissions: bool) -> Self {
        let (permissions, shared_drive_id) = if include_permissions {
            (
                file.permissions.iter().map(|p| TreePermission {
                    perm_type: p.permission_type.clone().unwrap_or_default(),
                    role: p.role.clone().unwrap_or_default(),
                    email: p.email_address.clone(),
                    domain: p.domain.clone(),
                }).collect(),
                file.drive_id.clone(),
            )
        } else {
            (vec![], None)
        };

        Self {
            id: file.id.clone(),
            name: file.name.clone(),
            mime_type: file.mime_type.clone(),
            depth,
            parent_id: parent_id.to_string(),
            owner: file.owners.first().and_then(|o| o.email_address.clone()),
            created_time: file.created_time.clone(),
            modified_time: file.modified_time.clone(),
            size: file.size.clone(),
            shared: file.shared.unwrap_or(false),
            permissions,
            shared_drive_id,
            viewed_by_me_time: file.viewed_by_me_time.clone(),
            shortcut_details: file.shortcut_details.as_ref().and_then(|sd| {
                sd.target_id.as_ref().map(|tid| TreeShortcutDetails {
                    target_id: tid.clone(),
                    target_mime_type: sd.target_mime_type.clone(),
                })
            }),
        }
    }

    pub fn is_folder(&self) -> bool {
        self.mime_type == "application/vnd.google-apps.folder"
    }
}

/// List ALL children of a folder (handles pagination)
async fn list_all_children(client: &ApiClient, parent_id: &str, include_permissions: bool) -> Result<Vec<File>> {
    let mut all_files = Vec::new();
    let mut page_token: Option<String> = None;

    loop {
        let params = ListParams {
            query: Some(format!("'{}' in parents and trashed = false", parent_id)),
            max_results: 100,
            page_token: page_token.clone(),
            include_permissions,
            ..Default::default()
        };

        let result = list_files(client, params).await?;
        all_files.extend(result.files);

        match result.next_page_token {
            Some(token) if !token.is_empty() => page_token = Some(token),
            _ => break,
        }
    }

    Ok(all_files)
}

/// Recursively crawl a folder tree with controlled concurrency
pub async fn crawl_tree(
    client: &ApiClient,
    root_id: &str,
    max_depth: Option<u32>,
    concurrency: usize,
    include_permissions: bool,
) -> Result<TreeResult> {
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut all_nodes: Vec<TreeNode> = Vec::new();

    // BFS with concurrent level processing
    let mut current_level: Vec<(String, u32)> = vec![(root_id.to_string(), 0)]; // (folder_id, depth)

    while !current_level.is_empty() {
        // Check depth limit
        let depth = current_level[0].1;
        if let Some(max) = max_depth {
            if depth > max {
                break;
            }
        }

        // Process all folders at current level concurrently
        let mut handles = Vec::new();
        for (folder_id, depth) in current_level.drain(..) {
            let client = client.clone();
            let sem = semaphore.clone();
            let fid = folder_id.clone();

            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                let children = list_all_children(&client, &fid, include_permissions).await;
                (fid, depth, children)
            }));
        }

        let mut next_level: Vec<(String, u32)> = Vec::new();

        for handle in handles {
            match handle.await {
                Ok((parent_id, depth, Ok(children))) => {
                    for file in &children {
                        let node = TreeNode::from_file(file, depth, &parent_id, include_permissions);
                        if node.is_folder() {
                            next_level.push((file.id.clone(), depth + 1));
                        }
                        all_nodes.push(node);
                    }
                    eprint!("\r  {} items found, depth {}...", all_nodes.len(), depth);
                }
                Ok((parent_id, depth, Err(e))) => {
                    eprintln!("\nWarning: failed to list children of {} at depth {}: {}", parent_id, depth, e);
                }
                Err(e) => {
                    eprintln!("\nWarning: task join error: {}", e);
                }
            }
        }

        current_level = next_level;
    }
    eprintln!(); // newline after progress

    let total_folders = all_nodes.iter().filter(|n| n.is_folder()).count();
    let total_files = all_nodes.len() - total_folders;
    let max_depth_found = all_nodes.iter().map(|n| n.depth).max().unwrap_or(0);

    Ok(TreeResult {
        root_id: root_id.to_string(),
        total_items: all_nodes.len(),
        total_folders,
        total_files,
        max_depth: max_depth_found,
        nodes: all_nodes,
    })
}
