use std::collections::VecDeque;
use std::io::{BufRead, BufReader};

use tauri::AppHandle;
use tauri::Manager;

const MAX_LINES: usize = 500;

#[tauri::command]
pub async fn get_debug_logs(app: AppHandle) -> Result<String, String> {
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|e| format!("Failed to get log directory: {}", e))?;

    if !log_dir.exists() {
        return Ok(String::new());
    }

    // ログディレクトリ内の .log ファイルを探す
    let mut log_files: Vec<_> = std::fs::read_dir(&log_dir)
        .map_err(|e| format!("Failed to read log directory: {}", e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|ext| ext == "log")
                .unwrap_or(false)
        })
        .collect();

    if log_files.is_empty() {
        return Ok(String::new());
    }

    // 最新のログファイルを選択
    log_files.sort_by_key(|entry| {
        entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    let log_path = log_files.last().unwrap().path();

    let file =
        std::fs::File::open(&log_path).map_err(|e| format!("Failed to open log file: {}", e))?;
    let reader = BufReader::new(file);

    // リングバッファで末尾 N 行のみ保持（メモリ効率化）
    let mut tail: VecDeque<String> = VecDeque::with_capacity(MAX_LINES);
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read log line: {}", e))?;
        if tail.len() == MAX_LINES {
            tail.pop_front();
        }
        tail.push_back(line);
    }

    Ok(tail.into_iter().collect::<Vec<_>>().join("\n"))
}
