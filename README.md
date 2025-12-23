# workspace-cli

High-performance Google Workspace CLI optimized for AI agent integration.

## Overview

`workspace-cli` is a Rust-based command-line tool designed to provide programmatic access to Google Workspace APIs with structured JSON output optimized for AI agent consumption. Built for speed, efficiency, and deterministic execution.

## Features

- **Gmail**: List, read, send, and draft emails with advanced filtering
- **Drive**: List, upload, download, and manage files
- **Calendar**: List, create, update, and delete events with sync token support
- **Docs**: Read documents as Markdown and append content
- **Sheets**: Read and write spreadsheet data with A1 notation support
- **Slides**: Extract text from presentations
- **Tasks**: Manage task lists and individual tasks

### Key Capabilities

- **Structured Output**: All responses in JSON/JSONL/CSV formats
- **Field Masking**: Reduce token costs by selecting only needed fields
- **Rate Limiting**: Built-in retry logic with exponential backoff
- **Streaming**: JSONL output for real-time processing of paginated results
- **Secure Auth**: OS keyring integration for token storage
- **Error Handling**: Structured, actionable error messages with retry guidance

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

# Get a specific message with decoded body
workspace-cli gmail get <message-id> --decode-body

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
```

### Drive Examples

```bash
# List all files
workspace-cli drive list --limit 10

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
```

### Calendar Examples

```bash
# List today's events
workspace-cli calendar list --time-min "2024-01-01T00:00:00Z"

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

# Get raw document JSON
workspace-cli docs get <doc-id>

# Append text to document
workspace-cli docs append <doc-id> "New paragraph content"
```

### Sheets Examples

```bash
# Read spreadsheet range
workspace-cli sheets get <sheet-id> --range "Sheet1!A1:C10"

# Update spreadsheet values
workspace-cli sheets update <sheet-id> \
  --range "Sheet1!A1:B2" \
  --values '[["Name","Age"],["Alice","30"]]'

# Append rows to spreadsheet
workspace-cli sheets append <sheet-id> \
  --range "Sheet1!A1" \
  --values '[["Bob","25"],["Carol","28"]]'
```

### Slides Examples

```bash
# Get presentation info
workspace-cli slides get <presentation-id>

# Get text only
workspace-cli slides get <presentation-id> --text-only

# Get specific page
workspace-cli slides page <presentation-id> --page 0
```

### Tasks Examples

```bash
# List all task lists
workspace-cli tasks lists

# List tasks in default list
workspace-cli tasks list

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

## Output Formats

Control output format with the `--format` flag:

### JSON (Default)
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
      "threadId": "18c1a2b3c4d5e6f7"
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
{"id":"18c1a2b3c4d5e6f7","threadId":"18c1a2b3c4d5e6f7"}
{"id":"28d1a2b3c4d5e6f8","threadId":"28d1a2b3c4d5e6f8"}
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
| `gmail get` | Get a specific message | `--decode-body` |
| `gmail send` | Send an email | `--to`, `--subject`, `--body`, `--body-file` |
| `gmail draft` | Create a draft | `--to`, `--subject`, `--body` |

### Drive Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `drive list` | List files | `--query`, `--limit`, `--parent` |
| `drive get` | Get file metadata | None |
| `drive upload` | Upload a file | `--parent`, `--name` |
| `drive download` | Download a file | `--output` |

### Calendar Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `calendar list` | List events | `--calendar`, `--time-min`, `--time-max`, `--sync-token` |
| `calendar create` | Create an event | `--summary`, `--start`, `--end`, `--description` |
| `calendar update` | Update an event | `--summary`, `--start`, `--end` |
| `calendar delete` | Delete an event | None |

### Docs Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `docs get` | Get document content | `--markdown` |
| `docs append` | Append text to document | None |

### Sheets Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `sheets get` | Get spreadsheet values | `--range` |
| `sheets update` | Update spreadsheet values | `--range`, `--values` |
| `sheets append` | Append rows to spreadsheet | `--range`, `--values` |

### Slides Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `slides get` | Get presentation info | `--text-only` |
| `slides page` | Get specific page | `--page`, `--text-only` |

### Tasks Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `tasks lists` | List task lists | None |
| `tasks list` | List tasks | `--list`, `--show-completed` |
| `tasks create` | Create a task | `--list`, `--due`, `--notes` |
| `tasks update` | Update a task | `--list`, `--title`, `--complete` |
| `tasks delete` | Delete a task | `--list` |

### Auth Commands

| Command | Description | Key Options |
|---------|-------------|-------------|
| `auth login` | Login with OAuth2 | `--credentials` |
| `auth logout` | Logout and clear tokens | None |
| `auth status` | Show authentication status | None |

## Environment Variables

Configure workspace-cli behavior via environment variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `WORKSPACE_CREDENTIALS_PATH` | Path to OAuth credentials JSON | `/path/to/credentials.json` |
| `GOOGLE_APPLICATION_CREDENTIALS` | Path to service account key JSON | `/path/to/service-account.json` |
| `WORKSPACE_OUTPUT_FORMAT` | Default output format | `json`, `jsonl`, `csv` |
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

### Token Efficiency
Minimize token costs for AI agents:

```bash
# Use field masking to get only needed fields
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
# Extract specific fields
workspace-cli gmail list --format json | jq '.messages[] | {id, threadId}'

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

- [ ] Implement remaining commands (Drive upload/download, Docs append, etc.)
- [ ] Model Context Protocol (MCP) server mode
- [ ] Batch operations for bulk processing
- [ ] Webhook support for real-time notifications
- [ ] Extended field filtering and transformations
- [ ] Performance benchmarks and optimizations

---

Built with Rust for speed, security, and reliability.
