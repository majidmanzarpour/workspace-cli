use serde::{Deserialize, Serialize};
use crate::output::pagination::Timestamped;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskList {
    pub kind: Option<String>,
    pub id: String,
    pub etag: Option<String>,
    pub title: String,
    pub updated: Option<String>,
    #[serde(rename = "selfLink")]
    pub self_link: Option<String>,
}

impl Timestamped for TaskList {
    fn timestamp(&self) -> Option<&str> {
        self.updated.as_deref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskLists {
    #[serde(default)]
    pub items: Vec<TaskList>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub kind: Option<String>,
    pub id: Option<String>,
    pub etag: Option<String>,
    pub title: String,
    pub notes: Option<String>,
    pub status: Option<String>, // "needsAction" or "completed"
    pub due: Option<String>,    // RFC3339
    pub completed: Option<String>,
    pub parent: Option<String>,
    pub position: Option<String>,
    #[serde(default)]
    pub links: Vec<TaskLink>,
    pub updated: Option<String>,
    #[serde(rename = "selfLink")]
    pub self_link: Option<String>,
    pub hidden: Option<bool>,
    pub deleted: Option<bool>,
}

impl Timestamped for Task {
    fn timestamp(&self) -> Option<&str> {
        self.updated.as_deref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLink {
    pub r#type: String,
    pub description: Option<String>,
    pub link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tasks {
    #[serde(default)]
    pub items: Vec<Task>,
    pub next_page_token: Option<String>,
}

/// Minimal task format optimized for AI agents (reduced token usage)
/// Excludes: kind, etag, selfLink, links, parent, position, hidden, deleted, updated
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinimalTask {
    pub id: Option<String>,
    pub title: String,
    pub status: Option<String>,
    pub due: Option<String>,
    pub notes: Option<String>,
    pub completed: Option<String>,
}

impl Timestamped for MinimalTask {
    fn timestamp(&self) -> Option<&str> {
        self.completed.as_deref()
    }
}

impl MinimalTask {
    pub fn from_task(task: &Task) -> Self {
        Self {
            id: task.id.clone(),
            title: task.title.clone(),
            status: task.status.clone(),
            due: task.due.clone(),
            notes: task.notes.clone(),
            completed: task.completed.clone(),
        }
    }
}

/// Minimal task list response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinimalTasks {
    #[serde(default)]
    pub items: Vec<MinimalTask>,
    pub next_page_token: Option<String>,
}

impl MinimalTasks {
    pub fn from_tasks(tasks: &Tasks) -> Self {
        Self {
            items: tasks.items.iter().map(MinimalTask::from_task).collect(),
            next_page_token: tasks.next_page_token.clone(),
        }
    }
}

impl Task {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            kind: None,
            id: None,
            etag: None,
            title: title.into(),
            notes: None,
            status: Some("needsAction".to_string()),
            due: None,
            completed: None,
            parent: None,
            position: None,
            links: Vec::new(),
            updated: None,
            self_link: None,
            hidden: None,
            deleted: None,
        }
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    pub fn with_due(mut self, due: impl Into<String>) -> Self {
        self.due = Some(due.into());
        self
    }
}
