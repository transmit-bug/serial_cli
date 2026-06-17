//! Batch runner
//!
//! This module provides batch processing mode for running multiple scripts.

use crate::error::{Result, SerialError};
use crate::lua::executor::ScriptEngine;
use crate::task::executor::TaskExecutor;
use crate::task::{Task, TaskPriority, TaskType};
use crate::utils::ProgressReporter;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::time::Duration;

/// Batch script line type
#[derive(Debug, Clone)]
pub enum BatchLine {
    /// Comment line
    Comment(String),
    /// Script to execute
    Script(std::path::PathBuf),
    /// Set a variable: `set NAME value`
    Set { key: String, value: String },
    /// Loop execution: `loop N` ... `end`
    Loop { count: usize, body: Vec<BatchLine> },
    /// Sleep/delay: `sleep MS`
    Sleep(Duration),
}

/// Batch runner configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum concurrent tasks
    pub max_concurrent: usize,
    /// Task timeout in seconds
    pub timeout_secs: u64,
    /// Continue on error
    pub continue_on_error: bool,
    /// Enable progress reporting
    pub show_progress: bool,
    /// Verbose output
    pub verbose: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 5,
            timeout_secs: 60,
            continue_on_error: false,
            show_progress: true,
            verbose: false,
        }
    }
}

/// Batch runner
pub struct BatchRunner {
    config: BatchConfig,
    executor: Arc<TaskExecutor>,
    /// Variables set via `set` directives in batch files
    variables: HashMap<String, String>,
}

impl BatchRunner {
    /// Create a new batch runner
    pub fn new(config: BatchConfig) -> Result<Self> {
        let executor = Arc::new(TaskExecutor::new(config.max_concurrent));

        Ok(Self {
            config,
            executor,
            variables: HashMap::new(),
        })
    }

    /// Resolve variables in a string.
    /// Supports `${VAR}` and `$VAR` syntax, falls back to environment variables.
    fn resolve_variables(&self, input: &str) -> String {
        let mut result = input.to_string();
        let mut iteration = 0;

        // Resolve ${VAR} syntax (with iteration guard against self-referential loops)
        while let Some(start) = result.find("${") {
            if let Some(end) = result[start..].find('}') {
                let var_name = &result[start + 2..start + end];
                let value: String = self
                    .variables
                    .get(var_name)
                    .cloned()
                    .or_else(|| std::env::var(var_name).ok())
                    .unwrap_or_default();
                result = format!(
                    "{}{}{}",
                    &result[..start],
                    value,
                    &result[start + end + 1..]
                );
                iteration += 1;
                if iteration > 100 {
                    break; // Guard against self-referential loops like ${A} = ${A}
                }
            } else {
                break;
            }
        }

        // Resolve $VAR syntax (alphanumeric + underscore, stops at non-word char)
        let mut output = String::new();
        let mut chars = result.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '$' && chars.peek().is_some_and(|c| c.is_alphabetic() || *c == '_') {
                let mut var_name = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc.is_alphanumeric() || nc == '_' {
                        var_name.push(nc);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let value: String = self
                    .variables
                    .get(&var_name)
                    .cloned()
                    .or_else(|| std::env::var(&var_name).ok())
                    .unwrap_or_default();
                output.push_str(&value);
            } else {
                output.push(c);
            }
        }

        output
    }

    /// Set a variable
    pub fn set_variable(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    /// Get a variable
    pub fn get_variable(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Run a single script
    pub async fn run_script(&self, script_path: &Path) -> Result<()> {
        let engine = ScriptEngine::new()?;
        engine.execute_file(script_path)?;
        Ok(())
    }

    /// Run multiple scripts in sequence
    pub async fn run_scripts(&self, script_paths: Vec<&Path>) -> Result<BatchResult> {
        let executor = self.executor.clone();
        executor.start().await?;

        let mut results = Vec::new();

        for script_path in script_paths {
            let script_content = std::fs::read_to_string(script_path).map_err(SerialError::Io)?;

            let task = Task::new(TaskType::Script {
                name: script_path.display().to_string(),
                content: script_content,
            });

            let task_id = task.id().clone();
            self.executor.submit(task, TaskPriority::Normal).await?;

            let start = std::time::Instant::now();
            let mut task_completed = false;

            while !task_completed && start.elapsed() < Duration::from_secs(self.config.timeout_secs)
            {
                tokio::time::sleep(Duration::from_millis(100)).await;

                let completed = self.executor.get_completed().await;
                if let Some(completion) = completed.iter().find(|c| c.task_id == task_id) {
                    results.push(ScriptResult {
                        script: script_path.display().to_string(),
                        success: matches!(completion.result, crate::task::TaskResult::Success),
                        duration: completion.duration,
                        error: None,
                    });

                    if !self.config.continue_on_error
                        && matches!(completion.result, crate::task::TaskResult::Error(_))
                    {
                        executor.stop().await?;
                        return Err(SerialError::Script(crate::error::ScriptError::ApiError(
                            "Script execution failed".to_string(),
                        )));
                    }
                    task_completed = true;
                    break;
                }
            }

            if !task_completed {
                executor.stop().await?;
                return Err(SerialError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Script execution timeout",
                )));
            }
        }

        executor.stop().await?;

        Ok(BatchResult { results })
    }

    /// Run scripts in parallel
    pub async fn run_scripts_parallel(&self, script_paths: Vec<&Path>) -> Result<BatchResult> {
        let executor = self.executor.clone();
        executor.start().await?;

        for script_path in script_paths {
            let script_content = std::fs::read_to_string(script_path).map_err(SerialError::Io)?;

            let task = Task::new(TaskType::Script {
                name: script_path.display().to_string(),
                content: script_content,
            });

            self.executor.submit(task, TaskPriority::Normal).await?;
        }

        let start = std::time::Instant::now();
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;

            let completed = self.executor.get_completed().await;
            let pending = self.executor.pending_count().await;
            let running = self.executor.running_count().await;

            if pending == 0 && running == 0 && completed.len() >= self.config.max_concurrent {
                break;
            }

            if start.elapsed() > Duration::from_secs(self.config.timeout_secs * 2) {
                return Err(SerialError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Batch execution timeout",
                )));
            }
        }

        let completed = self.executor.get_completed().await;
        let results: Vec<ScriptResult> = completed
            .into_iter()
            .map(|c| ScriptResult {
                script: c.task_id.clone(),
                success: matches!(c.result, crate::task::TaskResult::Success),
                duration: c.duration,
                error: None,
            })
            .collect();

        executor.stop().await?;

        Ok(BatchResult { results })
    }

    /// Parse a batch file into executable lines
    pub fn parse_batch_file(&self, path: &Path) -> Result<Vec<BatchLine>> {
        let content = std::fs::read_to_string(path).map_err(SerialError::Io)?;
        parse_batch_lines(&content)
    }

    /// Run parsed batch lines with enhanced features
    pub async fn run_batch_lines(&mut self, batch_lines: Vec<BatchLine>) -> Result<BatchResult> {
        self.run_batch_lines_impl(batch_lines).await
    }

    /// Internal implementation — uses explicit boxing for async recursion
    async fn run_batch_lines_impl(&mut self, batch_lines: Vec<BatchLine>) -> Result<BatchResult> {
        let mut results = Vec::new();
        let total_lines = batch_lines.len();
        let mut progress = if self.config.show_progress {
            Some(ProgressReporter::new(
                "Batch execution".to_string(),
                total_lines,
            ))
        } else {
            None
        };

        let mut i = 0;
        while i < batch_lines.len() {
            match &batch_lines[i] {
                BatchLine::Comment(msg) => {
                    if self.config.verbose {
                        println!("  # {}", msg);
                    }
                }
                BatchLine::Set { key, value } => {
                    let resolved = self.resolve_variables(value);
                    if self.config.verbose {
                        println!("  set {} = {}", key, resolved);
                    }
                    self.variables.insert(key.clone(), resolved);
                }
                BatchLine::Script(script_path) => {
                    let path_str = script_path.to_string_lossy();
                    let resolved = self.resolve_variables(&path_str);
                    let resolved_path = Path::new(&resolved);

                    if self.config.verbose {
                        println!("Running: {}", resolved_path.display());
                    }

                    match self.run_script(resolved_path).await {
                        Ok(_) => {
                            results.push(ScriptResult {
                                script: resolved_path.display().to_string(),
                                success: true,
                                duration: Duration::ZERO,
                                error: None,
                            });
                        }
                        Err(e) => {
                            let err_msg = format!("{}", e);
                            results.push(ScriptResult {
                                script: resolved_path.display().to_string(),
                                success: false,
                                duration: Duration::ZERO,
                                error: Some(err_msg.clone()),
                            });

                            if !self.config.continue_on_error {
                                eprintln!("Error executing {}: {}", resolved_path.display(), e);
                                break;
                            }
                        }
                    }

                    if let Some(ref mut p) = progress {
                        p.update(1);
                    }
                }
                BatchLine::Sleep(duration) => {
                    if self.config.verbose {
                        println!("Sleeping for {:?}", duration);
                    }
                    tokio::time::sleep(*duration).await;
                }
                BatchLine::Loop { count, body } => {
                    let loop_count = *count;
                    if self.config.verbose {
                        println!("Starting loop ({} iterations)", loop_count);
                    }

                    for iteration in 0..loop_count {
                        if self.config.verbose {
                            println!("  Loop iteration {}/{}", iteration + 1, loop_count);
                        }

                        let body_clone = body.clone();
                        // Async recursion requires Box::pin because the recursive Future
                        // has a different (larger) size than the parent Future.
                        let future = Box::pin(self.run_batch_lines_impl(body_clone));
                        let body_results = future.await?;

                        for r in body_results.results {
                            if !r.success && !self.config.continue_on_error {
                                eprintln!(
                                    "Loop iteration {} failed: {}",
                                    iteration + 1,
                                    r.error.as_deref().unwrap_or("unknown error")
                                );
                                return Ok(BatchResult { results });
                            }
                        }
                    }
                }
            }

            i += 1;
        }

        if let Some(ref mut p) = progress {
            p.update(total_lines);
        }

        Ok(BatchResult { results })
    }
}

/// Parse batch file content into a list of BatchLine entries.
/// Supports: comments (#), set directives, loop/end blocks, sleep, and script paths.
fn parse_batch_lines(content: &str) -> Result<Vec<BatchLine>> {
    let mut lines = Vec::new();
    let mut in_loop = false;
    let mut loop_count: usize = 0;
    let mut loop_body = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('#') {
            let entry = BatchLine::Comment(trimmed.to_string());
            if in_loop {
                loop_body.push(entry);
            } else {
                lines.push(entry);
            }
            continue;
        }

        // End of loop block
        if trimmed.eq_ignore_ascii_case("end") {
            if in_loop {
                lines.push(BatchLine::Loop {
                    count: loop_count,
                    body: std::mem::take(&mut loop_body),
                });
                in_loop = false;
                loop_count = 0;
            } else {
                return Err(SerialError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Unexpected 'end' without matching 'loop' at line {}",
                        line_num + 1
                    ),
                )));
            }
            continue;
        }

        // Loop directive
        if trimmed.to_lowercase().starts_with("loop ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(count) = parts[1].parse::<usize>() {
                    if in_loop {
                        return Err(SerialError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Nested loops not supported at line {}", line_num + 1),
                        )));
                    }
                    in_loop = true;
                    loop_count = count;
                    loop_body = Vec::new();
                }
            }
            continue;
        }

        // Set directive: `set NAME value`
        if trimmed.to_lowercase().starts_with("set ") {
            let rest = &trimmed[4..];
            if let Some(space_pos) = rest.find(|c: char| c.is_whitespace()) {
                let key = &rest[..space_pos];
                let value = rest[space_pos..].trim();
                let entry = BatchLine::Set {
                    key: key.to_string(),
                    value: value.to_string(),
                };
                if in_loop {
                    loop_body.push(entry);
                } else {
                    lines.push(entry);
                }
            } else {
                // `set NAME` with no value — set to empty string
                let entry = BatchLine::Set {
                    key: rest.to_string(),
                    value: String::new(),
                };
                if in_loop {
                    loop_body.push(entry);
                } else {
                    lines.push(entry);
                }
            }
            continue;
        }

        // Sleep directive: `sleep MS`
        if trimmed.to_lowercase().starts_with("sleep ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let ms: u64 = parts[1].parse().unwrap_or(1000);
                let entry = BatchLine::Sleep(Duration::from_millis(ms));
                if in_loop {
                    loop_body.push(entry);
                } else {
                    lines.push(entry);
                }
            }
            continue;
        }

        // Script path
        let entry = BatchLine::Script(Path::new(trimmed).to_path_buf());
        if in_loop {
            loop_body.push(entry);
        } else {
            lines.push(entry);
        }
    }

    if in_loop {
        return Err(SerialError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Unclosed 'loop' block — missing 'end'",
        )));
    }

    Ok(lines)
}

/// Batch execution result
#[derive(Debug, Clone)]
pub struct BatchResult {
    pub results: Vec<ScriptResult>,
}

/// Script execution result
#[derive(Debug, Clone)]
pub struct ScriptResult {
    pub script: String,
    pub success: bool,
    pub duration: Duration,
    /// Error message if the script failed
    pub error: Option<String>,
}

impl Default for BatchRunner {
    fn default() -> Self {
        Self::new(BatchConfig::default()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_runner_creation() {
        let runner = BatchRunner::new(BatchConfig::default()).unwrap();
        assert_eq!(runner.config.max_concurrent, 5);
    }

    #[tokio::test]
    async fn test_run_single_script() {
        let runner = BatchRunner::new(BatchConfig::default()).unwrap();

        let script_path = std::env::temp_dir().join("test_batch.lua");
        std::fs::write(&script_path, "print('test')").unwrap();

        let result = runner.run_script(&script_path).await;
        assert!(result.is_ok());

        let _ = std::fs::remove_file(script_path);
    }

    #[test]
    fn test_variable_resolution() {
        let mut runner = BatchRunner::new(BatchConfig::default()).unwrap();
        runner.set_variable("PORT", "/dev/ttyUSB0");
        runner.set_variable("SCRIPT", "modbus");

        let resolved = runner.resolve_variables("scripts/${SCRIPT}.lua");
        assert_eq!(resolved, "scripts/modbus.lua");

        let resolved2 = runner.resolve_variables("$PORT/config.toml");
        assert_eq!(resolved2, "/dev/ttyUSB0/config.toml");
    }

    #[test]
    fn test_parse_batch_file_with_set_and_loop() {
        let runner = BatchRunner::new(BatchConfig::default()).unwrap();

        let batch_path = std::env::temp_dir().join("test.batch");
        std::fs::write(
            &batch_path,
            "# Test batch file\nset PORT /dev/ttyUSB0\nloop 2\n  scripts/${PORT}/test.lua\n  sleep 100\nend\n",
        )
        .unwrap();

        let lines = runner.parse_batch_file(&batch_path).unwrap();
        assert_eq!(lines.len(), 3); // Comment, Set, Loop

        if let BatchLine::Loop { count, body } = &lines[2] {
            assert_eq!(*count, 2);
            assert_eq!(body.len(), 2); // Script + Sleep
        } else {
            panic!("Expected Loop variant");
        }

        let _ = std::fs::remove_file(batch_path);
    }

    #[test]
    fn test_parse_unclosed_loop() {
        let result = parse_batch_lines("loop 3\nscript.lua\n");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unexpected_end() {
        let result = parse_batch_lines("end\n");
        assert!(result.is_err());
    }
}
