pub mod types;
pub mod list;
pub mod create;
pub mod update;

// Re-export commonly used types and functions
pub use types::{Task, TaskList, TaskLists, Tasks, TaskLink};
pub use list::{
    list_task_lists,
    list_tasks,
    get_task,
    flatten_tasks,
    ListTasksParams,
};
pub use create::{
    create_task,
    create_task_list,
    CreateTaskParams,
};
pub use update::{
    update_task,
    complete_task,
    delete_task,
    UpdateTaskParams,
    TaskStatus,
};
