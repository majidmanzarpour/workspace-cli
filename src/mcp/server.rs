use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router, ServiceExt,
    transport::stdio,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::auth::TokenManager;
use crate::client::ApiClient;

fn ok_json<T: serde::Serialize>(val: &T) -> String {
    serde_json::to_string_pretty(val)
        .unwrap_or_else(|e| format!(r#"{{"error":"serialization failed: {}"}}"#, e))
}

fn err_json(msg: impl std::fmt::Display) -> String {
    serde_json::json!({"error": msg.to_string()}).to_string()
}

// ============================================================
// Argument Structs — Gmail
// ============================================================

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GmailListArgs {
    #[schemars(description = "Gmail search query (e.g. 'is:unread', 'from:user@example.com subject:hello')")]
    query: Option<String>,
    #[schemars(description = "Maximum results (default: 20, max: 500)")]
    limit: Option<u32>,
    #[schemars(description = "Label to filter by (e.g. 'INBOX', 'SENT', 'STARRED')")]
    label: Option<String>,
    #[schemars(description = "Page token for continuing a previous list (from next_page_token field)")]
    page_token: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GmailGetArgs {
    #[schemars(description = "Gmail message ID")]
    id: String,
    #[schemars(description = "Return full MIME structure (default: minimal with headers + plain text body)")]
    full: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GmailSendArgs {
    #[schemars(description = "Recipient email address")]
    to: String,
    #[schemars(description = "Email subject line")]
    subject: String,
    #[schemars(description = "Email body text")]
    body: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GmailReplyArgs {
    #[schemars(description = "Message ID to reply to")]
    id: String,
    #[schemars(description = "Reply body text")]
    body: String,
    #[schemars(description = "Reply-all (include all original recipients)")]
    all: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GmailModifyArgs {
    #[schemars(description = "Message ID to modify")]
    id: String,
    #[schemars(description = "Mark as read")]
    mark_read: Option<bool>,
    #[schemars(description = "Mark as unread")]
    mark_unread: Option<bool>,
    #[schemars(description = "Star the message")]
    star: Option<bool>,
    #[schemars(description = "Archive the message (remove from INBOX)")]
    archive: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GmailIdArgs {
    #[schemars(description = "Gmail message ID")]
    id: String,
}

// ============================================================
// Argument Structs — Drive
// ============================================================

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DriveListArgs {
    #[schemars(description = "Drive query (e.g. 'name contains report', 'mimeType=application/vnd.google-apps.folder')")]
    query: Option<String>,
    #[schemars(description = "Maximum results (default: 20)")]
    limit: Option<u32>,
    #[schemars(description = "Parent folder ID to list files in")]
    parent: Option<String>,
    #[schemars(description = "Page token for continuing a previous list")]
    page_token: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DriveGetArgs {
    #[schemars(description = "Drive file/folder ID")]
    id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DriveMkdirArgs {
    #[schemars(description = "Folder name to create")]
    name: String,
    #[schemars(description = "Parent folder ID (optional, defaults to My Drive root)")]
    parent: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DriveMoveArgs {
    #[schemars(description = "File ID to move")]
    id: String,
    #[schemars(description = "Destination folder ID")]
    to: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DriveCopyArgs {
    #[schemars(description = "File ID to copy")]
    id: String,
    #[schemars(description = "New name for the copy (optional)")]
    name: Option<String>,
    #[schemars(description = "Destination folder ID (optional)")]
    parent: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DriveRenameArgs {
    #[schemars(description = "File ID to rename")]
    id: String,
    #[schemars(description = "New name")]
    name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DriveShareArgs {
    #[schemars(description = "File ID to share")]
    id: String,
    #[schemars(description = "Email address to share with")]
    email: Option<String>,
    #[schemars(description = "Make public (share with anyone)")]
    anyone: Option<bool>,
    #[schemars(description = "Permission role: reader, writer, commenter (default: reader)")]
    role: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DriveIdArgs {
    #[schemars(description = "Drive file/folder ID")]
    id: String,
}

// ============================================================
// Argument Structs — Calendar
// ============================================================

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CalendarListArgs {
    #[schemars(description = "Start time in RFC3339 format (e.g. 2026-03-01T00:00:00Z)")]
    time_min: Option<String>,
    #[schemars(description = "End time in RFC3339 format")]
    time_max: Option<String>,
    #[schemars(description = "Maximum results (default: 20)")]
    limit: Option<u32>,
    #[schemars(description = "Calendar ID (default: primary)")]
    calendar: Option<String>,
    #[schemars(description = "Page token for continuing a previous list")]
    page_token: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CalendarCreateArgs {
    #[schemars(description = "Event title/summary")]
    summary: String,
    #[schemars(description = "Start time in RFC3339 format (e.g. 2026-03-10T14:00:00Z)")]
    start: String,
    #[schemars(description = "End time in RFC3339 format")]
    end: String,
    #[schemars(description = "Event description (optional)")]
    description: Option<String>,
    #[schemars(description = "Calendar ID (default: primary)")]
    calendar: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CalendarUpdateArgs {
    #[schemars(description = "Event ID to update")]
    id: String,
    #[schemars(description = "New event title/summary")]
    summary: Option<String>,
    #[schemars(description = "New start time in RFC3339 format")]
    start: Option<String>,
    #[schemars(description = "New end time in RFC3339 format")]
    end: Option<String>,
    #[schemars(description = "Calendar ID (default: primary)")]
    calendar: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CalendarDeleteArgs {
    #[schemars(description = "Event ID to delete")]
    id: String,
    #[schemars(description = "Calendar ID (default: primary)")]
    calendar: Option<String>,
}

// ============================================================
// Argument Structs — Docs
// ============================================================

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DocsGetArgs {
    #[schemars(description = "Google Docs document ID")]
    id: String,
    #[schemars(description = "Return as markdown (default: plain text)")]
    markdown: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DocsCreateArgs {
    #[schemars(description = "Document title")]
    title: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DocsAppendArgs {
    #[schemars(description = "Google Docs document ID")]
    id: String,
    #[schemars(description = "Text to append to the document")]
    text: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DocsReplaceArgs {
    #[schemars(description = "Google Docs document ID")]
    id: String,
    #[schemars(description = "Text to find")]
    find: String,
    #[schemars(description = "Replacement text")]
    replace_with: String,
    #[schemars(description = "Case-sensitive match (default: false)")]
    match_case: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DocsBatchUpdateArgs {
    #[schemars(description = "Google Docs document ID")]
    id: String,
    #[schemars(description = "JSON string with batchUpdate requests array")]
    payload: String,
}

// ============================================================
// Argument Structs — Sheets
// ============================================================

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SheetsGetArgs {
    #[schemars(description = "Google Sheets spreadsheet ID")]
    id: String,
    #[schemars(description = "A1 notation range (e.g. Sheet1!A1:C10 or Sheet1!A:Z)")]
    range: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SheetsCreateArgs {
    #[schemars(description = "Spreadsheet title")]
    title: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SheetsUpdateArgs {
    #[schemars(description = "Google Sheets spreadsheet ID")]
    id: String,
    #[schemars(description = "A1 notation range to write to")]
    range: String,
    #[schemars(description = "JSON 2D array of values, e.g. [[\"Name\",\"Age\"],[\"Alice\",\"30\"]]")]
    values: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SheetsAppendArgs {
    #[schemars(description = "Google Sheets spreadsheet ID")]
    id: String,
    #[schemars(description = "A1 notation range to append after")]
    range: String,
    #[schemars(description = "JSON 2D array of values to append")]
    values: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SheetsClearArgs {
    #[schemars(description = "Google Sheets spreadsheet ID")]
    id: String,
    #[schemars(description = "A1 notation range to clear")]
    range: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SheetsIdArgs {
    #[schemars(description = "Google Sheets spreadsheet ID")]
    id: String,
}

// ============================================================
// Argument Structs — Slides
// ============================================================

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SlidesGetArgs {
    #[schemars(description = "Google Slides presentation ID")]
    id: String,
    #[schemars(description = "Return full JSON structure (default: text extraction only)")]
    full: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SlidesPageArgs {
    #[schemars(description = "Google Slides presentation ID")]
    id: String,
    #[schemars(description = "Slide index (0-indexed)")]
    page: u32,
    #[schemars(description = "Return full JSON structure (default: text extraction only)")]
    full: Option<bool>,
}

// ============================================================
// Argument Structs — Tasks
// ============================================================

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct TasksListArgs {
    #[schemars(description = "Task list ID (default: @default for primary list)")]
    list: Option<String>,
    #[schemars(description = "Maximum results (default: 20)")]
    limit: Option<u32>,
    #[schemars(description = "Include completed tasks (default: false)")]
    show_completed: Option<bool>,
    #[schemars(description = "Page token for continuing a previous list")]
    page_token: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct TasksCreateArgs {
    #[schemars(description = "Task title")]
    title: String,
    #[schemars(description = "Task list ID (default: @default)")]
    list: Option<String>,
    #[schemars(description = "Due date in RFC3339 format (e.g. 2026-03-15T17:00:00Z)")]
    due: Option<String>,
    #[schemars(description = "Task notes/description")]
    notes: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct TasksUpdateArgs {
    #[schemars(description = "Task ID to update")]
    id: String,
    #[schemars(description = "Task list ID (default: @default)")]
    list: Option<String>,
    #[schemars(description = "New title")]
    title: Option<String>,
    #[schemars(description = "Mark as completed")]
    complete: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct TasksDeleteArgs {
    #[schemars(description = "Task ID to delete")]
    id: String,
    #[schemars(description = "Task list ID (default: @default)")]
    list: Option<String>,
}

// ============================================================
// Argument Structs — Chat
// ============================================================

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ChatSpacesListArgs {
    #[schemars(description = "Space type filter: SPACE, DIRECT_MESSAGE, GROUP_CHAT (optional)")]
    space_type: Option<String>,
    #[schemars(description = "Maximum results (default: 100)")]
    limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ChatFindDmArgs {
    #[schemars(description = "Email of the other participant to find DM space with")]
    email: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ChatMessagesListArgs {
    #[schemars(description = "Space name (e.g. spaces/abc123)")]
    space: String,
    #[schemars(description = "Maximum results (default: 50)")]
    limit: Option<u32>,
    #[schemars(description = "Only messages from today")]
    today: Option<bool>,
    #[schemars(description = "Page token for continuing a previous list")]
    page_token: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ChatSendArgs {
    #[schemars(description = "Space name (e.g. spaces/abc123)")]
    space: String,
    #[schemars(description = "Message text to send")]
    text: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ChatUnreadArgs {
    #[schemars(description = "Time range: e.g. '7d', '24h', '1h' (default: 7d)")]
    since: Option<String>,
    #[schemars(description = "Space type: SPACE, DIRECT_MESSAGE, GROUP_CHAT (default: all)")]
    space_type: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ChatMarkReadArgs {
    #[schemars(description = "Space name to mark as read (e.g. spaces/abc123)")]
    space: Option<String>,
    #[schemars(description = "Mark all spaces as read")]
    all: Option<bool>,
}

// ============================================================
// Argument Structs — Contacts
// ============================================================

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ContactsListArgs {
    #[schemars(description = "Maximum results (default: 100)")]
    limit: Option<u32>,
    #[schemars(description = "Page token for continuing a previous list")]
    page_token: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ContactsSearchArgs {
    #[schemars(description = "Search query (name, email, phone number)")]
    query: String,
    #[schemars(description = "Maximum results (default: 50)")]
    limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ContactsGetArgs {
    #[schemars(description = "Contact resource name (e.g. people/c123456)")]
    name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ContactsCreateArgs {
    #[schemars(description = "Given (first) name")]
    given: String,
    #[schemars(description = "Family (last) name")]
    family: Option<String>,
    #[schemars(description = "Email address")]
    email: Option<String>,
    #[schemars(description = "Phone number")]
    phone: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ContactsDeleteArgs {
    #[schemars(description = "Contact resource name (e.g. people/c123456)")]
    name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ContactsDirectoryArgs {
    #[schemars(description = "Maximum results (default: 100)")]
    limit: Option<u32>,
    #[schemars(description = "Page token for continuing a previous list")]
    page_token: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ContactsDirectorySearchArgs {
    #[schemars(description = "Search query")]
    query: String,
    #[schemars(description = "Maximum results (default: 50)")]
    limit: Option<u32>,
}

// ============================================================
// Server struct
// ============================================================

#[derive(Clone)]
pub struct WorkspaceServer {
    tool_router: ToolRouter<Self>,
    token_manager: Arc<RwLock<TokenManager>>,
}

#[tool_router]
impl WorkspaceServer {
    pub fn new(token_manager: Arc<RwLock<TokenManager>>) -> Self {
        Self {
            tool_router: Self::tool_router(),
            token_manager,
        }
    }

    async fn get_token(&self) -> Result<String, String> {
        let tm = self.token_manager.read().await;
        tm.get_access_token().await.map_err(|e| e.to_string())
    }

    // ===== GMAIL TOOLS =====

    #[tool(description = "List Gmail messages with metadata. Returns JSON with messages array containing id, threadId, subject, from, date, snippet.")]
    async fn gmail_list(&self, Parameters(args): Parameters<GmailListArgs>) -> String {
        let client = ApiClient::gmail(self.token_manager.clone());
        let token = match self.get_token().await {
            Ok(t) => t,
            Err(e) => return err_json(e),
        };
        let params = crate::commands::gmail::list::ListParams {
            query: args.query,
            max_results: args.limit.unwrap_or(20),
            label_ids: args.label.map(|l| vec![l]),
            page_token: args.page_token,
        };
        match crate::commands::gmail::list::list_messages_with_metadata(&client, params, &token).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Get a specific Gmail message by ID. Returns minimal format (headers + plain text body) by default, or full MIME structure with full=true.")]
    async fn gmail_get(&self, Parameters(args): Parameters<GmailGetArgs>) -> String {
        let client = ApiClient::gmail(self.token_manager.clone());
        if args.full.unwrap_or(false) {
            match crate::commands::gmail::get::get_message(&client, &args.id, "full").await {
                Ok(r) => ok_json(&r),
                Err(e) => err_json(e),
            }
        } else {
            match crate::commands::gmail::get::get_message_minimal(&client, &args.id).await {
                Ok(r) => ok_json(&r),
                Err(e) => err_json(e),
            }
        }
    }

    #[tool(description = "Send an email via Gmail.")]
    async fn gmail_send(&self, Parameters(args): Parameters<GmailSendArgs>) -> String {
        let client = ApiClient::gmail(self.token_manager.clone());
        let params = crate::commands::gmail::send::ComposeParams {
            to: args.to,
            subject: args.subject,
            body: args.body,
            from: None,
            cc: None,
            in_reply_to: None,
            references: None,
            thread_id: None,
        };
        match crate::commands::gmail::send::send_message(&client, params).await {
            Ok(r) => serde_json::json!({"success": true, "id": r.id, "threadId": r.thread_id}).to_string(),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Reply to a Gmail message. Fetches the original to extract threading headers, then sends reply.")]
    async fn gmail_reply(&self, Parameters(args): Parameters<GmailReplyArgs>) -> String {
        let client = ApiClient::gmail(self.token_manager.clone());
        // Get the original message to extract reply metadata
        let original = match crate::commands::gmail::get::get_message(&client, &args.id, "full").await {
            Ok(m) => m,
            Err(e) => return err_json(format!("Failed to fetch original message: {}", e)),
        };
        let meta = match crate::commands::gmail::send::extract_reply_metadata(&original) {
            Some(m) => m,
            None => return err_json("Could not extract reply metadata from message"),
        };
        let cc = if args.all.unwrap_or(false) { meta.cc } else { None };
        let params = crate::commands::gmail::send::ComposeParams {
            to: meta.to,
            subject: meta.subject,
            body: args.body,
            from: None,
            cc,
            in_reply_to: Some(meta.in_reply_to),
            references: Some(meta.references),
            thread_id: Some(meta.thread_id),
        };
        match crate::commands::gmail::send::send_message(&client, params).await {
            Ok(r) => serde_json::json!({"success": true, "id": r.id, "threadId": r.thread_id}).to_string(),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "List all Gmail labels in the mailbox.")]
    async fn gmail_labels(&self) -> String {
        let client = ApiClient::gmail(self.token_manager.clone());
        match crate::commands::gmail::labels::list_labels(&client).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Modify a Gmail message: mark read/unread, star, or archive.")]
    async fn gmail_modify(&self, Parameters(args): Parameters<GmailModifyArgs>) -> String {
        let client = ApiClient::gmail(self.token_manager.clone());
        // Apply the first matching modification
        let result = if args.mark_read.unwrap_or(false) {
            crate::commands::gmail::modify::mark_read(&client, &args.id).await
        } else if args.mark_unread.unwrap_or(false) {
            crate::commands::gmail::modify::mark_unread(&client, &args.id).await
        } else if args.star.unwrap_or(false) {
            crate::commands::gmail::modify::star_message(&client, &args.id).await
        } else if args.archive.unwrap_or(false) {
            crate::commands::gmail::modify::archive_message(&client, &args.id).await
        } else {
            return err_json("No modification action specified. Use mark_read, mark_unread, star, or archive.");
        };
        match result {
            Ok(r) => serde_json::json!({"success": true, "id": r.id, "labels": r.label_ids}).to_string(),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Move a Gmail message to trash.")]
    async fn gmail_trash(&self, Parameters(args): Parameters<GmailIdArgs>) -> String {
        let client = ApiClient::gmail(self.token_manager.clone());
        match crate::commands::gmail::trash::trash_message(&client, &args.id).await {
            Ok(r) => serde_json::json!({"success": true, "id": r.id}).to_string(),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Permanently delete a Gmail message (bypasses trash). Use gmail_trash for recoverable deletion.")]
    async fn gmail_delete(&self, Parameters(args): Parameters<GmailIdArgs>) -> String {
        let client = ApiClient::gmail(self.token_manager.clone());
        match crate::commands::gmail::delete::delete_message(&client, &args.id).await {
            Ok(()) => r#"{"success":true}"#.to_string(),
            Err(e) => err_json(e),
        }
    }

    // ===== DRIVE TOOLS =====

    #[tool(description = "List Google Drive files and folders. Returns JSON with files array containing id, name, mimeType, size, modifiedTime, parents.")]
    async fn drive_list(&self, Parameters(args): Parameters<DriveListArgs>) -> String {
        let client = ApiClient::drive(self.token_manager.clone());
        let mut query = args.query.unwrap_or_default();
        if let Some(ref parent) = args.parent {
            if query.is_empty() {
                query = format!("'{}' in parents", parent);
            } else {
                query = format!("{} and '{}' in parents", query, parent);
            }
        }
        let params = crate::commands::drive::list::ListParams {
            query: if query.is_empty() { None } else { Some(query) },
            max_results: args.limit.unwrap_or(20),
            page_token: args.page_token,
            order_by: None,
            fields: None,
            corpora: None,
            include_permissions: false,
        };
        match crate::commands::drive::list::list_files(&client, params).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Get metadata for a specific Google Drive file or folder by ID.")]
    async fn drive_get(&self, Parameters(args): Parameters<DriveGetArgs>) -> String {
        let client = ApiClient::drive(self.token_manager.clone());
        match crate::commands::drive::list::get_file(&client, &args.id, None).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Create a new folder in Google Drive.")]
    async fn drive_mkdir(&self, Parameters(args): Parameters<DriveMkdirArgs>) -> String {
        let client = ApiClient::drive(self.token_manager.clone());
        match crate::commands::drive::mkdir::create_folder(&client, &args.name, args.parent.as_deref()).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Move a Google Drive file to a different folder.")]
    async fn drive_move(&self, Parameters(args): Parameters<DriveMoveArgs>) -> String {
        let client = ApiClient::drive(self.token_manager.clone());
        match crate::commands::drive::operations::move_file(&client, &args.id, &args.to, true).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Copy a Google Drive file, optionally to a new name or folder.")]
    async fn drive_copy(&self, Parameters(args): Parameters<DriveCopyArgs>) -> String {
        let client = ApiClient::drive(self.token_manager.clone());
        match crate::commands::drive::operations::copy_file(
            &client,
            &args.id,
            args.name.as_deref(),
            args.parent.as_deref(),
        ).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Rename a Google Drive file or folder.")]
    async fn drive_rename(&self, Parameters(args): Parameters<DriveRenameArgs>) -> String {
        let client = ApiClient::drive(self.token_manager.clone());
        match crate::commands::drive::operations::rename_file(&client, &args.id, &args.name).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Share a Google Drive file with a user (by email) or make it public (anyone=true). Role: reader, writer, commenter.")]
    async fn drive_share(&self, Parameters(args): Parameters<DriveShareArgs>) -> String {
        let client = ApiClient::drive(self.token_manager.clone());
        let role = args.role.as_deref().unwrap_or("reader");
        if args.anyone.unwrap_or(false) {
            match crate::commands::drive::share::share_with_anyone(&client, &args.id, role).await {
                Ok(r) => ok_json(&r),
                Err(e) => err_json(e),
            }
        } else if let Some(ref email) = args.email {
            match crate::commands::drive::share::share_with_user(&client, &args.id, email, role).await {
                Ok(r) => ok_json(&r),
                Err(e) => err_json(e),
            }
        } else {
            err_json("Must provide either email or anyone=true")
        }
    }

    #[tool(description = "List permissions for a Google Drive file or folder.")]
    async fn drive_permissions(&self, Parameters(args): Parameters<DriveIdArgs>) -> String {
        let client = ApiClient::drive(self.token_manager.clone());
        match crate::commands::drive::share::list_permissions(&client, &args.id).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Move a Google Drive file to trash (recoverable).")]
    async fn drive_trash(&self, Parameters(args): Parameters<DriveIdArgs>) -> String {
        let client = ApiClient::drive(self.token_manager.clone());
        match crate::commands::drive::delete::trash_file(&client, &args.id).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Permanently delete a Google Drive file (bypasses trash). Use drive_trash for recoverable deletion.")]
    async fn drive_delete(&self, Parameters(args): Parameters<DriveIdArgs>) -> String {
        let client = ApiClient::drive(self.token_manager.clone());
        match crate::commands::drive::delete::delete_file(&client, &args.id).await {
            Ok(()) => r#"{"success":true}"#.to_string(),
            Err(e) => err_json(e),
        }
    }

    // ===== CALENDAR TOOLS =====

    #[tool(description = "List Google Calendar events. Returns JSON with items array of events.")]
    async fn calendar_list(&self, Parameters(args): Parameters<CalendarListArgs>) -> String {
        let client = ApiClient::calendar(self.token_manager.clone());
        let params = crate::commands::calendar::list::ListEventsParams {
            calendar_id: args.calendar.unwrap_or_else(|| "primary".to_string()),
            time_min: args.time_min,
            time_max: args.time_max,
            max_results: args.limit.unwrap_or(20),
            single_events: true,
            order_by: Some("startTime".to_string()),
            page_token: args.page_token,
            sync_token: None,
        };
        match crate::commands::calendar::list::list_events(&client, params).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Create a new Google Calendar event.")]
    async fn calendar_create(&self, Parameters(args): Parameters<CalendarCreateArgs>) -> String {
        let client = ApiClient::calendar(self.token_manager.clone());
        let params = crate::commands::calendar::create::CreateEventParams {
            calendar_id: args.calendar.unwrap_or_else(|| "primary".to_string()),
            summary: args.summary,
            start: args.start,
            end: args.end,
            description: args.description,
            location: None,
            attendees: None,
            time_zone: None,
        };
        match crate::commands::calendar::create::create_event(&client, params).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Update an existing Google Calendar event.")]
    async fn calendar_update(&self, Parameters(args): Parameters<CalendarUpdateArgs>) -> String {
        let client = ApiClient::calendar(self.token_manager.clone());
        let params = crate::commands::calendar::update::UpdateEventParams {
            calendar_id: args.calendar.unwrap_or_else(|| "primary".to_string()),
            event_id: args.id,
            summary: args.summary,
            description: None,
            location: None,
            start: args.start,
            end: args.end,
            time_zone: None,
        };
        match crate::commands::calendar::update::update_event(&client, params).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Delete a Google Calendar event.")]
    async fn calendar_delete(&self, Parameters(args): Parameters<CalendarDeleteArgs>) -> String {
        let client = ApiClient::calendar(self.token_manager.clone());
        let calendar_id = args.calendar.as_deref().unwrap_or("primary");
        match crate::commands::calendar::delete::delete_event(&client, calendar_id, &args.id).await {
            Ok(()) => r#"{"success":true}"#.to_string(),
            Err(e) => err_json(e),
        }
    }

    // ===== DOCS TOOLS =====

    #[tool(description = "Get the content of a Google Docs document as plain text (default) or markdown.")]
    async fn docs_get(&self, Parameters(args): Parameters<DocsGetArgs>) -> String {
        let client = ApiClient::docs(self.token_manager.clone());
        match crate::commands::docs::get::get_document(&client, &args.id).await {
            Ok(doc) => {
                if args.markdown.unwrap_or(false) {
                    let md = crate::commands::docs::get::document_to_markdown(&doc);
                    serde_json::json!({"documentId": doc.document_id, "title": doc.title, "content": md}).to_string()
                } else {
                    let text = crate::commands::docs::get::document_to_text(&doc);
                    serde_json::json!({"documentId": doc.document_id, "title": doc.title, "content": text}).to_string()
                }
            }
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Create a new Google Docs document with the given title.")]
    async fn docs_create(&self, Parameters(args): Parameters<DocsCreateArgs>) -> String {
        let client = ApiClient::docs(self.token_manager.clone());
        match crate::commands::docs::create::create_document(&client, &args.title).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Append text to the end of a Google Docs document.")]
    async fn docs_append(&self, Parameters(args): Parameters<DocsAppendArgs>) -> String {
        let client = ApiClient::docs(self.token_manager.clone());
        match crate::commands::docs::update::append_text(&client, &args.id, &args.text).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Replace all occurrences of text in a Google Docs document.")]
    async fn docs_replace(&self, Parameters(args): Parameters<DocsReplaceArgs>) -> String {
        let client = ApiClient::docs(self.token_manager.clone());
        match crate::commands::docs::update::replace_text(
            &client,
            &args.id,
            &args.find,
            &args.replace_with,
            args.match_case.unwrap_or(false),
        ).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Execute a batchUpdate on a Google Docs document. payload must be a JSON string with a 'requests' array.")]
    async fn docs_batch_update(&self, Parameters(args): Parameters<DocsBatchUpdateArgs>) -> String {
        let client = ApiClient::docs(self.token_manager.clone());
        let payload: serde_json::Value = match serde_json::from_str(&args.payload) {
            Ok(v) => v,
            Err(e) => return err_json(format!("Invalid JSON payload: {}", e)),
        };
        let path = format!("/documents/{}:batchUpdate", args.id);
        match client.post::<serde_json::Value, serde_json::Value>(&path, &payload).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    // ===== SHEETS TOOLS =====

    #[tool(description = "Get values from a Google Sheets range. range must be in A1 notation e.g. Sheet1!A1:C10.")]
    async fn sheets_get(&self, Parameters(args): Parameters<SheetsGetArgs>) -> String {
        let client = ApiClient::sheets(self.token_manager.clone());
        match crate::commands::sheets::get::get_values(&client, &args.id, &args.range).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Create a new Google Sheets spreadsheet with the given title.")]
    async fn sheets_create(&self, Parameters(args): Parameters<SheetsCreateArgs>) -> String {
        let client = ApiClient::sheets(self.token_manager.clone());
        match crate::commands::sheets::create::create_spreadsheet(&client, &args.title).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Update values in a Google Sheets range. values must be a JSON 2D array string e.g. [[\"Name\",\"Age\"],[\"Alice\",\"30\"]].")]
    async fn sheets_update(&self, Parameters(args): Parameters<SheetsUpdateArgs>) -> String {
        let client = ApiClient::sheets(self.token_manager.clone());
        let values: Vec<Vec<serde_json::Value>> = match serde_json::from_str(&args.values) {
            Ok(v) => v,
            Err(e) => return err_json(format!("Invalid JSON values: {}", e)),
        };
        let params = crate::commands::sheets::update::UpdateParams {
            spreadsheet_id: args.id,
            range: args.range,
            values,
            value_input_option: crate::commands::sheets::update::ValueInputOption::UserEntered,
        };
        match crate::commands::sheets::update::update_values(&client, params).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Append rows to a Google Sheets range. values must be a JSON 2D array string.")]
    async fn sheets_append(&self, Parameters(args): Parameters<SheetsAppendArgs>) -> String {
        let client = ApiClient::sheets(self.token_manager.clone());
        let values: Vec<Vec<serde_json::Value>> = match serde_json::from_str(&args.values) {
            Ok(v) => v,
            Err(e) => return err_json(format!("Invalid JSON values: {}", e)),
        };
        match crate::commands::sheets::update::append_values(
            &client,
            &args.id,
            &args.range,
            values,
            crate::commands::sheets::update::ValueInputOption::UserEntered,
        ).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Clear values in a Google Sheets range.")]
    async fn sheets_clear(&self, Parameters(args): Parameters<SheetsClearArgs>) -> String {
        let client = ApiClient::sheets(self.token_manager.clone());
        match crate::commands::sheets::update::clear_values(&client, &args.id, &args.range).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "List all sheet tabs in a Google Sheets spreadsheet.")]
    async fn sheets_list_sheets(&self, Parameters(args): Parameters<SheetsIdArgs>) -> String {
        let client = ApiClient::sheets(self.token_manager.clone());
        match crate::commands::sheets::get::get_spreadsheet(&client, &args.id).await {
            Ok(r) => {
                let sheets: Vec<serde_json::Value> = r.sheets.iter().map(|s| {
                    serde_json::json!({
                        "sheetId": s.properties.sheet_id,
                        "title": s.properties.title,
                        "index": s.properties.index,
                    })
                }).collect();
                serde_json::json!({"spreadsheetId": r.spreadsheet_id, "title": r.properties.title, "sheets": sheets}).to_string()
            }
            Err(e) => err_json(e),
        }
    }

    // ===== SLIDES TOOLS =====

    #[tool(description = "Get the content of a Google Slides presentation. Returns extracted text by default, or full JSON structure with full=true.")]
    async fn slides_get(&self, Parameters(args): Parameters<SlidesGetArgs>) -> String {
        let client = ApiClient::slides(self.token_manager.clone());
        match crate::commands::slides::get::get_presentation(&client, &args.id).await {
            Ok(p) => {
                if args.full.unwrap_or(false) {
                    ok_json(&p)
                } else {
                    let text = crate::commands::slides::get::extract_all_text(&p);
                    serde_json::json!({"presentationId": p.presentation_id, "title": p.title, "slideCount": p.slides.len(), "content": text}).to_string()
                }
            }
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Get the content of a specific slide by 0-based index from a Google Slides presentation.")]
    async fn slides_page(&self, Parameters(args): Parameters<SlidesPageArgs>) -> String {
        let client = ApiClient::slides(self.token_manager.clone());
        match crate::commands::slides::get::get_presentation(&client, &args.id).await {
            Ok(p) => {
                let idx = args.page as usize;
                if idx >= p.slides.len() {
                    return err_json(format!("Slide index {} out of range (presentation has {} slides)", idx, p.slides.len()));
                }
                if args.full.unwrap_or(false) {
                    ok_json(&p.slides[idx])
                } else {
                    let text = crate::commands::slides::get::extract_page_text(&p.slides[idx]);
                    serde_json::json!({"presentationId": p.presentation_id, "slideIndex": idx, "content": text}).to_string()
                }
            }
            Err(e) => err_json(e),
        }
    }

    // ===== TASKS TOOLS =====

    #[tool(description = "List all task lists (collections) in Google Tasks.")]
    async fn tasks_lists(&self) -> String {
        let client = ApiClient::tasks(self.token_manager.clone());
        match crate::commands::tasks::list::list_task_lists(&client).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "List tasks in a Google Tasks list. Use list=@default for the primary list.")]
    async fn tasks_list(&self, Parameters(args): Parameters<TasksListArgs>) -> String {
        let client = ApiClient::tasks(self.token_manager.clone());
        let params = crate::commands::tasks::list::ListTasksParams {
            task_list_id: args.list.unwrap_or_else(|| "@default".to_string()),
            max_results: args.limit.unwrap_or(20),
            show_completed: args.show_completed.unwrap_or(false),
            show_hidden: false,
            page_token: args.page_token,
        };
        match crate::commands::tasks::list::list_tasks(&client, params).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Create a new task in Google Tasks.")]
    async fn tasks_create(&self, Parameters(args): Parameters<TasksCreateArgs>) -> String {
        let client = ApiClient::tasks(self.token_manager.clone());
        let params = crate::commands::tasks::create::CreateTaskParams {
            task_list_id: args.list.unwrap_or_else(|| "@default".to_string()),
            title: args.title,
            notes: args.notes,
            due: args.due,
            parent: None,
        };
        match crate::commands::tasks::create::create_task(&client, params).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Update a Google Tasks task: change title or mark as complete.")]
    async fn tasks_update(&self, Parameters(args): Parameters<TasksUpdateArgs>) -> String {
        let client = ApiClient::tasks(self.token_manager.clone());
        let list_id = args.list.unwrap_or_else(|| "@default".to_string());
        let status = args.complete.map(|c| {
            if c {
                crate::commands::tasks::update::TaskStatus::Completed
            } else {
                crate::commands::tasks::update::TaskStatus::NeedsAction
            }
        });
        let params = crate::commands::tasks::update::UpdateTaskParams {
            task_list_id: list_id,
            task_id: args.id,
            title: args.title,
            notes: None,
            due: None,
            status,
        };
        match crate::commands::tasks::update::update_task(&client, params).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Delete a Google Tasks task permanently.")]
    async fn tasks_delete(&self, Parameters(args): Parameters<TasksDeleteArgs>) -> String {
        let client = ApiClient::tasks(self.token_manager.clone());
        let list_id = args.list.as_deref().unwrap_or("@default");
        match crate::commands::tasks::update::delete_task(&client, list_id, &args.id).await {
            Ok(()) => r#"{"success":true}"#.to_string(),
            Err(e) => err_json(e),
        }
    }

    // ===== CHAT TOOLS =====

    #[tool(description = "List Google Chat spaces (rooms, DMs, group chats) the user belongs to.")]
    async fn chat_spaces_list(&self, Parameters(args): Parameters<ChatSpacesListArgs>) -> String {
        let client = ApiClient::chat(self.token_manager.clone());
        let filter = args.space_type.map(|t| format!("spaceType = \"{}\"", t));
        let params = crate::commands::chat::spaces::ListSpacesParams {
            page_size: args.limit.unwrap_or(100),
            page_token: None,
            filter,
        };
        match crate::commands::chat::spaces::list_spaces(&client, params).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Find the Google Chat direct message space with a specific user by their email address.")]
    async fn chat_find_dm(&self, Parameters(args): Parameters<ChatFindDmArgs>) -> String {
        let client = ApiClient::chat(self.token_manager.clone());
        match crate::commands::chat::spaces::find_direct_message(&client, &args.email).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "List messages in a Google Chat space. space should be in 'spaces/abc123' format.")]
    async fn chat_messages_list(&self, Parameters(args): Parameters<ChatMessagesListArgs>) -> String {
        let client = ApiClient::chat(self.token_manager.clone());
        let filter = if args.today.unwrap_or(false) {
            let today = chrono::Utc::now()
                .format("%Y-%m-%dT00:00:00Z")
                .to_string();
            Some(format!("createTime > \"{}\"", today))
        } else {
            None
        };
        let params = crate::commands::chat::messages::ListMessagesParams {
            space_name: args.space,
            page_size: args.limit.unwrap_or(50),
            page_token: args.page_token,
            order_by: Some("createTime DESC".to_string()),
            filter,
        };
        match crate::commands::chat::messages::list_messages(&client, params).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Send a message to a Google Chat space. space should be in 'spaces/abc123' format.")]
    async fn chat_send(&self, Parameters(args): Parameters<ChatSendArgs>) -> String {
        let client = ApiClient::chat(self.token_manager.clone());
        match crate::commands::chat::messages::send_message(&client, &args.space, &args.text, None).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Get unread Google Chat messages across spaces. Returns spaces with unread messages since the given time period (e.g. '7d', '24h').")]
    async fn chat_unread(&self, Parameters(args): Parameters<ChatUnreadArgs>) -> String {
        let client = ApiClient::chat(self.token_manager.clone());
        let token = match self.get_token().await {
            Ok(t) => t,
            Err(e) => return err_json(e),
        };
        let since = args.since.as_deref().unwrap_or("7d");
        let space_type = args.space_type.as_deref();
        match crate::commands::chat::read_state::get_unread_messages(&client, 25, space_type, since, false, Some(&token)).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Mark a Google Chat space as read (updates the read state to current time). Provide space name or all=true to mark all spaces.")]
    async fn chat_mark_read(&self, Parameters(args): Parameters<ChatMarkReadArgs>) -> String {
        let client = ApiClient::chat(self.token_manager.clone());
        let now = chrono::Utc::now().to_rfc3339();
        if args.all.unwrap_or(false) {
            // List all spaces and mark each as read
            let params = crate::commands::chat::spaces::ListSpacesParams {
                page_size: 200,
                page_token: None,
                filter: None,
            };
            match crate::commands::chat::spaces::list_spaces(&client, params).await {
                Ok(spaces_resp) => {
                    let mut marked = 0usize;
                    let mut errors = Vec::new();
                    for space in &spaces_resp.spaces {
                        if let Some(ref name) = space.name {
                            match crate::commands::chat::read_state::update_space_read_state(&client, name, &now).await {
                                Ok(_) => marked += 1,
                                Err(e) => errors.push(format!("{}: {}", name, e)),
                            }
                        }
                    }
                    serde_json::json!({"success": true, "marked": marked, "errors": errors}).to_string()
                }
                Err(e) => err_json(e),
            }
        } else if let Some(ref space) = args.space {
            match crate::commands::chat::read_state::update_space_read_state(&client, space, &now).await {
                Ok(r) => ok_json(&r),
                Err(e) => err_json(e),
            }
        } else {
            err_json("Must provide either space name or all=true")
        }
    }

    // ===== CONTACTS TOOLS =====

    #[tool(description = "List personal Google Contacts.")]
    async fn contacts_list(&self, Parameters(args): Parameters<ContactsListArgs>) -> String {
        let client = ApiClient::contacts(self.token_manager.clone());
        let params = crate::commands::contacts::list::ListContactsParams {
            page_size: args.limit.unwrap_or(100),
            page_token: args.page_token,
        };
        match crate::commands::contacts::list::list_contacts(&client, params).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Search personal Google Contacts by name, email, or phone number.")]
    async fn contacts_search(&self, Parameters(args): Parameters<ContactsSearchArgs>) -> String {
        let client = ApiClient::contacts(self.token_manager.clone());
        match crate::commands::contacts::search::search_contacts(&client, &args.query, args.limit.unwrap_or(50)).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Get a specific Google Contact by resource name (e.g. people/c123456).")]
    async fn contacts_get(&self, Parameters(args): Parameters<ContactsGetArgs>) -> String {
        let client = ApiClient::contacts(self.token_manager.clone());
        match crate::commands::contacts::list::get_contact(&client, &args.name).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Create a new Google Contact.")]
    async fn contacts_create(&self, Parameters(args): Parameters<ContactsCreateArgs>) -> String {
        let client = ApiClient::contacts(self.token_manager.clone());
        let params = crate::commands::contacts::create::CreateContactParams {
            given_name: args.given,
            family_name: args.family,
            email: args.email,
            phone: args.phone,
            org_name: None,
            org_title: None,
        };
        match crate::commands::contacts::create::create_contact(&client, params).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Delete a Google Contact by resource name (e.g. people/c123456).")]
    async fn contacts_delete(&self, Parameters(args): Parameters<ContactsDeleteArgs>) -> String {
        let client = ApiClient::contacts(self.token_manager.clone());
        match crate::commands::contacts::create::delete_contact(&client, &args.name).await {
            Ok(()) => r#"{"success":true}"#.to_string(),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "List the Google Workspace directory (domain users). Requires Workspace account.")]
    async fn contacts_directory_list(&self, Parameters(args): Parameters<ContactsDirectoryArgs>) -> String {
        let client = ApiClient::contacts(self.token_manager.clone());
        let params = crate::commands::contacts::search::DirectoryListParams {
            page_size: args.limit.unwrap_or(100),
            page_token: args.page_token,
        };
        match crate::commands::contacts::search::list_directory(&client, params).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }

    #[tool(description = "Search the Google Workspace directory by name or email. Requires Workspace account.")]
    async fn contacts_directory_search(&self, Parameters(args): Parameters<ContactsDirectorySearchArgs>) -> String {
        let client = ApiClient::contacts(self.token_manager.clone());
        match crate::commands::contacts::search::search_directory(&client, &args.query, args.limit.unwrap_or(50), None).await {
            Ok(r) => ok_json(&r),
            Err(e) => err_json(e),
        }
    }
}

// ============================================================
// ServerHandler implementation
// ============================================================

#[tool_handler]
impl ServerHandler for WorkspaceServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("Google Workspace MCP server. Available tools: gmail_list, gmail_get, gmail_send, gmail_reply, gmail_labels, gmail_modify, gmail_trash, gmail_delete, drive_list, drive_get, drive_mkdir, drive_move, drive_copy, drive_rename, drive_share, drive_permissions, drive_trash, drive_delete, calendar_list, calendar_create, calendar_update, calendar_delete, docs_get, docs_create, docs_append, docs_replace, docs_batch_update, sheets_get, sheets_create, sheets_update, sheets_append, sheets_clear, sheets_list_sheets, slides_get, slides_page, tasks_lists, tasks_list, tasks_create, tasks_update, tasks_delete, chat_spaces_list, chat_find_dm, chat_messages_list, chat_send, chat_unread, chat_mark_read, contacts_list, contacts_search, contacts_get, contacts_create, contacts_delete, contacts_directory_list, contacts_directory_search")
    }
}

// ============================================================
// Entry point
// ============================================================

pub async fn run(token_manager: Arc<RwLock<TokenManager>>) {
    let service = WorkspaceServer::new(token_manager)
        .serve(stdio())
        .await
        .expect("MCP server failed to start");
    service.waiting().await.expect("MCP server error");
}
