# Token Usage Tracking — Design Spec

## Overview

Track accumulated token consumption from Gemini API calls, stored in a local SQLite database, displayed in a new "Usage" tab in the ConfigPanel. Shows breakdown by feature (screenshot, transcription, watcher) with input/output token counts extracted from the API's `usageMetadata` response field.

## Database

**Dependency:** `rusqlite` with `bundled` feature (compiles SQLite, no system dependency).

**Location:** `app_data_dir/phantom_usage.db` (via Tauri path API).

**Schema:**

```sql
CREATE TABLE IF NOT EXISTS token_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    feature TEXT NOT NULL,
    model TEXT NOT NULL,
    input_tokens INTEGER NOT NULL,
    output_tokens INTEGER NOT NULL
);
```

**Module:** `src-tauri/src/usage_db.rs`

Functions:
- `init_db(path: &Path) -> Result<Connection>` — create/open database, create table if needed
- `record_usage(conn: &Connection, feature: &str, model: &str, input_tokens: u32, output_tokens: u32)` — insert a row
- `get_usage_by_feature(conn: &Connection) -> Vec<UsageSummary>` — `SELECT feature, model, SUM(input_tokens), SUM(output_tokens) FROM token_usage GROUP BY feature, model`

**Initialization:** In `setup()`, open the database and store the path in AppState as `usage_db_path: Mutex<Option<String>>`.

## Gemini API Changes

### TokenUsage struct

```rust
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}
```

### Response parsing

Add `usageMetadata` to `GeminiResponse`:

```rust
#[derive(Deserialize)]
struct UsageMetadata {
    #[serde(default)]
    prompt_token_count: u32,
    #[serde(default)]
    candidates_token_count: u32,
}
```

### Function signatures

All three functions change return type:
- `analyze_screenshot` → `Result<(String, TokenUsage), String>`
- `send_to_gemini` → `Result<(String, TokenUsage), String>`
- `send_text_prompt` → `Result<(String, TokenUsage), String>`

If `usageMetadata` is absent, return `TokenUsage { input_tokens: 0, output_tokens: 0 }`.

## Callers — Recording Usage

Each caller receives `TokenUsage` and records it:

- `handle_capture` in `lib.rs` — feature: `"screenshot"`, model from state
- `send_transcription_to_gemini` in `commands.rs` — feature: `"transcription"`, model from state
- Watcher loop in `watcher.rs` — feature: `"watcher"`, records both Flash and Pro calls separately

Recording pattern:
```rust
if let Some(db_path) = state.get_usage_db_path() {
    let _ = usage_db::record_usage_at_path(&db_path, feature, model, usage.input_tokens, usage.output_tokens);
}
```

## IPC Command

```rust
#[tauri::command]
pub fn get_token_usage(state: tauri::State<'_, AppState>) -> Vec<UsageSummary>
```

Returns aggregated totals grouped by feature and model.

```rust
pub struct UsageSummary {
    pub feature: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
}
```

## Frontend — Usage Tab

**New tab** in ConfigPanel: "Usage" with a bar chart icon.

**Content:**
- Table/list showing per-feature breakdown:
  - Feature name (Screenshot, Transcription, Watcher)
  - Model used
  - Input tokens, Output tokens, Total
- Total row at the bottom summing all features

**Data loading:** `invoke("get_token_usage")` called when the Usage tab is selected. No auto-refresh.

## Error Handling

| Scenario | Behavior |
|---|---|
| Database inaccessible | Log error, app continues, usage not recorded |
| API missing usageMetadata | Record with 0/0 tokens |
| Database corrupted | Log error, set path to None, skip all recording |
| get_token_usage with no DB | Return empty array |

## Constraints

- Database failures never block API calls
- No data reset functionality — accumulates from installation
- No cost estimation — tokens only
