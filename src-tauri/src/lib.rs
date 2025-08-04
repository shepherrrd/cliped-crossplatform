#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(feature = "clipboard")]
use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::net::UdpSocket;
use tokio::time::{sleep, Duration};
use local_ip_address::local_ip;
use rusqlite::Connection;
use directories::ProjectDirs;
use rfd::FileDialog;
use base64::{Engine as _, engine::general_purpose};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Device {
    id: u32,
    name: String,
    icon: String,
    ip: String,
    status: DeviceStatus,
    sync_mode: SyncMode,
    last_seen: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
enum DeviceStatus {
    Pending,    // Connection request sent/received
    Connected,  // Accepted and connected
    Denied,     // Connection denied
    Offline,    // Device not responding
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
enum SyncMode {
    TotalSync,   // Sync entire history
    PartialSync, // Sync only new items from now on
    Disabled,    // No syncing
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct NetworkMessage {
    msg_type: MessageType,
    device_id: u32,
    device_name: String,
    data: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum MessageType {
    Discovery,        // Device announcing presence
    ConnectionRequest, // Request to connect
    ConnectionAccept,  // Accept connection
    ConnectionDeny,    // Deny connection
    ConnectionRemove,  // Device disconnected/removed
    ClipboardSync,    // Sync clipboard item
    FileTransfer,     // File transfer request
    FileTransferChunk, // File data chunk
    FileTransferComplete, // File transfer completion
    Heartbeat,        // Keep connection alive
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClipboardItem {
    id: String,
    content: String,
    timestamp: String,
    device: String,
    content_type: String,
    file_path: Option<String>,
    file_size: Option<u64>,
    file_name: Option<String>,
}

type ClipboardState = Arc<Mutex<Vec<ClipboardItem>>>;

#[derive(Default)]
struct AppState {
    devices: Arc<Mutex<HashMap<u32, Device>>>,
    clipboard_history: ClipboardState,
    last_clipboard_content: Arc<Mutex<String>>,
    enabled: Arc<Mutex<bool>>,
    local_device: Arc<Mutex<Option<Device>>>,
    db_path: Arc<Mutex<Option<String>>>,
    pending_connections: Arc<Mutex<Vec<Device>>>,
    discovered_devices: Arc<Mutex<Vec<Device>>>,
    ignore_next_clipboard_change: Arc<Mutex<bool>>, // Flag to ignore clipboard changes from sync
}

// Utility functions
fn init_database() -> Result<String, String> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "cliped", "cliped") {
        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;
        
        let db_path = data_dir.join("clipboard.db");
        let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard_items (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                device TEXT NOT NULL,
                content_type TEXT NOT NULL,
                file_path TEXT,
                file_size INTEGER,
                file_name TEXT
            )",
            [],
        ).map_err(|e| e.to_string())?;
        
        // Add new columns if they don't exist (for existing databases)
        let _ = conn.execute(
            "ALTER TABLE clipboard_items ADD COLUMN file_path TEXT",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE clipboard_items ADD COLUMN file_size INTEGER",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE clipboard_items ADD COLUMN file_name TEXT",
            [],
        );
        
        Ok(db_path.to_string_lossy().to_string())
    } else {
        Err("Failed to get project directories".to_string())
    }
}

fn generate_device_info() -> Device {
    let id = generate_id();
    let device_name = format!("Device-{}", generate_random_suffix());
    let ip = get_local_ip();
    
    Device {
        id,
        name: device_name,
        icon: "laptop".to_string(),
        ip,
        status: DeviceStatus::Connected,
        sync_mode: SyncMode::Disabled,
        last_seen: get_current_timestamp(),
    }
}

fn generate_id() -> u32 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    
    let mut hasher = DefaultHasher::new();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .hash(&mut hasher);
    
    (hasher.finish() % u32::MAX as u64) as u32
}

fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn generate_random_suffix() -> String {
    format!("{:04}", rand::random::<u16>() % 10000)
}

fn get_local_ip() -> String {
    local_ip().map(|ip| ip.to_string()).unwrap_or_else(|_| "127.0.0.1".to_string())
}

fn load_clipboard_history_from_db(db_path: &str) -> Result<Vec<ClipboardItem>, String> {
    load_clipboard_history_paginated(db_path, 0, 50)
}

fn load_clipboard_history_paginated(db_path: &str, offset: u32, limit: u32) -> Result<Vec<ClipboardItem>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT id, content, timestamp, device, content_type, file_path, file_size, file_name FROM clipboard_items WHERE content_type != 'file' ORDER BY timestamp DESC LIMIT ?1 OFFSET ?2"
    ).map_err(|e| e.to_string())?;
    
    let clipboard_iter = stmt.query_map([limit, offset], |row| {
        Ok(ClipboardItem {
            id: row.get(0)?,
            content: row.get(1)?,
            timestamp: row.get(2)?,
            device: row.get(3)?,
            content_type: row.get(4)?,
            file_path: row.get(5).ok(),
            file_size: row.get(6).ok(),
            file_name: row.get(7).ok(),
        })
    }).map_err(|e| e.to_string())?;
    
    let mut items = Vec::new();
    for item in clipboard_iter {
        items.push(item.map_err(|e| e.to_string())?);
    }
    
    Ok(items)
}

fn get_clipboard_history_count_from_db(db_path: &str) -> Result<u32, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    let count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM clipboard_items WHERE content_type != 'file'",
        [],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;
    
    Ok(count)
}

fn get_clipboard_files_count_from_db(db_path: &str) -> Result<u32, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    let count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM clipboard_items WHERE content_type = 'file'",
        [],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;
    
    Ok(count)
}

fn get_clipboard_files_paginated_from_db(db_path: &str, offset: u32, limit: u32) -> Result<Vec<ClipboardItem>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT id, content, timestamp, device, content_type, file_path, file_size, file_name 
         FROM clipboard_items 
         WHERE content_type = 'file'
         ORDER BY timestamp DESC 
         LIMIT ? OFFSET ?"
    ).map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map([limit, offset], |row| {
        Ok(ClipboardItem {
            id: row.get(0)?,
            content: row.get(1)?,
            timestamp: row.get(2)?,
            device: row.get(3)?,
            content_type: row.get(4)?,
            file_path: row.get(5)?,
            file_size: row.get(6)?,
            file_name: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| e.to_string())?);
    }
    
    Ok(items)
}

fn save_clipboard_item_to_db(db_path: &str, item: &ClipboardItem) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT OR REPLACE INTO clipboard_items (id, content, timestamp, device, content_type, file_path, file_size, file_name) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        [
            &item.id, 
            &item.content, 
            &item.timestamp, 
            &item.device, 
            &item.content_type,
            &item.file_path.as_ref().unwrap_or(&String::new()),
            &item.file_size.map(|s| s.to_string()).unwrap_or_default(),
            &item.file_name.as_ref().unwrap_or(&String::new()),
        ],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

fn clear_clipboard_history_from_db(db_path: &str) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    conn.execute("DELETE FROM clipboard_items", [])
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

fn delete_clipboard_item_from_db(db_path: &str, item_id: &str) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    conn.execute("DELETE FROM clipboard_items WHERE id = ?1", [item_id])
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

fn store_file_content(file_content: &[u8], file_name: &str, file_id: &str) -> Result<String, String> {
    use std::fs;
    use std::path::Path;
    
    // Get app data directory for storing files
    if let Some(proj_dirs) = ProjectDirs::from("com", "cliped", "cliped") {
        let data_dir = proj_dirs.data_dir();
        let files_dir = data_dir.join("files");
        
        // Create files directory if it doesn't exist
        fs::create_dir_all(&files_dir).map_err(|e| format!("Failed to create files directory: {}", e))?;
        
        // Extract file extension to preserve it
        let extension = Path::new(file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        // Create stored filename: file_id + original extension
        let stored_filename = if extension.is_empty() {
            file_id.to_string()
        } else {
            format!("{}.{}", file_id, extension)
        };
        
        let stored_path = files_dir.join(&stored_filename);
        
        // Write file content to storage
        fs::write(&stored_path, file_content)
            .map_err(|e| format!("Failed to write file to storage: {}", e))?;
        
        println!("File stored successfully: {} -> {}", file_name, stored_path.display());
        Ok(stored_path.to_string_lossy().to_string())
    } else {
        Err("Failed to get project directories for file storage".to_string())
    }
}

fn get_files_storage_directory() -> Result<String, String> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "cliped", "cliped") {
        let data_dir = proj_dirs.data_dir();
        let files_dir = data_dir.join("files");
        Ok(files_dir.to_string_lossy().to_string())
    } else {
        Err("Failed to get project directories".to_string())
    }
}

async fn handle_network_discovery(_app_handle: AppHandle, _state: Arc<AppState>) {
    // Placeholder for network discovery logic
    println!("Network discovery service started");
    
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        // Periodic discovery logic would go here
    }
}

// Store functionality disabled - using in-memory storage only for now

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Start UDP server for device discovery in an async task
            let app_handle_for_udp = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(udp_socket) = UdpSocket::bind("0.0.0.0:51847").await {
                    println!("UDP server listening on port 51847 for device discovery");
                    let mut buf = [0; 1024];
                    
                    loop {
                        if let Ok((len, addr)) = udp_socket.recv_from(&mut buf).await {
                            let message_str = String::from_utf8_lossy(&buf[..len]);
                            println!("Received UDP message from {}: {}", addr, message_str);
                            
                            // Try to parse as NetworkMessage
                            if let Ok(network_msg) = serde_json::from_str::<NetworkMessage>(&message_str) {
                                match network_msg.msg_type {
                                    MessageType::Discovery => {
                                        println!("Discovery request from device: {} ({})", network_msg.device_name, network_msg.device_id);
                                        
                                        // Get state to both respond and potentially add discovered device
                                        let app_state = app_handle_for_udp.state::<AppState>();
                                        
                                        // Extract data before any async operations
                                        let (should_add_device, response_msg) = {
                                            if let Ok(local_device_lock) = app_state.local_device.lock() {
                                                if let Some(ref local_device) = *local_device_lock {
                                                    let should_add = network_msg.device_id != local_device.id;
                                                    let response = NetworkMessage {
                                                        msg_type: MessageType::Discovery,
                                                        device_id: local_device.id,
                                                        device_name: local_device.name.clone(),
                                                        data: None,
                                                    };
                                                    (should_add, Some(response))
                                                } else {
                                                    (false, None)
                                                }
                                            } else {
                                                (false, None)
                                            }
                                        };
                                        
                                        // Add discovered device if needed
                                        if should_add_device {
                                            let sender_ip = addr.ip().to_string();
                                            let discovered_device = Device {
                                                id: network_msg.device_id,
                                                name: network_msg.device_name.clone(),
                                                icon: "laptop".to_string(),
                                                ip: sender_ip,
                                                status: DeviceStatus::Offline,
                                                sync_mode: SyncMode::Disabled,
                                                last_seen: get_current_timestamp(),
                                            };
                                            
                                            if let Ok(mut discovered) = app_state.discovered_devices.lock() {
                                                if !discovered.iter().any(|d| d.id == network_msg.device_id) {
                                                    discovered.push(discovered_device);
                                                    println!("Added discovered device: {} at {}", network_msg.device_name, addr.ip());
                                                }
                                            }
                                        }
                                        
                                        // Send response
                                        if let Some(response) = response_msg {
                                            if let Ok(response_json) = serde_json::to_string(&response) {
                                                // Send response back to the sender's port (not port 51847)
                                                let _ = udp_socket.send_to(response_json.as_bytes(), addr).await;
                                                println!("Sent discovery response to {}", addr);
                                            }
                                        }
                                    },
                                    MessageType::ConnectionRequest => {
                                        println!("Connection request from: {} ({})", network_msg.device_name, network_msg.device_id);
                                        
                                        // Add to pending connections
                                        let app_state = app_handle_for_udp.state::<AppState>();
                                        let sender_ip = addr.ip().to_string();
                                        let requesting_device = Device {
                                            id: network_msg.device_id,
                                            name: network_msg.device_name.clone(),
                                            icon: "laptop".to_string(),
                                            ip: sender_ip,
                                            status: DeviceStatus::Pending,
                                            sync_mode: SyncMode::Disabled,
                                            last_seen: get_current_timestamp(),
                                        };
                                        
                                        // Add to pending connections with proper scope
                                        {
                                            if let Ok(mut pending) = app_state.pending_connections.lock() {
                                                if !pending.iter().any(|d| d.id == network_msg.device_id) {
                                                    pending.push(requesting_device.clone());
                                                    println!("Added connection request from: {}", network_msg.device_name);
                                                    
                                                    // Emit event to frontend to notify of new connection request
                                                    let _ = app_handle_for_udp.emit("connection-request-received", &requesting_device);
                                                }
                                            }
                                        }
                                        
                                        // Emit event to frontend
                                        let _ = app_handle_for_udp.emit("connection-request", &network_msg);
                                    },
                                    MessageType::ConnectionAccept => {
                                        println!("Connection accepted by: {} ({})", network_msg.device_name, network_msg.device_id);
                                        
                                        // When we receive an acceptance, add the accepting device to our connected devices
                                        let app_state = app_handle_for_udp.state::<AppState>();
                                        let sender_ip = addr.ip().to_string();
                                        let accepting_device = Device {
                                            id: network_msg.device_id,
                                            name: network_msg.device_name.clone(),
                                            icon: "laptop".to_string(),
                                            ip: sender_ip,
                                            status: DeviceStatus::Connected,
                                            sync_mode: SyncMode::PartialSync, // Default to partial sync
                                            last_seen: get_current_timestamp(),
                                        };
                                        
                                        {
                                            let mut devices = app_state.devices.lock().unwrap();
                                            devices.insert(network_msg.device_id, accepting_device);
                                            println!("Added accepted connection: {} at {}", network_msg.device_name, addr.ip());
                                        }
                                        
                                        // Emit event to frontend to refresh device list
                                        let _ = app_handle_for_udp.emit("connection-accepted", &network_msg.device_id);
                                    },
                                    MessageType::ConnectionDeny => {
                                        println!("Connection denied by: {} ({})", network_msg.device_name, network_msg.device_id);
                                        // Handle connection denial
                                    },
                                    MessageType::ClipboardSync => {
                                        println!("Clipboard sync from: {} ({})", network_msg.device_name, network_msg.device_id);
                                        
                                        // Check if we have any connected devices first
                                        let app_state = app_handle_for_udp.state::<AppState>();
                                        let devices = app_state.devices.lock().unwrap();
                                        
                                        // If no connected devices, ignore all clipboard sync messages
                                        if devices.is_empty() {
                                            println!("No connected devices - ignoring clipboard sync from: {} ({})", 
                                                    network_msg.device_name, network_msg.device_id);
                                            continue;
                                        }
                                        
                                        // Check if device is actually connected and verify IP matches
                                        let sender_ip = addr.ip().to_string();
                                        let is_valid_device = devices.get(&network_msg.device_id)
                                            .map(|device| device.ip == sender_ip)
                                            .unwrap_or(false);
                                        
                                        if !is_valid_device {
                                            println!("Ignoring clipboard sync from unknown/unconnected device or wrong IP: {} ({}) from {}", 
                                                    network_msg.device_name, network_msg.device_id, sender_ip);
                                            continue;
                                        }
                                        
                                        drop(devices);
                                        
                                        // Handle incoming clipboard sync
                                        #[cfg(feature = "clipboard")]
                                        if let Some(item_data) = network_msg.data {
                                            if let Ok(synced_item) = serde_json::from_str::<ClipboardItem>(&item_data) {
                                                
                                                // Check if this content is different from what's currently in clipboard
                                                let should_update = {
                                                    if let Ok(mut clipboard) = Clipboard::new() {
                                                        if let Ok(current_text) = clipboard.get_text() {
                                                            current_text != synced_item.content
                                                        } else {
                                                            true // If we can't read clipboard, assume we should update
                                                        }
                                                    } else {
                                                        true // If we can't access clipboard, assume we should update
                                                    }
                                                };
                                                
                                                if should_update {
                                                    // Set ignore flag to prevent sync loop - the monitor will handle adding to history
                                                    {
                                                        let mut ignore = app_state.ignore_next_clipboard_change.lock().unwrap();
                                                        *ignore = true;
                                                        println!("Setting ignore flag for synced content from {}", network_msg.device_name);
                                                    }
                                                    
                                                    // Set the clipboard content - the monitor will detect this and add to history
                                                    if let Ok(mut clipboard) = Clipboard::new() {
                                                        if let Err(e) = clipboard.set_text(&synced_item.content) {
                                                            eprintln!("Failed to set clipboard content: {}", e);
                                                        } else {
                                                            println!("Set clipboard content from connected device {}: {}", 
                                                                    network_msg.device_name, 
                                                                    synced_item.content.chars().take(50).collect::<String>());
                                                        }
                                                    }
                                                } else {
                                                    println!("Synced content is same as current clipboard, skipping update");
                                                }
                                            }
                                        }
                                        
                                        #[cfg(not(feature = "clipboard"))]
                                        if let Some(_item_data) = network_msg.data {
                                            println!("Received clipboard sync but clipboard functionality not available on this platform");
                                        }
                                    },
                                    MessageType::ConnectionRemove => {
                                        println!("Connection removed by: {} ({})", network_msg.device_name, network_msg.device_id);
                                        
                                        // Remove the device from our connected devices list
                                        let app_state = app_handle_for_udp.state::<AppState>();
                                        {
                                            let mut devices = app_state.devices.lock().unwrap();
                                            devices.remove(&network_msg.device_id);
                                            println!("Removed disconnected device: {}", network_msg.device_name);
                                        }
                                        
                                        // Emit event to frontend to refresh device list
                                        let _ = app_handle_for_udp.emit("device-disconnected", &network_msg.device_id);
                                    },
                                    MessageType::Heartbeat => {
                                        println!("Heartbeat from: {} ({})", network_msg.device_name, network_msg.device_id);
                                        // Handle heartbeat
                                    },
                                    MessageType::FileTransfer => {
                                        println!("File transfer from: {} ({})", network_msg.device_name, network_msg.device_id);
                                        
                                        // Check if device is connected
                                        let app_state = app_handle_for_udp.state::<AppState>();
                                        let devices = app_state.devices.lock().unwrap();
                                        let sender_ip = addr.ip().to_string();
                                        let is_valid_device = devices.get(&network_msg.device_id)
                                            .map(|device| device.ip == sender_ip)
                                            .unwrap_or(false);
                                        
                                        if !is_valid_device {
                                            println!("Ignoring file transfer from unknown/unconnected device: {} ({})", 
                                                    network_msg.device_name, network_msg.device_id);
                                            continue;
                                        }
                                        
                                        drop(devices);
                                        
                                        // Handle incoming file transfer
                                        if let Some(file_data) = network_msg.data {
                                            if let Ok(parsed_data) = serde_json::from_str::<serde_json::Value>(&file_data) {
                                                if let (Some(item_data), Some(file_content_b64)) = (
                                                    parsed_data.get("item"),
                                                    parsed_data.get("file_content").and_then(|v| v.as_str())
                                                ) {
                                                    // Decode the file content
                                                    if let Ok(file_content) = general_purpose::STANDARD.decode(file_content_b64) {
                                                        if let Ok(received_item) = serde_json::from_value::<ClipboardItem>(item_data.clone()) {
                                                            
                                                            // Store the received file
                                                            let file_name = received_item.file_name.as_ref()
                                                                .unwrap_or(&"received_file".to_string()).clone();
                                                            
                                                            match store_file_content(&file_content, &file_name, &received_item.id) {
                                                                Ok(stored_path) => {
                                                                    // Create new item with our local storage path
                                                                    let local_item = ClipboardItem {
                                                                        id: received_item.id,
                                                                        content: received_item.content,
                                                                        timestamp: received_item.timestamp,
                                                                        device: received_item.device,
                                                                        content_type: received_item.content_type,
                                                                        file_path: Some(stored_path),
                                                                        file_size: received_item.file_size,
                                                                        file_name: received_item.file_name,
                                                                    };
                                                                    
                                                                    // Files are not added to in-memory history - only stored in database
                                                                    
                                                                    // Save to database
                                                                    let db_path = app_state.db_path.lock().unwrap().clone();
                                                                    if let Some(db_path) = db_path {
                                                                        let _ = save_clipboard_item_to_db(&db_path, &local_item);
                                                                    }
                                                                    
                                                                    // Emit to frontend
                                                                    let _ = app_handle_for_udp.emit("clipboard-updated", &local_item);
                                                                    
                                                                    println!("Received and stored file: {} ({} bytes) from {}", 
                                                                            file_name, file_content.len(), network_msg.device_name);
                                                                },
                                                                Err(e) => {
                                                                    eprintln!("Failed to store received file: {}", e);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    MessageType::FileTransferChunk => {
                                        println!("File transfer chunk from: {} ({})", network_msg.device_name, network_msg.device_id);
                                        // TODO: Handle file transfer chunk
                                    },
                                    MessageType::FileTransferComplete => {
                                        println!("File transfer complete from: {} ({})", network_msg.device_name, network_msg.device_id);
                                        // TODO: Handle file transfer completion
                                    }
                                }
                            } else {
                                println!("Failed to parse network message: {}", message_str);
                            }
                        }
                    }
                } else {
                    eprintln!("Failed to bind UDP socket on port 51847");
                }
            });

            // Initialize state
            let state: State<AppState> = app.state();
            let _clipboard_history = Arc::clone(&state.clipboard_history);
            let enabled = Arc::clone(&state.enabled);
            
            // Clear all cached/stale connected devices on startup
            {
                let mut devices = state.devices.lock().unwrap();
                devices.clear();
                println!("Cleared all cached connected devices on startup");
            }
            
            // Clear any pending connections
            {
                let mut pending = state.pending_connections.lock().unwrap();
                pending.clear();
                println!("Cleared all pending connections on startup");
            }
            
            // Clear discovered devices
            {
                let mut discovered = state.discovered_devices.lock().unwrap();
                discovered.clear();
                println!("Cleared all discovered devices on startup");
            }
            
            
            
            // Set enabled to true by default
            *enabled.lock().unwrap() = true;
            
            println!("🚀 Cliped app starting...");
            println!("✨ Beautiful UI clipboard manager ready!");

            // Start clipboard monitoring after a short delay to ensure runtime is ready
            let state: State<AppState> = app.state();
            
            let app_handle_for_monitor = app_handle.clone();
            let clipboard_history_clone = Arc::clone(&state.clipboard_history);
            let last_content_clone = Arc::clone(&state.last_clipboard_content);
            let enabled_clone = Arc::clone(&state.enabled);
            let devices_clone = Arc::clone(&state.devices);
            let local_device_clone = Arc::clone(&state.local_device);
            tauri::async_runtime::spawn(async move {
                // Small delay to ensure everything is initialized
                tokio::time::sleep(Duration::from_millis(100)).await;
                monitor_clipboard(app_handle_for_monitor, clipboard_history_clone, last_content_clone, enabled_clone, devices_clone, local_device_clone).await;
            });

            // Initialize database and load existing history
            match init_database() {
                Ok(path) => {
                    println!("Database initialized at: {}", path);
                    
                    // Load existing clipboard history from database
                    match load_clipboard_history_from_db(&path) {
                        Ok(history) => {
                            let mut clipboard_state = state.clipboard_history.lock().unwrap();
                            *clipboard_state = history;
                            println!("Loaded {} items from database", clipboard_state.len());
                        },
                        Err(e) => {
                            eprintln!("Failed to load clipboard history: {}", e);
                        }
                    }
                    
                    // Store the database path
                    *state.db_path.lock().unwrap() = Some(path.clone());
                },
                Err(e) => {
                    eprintln!("Failed to initialize database: {}", e);
                }
            };

            // Generate and set local device info
            let local_device = generate_device_info();
            {
                let mut devices = state.devices.lock().unwrap();
                devices.insert(local_device.id, local_device.clone());
            }
            *state.local_device.lock().unwrap() = Some(local_device);

            // Start network discovery service
            let state_arc = Arc::new(AppState::default()); // We'll initialize properly later
            let state_for_discovery = Arc::clone(&state_arc);
            tauri::async_runtime::spawn(async move {
                handle_network_discovery(app_handle, state_for_discovery).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_clipboard_history,
            get_clipboard_history_paginated,
            get_clipboard_history_count,
            get_clipboard_files_count,
            get_clipboard_files_paginated,
            clear_clipboard_history,
            delete_clipboard_item,
            set_clipboard_content,
            toggle_monitoring,
            is_monitoring_enabled,
            add_clipboard_item,
            add_device,
            remove_device,
            sync_clipboard,
            get_local_device,
            get_connected_devices,
            send_connection_request,
            accept_connection,
            deny_connection,
            get_pending_connections,
            set_sync_mode,
            discover_devices,
            update_device_name,
            send_connection_request_to_device,
            add_file_to_clipboard,
            get_file_content,
            save_received_file,
            save_file_to_path,
            show_open_dialog,
            show_save_dialog,
            get_file_preview,
            get_files_storage_directory_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn main() {
    run();
}

#[cfg(feature = "clipboard")]
async fn monitor_clipboard(
    app_handle: AppHandle,
    clipboard_history: ClipboardState,
    last_content: Arc<Mutex<String>>,
    enabled: Arc<Mutex<bool>>,
    devices: Arc<Mutex<HashMap<u32, Device>>>,
    local_device: Arc<Mutex<Option<Device>>>,
) {
    println!("Clipboard monitoring started!");
    let mut clipboard = Clipboard::new().unwrap();
    
    // Get database path and ignore flag
    let (db_path, ignore_flag) = {
        let app_state = app_handle.state::<AppState>();
        let db_path = app_state.db_path.lock().unwrap().clone();
        let ignore_flag = Arc::clone(&app_state.ignore_next_clipboard_change);
        (db_path, ignore_flag)
    };
    
    // Check if clipboard is available first
    if clipboard.get_text().is_err() {
        println!("Clipboard not available on this platform - skipping clipboard monitoring");
        return;
    }
    
    loop {
        sleep(Duration::from_millis(500)).await;
        
        // Check if monitoring is enabled
        if !*enabled.lock().unwrap() {
            continue;
        }
        
        if let Ok(text) = clipboard.get_text() {
            let should_process = {
                let mut last = last_content.lock().unwrap();
                let mut ignore = ignore_flag.lock().unwrap();
                
                // Check if we should ignore this change (it's from a sync)
                if *ignore {
                    println!("Ignoring clipboard change from sync");
                    *ignore = false;
                    *last = text.clone(); // Update last content to avoid future triggers
                    false
                } else if text != *last && !text.trim().is_empty() {
                    println!("New clipboard content detected: {}", text.chars().take(50).collect::<String>());
                    *last = text.clone();
                    true
                } else {
                    false
                }
            }; // Drop the locks here
            
            if should_process {
                let item = ClipboardItem {
                    id: generate_id().to_string(),
                    content: text,
                    timestamp: get_current_timestamp().to_string(),
                    device: whoami::fallible::hostname().unwrap_or("Unknown".to_string()),
                    content_type: "text".to_string(),
                    file_path: None,
                    file_size: None,
                    file_name: None,
                };

                // Add to local history first
                {
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
                } // Drop the history lock here

                // Save to database
                if let Some(ref db_path) = db_path {
                    if let Err(e) = save_clipboard_item_to_db(db_path, &item) {
                        eprintln!("Failed to save clipboard item to database: {}", e);
                    }
                }

                // Check if we have connected devices before syncing
                let has_connected_devices = {
                    let devices = devices.lock().unwrap();
                    devices.values().any(|device| {
                        matches!(device.status, DeviceStatus::Connected) &&
                        !matches!(device.sync_mode, SyncMode::Disabled)
                    })
                };

                // Only sync if we have connected devices with sync enabled
                if has_connected_devices {
                    sync_to_connected_devices(&devices, &local_device, &item).await;
                } else {
                    println!("No connected devices with sync enabled - skipping clipboard sync");
                }

                // Emit to frontend
                let _ = app_handle.emit("clipboard-updated", &item);
                println!("Emitted clipboard-updated event");
            }
        }
    }
}

#[cfg(not(feature = "clipboard"))]
async fn monitor_clipboard(
    _app_handle: AppHandle,
    _clipboard_history: ClipboardState,
    _last_content: Arc<Mutex<String>>,
    _enabled: Arc<Mutex<bool>>,
    _devices: Arc<Mutex<HashMap<u32, Device>>>,
    _local_device: Arc<Mutex<Option<Device>>>,
) {
    println!("Clipboard monitoring not available on this platform (mobile)");
    // On mobile, clipboard monitoring is handled differently or not available
    // This function exists to satisfy the type system but does nothing
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

async fn sync_to_connected_devices(
    devices: &Arc<Mutex<HashMap<u32, Device>>>, 
    local_device: &Arc<Mutex<Option<Device>>>, 
    item: &ClipboardItem
) {
    // Get connected devices and local device info - get fresh data each time
    let (devices_to_sync, local) = {
        let devices = devices.lock().unwrap();
        let local = local_device.lock().unwrap();
        
        // Filter devices to sync to (get fresh data, don't clone the entire HashMap)
        let devices_to_sync: Vec<Device> = devices
            .values()
            .filter(|device| {
                matches!(device.status, DeviceStatus::Connected) &&
                !matches!(device.sync_mode, SyncMode::Disabled) &&
                device.id != local.as_ref().map(|l| l.id).unwrap_or(0) // Don't sync to ourselves
            })
            .cloned()
            .collect();
        
        (devices_to_sync, local.clone())
    };
    
    // If no connected devices, don't send any broadcasts
    if devices_to_sync.is_empty() {
        println!("No connected devices with sync enabled - skipping all clipboard sync broadcasts");
        return;
    }
    
    if let Some(local) = local {
        println!("Syncing clipboard item to {} connected devices", devices_to_sync.len());
        
        // Only send to specific connected devices, no broadcasting
        for device in devices_to_sync {
            // Create sync message
            let message = NetworkMessage {
                msg_type: MessageType::ClipboardSync,
                device_id: local.id,
                device_name: local.name.clone(),
                data: Some(serde_json::to_string(item).unwrap_or_default()),
            };
            
            // Send directly to specific device IP
            if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
                let message_json = serde_json::to_string(&message).unwrap_or_default();
                let target_addr = format!("{}:51847", device.ip);
                let _ = socket.send_to(message_json.as_bytes(), &target_addr).await;
                println!("Synced clipboard to connected device: {} at {}", device.name, device.ip);
            }
        }
    }
}

async fn sync_file_to_connected_devices(
    devices: &Arc<Mutex<HashMap<u32, Device>>>, 
    local_device: &Arc<Mutex<Option<Device>>>, 
    item: &ClipboardItem,
    file_content: &[u8]
) {
    // Get connected devices and local device info
    let (devices_to_sync, local) = {
        let devices = devices.lock().unwrap();
        let local = local_device.lock().unwrap();
        
        // Filter devices to sync to
        let devices_to_sync: Vec<Device> = devices
            .values()
            .filter(|device| {
                matches!(device.status, DeviceStatus::Connected) &&
                !matches!(device.sync_mode, SyncMode::Disabled) &&
                device.id != local.as_ref().map(|l| l.id).unwrap_or(0)
            })
            .cloned()
            .collect();
        
        (devices_to_sync, local.clone())
    };
    
    if devices_to_sync.is_empty() {
        println!("No connected devices with sync enabled - skipping file sync");
        return;
    }
    
    if let Some(local) = local {
        println!("Syncing file to {} connected devices: {} ({} bytes)", 
                devices_to_sync.len(), 
                item.file_name.as_ref().unwrap_or(&"unknown".to_string()),
                file_content.len());
        
        for device in devices_to_sync {
            // Create file transfer message with complete file content
            let file_data = serde_json::json!({
                "item": item,
                "file_content": general_purpose::STANDARD.encode(file_content)
            });
            
            let message = NetworkMessage {
                msg_type: MessageType::FileTransfer,
                device_id: local.id,
                device_name: local.name.clone(),
                data: Some(file_data.to_string()),
            };
            
            // Send directly to specific device IP
            if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
                let message_json = serde_json::to_string(&message).unwrap_or_default();
                let target_addr = format!("{}:51847", device.ip);
                let _ = socket.send_to(message_json.as_bytes(), &target_addr).await;
                println!("Synced file to connected device: {} at {}", device.name, device.ip);
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
async fn get_clipboard_history_paginated(state: State<'_, AppState>, offset: u32, limit: u32) -> Result<Vec<ClipboardItem>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    if let Some(db_path) = db_path {
        load_clipboard_history_paginated(&db_path, offset, limit)
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
async fn get_clipboard_history_count(state: State<'_, AppState>) -> Result<u32, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    if let Some(db_path) = db_path {
        get_clipboard_history_count_from_db(&db_path)
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
async fn get_clipboard_files_count(state: State<'_, AppState>) -> Result<u32, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    if let Some(db_path) = db_path {
        get_clipboard_files_count_from_db(&db_path)
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
async fn get_clipboard_files_paginated(state: State<'_, AppState>, offset: u32, limit: u32) -> Result<Vec<ClipboardItem>, String> {
    let db_path = state.db_path.lock().unwrap().clone();
    if let Some(db_path) = db_path {
        get_clipboard_files_paginated_from_db(&db_path, offset, limit)
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
async fn clear_clipboard_history(state: State<'_, AppState>) -> Result<(), String> {
    // Clear in-memory history
    {
        let mut history = state.clipboard_history.lock().unwrap();
        history.clear();
    }
    
    // Clear database
    let db_path = state.db_path.lock().unwrap().clone();
    if let Some(db_path) = db_path {
        if let Err(e) = clear_clipboard_history_from_db(&db_path) {
            eprintln!("Failed to clear clipboard history from database: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

#[tauri::command]
async fn delete_clipboard_item(state: State<'_, AppState>, id: String) -> Result<(), String> {
    // Delete from in-memory history
    {
        let mut history = state.clipboard_history.lock().unwrap();
        history.retain(|item| item.id != id);
    }
    
    // Delete from database
    let db_path = state.db_path.lock().unwrap().clone();
    if let Some(db_path) = db_path {
        if let Err(e) = delete_clipboard_item_from_db(&db_path, &id) {
            eprintln!("Failed to delete clipboard item from database: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

#[cfg(feature = "clipboard")]
#[tauri::command]
async fn set_clipboard_content(content: String, state: State<'_, AppState>) -> Result<(), String> {
    // Set ignore flag to prevent the monitor from detecting this as a new change
    {
        let mut ignore = state.ignore_next_clipboard_change.lock().unwrap();
        *ignore = true;
    }
    
    if let Ok(mut clipboard) = Clipboard::new() {
        clipboard.set_text(content).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(not(feature = "clipboard"))]
#[tauri::command]
async fn set_clipboard_content(_content: String, _state: State<'_, AppState>) -> Result<(), String> {
    Err("Clipboard functionality not available on this platform".to_string())
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

#[tauri::command]
fn add_device(state: State<AppState>, device: Device) {
    let mut devices = state.devices.lock().unwrap();
    devices.insert(device.id, device);
}

#[tauri::command]
async fn remove_device(state: State<'_, AppState>, device_id: u32) -> Result<(), String> {
    // Get device info before removing it
    let device_to_remove = {
        let devices = state.devices.lock().unwrap();
        devices.get(&device_id).cloned()
    };
    
    if let Some(device) = device_to_remove {
        // Get local device info for the disconnection message
        let local_device = {
            let local = state.local_device.lock().unwrap();
            local.clone()
        };
        
        // Send disconnection message to the device being removed
        if let Some(local) = local_device {
            let message = NetworkMessage {
                msg_type: MessageType::ConnectionRemove,
                device_id: local.id,
                device_name: local.name,
                data: None,
            };
            
            if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
                let message_json = serde_json::to_string(&message).map_err(|e| e.to_string())?;
                let target_addr = format!("{}:51847", device.ip);
                let _ = socket.send_to(message_json.as_bytes(), &target_addr).await;
                println!("Sent disconnection notice to {} at {}", device.name, device.ip);
            }
        }
        
        // Remove from local devices list
        {
            let mut devices = state.devices.lock().unwrap();
            let removed = devices.remove(&device_id);
            println!("Device removal from HashMap: {:?}", removed.is_some());
            println!("Remaining connected devices: {}", devices.len());
            for (id, dev) in devices.iter() {
                println!("  - {} (ID: {}): {:?} at {}", dev.name, id, dev.status, dev.ip);
            }
        }
        
        println!("Removed device: {} ({})", device.name, device_id);
        Ok(())
    } else {
        Err("Device not found".to_string())
    }
}

#[tauri::command]
fn sync_clipboard(state: State<AppState>, item: ClipboardItem) {
    let mut history = state.clipboard_history.lock().unwrap();
    history.push(item);
}

#[tauri::command]
fn get_local_device(state: State<AppState>) -> Option<Device> {
    state.local_device.lock().unwrap().clone()
}

#[tauri::command]
fn get_connected_devices(state: State<AppState>) -> Vec<Device> {
    let devices = state.devices.lock().unwrap();
    devices.values().cloned().collect()
}

#[tauri::command]
async fn send_connection_request(state: State<'_, AppState>, ip_or_tag: String) -> Result<(), String> {
    let local_device = state.local_device.lock().unwrap().clone();
    if let Some(device) = local_device {
        let message = NetworkMessage {
            msg_type: MessageType::ConnectionRequest,
            device_id: device.id,
            device_name: device.name,
            data: None,
        };
        
        // Parse IP or tag
        let target_ip = if ip_or_tag.starts_with('#') {
            // TODO: Resolve tag to IP through device discovery
            return Err("Tag resolution not yet implemented".to_string());
        } else {
            ip_or_tag
        };
        
        // Send UDP message
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
            let message_json = serde_json::to_string(&message).map_err(|e| e.to_string())?;
            let target_addr = format!("{}:51847", target_ip);
            if let Err(e) = socket.send_to(message_json.as_bytes(), &target_addr).await {
                return Err(format!("Failed to send connection request: {}", e));
            }
            println!("Connection request sent to {}", target_addr);
            Ok(())
        } else {
            Err("Failed to create UDP socket".to_string())
        }
    } else {
        Err("Local device not initialized".to_string())
    }
}

#[tauri::command]
async fn accept_connection(state: State<'_, AppState>, device_id: u32) -> Result<(), String> {
    // Extract data from locks before any async operations
    let device_opt = {
        let mut pending = state.pending_connections.lock().unwrap();
        if let Some(pos) = pending.iter().position(|d| d.id == device_id) {
            let mut device = pending.remove(pos);
            device.status = DeviceStatus::Connected;
            device.sync_mode = SyncMode::PartialSync; // Default to partial sync
            Some(device)
        } else {
            None
        }
    };
    
    if let Some(device) = device_opt {
        // Add to connected devices
        {
            let mut devices = state.devices.lock().unwrap();
            devices.insert(device_id, device.clone());
        }
        
        // Get local device info
        let local_device = {
            let local = state.local_device.lock().unwrap();
            local.clone()
        };
        
        // Send acceptance message
        if let Some(local) = local_device {
            let message = NetworkMessage {
                msg_type: MessageType::ConnectionAccept,
                device_id: local.id,
                device_name: local.name,
                data: None,
            };
            
            if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
                let message_json = serde_json::to_string(&message).map_err(|e| e.to_string())?;
                let target_addr = format!("{}:51847", device.ip);
                let _ = socket.send_to(message_json.as_bytes(), &target_addr).await;
            }
        }
        
        println!("Connection accepted for device: {}", device.name);
        Ok(())
    } else {
        Err("Device not found in pending connections".to_string())
    }
}

#[tauri::command]
async fn deny_connection(state: State<'_, AppState>, device_id: u32) -> Result<(), String> {
    // Extract data from locks before any async operations
    let device_opt = {
        let mut pending = state.pending_connections.lock().unwrap();
        if let Some(pos) = pending.iter().position(|d| d.id == device_id) {
            Some(pending.remove(pos))
        } else {
            None
        }
    };
    
    if let Some(device) = device_opt {
        // Get local device info
        let local_device = {
            let local = state.local_device.lock().unwrap();
            local.clone()
        };
        
        // Send denial message
        if let Some(local) = local_device {
            let message = NetworkMessage {
                msg_type: MessageType::ConnectionDeny,
                device_id: local.id,
                device_name: local.name,
                data: None,
            };
            
            if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
                let message_json = serde_json::to_string(&message).map_err(|e| e.to_string())?;
                let target_addr = format!("{}:51847", device.ip);
                let _ = socket.send_to(message_json.as_bytes(), &target_addr).await;
            }
        }
        
        println!("Connection denied for device: {}", device.name);
        Ok(())
    } else {
        Err("Device not found in pending connections".to_string())
    }
}

#[tauri::command]
fn get_pending_connections(state: State<AppState>) -> Vec<Device> {
    state.pending_connections.lock().unwrap().clone()
}

#[tauri::command]
async fn set_sync_mode(state: State<'_, AppState>, device_id: u32, sync_mode: String) -> Result<(), String> {
    // Parse sync mode first
    let parsed_sync_mode = match sync_mode.as_str() {
        "total" => SyncMode::TotalSync,
        "partial" => SyncMode::PartialSync,
        "disabled" => SyncMode::Disabled,
        _ => return Err("Invalid sync mode".to_string()),
    };
    
    // Extract data before async operations
    let (device_info, history, local_device) = {
        let mut devices = state.devices.lock().unwrap();
        if let Some(device) = devices.get_mut(&device_id) {
            device.sync_mode = parsed_sync_mode.clone();
            let device_info = (device.ip.clone(), device.name.clone());
            
            // Get history and local device if needed for total sync
            let history = if matches!(parsed_sync_mode, SyncMode::TotalSync) {
                state.clipboard_history.lock().unwrap().clone()
            } else {
                Vec::new()
            };
            
            let local_device = state.local_device.lock().unwrap().clone();
            
            (Some(device_info), history, local_device)
        } else {
            (None, Vec::new(), None)
        }
    };
    
    if let Some((device_ip, device_name)) = device_info {
        // If switching to total sync, send entire history
        if matches!(parsed_sync_mode, SyncMode::TotalSync) && !history.is_empty() {
            if let Some(local) = local_device {
                for item in history {
                    // Send each item to the device
                    let message = NetworkMessage {
                        msg_type: MessageType::ClipboardSync,
                        device_id: local.id,
                        device_name: local.name.clone(),
                        data: Some(serde_json::to_string(&item).unwrap_or_default()),
                    };
                    
                    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
                        let message_json = serde_json::to_string(&message).unwrap_or_default();
                        let target_addr = format!("{}:51847", device_ip);
                        let _ = socket.send_to(message_json.as_bytes(), &target_addr).await;
                    }
                }
                println!("Total sync initiated for device: {}", device_name);
            }
        }
        
        println!("Sync mode updated for {}: {:?}", device_name, parsed_sync_mode);
        Ok(())
    } else {
        Err("Device not found".to_string())
    }
}

#[tauri::command]
async fn discover_devices(state: State<'_, AppState>) -> Result<Vec<Device>, String> {
    println!("Starting device discovery...");
    
    // Clear previous discoveries
    {
        let mut discovered = state.discovered_devices.lock().unwrap();
        discovered.clear();
    }
    
    // Get local device info to broadcast
    let local_device = {
        let local = state.local_device.lock().unwrap();
        local.clone()
    };
    
    if let Some(local) = local_device {
        // Create discovery message
        let discovery_message = NetworkMessage {
            msg_type: MessageType::Discovery,
            device_id: local.id,
            device_name: local.name.clone(),
            data: None,
        };
        
        // Broadcast discovery message to the network
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
            let message_json = serde_json::to_string(&discovery_message).map_err(|e| e.to_string())?;
            
            // Get the local port this socket is bound to
            let local_port = socket.local_addr().map_err(|e| e.to_string())?.port();
            println!("Discovery socket listening on port {}", local_port);
            
            // Broadcast to local network
            let local_ip = get_local_ip();
            let ip_parts: Vec<&str> = local_ip.split('.').collect();
            
            if ip_parts.len() == 4 {
                let network_base = format!("{}.{}.{}", ip_parts[0], ip_parts[1], ip_parts[2]);
                
                // Try broadcasting to common IP ranges
                for i in 1..255 {
                    let target_ip = format!("{}.{}", network_base, i);
                    if target_ip != local_ip {  // Don't send to ourselves
                        let target_addr = format!("{}:51847", target_ip);
                        let _ = socket.send_to(message_json.as_bytes(), &target_addr).await;
                    }
                }
                
                println!("Discovery broadcast sent to network {}.x", network_base);
            }
            
            // Listen for responses on this socket
            let mut buf = [0; 1024];
            let start_time = tokio::time::Instant::now();
            let timeout = tokio::time::Duration::from_millis(3000); // 3 second timeout
            
            while tokio::time::Instant::now().duration_since(start_time) < timeout {
                // Set a shorter timeout for each receive attempt
                let receive_timeout = tokio::time::timeout(
                    tokio::time::Duration::from_millis(100), 
                    socket.recv_from(&mut buf)
                ).await;
                
                if let Ok(Ok((len, addr))) = receive_timeout {
                    let message_str = String::from_utf8_lossy(&buf[..len]);
                    println!("Discovery response from {}: {}", addr, message_str);
                    
                    // Try to parse as NetworkMessage
                    if let Ok(network_msg) = serde_json::from_str::<NetworkMessage>(&message_str) {
                        if matches!(network_msg.msg_type, MessageType::Discovery) && network_msg.device_id != local.id {
                            let sender_ip = addr.ip().to_string();
                            let discovered_device = Device {
                                id: network_msg.device_id,
                                name: network_msg.device_name.clone(),
                                icon: "laptop".to_string(),
                                ip: sender_ip.clone(),
                                status: DeviceStatus::Offline,
                                sync_mode: SyncMode::Disabled,
                                last_seen: get_current_timestamp(),
                            };
                            
                            // Add to discovered devices
                            {
                                let mut discovered = state.discovered_devices.lock().unwrap();
                                if !discovered.iter().any(|d| d.id == network_msg.device_id) {
                                    discovered.push(discovered_device);
                                    println!("Added discovered device: {} at {}", network_msg.device_name, sender_ip);
                                }
                            }
                        }
                    }
                }
            }
            
            // Return discovered devices
            let discovered = state.discovered_devices.lock().unwrap();
            let result = discovered.clone();
            println!("Discovery scan completed. Found {} devices.", result.len());
            Ok(result)
        } else {
            Err("Failed to create UDP socket for discovery".to_string())
        }
    } else {
        Err("Local device not initialized".to_string())
    }
}

#[tauri::command]
async fn update_device_name(state: State<'_, AppState>, new_name: String) -> Result<(), String> {
    // Update local device name
    let mut local_device = state.local_device.lock().unwrap();
    if let Some(ref mut device) = *local_device {
        device.name = new_name.clone();
        
        // Also update in the devices map
        let mut devices = state.devices.lock().unwrap();
        if let Some(device_in_map) = devices.get_mut(&device.id) {
            device_in_map.name = new_name;
        }
    }
    
    Ok(())
}

#[tauri::command]
async fn send_connection_request_to_device(state: State<'_, AppState>, target_device: Device) -> Result<(), String> {
    let local_device = state.local_device.lock().unwrap().clone();
    if let Some(device) = local_device {
        let message = NetworkMessage {
            msg_type: MessageType::ConnectionRequest,
            device_id: device.id,
            device_name: device.name,
            data: None,
        };
        
        // Send UDP message to target device
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
            let message_json = serde_json::to_string(&message).map_err(|e| e.to_string())?;
            let target_addr = format!("{}:51847", target_device.ip);
            if let Err(e) = socket.send_to(message_json.as_bytes(), &target_addr).await {
                return Err(format!("Failed to send connection request: {}", e));
            }
            println!("Connection request sent to {} at {}", target_device.name, target_addr);
            Ok(())
        } else {
            Err("Failed to create UDP socket".to_string())
        }
    } else {
        Err("Local device not initialized".to_string())
    }
}

#[tauri::command]
async fn add_file_to_clipboard(state: State<'_, AppState>, file_path: String) -> Result<(), String> {
    use std::fs;
    use std::path::Path;
    
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("File does not exist".to_string());
    }
    
    let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    // Check file size limit (10MB)
    const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB
    if metadata.len() > MAX_FILE_SIZE {
        return Err(format!("File '{}' is too large ({}MB). Maximum size is 10MB.", 
                          file_name, metadata.len() / 1024 / 1024));
    }
    
    // Allow any file format - no restrictions on file type
    
    // Read the full file content into memory
    println!("Reading file content: {} ({} bytes)", file_name, metadata.len());
    let file_content = fs::read(&file_path).map_err(|e| format!("Failed to read file: {}", e))?;
    println!("Successfully read {} bytes from file", file_content.len());
    
    // Create a unique file ID and store the file in our files directory
    let file_id = generate_id().to_string();
    let stored_file_path = store_file_content(&file_content, &file_name, &file_id)?;
    println!("Stored file at: {}", stored_file_path);
    
    let item = ClipboardItem {
        id: file_id.clone(),
        content: format!("File: {} ({} bytes)", file_name, file_content.len()),
        timestamp: get_current_timestamp().to_string(),
        device: whoami::fallible::hostname().unwrap_or("Unknown".to_string()),
        content_type: "file".to_string(),
        file_path: Some(stored_file_path), // Now points to our stored copy
        file_size: Some(metadata.len()),
        file_name: Some(file_name),
    };
    
    // Files are not added to in-memory history - they're only stored in database
    // and retrieved via files-specific queries
    
    // Save to database
    let db_path = state.db_path.lock().unwrap().clone();
    if let Some(db_path) = db_path {
        save_clipboard_item_to_db(&db_path, &item)?;
    }
    
    // Sync to connected devices with full file content
    sync_file_to_connected_devices(&state.devices, &state.local_device, &item, &file_content).await;
    
    Ok(())
}

#[tauri::command]
async fn get_file_content(file_path: String) -> Result<Vec<u8>, String> {
    use std::fs;
    
    fs::read(&file_path).map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
async fn save_received_file(content: Vec<u8>, file_name: String) -> Result<String, String> {
    use std::fs;
    
    // Save to Downloads folder
    let downloads_dir = dirs::download_dir()
        .ok_or("Could not find downloads directory".to_string())?;
    
    let file_path = downloads_dir.join(&file_name);
    
    // Handle file name conflicts
    let mut final_path = file_path.clone();
    let mut counter = 1;
    while final_path.exists() {
        let stem = file_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let extension = file_path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        let new_name = if extension.is_empty() {
            format!("{} ({})", stem, counter)
        } else {
            format!("{} ({}).{}", stem, counter, extension)
        };
        
        final_path = downloads_dir.join(new_name);
        counter += 1;
    }
    
    fs::write(&final_path, content)
        .map_err(|e| format!("Failed to save file: {}", e))?;
    
    Ok(final_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn save_file_to_path(content: Vec<u8>, file_path: String) -> Result<String, String> {
    use std::fs;
    
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to save file: {}", e))?;
    
    Ok(file_path)
}

#[tauri::command]
async fn show_open_dialog(title: String, multiple: bool) -> Result<Option<String>, String> {
    println!("Opening file dialog with title: {}", title);
    
    let dialog = FileDialog::new()
        .set_title(&title);
    
    if multiple {
        // For now, just return the first file if multiple is true
        let files = dialog.pick_files();
        if let Some(files) = files {
            if let Some(first_file) = files.first() {
                let path = first_file.to_string_lossy().to_string();
                println!("Selected file: {}", path);
                return Ok(Some(path));
            }
        }
    } else {
        let file = dialog.pick_file();
        if let Some(file) = file {
            let path = file.to_string_lossy().to_string();
            println!("Selected file: {}", path);
            return Ok(Some(path));
        }
    }
    
    println!("No file selected");
    Ok(None)
}

#[tauri::command]
async fn show_save_dialog(suggested_name: String) -> Result<Option<String>, String> {
    println!("Opening save dialog with suggested name: {}", suggested_name);
    
    let dialog = FileDialog::new()
        .set_title("Save file as...")
        .set_file_name(&suggested_name);
    
    let file = dialog.save_file();
    if let Some(file) = file {
        let path = file.to_string_lossy().to_string();
        println!("Save location selected: {}", path);
        return Ok(Some(path));
    }
    
    println!("Save dialog cancelled");
    Ok(None)
}

#[tauri::command]
async fn get_file_preview(file_path: String, max_length: Option<usize>) -> Result<Option<String>, String> {
    use std::fs;
    use std::path::Path;
    
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("File does not exist".to_string());
    }
    
    // Get file extension to determine if it's likely a text file
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    // List of text-based file extensions
    let text_extensions = [
        "txt", "md", "json", "xml", "html", "htm", "css", "js", "ts", "jsx", "tsx",
        "py", "rs", "go", "java", "c", "cpp", "h", "hpp", "cs", "php", "rb", "pl",
        "sh", "bash", "zsh", "fish", "ps1", "bat", "cmd", "sql", "log", "cfg", "conf",
        "ini", "toml", "yaml", "yml", "csv", "tsv", "rtf", "tex", "dockerfile", "gitignore",
        "readme", "license", "changelog", "makefile", "cmake", "vcxproj", "csproj",
        "swift", "kt", "scala", "clj", "hs", "elm", "dart", "lua", "r", "jl", "m", "mm"
    ];
    
    if !text_extensions.contains(&extension.as_str()) {
        return Ok(None); // Not a text file, no preview available
    }
    
    // Try to read the file as text
    match fs::read_to_string(&file_path) {
        Ok(content) => {
            let max_len = max_length.unwrap_or(200); // Default to 200 characters
            if content.len() <= max_len {
                Ok(Some(content))
            } else {
                // Truncate at word boundary if possible
                let truncated = &content[..max_len];
                if let Some(last_space) = truncated.rfind(' ') {
                    Ok(Some(format!("{}...", &content[..last_space])))
                } else {
                    Ok(Some(format!("{}...", truncated)))
                }
            }
        },
        Err(_) => {
            // File exists but couldn't be read as text (binary file, encoding issues, etc.)
            Ok(None)
        }
    }
}

#[tauri::command]
async fn get_files_storage_directory_path() -> Result<String, String> {
    get_files_storage_directory()
}
