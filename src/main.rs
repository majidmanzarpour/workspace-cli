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
    All commands output JSON by default. Use --format to change output format.\n\
    Use --fields to limit response fields for token efficiency.")]
#[command(author, version)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format: json, jsonl, csv
    #[arg(long, short = 'f', global = true, default_value = "json")]
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
}

#[derive(Debug, Subcommand)]
enum GmailCommands {
    /// List messages
    List {
        /// Search query (Gmail search syntax)
        #[arg(long, short = 'q')]
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
        /// Decode body content
        #[arg(long)]
        decode_body: bool,
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
}

#[derive(Debug, Subcommand)]
enum DriveCommands {
    /// List files
    List {
        /// Search query (Drive query syntax)
        #[arg(long, short = 'q')]
        query: Option<String>,
        /// Maximum results
        #[arg(long, default_value = "20")]
        limit: u32,
        /// Parent folder ID
        #[arg(long)]
        parent: Option<String>,
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
    },
    /// Clear a range of cells
    Clear {
        /// Spreadsheet ID
        id: String,
        /// Range to clear in A1 notation
        #[arg(long)]
        range: String,
    },
}

#[derive(Debug, Subcommand)]
enum SlidesCommands {
    /// Get presentation info
    Get {
        /// Presentation ID
        id: String,
        /// Extract text only
        #[arg(long)]
        text_only: bool,
    },
    /// Get specific page
    Page {
        /// Presentation ID
        id: String,
        /// Page number (0-indexed)
        #[arg(long)]
        page: u32,
        /// Extract text only
        #[arg(long)]
        text_only: bool,
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
        /// Show completed tasks
        #[arg(long)]
        show_completed: bool,
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
    let token_manager = Arc::new(RwLock::new(TokenManager::new(config.clone())));

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
                    match workspace_cli::commands::gmail::list::list_messages(&client, params).await {
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
                GmailCommands::Get { id, decode_body } => {
                    let format_param = if decode_body { "full" } else { "metadata" };
                    match workspace_cli::commands::gmail::get::get_message(&client, &id, format_param).await {
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
                    };

                    match workspace_cli::commands::gmail::send::send_message(&client, params).await {
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
                GmailCommands::Draft { to, subject, body } => {
                    let body_content = body.unwrap_or_default();

                    let params = workspace_cli::commands::gmail::send::ComposeParams {
                        to,
                        subject,
                        body: body_content,
                        from: None,
                        cc: None,
                    };

                    match workspace_cli::commands::gmail::send::create_draft(&client, params).await {
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
                DriveCommands::List { query, limit, parent } => {
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
                        page_token: None,
                        fields: None,
                        order_by: None,
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
                CalendarCommands::List { calendar, time_min, time_max, limit, sync_token } => {
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
                DocsCommands::Get { id, markdown } => {
                    match workspace_cli::commands::docs::get::get_document(&client, &id).await {
                        Ok(doc) => {
                            if markdown {
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
                SheetsCommands::Get { id, range } => {
                    match workspace_cli::commands::sheets::get::get_values(&client, &id, &range).await {
                        Ok(response) => {
                            if format == OutputFormat::Csv {
                                let csv = workspace_cli::commands::sheets::get::values_to_csv(&response);
                                if let Some(ref output_path) = cli.output {
                                    std::fs::write(output_path, &csv)?;
                                } else if !quiet {
                                    print!("{}", csv);
                                }
                            } else if let Some(ref output_path) = cli.output {
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
                SheetsCommands::Create { title } => {
                    match workspace_cli::commands::sheets::create::create_spreadsheet(&client, &title).await {
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
                SlidesCommands::Get { id, text_only } => {
                    match workspace_cli::commands::slides::get::get_presentation(&client, &id).await {
                        Ok(presentation) => {
                            if text_only {
                                let text = workspace_cli::commands::slides::get::extract_all_text(&presentation);
                                if let Some(ref output_path) = cli.output {
                                    std::fs::write(output_path, &text)?;
                                } else {
                                    println!("{}", text);
                                }
                            } else if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(&presentation)?;
                            } else {
                                formatter.write(&presentation)?;
                            }
                        }
                        Err(e) => {
                            eprintln!(r#"{{"status":"error","message":"{}"}}"#, e);
                            std::process::exit(1);
                        }
                    }
                }
                SlidesCommands::Page { id, page, text_only } => {
                    // Get the presentation first, then extract the specific slide
                    match workspace_cli::commands::slides::get::get_presentation(&client, &id).await {
                        Ok(presentation) => {
                            let page_index = page as usize;
                            if page_index >= presentation.slides.len() {
                                eprintln!(r#"{{"status":"error","message":"Page {} not found. Presentation has {} slides."}}"#, page, presentation.slides.len());
                                std::process::exit(1);
                            }

                            let slide = &presentation.slides[page_index];
                            if text_only {
                                let text = workspace_cli::commands::slides::get::extract_page_text(slide);
                                if let Some(ref output_path) = cli.output {
                                    std::fs::write(output_path, &text)?;
                                } else {
                                    println!("{}", text);
                                }
                            } else if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_fields(fields.clone()).with_quiet(quiet).with_writer(file);
                                file_formatter.write(slide)?;
                            } else {
                                formatter.write(slide)?;
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
                TasksCommands::List { list, show_completed } => {
                    let params = workspace_cli::commands::tasks::list::ListTasksParams {
                        task_list_id: list,
                        max_results: 100,
                        show_completed,
                        show_hidden: false,
                        page_token: None,
                    };
                    match workspace_cli::commands::tasks::list::list_tasks(&client, params).await {
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
                    let mut tm = token_manager.write().await;
                    match tm.login_interactive(credentials.map(std::path::PathBuf::from)).await {
                        Ok(()) => {
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
    }

    Ok(())
}
