# CLAUDE.md - workspace-cli Development Guide

## Project Overview

**workspace-cli** is a high-performance Rust CLI for Google Workspace APIs, optimized for AI agent integration. It provides structured JSON output for Gmail, Drive, Calendar, Docs, Sheets, Slides, and Tasks.

**Author:** Majid Manzarpour
**License:** MIT
**Rust Edition:** 2021

## Quick Command Reference

### Build & Test
```bash
cargo build --release          # Release build (optimized, ~4MB binary)
cargo build                    # Debug build
cargo test                     # Run all tests (9 tests)
cargo clippy                   # Lint check
```

### Binary Location
```
./target/release/workspace-cli   # Release binary
./target/debug/workspace-cli     # Debug binary
```

### Authentication
```bash
workspace-cli auth login --credentials /path/to/credentials.json
workspace-cli auth status
workspace-cli auth logout
```

## Architecture

```
src/
├── main.rs              # CLI entry point, clap command definitions (~2000 lines)
├── lib.rs               # Library exports
├── cli.rs               # CLI context utilities
├── auth/                # OAuth2 & token management
│   ├── oauth.rs         # WorkspaceAuthenticator, SCOPES
│   ├── token.rs         # TokenManager (get_access_token, ensure_authenticated)
│   └── keyring_storage.rs  # OS keyring integration
├── client/              # API client infrastructure
│   ├── api_client.rs    # ApiClient with rate limiting & retry
│   ├── rate_limiter.rs  # Per-API rate limiters
│   ├── retry.rs         # Exponential backoff retry logic
│   └── batch.rs         # BatchClient for multipart/mixed batch requests
├── commands/            # Service implementations
│   ├── gmail/           # list, get, send, reply, delete, trash, labels
│   ├── drive/           # list, upload, download, mkdir, share, etc.
│   ├── calendar/        # list, create, update, delete events
│   ├── docs/            # get, append, create, replace
│   ├── sheets/          # get, update, append, create, clear
│   ├── slides/          # get presentation/page
│   ├── tasks/           # lists, list, create, update, delete
│   ├── chat/            # spaces, messages, read_state (unread detection)
│   ├── contacts/        # list, search, create, delete, directory
│   ├── groups/          # list (Admin SDK), members (Cloud Identity)
│   └── batch/           # CLI wrapper for batch API requests
├── config/              # Config file handling (~/.config/workspace-cli/)
├── error/               # Structured error types (CliError, WorkspaceError)
├── output/              # Formatter (JSON/JSONL/CSV), field filtering
└── utils/               # base64, field_mask, html_to_md
```

## Key Components

### ApiClient (`src/client/api_client.rs`)
Factory methods create service-specific clients with appropriate rate limiters:
```rust
ApiClient::gmail(token_manager)    // Gmail API client
ApiClient::drive(token_manager)    // Drive API client
ApiClient::calendar(token_manager) // Calendar API client
ApiClient::docs(token_manager)     // Docs API client
ApiClient::sheets(token_manager)   // Sheets API client
ApiClient::slides(token_manager)   // Slides API client
ApiClient::tasks(token_manager)    // Tasks API client
ApiClient::chat(token_manager)     // Chat API client
ApiClient::contacts(token_manager) // Contacts (People API) client
ApiClient::groups(token_manager)   // Groups (Cloud Identity) client
ApiClient::admin(token_manager)    // Admin SDK Directory client
```

### API Endpoints (`src/client/api_client.rs:11-19`)
```rust
GMAIL:    "https://gmail.googleapis.com/gmail/v1"
DRIVE:    "https://www.googleapis.com/drive/v3"
CALENDAR: "https://www.googleapis.com/calendar/v3"
DOCS:     "https://docs.googleapis.com/v1"
SHEETS:   "https://sheets.googleapis.com/v4"
SLIDES:   "https://slides.googleapis.com/v1"
TASKS:    "https://tasks.googleapis.com/tasks/v1"
CHAT:     "https://chat.googleapis.com/v1"
CONTACTS: "https://people.googleapis.com/v1"
GROUPS:   "https://cloudidentity.googleapis.com/v1"
ADMIN:    "https://admin.googleapis.com/admin/directory/v1"
```

### Output Formatter (`src/output/formatter.rs`)
Handles JSON, JSON-compact, JSONL, and CSV output with field filtering:
- `--format json|json-compact|jsonl|csv`
- `--fields "id,name,mimeType"` - Filter output fields
- `--quiet` - Suppress output
- `--output file.json` - Write to file

**Important:** Field filtering handles wrapper objects (`files`, `messages`, `items`, `labels`, `permissions`) by filtering array items, not the wrapper itself.

### Error Handling (`src/error/types.rs`)
Structured errors for agent consumption:
```rust
ErrorCode::AuthenticationFailed
ErrorCode::TokenExpired
ErrorCode::RateLimitExceeded
ErrorCode::QuotaExceeded
ErrorCode::NotFound
ErrorCode::PermissionDenied
ErrorCode::InvalidRequest
ErrorCode::NetworkError
ErrorCode::ServerError
```

## CLI Command Structure

### Global Options (all commands)
```
-f, --format <FORMAT>    Output format: json, jsonl, csv [default: json]
--fields <FIELDS>        Comma-separated fields to include
-o, --output <FILE>      Write output to file
-q, --quiet              Suppress non-essential output
```

### Gmail Commands
```bash
gmail list [--query "is:unread"] [--limit 20] [--label INBOX]
gmail get <id> [--full]                    # Minimal by default (headers + plain text body)
gmail send --to <email> --subject <text> --body <text> [--body-file <path>]
gmail draft --to <email> --subject <text> [--body <text>]
gmail reply <id> --body <text> [--body-file <path>] [--all]
gmail reply-draft <id> --body <text> [--all]
gmail delete <id>
gmail trash <id>
gmail untrash <id>
gmail labels
gmail modify <id> [--add-labels L1,L2] [--remove-labels L3] [--mark-read] [--mark-unread] [--star] [--unstar] [--archive]
```

**Token Optimization (defaults to minimal output):**
- `gmail get` returns essential headers (from, to, subject, date) + plain text body (~88% reduction). Use `--full` for raw message structure.
- `gmail send/reply/draft` return only `{success, id, threadId}` (~90% reduction)
- `gmail modify` returns only `{success, id, labels}` (~99% reduction)

### Drive Commands
```bash
drive list [--query <q>] [--limit 20] [--parent <folder-id>]
drive get <id>
drive upload <file> [--parent <folder-id>] [--name <name>]
drive download <id> [--output <path>]
drive delete <id>
drive trash <id>
drive untrash <id>
drive mkdir <name> [--parent <folder-id>]
drive move <id> --to <folder-id>
drive copy <id> [--name <name>] [--parent <folder-id>]
drive rename <id> <new-name>
drive share <id> --email <email> --role reader|writer|commenter
drive share <id> --anyone --role reader
drive permissions <id>
drive unshare <id> <permission-id>
```

### Calendar Commands
```bash
calendar list [--calendar primary] [--time-min 2025-01-01T00:00:00Z] [--time-max ...] [--limit 20] [--sync-token <token>] [--full]
calendar create --summary <title> --start <datetime> --end <datetime> [--description <text>] [--calendar primary]
calendar update <id> [--summary <title>] [--start <datetime>] [--end <datetime>] [--calendar primary]
calendar delete <id> [--calendar primary]
```

**Token Optimization:** `calendar list` returns minimal event fields (id, summary, start, end, status) by default (~50% reduction). Use `--full` for attendees, organizer, description, recurrence, etc.

### Docs Commands
```bash
docs get <id> [--markdown] [--text]        # --text for plain text output
docs create <title>
docs append <id> <text>
docs replace <id> --find <text> --with <replacement> [--match-case]
```

**Token Optimization:** Use `--text` for plain text extraction (~70% reduction vs JSON structure). Use `--markdown` for formatted text.

### Sheets Commands
```bash
sheets get <id> --range "Sheet1!A1:C10" [--full]  # Values array by default
sheets create <title>
sheets update <id> --range "Sheet1!A1:B2" --values '[["Name","Value"],["A","1"]]'
sheets append <id> --range "Sheet1!A1" --values '[["Row1","Data"]]'
sheets clear <id> --range "Sheet1!A1:C10"
```

**Token Optimization:** `sheets get` returns just the values array by default (~50% reduction). Use `--full` for range metadata wrapper.

### Slides Commands
```bash
slides get <id> [--full]                   # Text extraction by default
slides page <id> --page 0 [--full]         # Text extraction by default
```

**Token Optimization:** Returns extracted text content by default (~93% reduction). Use `--full` for complete presentation structure (masters, layouts, transforms, colors).

### Tasks Commands
```bash
tasks lists                           # List all task lists
tasks list [--list @default] [--limit 20] [--show-completed] [--full]
tasks create <title> [--list @default] [--due 2025-01-20T12:00:00Z] [--notes <text>]
tasks update <id> [--list @default] [--title <text>] [--complete]
tasks delete <id> [--list @default]
```

**Token Optimization:** `tasks list` returns minimal task fields (id, title, status, due, notes, completed) by default (~40% reduction). Use `--full` for etag, selfLink, links, parent, position, etc.

### Chat Commands
```bash
chat spaces-list [--limit 100]                              # List all Chat spaces
chat spaces-find --name <name>                              # Find space by display name
chat spaces-create --name <name> --member <email>           # Create a space with members
chat messages-list --space <id> [--limit 25] [--order desc] # List messages (newest first by default)
chat messages-list --space <id> --after <RFC-3339>          # Messages after timestamp
chat messages-list --space <id> --before <RFC-3339>         # Messages before timestamp
chat send --space <id> --text <text> [--thread <thread>]    # Send message (optionally in thread)
chat get <message-name>                                     # Get a specific message
chat read-state --space <id>                                # Get space read state (lastReadTime)
chat thread-read-state --space <id> --thread <thread>       # Get thread read state
chat unread [--limit 10] [--type SPACE]                     # Show unread messages across spaces
chat unread --type DIRECT_MESSAGE                           # Unread DMs only (last 7 days)
chat unread --type DIRECT_MESSAGE --since 30d               # Unread DMs from last 30 days
chat unread --type all --since all                          # All space types, no time limit
```

**Note:** `chat unread` uses a multi-stage optimization pipeline:
1. Server-side `spaceType` filter reduces API calls
2. `--since` flag (default: 7d) filters by `lastActiveTime` before any read state fetches
3. `lastActiveTime` vs `lastReadTime` comparison skips already-read spaces
4. Read states fetched concurrently in batches of 50, bot DMs filtered out
5. Messages fetched only for spaces with confirmed unread activity

The `--type` flag accepts: SPACE (default), DIRECT_MESSAGE, GROUP_CHAT, or all.
The `--since` flag accepts: 1d, 7d, 30d, or all (no limit).

### Contacts Commands
```bash
contacts list [--limit 100]                                 # List personal contacts
contacts search --query <q>                                 # Search contacts by name/email
contacts get <resourceName>                                 # Get a specific contact
contacts create --given <name> --email <email>              # Create a contact
contacts delete <resourceName>                              # Delete a contact
contacts directory-list [--limit 100]                       # List domain directory people
contacts directory-search --query <q>                       # Search domain directory
```

### Groups Commands
```bash
groups list [--email <email>] [--limit 200]                 # List groups for user (Admin SDK)
groups members <groupEmail> [--limit 200]                   # List group members (Cloud Identity)
```

### Batch Commands
Execute up to 100 API requests in a single HTTP call:
```bash
batch gmail --requests '<json-array>'     # Batch Gmail API requests
batch gmail --file requests.json          # Read requests from file
batch drive --requests '<json-array>'     # Batch Drive API requests
batch calendar --requests '<json-array>'  # Batch Calendar API requests
echo '<json>' | batch gmail               # Read from stdin
```

**Request format:**
```json
[
  {"id": "req1", "method": "GET", "path": "/gmail/v1/users/me/messages/abc123"},
  {"id": "req2", "method": "POST", "path": "/gmail/v1/users/me/messages/xyz/modify", "body": {"addLabelIds": ["STARRED"]}}
]
```

**Response format:**
```json
{
  "status": "success|partial|error",
  "results": [{"id": "req1", "status": 200, "body": {...}}],
  "errors": [{"id": "req2", "status": 400, "message": "..."}]
}
```

**Path prefixes by service:**
- Gmail: `/gmail/v1/...`
- Drive: `/drive/v3/...`
- Calendar: `/calendar/v3/...`

## Interpreting User Requests

### Common Patterns

| User Says | Command |
|-----------|---------|
| "list my emails" / "show inbox" | `gmail list --limit 20` |
| "unread emails" | `gmail list --query "is:unread"` |
| "emails from X" | `gmail list --query "from:X"` |
| "read email <id>" | `gmail get <id>` |
| "send email to X" | `gmail send --to X --subject "..." --body "..."` |
| "reply to email" | `gmail reply <id> --body "..."` |
| "reply all" | `gmail reply <id> --body "..." --all` |
| "draft a reply" | `gmail reply-draft <id> --body "..."` |
| "list files" / "my drive" | `drive list --limit 20` |
| "files in folder" | `drive list --parent <folder-id>` |
| "search for X" | `drive list --query "name contains 'X'"` |
| "upload file" | `drive upload <path>` |
| "download file" | `drive download <id> --output <path>` |
| "share with X" | `drive share <id> --email X --role writer` |
| "who has access" | `drive permissions <id>` |
| "my calendar" / "events" | `calendar list --time-min <today>` |
| "schedule meeting" | `calendar create --summary "..." --start ... --end ...` |
| "read document" | `docs get <id> --markdown` |
| "add to doc" | `docs append <id> "text"` |
| "spreadsheet data" | `sheets get <id> --range "Sheet1!A:Z"` |
| "my tasks" / "todo list" | `tasks list` |
| "add task" | `tasks create "title"` |
| "complete task" | `tasks update <id> --complete` |
| "unread chats" / "new messages" | `chat unread` |
| "unread DMs" / "new direct messages" | `chat unread --type DIRECT_MESSAGE` |
| "all unread" / "everything unread" | `chat unread --type all --since all` |
| "chat spaces" / "list rooms" | `chat spaces-list` |
| "messages in room" | `chat messages-list --space <id>` |
| "send chat" / "message room" | `chat send --space <id> --text "..."` |
| "find chat room" | `chat spaces-find --name "..."` |
| "my contacts" | `contacts list` |
| "find contact" | `contacts search --query "..."` |
| "company directory" | `contacts directory-list` |
| "search directory" | `contacts directory-search --query "..."` |
| "my groups" | `groups list` |
| "group members" | `groups members <email>` |
| "batch request" / "bulk operation" | `batch gmail/drive/calendar --requests '[...]'` |
| "get multiple emails at once" | `batch gmail --requests '[{"id":"1","method":"GET","path":"/gmail/v1/users/me/messages/id1"},...]'` |
| "star all these messages" | `batch gmail --requests '[{"id":"1","method":"POST","path":"/gmail/v1/users/me/messages/id1/modify","body":{"addLabelIds":["STARRED"]}}]'` |

### ID Extraction
Google Workspace IDs are found in URLs:
- Drive: `https://drive.google.com/file/d/<ID>/view`
- Docs: `https://docs.google.com/document/d/<ID>/edit`
- Sheets: `https://docs.google.com/spreadsheets/d/<ID>/edit`
- Slides: `https://docs.google.com/presentation/d/<ID>/edit`

### Date/Time Format
All datetime parameters use RFC3339 format:
```
2025-01-15T14:00:00Z      # UTC
2025-01-15T14:00:00-08:00 # With timezone
```

## Configuration

### Config File Location
```
~/.config/workspace-cli/config.toml
```

### Config Structure
```toml
[auth]
credentials_path = "/path/to/credentials.json"
service_account_path = "/path/to/service-account.json"

[output]
format = "json"
compact = false

[api]
timeout_seconds = 30
max_retries = 3
```

### Environment Variables
```bash
WORKSPACE_CREDENTIALS_PATH      # OAuth credentials JSON path
GOOGLE_APPLICATION_CREDENTIALS  # Service account JSON path
WORKSPACE_OUTPUT_FORMAT         # Default output format
WORKSPACE_OUTPUT_COMPACT        # true/false
WORKSPACE_API_TIMEOUT           # Timeout in seconds
WORKSPACE_API_MAX_RETRIES       # Max retry attempts
RUST_LOG                        # Logging level (debug, info, warn, error)
```

## Known Patterns & Gotchas

### List Response Wrappers
API list responses wrap items in arrays:
- Drive: `{ "files": [...], "nextPageToken": "..." }`
- Gmail: `{ "messages": [...], "resultSizeEstimate": N }`
- Tasks: `{ "items": [...] }`
- Calendar: `{ "items": [...] }`

The `--fields` flag filters within these arrays, not at root level.

### Gmail Query Syntax
Uses Gmail's search syntax:
```
is:unread
from:user@example.com
subject:keyword
has:attachment
after:2025/01/01
before:2025/12/31
label:INBOX
```

### Drive Query Syntax
Uses Drive's query syntax:
```
name contains 'report'
mimeType = 'application/vnd.google-apps.folder'
'folder-id' in parents
trashed = false
modifiedTime > '2025-01-01T00:00:00'
```

### Tasks API Limits
- `maxResults`: 1-100 (default 20)
- List ID `@default` refers to primary task list

### Email Subject Encoding
Non-ASCII subjects (emojis, special characters) are RFC 2047 Base64 encoded automatically.

## Development Notes

### Adding New Commands
1. Add subcommand enum variant in `src/main.rs`
2. Create handler in appropriate `src/commands/<service>/` module
3. Wire up in `run()` function match statement
4. Add types in `types.rs` if needed

### Testing Against Live API
```bash
# Enable debug logging
RUST_LOG=debug ./target/release/workspace-cli gmail list --limit 1

# Test read-only operations
./target/release/workspace-cli drive list --limit 3
./target/release/workspace-cli gmail labels
./target/release/workspace-cli tasks lists
./target/release/workspace-cli calendar list --time-min "2025-01-01T00:00:00Z"
```

### Common Build Issues
- Keyring issues on Linux: May need `gnome-keyring` or `libsecret`
- SSL issues: Ensure `openssl-dev` is installed

## Dependencies (Key)

| Crate | Purpose |
|-------|---------|
| tokio | Async runtime |
| clap | CLI parsing |
| reqwest | HTTP client |
| serde/serde_json | JSON serialization |
| yup-oauth2 | Google OAuth2 |
| keyring | OS credential storage |
| base64 | Encoding |
| chrono | Date/time handling |
| thiserror | Error types |
