# workspace-cli

High-performance Google Workspace CLI optimized for AI agent integration.

## Overview

`workspace-cli` is a Rust-based command-line tool designed to provide programmatic access to Google Workspace APIs with structured output (TOON/JSON/JSONL/CSV) optimized for AI agent consumption. Built for speed, efficiency, and deterministic execution.

## Features

- **Gmail**: List, read, send, draft, reply, delete, trash/untrash, labels management, and modify messages
- **Drive**: List, upload, download, delete, trash/untrash, mkdir, move, copy, rename, share, and manage permissions
- **Calendar**: List, create, update, and delete events with sync token support
- **Docs**: Read documents as Markdown, append content, create documents, find/replace text, and apply rich formatting via batchUpdate (headings, bold, bullets)
- **Sheets**: Read, write, append, create spreadsheets, and clear ranges
- **Slides**: Get presentations, extract text, create presentations, add slides/shapes/tables/charts, delete elements, and raw batchUpdate
- **Chat**: List spaces, send messages, DMs, unread detection, mark-read (single + bulk), mute-aware filtering
- **Contacts**: List, search, get, create, update, delete contacts, directory list/search
- **Groups**: List group memberships, list group members
- **Tasks**: Manage task lists and individual tasks
- **Batch**: Execute up to 100 API requests in a single HTTP call for maximum efficiency

### Key Capabilities

- **Structured Output**: All responses in TOON (default, token-efficient), JSON, JSONL, or CSV formats
- **Field Masking**: Reduce token costs by selecting only needed fields
- **Auto-Pagination**: Fetch all pages automatically with `--page-all`, `--page-limit`, and `--page-delay`
- **Dry Run**: Preview any API request before executing it with `--dry-run`
- **Auth Export**: Export stored credentials for use in scripts or CI with `auth export`
- **MCP Server**: Built-in Model Context Protocol server exposing ~50 tools for AI agents (`workspace-cli mcp`)
- **Rate Limiting**: Built-in retry logic with exponential backoff
- **Streaming**: JSONL output for real-time processing of paginated results
- **Secure Auth**: OS keyring integration for token storage
- **Error Handling**: Structured, actionable error messages with retry guidance

## Pagination

All list commands support automatic multi-page fetching:

```bash
# Fetch all pages automatically (streams items as they arrive)
workspace-cli gmail list --query "is:unread" --page-all

# Limit to first 3 pages
workspace-cli drive list --page-all --page-limit 3

# Add delay between pages to avoid rate limits
workspace-cli calendar list --time-min "2026-01-01T00:00:00Z" --page-all --page-delay 200

# --page-limit 0 = unlimited (same as --page-all)
workspace-cli contacts list --page-limit 0
```

Global pagination flags (work with any list command):
- `--page-all` — fetch all pages (overrides `--page-limit`)
- `--page-limit N` — max pages to fetch (default: 10; 0 = unlimited)
- `--page-delay N` — milliseconds between page requests (default: 100)

---

## Dry Run

Preview any API request before executing it:

```bash
workspace-cli gmail send --to test@example.com --subject "Test" --body "Hello" --dry-run
workspace-cli drive list --query "name='test'" --dry-run
workspace-cli calendar create --summary "Meeting" --start "2026-03-10T14:00:00Z" --end "2026-03-10T15:00:00Z" --dry-run
```

Output:
```json
{
  "dry_run": true,
  "method": "POST",
  "url": "https://gmail.googleapis.com/gmail/v1/users/me/messages/send",
  "body": { },
  "auth": "Bearer [REDACTED]"
}
```

The `--dry-run` flag works globally with any command that makes API calls.

---

## Auth Export

Export stored credentials for use in scripts or CI:

```bash
# Show token info (masked by default)
workspace-cli auth export

# Show full unmasked token
workspace-cli auth export --unmasked

# Write to file
workspace-cli auth export --output /tmp/creds.json
```

Note: Access tokens expire in ~1 hour. For long-running processes, copy the `token_cache_path` file instead.

---

## MCP Server

workspace-cli includes a built-in MCP (Model Context Protocol) server that exposes all Google Workspace operations as tools for AI agents.

```bash
# Start the MCP server (stdio transport)
workspace-cli mcp
```

The MCP server exposes ~50 tools covering all services:

| Service | Tools |
|---------|-------|
| Gmail | gmail_list, gmail_get, gmail_send, gmail_reply, gmail_labels, gmail_modify, gmail_trash, gmail_delete |
| Drive | drive_list, drive_get, drive_mkdir, drive_move, drive_copy, drive_rename, drive_share, drive_permissions, drive_trash, drive_delete |
| Calendar | calendar_list, calendar_create, calendar_update, calendar_delete |
| Docs | docs_get, docs_create, docs_append, docs_replace, docs_batch_update |
| Sheets | sheets_get, sheets_create, sheets_update, sheets_append, sheets_clear, sheets_list_sheets |
| Slides | slides_get, slides_page, slides_create, slides_add_slide, slides_add_shape, slides_add_table, slides_add_chart, slides_delete, slides_batch_update |
| Tasks | tasks_lists, tasks_list, tasks_create, tasks_update, tasks_delete |
| Chat | chat_spaces_list, chat_find_dm, chat_messages_list, chat_send, chat_unread, chat_mark_read |
| Contacts | contacts_list, contacts_search, contacts_get, contacts_create, contacts_delete, contacts_directory_list, contacts_directory_search |

**Building with MCP support:**
```bash
cargo build --release --features mcp
```

**Using with Claude Desktop or other MCP clients:**
```json
{
  "mcpServers": {
    "workspace": {
      "command": "workspace-cli",
      "args": ["mcp"]
    }
  }
}
```

The MCP binary is built with `--features mcp`. The standard binary (without `--features mcp`) does not include the MCP server to keep binary size minimal.

---

## Installation

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- Google Cloud project with Workspace API access

### Build from Source

```bash
# Clone the repository
cd workspace-cli

# Build release binary
cargo build --release

# Build with MCP server support
cargo build --release --features mcp

# Install to system path
cp target/release/workspace-cli /usr/local/bin/
# or on macOS/Linux
sudo install -m 755 target/release/workspace-cli /usr/local/bin/

# Verify installation
workspace-cli --version
```

## Authentication

### OAuth2 Setup (Interactive)

For user-attended sessions with browser-based authentication:

1. **Create a Google Cloud Project**
   - Go to [Google Cloud Console](https://console.cloud.google.com)
   - Create a new project or select an existing one

2. **Enable Required APIs**
   - Navigate to "APIs & Services" > "Library"
   - Enable the following APIs:
     - Gmail API
     - Google Drive API
     - Google Calendar API
     - Google Docs API
     - Google Sheets API
     - Google Slides API
     - Google Tasks API
     - Google Chat API (requires additional setup — see note below)
     - People API (Contacts)
     - Admin SDK API (Groups)

   **Chat API Setup:** Enabling the Chat API alone is not sufficient. You must also configure a Chat app in the [Google Cloud Console](https://console.cloud.google.com/apis/api/chat.googleapis.com/hangouts-chat) (set an app name and configure it) before Chat commands will work.

   **Groups:** The `groups` commands use the Admin SDK API and require a Google Workspace account. They will not work with personal Gmail accounts.

3. **Create OAuth2 Credentials**
   - Go to "APIs & Services" > "Credentials"
   - Click "Create Credentials" > "OAuth client ID"
   - Choose "Desktop application" as the application type
   - Name it (e.g., "workspace-cli")
   - Download the credentials JSON file

4. **Login with OAuth2**
   ```bash
   workspace-cli auth login --credentials path/to/credentials.json
   ```
   - This will open your browser for authentication
   - Tokens are securely stored in your OS keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service)

5. **Check Authentication Status**
   ```bash
   workspace-cli auth status
   ```

### Service Account (Headless)

For server environments, CI/CD pipelines, or automated workflows:

1. **Create a Service Account**
   - In Google Cloud Console, go to "IAM & Admin" > "Service Accounts"
   - Click "Create Service Account"
   - Grant necessary permissions
   - Create and download a JSON key

2. **Set Environment Variable**
   ```bash
   export GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account-key.json
   ```

3. **Domain-Wide Delegation (Optional)**
   - For Google Workspace admin access, enable domain-wide delegation
   - Configure OAuth scopes in Workspace admin console

### Logout

```bash
workspace-cli auth logout
```

## Quick Start

### Gmail Examples

```bash
# List unread emails
workspace-cli gmail list --query "is:unread" --limit 5

# Search emails from specific sender
workspace-cli gmail list --query "from:boss@company.com" --limit 10

# Get a specific message (minimal by default - headers + plain text body)
workspace-cli gmail get <message-id>

# Get full message structure (includes raw payload, MIME parts, etc.)
workspace-cli gmail get <message-id> --full

# Send an email
workspace-cli gmail send \
  --to user@example.com \
  --subject "Hello from workspace-cli" \
  --body "This is a test email"

# Send email with body from file
workspace-cli gmail send \
  --to user@example.com \
  --subject "Report" \
  --body-file report.txt

# Filter by label
workspace-cli gmail list --label "INBOX" --limit 20

# List all labels
workspace-cli gmail labels

# Move message to trash
workspace-cli gmail trash <message-id>

# Restore message from trash
workspace-cli gmail untrash <message-id>

# Permanently delete a message
workspace-cli gmail delete <message-id>

# Mark message as read
workspace-cli gmail modify <message-id> --mark-read

# Mark message as unread and star it
workspace-cli gmail modify <message-id> --mark-unread --star

# Archive a message (remove from inbox)
workspace-cli gmail modify <message-id> --archive

# Add/remove labels
workspace-cli gmail modify <message-id> --add-labels "Label1,Label2" --remove-labels "INBOX"
```

### Drive Examples

```bash
# List all files
workspace-cli drive list --limit 10

# List files in a specific folder
workspace-cli drive list --parent <folder-id>

# Search for specific files
workspace-cli drive list --query "name contains 'report'" --limit 5

# Upload a file
workspace-cli drive upload myfile.pdf

# Upload to specific folder
workspace-cli drive upload myfile.pdf --parent <folder-id>

# Upload with custom name
workspace-cli drive upload myfile.pdf --name "renamed-file.pdf"

# Download a file
workspace-cli drive download <file-id> --output ./downloaded-file.pdf

# Get file metadata
workspace-cli drive get <file-id>

# Create a folder
workspace-cli drive mkdir "New Folder"

# Create folder in specific parent
workspace-cli drive mkdir "Subfolder" --parent <folder-id>

# Move a file to a different folder
workspace-cli drive move <file-id> --to <folder-id>

# Copy a file
workspace-cli drive copy <file-id> --name "Copy of file"

# Rename a file
workspace-cli drive rename <file-id> "new-name.pdf"

# Move file to trash
workspace-cli drive trash <file-id>

# Restore file from trash
workspace-cli drive untrash <file-id>

# Permanently delete a file
workspace-cli drive delete <file-id>

# Share file with a user
workspace-cli drive share <file-id> --email user@example.com --role writer

# Make file public (anyone with link)
workspace-cli drive share <file-id> --anyone --role reader

# List file permissions
workspace-cli drive permissions <file-id>

# Remove a permission
workspace-cli drive unshare <file-id> <permission-id>
```

### Calendar Examples

```bash
# List today's events (minimal by default - id, summary, start, end, status)
workspace-cli calendar list --time-min "2024-01-01T00:00:00Z"

# List events with full details (attendees, organizer, description, etc.)
workspace-cli calendar list --time-min "2024-01-01T00:00:00Z" --full

# List events in a date range
workspace-cli calendar list \
  --time-min "2024-01-01T00:00:00Z" \
  --time-max "2024-01-31T23:59:59Z" \
  --limit 50

# Create an event
workspace-cli calendar create \
  --summary "Team Meeting" \
  --start "2024-01-15T10:00:00Z" \
  --end "2024-01-15T11:00:00Z" \
  --description "Quarterly planning session"

# Update an event
workspace-cli calendar update <event-id> \
  --summary "Updated Meeting Title" \
  --start "2024-01-15T14:00:00Z"

# Delete an event
workspace-cli calendar delete <event-id>

# Use sync token for incremental updates
workspace-cli calendar list --sync-token <token>
```

### Docs Examples

```bash
# Get document content as Markdown
workspace-cli docs get <doc-id> --markdown

# Get document content as plain text (optimized for AI agents)
workspace-cli docs get <doc-id> --text

# Get raw document JSON
workspace-cli docs get <doc-id>

# Create a new document
workspace-cli docs create "My New Document"

# Append text to document
workspace-cli docs append <doc-id> "New paragraph content"

# Find and replace text in document
workspace-cli docs replace <doc-id> --find "old text" --with "new text"

# Case-sensitive find and replace
workspace-cli docs replace <doc-id> --find "OldText" --with "NewText" --match-case

# Apply rich formatting via the Google Docs batchUpdate API
workspace-cli docs batch-update <doc-id> --payload '{"requests":[
  {"insertText":{"location":{"index":1},"text":"Title\n"}},
  {"updateParagraphStyle":{"range":{"startIndex":1,"endIndex":6},"paragraphStyle":{"namedStyleType":"HEADING_1"},"fields":"namedStyleType"}},
  {"createParagraphBullets":{"range":{"startIndex":8,"endIndex":20},"bulletPreset":"BULLET_DISC_CIRCLE_SQUARE"}}
]}'

# Load batchUpdate requests from a JSON file
workspace-cli docs batch-update <doc-id> --file requests.json
```

### Sheets Examples

```bash
# Create a new spreadsheet
workspace-cli sheets create "My Spreadsheet"

# Read spreadsheet range (returns values array by default)
workspace-cli sheets get <sheet-id> --range "Sheet1!A1:C10"

# Read with full range metadata wrapper
workspace-cli sheets get <sheet-id> --range "Sheet1!A1:C10" --full

# Read as CSV format
workspace-cli sheets get <sheet-id> --range "Sheet1!A1:C10" --format csv

# Update spreadsheet values
workspace-cli sheets update <sheet-id> \
  --range "Sheet1!A1:B2" \
  --values '[["Name","Age"],["Alice","30"]]'

# Append rows to spreadsheet
workspace-cli sheets append <sheet-id> \
  --range "Sheet1!A1" \
  --values '[["Bob","25"],["Carol","28"]]'

# Clear a range of cells
workspace-cli sheets clear <sheet-id> --range "Sheet1!A1:C10"
```

### Slides Examples

```bash
# Get presentation text content (default - optimized for AI agents)
workspace-cli slides get <presentation-id>

# Get full presentation structure (masters, layouts, transforms, colors)
workspace-cli slides get <presentation-id> --full

# Get specific page text content
workspace-cli slides page <presentation-id> --page 0

# Get specific page with full structure
workspace-cli slides page <presentation-id> --page 0 --full

# Create a new presentation
workspace-cli slides create "Quarterly Review"

# Add a slide (append by default, or specify index and layout)
workspace-cli slides add-slide <presentation-id>
workspace-cli slides add-slide <presentation-id> --index 1 --layout TITLE_AND_BODY

# Add a shape with text and styling
workspace-cli slides add-shape <presentation-id> --slide <slide-id> \
  --type RECTANGLE --text "Hello World" \
  --x 100 --y 50 --width 400 --height 80 \
  --fill "#3366CC" --font-size 24 --bold

# Add a text box
workspace-cli slides add-shape <presentation-id> --slide <slide-id> \
  --type TEXT_BOX --text "Some content" \
  --x 50 --y 200 --width 600 --height 40

# Add a table with data and header color
workspace-cli slides add-table <presentation-id> --slide <slide-id> \
  --rows 3 --cols 2 \
  --data '[["Name","Role"],["Alice","Engineer"],["Bob","Designer"]]' \
  --header-color "#333333"

# Embed a Google Sheets chart (linked for auto-updates)
workspace-cli slides add-chart <presentation-id> --slide <slide-id> \
  --spreadsheet <spreadsheet-id> --chart-id 12345 --linked

# Delete a slide or page element
workspace-cli slides delete <presentation-id> <object-id>

# Raw batchUpdate for advanced operations
workspace-cli slides batch-update <presentation-id> \
  --requests '[{"createSlide":{"slideLayoutReference":{"predefinedLayout":"BLANK"}}}]'
```

### Tasks Examples

```bash
# List all task lists
workspace-cli tasks lists

# List tasks in default list (minimal by default - id, title, status, due, notes)
workspace-cli tasks list

# List tasks with full metadata (etag, selfLink, links, parent, position, etc.)
workspace-cli tasks list --full

# List tasks in specific list
workspace-cli tasks list --list <list-id>

# Include completed tasks
workspace-cli tasks list --show-completed

# Create a task
workspace-cli tasks create "Finish report" \
  --due "2024-01-15T17:00:00Z" \
  --notes "Include Q4 metrics"

# Update a task
workspace-cli tasks update <task-id> --title "Updated task title"

# Mark task as complete
workspace-cli tasks update <task-id> --complete

# Delete a task
workspace-cli tasks delete <task-id>
```

### Chat Examples

```bash
# List all Chat spaces
workspace-cli chat spaces-list

# List only DM spaces
workspace-cli chat spaces-list --type DIRECT_MESSAGE

# Find spaces by name
workspace-cli chat spaces-find --query "Project Alpha"

# Find a DM space by email
workspace-cli chat find-dm --email user@company.com

# List messages in a space
workspace-cli chat messages-list --space spaces/abc123

# List today's messages
workspace-cli chat messages-list --space spaces/abc123 --today

# Get a specific message
workspace-cli chat get spaces/abc123/messages/msg456

# Check unread messages across all spaces
workspace-cli chat unread

# Get read state for a space
workspace-cli chat read-state --space spaces/abc123

# Send a message
workspace-cli chat send --space spaces/abc123 --text 'Hello team'

# Mark a space as read
workspace-cli chat mark-read --space spaces/abc123
```

### Contacts Examples

```bash
# List contacts
workspace-cli contacts list --limit 20

# Search contacts
workspace-cli contacts search --query "John"

# Get a specific contact
workspace-cli contacts get people/c123456

# List workspace directory
workspace-cli contacts directory-list

# Search workspace directory
workspace-cli contacts directory-search --query "Jane"
```

### Groups Examples

```bash
# List groups for a user
workspace-cli groups list --email user@company.com

# List all groups in a domain
workspace-cli groups list --domain company.com

# List group members
workspace-cli groups members group@company.com
```

### Batch Examples

Execute multiple API requests in a single HTTP call for maximum efficiency:

```bash
# Batch Gmail requests - get multiple messages at once
workspace-cli batch gmail --requests '[
  {"id":"msg1","method":"GET","path":"/gmail/v1/users/me/messages/abc123"},
  {"id":"msg2","method":"GET","path":"/gmail/v1/users/me/messages/def456"}
]'

# Batch Gmail requests - modify multiple messages
workspace-cli batch gmail --requests '[
  {"id":"star1","method":"POST","path":"/gmail/v1/users/me/messages/abc123/modify","body":{"addLabelIds":["STARRED"]}},
  {"id":"star2","method":"POST","path":"/gmail/v1/users/me/messages/def456/modify","body":{"addLabelIds":["STARRED"]}}
]'

# Batch Drive requests - get metadata for multiple files
workspace-cli batch drive --requests '[
  {"id":"file1","method":"GET","path":"/drive/v3/files/abc123"},
  {"id":"file2","method":"GET","path":"/drive/v3/files/def456"}
]'

# Batch Calendar requests - delete multiple events
workspace-cli batch calendar --requests '[
  {"id":"del1","method":"DELETE","path":"/calendar/v3/calendars/primary/events/evt1"},
  {"id":"del2","method":"DELETE","path":"/calendar/v3/calendars/primary/events/evt2"}
]'

# Read requests from a JSON file
workspace-cli batch gmail --file batch_requests.json

# Pipe requests from stdin
echo '[{"id":"1","method":"GET","path":"/gmail/v1/users/me/messages/abc"}]' | workspace-cli batch gmail
```

Batch output format:
```json
{
  "status": "success",
  "results": [
    {"id": "msg1", "status": 200, "body": {...}},
    {"id": "msg2", "status": 200, "body": {...}}
  ],
  "errors": []
}
```

Status values:
- `success`: All requests succeeded
- `partial`: Some requests succeeded, some failed
- `error`: All requests failed

## Output Formats

Control output format with the `--format` flag:

### TOON (Default)
Token-Oriented Object Notation — ~50-60% fewer tokens than JSON, ideal for LLM consumption:
```bash
workspace-cli gmail list --limit 2
```

### JSON
Pretty-printed JSON for human readability:
```bash
workspace-cli gmail list --limit 2 --format json
```

Output:
```json
{
  "messages": [
    {
      "id": "18c1a2b3c4d5e6f7",
      "threadId": "18c1a2b3c4d5e6f7",
      "subject": "Weekly team sync notes",
      "from": "Alice <alice@company.com>",
      "date": "Mon, 23 Dec 2025 10:30:00 +0000",
      "snippet": "Here are the notes from today's meeting..."
    }
  ],
  "resultSizeEstimate": 150
}
```

### JSON Compact
Compact JSON without whitespace:
```bash
workspace-cli gmail list --limit 2 --format json-compact
```

### JSONL (Newline-Delimited JSON)
Stream-friendly format for processing large datasets:
```bash
workspace-cli gmail list --limit 100 --format jsonl
```

Output:
```jsonl
{"id":"18c1a2b3c4d5e6f7","threadId":"18c1a2b3c4d5e6f7","subject":"Weekly sync","from":"Alice <alice@company.com>","date":"Mon, 23 Dec 2025 10:30:00 +0000","snippet":"Meeting notes..."}
{"id":"28d1a2b3c4d5e6f8","threadId":"28d1a2b3c4d5e6f8","subject":"Project update","from":"Bob <bob@company.com>","date":"Mon, 23 Dec 2025 09:15:00 +0000","snippet":"Latest status..."}
```

Ideal for piping to tools like `jq`:
```bash
workspace-cli gmail list --format jsonl | jq -r '.id'
```

### CSV
Comma-separated values for spreadsheet import:
```bash
workspace-cli drive list --limit 10 --format csv > files.csv
```

## Global Options

### Field Selection
Reduce response size and token costs by selecting specific fields:

```bash
# Only get ID and name
workspace-cli drive list --fields "id,name" --limit 10

# Multiple fields
workspace-cli gmail list --fields "id,threadId,snippet" --limit 5
```

### Output to File
Save results to a file instead of stdout:

```bash
workspace-cli gmail list --limit 100 --output emails.json

workspace-cli drive list --format csv --output files.csv
```

### Quiet Mode
Suppress non-essential output:

```bash
workspace-cli gmail send --to user@example.com --subject "Test" --body "Hello" --quiet
```

## Command Reference

### Gmail Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `gmail list` | List messages | `--query`, `--limit`, `--label` |
| `gmail get` | Get a specific message | `--full` (minimal by default) |
| `gmail send` | Send an email | `--to`, `--subject`, `--body`, `--body-file` |
| `gmail draft` | Create a draft | `--to`, `--subject`, `--body` |
| `gmail delete` | Permanently delete message | None |
| `gmail trash` | Move message to trash | None |
| `gmail untrash` | Restore message from trash | None |
| `gmail labels` | List all labels | None |
| `gmail modify` | Modify message labels | `--add-labels`, `--remove-labels`, `--mark-read`, `--mark-unread`, `--star`, `--unstar`, `--archive` |

### Drive Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `drive list` | List files | `--query`, `--limit`, `--parent` |
| `drive get` | Get file metadata | None |
| `drive upload` | Upload a file | `--parent`, `--name` |
| `drive download` | Download a file | `--output` |
| `drive delete` | Permanently delete file | None |
| `drive trash` | Move file to trash | None |
| `drive untrash` | Restore file from trash | None |
| `drive mkdir` | Create a folder | `--parent` |
| `drive move` | Move file to folder | `--to` |
| `drive copy` | Copy a file | `--name`, `--parent` |
| `drive rename` | Rename a file | None |
| `drive share` | Share a file | `--email`, `--anyone`, `--role` |
| `drive permissions` | List file permissions | None |
| `drive unshare` | Remove a permission | None |

### Calendar Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `calendar list` | List events | `--calendar`, `--time-min`, `--time-max`, `--sync-token`, `--full` |
| `calendar create` | Create an event | `--summary`, `--start`, `--end`, `--description` |
| `calendar update` | Update an event | `--summary`, `--start`, `--end` |
| `calendar delete` | Delete an event | None |

### Docs Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `docs get` | Get document content | `--markdown`, `--text` |
| `docs create` | Create a new document | None |
| `docs append` | Append text to document | None |
| `docs replace` | Find and replace text | `--find`, `--with`, `--match-case` |
| `docs batch-update` | Apply batchUpdate requests (headings, bold, bullets, etc.) | `--payload`, `--file` |

### Sheets Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `sheets get` | Get spreadsheet values | `--range`, `--full` |
| `sheets create` | Create a new spreadsheet | `--sheets` |
| `sheets update` | Update spreadsheet values | `--range`, `--values` |
| `sheets append` | Append rows to spreadsheet | `--range`, `--values` |
| `sheets clear` | Clear a range of cells | `--range` |

### Slides Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `slides get` | Get presentation text | `--full` (text by default) |
| `slides page` | Get specific page text | `--page`, `--full` |
| `slides create` | Create a new presentation | None |
| `slides add-slide` | Add a slide | `--index`, `--layout`, `--object-id` |
| `slides add-shape` | Add a shape to a slide | `--slide`, `--type`, `--text`, `--x/y/width/height`, `--fill`, `--font-size`, `--bold` |
| `slides add-table` | Add a table to a slide | `--slide`, `--rows`, `--cols`, `--data`, `--header-color` |
| `slides add-chart` | Embed a Sheets chart | `--slide`, `--spreadsheet`, `--chart-id`, `--linked`, `--x/y/width/height` |
| `slides delete` | Delete a slide or element | None |
| `slides batch-update` | Raw batchUpdate passthrough | `--requests`, `--file` |

### Tasks Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `tasks lists` | List task lists | None |
| `tasks list` | List tasks | `--list`, `--show-completed`, `--full` |
| `tasks create` | Create a task | `--list`, `--due`, `--notes` |
| `tasks update` | Update a task | `--list`, `--title`, `--complete` |
| `tasks delete` | Delete a task | `--list` |

### Chat Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `chat spaces-list` | List Chat spaces | `--type` |
| `chat spaces-find` | Find spaces by name | `--query` |
| `chat find-dm` | Find DM space by email | `--email` |
| `chat messages-list` | List messages in a space | `--space`, `--today` |
| `chat get` | Get a specific message | None |
| `chat unread` | Show unread messages | `--since`, `--include-muted` |
| `chat read-state` | Get read state for a space | `--space` |
| `chat thread-read-state` | Get read state for a thread | `--space`, `--thread` |
| `chat mark-read` | Mark a space as read | `--space` |
| `chat send` | Send a message | `--space`, `--text` |

### Contacts Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `contacts list` | List contacts | `--limit` |
| `contacts search` | Search contacts | `--query` |
| `contacts get` | Get a specific contact | None |
| `contacts create` | Create a contact | `--given-name`, `--email`, `--phone` |
| `contacts delete` | Delete a contact | None |
| `contacts directory-list` | List workspace directory | `--limit` |
| `contacts directory-search` | Search workspace directory | `--query` |

### Groups Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `groups list` | List groups | `--email`, `--domain` |
| `groups members` | List group members | None |

### Batch Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `batch gmail` | Execute batch Gmail API requests | `--requests`, `--file` |
| `batch drive` | Execute batch Drive API requests | `--requests`, `--file` |
| `batch calendar` | Execute batch Calendar API requests | `--requests`, `--file` |

### Auth Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `auth login` | Login with OAuth2 | `--credentials` |
| `auth logout` | Logout and clear tokens | None |
| `auth status` | Show authentication status | None |
| `auth export` | Export stored credentials | `--unmasked`, `--output` |

### MCP Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `mcp` | Start stdio MCP server (~50 tools) | None |

## Environment Variables

Configure workspace-cli behavior via environment variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `WORKSPACE_CREDENTIALS_PATH` | Path to OAuth credentials JSON | `/path/to/credentials.json` |
| `GOOGLE_APPLICATION_CREDENTIALS` | Path to service account key JSON | `/path/to/service-account.json` |
| `WORKSPACE_OUTPUT_FORMAT` | Default output format | `toon`, `json`, `jsonl`, `csv` |
| `WORKSPACE_IMPERSONATE` | Email to impersonate via domain-wide delegation | `user@company.com` |
| `RUST_LOG` | Logging level | `debug`, `info`, `warn`, `error` |

Example usage:
```bash
export WORKSPACE_OUTPUT_FORMAT=jsonl
export RUST_LOG=info
workspace-cli gmail list
```

## Configuration File

Create a config file at `~/.config/workspace-cli/config.toml`:

```toml
[auth]
credentials_path = "/path/to/credentials.json"

[output]
format = "json"
compact = false

[api]
timeout_seconds = 30
max_retries = 3
```

## Error Handling

All errors are returned as structured JSON for easy parsing by scripts and AI agents:

### Example Error Response

```json
{
  "status": "error",
  "error_code": "rate_limit_exceeded",
  "domain": "gmail",
  "message": "User rate limit exceeded. Retry after 45 seconds.",
  "retry_after_seconds": 45,
  "actionable_fix": "Wait 45 seconds and retry with a smaller batch size."
}
```

### Error Codes

| Error Code | Description | Common Fix |
|------------|-------------|------------|
| `authentication_failed` | OAuth token invalid or expired | Run `workspace-cli auth login` |
| `token_expired` | Access token expired | Automatic refresh, or re-login |
| `rate_limit_exceeded` | API rate limit hit | Wait for `retry_after_seconds` |
| `quota_exceeded` | Daily quota exhausted | Wait until quota resets |
| `not_found` | Resource not found | Verify ID is correct |
| `permission_denied` | Insufficient permissions | Check OAuth scopes or share settings |
| `invalid_request` | Malformed request | Check command syntax |
| `network_error` | Network connectivity issue | Check internet connection |
| `server_error` | Google API server error | Retry after a delay |

### Debugging

Enable debug logging to troubleshoot issues:

```bash
RUST_LOG=debug workspace-cli gmail list --limit 1
```

## Performance Optimization

### Token Efficiency (Minimal by Default)
All commands are optimized for AI agents with minimal output by default:

```bash
# Gmail get - minimal by default (headers + plain text body, ~88% reduction)
workspace-cli gmail get <id>           # Minimal (default)
workspace-cli gmail get <id> --full    # Full message structure

# Gmail send/reply/modify - minimal responses (~90-99% reduction)
workspace-cli gmail send --to user@example.com --subject "Hi" --body "Hello"
# Returns: {"success":true,"id":"...","threadId":"..."}

# Calendar list - minimal events by default (~50% reduction)
workspace-cli calendar list --time-min "2024-01-01T00:00:00Z"
workspace-cli calendar list --time-min "2024-01-01T00:00:00Z" --full

# Sheets get - values array by default (~50% reduction)
workspace-cli sheets get <id> --range "Sheet1!A1:C10"
workspace-cli sheets get <id> --range "Sheet1!A1:C10" --full

# Slides get - text extraction by default (~93% reduction)
workspace-cli slides get <id>          # Text only
workspace-cli slides get <id> --full   # Full structure

# Tasks list - minimal fields by default (~40% reduction)
workspace-cli tasks list
workspace-cli tasks list --full

# Use field masking for additional filtering
workspace-cli gmail list --fields "id,snippet" --limit 100

# Use JSONL for streaming large results
workspace-cli drive list --format jsonl --limit 1000 | jq -r '.name'
```

### Batch Operations
Process multiple items efficiently:

```bash
# Stream emails and process in parallel
workspace-cli gmail list --format jsonl --limit 100 | \
  parallel -j 10 "workspace-cli gmail get {}"
```

### Rate Limiting
The CLI automatically handles rate limiting with exponential backoff and respects `retry_after_seconds` from error responses.

## Advanced Usage

### Piping and Chaining

```bash
# Get email IDs and fetch each message
workspace-cli gmail list --format jsonl --limit 10 | \
  jq -r '.messages[].id' | \
  xargs -I {} workspace-cli gmail get {}

# Upload multiple files
find ./documents -name "*.pdf" | \
  xargs -I {} workspace-cli drive upload {}
```

### Integration with jq

```bash
# Extract specific fields (gmail list includes subject, from, date by default)
workspace-cli gmail list --format json | jq '.messages[] | {id, subject, from}'

# Count unread emails
workspace-cli gmail list --query "is:unread" --format json | jq '.resultSizeEstimate'
```

### Cron Jobs

```bash
# Daily backup of calendar events
0 2 * * * workspace-cli calendar list \
  --time-min "$(date -u +%Y-%m-%dT00:00:00Z)" \
  --format json \
  --output ~/backups/calendar-$(date +%Y%m%d).json
```

## Security Best Practices

1. **Never commit credentials**: Add `credentials.json` and service account keys to `.gitignore`
2. **Use minimal OAuth scopes**: Only request scopes your application needs
3. **Rotate service account keys**: Regularly rotate keys used in production
4. **Use keyring storage**: OAuth tokens are stored securely in OS keyring by default
5. **Audit access**: Regularly review OAuth consent at [Google Account Permissions](https://myaccount.google.com/permissions)

## Troubleshooting

### Authentication Issues

**Problem**: `authentication_failed` error
```bash
workspace-cli auth status
workspace-cli auth logout
workspace-cli auth login --credentials credentials.json
```

**Problem**: Keyring access denied on Linux
```bash
# Install gnome-keyring or use environment variable
export WORKSPACE_CREDENTIALS_PATH=/secure/path/credentials.json
```

### API Quota Issues

**Problem**: `quota_exceeded` error

- Check your quota usage in [Google Cloud Console](https://console.cloud.google.com/apis/dashboard)
- Consider requesting quota increase
- Use field masking to reduce API calls

### Network Issues

**Problem**: `network_error` or timeouts

```bash
# Increase timeout (default: 30s)
# Edit ~/.config/workspace-cli/config.toml
[api]
timeout_seconds = 60
```

## Development

### Building from Source

```bash
# Debug build
cargo build

# Release build with optimizations
cargo build --release

# Run tests
cargo test

# Check for issues
cargo clippy
```

### Project Structure

```
workspace-cli/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Library exports
│   ├── cli.rs            # CLI context and utilities
│   ├── auth/             # OAuth & token management
│   ├── client/           # API client, retry, rate limiting
│   ├── commands/         # Service-specific commands
│   │   ├── gmail/
│   │   ├── drive/
│   │   ├── calendar/
│   │   ├── docs/
│   │   ├── sheets/
│   │   ├── slides/
│   │   └── tasks/
│   ├── config/           # Configuration management
│   ├── error/            # Error types and handling
│   ├── output/           # Output formatting (JSON/CSV/JSONL)
│   └── utils/            # Helper utilities
├── Cargo.toml            # Dependencies and metadata
└── README.md             # This file
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

## License

MIT License - see LICENSE file for details

## Support

For issues, questions, or contributions:
- GitHub Issues: [Report a bug or request a feature]
- Documentation: This README
- Google Workspace API Docs: [developers.google.com/workspace](https://developers.google.com/workspace)

## Roadmap

- [x] ~~Implement remaining commands~~ (All core commands implemented!)
- [x] ~~Extended field filtering~~ (`--fields` flag for JSON field selection)
- [x] ~~Batch operations for bulk processing~~ (`batch gmail/drive/calendar` commands)
- [x] ~~Google Docs batchUpdate support~~ (`docs batch-update` for rich document formatting)
- [x] Auto-pagination (`--page-all`, `--page-limit`, `--page-delay`)
- [x] Dry-run mode (`--dry-run`)
- [x] Auth export (`auth export --unmasked`)
- [x] MCP server (`workspace-cli mcp`, ~50 tools)
- [x] Slides write support (create, shapes, tables, charts, delete, batchUpdate)
- [ ] Webhook support for real-time notifications
- [ ] Performance benchmarks and optimizations

---

Built with Rust for speed, security, and reliability.
