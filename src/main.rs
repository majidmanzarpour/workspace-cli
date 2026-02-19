use clap::{Parser, Subcommand};
use std::sync::Arc;
use tokio::sync::RwLock;
use workspace_cli::Config;
use workspace_cli::auth::TokenManager;
use workspace_cli::client::ApiClient;
use workspace_cli::output::{Formatter, OutputFormat};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "workspace-cli")]
#[command(about = "High-performance Google Workspace CLI for AI agent integration")]
#[command(long_about = "workspace-cli provides programmatic access to Google Workspace APIs \
    (Gmail, Drive, Calendar, Docs, Sheets, Slides, Tasks) with structured JSON output \
    optimized for AI agent consumption.\n\n\
    All commands output TOON by default for token efficiency. Use --format json for JSON.\n\
    Use --fields to limit response fields.")]
#[command(author, version)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format: toon, json, jsonl, csv
    #[arg(long, short = 'f', global = true, default_value = "toon")]
    format: String,

    /// Fields to include in response (comma-separated)
    #[arg(long, global = true)]
    fields: Option<String>,

    /// Write output to file instead of stdout
    #[arg(long, short = 'o', global = true)]
    output: Option<String>,

    /// Suppress non-essential output
    #[arg(long, short = 'q', global = true)]
    quiet: bool,

    /// Impersonate user via domain-wide delegation (requires service account)
    #[arg(long = "as", global = true, value_name = "EMAIL")]
    impersonate: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Gmail operations
    #[command(long_about = "Gmail operations for listing, reading, and sending emails.\n\n\
        Examples:\n\
        List unread emails:\n  \
        workspace-cli gmail list --query 'is:unread' --limit 10\n\n\
        Get specific email with decoded body:\n  \
        workspace-cli gmail get <message-id> --decode-body\n\n\
        Send an email:\n  \
        workspace-cli gmail send --to user@example.com --subject 'Hello' --body 'Message'\n\n\
        Search emails by sender:\n  \
        workspace-cli gmail list --query 'from:boss@company.com' --limit 5")]
    Gmail {
        #[command(subcommand)]
        command: GmailCommands,
    },
    /// Google Drive operations
    #[command(long_about = "Google Drive operations for file management and storage.\n\n\
        Examples:\n\
        List recent files:\n  \
        workspace-cli drive list --limit 20\n\n\
        Search for documents:\n  \
        workspace-cli drive list --query \"mimeType='application/vnd.google-apps.document'\"\n\n\
        Get file metadata:\n  \
        workspace-cli drive get <file-id>\n\n\
        Upload a file:\n  \
        workspace-cli drive upload /path/to/file.pdf --parent <folder-id>\n\n\
        Download a file:\n  \
        workspace-cli drive download <file-id> --output /path/to/save")]
    Drive {
        #[command(subcommand)]
        command: DriveCommands,
    },
    /// Google Calendar operations
    #[command(long_about = "Google Calendar operations for event management and scheduling.\n\n\
        Examples:\n\
        List upcoming events:\n  \
        workspace-cli calendar list --time-min 2025-01-01T00:00:00Z --limit 10\n\n\
        List today's events:\n  \
        workspace-cli calendar list --time-min $(date -u +%Y-%m-%dT00:00:00Z) \\\n    \
        --time-max $(date -u -d '+1 day' +%Y-%m-%dT00:00:00Z)\n\n\
        Create an event:\n  \
        workspace-cli calendar create --summary 'Team Meeting' \\\n    \
        --start 2025-01-15T14:00:00Z --end 2025-01-15T15:00:00Z\n\n\
        Update an event:\n  \
        workspace-cli calendar update <event-id> --summary 'Updated Meeting'\n\n\
        Delete an event:\n  \
        workspace-cli calendar delete <event-id>")]
    Calendar {
        #[command(subcommand)]
        command: CalendarCommands,
    },
    /// Google Docs operations
    #[command(long_about = "Google Docs operations for document access and editing.\n\n\
        Examples:\n\
        Get document content:\n  \
        workspace-cli docs get <document-id>\n\n\
        Get document as markdown:\n  \
        workspace-cli docs get <document-id> --markdown\n\n\
        Append text to document:\n  \
        workspace-cli docs append <document-id> 'New paragraph text'\n\n\
        Extract content for AI processing:\n  \
        workspace-cli docs get <document-id> --markdown --fields content")]
    Docs {
        #[command(subcommand)]
        command: DocsCommands,
    },
    /// Google Sheets operations
    #[command(long_about = "Google Sheets operations for spreadsheet data access and manipulation.\n\n\
        Examples:\n\
        Get range of cells:\n  \
        workspace-cli sheets get <spreadsheet-id> --range 'Sheet1!A1:C10'\n\n\
        Update cells:\n  \
        workspace-cli sheets update <spreadsheet-id> --range 'Sheet1!A1:B2' \\\n    \
        --values '[[\"Name\",\"Value\"],[\"Item1\",\"100\"]]'\n\n\
        Append rows:\n  \
        workspace-cli sheets append <spreadsheet-id> --range 'Sheet1!A:B' \\\n    \
        --values '[[\"New Row\",\"Data\"]]'\n\n\
        Extract data for analysis:\n  \
        workspace-cli sheets get <spreadsheet-id> --range 'Sheet1!A:Z' --format jsonl")]
    Sheets {
        #[command(subcommand)]
        command: SheetsCommands,
    },
    /// Google Slides operations
    #[command(long_about = "Google Slides operations for presentation access and content extraction.\n\n\
        Examples:\n\
        Get presentation info:\n  \
        workspace-cli slides get <presentation-id>\n\n\
        Get presentation text only:\n  \
        workspace-cli slides get <presentation-id> --text-only\n\n\
        Get specific slide:\n  \
        workspace-cli slides page <presentation-id> --page 0\n\n\
        Extract all text from presentation:\n  \
        workspace-cli slides get <presentation-id> --text-only --format json")]
    Slides {
        #[command(subcommand)]
        command: SlidesCommands,
    },
    /// Google Tasks operations
    #[command(long_about = "Google Tasks operations for task and to-do list management.\n\n\
        Examples:\n\
        List all task lists:\n  \
        workspace-cli tasks lists\n\n\
        List tasks in default list:\n  \
        workspace-cli tasks list\n\n\
        List tasks including completed:\n  \
        workspace-cli tasks list --show-completed\n\n\
        Create a task:\n  \
        workspace-cli tasks create 'Buy groceries' --due 2025-01-20T12:00:00Z\n\n\
        Update and complete a task:\n  \
        workspace-cli tasks update <task-id> --complete\n\n\
        Delete a task:\n  \
        workspace-cli tasks delete <task-id>")]
    Tasks {
        #[command(subcommand)]
        command: TasksCommands,
    },
    /// Authentication management
    #[command(long_about = "Authentication management for Google Workspace APIs.\n\n\
        Examples:\n\
        Login with OAuth2 (interactive browser flow):\n  \
        workspace-cli auth login\n\n\
        Login with custom credentials file:\n  \
        workspace-cli auth login --credentials /path/to/credentials.json\n\n\
        Check authentication status:\n  \
        workspace-cli auth status\n\n\
        Logout and clear stored tokens:\n  \
        workspace-cli auth logout\n\n\
        Note: First-time login requires OAuth2 credentials from Google Cloud Console.")]
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    /// Execute batch API requests (up to 100 per batch)
    #[command(long_about = "Execute multiple API requests in a single HTTP call for efficiency.\n\n\
        Batch requests allow you to combine up to 100 API calls into a single request,\n\
        significantly reducing latency and quota usage for bulk operations.\n\n\
        Input format (JSON array):\n  \
        [{\"id\":\"req1\",\"method\":\"GET\",\"path\":\"/users/me/messages/abc123\"},\n   \
        {\"id\":\"req2\",\"method\":\"POST\",\"path\":\"/users/me/messages/xyz/modify\",\n    \
        \"body\":{\"addLabelIds\":[\"STARRED\"]}}]\n\n\
        Examples:\n\
        Batch Gmail requests from JSON string:\n  \
        workspace-cli batch gmail --requests '[{\"id\":\"1\",\"method\":\"GET\",\"path\":\"/users/me/messages/abc\"}]'\n\n\
        Batch Gmail requests from file:\n  \
        workspace-cli batch gmail --file requests.json\n\n\
        Batch from stdin (pipe):\n  \
        echo '[{\"id\":\"1\",\"method\":\"GET\",\"path\":\"/users/me/messages/abc\"}]' | workspace-cli batch gmail")]
    Batch {
        #[command(subcommand)]
        command: BatchCommands,
    },
    /// Google Chat operations
    #[command(long_about = "Google Chat operations for messaging and space management.\n\n\
        Examples:\n\
        List spaces:\n  \
        workspace-cli chat spaces-list\n\n\
        List only DM spaces:\n  \
        workspace-cli chat spaces-list --type DIRECT_MESSAGE\n\n\
        Find DM space by email:\n  \
        workspace-cli chat find-dm --email user@company.com\n\n\
        List messages in a space:\n  \
        workspace-cli chat messages-list --space spaces/abc123\n\n\
        List today's messages:\n  \
        workspace-cli chat messages-list --space spaces/abc123 --today\n\n\
        Send a message:\n  \
        workspace-cli chat send --space spaces/abc123 --text 'Hello team'")]
    Chat {
        #[command(subcommand)]
        command: ChatCommands,
    },
    /// Google Contacts operations (People API)
    #[command(long_about = "Google Contacts operations using the People API.\n\n\
        Examples:\n\
        List contacts:\n  \
        workspace-cli contacts list --limit 20\n\n\
        Search contacts:\n  \
        workspace-cli contacts search --query 'John'\n\n\
        Get a contact:\n  \
        workspace-cli contacts get people/c123456\n\n\
        List workspace directory:\n  \
        workspace-cli contacts directory-list")]
    Contacts {
        #[command(subcommand)]
        command: ContactsCommands,
    },
    /// Google Groups operations (Admin Directory API)
    #[command(long_about = "Google Groups operations using the Admin Directory API.\n\n\
        Examples:\n\
        List groups you belong to:\n  \
        workspace-cli groups list --email user@company.com\n\n\
        List all groups in a domain:\n  \
        workspace-cli groups list --domain company.com\n\n\
        List group members:\n  \
        workspace-cli groups members group@company.com")]
    Groups {
        #[command(subcommand)]
        command: GroupsCommands,
    },
    /// Admin directory operations (users)
    Admin {
        #[command(subcommand)]
        command: AdminCommands,
    },
}

#[derive(Debug, Subcommand)]
enum GmailCommands {
    /// List messages
    List {
        /// Search query (Gmail search syntax)
        #[arg(long)]
        query: Option<String>,
        /// Maximum number of results
        #[arg(long, default_value = "20")]
        limit: u32,
        /// Label ID to filter by
        #[arg(long)]
        label: Option<String>,
    },
    /// Get a specific message
    Get {
        /// Message ID
        id: String,
        /// Return full message structure (includes all headers, MIME parts, raw body)
        #[arg(long)]
        full: bool,
    },
    /// Send an email
    Send {
        /// Recipient email
        #[arg(long)]
        to: String,
        /// Email subject
        #[arg(long)]
        subject: String,
        /// Email body (or use --body-file)
        #[arg(long)]
        body: Option<String>,
        /// Read body from file
        #[arg(long)]
        body_file: Option<String>,
    },
    /// Create a draft
    Draft {
        /// Recipient email
        #[arg(long)]
        to: String,
        /// Email subject
        #[arg(long)]
        subject: String,
        /// Email body
        #[arg(long)]
        body: Option<String>,
    },
    /// Permanently delete a message (bypasses trash)
    Delete {
        /// Message ID to delete
        id: String,
    },
    /// Move message to trash
    Trash {
        /// Message ID to trash
        id: String,
    },
    /// Remove message from trash
    Untrash {
        /// Message ID to untrash
        id: String,
    },
    /// List all labels
    Labels,
    /// Modify labels on a message
    Modify {
        /// Message ID
        id: String,
        /// Labels to add (comma-separated)
        #[arg(long)]
        add_labels: Option<String>,
        /// Labels to remove (comma-separated)
        #[arg(long)]
        remove_labels: Option<String>,
        /// Mark as read
        #[arg(long)]
        mark_read: bool,
        /// Mark as unread
        #[arg(long)]
        mark_unread: bool,
        /// Star message
        #[arg(long)]
        star: bool,
        /// Unstar message
        #[arg(long)]
        unstar: bool,
        /// Archive message (remove from inbox)
        #[arg(long)]
        archive: bool,
    },
    /// Reply to a message
    Reply {
        /// Message ID to reply to
        id: String,
        /// Reply body (or use --body-file)
        #[arg(long)]
        body: Option<String>,
        /// Read body from file
        #[arg(long)]
        body_file: Option<String>,
        /// Reply-all (include Cc recipients)
        #[arg(long)]
        all: bool,
    },
    /// Create a draft reply to a message
    ReplyDraft {
        /// Message ID to reply to
        id: String,
        /// Reply body
        #[arg(long)]
        body: Option<String>,
        /// Reply-all (include Cc recipients)
        #[arg(long)]
        all: bool,
    },
}

#[derive(Debug, Subcommand)]
enum DriveCommands {
    /// List files
    List {
        /// Search query (Drive query syntax)
        #[arg(long)]
        query: Option<String>,
        /// Maximum results
        #[arg(long, default_value = "20")]
        limit: u32,
        /// Parent folder ID
        #[arg(long)]
        parent: Option<String>,
        /// Corpora to search: user, domain, drive, allDrives
        #[arg(long)]
        corpora: Option<String>,
        /// Include file permissions in response
        #[arg(long)]
        include_permissions: bool,
        /// Page token for pagination (from nextPageToken in previous response)
        #[arg(long)]
        page_token: Option<String>,
    },
    /// Upload a file
    Upload {
        /// Local file path
        file: String,
        /// Destination folder ID
        #[arg(long)]
        parent: Option<String>,
        /// Custom name for uploaded file
        #[arg(long)]
        name: Option<String>,
    },
    /// Download a file
    Download {
        /// File ID
        id: String,
        /// Output path
        #[arg(long, short = 'o')]
        output: Option<String>,
    },
    /// Get file metadata
    Get {
        /// File ID
        id: String,
    },
    /// Permanently delete a file (bypasses trash)
    Delete {
        /// File ID to delete
        id: String,
    },
    /// Move file to trash
    Trash {
        /// File ID to trash
        id: String,
    },
    /// Restore file from trash
    Untrash {
        /// File ID to restore
        id: String,
    },
    /// Create a new folder
    Mkdir {
        /// Folder name
        name: String,
        /// Parent folder ID
        #[arg(long)]
        parent: Option<String>,
    },
    /// Move a file to a different folder
    Move {
        /// File ID to move
        id: String,
        /// Destination folder ID
        #[arg(long)]
        to: String,
    },
    /// Copy a file
    Copy {
        /// File ID to copy
        id: String,
        /// New name for the copy
        #[arg(long)]
        name: Option<String>,
        /// Destination folder ID
        #[arg(long)]
        parent: Option<String>,
    },
    /// Rename a file
    Rename {
        /// File ID to rename
        id: String,
        /// New name
        name: String,
    },
    /// Share a file
    Share {
        /// File ID to share
        id: String,
        /// Share with this email address
        #[arg(long)]
        email: Option<String>,
        /// Share with anyone (make public)
        #[arg(long)]
        anyone: bool,
        /// Role: reader, commenter, writer
        #[arg(long, default_value = "reader")]
        role: String,
    },
    /// List permissions on a file
    Permissions {
        /// File ID
        id: String,
    },
    /// Remove a permission from a file
    Unshare {
        /// File ID
        id: String,
        /// Permission ID to remove
        permission_id: String,
    },
    /// List all Shared Drives in the domain
    #[command(name = "drives-list")]
    DrivesList {
        /// Maximum results
        #[arg(long, default_value = "100")]
        limit: u32,
    },
    /// Recursively crawl a folder tree with concurrent API calls
    Tree {
        /// Root folder ID to start crawling from
        folder_id: String,
        /// Maximum depth to crawl (default: unlimited)
        #[arg(long)]
        max_depth: Option<u32>,
        /// Number of concurrent API requests (default: 10)
        #[arg(long, default_value = "10")]
        concurrency: usize,
        /// Include permissions for each item
        #[arg(long)]
        include_permissions: bool,
    },
}

#[derive(Debug, Subcommand)]
enum CalendarCommands {
    /// List events
    List {
        /// Calendar ID (default: primary)
        #[arg(long, default_value = "primary")]
        calendar: String,
        /// Start time (RFC3339)
        #[arg(long)]
        time_min: Option<String>,
        /// End time (RFC3339)
        #[arg(long)]
        time_max: Option<String>,
        /// Maximum results
        #[arg(long, default_value = "20")]
        limit: u32,
        /// Sync token for incremental sync
        #[arg(long)]
        sync_token: Option<String>,
        /// Return full event data (includes attendees, organizer, description, etc.)
        #[arg(long)]
        full: bool,
    },
    /// Create an event
    Create {
        /// Event summary/title
        #[arg(long)]
        summary: String,
        /// Start time (RFC3339)
        #[arg(long)]
        start: String,
        /// End time (RFC3339)
        #[arg(long)]
        end: String,
        /// Description
        #[arg(long)]
        description: Option<String>,
        /// Calendar ID
        #[arg(long, default_value = "primary")]
        calendar: String,
    },
    /// Update an event
    Update {
        /// Event ID
        id: String,
        /// New summary
        #[arg(long)]
        summary: Option<String>,
        /// New start time
        #[arg(long)]
        start: Option<String>,
        /// New end time
        #[arg(long)]
        end: Option<String>,
        /// Calendar ID
        #[arg(long, default_value = "primary")]
        calendar: String,
    },
    /// Delete an event
    Delete {
        /// Event ID
        id: String,
        /// Calendar ID
        #[arg(long, default_value = "primary")]
        calendar: String,
    },
}

#[derive(Debug, Subcommand)]
enum DocsCommands {
    /// Get document content
    Get {
        /// Document ID
        id: String,
        /// Output as markdown
        #[arg(long)]
        markdown: bool,
        /// Output as plain text (most token-efficient)
        #[arg(long)]
        text: bool,
    },
    /// Append text to document
    Append {
        /// Document ID
        id: String,
        /// Text to append
        text: String,
    },
    /// Create a new document
    Create {
        /// Document title
        title: String,
    },
    /// Replace text in document
    Replace {
        /// Document ID
        id: String,
        /// Text to find
        #[arg(long)]
        find: String,
        /// Text to replace with
        #[arg(long, name = "with")]
        replace_with: String,
        /// Match case
        #[arg(long)]
        match_case: bool,
    },
}

#[derive(Debug, Subcommand)]
enum SheetsCommands {
    /// Get spreadsheet values
    Get {
        /// Spreadsheet ID
        id: String,
        /// Range in A1 notation (e.g., Sheet1!A1:C10)
        #[arg(long)]
        range: String,
        /// Return full ValueRange (includes range, majorDimension metadata)
        #[arg(long)]
        full: bool,
    },
    /// Update spreadsheet values
    Update {
        /// Spreadsheet ID
        id: String,
        /// Range in A1 notation
        #[arg(long)]
        range: String,
        /// Values as JSON array of arrays
        #[arg(long)]
        values: String,
    },
    /// Append rows to spreadsheet
    Append {
        /// Spreadsheet ID
        id: String,
        /// Range in A1 notation
        #[arg(long)]
        range: String,
        /// Values as JSON array of arrays
        #[arg(long)]
        values: String,
    },
    /// Create a new spreadsheet
    Create {
        /// Spreadsheet title
        title: String,
        /// Optional sheet tab names (creates multiple tabs)
        #[arg(long, num_args = 1..)]
        sheets: Option<Vec<String>>,
    },
    /// Clear a range of cells
    Clear {
        /// Spreadsheet ID
        id: String,
        /// Range to clear in A1 notation
        #[arg(long)]
        range: String,
    },
    /// Add a new sheet tab to an existing spreadsheet
    #[command(name = "add-sheet")]
    AddSheet {
        /// Spreadsheet ID
        id: String,
        /// Name for the new sheet tab
        title: String,
    },
    /// Delete a sheet tab by its numeric sheet ID
    #[command(name = "delete-sheet")]
    DeleteSheet {
        /// Spreadsheet ID
        id: String,
        /// Sheet ID (numeric, from sheet properties)
        #[arg(long)]
        sheet_id: i64,
    },
    /// List all sheet tabs with their numeric sheet IDs
    #[command(name = "list-sheets")]
    ListSheets {
        /// Spreadsheet ID
        id: String,
    },
}

#[derive(Debug, Subcommand)]
enum SlidesCommands {
    /// Get presentation info
    Get {
        /// Presentation ID
        id: String,
        /// Return full presentation structure (includes masters, layouts, transforms, etc.)
        #[arg(long)]
        full: bool,
    },
    /// Get specific page
    Page {
        /// Presentation ID
        id: String,
        /// Page number (0-indexed)
        #[arg(long)]
        page: u32,
        /// Return full page structure (includes transforms, sizes, etc.)
        #[arg(long)]
        full: bool,
    },
}

#[derive(Debug, Subcommand)]
enum TasksCommands {
    /// List task lists
    Lists,
    /// List tasks in a task list
    List {
        /// Task list ID
        #[arg(long, default_value = "@default")]
        list: String,
        /// Maximum number of results (1-100)
        #[arg(long, default_value = "20")]
        limit: u32,
        /// Show completed tasks
        #[arg(long)]
        show_completed: bool,
        /// Return full task data (includes kind, etag, links, position, etc.)
        #[arg(long)]
        full: bool,
    },
    /// Create a task
    Create {
        /// Task title
        title: String,
        /// Task list ID
        #[arg(long, default_value = "@default")]
        list: String,
        /// Due date (RFC3339)
        #[arg(long)]
        due: Option<String>,
        /// Notes
        #[arg(long)]
        notes: Option<String>,
    },
    /// Update a task
    Update {
        /// Task ID
        id: String,
        /// Task list ID
        #[arg(long, default_value = "@default")]
        list: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// Mark as completed
        #[arg(long)]
        complete: bool,
    },
    /// Delete a task
    Delete {
        /// Task ID
        id: String,
        /// Task list ID
        #[arg(long, default_value = "@default")]
        list: String,
    },
}

#[derive(Debug, Subcommand)]
enum AuthCommands {
    /// Login with OAuth2 (interactive browser flow)
    Login {
        /// Path to OAuth2 client credentials JSON
        #[arg(long)]
        credentials: Option<String>,
    },
    /// Logout and clear stored tokens
    Logout,
    /// Show current authentication status
    Status,
}

#[derive(Debug, Subcommand)]
enum BatchCommands {
    /// Execute a batch of Gmail API requests
    Gmail {
        /// JSON array of requests
        #[arg(long)]
        requests: Option<String>,
        /// Read requests from JSON file
        #[arg(long)]
        file: Option<String>,
    },
    /// Execute a batch of Drive API requests
    Drive {
        /// JSON array of requests
        #[arg(long)]
        requests: Option<String>,
        /// Read requests from JSON file
        #[arg(long)]
        file: Option<String>,
    },
    /// Execute a batch of Calendar API requests
    Calendar {
        /// JSON array of requests
        #[arg(long)]
        requests: Option<String>,
        /// Read requests from JSON file
        #[arg(long)]
        file: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum ChatCommands {
    /// List Chat spaces
    SpacesList {
        /// Maximum results
        #[arg(long, default_value = "100")]
        limit: u32,
        /// Filter by space type: SPACE, DIRECT_MESSAGE, GROUP_CHAT
        #[arg(long = "type")]
        space_type: Option<String>,
    },
    /// Find spaces by display name
    SpacesFind {
        /// Display name to search for
        #[arg(long)]
        name: String,
    },
    /// Find a direct message space by participant email (single API call)
    FindDm {
        /// Email of the other participant
        #[arg(long)]
        email: String,
    },
    /// Create a Chat space
    SpacesCreate {
        /// Space display name
        #[arg(long)]
        name: String,
        /// Member emails to add
        #[arg(long)]
        member: Vec<String>,
    },
    /// List messages in a space
    MessagesList {
        /// Space name (e.g. spaces/abc123)
        #[arg(long)]
        space: String,
        /// Maximum results
        #[arg(long, default_value = "25")]
        limit: u32,
        /// Sort order: desc (newest first) or asc (oldest first)
        #[arg(long, default_value = "desc")]
        order: String,
        /// Filter messages created after this time (RFC-3339)
        #[arg(long)]
        after: Option<String>,
        /// Filter messages created before this time (RFC-3339)
        #[arg(long)]
        before: Option<String>,
        /// Only show messages from today (shortcut for --after with today's date)
        #[arg(long)]
        today: bool,
    },
    /// Get read state for a space (shows lastReadTime)
    ReadState {
        /// Space name (e.g. spaces/abc123)
        #[arg(long)]
        space: String,
    },
    /// Get read state for a thread
    ThreadReadState {
        /// Space name
        #[arg(long)]
        space: String,
        /// Thread name
        #[arg(long)]
        thread: String,
    },
    /// Show unread messages across all spaces
    Unread {
        /// Maximum messages per space
        #[arg(long, default_value = "10")]
        limit: u32,
        /// Filter by space type: SPACE, DIRECT_MESSAGE, GROUP_CHAT, or all
        #[arg(long, default_value = "SPACE")]
        r#type: String,
        /// Only check spaces active within this period (e.g. 1d, 7d, 30d, all). Default: 7d
        #[arg(long, default_value = "7d")]
        since: String,
        /// Include muted spaces (skipped by default)
        #[arg(long, default_value = "false")]
        include_muted: bool,
    },
    /// Mark a space as read (set lastReadTime to now or specific timestamp)
    MarkRead {
        /// Space name (e.g. spaces/abc123). Omit for --all mode
        #[arg(long)]
        space: Option<String>,
        /// Mark all currently unread spaces as read
        #[arg(long, default_value = "false")]
        all: bool,
        /// Specific timestamp to mark as read (RFC-3339). Default: now
        #[arg(long)]
        time: Option<String>,
        /// Space type filter for --all mode
        #[arg(long, default_value = "SPACE")]
        r#type: String,
        /// Since filter for --all mode
        #[arg(long, default_value = "7d")]
        since: String,
    },
    /// Send a message to a space
    Send {
        /// Space name (e.g. spaces/abc123)
        #[arg(long)]
        space: String,
        /// Message text
        #[arg(long)]
        text: String,
        /// Thread name for threaded reply
        #[arg(long)]
        thread: Option<String>,
    },
    /// Get a specific message
    Get {
        /// Message resource name
        name: String,
    },
}

#[derive(Debug, Subcommand)]
enum ContactsCommands {
    /// List contacts
    List {
        /// Maximum results
        #[arg(long, default_value = "100")]
        limit: u32,
    },
    /// Search contacts by query
    Search {
        /// Search query (name, email, phone)
        #[arg(long)]
        query: String,
        /// Maximum results
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// Get a specific contact
    Get {
        /// Resource name (e.g. people/c123456)
        name: String,
    },
    /// Create a new contact
    Create {
        /// Given (first) name
        #[arg(long)]
        given: String,
        /// Family (last) name
        #[arg(long)]
        family: Option<String>,
        /// Email address
        #[arg(long)]
        email: Option<String>,
        /// Phone number
        #[arg(long)]
        phone: Option<String>,
        /// Organization name
        #[arg(long)]
        org: Option<String>,
        /// Job title
        #[arg(long)]
        title: Option<String>,
    },
    /// Delete a contact
    Delete {
        /// Resource name (e.g. people/c123456)
        name: String,
    },
    /// List workspace directory people
    DirectoryList {
        /// Maximum results
        #[arg(long, default_value = "100")]
        limit: u32,
    },
    /// Search workspace directory
    DirectorySearch {
        /// Search query
        #[arg(long)]
        query: String,
        /// Maximum results
        #[arg(long, default_value = "50")]
        limit: u32,
    },
}

#[derive(Debug, Subcommand)]
enum GroupsCommands {
    /// List groups (by user email or by domain)
    List {
        /// User email to list groups for
        #[arg(long)]
        email: Option<String>,
        /// Domain to list all groups for (e.g. tisso.de)
        #[arg(long)]
        domain: Option<String>,
        /// Maximum results
        #[arg(long, default_value = "100")]
        limit: u32,
    },
    /// List members of a group
    Members {
        /// Group email address
        group_email: String,
        /// Maximum results
        #[arg(long, default_value = "100")]
        limit: u32,
    },
}

#[derive(Debug, Subcommand)]
enum AdminCommands {
    /// List users in the domain
    UsersList {
        /// Domain to list users for
        #[arg(long)]
        domain: Option<String>,
        /// Search query (Admin SDK query syntax)
        #[arg(long)]
        query: Option<String>,
        /// Maximum results
        #[arg(long, default_value = "100")]
        limit: u32,
    },
    /// Get a specific user
    UsersGet {
        /// User email or ID
        user_key: String,
    },
    /// Query Drive audit events from Admin Reports API
    ReportsDriveActivity {
        /// Event name to filter (view, edit, download, print, preview)
        #[arg(long, default_value = "view")]
        event_name: String,
        /// Start time (ISO 8601, e.g. 2025-08-01T00:00:00Z)
        #[arg(long)]
        start_time: Option<String>,
        /// End time (ISO 8601, e.g. 2026-02-17T00:00:00Z)
        #[arg(long)]
        end_time: Option<String>,
        /// Filter expression (e.g. doc_id==FILE_ID)
        #[arg(long)]
        filters: Option<String>,
        /// Max results per page (max 1000)
        #[arg(long, default_value = "1000")]
        max_results: u32,
    },
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    // Load config and create shared token manager
    let config = Config::load().with_env_overrides();
    let mut tm = TokenManager::new(config.clone());

    // --as flag: set impersonation subject (CLI overrides config/env)
    if let Some(ref email) = cli.impersonate {
        tm.set_subject(Some(email.clone()));
    }

    // Set service for per-service scope selection (used with --as)
    let service_name = match &cli.command {
        Commands::Gmail { .. } => "gmail",
        Commands::Drive { .. } => "drive",
        Commands::Calendar { .. } => "calendar",
        Commands::Docs { .. } => "docs",
        Commands::Sheets { .. } => "sheets",
        Commands::Slides { .. } => "slides",
        Commands::Tasks { .. } => "tasks",
        Commands::Chat { .. } => "chat",
        Commands::Contacts { .. } => "contacts",
        Commands::Groups { .. } => "groups",
        Commands::Admin { .. } => "admin",
        Commands::Batch { .. } => "drive",
        Commands::Auth { .. } => "drive",
    };
    tm.set_service(service_name);

    let token_manager = Arc::new(RwLock::new(tm));

    // Determine output format
    let format = OutputFormat::from_str(&cli.format).unwrap_or(OutputFormat::Json);

    // Parse fields for filtering
    let fields: Option<Vec<String>> = cli.fields.as_ref().map(|f| {
        f.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    });
    let quiet = cli.quiet;

    // Route commands
    match cli.command {
        Commands::Gmail { command } => {
            // Ensure we're authenticated before making API calls
            {
                let mut tm = token_manager.write().await;
                if let Err(e) = tm.ensure_authenticated().await {
                    eprintln!(r#"{{"status":"error","message":"Authentication failed: {}"}}"#, e);
                    std::process::exit(1);
                }
            }

            let client = ApiClient::gmail(token_manager.clone());
            let mut formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet);

            match command {
                GmailCommands::List { query, limit, label } => {
                    let params = workspace_cli::commands::gmail::list::ListParams {
                        query,
                        max_results: limit,
                        label_ids: label.map(|l| vec![l]),
                        page_token: None,
                    };
                    // Get access token for batch metadata request
                    let access_token = {
                        let tm = token_manager.read().await;
                        tm.get_access_token().await.map_err(|e| {
                            eprintln!(r#"{{"status":"error","message":"Failed to get token: {}"}}"#, e);
                            std::process::exit(1);
                        }).unwrap()
                    };
                    match workspace_cli::commands::gmail::list::list_messages_with_metadata(&client, params, &access_token).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                GmailCommands::Get { id, full } => {
                    if full {
                        // Return full message structure
                        match workspace_cli::commands::gmail::get::get_message(&client, &id, "full").await {
                            Ok(response) => {
                                if let Some(ref output_path) = cli.output {
                                    let file = std::fs::File::create(output_path)?;
                                    let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                    file_formatter.write(&response)?;
                                } else {
                                    formatter.write(&response)?;
                                }
                            }
                            Err(e) => {
                                eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        // Default: minimal format (essential headers + plain text body)
                        match workspace_cli::commands::gmail::get::get_message_minimal(&client, &id).await {
                            Ok(response) => {
                                if let Some(ref output_path) = cli.output {
                                    let file = std::fs::File::create(output_path)?;
                                    let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                    file_formatter.write(&response)?;
                                } else {
                                    formatter.write(&response)?;
                                }
                            }
                            Err(e) => {
                                eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                                std::process::exit(1);
                            }
                        }
                    }
                }
                GmailCommands::Send { to, subject, body, body_file } => {
                    let body_content = if let Some(file_path) = body_file {
                        std::fs::read_to_string(file_path)?
                    } else {
                        body.unwrap_or_default()
                    };

                    let params = workspace_cli::commands::gmail::send::ComposeParams {
                        to,
                        subject,
                        body: body_content,
                        from: None,
                        cc: None,
                        in_reply_to: None,
                        references: None,
                        thread_id: None,
                    };

                    match workspace_cli::commands::gmail::send::send_message(&client, params).await {
                        Ok(message) => {
                            // Return minimal response (success + id + threadId)
                            let response = workspace_cli::commands::gmail::types::SendResponse::from_message(&message);
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                GmailCommands::Draft { to, subject, body } => {
                    let body_content = body.unwrap_or_default();

                    let params = workspace_cli::commands::gmail::send::ComposeParams {
                        to,
                        subject,
                        body: body_content,
                        from: None,
                        cc: None,
                        in_reply_to: None,
                        references: None,
                        thread_id: None,
                    };

                    match workspace_cli::commands::gmail::send::create_draft(&client, params).await {
                        Ok(draft) => {
                            // Return minimal response (success + draft id + message info)
                            let response = workspace_cli::commands::gmail::types::DraftResponse {
                                success: true,
                                id: draft["id"].as_str().unwrap_or("").to_string(),
                                message_id: draft["message"]["id"].as_str().map(String::from),
                                thread_id: draft["message"]["threadId"].as_str().map(String::from),
                            };
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                GmailCommands::Delete { id } => {
                    match workspace_cli::commands::gmail::delete::delete_message(&client, &id).await {
                        Ok(()) => {
                            if !quiet {
                                println!(r#"{{"status":"success","message":"Message deleted permanently"}}"#);
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                GmailCommands::Trash { id } => {
                    match workspace_cli::commands::gmail::trash::trash_message(&client, &id).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                GmailCommands::Untrash { id } => {
                    match workspace_cli::commands::gmail::trash::untrash_message(&client, &id).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                GmailCommands::Labels => {
                    match workspace_cli::commands::gmail::labels::list_labels(&client).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                GmailCommands::Modify { id, add_labels, remove_labels, mark_read, mark_unread, star, unstar, archive } => {
                    // Build label modifications
                    let mut add: Vec<String> = add_labels
                        .map(|s| s.split(',').map(|l| l.trim().to_string()).collect())
                        .unwrap_or_default();
                    let mut remove: Vec<String> = remove_labels
                        .map(|s| s.split(',').map(|l| l.trim().to_string()).collect())
                        .unwrap_or_default();

                    // Handle convenience flags
                    if mark_read {
                        remove.push("UNREAD".to_string());
                    }
                    if mark_unread {
                        add.push("UNREAD".to_string());
                    }
                    if star {
                        add.push("STARRED".to_string());
                    }
                    if unstar {
                        remove.push("STARRED".to_string());
                    }
                    if archive {
                        remove.push("INBOX".to_string());
                    }

                    match workspace_cli::commands::gmail::labels::modify_labels(&client, &id, add, remove).await {
                        Ok(response) => {
                            // Return minimal response (success + id + labels) to reduce token usage
                            let minimal = workspace_cli::commands::gmail::types::ModifyResponse::from_message(&response);
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&minimal)?;
                            } else {
                                formatter.write(&minimal)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                GmailCommands::Reply { id, body, body_file, all } => {
                    // Fetch original message to get headers
                    let original = match workspace_cli::commands::gmail::get::get_message(&client, &id, "metadata").await {
                        Ok(msg) => msg,
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"Failed to fetch original message: {}"}}"#, e);
                            std::process::exit(1);
                        }
                    };

                    // Extract reply metadata
                    let metadata = match workspace_cli::commands::gmail::send::extract_reply_metadata(&original) {
                        Some(m) => m,
                        None => {
                            eprintln!(r#"{{"status":"error","message":"Could not extract reply metadata from message (missing Message-ID or From header)"}}"#);
                            std::process::exit(1);
                        }
                    };

                    // Get reply body
                    let body_content = if let Some(file_path) = body_file {
                        match std::fs::read_to_string(&file_path) {
                            Ok(content) => content,
                            Err(e) => {
                                eprintln!(r#"{{"status":"error","message":"Failed to read body file: {}"}}"#, e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        body.unwrap_or_default()
                    };

                    // Build ComposeParams with reply fields
                    let params = workspace_cli::commands::gmail::send::ComposeParams {
                        to: metadata.to,
                        subject: metadata.subject,
                        body: body_content,
                        from: None,
                        cc: if all { metadata.cc } else { None },
                        in_reply_to: Some(metadata.in_reply_to),
                        references: Some(metadata.references),
                        thread_id: Some(metadata.thread_id),
                    };

                    match workspace_cli::commands::gmail::send::send_message(&client, params).await {
                        Ok(message) => {
                            // Return minimal response (success + id + threadId)
                            let response = workspace_cli::commands::gmail::types::SendResponse::from_message(&message);
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                GmailCommands::ReplyDraft { id, body, all } => {
                    // Fetch original message to get headers
                    let original = match workspace_cli::commands::gmail::get::get_message(&client, &id, "metadata").await {
                        Ok(msg) => msg,
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"Failed to fetch original message: {}"}}"#, e);
                            std::process::exit(1);
                        }
                    };

                    // Extract reply metadata
                    let metadata = match workspace_cli::commands::gmail::send::extract_reply_metadata(&original) {
                        Some(m) => m,
                        None => {
                            eprintln!(r#"{{"status":"error","message":"Could not extract reply metadata from message (missing Message-ID or From header)"}}"#);
                            std::process::exit(1);
                        }
                    };

                    // Build ComposeParams with reply fields
                    let params = workspace_cli::commands::gmail::send::ComposeParams {
                        to: metadata.to,
                        subject: metadata.subject,
                        body: body.unwrap_or_default(),
                        from: None,
                        cc: if all { metadata.cc } else { None },
                        in_reply_to: Some(metadata.in_reply_to),
                        references: Some(metadata.references),
                        thread_id: Some(metadata.thread_id),
                    };

                    match workspace_cli::commands::gmail::send::create_draft(&client, params).await {
                        Ok(draft) => {
                            // Return minimal response (success + draft id + message info)
                            let response = workspace_cli::commands::gmail::types::DraftResponse {
                                success: true,
                                id: draft["id"].as_str().unwrap_or("").to_string(),
                                message_id: draft["message"]["id"].as_str().map(String::from),
                                thread_id: draft["message"]["threadId"].as_str().map(String::from),
                            };
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::Drive { command } => {
            // Ensure we're authenticated before making API calls
            {
                let mut tm = token_manager.write().await;
                if let Err(e) = tm.ensure_authenticated().await {
                    eprintln!(r#"{{"status":"error","message":"Authentication failed: {}"}}"#, e);
                    std::process::exit(1);
                }
            }

            let client = ApiClient::drive(token_manager.clone());
            let mut formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet);

            match command {
                DriveCommands::List { query, limit, parent, corpora, include_permissions, page_token } => {
                    // Build query with optional parent filter
                    let final_query = match (query, parent) {
                        (Some(q), Some(p)) => Some(format!("'{}' in parents and ({})", p, q)),
                        (Some(q), None) => Some(q),
                        (None, Some(p)) => Some(format!("'{}' in parents", p)),
                        (None, None) => None,
                    };

                    let params = workspace_cli::commands::drive::list::ListParams {
                        query: final_query,
                        max_results: limit,
                        page_token,
                        fields: fields.as_ref().map(|f| f.join(",")),
                        order_by: None,
                        corpora,
                        include_permissions,
                    };
                    match workspace_cli::commands::drive::list::list_files(&client, params).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DriveCommands::Upload { file, parent, name } => {
                    // Get access token for direct upload
                    let token = {
                        let tm = token_manager.read().await;
                        tm.get_access_token().await.map_err(|e| {
                            eprintln!(r#"{{"status":"error","message":"Failed to get token: {}"}}"#, e);
                            std::process::exit(1);
                        }).unwrap()
                    };

                    let params = workspace_cli::commands::drive::upload::UploadParams {
                        file_path: file,
                        name,
                        parent_id: parent,
                        mime_type: None,
                    };

                    match workspace_cli::commands::drive::upload::upload_file(&token, params).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DriveCommands::Download { id, output } => {
                    // Get access token for direct download
                    let token = {
                        let tm = token_manager.read().await;
                        tm.get_access_token().await.map_err(|e| {
                            eprintln!(r#"{{"status":"error","message":"Failed to get token: {}"}}"#, e);
                            std::process::exit(1);
                        }).unwrap()
                    };

                    let output_path = output
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|| std::path::PathBuf::from(&id));

                    match workspace_cli::commands::drive::download::download_file(&token, &id, &output_path).await {
                        Ok(bytes) => {
                            if !quiet {
                                println!(r#"{{"status":"success","file":"{}","bytes":{}}}"#, output_path.display(), bytes);
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DriveCommands::Get { id } => {
                    match workspace_cli::commands::drive::list::get_file(&client, &id, None).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DriveCommands::Delete { id } => {
                    match workspace_cli::commands::drive::delete::delete_file(&client, &id).await {
                        Ok(()) => {
                            if !quiet {
                                println!(r#"{{"status":"success","message":"File deleted permanently"}}"#);
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DriveCommands::Trash { id } => {
                    match workspace_cli::commands::drive::delete::trash_file(&client, &id).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DriveCommands::Untrash { id } => {
                    match workspace_cli::commands::drive::delete::untrash_file(&client, &id).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DriveCommands::Mkdir { name, parent } => {
                    match workspace_cli::commands::drive::mkdir::create_folder(&client, &name, parent.as_deref()).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DriveCommands::Move { id, to } => {
                    match workspace_cli::commands::drive::operations::move_file(&client, &id, &to, true).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DriveCommands::Copy { id, name, parent } => {
                    match workspace_cli::commands::drive::operations::copy_file(&client, &id, name.as_deref(), parent.as_deref()).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DriveCommands::Rename { id, name } => {
                    match workspace_cli::commands::drive::operations::rename_file(&client, &id, &name).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DriveCommands::Share { id, email, anyone, role } => {
                    let result = if anyone {
                        workspace_cli::commands::drive::share::share_with_anyone(&client, &id, &role).await
                    } else if let Some(email) = email {
                        workspace_cli::commands::drive::share::share_with_user(&client, &id, &email, &role).await
                    } else {
                        eprintln!(r#"{{"status":"error","message":"Must specify --email or --anyone"}}"#);
                        std::process::exit(1);
                    };

                    match result {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DriveCommands::Permissions { id } => {
                    match workspace_cli::commands::drive::share::list_permissions(&client, &id).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DriveCommands::Unshare { id, permission_id } => {
                    match workspace_cli::commands::drive::share::remove_permission(&client, &id, &permission_id).await {
                        Ok(()) => {
                            if !quiet {
                                println!(r#"{{"status":"success","message":"Permission removed"}}"#);
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DriveCommands::DrivesList { limit } => {
                    match workspace_cli::commands::drive::list::list_drives(&client, limit, None).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DriveCommands::Tree { folder_id, max_depth, concurrency, include_permissions } => {
                    match workspace_cli::commands::drive::tree::crawl_tree(&client, &folder_id, max_depth, concurrency, include_permissions).await {
                        Ok(result) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&result)?;
                            } else {
                                formatter.write(&result)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::Calendar { command } => {
            // Ensure we're authenticated before making API calls
            {
                let mut tm = token_manager.write().await;
                if let Err(e) = tm.ensure_authenticated().await {
                    eprintln!(r#"{{"status":"error","message":"Authentication failed: {}"}}"#, e);
                    std::process::exit(1);
                }
            }

            let client = ApiClient::calendar(token_manager.clone());
            let mut formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet);

            match command {
                CalendarCommands::List { calendar, time_min, time_max, limit, sync_token, full } => {
                    let params = workspace_cli::commands::calendar::list::ListEventsParams {
                        calendar_id: calendar,
                        time_min,
                        time_max,
                        max_results: limit,
                        single_events: true,
                        order_by: Some("startTime".to_string()),
                        sync_token,
                        page_token: None,
                    };
                    match workspace_cli::commands::calendar::list::list_events(&client, params).await {
                        Ok(response) => {
                            if full {
                                // Return full event data
                                if let Some(ref output_path) = cli.output {
                                    let file = std::fs::File::create(output_path)?;
                                    let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                    file_formatter.write(&response)?;
                                } else {
                                    formatter.write(&response)?;
                                }
                            } else {
                                // Default: minimal event data (id, summary, start, end, status)
                                let minimal = workspace_cli::commands::calendar::types::MinimalEventList::from_event_list(&response);
                                if let Some(ref output_path) = cli.output {
                                    let file = std::fs::File::create(output_path)?;
                                    let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                    file_formatter.write(&minimal)?;
                                } else {
                                    formatter.write(&minimal)?;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                CalendarCommands::Create { summary, start, end, description, calendar } => {
                    let params = workspace_cli::commands::calendar::create::CreateEventParams {
                        calendar_id: calendar,
                        summary,
                        start,
                        end,
                        description,
                        location: None,
                        attendees: None,
                        time_zone: None,
                    };

                    match workspace_cli::commands::calendar::create::create_event(&client, params).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                CalendarCommands::Update { id, summary, start, end, calendar } => {
                    let params = workspace_cli::commands::calendar::update::UpdateEventParams {
                        calendar_id: calendar,
                        event_id: id,
                        summary,
                        description: None,
                        location: None,
                        start,
                        end,
                        time_zone: None,
                    };

                    match workspace_cli::commands::calendar::update::update_event(&client, params).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                CalendarCommands::Delete { id, calendar } => {
                    match workspace_cli::commands::calendar::delete::delete_event(&client, &calendar, &id).await {
                        Ok(()) => {
                            if !quiet {
                                println!(r#"{{"status":"success","message":"Event deleted"}}"#);
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::Docs { command } => {
            // Ensure we're authenticated before making API calls
            {
                let mut tm = token_manager.write().await;
                if let Err(e) = tm.ensure_authenticated().await {
                    eprintln!(r#"{{"status":"error","message":"Authentication failed: {}"}}"#, e);
                    std::process::exit(1);
                }
            }

            let client = ApiClient::docs(token_manager.clone());
            let mut formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet);

            match command {
                DocsCommands::Get { id, markdown, text } => {
                    match workspace_cli::commands::docs::get::get_document(&client, &id).await {
                        Ok(doc) => {
                            if text {
                                // Plain text output (most token-efficient)
                                let txt = workspace_cli::commands::docs::get::document_to_text(&doc);
                                println!("{}", txt);
                            } else if markdown {
                                let md = workspace_cli::commands::docs::get::document_to_markdown(&doc);
                                println!("{}", md);
                            } else if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&doc)?;
                            } else {
                                formatter.write(&doc)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DocsCommands::Append { id, text } => {
                    match workspace_cli::commands::docs::update::append_text(&client, &id, &text).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DocsCommands::Create { title } => {
                    match workspace_cli::commands::docs::create::create_document(&client, &title).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                DocsCommands::Replace { id, find, replace_with, match_case } => {
                    match workspace_cli::commands::docs::update::replace_text(&client, &id, &find, &replace_with, match_case).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::Sheets { command } => {
            // Ensure we're authenticated before making API calls
            {
                let mut tm = token_manager.write().await;
                if let Err(e) = tm.ensure_authenticated().await {
                    eprintln!(r#"{{"status":"error","message":"Authentication failed: {}"}}"#, e);
                    std::process::exit(1);
                }
            }

            let client = ApiClient::sheets(token_manager.clone());
            let mut formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet);

            match command {
                SheetsCommands::Get { id, range, full } => {
                    match workspace_cli::commands::sheets::get::get_values(&client, &id, &range).await {
                        Ok(response) => {
                            if format == OutputFormat::Csv {
                                let csv = workspace_cli::commands::sheets::get::values_to_csv(&response);
                                if let Some(ref output_path) = cli.output {
                                    std::fs::write(output_path, &csv)?;
                                } else if !quiet {
                                    print!("{}", csv);
                                }
                            } else if full {
                                // Return full ValueRange with metadata
                                if let Some(ref output_path) = cli.output {
                                    let file = std::fs::File::create(output_path)?;
                                    let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                    file_formatter.write(&response)?;
                                } else {
                                    formatter.write(&response)?;
                                }
                            } else {
                                // Default: return just the values array (minimal, token-efficient)
                                if let Some(ref output_path) = cli.output {
                                    let file = std::fs::File::create(output_path)?;
                                    let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                    file_formatter.write(&response.values)?;
                                } else {
                                    formatter.write(&response.values)?;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                SheetsCommands::Update { id, range, values } => {
                    let parsed_values = workspace_cli::commands::sheets::update::parse_values_json(&values)?;
                    let params = workspace_cli::commands::sheets::update::UpdateParams {
                        spreadsheet_id: id,
                        range,
                        values: parsed_values,
                        value_input_option: workspace_cli::commands::sheets::update::ValueInputOption::UserEntered,
                    };

                    match workspace_cli::commands::sheets::update::update_values(&client, params).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                SheetsCommands::Append { id, range, values } => {
                    let parsed_values = workspace_cli::commands::sheets::update::parse_values_json(&values)?;

                    match workspace_cli::commands::sheets::update::append_values(
                        &client,
                        &id,
                        &range,
                        parsed_values,
                        workspace_cli::commands::sheets::update::ValueInputOption::UserEntered,
                    ).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                SheetsCommands::Create { title, sheets: sheet_names } => {
                    let result = if let Some(ref names) = sheet_names {
                        workspace_cli::commands::sheets::create::create_spreadsheet_with_sheets(&client, &title, names).await
                    } else {
                        workspace_cli::commands::sheets::create::create_spreadsheet(&client, &title).await
                    };
                    match result {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                SheetsCommands::Clear { id, range } => {
                    match workspace_cli::commands::sheets::update::clear_values(&client, &id, &range).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                SheetsCommands::AddSheet { id, title } => {
                    match workspace_cli::commands::sheets::manage::add_sheet(&client, &id, &title).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                SheetsCommands::DeleteSheet { id, sheet_id } => {
                    match workspace_cli::commands::sheets::manage::delete_sheet(&client, &id, sheet_id).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                SheetsCommands::ListSheets { id } => {
                    match workspace_cli::commands::sheets::get::get_spreadsheet(&client, &id).await {
                        Ok(spreadsheet) => {
                            let tab_ids: std::collections::HashMap<String, i64> = spreadsheet
                                .sheets
                                .into_iter()
                                .map(|s| (s.properties.title, s.properties.sheet_id))
                                .collect();
                            formatter.write(&tab_ids)?;
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::Slides { command } => {
            // Ensure we're authenticated before making API calls
            {
                let mut tm = token_manager.write().await;
                if let Err(e) = tm.ensure_authenticated().await {
                    eprintln!(r#"{{"status":"error","message":"Authentication failed: {}"}}"#, e);
                    std::process::exit(1);
                }
            }

            let client = ApiClient::slides(token_manager.clone());
            let mut formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet);

            match command {
                SlidesCommands::Get { id, full } => {
                    match workspace_cli::commands::slides::get::get_presentation(&client, &id).await {
                        Ok(presentation) => {
                            if full {
                                // Return full presentation structure
                                if let Some(ref output_path) = cli.output {
                                    let file = std::fs::File::create(output_path)?;
                                    let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                    file_formatter.write(&presentation)?;
                                } else {
                                    formatter.write(&presentation)?;
                                }
                            } else {
                                // Default: text extraction (minimal, token-efficient)
                                let text = workspace_cli::commands::slides::get::extract_all_text(&presentation);
                                if let Some(ref output_path) = cli.output {
                                    std::fs::write(output_path, &text)?;
                                } else {
                                    println!("{}", text);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                SlidesCommands::Page { id, page, full } => {
                    // Get the presentation first, then extract the specific slide
                    match workspace_cli::commands::slides::get::get_presentation(&client, &id).await {
                        Ok(presentation) => {
                            let page_index = page as usize;
                            if page_index >= presentation.slides.len() {
                                eprintln!(r#"{{"status":"error","message":"Page {} not found. Presentation has {} slides."}}"#, page, presentation.slides.len());
                                std::process::exit(1);
                            }

                            let slide = &presentation.slides[page_index];
                            if full {
                                // Return full page structure
                                if let Some(ref output_path) = cli.output {
                                    let file = std::fs::File::create(output_path)?;
                                    let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                    file_formatter.write(slide)?;
                                } else {
                                    formatter.write(slide)?;
                                }
                            } else {
                                // Default: text extraction (minimal, token-efficient)
                                let text = workspace_cli::commands::slides::get::extract_page_text(slide);
                                if let Some(ref output_path) = cli.output {
                                    std::fs::write(output_path, &text)?;
                                } else {
                                    println!("{}", text);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::Tasks { command } => {
            // Ensure we're authenticated before making API calls
            {
                let mut tm = token_manager.write().await;
                if let Err(e) = tm.ensure_authenticated().await {
                    eprintln!(r#"{{"status":"error","message":"Authentication failed: {}"}}"#, e);
                    std::process::exit(1);
                }
            }

            let client = ApiClient::tasks(token_manager.clone());
            let mut formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet);

            match command {
                TasksCommands::Lists => {
                    match workspace_cli::commands::tasks::list::list_task_lists(&client).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                TasksCommands::List { list, limit, show_completed, full } => {
                    let params = workspace_cli::commands::tasks::list::ListTasksParams {
                        task_list_id: list,
                        max_results: limit.min(100),  // API max is 100
                        show_completed,
                        show_hidden: false,
                        page_token: None,
                    };
                    match workspace_cli::commands::tasks::list::list_tasks(&client, params).await {
                        Ok(response) => {
                            if full {
                                // Return full task data
                                if let Some(ref output_path) = cli.output {
                                    let file = std::fs::File::create(output_path)?;
                                    let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                    file_formatter.write(&response)?;
                                } else {
                                    formatter.write(&response)?;
                                }
                            } else {
                                // Default: minimal task data (id, title, status, due, notes, completed)
                                let minimal = workspace_cli::commands::tasks::types::MinimalTasks::from_tasks(&response);
                                if let Some(ref output_path) = cli.output {
                                    let file = std::fs::File::create(output_path)?;
                                    let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                    file_formatter.write(&minimal)?;
                                } else {
                                    formatter.write(&minimal)?;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                TasksCommands::Create { title, list, due, notes } => {
                    let params = workspace_cli::commands::tasks::create::CreateTaskParams {
                        task_list_id: list,
                        title,
                        notes,
                        due,
                        parent: None,
                    };
                    match workspace_cli::commands::tasks::create::create_task(&client, params).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                TasksCommands::Update { id, list, title, complete } => {
                    use workspace_cli::commands::tasks::update::TaskStatus;
                    let params = workspace_cli::commands::tasks::update::UpdateTaskParams {
                        task_list_id: list,
                        task_id: id,
                        title,
                        status: if complete { Some(TaskStatus::Completed) } else { None },
                        notes: None,
                        due: None,
                    };
                    match workspace_cli::commands::tasks::update::update_task(&client, params).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&response)?;
                            } else {
                                formatter.write(&response)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                TasksCommands::Delete { id, list } => {
                    match workspace_cli::commands::tasks::update::delete_task(&client, &list, &id).await {
                        Ok(_) => {
                            if !quiet {
                                println!(r#"{{"status":"success","message":"Task deleted"}}"#);
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::Auth { command } => {
            match command {
                AuthCommands::Login { credentials } => {
                    let creds_path = credentials.map(std::path::PathBuf::from);
                    let mut tm = token_manager.write().await;
                    match tm.login_interactive(creds_path.clone()).await {
                        Ok(()) => {
                            // Save credentials path to config for future use
                            if let Some(path) = creds_path {
                                // Canonicalize to absolute path
                                let abs_path = std::fs::canonicalize(&path).unwrap_or(path);
                                let mut config = workspace_cli::config::Config::load();
                                config.auth.credentials_path = Some(abs_path);
                                if let Err(e) = config.save() {
                                    eprintln!(r#"{{"status":"warning","message":"Login succeeded but failed to save config: {}"}}"#, e);
                                }
                            }
                            if !quiet {
                                println!(r#"{{"status":"success","message":"Login successful"}}"#);
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                AuthCommands::Logout => {
                    let mut tm = token_manager.write().await;
                    match tm.logout() {
                        Ok(()) => {
                            if !quiet {
                                println!(r#"{{"status":"success","message":"Logged out"}}"#);
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                AuthCommands::Status => {
                    let tm = token_manager.read().await;
                    let status = tm.status();
                    if !quiet {
                        println!("{}", serde_json::to_string_pretty(&status).unwrap());
                    }
                }
            }
        }
        Commands::Batch { command } => {
            // Ensure we're authenticated before making API calls
            let access_token = {
                let mut tm = token_manager.write().await;
                if let Err(e) = tm.ensure_authenticated().await {
                    eprintln!(r#"{{"status":"error","message":"Authentication failed: {}"}}"#, e);
                    std::process::exit(1);
                }
                match tm.get_access_token().await {
                    Ok(token) => token,
                    Err(e) => {
                        eprintln!(r#"{{"status":"error","message":"Failed to get access token: {}"}}"#, e);
                        std::process::exit(1);
                    }
                }
            };

            let mut formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet);

            // Determine service and get JSON input
            let (service, requests_json, file_path) = match command {
                BatchCommands::Gmail { requests, file } => ("gmail", requests, file),
                BatchCommands::Drive { requests, file } => ("drive", requests, file),
                BatchCommands::Calendar { requests, file } => ("calendar", requests, file),
            };

            // Parse input JSON from argument, file, or stdin
            let json_str = if let Some(path) = file_path {
                match std::fs::read_to_string(&path) {
                    Ok(content) => content,
                    Err(e) => {
                        eprintln!(r#"{{"status":"error","message":"Failed to read file '{}': {}"}}"#, path, e);
                        std::process::exit(1);
                    }
                }
            } else if let Some(json) = requests_json {
                json
            } else {
                // Read from stdin for piping
                use std::io::Read;
                let mut buffer = String::new();
                if let Err(e) = std::io::stdin().read_to_string(&mut buffer) {
                    eprintln!(r#"{{"status":"error","message":"Failed to read from stdin: {}"}}"#, e);
                    std::process::exit(1);
                }
                buffer
            };

            // Parse JSON array of requests
            let inputs: Vec<workspace_cli::commands::batch::BatchRequestInput> = match serde_json::from_str(&json_str) {
                Ok(inputs) => inputs,
                Err(e) => {
                    eprintln!(r#"{{"status":"error","message":"Invalid JSON input: {}"}}"#, e);
                    std::process::exit(1);
                }
            };

            // Execute batch
            match workspace_cli::commands::batch::execute_batch(service, inputs, &access_token).await {
                Ok(output) => {
                    if let Some(ref output_path) = cli.output {
                        let file = std::fs::File::create(output_path)?;
                        let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                        file_formatter.write(&output)?;
                    } else {
                        formatter.write(&output)?;
                    }
                }
                Err(e) => {
                    eprintln!(r#"{{"status":"error","message":"Batch request failed: {}"}}"#, e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Chat { command } => {
            {
                let mut tm = token_manager.write().await;
                if let Err(e) = tm.ensure_authenticated().await {
                    eprintln!(r#"{{"status":"error","message":"Authentication failed: {}"}}"#, e);
                    std::process::exit(1);
                }
            }

            let client = ApiClient::chat(token_manager.clone());
            let mut formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet);

            match command {
                ChatCommands::SpacesList { limit, space_type } => {
                    let filter = space_type.map(|t| format!("spaceType = \"{}\"", t.to_uppercase()));
                    let params = workspace_cli::commands::chat::spaces::ListSpacesParams {
                        page_size: limit, page_token: None, filter,
                    };
                    match workspace_cli::commands::chat::spaces::list_spaces(&client, params).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&response)?;
                            } else { formatter.write(&response)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                ChatCommands::SpacesFind { name } => {
                    match workspace_cli::commands::chat::spaces::find_space_by_name(&client, &name).await {
                        Ok(spaces) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&spaces)?;
                            } else { formatter.write(&spaces)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                ChatCommands::FindDm { email } => {
                    match workspace_cli::commands::chat::spaces::find_direct_message(&client, &email).await {
                        Ok(space) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&space)?;
                            } else { formatter.write(&space)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                ChatCommands::SpacesCreate { name, member } => {
                    match workspace_cli::commands::chat::spaces::create_space(&client, &name, &member).await {
                        Ok(space) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&space)?;
                            } else { formatter.write(&space)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                ChatCommands::MessagesList { space, limit, order, after, before, today } => {
                    let order_by = format!("createTime {}", if order.to_lowercase() == "asc" { "ASC" } else { "DESC" });
                    let mut filter_parts: Vec<String> = Vec::new();
                    if today {
                        let today_start = chrono::Local::now().format("%Y-%m-%dT00:00:00Z").to_string();
                        filter_parts.push(format!("createTime > \"{}\"", today_start));
                    } else if let Some(ref t) = after {
                        filter_parts.push(format!("createTime > \"{}\"", t));
                    }
                    if let Some(ref t) = before {
                        filter_parts.push(format!("createTime < \"{}\"", t));
                    }
                    let filter = if filter_parts.is_empty() { None } else { Some(filter_parts.join(" AND ")) };
                    let params = workspace_cli::commands::chat::messages::ListMessagesParams {
                        space_name: space,
                        page_size: limit,
                        page_token: None,
                        order_by: Some(order_by),
                        filter,
                    };
                    match workspace_cli::commands::chat::messages::list_messages(&client, params).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&response)?;
                            } else { formatter.write(&response)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                ChatCommands::ReadState { space } => {
                    match workspace_cli::commands::chat::read_state::get_space_read_state(&client, &space).await {
                        Ok(state) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&state)?;
                            } else { formatter.write(&state)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                ChatCommands::ThreadReadState { space, thread } => {
                    match workspace_cli::commands::chat::read_state::get_thread_read_state(&client, &space, &thread).await {
                        Ok(state) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&state)?;
                            } else { formatter.write(&state)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                ChatCommands::Unread { limit, r#type, since, include_muted } => {
                    let type_filter = if r#type.to_lowercase() == "all" { None } else { Some(r#type.as_str()) };
                    match workspace_cli::commands::chat::read_state::get_unread_messages(&client, limit, type_filter, &since, include_muted).await {
                        Ok(result) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&result)?;
                            } else { formatter.write(&result)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                ChatCommands::MarkRead { space, all, time, r#type, since } => {
                    let now = chrono::Utc::now().to_rfc3339();
                    let read_time = time.as_deref().unwrap_or(&now);

                    if all {
                        // Bulk mode: find all unread spaces, then mark them read
                        let type_filter = if r#type.to_lowercase() == "all" { None } else { Some(r#type.as_str()) };
                        match workspace_cli::commands::chat::read_state::get_unread_messages(&client, 1, type_filter, &since, false).await {
                            Ok(result) => {
                                let mut marked = 0usize;
                                for us in &result.spaces {
                                    if let Some(ref sn) = us.space_name {
                                        // Use the latest message time or now
                                        let mark_time = us.messages.first()
                                            .and_then(|m| m.create_time.as_deref())
                                            .unwrap_or(read_time);
                                        if let Ok(_) = workspace_cli::commands::chat::read_state::update_space_read_state(&client, sn, mark_time).await {
                                            marked += 1;
                                            eprintln!("Marked read: {} ({})", us.display_name.as_deref().unwrap_or(sn), mark_time);
                                        }
                                    }
                                }
                                let summary = serde_json::json!({ "status": "ok", "spacesMarkedRead": marked });
                                if let Some(ref output_path) = cli.output {
                                    let file = std::fs::File::create(output_path)?;
                                    let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                    ff.write(&summary)?;
                                } else { formatter.write(&summary)?; }
                            }
                            Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                        }
                    } else if let Some(ref space_name) = space {
                        match workspace_cli::commands::chat::read_state::update_space_read_state(&client, space_name, read_time).await {
                            Ok(state) => {
                                if let Some(ref output_path) = cli.output {
                                    let file = std::fs::File::create(output_path)?;
                                    let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                    ff.write(&state)?;
                                } else { formatter.write(&state)?; }
                            }
                            Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                        }
                    } else {
                        eprintln!(r#"{{"status":"error","message":"Provide --space or --all"}}"#);
                        std::process::exit(1);
                    }
                }
                ChatCommands::Send { space, text, thread } => {
                    match workspace_cli::commands::chat::messages::send_message(&client, &space, &text, thread.as_deref()).await {
                        Ok(msg) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&msg)?;
                            } else { formatter.write(&msg)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                ChatCommands::Get { name } => {
                    match workspace_cli::commands::chat::messages::get_message(&client, &name).await {
                        Ok(msg) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&msg)?;
                            } else { formatter.write(&msg)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
            }
        }

        Commands::Contacts { command } => {
            {
                let mut tm = token_manager.write().await;
                if let Err(e) = tm.ensure_authenticated().await {
                    eprintln!(r#"{{"status":"error","message":"Authentication failed: {}"}}"#, e);
                    std::process::exit(1);
                }
            }

            let client = ApiClient::contacts(token_manager.clone());
            let mut formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet);

            match command {
                ContactsCommands::List { limit } => {
                    let params = workspace_cli::commands::contacts::list::ListContactsParams {
                        page_size: limit, page_token: None,
                    };
                    match workspace_cli::commands::contacts::list::list_contacts(&client, params).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&response)?;
                            } else { formatter.write(&response)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                ContactsCommands::Search { query, limit } => {
                    match workspace_cli::commands::contacts::search::search_contacts(&client, &query, limit).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&response)?;
                            } else { formatter.write(&response)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                ContactsCommands::Get { name } => {
                    match workspace_cli::commands::contacts::list::get_contact(&client, &name).await {
                        Ok(person) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&person)?;
                            } else { formatter.write(&person)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                ContactsCommands::Create { given, family, email, phone, org, title } => {
                    let params = workspace_cli::commands::contacts::create::CreateContactParams {
                        given_name: given, family_name: family, email, phone,
                        org_name: org, org_title: title,
                    };
                    match workspace_cli::commands::contacts::create::create_contact(&client, params).await {
                        Ok(person) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&person)?;
                            } else { formatter.write(&person)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                ContactsCommands::Delete { name } => {
                    match workspace_cli::commands::contacts::create::delete_contact(&client, &name).await {
                        Ok(()) => {
                            if !quiet {
                                eprintln!(r#"{{"status":"success","message":"Contact deleted"}}"#);
                            }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                ContactsCommands::DirectoryList { limit } => {
                    let params = workspace_cli::commands::contacts::search::DirectoryListParams {
                        page_size: limit, page_token: None,
                    };
                    match workspace_cli::commands::contacts::search::list_directory(&client, params).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&response)?;
                            } else { formatter.write(&response)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                ContactsCommands::DirectorySearch { query, limit } => {
                    match workspace_cli::commands::contacts::search::search_directory(&client, &query, limit, None).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&response)?;
                            } else { formatter.write(&response)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
            }
        }

        Commands::Groups { command } => {
            {
                let mut tm = token_manager.write().await;
                if let Err(e) = tm.ensure_authenticated().await {
                    eprintln!(r#"{{"status":"error","message":"Authentication failed: {}"}}"#, e);
                    std::process::exit(1);
                }
            }

            let client = ApiClient::groups(token_manager.clone());
            let mut formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet);

            match command {
                GroupsCommands::List { email, domain, limit } => {
                    if email.is_none() && domain.is_none() {
                        eprintln!(r#"{{"status":"error","message":"Either --email or --domain is required"}}"#);
                        std::process::exit(1);
                    }
                    let admin_client = ApiClient::admin(token_manager.clone());
                    let params = workspace_cli::commands::groups::list::ListGroupsParams {
                        email, domain, page_size: limit, page_token: None,
                    };
                    match workspace_cli::commands::groups::list::list_groups(&admin_client, params).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&response)?;
                            } else { formatter.write(&response)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                GroupsCommands::Members { group_email, limit } => {
                    match workspace_cli::commands::groups::members::list_members_by_email(&client, &group_email, limit).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&response)?;
                            } else { formatter.write(&response)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
            }
        }
        Commands::Admin { command } => {
            {
                let mut tm = token_manager.write().await;
                if let Err(e) = tm.ensure_authenticated().await {
                    eprintln!(r#"{{"status":"error","message":"Authentication failed: {}"}}"#, e);
                    std::process::exit(1);
                }
            }

            let client = ApiClient::admin(token_manager.clone());
            let mut formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet);

            match command {
                AdminCommands::UsersList { domain, query, limit } => {
                    let params = workspace_cli::commands::admin::users::ListUsersParams {
                        domain, query, max_results: limit, page_token: None,
                    };
                    match workspace_cli::commands::admin::users::list_users(&client, params).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&response)?;
                            } else { formatter.write(&response)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                AdminCommands::UsersGet { user_key } => {
                    match workspace_cli::commands::admin::users::get_user(&client, &user_key).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&response)?;
                            } else { formatter.write(&response)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
                AdminCommands::ReportsDriveActivity { event_name, start_time, end_time, filters, max_results } => {
                    let reports_client = ApiClient::admin_reports(token_manager.clone());
                    let params = workspace_cli::commands::admin::reports::DriveActivityParams {
                        event_name, start_time, end_time, filters, max_results,
                    };
                    match workspace_cli::commands::admin::reports::list_drive_activity(&reports_client, params).await {
                        Ok(events) => {
                            eprintln!("Total events: {}", events.len());
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut ff = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                ff.write(&events)?;
                            } else { formatter.write(&events)?; }
                        }
                        Err(e) => { eprintln!(r#"{{"status":"error","message":"{}"}}"#, e); std::process::exit(1); }
                    }
                }
            }
        }
    }

    Ok(())
}
