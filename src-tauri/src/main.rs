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
mod tray;

use state::app_state::AppState;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    // Create global app state
    let app_state = AppState::new().await;

    // Build Tauri application
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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
            // Protocol commands
            commands::protocol::list_protocols,
            commands::protocol::load_protocol,
            commands::protocol::unload_protocol,
            commands::protocol::reload_protocol,
            commands::protocol::validate_protocol,
            commands::protocol::get_protocol_info,
            // Script commands
            commands::script::execute_script,
            commands::script::validate_script,
            commands::script::list_scripts,
            commands::script::save_script,
            commands::script::delete_script,
            // Config commands
            commands::config::get_config,
            commands::config::update_config,
            commands::config::reset_config,
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
        ])
        .setup(|app| {
            // Setup event system
            events::emitter::setup_event_system(app.handle().clone())?;

            // Setup system tray
            tray::create_system_tray(app.handle())?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
