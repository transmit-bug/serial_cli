//! Concurrency stress tests for task executor and script manager
//!
//! These tests verify thread safety and correctness under high concurrency.

use serial_cli::lua::executor::ScriptEngine;
use serial_cli::script::ScriptManager;
use serial_cli::task::{executor::TaskExecutor, Task, TaskPriority, TaskType};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Barrier, Mutex};
use tokio::time::timeout;

/// Test concurrent task submission from multiple threads
#[tokio::test]
async fn test_concurrent_task_submission() {
    let executor = Arc::new(TaskExecutor::new(10));
    let executor_clone1 = executor.clone();
    let executor_clone2 = executor.clone();

    executor_clone1.start().await.unwrap();

    let num_threads = 20; // Reduced for faster testing
    let tasks_per_thread = 5;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];

    // Spawn multiple threads that all submit tasks simultaneously
    for thread_id in 0..num_threads {
        let executor_thread = executor.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await; // Synchronize start

            for task_id in 0..tasks_per_thread {
                let task = Task::new(TaskType::Custom {
                    name: format!("thread_{}_task_{}", thread_id, task_id),
                    data: format!("data_{}", task_id),
                });

                executor_thread
                    .submit(task, TaskPriority::Normal)
                    .await
                    .unwrap();
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all tasks were submitted (some may have started executing)
    let expected_total = num_threads * tasks_per_thread;
    let actual_pending = executor_clone2.pending_count().await;
    let actual_completed = executor_clone2.get_completed().await.len();
    let actual_running = executor_clone2.running_count().await;

    // All tasks should be accounted for (pending + completed + running)
    let total_accounted = actual_pending + actual_completed + actual_running;
    assert_eq!(
        total_accounted, expected_total,
        "All tasks should be accounted for: expected {}, got pending={} completed={} running={}",
        expected_total, actual_pending, actual_completed, actual_running
    );

    executor_clone1.stop().await.unwrap();
}

/// Test concurrent script operations
#[tokio::test]
async fn test_concurrent_script_operations() {
    let manager = Arc::new(Mutex::new(ScriptManager::new()));
    let num_threads = 20;
    let ops_per_thread = 10; // Reduced for faster execution
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];

    // Spawn multiple threads performing script operations
    for thread_id in 0..num_threads {
        let manager_clone = manager.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await; // Synchronize start

            for _op_id in 0..ops_per_thread {
                let mgr = manager_clone.lock().await;

                // List scripts
                let scripts = mgr.list();

                // Verify we have scripts
                assert!(!scripts.is_empty(), "Should have registered scripts");
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        let result = timeout(Duration::from_secs(10), handle).await;
        assert!(result.is_ok(), "Thread operation timed out");
    }
}

/// Test task executor under high load
#[tokio::test]
async fn test_high_load_task_execution() {
    let executor = TaskExecutor::new(50); // Higher concurrency
    executor.start().await.unwrap();

    let num_tasks = 100; // Reduced for faster execution
    let start_time = Instant::now();

    // Submit many tasks rapidly
    for i in 0..num_tasks {
        let task = Task::new(TaskType::Custom {
            name: format!("load_test_{}", i),
            data: format!("load_data_{}", i),
        });

        executor.submit(task, TaskPriority::Normal).await.unwrap();
    }

    // Wait for tasks to be submitted and start processing
    tokio::time::sleep(Duration::from_millis(500)).await;

    let completed = executor.get_completed().await;
    let running = executor.running_count().await;
    let pending = executor.pending_count().await;

    let elapsed = start_time.elapsed();

    // Verify all tasks are accounted for (pending, running, or completed)
    let total_accounted = completed.len() + running + pending;
    assert_eq!(
        total_accounted,
        num_tasks,
        "All tasks should be accounted for: expected {}, got completed={} running={} pending={}",
        num_tasks,
        completed.len(),
        running,
        pending
    );

    // Verify performance (should not be too slow)
    assert!(
        elapsed < Duration::from_secs(5),
        "Task execution took too long: {:?}",
        elapsed
    );

    executor.stop().await.unwrap();
}

/// Test concurrent access to script manager
#[tokio::test]
async fn test_concurrent_script_manager_access() {
    let manager = Arc::new(Mutex::new(ScriptManager::new()));
    let num_threads = 20;
    let ops_per_thread = 10;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];

    // Spawn threads that perform script operations concurrently
    for _thread_id in 0..num_threads {
        let manager_clone = manager.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await; // Synchronize start

            for _op_id in 0..ops_per_thread {
                let mgr = manager_clone.lock().await;

                // List scripts
                let scripts = mgr.list();

                // Verify we have scripts
                assert!(!scripts.is_empty(), "Should have registered scripts");
            }
        });

        handles.push(handle);
    }

    // Wait for all threads with timeout
    for handle in handles {
        let result = timeout(Duration::from_secs(15), handle).await;
        assert!(result.is_ok(), "Script manager operation timed out");
    }

    // Verify final state
    let final_mgr = manager.lock().await;
    let final_scripts = final_mgr.list();
    assert!(
        !final_scripts.is_empty(),
        "Should have built-in scripts"
    );
}

/// Test race condition prevention in task completion tracking
#[tokio::test]
async fn test_task_completion_tracking_consistency() {
    let executor = TaskExecutor::new(10);
    executor.start().await.unwrap();

    let num_tasks = 100;
    let mut task_ids = vec![];

    // Submit tasks and track their IDs
    for i in 0..num_tasks {
        let task = Task::new(TaskType::Custom {
            name: format!("consistency_test_{}", i),
            data: format!("data_{}", i),
        });

        let id = task.id();
        executor.submit(task, TaskPriority::Normal).await.unwrap();
        task_ids.push(id);
    }

    // Wait for tasks to process
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify completion tracking is consistent
    let completed = executor.get_completed().await;

    // Check that there are no duplicate task IDs in completions
    let mut unique_ids = std::collections::HashSet::new();
    for completion in &completed {
        assert!(
            unique_ids.insert(&completion.task_id),
            "Duplicate task ID in completions: {}",
            completion.task_id
        );
    }

    executor.stop().await.unwrap();
}

/// Test concurrent Lua script execution
#[tokio::test]
async fn test_concurrent_lua_execution() {
    let num_threads = 20;
    let scripts_per_thread = 10;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await; // Synchronize start

            for script_id in 0..scripts_per_thread {
                let engine = ScriptEngine::new().unwrap();
                let script = format!(
                    r#"
                    local x = {}
                    local y = {}
                    return x + y
                "#,
                    thread_id, script_id
                );

                let result = engine.execute_string(&script);
                assert!(
                    result.is_ok(),
                    "Lua execution failed for thread {}, script {}",
                    thread_id,
                    script_id
                );
            }
        });

        handles.push(handle);
    }

    // Wait for all threads with timeout
    for handle in handles {
        let result = timeout(Duration::from_secs(10), handle).await;
        assert!(result.is_ok(), "Concurrent Lua execution timed out");
    }
}
