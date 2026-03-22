# Continuous OCR Watcher — Design Spec

## Overview

A background screen monitoring feature that periodically captures the screen, extracts text via macOS Vision framework OCR, detects meaningful changes, and processes content through a two-stage AI pipeline (Gemini Flash for filtering/structuring, Gemini Pro for response generation). Activated/deactivated via `Cmd+Shift+O`.

## Architecture

```
[Cmd+Shift+O] → Toggle Watcher ON/OFF
                      │
                      ▼
              ┌─── Watcher Loop (background thread) ───┐
              │                                         │
              │  1. Screenshot (screencapture → temp)   │
              │  2. OCR via Vision framework            │
              │  3. Delete screenshot immediately       │
              │  4. Diff with last extracted text        │
              │     ├─ no change → increase interval    │
              │     └─ changed → continue               │
              │  5. Gemini Flash: filter + structure     │
              │  6. Gemini Pro: process and respond      │
              │  7. Emit event → React (MainPanel)      │
              │  8. Adjust interval                     │
              │  9. Wait interval → back to 1           │
              └─────────────────────────────────────────┘
```

## Module

New file: `src-tauri/src/watcher.rs`

Contains all watcher logic: loop control, OCR integration, diff, model coordination.

## OCR — macOS Vision Framework

- Screenshot via `screencapture -x -C -t png` to temp file
- Load image with `NSImage` / `CGImage`
- `VNRecognizeTextRequest` with `recognitionLevel: accurate`
- `automaticallyDetectsLanguage = true` (PT/EN without config)
- Extract text from all `VNRecognizedTextObservation` results
- Delete temp file immediately after image is loaded into memory
- PNG format (better OCR accuracy than JPG, file is ephemeral so size doesn't matter)

## Content Diff

- Store last extracted text in `AppState` as `last_ocr_text: String`
- Similarity: Jaccard by words (intersection / union)
- Threshold: 85% — above this means "no change"
- Diff runs BEFORE any API call — no tokens wasted on static screens

## Adaptive Interval

| Situation | Interval action |
|---|---|
| No change detected | +1s (max 10s) |
| Change detected | Reset to 2s |
| After model processing | Set to 3s (default) |

- Default start: 3s
- Range: 2s–10s

## Model Pipeline

### Stage 1 — Gemini 2.0 Flash (filter + structure)

- Input: raw OCR text
- System prompt: extract main content, ignore UI elements (menus, navigation bars, notifications). Structure as: context, main content, question and alternatives if present.
- If Flash determines no relevant content → skip Pro call

### Stage 2 — Gemini 2.5 Pro (process)

- Input: structured content from Flash + user's configured prompt
- Output: response emitted via `capture-response` event (reuses existing event system)

## State (AppState additions)

```
watcher_active: bool
watcher_interval_ms: u64
last_ocr_text: String
```

## Shortcut

- `Cmd+Shift+O` — registered in `lib.rs` handler alongside existing shortcuts
- ON: spawn watcher background thread, set `watcher_active = true`
- OFF: set `watcher_active = false`, thread checks flag each cycle and exits

## Events (Rust → React)

| Event | Purpose |
|---|---|
| `watcher-started` | Watcher activated |
| `watcher-stopped` | Watcher deactivated |
| `watcher-ocr-tick` | OCR cycle completed (status: "no_change" or "processing") |
| `capture-response` | Pro response ready (reuses existing event) |
| `capture-error` | Error in any stage |

## UI

### MainPanel
- Visual indicator when watcher is active (pulsing dot or "Watching..." text)
- Responses appear in existing response panel

### ConfigPanel
- Toggle control mirroring watcher state
- No extra configuration (intervals are automatic)

## Error Handling

| Scenario | Behavior |
|---|---|
| Screenshot fails | Log error, keep interval, retry next cycle |
| OCR returns empty | Treat as "no change", increment interval |
| Gemini Flash fails | Emit `capture-error`, backoff interval +2s |
| Gemini Pro fails | Emit `capture-error`, backoff interval +2s |
| Toggle OFF during processing | Discard in-flight response, don't emit event |

## Constraints

- No cycle overlap: new OCR cycle only starts after previous cycle completes
- Screenshot file is always deleted after OCR extraction, no matter what
- Single background thread, no parallelism between cycles
