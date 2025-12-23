use crate::client::ApiClient;
use crate::error::Result;
use super::types::Task;

pub struct CreateTaskParams {
    pub task_list_id: String,
    pub title: String,
    pub notes: Option<String>,
    pub due: Option<String>,
    pub parent: Option<String>,
}

impl CreateTaskParams {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            task_list_id: "@default".to_string(),
            title: title.into(),
            notes: None,
            due: None,
            parent: None,
        }
    }
}

pub async fn create_task(client: &ApiClient, params: CreateTaskParams) -> Result<Task> {
    let mut task = Task::new(params.title);
    task.notes = params.notes;
    task.due = params.due;
    task.parent = params.parent;

    let path = format!("/lists/{}/tasks", params.task_list_id);
    client.post(&path, &task).await
}

pub async fn create_task_list(client: &ApiClient, title: &str) -> Result<super::types::TaskList> {
    let body = serde_json::json!({
        "title": title
    });
    client.post("/users/@me/lists", &body).await
}
