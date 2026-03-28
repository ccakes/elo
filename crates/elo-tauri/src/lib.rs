use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::State;
use elo_core::Session;

/// Shared session state managed by Tauri
struct AppState {
    session: Mutex<Session>,
}

#[derive(Serialize, Deserialize)]
struct LineResult {
    input: String,
    display: String,
    is_empty: bool,
    is_error: bool,
}

/// Evaluate an entire document (multi-line) and return results for each line
#[tauri::command]
fn evaluate_document(text: &str, state: State<AppState>) -> Vec<LineResult> {
    let mut session = state.session.lock().unwrap();
    // Reset session for fresh evaluation
    *session = Session::new();

    text.lines()
        .map(|line| {
            let result = session.eval_line(line);
            LineResult {
                input: line.to_string(),
                display: result.display,
                is_empty: result.value.is_empty(),
                is_error: result.value.is_error(),
            }
        })
        .collect()
}

/// Evaluate a single line (for incremental updates)
#[tauri::command]
fn evaluate_line(line: &str, state: State<AppState>) -> LineResult {
    let mut session = state.session.lock().unwrap();
    let result = session.eval_line(line);
    LineResult {
        input: line.to_string(),
        display: result.display,
        is_empty: result.value.is_empty(),
        is_error: result.value.is_error(),
    }
}

/// Reset the session state
#[tauri::command]
fn reset_session(state: State<AppState>) {
    let mut session = state.session.lock().unwrap();
    *session = Session::new();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            session: Mutex::new(Session::new()),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            evaluate_document,
            evaluate_line,
            reset_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
