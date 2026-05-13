//! Test data generation utilities

/// Generate repeating pattern data
#[allow(dead_code)]
pub fn generate_pattern_data(size: usize, pattern: u8) -> Vec<u8> {
    vec![pattern; size]
}

/// Generate Modbus RTU read request
///
/// Args:
///   slave_id: Modbus slave address (1-247)
///   function_code: Modbus function (e.g., 0x03 for read holding registers)
///   start_addr: Starting register address
///   quantity: Number of registers to read
#[allow(dead_code)]
pub fn generate_modbus_request(
    slave_id: u8,
    function_code: u8,
    start_addr: u16,
    quantity: u16,
) -> Vec<u8> {
    let mut frame = vec![slave_id, function_code];
    frame.extend_from_slice(&start_addr.to_be_bytes());
    frame.extend_from_slice(&quantity.to_be_bytes());

    // Calculate CRC
    let crc = calculate_modbus_crc(&frame);
    frame.extend_from_slice(&crc.to_le_bytes());

    frame
}

/// Calculate Modbus CRC16
#[allow(dead_code)]
fn calculate_modbus_crc(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if crc & 0x0001 != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

/// Generate AT command string
#[allow(dead_code)]
pub fn generate_at_command(command: &str) -> String {
    format!("{}\r\n", command)
}

/// Generate multi-line AT response
#[allow(dead_code)]
pub fn generate_at_response(lines: &[&str]) -> Vec<u8> {
    let response = lines.join("\r\n");
    format!("{}\r\nOK\r\n", response).into_bytes()
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_generate_pattern_data() {
        let data = generate_pattern_data(50, 0xAB);
        assert_eq!(data.len(), 50);
        assert!(data.iter().all(|&b| b == 0xAB));
    }

    #[test]
    fn test_generate_modbus_request() {
        let frame = generate_modbus_request(1, 0x03, 0x0000, 10);
        assert_eq!(frame.len(), 8); // 6 bytes + 2 bytes CRC
        assert_eq!(frame[0], 1); // slave_id
        assert_eq!(frame[1], 0x03); // function_code
    }

    #[test]
    fn test_generate_at_command() {
        let cmd = generate_at_command("AT");
        assert!(cmd.contains("AT"));
        assert!(cmd.ends_with("\r\n"));
    }
}
