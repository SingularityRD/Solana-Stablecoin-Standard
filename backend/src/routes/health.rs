use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;

use crate::{AppState, VERSION, START_TIME};

/// Detailed health status for a component
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

impl ComponentHealth {
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            message: None,
            latency_ms: None,
        }
    }

    pub fn healthy_with_latency(latency_ms: u64) -> Self {
        Self {
            status: "healthy".to_string(),
            message: None,
            latency_ms: Some(latency_ms),
        }
    }

    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: "unhealthy".to_string(),
            message: Some(message.into()),
            latency_ms: None,
        }
    }

    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: "degraded".to_string(),
            message: Some(message.into()),
            latency_ms: None,
        }
    }
}

/// Detailed health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: String,
    pub environment: String,
    pub components: HealthComponents,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthComponents {
    pub database: ComponentHealth,
    pub solana_rpc: ComponentHealth,
    pub memory: MemoryHealth,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryHealth {
    pub status: String,
    pub used_mb: u64,
    pub total_mb: u64,
    pub usage_percent: f64,
}

/// Readiness check response (for Kubernetes)
#[derive(Debug, Serialize, Deserialize)]
pub struct ReadinessResponse {
    pub ready: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<std::collections::HashMap<String, bool>>,
}

/// Liveness check response (for Kubernetes)
#[derive(Debug, Serialize, Deserialize)]
pub struct LivenessResponse {
    pub alive: bool,
}

/// Basic health check handler (legacy endpoint)
pub async fn handler() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

/// Detailed health check handler
/// Returns comprehensive health information including:
/// - Database status
/// - Solana RPC status
/// - Memory usage
/// - Uptime
/// - Version info
pub async fn detailed_handler(
    State(state): State<AppState>,
) -> (StatusCode, Json<HealthResponse>) {
    let start = Instant::now();
    
    // Check database health
    let db_health = check_database(&state).await;
    
    // Check Solana RPC health
    let rpc_health = check_solana_rpc(&state).await;
    
    // Get memory stats
    let memory_health = get_memory_stats();
    
    // Determine overall status
    let overall_status = if db_health.status == "unhealthy" || rpc_health.status == "unhealthy" {
        "unhealthy"
    } else if db_health.status == "degraded" || rpc_health.status == "degraded" || memory_health.status == "degraded" {
        "degraded"
    } else {
        "healthy"
    };

    // Calculate uptime
    let uptime_seconds = START_TIME
        .get()
        .map(|t| t.elapsed().as_secs())
        .unwrap_or(0);

    let response = HealthResponse {
        status: overall_status.to_string(),
        version: VERSION.to_string(),
        uptime_seconds,
        timestamp: chrono::Utc::now().to_rfc3339(),
        environment: state.config.environment.to_string(),
        components: HealthComponents {
            database: db_health,
            solana_rpc: rpc_health,
            memory: memory_health,
        },
    };

    let status_code = if overall_status == "unhealthy" {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };

    tracing::debug!(
        "Health check completed in {:?}ms with status: {}",
        start.elapsed().as_millis(),
        overall_status
    );

    (status_code, Json(response))
}

/// Readiness probe handler for Kubernetes
/// Returns 200 OK only if the service is ready to accept traffic
/// Checks: database connectivity, Solana RPC availability
pub async fn readiness_handler(
    State(state): State<AppState>,
) -> (StatusCode, Json<ReadinessResponse>) {
    let mut checks = std::collections::HashMap::new();
    let mut all_ready = true;

    // Check database
    let db_ready = state.db.health_check().await.is_ok();
    checks.insert("database".to_string(), db_ready);
    if !db_ready {
        all_ready = false;
    }

    // Check Solana RPC
    let rpc_ready = state.solana.health_check().await.unwrap_or(false);
    checks.insert("solana_rpc".to_string(), rpc_ready);
    if !rpc_ready {
        all_ready = false;
    }

    let status_code = if all_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(ReadinessResponse {
            ready: all_ready,
            checks: Some(checks),
        }),
    )
}

/// Liveness probe handler for Kubernetes
/// Returns 200 OK if the service is running (doesn't check dependencies)
pub async fn liveness_handler() -> (StatusCode, Json<LivenessResponse>) {
    (
        StatusCode::OK,
        Json(LivenessResponse { alive: true }),
    )
}

/// Check database connectivity and response time
async fn check_database(state: &AppState) -> ComponentHealth {
    let start = Instant::now();
    
    match state.db.health_check().await {
        Ok(()) => {
            let latency = start.elapsed().as_millis() as u64;
            if latency > 100 {
                ComponentHealth::degraded(format!(
                    "Database response time slow: {}ms",
                    latency
                ))
            } else {
                ComponentHealth::healthy_with_latency(latency)
            }
        }
        Err(e) => ComponentHealth::unhealthy(format!("Database connection failed: {}", e)),
    }
}

/// Check Solana RPC connectivity and response time
async fn check_solana_rpc(state: &AppState) -> ComponentHealth {
    let start = Instant::now();
    
    match state.solana.health_check().await {
        Ok(true) => {
            let latency = start.elapsed().as_millis() as u64;
            if latency > 500 {
                ComponentHealth::degraded(format!(
                    "RPC response time slow: {}ms",
                    latency
                ))
            } else {
                ComponentHealth::healthy_with_latency(latency)
            }
        }
        Ok(false) => ComponentHealth::unhealthy("RPC health check returned false"),
        Err(e) => ComponentHealth::unhealthy(format!("RPC connection failed: {}", e)),
    }
}

/// Get memory usage statistics
fn get_memory_stats() -> MemoryHealth {
    // Use a simple approach that works cross-platform
    // For more detailed stats, consider using the `sysinfo` crate
    
    // Get memory info using standard library where possible
    // Note: Rust doesn't have built-in memory stats, so we use a heuristic
    
    // On Windows/Linux, we can try to get memory info
    #[cfg(target_os = "windows")]
    {
        // Windows: Use GlobalMemoryStatusEx equivalent
        get_memory_stats_windows()
    }
    
    #[cfg(target_os = "linux")]
    {
        get_memory_stats_linux()
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        // Fallback for other platforms
        MemoryHealth {
            status: "healthy".to_string(),
            used_mb: 0,
            total_mb: 0,
            usage_percent: 0.0,
        }
    }
}

#[cfg(target_os = "windows")]
fn get_memory_stats_windows() -> MemoryHealth {
    use std::mem::{size_of, zeroed};
    
    #[repr(C)]
    struct MemoryStatusEx {
        dw_length: u32,
        dw_memory_load: u32,
        ull_total_phys: u64,
        ull_avail_phys: u64,
        ull_total_page_file: u64,
        ull_avail_page_file: u64,
        ull_total_virtual: u64,
        ull_avail_virtual: u64,
        ull_avail_extended_virtual: u64,
    }
    
    // SAFETY: We're using kernel32 API which is available on all Windows versions
    // This is a simple fallback implementation
    unsafe {
        let mut status: MemoryStatusEx = zeroed();
        status.dw_length = size_of::<MemoryStatusEx>() as u32;
        
        // kernel32::GlobalMemoryStatusEx
        #[link(name = "kernel32")]
        extern "system" {
            fn GlobalMemoryStatusEx(lp_buffer: *mut MemoryStatusEx) -> i32;
        }
        
        let result = GlobalMemoryStatusEx(&mut status);
        
        if result != 0 {
            let total_mb = status.ull_total_phys / (1024 * 1024);
            let used_mb = (status.ull_total_phys - status.ull_avail_phys) / (1024 * 1024);
            let usage_percent = status.dw_memory_load as f64;
            
            let mem_status = if usage_percent > 90.0 {
                "critical"
            } else if usage_percent > 75.0 {
                "degraded"
            } else {
                "healthy"
            };
            
            MemoryHealth {
                status: mem_status.to_string(),
                used_mb,
                total_mb,
                usage_percent,
            }
        } else {
            MemoryHealth {
                status: "healthy".to_string(),
                used_mb: 0,
                total_mb: 0,
                usage_percent: 0.0,
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn get_memory_stats_linux() -> MemoryHealth {
    use std::fs;
    
    // Read /proc/meminfo for memory stats
    if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        let mut total_kb: u64 = 0;
        let mut available_kb: u64 = 0;
        
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total_kb = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                available_kb = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        }
        
        let total_mb = total_kb / 1024;
        let used_mb = (total_kb.saturating_sub(available_kb)) / 1024;
        let usage_percent = if total_kb > 0 {
            ((total_kb - available_kb) as f64 / total_kb as f64) * 100.0
        } else {
            0.0
        };
        
        let mem_status = if usage_percent > 90.0 {
            "critical"
        } else if usage_percent > 75.0 {
            "degraded"
        } else {
            "healthy"
        };
        
        MemoryHealth {
            status: mem_status.to_string(),
            used_mb,
            total_mb,
            usage_percent,
        }
    } else {
        MemoryHealth {
            status: "healthy".to_string(),
            used_mb: 0,
            total_mb: 0,
            usage_percent: 0.0,
        }
    }
}