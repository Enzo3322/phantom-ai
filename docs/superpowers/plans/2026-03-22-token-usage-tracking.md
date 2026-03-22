# Token Usage Tracking Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Track Gemini API token consumption in a local SQLite database and display per-feature breakdown in a new "Usage" tab in ConfigPanel.

**Architecture:** Extract `usageMetadata` from Gemini API responses by changing return types to include `TokenUsage`. A new `usage_db.rs` module handles SQLite operations. Each caller records usage after API calls. A new IPC command serves aggregated data to the frontend's new Usage tab.

**Tech Stack:** Rust (rusqlite with bundled SQLite), Tauri 2 path API, React/TypeScript frontend.

---

### Task 1: Add rusqlite dependency

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add rusqlite to dependencies**

In `src-tauri/Cargo.toml`, add after the `rand` line (line 26):

```toml
rusqlite = { version = "0.31", features = ["bundled"] }
```

- [ ] **Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: downloads rusqlite, compiles with no errors

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "feat: add rusqlite dependency for token usage tracking"
```

---

### Task 2: Create usage_db.rs module

**Files:**
- Create: `src-tauri/src/usage_db.rs`
- Modify: `src-tauri/src/lib.rs` (add `mod usage_db;`)

- [ ] **Step 1: Create `usage_db.rs`**

```rust
use rusqlite::{Connection, params};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Clone, Debug)]
pub struct UsageSummary {
    pub feature: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

pub fn open_db(path: &Path) -> Result<Connection, String> {
    let conn = Connection::open(path)
        .map_err(|e| format!("Failed to open usage database: {e}"))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS token_usage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            feature TEXT NOT NULL,
            model TEXT NOT NULL,
            input_tokens INTEGER NOT NULL,
            output_tokens INTEGER NOT NULL
        );"
    ).map_err(|e| format!("Failed to create token_usage table: {e}"))?;

    Ok(conn)
}

pub fn record_usage(
    db_path: &str,
    feature: &str,
    model: &str,
    input_tokens: u32,
    output_tokens: u32,
) {
    let path = Path::new(db_path);
    let conn = match open_db(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[phantom] usage_db: failed to open db: {e}");
            return;
        }
    };

    if let Err(e) = conn.execute(
        "INSERT INTO token_usage (feature, model, input_tokens, output_tokens) VALUES (?1, ?2, ?3, ?4)",
        params![feature, model, input_tokens, output_tokens],
    ) {
        eprintln!("[phantom] usage_db: failed to record usage: {e}");
    }
}

pub fn get_usage_summary(db_path: &str) -> Vec<UsageSummary> {
    let path = Path::new(db_path);
    let conn = match open_db(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[phantom] usage_db: failed to open db: {e}");
            return Vec::new();
        }
    };

    let mut stmt = match conn.prepare(
        "SELECT feature, model, SUM(input_tokens), SUM(output_tokens)
         FROM token_usage
         GROUP BY feature, model
         ORDER BY feature, model"
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[phantom] usage_db: failed to prepare query: {e}");
            return Vec::new();
        }
    };

    let rows = stmt.query_map([], |row| {
        Ok(UsageSummary {
            feature: row.get(0)?,
            model: row.get(1)?,
            input_tokens: row.get(2)?,
            output_tokens: row.get(3)?,
        })
    });

    match rows {
        Ok(mapped) => mapped.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            eprintln!("[phantom] usage_db: query failed: {e}");
            Vec::new()
        }
    }
}
```

- [ ] **Step 2: Register module in lib.rs**

Add after `mod watcher;` (line 17):

```rust
mod usage_db;
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles (unused warnings ok)

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/usage_db.rs src-tauri/src/lib.rs
git commit -m "feat: add usage_db module for SQLite token tracking"
```

---

### Task 3: Add usage_db_path to AppState and initialize in setup

**Files:**
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add field to AppState struct**

In `state.rs`, add after `last_ocr_text` field (line 33):

```rust
// Token usage tracking
pub usage_db_path: Mutex<Option<String>>,
```

- [ ] **Step 2: Add getter/setter**

After `set_last_ocr_text`:

```rust
pub fn get_usage_db_path(&self) -> Option<String> {
    self.usage_db_path.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

pub fn set_usage_db_path(&self, val: Option<String>) {
    *self.usage_db_path.lock().unwrap_or_else(|e| e.into_inner()) = val;
}
```

- [ ] **Step 3: Add default**

In `Default` impl, after `last_ocr_text`:

```rust
usage_db_path: Mutex::new(None),
```

- [ ] **Step 4: Initialize database in setup**

In `lib.rs`, inside `.setup()` after `dodge::start_dodge_watcher(handle.clone());` (line 336), add:

```rust
// Initialize token usage database
if let Some(app_data) = handle.path().app_data_dir().ok() {
    let _ = std::fs::create_dir_all(&app_data);
    let db_path = app_data.join("phantom_usage.db");
    let db_path_str = db_path.to_string_lossy().to_string();
    match usage_db::open_db(&db_path) {
        Ok(_) => {
            eprintln!("[phantom] usage db initialized at: {db_path_str}");
            handle.state::<AppState>().set_usage_db_path(Some(db_path_str));
        }
        Err(e) => {
            eprintln!("[phantom] usage db init failed: {e}");
        }
    }
}
```

- [ ] **Step 5: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/state.rs src-tauri/src/lib.rs
git commit -m "feat: add usage_db_path to AppState and initialize in setup"
```

---

### Task 4: Add TokenUsage to gemini.rs and update return types

**Files:**
- Modify: `src-tauri/src/gemini.rs`

- [ ] **Step 1: Add UsageMetadata and TokenUsage structs**

After `ResponsePart` struct (line 57), add:

```rust
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct UsageMetadata {
    #[serde(default)]
    prompt_token_count: u32,
    #[serde(default)]
    candidates_token_count: u32,
}

#[derive(Clone, Debug, Default)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}
```

- [ ] **Step 2: Add `usage_metadata` to GeminiResponse**

Change `GeminiResponse` (line 32-35) to:

```rust
#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    error: Option<GeminiError>,
    #[serde(default, rename = "usageMetadata")]
    usage_metadata: Option<UsageMetadata>,
}
```

- [ ] **Step 3: Add helper to extract TokenUsage from GeminiResponse**

After the new structs:

```rust
fn extract_usage(metadata: Option<UsageMetadata>) -> TokenUsage {
    match metadata {
        Some(m) => TokenUsage {
            input_tokens: m.prompt_token_count,
            output_tokens: m.candidates_token_count,
        },
        None => TokenUsage::default(),
    }
}
```

- [ ] **Step 4: Update `analyze_screenshot` return type**

Change signature (line 59) from `Result<String, String>` to `Result<(String, TokenUsage), String>`.

Before returning `Ok(result)` (line 147), change to:

```rust
let usage = extract_usage(body.usage_metadata);
Ok((result, usage))
```

- [ ] **Step 5: Update `send_to_gemini` return type**

Change signature (line 150) from `Result<String, String>` to `Result<(String, TokenUsage), String>`.

Change the final expression (lines 213-217) to:

```rust
let usage = extract_usage(body.usage_metadata);
let text = body.candidates
    .and_then(|c| c.into_iter().next())
    .and_then(|c| c.content.parts.into_iter().next())
    .map(|p| p.text)
    .ok_or_else(|| "Empty response from Gemini".to_string())?;
Ok((text, usage))
```

- [ ] **Step 6: Update `send_text_prompt` return type**

Change signature (line 220) from `Result<String, String>` to `Result<(String, TokenUsage), String>`.

Change the final expression (lines 271-275) to:

```rust
let usage = extract_usage(body.usage_metadata);
let text = body.candidates
    .and_then(|c| c.into_iter().next())
    .and_then(|c| c.content.parts.into_iter().next())
    .map(|p| p.text)
    .ok_or_else(|| "Empty response from Gemini".to_string())?;
Ok((text, usage))
```

- [ ] **Step 7: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compile ERRORS in callers (lib.rs, commands.rs, watcher.rs) because return type changed — this is expected, we fix them in the next tasks.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/gemini.rs
git commit -m "feat: extract usageMetadata from Gemini responses as TokenUsage"
```

---

### Task 5: Update callers to handle TokenUsage and record to database

**Files:**
- Modify: `src-tauri/src/lib.rs` (handle_capture)
- Modify: `src-tauri/src/commands.rs` (send_transcription_to_gemini)
- Modify: `src-tauri/src/watcher.rs` (watcher loop — Flash and Pro calls)

- [ ] **Step 1: Update handle_capture in lib.rs**

In `handle_capture`, change the `match gemini::analyze_screenshot(...)` block (lines 204-216):

```rust
match gemini::analyze_screenshot(&api_key, &model, &base64_image, &prompt, &response_language, spoof_ua, jitter, proxy_ref).await {
    Ok((response, usage)) => {
        eprintln!("[phantom] capture: gemini ok, {} chars, tokens: in={} out={}", response.len(), usage.input_tokens, usage.output_tokens);
        state.set_last_response(Some(response.clone()));
        state.set_processing(false);
        let _ = app.emit("capture-response", serde_json::json!({ "text": response, "source": "screenshot" }));
        if let Some(db_path) = state.get_usage_db_path() {
            usage_db::record_usage(&db_path, "screenshot", &model, usage.input_tokens, usage.output_tokens);
        }
    }
    Err(e) => {
        eprintln!("[phantom] capture: gemini error: {e}");
        state.set_processing(false);
        state.set_last_response(Some(format!("Error: {e}")));
        let _ = app.emit("capture-error", format!("Error: {e}"));
    }
}
```

- [ ] **Step 2: Update send_transcription_to_gemini in commands.rs**

Change the `match crate::gemini::send_to_gemini(...)` block (lines 203-213):

```rust
match crate::gemini::send_to_gemini(&api_key, &model, &text, &prompt, &response_language, spoof_ua, jitter, proxy_ref).await {
    Ok((response, usage)) => {
        state.set_last_response(Some(response.clone()));
        let _ = app.emit("capture-response", serde_json::json!({ "text": response, "source": "transcription" }));
        if let Some(db_path) = state.get_usage_db_path() {
            crate::usage_db::record_usage(&db_path, "transcription", &model, usage.input_tokens, usage.output_tokens);
        }
        Ok(())
    }
    Err(e) => {
        let _ = app.emit("capture-error", format!("Error: {e}"));
        Err(e)
    }
}
```

- [ ] **Step 3: Update watcher.rs Flash call**

Change `let flash_result = crate::gemini::send_text_prompt(...)` and the match (around lines 360-383):

```rust
let flash_result = crate::gemini::send_text_prompt(
    &api_key, FLASH_MODEL, &flash_prompt, spoof_ua, jitter, proxy_ref,
).await;

let filtered_text = match flash_result {
    Ok((text, usage)) => {
        if let Some(db_path) = state.get_usage_db_path() {
            crate::usage_db::record_usage(&db_path, "watcher", FLASH_MODEL, usage.input_tokens, usage.output_tokens);
        }
        text
    }
    Err(e) => {
        eprintln!("[phantom] watcher: Flash API error: {e}");
        let _ = app.emit(
            "watcher-ocr-tick",
            serde_json::json!({ "status": "api_error", "error": e }),
        );
        let state = app.state::<AppState>();
        let current = state.get_watcher_interval_ms();
        state.set_watcher_interval_ms((current + ERROR_BACKOFF_MS).min(MAX_INTERVAL_MS));
        continue;
    }
};
```

- [ ] **Step 4: Update watcher.rs Pro call**

Change the Pro `match pro_result` block (around lines 425-453):

```rust
match pro_result {
    Ok((response, usage)) => {
        eprintln!("[phantom] watcher: Pro returned {} chars", response.len());
        let state = app.state::<AppState>();
        state.set_last_response(Some(response.clone()));
        let _ = app.emit(
            "capture-response",
            serde_json::json!({
                "text": response,
                "source": "watcher"
            }),
        );
        if let Some(db_path) = state.get_usage_db_path() {
            crate::usage_db::record_usage(&db_path, "watcher", PRO_MODEL, usage.input_tokens, usage.output_tokens);
        }
        state.set_watcher_interval_ms(DEFAULT_INTERVAL_MS);
    }
    Err(e) => {
        eprintln!("[phantom] watcher: Pro API error: {e}");
        let _ = app.emit(
            "watcher-ocr-tick",
            serde_json::json!({ "status": "api_error", "error": e }),
        );
        let state = app.state::<AppState>();
        let current = state.get_watcher_interval_ms();
        state.set_watcher_interval_ms((current + ERROR_BACKOFF_MS).min(MAX_INTERVAL_MS));
    }
}
```

- [ ] **Step 5: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles with no errors

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/commands.rs src-tauri/src/watcher.rs
git commit -m "feat: record token usage from all Gemini callers to SQLite"
```

---

### Task 6: Add get_token_usage IPC command

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add command to commands.rs**

At the end of `commands.rs`:

```rust
#[tauri::command]
pub fn get_token_usage(state: tauri::State<'_, AppState>) -> Vec<crate::usage_db::UsageSummary> {
    match state.get_usage_db_path() {
        Some(path) => crate::usage_db::get_usage_summary(&path),
        None => Vec::new(),
    }
}
```

- [ ] **Step 2: Register command in lib.rs**

In the `invoke_handler`, add after `commands::toggle_watcher,`:

```rust
commands::get_token_usage,
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat: add get_token_usage IPC command"
```

---

### Task 7: Add Usage tab to ConfigPanel

**Files:**
- Modify: `src/components/ConfigPanel/ConfigPanel.tsx`
- Modify: `src/components/ConfigPanel/ConfigPanel.css`

- [ ] **Step 1: Update Tab type**

Change the `Tab` type (around line 103) from:

```typescript
type Tab = "general" | "audio" | "stealth" | "shortcuts";
```

to:

```typescript
type Tab = "general" | "audio" | "stealth" | "shortcuts" | "usage";
```

- [ ] **Step 2: Add UsageSummary interface and state**

After the imports at the top of the file, add:

```typescript
interface UsageSummary {
  feature: string;
  model: string;
  input_tokens: number;
  output_tokens: number;
}
```

Inside `ConfigPanel()`, add after the `showKey` state:

```typescript
const [usageData, setUsageData] = useState<UsageSummary[]>([]);
```

- [ ] **Step 3: Load usage data when Usage tab is selected**

Add a `useEffect` after `resizeToFit`:

```typescript
useEffect(() => {
    if (activeTab === "usage") {
      invoke<UsageSummary[]>("get_token_usage").then(setUsageData);
    }
}, [activeTab]);
```

- [ ] **Step 4: Add Usage tab button**

After the "Shortcuts" tab button (before `</div>` of tab-bar), add:

```tsx
<button
  className={`tab-btn ${activeTab === "usage" ? "active" : ""}`}
  onClick={() => setActiveTab("usage")}
>
  <svg className="tab-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <line x1="18" y1="20" x2="18" y2="10" />
    <line x1="12" y1="20" x2="12" y2="4" />
    <line x1="6" y1="20" x2="6" y2="14" />
  </svg>
  Usage
</button>
```

- [ ] **Step 5: Add Usage tab content**

After the shortcuts tab content block (`{activeTab === "shortcuts" && (...)}`), add:

```tsx
{activeTab === "usage" && (
  <>
    <div className="field">
      <label>
        <svg className="field-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <line x1="18" y1="20" x2="18" y2="10" />
          <line x1="12" y1="20" x2="12" y2="4" />
          <line x1="6" y1="20" x2="6" y2="14" />
        </svg>
        Token Usage by Feature
      </label>
      {usageData.length === 0 ? (
        <div className="usage-empty">No usage data yet</div>
      ) : (
        <div className="usage-table">
          <div className="usage-header">
            <span>Feature</span>
            <span>Model</span>
            <span>Input</span>
            <span>Output</span>
            <span>Total</span>
          </div>
          {usageData.map((row, i) => (
            <div key={i} className="usage-row">
              <span className="usage-feature">{row.feature}</span>
              <span className="usage-model">{row.model.replace("gemini-", "")}</span>
              <span>{row.input_tokens.toLocaleString()}</span>
              <span>{row.output_tokens.toLocaleString()}</span>
              <span>{(row.input_tokens + row.output_tokens).toLocaleString()}</span>
            </div>
          ))}
          <div className="usage-row usage-total">
            <span>Total</span>
            <span></span>
            <span>{usageData.reduce((s, r) => s + r.input_tokens, 0).toLocaleString()}</span>
            <span>{usageData.reduce((s, r) => s + r.output_tokens, 0).toLocaleString()}</span>
            <span>{usageData.reduce((s, r) => s + r.input_tokens + r.output_tokens, 0).toLocaleString()}</span>
          </div>
        </div>
      )}
    </div>
  </>
)}
```

- [ ] **Step 6: Add CSS**

Append to `src/components/ConfigPanel/ConfigPanel.css`:

```css
.usage-empty {
  color: rgba(255, 255, 255, 0.4);
  font-size: 12px;
  text-align: center;
  padding: 20px 0;
}

.usage-table {
  font-size: 11px;
  border-radius: 6px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.03);
}

.usage-header,
.usage-row {
  display: grid;
  grid-template-columns: 1.2fr 1.2fr 0.8fr 0.8fr 0.8fr;
  padding: 6px 10px;
  gap: 4px;
}

.usage-header {
  color: rgba(255, 255, 255, 0.5);
  font-weight: 500;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.usage-row {
  color: rgba(255, 255, 255, 0.8);
  border-bottom: 1px solid rgba(255, 255, 255, 0.03);
}

.usage-row:last-child {
  border-bottom: none;
}

.usage-feature {
  text-transform: capitalize;
}

.usage-model {
  color: rgba(255, 255, 255, 0.5);
  font-size: 10px;
}

.usage-total {
  font-weight: 600;
  color: rgba(255, 255, 255, 0.95);
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.02);
}
```

- [ ] **Step 7: Verify frontend builds**

Run: `npm run build`
Expected: builds with no errors

- [ ] **Step 8: Commit**

```bash
git add src/components/ConfigPanel/ConfigPanel.tsx src/components/ConfigPanel/ConfigPanel.css
git commit -m "feat: add Usage tab to ConfigPanel with token breakdown"
```

---

### Task 8: Full build and verification

**Files:** None (verification only)

- [ ] **Step 1: Full Rust build**

Run: `cd src-tauri && cargo build`
Expected: compiles

- [ ] **Step 2: Full frontend build**

Run: `npm run build`
Expected: builds

- [ ] **Step 3: Run Tauri dev**

Run: `npm run tauri dev`
Expected: app launches, Usage tab visible in config panel

- [ ] **Step 4: Manual verification**

1. Open config panel (`Cmd+Shift+C`)
2. Navigate to Usage tab — should show "No usage data yet"
3. Take a screenshot (`Cmd+Shift+S`) — should record tokens
4. Open Usage tab again — should show screenshot row with token counts
5. Check terminal logs for `[phantom] usage_db:` messages
