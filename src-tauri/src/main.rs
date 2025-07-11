#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClipboardItem {
    id: String,
    content: String,
    timestamp: String,
    device: String,
    content_type: String,
}

type ClipboardState = Arc<Mutex<Vec<ClipboardItem>>>;

#[derive(Default)]
struct AppState {
    clipboard_history: ClipboardState,
    last_clipboard_content: Arc<Mutex<String>>,
    enabled: Arc<Mutex<bool>>,
}

// Store functionality disabled - using in-memory storage only for now

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            
            // Initialize state
            let state: State<AppState> = app.state();
            let _clipboard_history = Arc::clone(&state.clipboard_history);
            let enabled = Arc::clone(&state.enabled);
            
            // Set enabled to true by default
            *enabled.lock().unwrap() = true;
            
            println!("🚀 Cliped app starting...");
            println!("✨ Beautiful UI clipboard manager ready!");

            // Start clipboard monitoring after a short delay to ensure runtime is ready
            let state: State<AppState> = app.state();
            let clipboard_history = Arc::clone(&state.clipboard_history);
            let last_content = Arc::clone(&state.last_clipboard_content);
            let enabled = Arc::clone(&state.enabled);
            
            let app_handle_for_monitor = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                // Small delay to ensure everything is initialized
                tokio::time::sleep(Duration::from_millis(100)).await;
                monitor_clipboard(app_handle_for_monitor, clipboard_history, last_content, enabled).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_clipboard_history,
            clear_clipboard_history,
            delete_clipboard_item,
            set_clipboard_content,
            toggle_monitoring,
            is_monitoring_enabled,
            add_clipboard_item
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn monitor_clipboard(
    app_handle: AppHandle,
    clipboard_history: ClipboardState,
    last_content: Arc<Mutex<String>>,
    enabled: Arc<Mutex<bool>>,
) {
    println!("Clipboard monitoring started!");
    let mut clipboard = Clipboard::new().unwrap();
    
    loop {
        sleep(Duration::from_millis(500)).await;
        
        // Check if monitoring is enabled
        if !*enabled.lock().unwrap() {
            continue;
        }
        
        if let Ok(text) = clipboard.get_text() {
            let mut last = last_content.lock().unwrap();
            if text != *last && !text.trim().is_empty() {
                println!("New clipboard content detected: {}", text.chars().take(50).collect::<String>());
                *last = text.clone();
                drop(last);

                let item = ClipboardItem {
                    id: generate_id(),
                    content: text,
                    timestamp: get_current_timestamp(),
                    device: whoami::fallible::hostname().unwrap_or("Unknown".to_string()),
                    content_type: "text".to_string(),
                };

                let mut history = clipboard_history.lock().unwrap();
                
                // Remove duplicates
                history.retain(|existing| existing.content != item.content);
                
                // Insert at beginning
                history.insert(0, item.clone());
                
                // Limit to 50 items
                if history.len() > 50 {
                    history.truncate(50);
                }
                
                println!("Clipboard history now has {} items", history.len());

                // Emit to frontend
                let _ = app_handle.emit("clipboard-updated", &item);
                println!("Emitted clipboard-updated event");
            }
        }
    }
}

#[tauri::command]
async fn get_clipboard_history(state: State<'_, AppState>) -> Result<Vec<ClipboardItem>, String> {
    let history = state.clipboard_history.lock().unwrap();
    Ok(history.clone())
}

#[tauri::command]
async fn clear_clipboard_history(state: State<'_, AppState>) -> Result<(), String> {
    let mut history = state.clipboard_history.lock().unwrap();
    history.clear();
    Ok(())
}

#[tauri::command]
async fn delete_clipboard_item(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut history = state.clipboard_history.lock().unwrap();
    history.retain(|item| item.id != id);
    Ok(())
}

#[tauri::command]
async fn set_clipboard_content(content: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn toggle_monitoring(state: State<'_, AppState>) -> Result<bool, String> {
    let mut enabled = state.enabled.lock().unwrap();
    *enabled = !*enabled;
    let is_enabled = *enabled;
    println!("Clipboard monitoring {}", if is_enabled { "enabled" } else { "disabled" });
    Ok(is_enabled)
}

#[tauri::command]
async fn is_monitoring_enabled(state: State<'_, AppState>) -> Result<bool, String> {
    let enabled = state.enabled.lock().unwrap();
    Ok(*enabled)
}

#[tauri::command]
async fn add_clipboard_item(item: ClipboardItem, state: State<'_, AppState>) -> Result<(), String> {
    let mut history = state.clipboard_history.lock().unwrap();
    
    // Add item to the beginning of the history (LIFO)
    history.insert(0, item);
    
    // Keep only the latest 100 items
    if history.len() > 100 {
        history.truncate(100);
    }
    
    println!("Added clipboard item to history. Total items: {}", history.len());
    Ok(())
}

fn generate_id() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string()
}

fn get_current_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Format as readable time
    let datetime = chrono::DateTime::from_timestamp(now as i64, 0)
        .unwrap_or_default();
    datetime.format("%H:%M:%S").to_string()
}
