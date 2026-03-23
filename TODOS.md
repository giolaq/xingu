# TODOS

## P1 — Phase 2

### MCP Server Mode (`xingu serve`)
Run xingu as a native MCP server via stdio/SSE instead of agents shelling out for every call.
Eliminates cold-start overhead, enables streaming, makes xingu a first-class AI tool.
Use Rust MCP SDK (rmcp or mcp-rs). Expose same tool surface as tools.json but natively.
**Effort:** L | **Depends on:** stable MCP Rust SDK

### Integration Tests for Retry Logic (`wiremock`)
Add `wiremock`-based integration tests for `execute_with_retry` in `client.rs`.
Test scenarios: 3x 429 then success, 401 then refresh then success, 401+429 exhaustion terminates,
immediate success, network error propagation.
**Effort:** M | **Depends on:** nothing

## P2 — Performance

### Streaming Uploads for Large APKs
`client.rs` currently loads entire file into memory via `tokio::fs::read`. A 150MB APK = 150MB heap allocation.
Switch to `reqwest::Body::wrap_stream` with `tokio::fs::File` for streaming.
**Effort:** M | **Depends on:** nothing

### Pagination Support for `apps list`
Amazon API may paginate large app lists. Current code returns only the first page.
Add `--limit` flag and auto-pagination with cursor/token support.
**Effort:** M | **Depends on:** verifying Amazon's pagination format

## P3 — Polish

### Typed Error Enum
Replace string-based error classification in `classify_error` (main.rs) with a `thiserror` enum.
Define `XinguError::Auth`, `::Api`, `::Validation`, `::Network` and match on variants.
**Effort:** S | **Depends on:** nothing

### Shell Completions
Add `xingu completions bash/zsh/fish` using clap's built-in `clap_complete` crate.
**Effort:** S | **Depends on:** nothing

### `xingu auth check`
Validate credentials without a real API call. Fetch a token and show expiry time.
**Effort:** S | **Depends on:** nothing

### `xingu +diff <app-id>`
Show what changed between the live app version and the current edit.
Compare listings, APK versions, etc.
**Effort:** M | **Depends on:** nothing

### Upload Progress Bars
Use `indicatif` crate for progress bars during APK/image uploads.
**Effort:** S | **Depends on:** streaming uploads (P2)
