//! Task scheduler module
//!
//! This module provides task scheduling and execution.
//!
//! **Experimental** — not yet integrated into any CLI command.
//! Planned for v0.6.0 integration with a unified session management abstraction.
//!
//! # Key types
//!
//! - [`Task`] — a unit of work with a unique ID and creation timestamp
//! - [`TaskType`] — the kind of work (script execution, serial operation, custom)
//! - [`TaskStatus`] — lifecycle state of a task
//! - [`TaskResult`] — outcome of task execution
//! - [`TaskQueue`] — priority-based queue with concurrency limiting

#![allow(dead_code)]

pub mod executor;
pub mod monitor;
pub mod queue;

use uuid::Uuid;

/// Unique identifier for a task (UUID v4 string).
pub type TaskId = String;

/// The kind of work a [`Task`] represents.
#[derive(Debug, Clone)]
pub enum TaskType {
    /// Execute a Lua script by name with the given content.
    Script { name: String, content: String },
    /// Perform a serial port operation on the named port.
    SerialOp {
        port_name: String,
        operation: SerialOperation,
    },
    /// A custom task with an arbitrary name and data payload.
    Custom { name: String, data: String },
}

/// The specific serial port operation within a [`TaskType::SerialOp`].
///
/// **Experimental** — not yet integrated into any CLI command.
/// Marked `#[allow(dead_code)]` as this is planned for v0.6.0.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SerialOperation {
    /// Send raw bytes to the port.
    Send { data: Vec<u8> },
    /// Receive up to `bytes` from the port.
    Recv { bytes: usize },
    /// Open the port with the given [`SerialConfig`].
    Open { config: SerialConfig },
    /// Close the port.
    Close,
}

/// Serial configuration — experimental, planned for v0.6.0 integration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SerialConfig {
    /// Baud rate in bits per second.
    pub baudrate: u32,
    /// Data bits per frame (5–8).
    pub databits: u8,
    /// Stop bits (1 or 2).
    pub stopbits: u8,
    /// Parity mode as a string (`"none"`, `"odd"`, `"even"`).
    pub parity: String,
}

/// A unit of scheduled work with a unique ID and creation timestamp.
#[derive(Debug, Clone)]
pub struct Task {
    id: TaskId,
    task_type: TaskType,
    created_at: std::time::Instant,
}

impl Task {
    /// Create a new task with a generated UUID and current timestamp.
    pub fn new(task_type: TaskType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            task_type,
            created_at: std::time::Instant::now(),
        }
    }

    /// Get the task's unique identifier.
    pub fn id(&self) -> TaskId {
        self.id.clone()
    }

    /// Get the kind of work this task represents.
    pub fn task_type(&self) -> &TaskType {
        &self.task_type
    }

    /// Get the time elapsed since the task was created.
    pub fn elapsed(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }
}

/// Lifecycle state of a task in the scheduler.
///
/// **Experimental** — defined but not yet wired into the execution pipeline.
/// Planned for v0.6.0 integration with a `SessionManager` abstraction.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is queued and waiting to run.
    Pending,
    /// Task is currently executing.
    Running,
    /// Task completed successfully.
    Completed,
    /// Task failed with an error.
    Failed,
}

/// Outcome of a completed task.
#[derive(Debug, Clone)]
pub enum TaskResult {
    /// Task completed with no output data.
    Success,
    /// Task completed with a binary payload.
    SuccessWithData(Vec<u8>),
    /// Task completed with a text response.
    SuccessWithText(String),
    /// Task failed with an error message.
    Error(String),
}

pub use queue::{TaskPriority, TaskQueue};
