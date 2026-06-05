// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fs;
use std::path::PathBuf;
use tauri::State;

use crate::state::app_state::AppState;

/// Export packet data to a file on disk.
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
    let path = PathBuf::from(&path);

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }
    }

    let packets = data
        .as_array()
        .ok_or_else(|| "Data must be a JSON array".to_string())?;

    match format.as_str() {
        "txt" => export_txt(&path, packets),
        "csv" => export_csv(&path, packets),
        "json" => export_json(&path, packets),
        _ => Err(format!(
            "Unknown format: {}. Supported: txt, csv, json",
            format
        )),
    }
}

fn export_txt(path: &PathBuf, packets: &[serde_json::Value]) -> Result<(), String> {
    use std::io::Write;
    let mut file = fs::File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;

    writeln!(
        file,
        "Serial Data Export - {}",
        chrono::Utc::now().to_rfc3339()
    )
    .map_err(|e| e.to_string())?;

    for packet in packets {
        let dir = packet["direction"].as_str().unwrap_or("?");
        let ts = packet["timestamp_millis"].as_u64().unwrap_or(0);
        let data = packet["data"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|v| format!("{:02X}", v.as_u64().unwrap_or(0)))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();

        writeln!(file, "[{}] {} ({}): {}", ts, dir, data.len() / 3 + 1, data)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn export_csv(path: &PathBuf, packets: &[serde_json::Value]) -> Result<(), String> {
    use std::io::Write;
    let mut file = fs::File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;

    writeln!(file, "timestamp,direction,data_hex").map_err(|e| e.to_string())?;

    for packet in packets {
        let dir = packet["direction"].as_str().unwrap_or("?");
        let ts = packet["timestamp_millis"].as_u64().unwrap_or(0);
        let hex = packet["data"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|v| format!("{:02X}", v.as_u64().unwrap_or(0)))
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default();

        writeln!(file, "{},{},{}", ts, dir, hex).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn export_json(path: &PathBuf, packets: &[serde_json::Value]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(packets)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
    fs::write(path, json).map_err(|e| format!("Failed to write file: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn make_test_packet(direction: &str, data: &[u8], ts: u64) -> serde_json::Value {
        serde_json::json!({
            "direction": direction,
            "data": data,
            "timestamp_millis": ts,
        })
    }

    #[test]
    fn test_export_txt() {
        let packets = vec![
            make_test_packet("TX", &[0x01, 0x02, 0x03], 1234567890),
            make_test_packet("RX", &[0x04, 0x05], 1234567900),
        ];
        let temp_dir = env::temp_dir();
        let path = temp_dir.join("serial_cli_test_export.txt");

        export_txt(&path, &packets).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Serial Data Export"));
        assert!(content.contains("TX"));
        assert!(content.contains("RX"));
        assert!(content.contains("01 02 03"));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_export_csv() {
        let packets = vec![
            make_test_packet("TX", &[0xAA, 0xBB], 1000),
            make_test_packet("RX", &[0xCC], 2000),
        ];
        let temp_dir = env::temp_dir();
        let path = temp_dir.join("serial_cli_test_export.csv");

        export_csv(&path, &packets).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines[0], "timestamp,direction,data_hex");
        assert!(content.contains("1000,TX,AABB"));
        assert!(content.contains("2000,RX,CC"));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_export_json() {
        let packets = vec![
            make_test_packet("TX", &[0x01], 1000),
            make_test_packet("RX", &[0x02, 0x03], 2000),
        ];
        let temp_dir = env::temp_dir();
        let path = temp_dir.join("serial_cli_test_export.json");

        export_json(&path, &packets).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["direction"], "TX");
        assert_eq!(parsed[1]["data"].as_array().unwrap().len(), 2);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_export_txt_empty_packets() {
        let temp_dir = env::temp_dir();
        let path = temp_dir.join("serial_cli_test_empty.txt");

        export_txt(&path, &[]).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Serial Data Export"));

        let _ = fs::remove_file(&path);
    }
}
