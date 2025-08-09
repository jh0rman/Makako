# Makako HTTP Client

A fast, lightweight, and native desktop HTTP client built in Rust with [GPUI](https://gpui.rs/). Inspired by Bruno — all state and collections live on your local filesystem. No cloud, no accounts.

## Status

> **v3.0 — In progress: Tabs, Code Export & Assertions**

## Architecture

```
src/
├── main.rs                       # Entry point — opens the window and wires modules
├── ui_module/
│   ├── mod.rs                    # AppView: 3-panel shell, tab bar, sidebar
│   ├── headers_editor.rs         # HeadersEditor sub-view (key-value pairs)
│   └── response_panel.rs         # ResponsePanel sub-view (status, latency, body)
├── network_module/
│   └── mod.rs                    # HTTP execution (reqwest blocking + oneshot channel)
└── storage_module/
    └── mod.rs                    # JSON persistence + env loading + interpolation
```

### Module responsibilities

| Module           | Responsibility                                                        |
|------------------|-----------------------------------------------------------------------|
| `ui_module`      | All GPUI rendering: tab bar, sidebar tree, request editor, response   |
| `network_module` | Async HTTP calls via `reqwest` (GET, POST, PUT, DELETE)               |
| `storage_module` | Read/write request collections and env files from local filesystem    |

## UI Layout

```
┌──────────────────────────────────────────────────────────────┐
│  Sidebar (240 px)  │ [Tab 1] [Tab 2] [+]  │  Response        │
│                    │──────────────────────│  (420 px)        │
│  📁 jsonplaceholder│  GET ▾  URL input    │                  │
│    📄 get-posts    │  Headers             │  200 OK  42 ms   │
│    📄 create-post  │  Body (JSON)         │                  │
│  📁 httpbin        │                      │  { "id": 101 }   │
│    📄 get-anything │  [Save]  [Send]      │                  │
└──────────────────────────────────────────────────────────────┘
```

## Goals (v3) — In Progress

- [x] **Tab system:** `TabState` struct holding all per-request state; `tabs: Vec<TabState>` + `active_tab`
- [x] **Tab bar UI:** row of tab buttons above the editor; `+` button opens a new blank tab
- [x] **Tab isolation:** Send, URL edits, and responses are scoped to the active tab only
- [ ] **Code snippet export:** translate active request to `cURL`, `fetch` (JS), or `reqwest` (Rust)
- [ ] **Test assertions:** simple JSON DSL (`{"expect_status": 200, "expect_body_contains": "id"}`) evaluated after each response
- [ ] **GraphQL support:** dedicated Query + Variables editor body mode

## Non-Goals

- No cloud sync or user accounts
- No multiple OS windows — single app window
- No WebSockets or SSE (v3)
- No automatic OAuth2 flows — pass tokens manually in headers
- No `.bru` format parsing — JSON collections for now
- No embedded JS engine (v3) — assertions use a lightweight Rust DSL

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (Edition 2024)
- macOS (GPUI is optimized for macOS; requires `core-text`)

## Running

```bash
cargo run
```

## Building a release binary

```bash
cargo build --release
```

## Dependencies

| Crate                  | Purpose                                              |
|------------------------|------------------------------------------------------|
| `gpui`                 | High-performance native UI framework                 |
| `gpui-component`       | Additional GPUI component utilities                  |
| `core-text`            | macOS CoreText bindings                              |
| `reqwest`              | HTTP client (blocking, used in background thread)    |
| `futures`              | `oneshot` channel to bridge thread → async executor  |
| `serde` / `serde_json` | Serialize/deserialize saved requests as JSON         |
| `dirs`                 | Resolve `~/Documents/` cross-platform                |
