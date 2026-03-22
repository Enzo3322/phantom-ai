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
