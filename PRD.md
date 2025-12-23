# **Technical Product Requirements Document: Rust-Based Google Workspace CLI for AI Agent Integration**

## **1\. Executive Summary and Strategic Alignment**

The paradigm of software interaction is undergoing a tectonic shift from Human-Computer Interaction (HCI) to Agent-Computer Interaction (ACI). Traditional Command Line Interfaces (CLIs) are optimized for human readability, interactive discovery, and brevity. However, the emergence of autonomous AI agents—such as Claude Code, Gemini CLI, and other Large Language Model (LLM) driven systems—imposes a fundamentally different set of requirements. Agents require deterministic outputs, strictly typed structured data, explicit error handling, and, crucially, high-efficiency context management to operate within finite token windows.

This document serves as a comprehensive Deep Research Report and Technical Product Requirements Document (PRD) for workspace-cli, a high-performance, Rust-based utility designed to bridge the gap between AI agents and the Google Workspace ecosystem (Gmail, Drive, Calendar, Docs, Sheets, Slides, Tasks).

### **1.1 The Agent-First Design Philosophy**

Unlike existing wrappers which prioritize developer convenience, workspace-cli is architected as a "tool-use primitive" for LLMs. The research indicates that agents struggle with ambiguity. When an API returns a 400 error, a human developer investigates; an agent enters a hallucination loop unless the error is structured and actionable.1 Therefore, this tool acts as a translation layer, converting the heterogeneous, complex, and rate-limited Google Workspace APIs into a unified, type-safe, and token-efficient interface compliant with the Model Context Protocol (MCP).2

### **1.2 The Strategic Choice of Rust**

The requirement for the tool to be "extremely fast and efficient" mandates the use of Rust. The research highlights several critical advantages of Rust in this context:

* **Zero-Cost Abstractions:** Rust’s async/await model, powered by the tokio runtime, allows for massive concurrency (e.g., uploading 100 files to Drive simultaneously) with negligible memory overhead, a significant improvement over Python or Node.js equivalents.4  
* **Type Safety as Prompt Engineering:** The google-apis-rs crate provides strongly typed structures for every Google API resource. By exposing these types directly to the agent via JSON Schema, we enforce a contract that virtually eliminates "parameter hallucination," where an agent invents API fields that do not exist.4  
* **Binary Portability:** A single, statically linked binary simplifies deployment into ephemeral agent environments (e.g., CI/CD pipelines, secure containers) without the "dependency hell" associated with Python pip or Node node\_modules.5

## **2\. Core Architecture and Technology Stack**

To satisfy the performance and integration requirements, the system architecture deviates from standard CLI patterns. It adopts a dual-mode operation: a direct execution mode for piping and scripts, and a daemonized Server Mode implementing the Model Context Protocol (MCP) for persistent agent sessions.

### **2.1 Technology Selection Justification**

| Component | Technology | Research Justification |
| :---- | :---- | :---- |
| **Language** | Rust (2021 Edition) | Required for memory safety, speed, and binary size optimization.5 |
| **Async Runtime** | tokio (v1) | The industry standard for Rust async I/O; required for google-apis-rs and reqwest dependency trees.6 |
| **HTTP Client** | reqwest (v0.11+) | preferred over raw hyper for ergonomics while maintaining HTTP/2 multiplexing support, essential for batch operations.7 |
| **CLI Parser** | clap (v4) | Supports "derive" macros to auto-generate help text, which serves as the "System Prompt" for the agent to learn tool usage.8 |
| **Auth** | yup-oauth2 | The standard for Google OAuth2 in Rust, supporting Service Account and Installed Flow (PKCE).9 |
| **Secrets** | keyring | Cross-platform integration with macOS Keychain, Windows Credential Locker, and Linux Secret Service for secure token storage.10 |
| **Google SDK** | google-apis-rs | Auto-generated, strongly typed Rust bindings for all Workspace APIs, ensuring full schema compliance.4 |
| **Serialization** | serde / serde\_json | Zero-copy serialization is critical for processing large JSON payloads (e.g., Drive file lists) without GC pauses.4 |

### **2.2 Model Context Protocol (MCP) Implementation**

The tool will implement the MCP specification to standardize tool discovery. When running in server mode (workspace-cli server), it communicates via stdio using JSON-RPC 2.0. This allows agents like Claude Desktop or Gemini Code Assist to dynamically discover capabilities (e.g., gmail\_list\_messages, drive\_upload\_file) without hardcoded integration logic.3

The Rust implementation will utilize the mcp-sdk-rs (or equivalent community crate structure) to map internal Rust functions to MCP "Tools" and "Resources." A critical insight from the research is that MCP allows for "Resources" to be exposed directly.12 workspace-cli will map Google Drive files to MCP Resources (URI gdrive://{id}), allowing the agent to "read" a file as if it were local, handling the API fetching and content extraction transparently.

## **3\. Authentication and Security Module**

Security in agentic workflows is precarious; agents often run in background processes where interactive login prompts are blocking failures. The PRD mandates a robust, tiered authentication strategy.

### **3.1 OAuth 2.0 Flow Strategy**

The CLI must support the "Installed Application" flow using PKCE (Proof Key for Code Exchange) for user-attended sessions, and Service Account impersonation for headless server environments.9

* **Tier 1: Interactive Login:** The yup-oauth2 crate handles the browser interaction. Upon successful exchange, the refresh token is **not** stored in a plaintext file (a common security flaw). Instead, the TokenStorage trait must be implemented to interface with the OS keyring. This ensures that malicious agents or processes scanning the filesystem cannot exfiltrate credentials.14  
* **Tier 2: Headless/CI:** The tool must accept a base64-encoded service account key via the GOOGLE\_CREDENTIALS environment variable or a standard GOOGLE\_APPLICATION\_CREDENTIALS file path. This is essential for Gemini CLI workflows running in cloud shells or CI runners.9

### **3.2 Token Storage Implementation**

The research identifies keyring as the optimal crate. The implementation will define a struct KeyringStorage implementing yup\_oauth2::TokenStorage.

* **Set:** Serializes the TokenInfo struct to JSON and writes it to the platform's secure store under the service name workspace-cli.  
* **Get:** Retrieves and deserializes the token.  
* **Error Handling:** If the keyring is locked or unavailable (common in headless Linux), it must gracefully degrade to an encrypted file storage or memory-only storage (for single-session agents).10

## **4\. Universal Design Constraints for Agent Optimization**

Before detailing specific APIs, we must establish the cross-cutting requirements that apply to all interactions. These are derived from the constraints of LLM context windows and the need for deterministic execution.

### **4.1 Field Masking (The "Context Economy")**

Agents paying for tokens per request cannot afford full JSON dumps. A generic files.list call in Drive returns 30+ fields per file. For 100 files, this consumes thousands of tokens unnecessarily.

* **Requirement:** Every list and get command must support a \--fields flag.  
* **Default Behavior:** If no fields are specified, the CLI defaults to a "Context-Optimized" set (e.g., id, name, mimeType, webViewLink).  
* **Mechanism:** This maps directly to the Google API fields parameter (field mask), reducing network latency and token costs by up to 90%.16

### **4.2 Structured Error Handling**

Standard HTTP 400/500 errors are insufficient. The CLI must catch reqwest errors and mapped Google JSON errors, outputting a standardized error schema:

JSON

{  
  "status": "error",  
  "error\_code": "rate\_limit\_exceeded",  
  "domain": "gmail",  
  "message": "User rate limit exceeded.",  
  "retry\_after\_seconds": 45,  
  "actionable\_fix": "Wait 45 seconds and retry with a smaller batch size."  
}

This schema allows the agent to parse retry\_after\_seconds and autonomously sleep/retry without human intervention.1

### **4.3 Pagination as a Stream**

To support "extremely fast" execution, the CLI must not wait for all pages to load. It must implement a PaginationStream using Rust's futures::stream.

* **Logic:** The stream yields items to stdout (in JSONL format) as soon as a page arrives.  
* **Next Page Token:** The next\_page\_token handling is encapsulated within the iterator. If the agent pipes the output to another tool, processing begins immediately (Time to First Byte optimization).18

## **5\. Deep Dive: Communication APIs (Gmail & Calendar)**

### **5.1 Gmail API**

The Gmail API is high-volume and data-intensive. The primary challenge is the verbose JSON structure and base64 encoding of email bodies.

#### **5.1.1 Resource Analysis**

* **Message Payload:** The API returns bodies in payload.body.data as Base64Url strings.  
* **Agent Friction:** An agent reading 10 emails would waste massive context on base64 strings it cannot natively read efficiently.  
* **CLI Requirement:** The CLI must perform transparent Base64Url decoding. It should present the body as decoded UTF-8 text. If the email is HTML, it should optionally transcode to Markdown (using a crate like html2text) to save tokens while preserving semantic structure.20

#### **5.1.2 Rate Limiting Strategy**

Gmail enforces a per-user rate limit of 250 quota units/second. messages.get costs 5 units; messages.send costs 100\.21

* **Throttling:** The CLI must implement a client-side token bucket limiter initialized with these values to prevent 429 errors during bulk fetch operations.  
* **Batching:** For operations like "Mark 50 emails as read" (batchModify), the CLI must leverage the batch endpoint to group requests, minimizing HTTP overhead.21

#### **5.1.3 Command Specification (Draft)**

* gmail list \--query "is:unread" \--limit 20 \--format jsonl  
* gmail send \--to "user@example.com" \--subject "Status" \--body-file "./draft.md"  
* gmail draft create \--from-file "./message.json"

### **5.2 Google Calendar API**

Calendar data requires high precision regarding timezones and recurrences.

#### **5.2.1 Complexity: Recurrence Rules**

Events often use RRULEs (RFC 5545). Agents struggle to expand "Every Friday" into specific dates.

* **Requirement:** The CLI list command must default to singleEvents=true. This forces the API to expand recurring events into individual instances, simplifying the agent's reasoning task to checking temporal collisions rather than parsing RRULE syntax.23

#### **5.2.2 Synchronization Patterns**

To avoid re-reading the entire calendar (high token cost), the CLI must expose the syncToken mechanism.

* **Flow:** The first list returns a nextSyncToken. The agent can store this. Subsequent calls using \--sync-token return only created, updated, or deleted events (with status: cancelled). This allows the agent to maintain a "state" of the calendar efficiently.24

## **6\. Deep Dive: Storage & Organization (Google Drive)**

Drive is the filesystem of Workspace. The research highlights "Resumable Uploads" and "Search Queries" as critical integration points.

### **6.1 Resumable Upload Implementation**

For files \>5MB, simple multipart uploads are unstable. The CLI must implement the Resumable Upload protocol.25

1. **Initiation:** POST to /upload/drive/v3/files?uploadType=resumable. Capture Location header.  
2. **Streaming:** Read the local file using tokio::fs::File, creating a buffered stream.  
3. **Chunking:** PUT chunks (multiple of 256KB) to the session URI.  
4. **Recovery:** If a chunk fails (50x error), the CLI must query the upload status (PUT empty body) to find the byte offset and resume. This logic must be hidden from the agent; the agent simply issues drive upload \--file big\_video.mp4 and receives a success/failure JSON.27

### **6.2 Advanced Search**

Drive's q parameter offers a powerful DSL (name contains 'budget' and modifiedTime \> '2023-01-01'). The CLI must expose this directly.

* **Requirement:** workspace-cli drive list \--q "..."  
* **Optimization:** The CLI should validate the query syntax locally if possible, or minimally ensure proper URL encoding of the query string to prevent execution errors.28

### **6.3 Rate Limits & Quotas**

Drive has a generic 12,000 requests/minute limit per project, but a much stricter *write* limit of \~3 requests/second per account.29

* **Implication:** When an agent attempts drive upload \--batch-folder./my\_docs/, the CLI must internally serialize these uploads to 1-2 concurrent threads maximum to avoid hitting the write limit, utilizing a semaphore in the tokio runtime.29

## **7\. Deep Dive: Content Generation (Docs, Sheets, Slides)**

These APIs manipulate the *inside* of files. This is where the Agent's capability to read/write is most powerful but most complex due to JSON schema depth.

### **7.1 Google Docs API**

The document is a tree of StructuralElements.

* **Reading (Token Compression):** A full JSON dump of a Doc includes exhaustive formatting data (fonts, margins). This is noise for an agent just needing text. The CLI must implement a \--format markdown output for docs get. This parses the Body \> Paragraph \> TextRun hierarchy and converts it to Markdown headers, lists, and text, reducing token count by \~60-80%.30  
* **Writing (Batch Updates):** Editing requires batchUpdate with index-based commands (insertText, deleteContent). Calculating indices is error-prone for LLMs.  
* **Strategic Requirement:** The CLI should offer "append" or "replace" helper commands that abstract the index calculation where possible, or strictly validate the JSON payload against the BatchUpdateDocumentRequest schema before sending.30

### **7.2 Google Sheets API**

* **Data Access:** The agent acts as a data analyst. It needs raw data. The CLI must support exporting ranges as CSV (--format csv). This allows the agent to ingest a sheet into its code interpreter environment (e.g., Python pandas) easily.  
* **A1 Notation:** The CLI must enforce A1 notation (Sheet1\!A1:C10) validation.  
* **Updates:** The values.update method is efficient. The CLI must accept a JSON array of arrays \[\[col1, col2\], \[val1, val2\]\] and handle the ValueInputOption (RAW vs USER\_ENTERED) transparently (defaulting to USER\_ENTERED for flexibility).32

### **7.3 Google Slides API**

Slides are complex collections of PageElements.

* **Agent Use Case:** "Read the text on slide 3."  
* **CLI Function:** slides get \--id X \--page 3 \--extract-text. The CLI traverses the Page object, finding all Shape elements with TextContent, and concatenates them. This saves the agent from traversing the deep JSON graph of grouping and layout elements.34

## **8\. Deep Dive: Task Management**

The Tasks API is simpler but critical for action-oriented agents.

* **Links:** Tasks can link to Gmail or Drive. The CLI must expose the links array in the JSON output so the agent can traverse context ("This task refers to email ID X, let me fetch that email").35  
* **Hierarchy:** Tasks can have sub-tasks. The CLI list command should optionally flatten this hierarchy or present it as a nested JSON structure depending on the \--flat flag.36

## **9\. The Agent Integration Layer (MCP & Schema)**

This section defines how the CLI interfaces with the AI agent ecosystem.

### **9.1 MCP Server Architecture**

The workspace-cli will incorporate the mcp-sdk-rs to run as a persistent server.

* **Tool Registry:** On startup, the CLI registers tools matching its commands (e.g., gmail\_list, drive\_upload).  
* **Prompt Definitions:** The CLI will export "Prompts" (pre-written context templates) that help the user or agent use the tools. For example, a summarize\_daily\_email prompt that chains gmail list and messages get.37  
* **Resource Mapping:**  
  * gmail://{message\_id} \-\> Maps to messages.get.  
  * gdrive://{file\_id} \-\> Maps to files.get (with auto-download logic).  
    The MCP server implementation handles the fetching and content provision when the agent requests these URIs.12

### **9.2 Strict Schema Generation**

To prevent hallucinations, the CLI must generate JSON Schemas for its input parameters. Using the schemars crate in Rust, we can derive JSON Schema directly from the argument structs used by clap.

* **Workflow:** Rust Struct \-\> schemars \-\> JSON Schema \-\> MCP Tool Definition.  
* **Benefit:** This guarantees that the tool definition seen by the agent exactly matches the binary's expected input, eliminating "parameter mismatch" errors.39

## **10\. Performance, Batching, and Reliability**

### **10.1 Batch Request Implementation**

Rust's reqwest does not natively support the multipart/mixed content type required for Google Batching. The CLI must implement a custom BatchClient.

* **Logic:**  
  1. Agent submits a JSON array of commands to batch run.  
  2. BatchClient constructs a multipart/mixed body with a random boundary.  
  3. Each part contains Content-Type: application/http and the serialized inner HTTP request (Method, URL, Body).  
  4. The request is POSTed to the batch endpoint (e.g., https://www.googleapis.com/batch/drive/v3).  
  5. The response is parsed (multipart parsing) and re-assembled into a JSON array of results.40  
* **Limit:** Enforce the 100-request limit per batch to prevent API errors.41

### **10.2 Binary Size Optimization**

To ensure the tool is "efficient" and portable:

* **Feature Flags:** Use google-apis-rs feature flags to include *only* the Drive, Gmail, Docs, etc., APIs, stripping thousands of unused AWS-like service bindings.  
* **Strip Symbols:** Compile with strip \= true and opt-level \= "z" (size) or 3 (speed).  
* **LTO:** Enable Link Time Optimization (lto \= true) to remove dead code across crate boundaries.5

## **11\. Data Structures and Rate Limits Reference**

### **11.1 API Rate Limit Reference Table**

| API | Global Limit | Per-User Limit | Cost (Quota Units) | Strategy |
| :---- | :---- | :---- | :---- | :---- |
| **Gmail** | 1B units/day | 250 units/sec | List/Get: 5, Send: 100 | Token Bucket (250 cap) |
| **Drive** | 12,000 req/min | \~3 write/sec | 1 per op | Semaphore (Permits: 3\) |
| **Calendar** | 1M req/day | 500 req/100sec | 1 per op | Exponential Backoff |
| **Sheets** | 300 req/min | 60 req/min | 1 per op | Aggressive Throttling |
| **Docs** | 300 req/min | 60 req/min | 1 per op | Aggressive Throttling |
| **Slides** | 300 req/min | 60 req/min | 1 per op | Aggressive Throttling |
| **Tasks** | 50,000 req/day | N/A | 1 per op | Standard Backoff |

Source: Consolidated from 21

### **11.2 Resource Schema Mapping (Rust \-\> MCP)**

| Rust Struct | Google Resource | MCP Representation | Optimization |
| :---- | :---- | :---- | :---- |
| GmailMessage | Message | Tool Result (JSON) | Base64 Decode, Header Filter |
| DriveFile | File | Resource (gdrive://) | Stream content, Meta-only defaults |
| CalendarEvent | Event | Tool Result (JSON) | Expand Recurrences, Sync Token |
| SheetValues | ValueRange | Resource (csv) | CSV conversion for analysis |
| DocContent | Document | Resource (markdown) | Structure \-\> Markdown conversion |

## **12\. Implementation Roadmap**

### **Phase 1: Foundation (Weeks 1-2)**

* **Task 1.1:** Initialize Rust workspace with tokio, clap, reqwest.  
* **Task 1.2:** Implement yup-oauth2 flow with keyring integration.  
* **Task 1.3:** Create generic ApiClient struct with middleware for Rate Limiting and Backoff.

### **Phase 2: Core Data Modules (Weeks 3-4)**

* **Task 2.1:** Implement Gmail module (List, Get, Send) with body decoding.  
* **Task 2.2:** Implement Drive module with Resumable Uploads and Search.  
* **Task 2.3:** Implement Calendar module with Sync Token logic.

### **Phase 3: Content Modules (Weeks 5-6)**

* **Task 3.1:** Implement Docs parser (JSON \-\> Markdown).  
* **Task 3.2:** Implement Sheets ValueRange handlers (CSV export).  
* **Task 3.3:** Implement Slides and Tasks basic wrappers.

### **Phase 4: Agent Layer & Optimization (Weeks 7-8)**

* **Task 4.1:** Implement MCP Server Protocol using stdio.  
* **Task 4.2:** Integrate schemars for automatic tool definition generation.  
* **Task 4.3:** Implement multipart/mixed batching for bulk operations.  
* **Task 4.4:** Final binary optimization (LTO, strip) and CI/CD pipeline setup.

## **13\. Conclusion**

The workspace-cli represents a critical infrastructure component for the agentic era. By rigorously adhering to type safety, implementing transparent optimizations like batching and markdown conversion, and strictly following the Model Context Protocol, it transforms Google Workspace from a set of disjointed APIs into a coherent, high-speed operating environment for Artificial Intelligence. This tool allows agents to move beyond simple chat interactions to become capable, autonomous operators of enterprise data.

#### **Works cited**

1. Handling HTTP Errors in AI Agents: Lessons from the Field | by Pol Alvarez Vecino | Medium, accessed December 23, 2025, [https://medium.com/@pol.avec/handling-http-errors-in-ai-agents-lessons-from-the-field-4d22d991a269](https://medium.com/@pol.avec/handling-http-errors-in-ai-agents-lessons-from-the-field-4d22d991a269)  
2. mcp\_rust\_sdk \- Rust \- Docs.rs, accessed December 23, 2025, [https://docs.rs/mcp\_rust\_sdk](https://docs.rs/mcp_rust_sdk)  
3. Model Context Protocol (MCP). MCP is an open protocol that… | by Aserdargun | Nov, 2025, accessed December 23, 2025, [https://medium.com/@aserdargun/model-context-protocol-mcp-e453b47cf254](https://medium.com/@aserdargun/model-context-protocol-mcp-e453b47cf254)  
4. Building a Resilient and Type-Safe Rust API Client with reqwest and serde | Leapcell, accessed December 23, 2025, [https://leapcell.io/blog/building-a-resilient-and-type-safe-rust-api-client-with-reqwest-and-serde](https://leapcell.io/blog/building-a-resilient-and-type-safe-rust-api-client-with-reqwest-and-serde)  
5. Benchmarking HTTP Client-Server Binary Size in Rust | by Jonathas Conceição | O.S. Systems | Medium, accessed December 23, 2025, [https://medium.com/os-systems/benchmarking-http-client-server-binary-size-in-rust-3f4398f2aa07](https://medium.com/os-systems/benchmarking-http-client-server-binary-size-in-rust-3f4398f2aa07)  
6. \[Rust\] —Web Crawling Like a Boss: Reqwest-rs and Tl-rs duo is Awesome\!\!\!, accessed December 23, 2025, [https://levelup.gitconnected.com/rust-web-crawling-like-a-boss-reqwest-rs-and-tl-rs-duo-is-awesome-af0f0a6b1cc1](https://levelup.gitconnected.com/rust-web-crawling-like-a-boss-reqwest-rs-and-tl-rs-duo-is-awesome-af0f0a6b1cc1)  
7. How to choose the right Rust HTTP client \- LogRocket Blog, accessed December 23, 2025, [https://blog.logrocket.com/best-rust-http-client/](https://blog.logrocket.com/best-rust-http-client/)  
8. Keep the Terminal Relevant: Patterns for AI Agent Driven CLIs \- InfoQ, accessed December 23, 2025, [https://www.infoq.com/articles/ai-agent-cli/](https://www.infoq.com/articles/ai-agent-cli/)  
9. yup\_oauth2 \- Rust \- Docs.rs, accessed December 23, 2025, [https://docs.rs/yup-oauth2/](https://docs.rs/yup-oauth2/)  
10. keyring \- crates.io: Rust Package Registry, accessed December 23, 2025, [https://crates.io/crates/keyring/4.0.0-alpha.1](https://crates.io/crates/keyring/4.0.0-alpha.1)  
11. MCP servers with the Gemini CLI, accessed December 23, 2025, [https://geminicli.com/docs/tools/mcp-server/](https://geminicli.com/docs/tools/mcp-server/)  
12. Resources \- Model Context Protocol, accessed December 23, 2025, [https://modelcontextprotocol.io/specification/2025-11-25/server/resources](https://modelcontextprotocol.io/specification/2025-11-25/server/resources)  
13. Implement the OAuth 2.0 Authorization Code with PKCE Flow | Okta Developer, accessed December 23, 2025, [https://developer.okta.com/blog/2019/08/22/okta-authjs-pkce](https://developer.okta.com/blog/2019/08/22/okta-authjs-pkce)  
14. yup-oauth2 12.1.1 \- Docs.rs, accessed December 23, 2025, [https://docs.rs/crate/yup-oauth2/latest/source/examples/custom\_storage.rs](https://docs.rs/crate/yup-oauth2/latest/source/examples/custom_storage.rs)  
15. keyring \- crates.io: Rust Package Registry, accessed December 23, 2025, [https://crates.io/crates/keyring](https://crates.io/crates/keyring)  
16. Return specific fields | Google Drive, accessed December 23, 2025, [https://developers.google.com/workspace/drive/api/guides/fields-parameter](https://developers.google.com/workspace/drive/api/guides/fields-parameter)  
17. Cloud Storage JSON API overview \- Google Cloud Documentation, accessed December 23, 2025, [https://docs.cloud.google.com/storage/docs/json\_api](https://docs.cloud.google.com/storage/docs/json_api)  
18. paging\_stream \- Rust \- Docs.rs, accessed December 23, 2025, [https://docs.rs/paging-stream](https://docs.rs/paging-stream)  
19. Page through lists of resources | Google Calendar, accessed December 23, 2025, [https://developers.google.com/workspace/calendar/api/guides/pagination](https://developers.google.com/workspace/calendar/api/guides/pagination)  
20. Ultimate Guide to Gmail API: Features, Pricing, and Implementations \- Apidog, accessed December 23, 2025, [https://apidog.com/blog/gmail-api-guide/](https://apidog.com/blog/gmail-api-guide/)  
21. Usage limits | Gmail | Google for Developers, accessed December 23, 2025, [https://developers.google.com/workspace/gmail/api/reference/quota](https://developers.google.com/workspace/gmail/api/reference/quota)  
22. Batching Requests | People API \- Google for Developers, accessed December 23, 2025, [https://developers.google.com/people/v1/batch](https://developers.google.com/people/v1/batch)  
23. Events: list | Google Calendar, accessed December 23, 2025, [https://developers.google.com/workspace/calendar/api/v3/reference/events/list](https://developers.google.com/workspace/calendar/api/v3/reference/events/list)  
24. Manage quotas | Google Calendar | Google for Developers, accessed December 23, 2025, [https://developers.google.com/workspace/calendar/api/guides/quota](https://developers.google.com/workspace/calendar/api/guides/quota)  
25. Upload file data | Google Drive, accessed December 23, 2025, [https://developers.google.com/workspace/drive/api/guides/manage-uploads](https://developers.google.com/workspace/drive/api/guides/manage-uploads)  
26. Resumable Media Uploads in the Google Data Protocol, accessed December 23, 2025, [https://developers.google.com/gdata/docs/resumable\_upload](https://developers.google.com/gdata/docs/resumable_upload)  
27. Resumable uploads | Cloud Storage \- Google Cloud Documentation, accessed December 23, 2025, [https://docs.cloud.google.com/storage/docs/resumable-uploads](https://docs.cloud.google.com/storage/docs/resumable-uploads)  
28. Search for files and folders | Google Drive, accessed December 23, 2025, [https://developers.google.com/workspace/drive/api/guides/search-files](https://developers.google.com/workspace/drive/api/guides/search-files)  
29. How to Handle Google Drive API Rate Limits for Bulk Folder Copying and Automation, accessed December 23, 2025, [https://folderpal.io/articles/how-to-handle-google-drive-api-rate-limits-for-bulk-folder-copying-and-automation](https://folderpal.io/articles/how-to-handle-google-drive-api-rate-limits-for-bulk-folder-copying-and-automation)  
30. Structure of a Google Docs document, accessed December 23, 2025, [https://developers.google.com/workspace/docs/api/concepts/structure](https://developers.google.com/workspace/docs/api/concepts/structure)  
31. Output document contents as JSON with Docs API \- Google for Developers, accessed December 23, 2025, [https://developers.google.com/workspace/docs/api/samples/output-json](https://developers.google.com/workspace/docs/api/samples/output-json)  
32. Google Sheets API Overview, accessed December 23, 2025, [https://developers.google.com/workspace/sheets/api/guides/concepts](https://developers.google.com/workspace/sheets/api/guides/concepts)  
33. Read & write cell values | Google Sheets, accessed December 23, 2025, [https://developers.google.com/workspace/sheets/api/guides/values](https://developers.google.com/workspace/sheets/api/guides/values)  
34. REST Resource: presentations | Google Slides | Google for ..., accessed December 23, 2025, [https://developers.google.com/workspace/slides/api/reference/rest/v1/presentations](https://developers.google.com/workspace/slides/api/reference/rest/v1/presentations)  
35. REST Resource: tasks \- Google for Developers, accessed December 23, 2025, [https://developers.google.com/tasks/reference/rest/v1/tasks](https://developers.google.com/tasks/reference/rest/v1/tasks)  
36. Google Tasks API quotas and usage limits, accessed December 23, 2025, [https://developers.google.com/workspace/tasks/limits](https://developers.google.com/workspace/tasks/limits)  
37. Tools \- Model Context Protocol, accessed December 23, 2025, [https://modelcontextprotocol.io/specification/draft/server/tools](https://modelcontextprotocol.io/specification/draft/server/tools)  
38. felores/gdrive-mcp-server: Efficient implementation of the Google Drive MCP server \- GitHub, accessed December 23, 2025, [https://github.com/felores/gdrive-mcp-server](https://github.com/felores/gdrive-mcp-server)  
39. google/jsonschema-go: The Go library for JSON Schema. An official Google project providing a comprehensive toolkit for validation, reflection, and schema construction. \- GitHub, accessed December 23, 2025, [https://github.com/google/jsonschema-go](https://github.com/google/jsonschema-go)  
40. Sending Batch Requests | Cloud Deployment Manager, accessed December 23, 2025, [https://docs.cloud.google.com/deployment-manager/docs/reference/latest/batch](https://docs.cloud.google.com/deployment-manager/docs/reference/latest/batch)  
41. Sending batch requests \- Storage \- Google Cloud Documentation, accessed December 23, 2025, [https://docs.cloud.google.com/storage/docs/batch](https://docs.cloud.google.com/storage/docs/batch)  
42. Batch Requests | Manufacturer Center API \- Google for Developers, accessed December 23, 2025, [https://developers.google.com/manufacturers/how-tos/batch](https://developers.google.com/manufacturers/how-tos/batch)  
43. Benchmark of different combinations of server and client crates in Rust to check what offers lower binary size \- GitHub, accessed December 23, 2025, [https://github.com/OSSystems/web-client-server-binary-size-benchmark-rs](https://github.com/OSSystems/web-client-server-binary-size-benchmark-rs)  
44. Directory API: Limits and Quotas | Admin console \- Google for Developers, accessed December 23, 2025, [https://developers.google.com/workspace/admin/directory/v1/limits](https://developers.google.com/workspace/admin/directory/v1/limits)  
45. Usage limits | Google Drive | Google for Developers, accessed December 23, 2025, [https://developers.google.com/workspace/drive/api/guides/limits](https://developers.google.com/workspace/drive/api/guides/limits)  
46. Usage limits | Google Docs, accessed December 23, 2025, [https://developers.google.com/workspace/docs/api/limits](https://developers.google.com/workspace/docs/api/limits)  
47. Usage limits | Google Sheets | Google for Developers, accessed December 23, 2025, [https://developers.google.com/workspace/sheets/api/limits](https://developers.google.com/workspace/sheets/api/limits)  
48. Usage limits | Google Slides, accessed December 23, 2025, [https://developers.google.com/workspace/slides/api/limits](https://developers.google.com/workspace/slides/api/limits)