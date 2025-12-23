use crate::client::ApiClient;
use crate::error::Result;
use super::types::{TaskLists, Tasks, Task};

pub struct ListTasksParams {
    pub task_list_id: String,
    pub max_results: u32,
    pub show_completed: bool,
    pub show_hidden: bool,
    pub page_token: Option<String>,
}

impl Default for ListTasksParams {
    fn default() -> Self {
        Self {
            task_list_id: "@default".to_string(),
            max_results: 20,
            show_completed: false,
            show_hidden: false,
            page_token: None,
        }
    }
}

pub async fn list_task_lists(client: &ApiClient) -> Result<TaskLists> {
    client.get("/users/@me/lists").await
}

pub async fn list_tasks(client: &ApiClient, params: ListTasksParams) -> Result<Tasks> {
    let mut query_params: Vec<(&str, String)> = vec![
        ("maxResults", params.max_results.to_string()),
        ("showCompleted", params.show_completed.to_string()),
        ("showHidden", params.show_hidden.to_string()),
    ];

    if let Some(ref token) = params.page_token {
        query_params.push(("pageToken", token.clone()));
    }

    let path = format!("/lists/{}/tasks", params.task_list_id);
    client.get_with_query(&path, &query_params).await
}

pub async fn get_task(
    client: &ApiClient,
    task_list_id: &str,
    task_id: &str,
) -> Result<Task> {
    let path = format!("/lists/{}/tasks/{}", task_list_id, task_id);
    client.get(&path).await
}

/// Flatten nested tasks into a linear list with depth indicators
pub fn flatten_tasks(tasks: &[Task]) -> Vec<(usize, &Task)> {
    use std::collections::HashMap;

    let mut result = Vec::new();

    // Skip deleted and hidden tasks unless they have children
    let visible_tasks: Vec<&Task> = tasks.iter()
        .filter(|t| !t.deleted.unwrap_or(false))
        .collect();

    // Build parent-child relationships using task IDs
    let mut children: HashMap<Option<&str>, Vec<&Task>> = HashMap::new();
    for task in &visible_tasks {
        let parent_id = task.parent.as_deref();
        children.entry(parent_id).or_default().push(task);
    }

    // Recursively add tasks in hierarchical order
    fn add_tasks_recursive<'a>(
        parent_id: Option<&str>,
        depth: usize,
        children: &HashMap<Option<&str>, Vec<&'a Task>>,
        result: &mut Vec<(usize, &'a Task)>,
    ) {
        if let Some(child_tasks) = children.get(&parent_id) {
            // Sort by position to maintain order
            let mut sorted_children = child_tasks.clone();
            sorted_children.sort_by(|a, b| {
                match (&a.position, &b.position) {
                    (Some(pos_a), Some(pos_b)) => pos_a.cmp(pos_b),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                }
            });

            for task in sorted_children {
                result.push((depth, task));
                // Recursively add children
                if let Some(ref task_id) = task.id {
                    add_tasks_recursive(Some(task_id), depth + 1, children, result);
                }
            }
        }
    }

    // Start with root tasks (those with no parent)
    add_tasks_recursive(None, 0, &children, &mut result);

    result
}
