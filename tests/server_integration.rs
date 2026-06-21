//! Server 模式集成测试
//!
//! 测试 Server 的生命周期管理、错误处理和并发连接
//! 直接测试核心逻辑，避免依赖 Tauri 框架的测试基础设施

use serial_cli::server::{ConnectionContext, ServerConfig, ServerState};
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime};

// ============================================================================
// 生命周期测试
// ============================================================================

#[tokio::test]
async fn test_server_state_creation() {
    let config = ServerConfig::default();
    let state = ServerState::new(config).await;

    // 验证初始状态
    let stats = state.connection_stats().await;
    assert_eq!(stats.active, 0, "初始连接数应该为 0");
    assert_eq!(stats.max, 10, "默认最大连接数应该为 10");

    assert_eq!(
        state.total_requests.load(Ordering::SeqCst),
        0,
        "初始请求数应该为 0"
    );
    assert_eq!(
        state.total_errors.load(Ordering::SeqCst),
        0,
        "初始错误数应该为 0"
    );
}

#[tokio::test]
async fn test_server_config_customization() {
    let config = ServerConfig {
        socket_path: Some("/tmp/test-serial-cli.sock".into()),
        tcp_port: None,
        max_connections: 5,
        log_path: "/tmp/test-server.log".into(),
        idle_timeout_secs: 60,
    };

    let state = ServerState::new(config).await;
    let stats = state.connection_stats().await;

    assert_eq!(stats.max, 5, "自定义最大连接数应该生效");
}

// ============================================================================
// 连接管理测试
// ============================================================================

#[tokio::test]
async fn test_add_single_connection() {
    let config = ServerConfig::default();
    let state = ServerState::new(config).await;

    let ctx = ConnectionContext {
        connection_id: "test_conn_1".to_string(),
        port_id: Some("/dev/ttyUSB0".to_string()),
        protocol_name: Some("modbus_rtu".to_string()),
        created_at: SystemTime::now(),
        last_activity: SystemTime::now(),
        subscribed: false,
    };

    let result = state.add_connection(ctx).await;
    assert!(result.is_ok(), "添加连接应该成功");

    let stats = state.connection_stats().await;
    assert_eq!(stats.active, 1, "活跃连接数应该为 1");
}

#[tokio::test]
async fn test_remove_connection() {
    let config = ServerConfig::default();
    let state = ServerState::new(config).await;

    let ctx = ConnectionContext {
        connection_id: "test_conn_1".to_string(),
        port_id: Some("/dev/ttyUSB0".to_string()),
        protocol_name: None,
        created_at: SystemTime::now(),
        last_activity: SystemTime::now(),
        subscribed: false,
    };

    // 添加连接
    let _ = state.add_connection(ctx).await;

    // 移除连接
    let removed = state.remove_connection("test_conn_1").await;
    assert!(removed.is_some(), "应该能移除存在的连接");

    let stats = state.connection_stats().await;
    assert_eq!(stats.active, 0, "移除后活跃连接数应该为 0");
}

#[tokio::test]
async fn test_remove_nonexistent_connection() {
    let config = ServerConfig::default();
    let state = ServerState::new(config).await;

    let removed = state.remove_connection("nonexistent").await;
    assert!(removed.is_none(), "移除不存在的连接应该返回 None");
}

#[tokio::test]
async fn test_update_connection_activity() {
    let config = ServerConfig::default();
    let state = ServerState::new(config).await;

    let ctx = ConnectionContext {
        connection_id: "test_conn_1".to_string(),
        port_id: None,
        protocol_name: None,
        created_at: SystemTime::now(),
        last_activity: SystemTime::now() - Duration::from_secs(60), // 60秒前
        subscribed: false,
    };

    let _ = state.add_connection(ctx).await;

    // 更新活动状态
    state.update_activity("test_conn_1").await;

    // 验证活动时间已更新
    let connections = state.connections.read().await;
    let conn = connections.get("test_conn_1").unwrap();
    let now = SystemTime::now();
    let elapsed = now.duration_since(conn.last_activity).unwrap();

    assert!(
        elapsed < Duration::from_secs(1),
        "活动状态应该被更新为最近时间"
    );
}

// ============================================================================
// 错误处理测试
// ============================================================================

#[tokio::test]
async fn test_max_connections_limit() {
    let config = ServerConfig {
        socket_path: None,
        tcp_port: None,
        max_connections: 2,
        log_path: "/tmp/test-server.log".into(),
        idle_timeout_secs: 300,
    };
    let state = ServerState::new(config).await;

    // 添加第一个连接
    let ctx1 = ConnectionContext {
        connection_id: "conn_1".to_string(),
        port_id: None,
        protocol_name: None,
        created_at: SystemTime::now(),
        last_activity: SystemTime::now(),
        subscribed: false,
    };
    assert!(state.add_connection(ctx1).await.is_ok());

    // 添加第二个连接
    let ctx2 = ConnectionContext {
        connection_id: "conn_2".to_string(),
        port_id: None,
        protocol_name: None,
        created_at: SystemTime::now(),
        last_activity: SystemTime::now(),
        subscribed: false,
    };
    assert!(state.add_connection(ctx2).await.is_ok());

    // 尝试添加第三个连接（应该失败）
    let ctx3 = ConnectionContext {
        connection_id: "conn_3".to_string(),
        port_id: None,
        protocol_name: None,
        created_at: SystemTime::now(),
        last_activity: SystemTime::now(),
        subscribed: false,
    };
    assert!(
        state.add_connection(ctx3).await.is_err(),
        "超过最大连接数应该返回错误"
    );

    let stats = state.connection_stats().await;
    assert_eq!(stats.active, 2, "活跃连接数应该保持为 2");
}

#[tokio::test]
async fn test_is_max_connections_reached() {
    let config = ServerConfig {
        max_connections: 2,
        ..Default::default()
    };
    let state = ServerState::new(config).await;

    assert!(
        !state.is_max_connections_reached().await,
        "初始时未达到最大连接数"
    );

    // 添加一个连接
    let ctx = ConnectionContext {
        connection_id: "conn_1".to_string(),
        port_id: None,
        protocol_name: None,
        created_at: SystemTime::now(),
        last_activity: SystemTime::now(),
        subscribed: false,
    };
    let _ = state.add_connection(ctx).await;

    assert!(
        !state.is_max_connections_reached().await,
        "1个连接未达到最大值2"
    );

    // 添加第二个连接
    let ctx = ConnectionContext {
        connection_id: "conn_2".to_string(),
        port_id: None,
        protocol_name: None,
        created_at: SystemTime::now(),
        last_activity: SystemTime::now(),
        subscribed: false,
    };
    let _ = state.add_connection(ctx).await;

    assert!(
        state.is_max_connections_reached().await,
        "2个连接达到最大值2"
    );
}

// ============================================================================
// 并发测试
// ============================================================================

#[tokio::test]
async fn test_concurrent_connection_add_remove() {
    let config = ServerConfig::default();
    let state = ServerState::new(config).await;

    let mut handles = vec![];

    // 并发添加 10 个连接
    for i in 0..10 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            let ctx = ConnectionContext {
                connection_id: format!("conn_{}", i),
                port_id: None,
                protocol_name: None,
                created_at: SystemTime::now(),
                last_activity: SystemTime::now(),
                subscribed: false,
            };
            state_clone.add_connection(ctx).await
        });
        handles.push(handle);
    }

    // 等待所有添加完成
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "并发添加连接应该成功");
    }

    let stats = state.connection_stats().await;
    assert_eq!(stats.active, 10, "应该有 10 个活跃连接");

    // 并发移除 10 个连接
    let mut handles = vec![];
    for i in 0..10 {
        let state_clone = state.clone();
        let handle =
            tokio::spawn(
                async move { state_clone.remove_connection(&format!("conn_{}", i)).await },
            );
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_some(), "并发移除连接应该成功");
    }

    let stats = state.connection_stats().await;
    assert_eq!(stats.active, 0, "所有连接应该被移除");
}

#[tokio::test]
async fn test_concurrent_stats_queries() {
    let config = ServerConfig::default();
    let state = ServerState::new(config).await;

    // 添加一些连接
    for i in 0..5 {
        let ctx = ConnectionContext {
            connection_id: format!("conn_{}", i),
            port_id: None,
            protocol_name: None,
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
            subscribed: false,
        };
        let _ = state.add_connection(ctx).await;
    }

    // 并发查询统计
    let mut handles = vec![];
    for _ in 0..20 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move { state_clone.connection_stats().await });
        handles.push(handle);
    }

    // 所有查询应该返回一致的结果
    for handle in handles {
        let stats = handle.await.unwrap();
        assert_eq!(stats.active, 5, "所有查询应该返回相同的活跃连接数");
    }
}

#[tokio::test]
async fn test_concurrent_activity_updates() {
    let config = ServerConfig::default();
    let state = ServerState::new(config).await;

    let ctx = ConnectionContext {
        connection_id: "conn_1".to_string(),
        port_id: None,
        protocol_name: None,
        created_at: SystemTime::now(),
        last_activity: SystemTime::now(),
        subscribed: false,
    };
    let _ = state.add_connection(ctx).await;

    // 并发更新活动状态
    let mut handles = vec![];
    for _ in 0..10 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            state_clone.update_activity("conn_1").await;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // 验证活动时间已更新
    let connections = state.connections.read().await;
    let conn = connections.get("conn_1").unwrap();
    let now = SystemTime::now();
    let elapsed = now.duration_since(conn.last_activity).unwrap();

    assert!(elapsed < Duration::from_secs(1), "活动状态应该被正确更新");
}

// ============================================================================
// 空闲连接清理测试
// ============================================================================

#[tokio::test]
async fn test_idle_connection_cleanup() {
    let config = ServerConfig {
        idle_timeout_secs: 2, // 2秒超时
        ..Default::default()
    };
    let state = ServerState::new(config).await;

    // 添加一个旧连接（3秒前）
    let old_ctx = ConnectionContext {
        connection_id: "old_conn".to_string(),
        port_id: None,
        protocol_name: None,
        created_at: SystemTime::now() - Duration::from_secs(3),
        last_activity: SystemTime::now() - Duration::from_secs(3),
        subscribed: false,
    };
    let _ = state.add_connection(old_ctx).await;

    // 添加一个新连接
    let new_ctx = ConnectionContext {
        connection_id: "new_conn".to_string(),
        port_id: None,
        protocol_name: None,
        created_at: SystemTime::now(),
        last_activity: SystemTime::now(),
        subscribed: false,
    };
    let _ = state.add_connection(new_ctx).await;

    // 清理空闲连接
    let removed = state.cleanup_idle_connections().await;

    assert_eq!(removed.len(), 1, "应该移除 1 个空闲连接");
    assert_eq!(removed[0], "old_conn", "应该移除旧连接");

    let stats = state.connection_stats().await;
    assert_eq!(stats.active, 1, "应该剩余 1 个活跃连接");
}

#[tokio::test]
async fn test_no_idle_connections_to_cleanup() {
    let config = ServerConfig {
        idle_timeout_secs: 60,
        ..Default::default()
    };
    let state = ServerState::new(config).await;

    // 添加一个活跃连接
    let ctx = ConnectionContext {
        connection_id: "active_conn".to_string(),
        port_id: None,
        protocol_name: None,
        created_at: SystemTime::now(),
        last_activity: SystemTime::now(),
        subscribed: false,
    };
    let _ = state.add_connection(ctx).await;

    // 清理空闲连接
    let removed = state.cleanup_idle_connections().await;

    assert_eq!(removed.len(), 0, "不应该移除任何连接");

    let stats = state.connection_stats().await;
    assert_eq!(stats.active, 1, "活跃连接数应该保持不变");
}
