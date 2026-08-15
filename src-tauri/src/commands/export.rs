// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::path::PathBuf;
use tauri::State;

use crate::state::app_state::AppState;

/// Export packet data to a file on disk. Thin IPC wrapper over the library
/// exporter (`serial_cli::export`) — see map #85.
///
/// `format` must be one of: "txt", "csv", "json".
/// `data` is a JSON array of `{ direction, data, timestamp_millis }` objects
/// as received from the frontend's packet buffer.
#[tauri::command]
pub async fn export_data(
    path: String,
    format: String,
    data: serde_json::Value,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    let packets = data
        .as_array()
        .ok_or_else(|| "Data must be a JSON array".to_string())?;

    let export_packets: Vec<serial_cli::export::ExportPacket> = packets
        .iter()
        .map(|p| serial_cli::export::ExportPacket {
            direction: p["direction"].as_str().unwrap_or("?").to_string(),
            timestamp_millis: p["timestamp_millis"].as_u64().unwrap_or(0),
            data: p["data"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|v| v.as_u64().unwrap_or(0) as u8)
                        .collect::<Vec<u8>>()
                })
                .unwrap_or_default(),
        })
        .collect();

    serial_cli::export::export_packets(&PathBuf::from(&path), &format, &export_packets)
}
