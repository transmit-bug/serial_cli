// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod events;
mod state;

use state::app_state::AppState;
use tauri::Manager;

#[tokio::main]
async fn main() {
    // Initialize logging — stderr + optional file output
    {
        let log_dir = dirs::data_local_dir()
            .map(|mut p| {
                p.push("serial-cli");
                p.push("logs");
                p
            })
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let _ = std::fs::create_dir_all(&log_dir);

        // Try to open log file; fall back to stderr-only
        let log_path = log_dir.join("serial-cli.log");
        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            use std::sync::Arc;
            use tracing_subscriber::layer::SubscriberExt;
            use tracing_subscriber::util::SubscriberInitExt;
            use tracing_subscriber::Layer;

            let file = Arc::new(std::sync::Mutex::new(file));
            let file_layer = tracing_subscriber::fmt::layer()
                .with_writer(move || -> Box<dyn std::io::Write + Send + Sync> {
                    match Arc::clone(&file).lock().unwrap().try_clone() {
                        Ok(f) => Box::new(f),
                        Err(_) => Box::new(std::io::empty()),
                    }
                })
                .with_ansi(false)
                .with_filter(tracing_subscriber::filter::EnvFilter::new("info"));

            let stderr_layer = tracing_subscriber::fmt::layer()
                .with_filter(tracing_subscriber::filter::EnvFilter::new("warn"));

            tracing_subscriber::registry()
                .with(file_layer)
                .with(stderr_layer)
                .init();
        } else {
            tracing_subscriber::fmt().with_env_filter("info").init();
        }
    }

    // Create global app state
    let app_state = AppState::new().await;

    // Built-in scripts are registered automatically by ScriptManager::new()

    // Build Tauri application
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Port commands
            commands::port::list_ports,
            commands::port::open_port,
            commands::port::close_port,
            commands::port::get_port_status,
            commands::port::get_all_ports_status,
            commands::port::check_port_health,
            // Serial commands
            commands::serial::send_data,
            commands::serial::read_data,
            commands::serial::start_sniffing,
            commands::serial::stop_sniffing,
            // Serial script engine commands
            commands::serial_script::attach_script,
            commands::serial_script::detach_script,
            commands::serial_script::has_script,
            commands::serial_script::get_script_status,
            commands::serial_script::list_script_actions,
            commands::serial_script::call_script_function,
            // Protocol commands
            commands::protocol::list_protocols,
            commands::protocol::load_protocol,
            commands::protocol::unload_protocol,
            commands::protocol::reload_protocol,
            commands::protocol::validate_protocol,
            commands::protocol::protocol_encode,
            commands::protocol::protocol_decode,
            commands::protocol::set_port_protocol,
            commands::protocol::save_protocol_file,
            commands::protocol::get_protocol_info,
            // Script commands
            commands::script::execute_script,
            commands::script::validate_script,
            commands::script::list_scripts,
            commands::script::save_script,
            commands::script::delete_script,
            // Script UI actions commands
            commands::script_ui_actions::list_standalone_script_actions,
            commands::script_ui_actions::call_standalone_script_function,
            // Config commands
            commands::config::get_config,
            commands::config::update_config,
            commands::config::reset_config,
            commands::config::get_connection_presets,
            commands::config::save_connection_presets,
            commands::config::delete_connection_preset,
            commands::config::read_logs,
            commands::config::clear_logs,
            // Data export command
            commands::export::export_data,
            // Window commands
            commands::window::show_window,
            commands::window::hide_window,
            commands::window::toggle_window,
            // Virtual port commands
            commands::virtual_port::create_virtual_port,
            commands::virtual_port::list_virtual_ports,
            commands::virtual_port::stop_virtual_port,
            commands::virtual_port::get_virtual_port_stats,
            commands::virtual_port::check_virtual_port_health,
            commands::virtual_port::get_captured_packets,
            commands::virtual_port::send_to_virtual_port,
        ])
        .setup(|app| {
            // Setup event system
            events::emitter::setup_event_system(app.handle().clone())?;

            // Start port hot-plug monitor
            spawn_port_monitor(app.handle().clone());

            Ok(())
        })
        .on_window_event(|window, event| {
            use tauri::WindowEvent;

            if let WindowEvent::CloseRequested { api, .. } = event {
                // Prevent default close — we need to clean up first
                api.prevent_close();

                let app_handle = window.app_handle().clone();
                let state = app_handle.clone().state::<AppState>().inner().clone();

                // Spawn cleanup then exit
                tokio::spawn(async move {
                    shutdown(state, &app_handle).await;
                    app_handle.exit(0);
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Spawn a background task that polls hardware ports every 2s and emits
/// `ports-changed` events when ports are added or removed.
fn spawn_port_monitor(app: tauri::AppHandle) {
    use std::collections::HashSet;

    tokio::spawn(async move {
        let mut known: HashSet<String> = HashSet::new();

        // Seed with initial port list
        let manager = serial_cli::serial_core::PortManager::new();
        if let Ok(ports) = manager.list_ports() {
            for p in &ports {
                known.insert(p.port_name.clone());
            }
        }

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            let manager = serial_cli::serial_core::PortManager::new();
            let current = match manager.list_ports() {
                Ok(ports) => ports
                    .into_iter()
                    .map(|p| p.port_name)
                    .collect::<HashSet<_>>(),
                Err(_) => continue,
            };

            let added: Vec<String> = current.difference(&known).cloned().collect();
            let removed: Vec<String> = known.difference(&current).cloned().collect();

            if !added.is_empty() || !removed.is_empty() {
                tracing::debug!("Ports changed: +{} -{}", added.len(), removed.len());
                if let Err(e) =
                    events::emitter::emit_ports_changed(app.clone(), added, removed).await
                {
                    tracing::warn!("Failed to emit ports-changed event: {}", e);
                }
                known = current;
            }
        }
    });
}

/// Graceful shutdown: stop sniffers → close ports → stop virtual ports.
async fn shutdown(state: AppState, _app: &tauri::AppHandle) {
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    tracing::info!("Shutting down...");

    // 1. Stop all sniffers
    {
        let mut sniffers = state.active_sniffers.lock().await;
        let port_ids: Vec<String> = sniffers.keys().cloned().collect();
        for port_id in port_ids {
            if let Some(sniffer) = sniffers.remove(&port_id) {
                sniffer.stop_flag.store(true, Ordering::Relaxed);
                // Best-effort wait
                let _ = tokio::time::timeout(Duration::from_secs(1), sniffer.task_handle).await;
                let _ =
                    tokio::time::timeout(Duration::from_secs(1), sniffer.read_task_handle).await;
                tracing::info!("Stopped sniffer for port: {}", port_id);
            }
        }
    }

    // 2. Close all open serial ports
    {
        let manager = state.port_manager.lock().await;
        let open_ports = manager.list_open_ports().await;
        drop(manager);

        for (port_id, port_name) in open_ports {
            let manager = state.port_manager.lock().await;
            if let Ok(handle) = manager.get_port(&port_id).await {
                // Detach script if present
                let mut h = handle.lock().await;
                if h.has_script() {
                    h.detach_script();
                }
                drop(h);
            }
            if manager.close_port(&port_id).await.is_ok() {
                tracing::info!("Closed port: {} ({})", port_id, port_name);
            }
        }
    }

    // 3. Stop all virtual port pairs
    {
        let mut registry = state.virtual_port_registry.write().await;
        let ids: Vec<String> = registry.keys().cloned().collect();
        for id in ids {
            if let Some(pair) = registry.remove(&id) {
                if let Err(e) = pair.stop().await {
                    tracing::warn!("Error stopping virtual port {}: {}", id, e);
                } else {
                    tracing::info!("Stopped virtual port pair: {}", id);
                }
            }
        }
    }

    tracing::info!("Shutdown complete");
}
