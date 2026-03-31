use std::sync::Arc;
use tokio::sync::Mutex;
use rand::Rng;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct NetworkConditions {
    pub latency_ms: u64,
    pub packet_loss_rate: f64, // 0.0 to 1.0
}

#[derive(Clone)]
pub struct SimulationEngine {
    pub conditions: Arc<Mutex<NetworkConditions>>,
}

impl SimulationEngine {
    pub fn new() -> Self {
        Self {
            conditions: Arc::new(Mutex::new(NetworkConditions::default())),
        }
    }

    pub async fn apply(&self) {
        let cond = self.conditions.lock().await;
        
        // 1. Latency Simulation
        if cond.latency_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(cond.latency_ms)).await;
        }

        // 2. Packet Loss Simulation
        if cond.packet_loss_rate > 0.0 {
            let triggered = rand::thread_rng().gen_bool(cond.packet_loss_rate);
            if triggered {
                // In a real implementation, we'd return a ConnectionReset error here.
                // For simplicity in this demo, we just add more latency to simulate retries.
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
    }
}
