//! Hex encoding and decoding utilities
//!
//! This module provides functions for converting between bytes and hexadecimal strings.

use crate::error::{Result, SerialError};

/// Encode bytes as a hexadecimal string
///
/// # Arguments
/// * `data` - Byte slice to encode
/// * `separator` - Optional separator between hex bytes (e.g., ":", " ", "")
///
/// # Returns
/// A hexadecimal string representation of the bytes
///
/// # Examples
/// ```
/// use serial_cli::utils::hex::hex_encode;
///
/// let bytes = vec![0x01, 0x02, 0xFF];
/// assert_eq!(hex_encode(&bytes, ""), "0102ff");
/// assert_eq!(hex_encode(&bytes, ":"), "01:02:ff");
/// assert_eq!(hex_encode(&bytes, " "), "01 02 ff");
/// ```
pub fn hex_encode(data: &[u8], separator: &str) -> String {
    data.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(separator)
}

/// Encode bytes as a hexadecimal string without separator
///
/// # Arguments
/// * `data` - Byte slice to encode
///
/// # Returns
/// A hexadecimal string representation of the bytes (lowercase, no separator)
pub fn hex_encode_simple(data: &[u8]) -> String {
    hex_encode(data, "")
}

/// Decode a hexadecimal string to bytes
///
/// # Arguments
/// * `hex` - Hexadecimal string to decode (supports optional separators and "0x" prefix)
///
/// # Returns
/// A vector of bytes, or an error if the input is invalid
///
/// # Errors
/// Returns an error if:
/// - The hex string has odd length
/// - The hex string contains non-hexadecimal characters
///
/// # Examples
/// ```
/// use serial_cli::utils::hex::hex_decode;
///
/// let bytes = hex_decode("0102ff").unwrap();
/// assert_eq!(bytes, vec![0x01, 0x02, 0xFF]);
///
/// let bytes = hex_decode("01:02:ff").unwrap();
/// assert_eq!(bytes, vec![0x01, 0x02, 0xFF]);
///
/// let bytes = hex_decode("0x0102ff").unwrap();
/// assert_eq!(bytes, vec![0x01, 0x02, 0xFF]);
/// ```
pub fn hex_decode(hex: &str) -> Result<Vec<u8>> {
    let hex = hex.trim();

    // Allow empty string → empty result
    if hex.is_empty() {
        return Ok(Vec::new());
    }

    // Remove optional "0x" / "0X" prefix
    let hex = hex
        .strip_prefix("0x")
        .or_else(|| hex.strip_prefix("0X"))
        .unwrap_or(hex);

    // Validate: every character must be a hex digit or a known separator
    for c in hex.chars() {
        if !c.is_ascii_hexdigit() && c != ':' && c != '-' && c != ' ' {
            return Err(SerialError::Config(format!(
                "Invalid character in hex string: '{}'",
                c
            )));
        }
    }

    // Strip known separators, leaving only hex digits
    let hex: String = hex.chars().filter(|c| c.is_ascii_hexdigit()).collect();

    if hex.is_empty() {
        return Ok(Vec::new());
    }

    if !hex.len().is_multiple_of(2) {
        return Err(SerialError::Config(format!(
            "Hex string must have even length, got {}",
            hex.len()
        )));
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte_str = &hex[i..i + 2];
        let byte = u8::from_str_radix(byte_str, 16)
            .map_err(|_| SerialError::Config(format!("Invalid hex byte: {}", byte_str)))?;
        bytes.push(byte);
    }

    Ok(bytes)
}

/// Decode a hexadecimal string to bytes (strict mode)
///
/// Unlike `hex_decode`, this function does not strip separators or prefixes.
/// It expects a clean hexadecimal string with even length.
///
/// # Arguments
/// * `hex` - Clean hexadecimal string (no prefixes, no separators)
///
/// # Returns
/// A vector of bytes, or an error if the input is invalid
pub fn hex_decode_strict(hex: &str) -> Result<Vec<u8>> {
    if hex.is_empty() {
        return Ok(Vec::new());
    }

    if !hex.len().is_multiple_of(2) {
        return Err(SerialError::Config(format!(
            "Hex string must have even length, got {}",
            hex.len()
        )));
    }

    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(SerialError::Config(
            "Hex string contains non-hexadecimal characters".to_string(),
        ));
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte_str = &hex[i..i + 2];
        let byte = u8::from_str_radix(byte_str, 16)
            .map_err(|_| SerialError::Config(format!("Invalid hex byte: {}", byte_str)))?;
        bytes.push(byte);
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_encode_empty() {
        let bytes: Vec<u8> = vec![];
        assert_eq!(hex_encode(&bytes, ""), "");
        assert_eq!(hex_encode(&bytes, ":"), "");
    }

    #[test]
    fn test_hex_encode_single_byte() {
        let bytes = vec![0x00];
        assert_eq!(hex_encode(&bytes, ""), "00");

        let bytes = vec![0xFF];
        assert_eq!(hex_encode(&bytes, ""), "ff");

        let bytes = vec![0xAB];
        assert_eq!(hex_encode(&bytes, ":"), "ab");
    }

    #[test]
    fn test_hex_encode_multiple_bytes() {
        let bytes = vec![0x01, 0x02, 0x03];
        assert_eq!(hex_encode(&bytes, ""), "010203");
        assert_eq!(hex_encode(&bytes, ":"), "01:02:03");
        assert_eq!(hex_encode(&bytes, " "), "01 02 03");
    }

    #[test]
    fn test_hex_encode_simple() {
        let bytes = vec![0x01, 0x02, 0xFF];
        assert_eq!(hex_encode_simple(&bytes), "0102ff");
    }

    #[test]
    fn test_hex_decode_empty() {
        let bytes = hex_decode("").unwrap();
        assert_eq!(bytes, Vec::<u8>::new());
    }

    #[test]
    fn test_hex_decode_simple() {
        let bytes = hex_decode("0102ff").unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0xFF]);
    }

    #[test]
    fn test_hex_decode_with_separators() {
        let bytes = hex_decode("01:02:ff").unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0xFF]);

        let bytes = hex_decode("01 02 ff").unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0xFF]);

        let bytes = hex_decode("01-02-ff").unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0xFF]);
    }

    #[test]
    fn test_hex_decode_with_prefix() {
        let bytes = hex_decode("0x0102ff").unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0xFF]);

        let bytes = hex_decode("0X0102ff").unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0xFF]);
    }

    #[test]
    fn test_hex_decode_uppercase() {
        let bytes = hex_decode("0102FF").unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0xFF]);

        let bytes = hex_decode("ABCDEF").unwrap();
        assert_eq!(bytes, vec![0xAB, 0xCD, 0xEF]);
    }

    #[test]
    fn test_hex_decode_mixed_case() {
        let bytes = hex_decode("aAbBcC").unwrap();
        assert_eq!(bytes, vec![0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_hex_decode_odd_length() {
        let result = hex_decode("012");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("even length"));
    }

    #[test]
    fn test_hex_decode_invalid_chars() {
        let result = hex_decode("01GG");
        assert!(result.is_err());
    }

    #[test]
    fn test_hex_decode_strict_empty() {
        let bytes = hex_decode_strict("").unwrap();
        assert_eq!(bytes, Vec::<u8>::new());
    }

    #[test]
    fn test_hex_decode_strict_simple() {
        let bytes = hex_decode_strict("0102ff").unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0xFF]);
    }

    #[test]
    fn test_hex_decode_strict_with_separators() {
        let result = hex_decode_strict("01:02:ff");
        assert!(result.is_err());
    }

    #[test]
    fn test_hex_decode_strict_with_prefix() {
        let result = hex_decode_strict("0x0102ff");
        assert!(result.is_err());
    }

    #[test]
    fn test_hex_decode_strict_odd_length() {
        let result = hex_decode_strict("012");
        assert!(result.is_err());
    }

    #[test]
    fn test_hex_decode_strict_invalid_chars() {
        let result = hex_decode_strict("01GG");
        assert!(result.is_err());
    }

    #[test]
    fn test_hex_roundtrip() {
        let original = vec![0x00, 0x7F, 0x80, 0xFF, 0xAB, 0xCD];
        let encoded = hex_encode_simple(&original);
        let decoded = hex_decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_hex_roundtrip_with_separators() {
        let original = vec![0x01, 0x02, 0x03, 0x04];
        let encoded = hex_encode(&original, ":");
        let decoded = hex_decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }
}
