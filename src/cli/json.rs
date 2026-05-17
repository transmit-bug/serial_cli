//! Enhanced JSON output formatter
//!
//! This module provides comprehensive JSON output support for AI interaction.

use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// JSON output formatter with full metadata
pub struct JsonFormatter {
    pretty: bool,
}

/// Standard JSON response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonResponse<T> {
    pub status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ResponseMetadata>,
}

/// Response status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Success,
    Error,
    Partial,
}

/// Error detail structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<String>,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statistics: Option<OperationStatistics>,
}

/// Operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStatistics {
    pub bytes_sent: usize,
    pub bytes_recv: usize,
    pub packets_sent: usize,
    pub packets_recv: usize,
    pub errors: usize,
}

impl JsonFormatter {
    /// Create a new JSON formatter
    pub fn new(pretty: bool) -> Self {
        Self { pretty }
    }

    /// Format success response
    pub fn format_success<T: Serialize>(
        &self,
        data: T,
        metadata: Option<ResponseMetadata>,
    ) -> Result<String> {
        let response = JsonResponse {
            status: ResponseStatus::Success,
            data: Some(data),
            error: None,
            metadata,
        };

        self.serialize(response)
    }

    /// Format error response
    pub fn format_error(
        &self,
        code: &str,
        message: &str,
        details: Vec<String>,
        suggestions: Vec<String>,
        metadata: Option<ResponseMetadata>,
    ) -> Result<String> {
        let error = ErrorDetail {
            code: code.to_string(),
            message: message.to_string(),
            details,
            suggestions,
        };

        let response: JsonResponse<()> = JsonResponse {
            status: ResponseStatus::Error,
            data: None,
            error: Some(error),
            metadata,
        };

        self.serialize(response)
    }

    /// Format partial response
    pub fn format_partial<T: Serialize>(
        &self,
        data: T,
        metadata: Option<ResponseMetadata>,
    ) -> Result<String> {
        let response = JsonResponse {
            status: ResponseStatus::Partial,
            data: Some(data),
            error: None,
            metadata,
        };

        self.serialize(response)
    }

    /// Serialize response to JSON string
    fn serialize<T: Serialize>(&self, value: T) -> Result<String> {
        if self.pretty {
            serde_json::to_string_pretty(&value)
        } else {
            serde_json::to_string(&value)
        }
        .map_err(|e| crate::error::SerialError::Parse(e.to_string()))
    }

    /// Create metadata from operation timing
    pub fn create_metadata(&self, start: Instant, port: Option<String>) -> ResponseMetadata {
        let duration = start.elapsed();

        ResponseMetadata {
            timestamp: Utc::now(),
            duration_ms: duration.as_millis() as u64,
            port,
            statistics: None,
        }
    }

    /// Create metadata with statistics
    pub fn create_metadata_with_stats(
        &self,
        start: Instant,
        port: Option<String>,
        stats: OperationStatistics,
    ) -> ResponseMetadata {
        let duration = start.elapsed();

        ResponseMetadata {
            timestamp: Utc::now(),
            duration_ms: duration.as_millis() as u64,
            port,
            statistics: Some(stats),
        }
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new(true)
    }
}

/// Helper to create success response
pub fn success_response<T: Serialize>(data: T) -> JsonResponse<T> {
    JsonResponse {
        status: ResponseStatus::Success,
        data: Some(data),
        error: None,
        metadata: None,
    }
}

/// Helper to create error response
pub fn error_response(code: &str, message: &str) -> JsonResponse<()> {
    JsonResponse {
        status: ResponseStatus::Error,
        data: None,
        error: Some(ErrorDetail {
            code: code.to_string(),
            message: message.to_string(),
            details: vec![],
            suggestions: vec![],
        }),
        metadata: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_formatter_success() {
        let formatter = JsonFormatter::new(true);
        let data = vec![1u8, 2u8, 3u8];

        let result = formatter.format_success(data.clone(), None);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        // Just check that it's valid JSON
        let _value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(json_str.contains("success"));
    }

    #[test]
    fn test_json_formatter_error() {
        let formatter = JsonFormatter::new(true);

        let result = formatter.format_error(
            "E001",
            "Test error",
            vec!["Detail 1".to_string()],
            vec!["Suggestion 1".to_string()],
            None,
        );

        assert!(result.is_ok());

        let json_str = result.unwrap();
        assert!(json_str.contains("error"));
        assert!(json_str.contains("E001"));
        assert!(json_str.contains("Test error"));
    }

    #[test]
    fn test_json_serialization() {
        let response: JsonResponse<Vec<i32>> = success_response(vec![1, 2, 3]);
        let json_str = serde_json::to_string_pretty(&response).unwrap();

        assert!(json_str.contains("\"status\": \"success\""));
        // Just verify it's valid JSON
        let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    }
}
