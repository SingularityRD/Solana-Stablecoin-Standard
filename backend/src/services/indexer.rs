use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio::sync::RwLock;

pub struct EventIndexer {
    pub rpc_url: String,
    pub program_id: String,
    running: Arc<RwLock<bool>>,
}

impl EventIndexer {
    pub fn new(rpc_url: &str, program_id: String) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            program_id,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Background task to poll signatures and parse Anchor events
    pub async fn start_polling(&self) {
        tracing::info!("Started indexing events for program {}", self.program_id);
        
        let mut running = self.running.write().await;
        *running = true;
        drop(running);
        
        loop {
            let is_running = *self.running.read().await;
            if !is_running {
                break;
            }
            
            // In production: 
            // 1. Fetch recent signatures for self.program_id
            // 2. GetTransaction for new signatures
            // 3. Parse inner instructions and log messages for Anchor Events
            // 4. Push to database or emit webhooks
            
            sleep(Duration::from_secs(10)).await;
        }
    }
    
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }
}