//! Packet data export (txt / csv / json).
//!
//! Moved from the Tauri layer (map #85) so the format logic lives in the
//! library; the Tauri command is now a thin wrapper.

use std::fs;
use std::io::Write;
use std::path::Path;

/// A single captured packet ready for export.
#[derive(Debug, Clone)]
pub struct ExportPacket {
    /// Packet direction label (e.g. `"TX"` / `"RX"`).
    pub direction: String,
    /// Capture timestamp in milliseconds since the Unix epoch.
    pub timestamp_millis: u64,
    /// Raw packet bytes.
    pub data: Vec<u8>,
}

/// Export packets to `path` in the given format (`"txt"`, `"csv"` or `"json"`).
pub fn export_packets(
    path: &Path,
    format: &str,
    packets: &[ExportPacket],
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {e}"))?;
        }
    }

    match format {
        "txt" => export_txt(path, packets),
        "csv" => export_csv(path, packets),
        "json" => export_json(path, packets),
        _ => Err(format!("Unknown format: {format}. Supported: txt, csv, json")),
    }
}

fn export_txt(path: &Path, packets: &[ExportPacket]) -> Result<(), String> {
    let mut file = fs::File::create(path).map_err(|e| format!("Failed to create file: {e}"))?;

    writeln!(
        file,
        "Serial Data Export - {}",
        chrono::Utc::now().to_rfc3339()
    )
    .map_err(|e| e.to_string())?;

    for packet in packets {
        let hex = packet
            .data
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(
            file,
            "[{}] {} ({}): {}",
            packet.timestamp_millis,
            packet.direction,
            packet.data.len(),
            hex
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn export_csv(path: &Path, packets: &[ExportPacket]) -> Result<(), String> {
    let mut file = fs::File::create(path).map_err(|e| format!("Failed to create file: {e}"))?;

    writeln!(file, "timestamp,direction,data_hex").map_err(|e| e.to_string())?;

    for packet in packets {
        let hex = packet
            .data
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<String>();
        writeln!(
            file,
            "{},{},{}",
            packet.timestamp_millis, packet.direction, hex
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn export_json(path: &Path, packets: &[ExportPacket]) -> Result<(), String> {
    let entries: Vec<serde_json::Value> = packets
        .iter()
        .map(|p| {
            serde_json::json!({
                "direction": p.direction,
                "data": p.data,
                "timestamp_millis": p.timestamp_millis,
            })
        })
        .collect();
    let json =
        serde_json::to_string_pretty(&entries).map_err(|e| format!("Failed to serialize JSON: {e}"))?;
    fs::write(path, json).map_err(|e| format!("Failed to write file: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn test_packet(direction: &str, data: &[u8], ts: u64) -> ExportPacket {
        ExportPacket {
            direction: direction.to_string(),
            timestamp_millis: ts,
            data: data.to_vec(),
        }
    }

    #[test]
    fn test_export_txt() {
        let packets = vec![
            test_packet("TX", &[0x01, 0x02, 0x03], 1234567890),
            test_packet("RX", &[0x04, 0x05], 1234567900),
        ];
        let path = env::temp_dir().join("serial_cli_test_export.txt");

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
            test_packet("TX", &[0xAA, 0xBB], 1000),
            test_packet("RX", &[0xCC], 2000),
        ];
        let path = env::temp_dir().join("serial_cli_test_export.csv");

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
            test_packet("TX", &[0x01], 1000),
            test_packet("RX", &[0x02, 0x03], 2000),
        ];
        let path = env::temp_dir().join("serial_cli_test_export.json");

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
        let path = env::temp_dir().join("serial_cli_test_empty.txt");

        export_txt(&path, &[]).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Serial Data Export"));

        let _ = fs::remove_file(&path);
    }
}
