use crate::client::ApiClient;
use crate::error::Result;
use super::types::Task;

pub struct UpdateTaskParams {
    pub task_list_id: String,
    pub task_id: String,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub due: Option<String>,
    pub status: Option<TaskStatus>,
}

#[derive(Debug, Clone, Copy)]
pub enum TaskStatus {
    NeedsAction,
    Completed,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NeedsAction => "needsAction",
            Self::Completed => "completed",
        }
    }
}

pub async fn update_task(client: &ApiClient, params: UpdateTaskParams) -> Result<Task> {
    // Build update payload with only fields that should be updated
    let mut update_payload = serde_json::json!({});

    if let Some(title) = params.title {
        update_payload["title"] = serde_json::json!(title);
    }
    if let Some(notes) = params.notes {
        update_payload["notes"] = serde_json::json!(notes);
    }
    if let Some(due) = params.due {
        update_payload["due"] = serde_json::json!(due);
    }
    if let Some(status) = params.status {
        update_payload["status"] = serde_json::json!(status.as_str());
        if matches!(status, TaskStatus::Completed) {
            update_payload["completed"] = serde_json::json!(chrono::Utc::now().to_rfc3339());
        } else {
            // Clear completed timestamp when marking as needsAction
            update_payload["completed"] = serde_json::json!(null);
        }
    }

    let path = format!("/lists/{}/tasks/{}", params.task_list_id, params.task_id);
    client.patch(&path, &update_payload).await
}

pub async fn complete_task(
    client: &ApiClient,
    task_list_id: &str,
    task_id: &str,
) -> Result<Task> {
    update_task(client, UpdateTaskParams {
        task_list_id: task_list_id.to_string(),
        task_id: task_id.to_string(),
        title: None,
        notes: None,
        due: None,
        status: Some(TaskStatus::Completed),
    }).await
}

pub async fn delete_task(
    client: &ApiClient,
    task_list_id: &str,
    task_id: &str,
) -> Result<()> {
    let path = format!("/lists/{}/tasks/{}", task_list_id, task_id);
    client.delete(&path).await
}
