use elo_core::session::LineKind;
use elo_core::{RateStore, Session, Value};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::State;

/// Shared session state managed by Tauri
struct AppState {
    session: Mutex<Session>,
    rates: Option<Arc<RateStore>>,
}

#[derive(Serialize, Deserialize)]
struct LineResult {
    input: String,
    display: String,
    is_empty: bool,
    is_error: bool,
    /// True when the line was treated as plain text / markdown prose rather than
    /// a formula. Such lines never carry an error.
    is_text: bool,
    error: Option<String>,
}

/// Evaluate an entire document (multi-line) and return results for each line
#[tauri::command]
fn evaluate_document(text: &str, state: State<AppState>) -> Vec<LineResult> {
    let mut session = state.session.lock().unwrap();
    // Reset session for fresh evaluation
    *session = Session::with_rates(state.rates.clone());

    text.lines()
        .map(|line| {
            let result = session.eval_line(line);
            let error = match &result.value {
                Value::Error(msg) => Some(msg.clone()),
                _ => None,
            };
            LineResult {
                input: line.to_string(),
                display: result.display,
                is_empty: result.value.is_empty(),
                is_error: result.value.is_error(),
                is_text: result.kind == LineKind::Text,
                error,
            }
        })
        .collect()
}

/// Evaluate a single line (for incremental updates)
#[tauri::command]
fn evaluate_line(line: &str, state: State<AppState>) -> LineResult {
    let mut session = state.session.lock().unwrap();
    let result = session.eval_line(line);
    let error = match &result.value {
        Value::Error(msg) => Some(msg.clone()),
        _ => None,
    };
    LineResult {
        input: line.to_string(),
        display: result.display,
        is_empty: result.value.is_empty(),
        is_error: result.value.is_error(),
        is_text: result.kind == LineKind::Text,
        error,
    }
}

/// Reset the session state
#[tauri::command]
fn reset_session(state: State<AppState>) {
    let mut session = state.session.lock().unwrap();
    *session = Session::with_rates(state.rates.clone());
}

/// iOS-only bridge to the app's iCloud ubiquity container.
///
/// `tauri-plugin-fs` can't reach the iCloud ubiquity container, so we resolve
/// the container URL via Foundation and do the I/O ourselves.
#[cfg(target_os = "ios")]
mod icloud {
    use objc2_foundation::{NSFileManager, NSString};
    use std::path::PathBuf;

    const CONTAINER: &str = "iCloud.com.elo.calculator";

    /// Resolve `<ubiquity-container>/Documents`, creating it if needed.
    ///
    /// NOTE: `URLForUbiquityContainerIdentifier` may block on first call — these
    /// commands run on Tauri's async runtime, which keeps it off the UI thread.
    pub fn documents_dir() -> Result<PathBuf, String> {
        unsafe {
            let fm = NSFileManager::defaultManager();
            let ident = NSString::from_str(CONTAINER);
            let base = fm
                .URLForUbiquityContainerIdentifier(Some(&ident))
                .ok_or("iCloud is not available (not signed in?)")?;
            let docs = base
                .URLByAppendingPathComponent(&NSString::from_str("Documents"))
                .ok_or("could not derive Documents URL")?;
            let path = docs.path().ok_or("no filesystem path")?.to_string();
            let pb = PathBuf::from(path);
            std::fs::create_dir_all(&pb).map_err(|e| e.to_string())?;
            Ok(pb)
        }
    }
}

/// Path to the app's iCloud Documents folder (iOS only).
#[tauri::command]
fn icloud_documents_dir() -> Result<String, String> {
    #[cfg(target_os = "ios")]
    {
        Ok(icloud::documents_dir()?.to_string_lossy().into_owned())
    }
    #[cfg(not(target_os = "ios"))]
    {
        Err("iCloud is only available on iOS".to_string())
    }
}

/// List `.elo`/`.txt`/`.md` documents in the iCloud Documents folder (iOS only).
#[tauri::command]
fn icloud_list_documents() -> Result<Vec<String>, String> {
    #[cfg(target_os = "ios")]
    {
        let dir = icloud::documents_dir()?;
        let mut names = vec![];
        for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().into_owned();
            // iCloud shows not-yet-downloaded files as ".<name>.icloud" stubs.
            let clean = name
                .strip_prefix('.')
                .and_then(|n| n.strip_suffix(".icloud"))
                .map(str::to_owned)
                .unwrap_or(name);
            if clean.ends_with(".elo") || clean.ends_with(".txt") || clean.ends_with(".md") {
                names.push(clean);
            }
        }
        names.sort();
        names.dedup();
        Ok(names)
    }
    #[cfg(not(target_os = "ios"))]
    {
        Err("iCloud is only available on iOS".to_string())
    }
}

/// Read a document from the iCloud Documents folder (iOS only).
#[tauri::command]
fn icloud_read_document(name: String) -> Result<String, String> {
    #[cfg(target_os = "ios")]
    {
        let path = icloud::documents_dir()?.join(&name);
        // For v1, std::fs::read works once the item is materialised. If
        // undownloaded-placeholder reads surface in testing, wire
        // startDownloadingUbiquitousItemAtURL + NSFileCoordinator here.
        std::fs::read_to_string(&path).map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "ios"))]
    {
        let _ = name;
        Err("iCloud is only available on iOS".to_string())
    }
}

/// Write a document to the iCloud Documents folder (iOS only).
#[tauri::command]
fn icloud_write_document(name: String, contents: String) -> Result<(), String> {
    #[cfg(target_os = "ios")]
    {
        let path = icloud::documents_dir()?.join(&name);
        std::fs::write(&path, contents).map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "ios"))]
    {
        let _ = (name, contents);
        Err("iCloud is only available on iOS".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let rates = RateStore::load();
    let builder = tauri::Builder::default()
        .manage(AppState {
            session: Mutex::new(Session::with_rates(rates.clone())),
            rates,
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init());

    // Global shortcuts have no equivalent on mobile — register only on desktop.
    let builder = builder.setup(|_app| {
        #[cfg(desktop)]
        {
            _app.handle()
                .plugin(tauri_plugin_global_shortcut::Builder::new().build())?;
        }
        Ok(())
    });

    builder
        .invoke_handler(tauri::generate_handler![
            evaluate_document,
            evaluate_line,
            reset_session,
            icloud_documents_dir,
            icloud_list_documents,
            icloud_read_document,
            icloud_write_document,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
