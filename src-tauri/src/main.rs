#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::net::UdpSocket;
use tokio::time::{sleep, Duration};
use local_ip_address::local_ip;
use rusqlite::{Connection, Result as SqlResult};
use directories::ProjectDirs;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
enum DeviceStatus {
    Pending,    // Connection request sent/received
    Connected,  // Accepted and connected
    Denied,     // Connection denied
    Offline,    // Device not responding
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    ClipboardSync,    // Sync clipboard item
    Heartbeat,        // Keep connection alive
}

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
    devices: Arc<Mutex<HashMap<u32, Device>>>,
    clipboard_history: ClipboardState,
    last_clipboard_content: Arc<Mutex<String>>,
    enabled: Arc<Mutex<bool>>,
    local_device: Arc<Mutex<Option<Device>>>,
    db_path: Arc<Mutex<Option<String>>>,
    pending_connections: Arc<Mutex<Vec<Device>>>,
    discovered_devices: Arc<Mutex<Vec<Device>>>,
}

// Utility functions
fn init_database() -> Result<String, String> {
    use rusqlite::{Connection, Result as SqlResult};
    use directories::ProjectDirs;
    
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
                content_type TEXT NOT NULL
            )",
            [],
        ).map_err(|e| e.to_string())?;
        
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

async fn handle_network_discovery(_app_handle: AppHandle, _state: Arc<AppState>) {
    // Placeholder for network discovery logic
    println!("Network discovery service started");
    
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        // Periodic discovery logic would go here
    }
}

// Store functionality disabled - using in-memory storage only for now

fn main() {
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
                                        // Handle connection acceptance
                                    },
                                    MessageType::ConnectionDeny => {
                                        println!("Connection denied by: {} ({})", network_msg.device_name, network_msg.device_id);
                                        // Handle connection denial
                                    },
                                    MessageType::ClipboardSync => {
                                        println!("Clipboard sync from: {} ({})", network_msg.device_name, network_msg.device_id);
                                        
                                        // Handle incoming clipboard sync
                                        if let Some(item_data) = network_msg.data {
                                            if let Ok(synced_item) = serde_json::from_str::<ClipboardItem>(&item_data) {
                                                let app_state = app_handle_for_udp.state::<AppState>();
                                                
                                                // Add to clipboard history without syncing back (prevent loops)
                                                {
                                                    let mut history = app_state.clipboard_history.lock().unwrap();
                                                    
                                                    // Remove duplicates
                                                    history.retain(|existing| existing.content != synced_item.content);
                                                    
                                                    // Insert at beginning
                                                    history.insert(0, synced_item.clone());
                                                    
                                                    // Limit to 50 items
                                                    if history.len() > 50 {
                                                        history.truncate(50);
                                                    }
                                                }
                                                
                                                // Emit to frontend
                                                let _ = app_handle_for_udp.emit("clipboard-updated", &synced_item);
                                                println!("Added synced clipboard item from {}", network_msg.device_name);
                                            }
                                        }
                                    },
                                    MessageType::Heartbeat => {
                                        println!("Heartbeat from: {} ({})", network_msg.device_name, network_msg.device_id);
                                        // Handle heartbeat
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

            // Initialize database
            match init_database() {
                Ok(db_path) => {
                    println!("Database initialized at: {}", db_path);
                },
                Err(e) => {
                    eprintln!("Failed to initialize database: {}", e);
                }
            }

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
            send_connection_request_to_device
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

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
    
    loop {
        sleep(Duration::from_millis(500)).await;
        
        // Check if monitoring is enabled
        if !*enabled.lock().unwrap() {
            continue;
        }
        
        if let Ok(text) = clipboard.get_text() {
            let should_process = {
                let mut last = last_content.lock().unwrap();
                if text != *last && !text.trim().is_empty() {
                    println!("New clipboard content detected: {}", text.chars().take(50).collect::<String>());
                    *last = text.clone();
                    true
                } else {
                    false
                }
            }; // Drop the lock here
            
            if should_process {

                let item = ClipboardItem {
                    id: generate_id().to_string(),
                    content: text,
                    timestamp: get_current_timestamp().to_string(),
                    device: whoami::fallible::hostname().unwrap_or("Unknown".to_string()),
                    content_type: "text".to_string(),
                };

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

                // Sync to all connected devices
                sync_to_connected_devices(&devices, &local_device, &item).await;

                // Emit to frontend
                let _ = app_handle.emit("clipboard-updated", &item);
                println!("Emitted clipboard-updated event");
            }
        }
    }
}

async fn sync_to_connected_devices(
    devices: &Arc<Mutex<HashMap<u32, Device>>>, 
    local_device: &Arc<Mutex<Option<Device>>>, 
    item: &ClipboardItem
) {
    // Get connected devices and local device info
    let (connected_devices, local) = {
        let devices = devices.lock().unwrap();
        let local = local_device.lock().unwrap();
        (devices.clone(), local.clone())
    };
    
    if let Some(local) = local {
        // Only sync to devices with enabled sync modes
        let devices_to_sync: Vec<Device> = connected_devices
            .values()
            .filter(|device| {
                matches!(device.status, DeviceStatus::Connected) &&
                !matches!(device.sync_mode, SyncMode::Disabled)
            })
            .cloned()
            .collect();
        
        if !devices_to_sync.is_empty() {
            println!("Syncing clipboard item to {} connected devices", devices_to_sync.len());
            
            for device in devices_to_sync {
                // Create sync message
                let message = NetworkMessage {
                    msg_type: MessageType::ClipboardSync,
                    device_id: local.id,
                    device_name: local.name.clone(),
                    data: Some(serde_json::to_string(item).unwrap_or_default()),
                };
                
                // Send to device
                if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
                    let message_json = serde_json::to_string(&message).unwrap_or_default();
                    let target_addr = format!("{}:51847", device.ip);
                    let _ = socket.send_to(message_json.as_bytes(), &target_addr).await;
                    println!("Synced clipboard to device: {} at {}", device.name, device.ip);
                }
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

#[tauri::command]
fn add_device(state: State<AppState>, device: Device) {
    let mut devices = state.devices.lock().unwrap();
    devices.insert(device.id, device);
}

#[tauri::command]
fn remove_device(state: State<AppState>, device_id: u32) {
    let mut devices = state.devices.lock().unwrap();
    devices.remove(&device_id);
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
