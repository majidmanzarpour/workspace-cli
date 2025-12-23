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
            let mut formatter = Formatter::new(format);

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
                                let mut file_formatter = Formatter::new(format).with_writer(file);
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
                                let mut file_formatter = Formatter::new(format).with_writer(file);
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
                                let mut file_formatter = Formatter::new(format).with_writer(file);
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
                                let mut file_formatter = Formatter::new(format).with_writer(file);
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
            let mut formatter = Formatter::new(format);

            match command {
                DriveCommands::List { query, limit, parent: _ } => {
                    let params = workspace_cli::commands::drive::list::ListParams {
                        query,
                        max_results: limit,
                        page_token: None,
                        fields: None,
                        order_by: None,
                    };
                    match workspace_cli::commands::drive::list::list_files(&client, params).await {
                        Ok(response) => {
                            if let Some(ref output_path) = cli.output {
                                let file = std::fs::File::create(output_path)?;
                                let mut file_formatter = Formatter::new(format).with_writer(file);
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
                DriveCommands::Upload { file: _, parent: _, name: _ } => {
                    println!(r#"{{"status":"error","message":"Upload command not implemented yet"}}"#);
                    std::process::exit(1);
                }
                DriveCommands::Download { id: _, output: _ } => {
                    println!(r#"{{"status":"error","message":"Download command not implemented yet"}}"#);
                    std::process::exit(1);
                }
                DriveCommands::Get { id: _ } => {
                    println!(r#"{{"status":"error","message":"Drive Get command not implemented yet"}}"#);
                    std::process::exit(1);
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
            let mut formatter = Formatter::new(format);

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
                                let mut file_formatter = Formatter::new(format).with_writer(file);
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
                CalendarCommands::Create { summary: _, start: _, end: _, description: _, calendar: _ } => {
                    println!(r#"{{"status":"error","message":"Create event command not implemented yet"}}"#);
                    std::process::exit(1);
                }
                CalendarCommands::Update { id: _, summary: _, start: _, end: _, calendar: _ } => {
                    println!(r#"{{"status":"error","message":"Update event command not implemented yet"}}"#);
                    std::process::exit(1);
                }
                CalendarCommands::Delete { id: _, calendar: _ } => {
                    println!(r#"{{"status":"error","message":"Delete event command not implemented yet"}}"#);
                    std::process::exit(1);
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

            match command {
                DocsCommands::Get { id: _, markdown: _ } => {
                    println!(r#"{{"status":"error","message":"Docs Get command not implemented yet"}}"#);
                    std::process::exit(1);
                }
                DocsCommands::Append { id: _, text: _ } => {
                    println!(r#"{{"status":"error","message":"Docs Append command not implemented yet"}}"#);
                    std::process::exit(1);
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

            match command {
                SheetsCommands::Get { id: _, range: _ } => {
                    println!(r#"{{"status":"error","message":"Sheets Get command not implemented yet"}}"#);
                    std::process::exit(1);
                }
                SheetsCommands::Update { id: _, range: _, values: _ } => {
                    println!(r#"{{"status":"error","message":"Sheets Update command not implemented yet"}}"#);
                    std::process::exit(1);
                }
                SheetsCommands::Append { id: _, range: _, values: _ } => {
                    println!(r#"{{"status":"error","message":"Sheets Append command not implemented yet"}}"#);
                    std::process::exit(1);
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

            match command {
                SlidesCommands::Get { id: _, text_only: _ } => {
                    println!(r#"{{"status":"error","message":"Slides Get command not implemented yet"}}"#);
                    std::process::exit(1);
                }
                SlidesCommands::Page { id: _, page: _, text_only: _ } => {
                    println!(r#"{{"status":"error","message":"Slides Page command not implemented yet"}}"#);
                    std::process::exit(1);
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

            match command {
                TasksCommands::Lists => {
                    println!(r#"{{"status":"error","message":"Tasks Lists command not implemented yet"}}"#);
                    std::process::exit(1);
                }
                TasksCommands::List { list: _, show_completed: _ } => {
                    println!(r#"{{"status":"error","message":"Tasks List command not implemented yet"}}"#);
                    std::process::exit(1);
                }
                TasksCommands::Create { title: _, list: _, due: _, notes: _ } => {
                    println!(r#"{{"status":"error","message":"Tasks Create command not implemented yet"}}"#);
                    std::process::exit(1);
                }
                TasksCommands::Update { id: _, list: _, title: _, complete: _ } => {
                    println!(r#"{{"status":"error","message":"Tasks Update command not implemented yet"}}"#);
                    std::process::exit(1);
                }
                TasksCommands::Delete { id: _, list: _ } => {
                    println!(r#"{{"status":"error","message":"Tasks Delete command not implemented yet"}}"#);
                    std::process::exit(1);
                }
            }
        }
        Commands::Auth { command } => {
            match command {
                AuthCommands::Login { credentials } => {
                    let mut tm = token_manager.write().await;
                    match tm.login_interactive(credentials.map(std::path::PathBuf::from)).await {
                        Ok(()) => {
                            println!(r#"{{"status":"success","message":"Login successful"}}"#);
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
                            println!(r#"{{"status":"success","message":"Logged out"}}"#);
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
                    println!("{}", serde_json::to_string_pretty(&status).unwrap());
                }
            }
        }
    }

    Ok(())
}
