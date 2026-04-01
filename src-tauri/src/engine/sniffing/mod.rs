pub mod handle;
pub mod orchestrator;

use std::sync::Arc;
use parking_lot::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    /// Global instance of the sniffing driver handle.
    pub static ref SNIFFER: Arc<Mutex<Option<handle::SniffingHandle>>> = Arc::new(Mutex::new(None));
}

/// Attempts to initialize the WFP driver handle.
pub fn init_sniffer() -> Result<(), String> {
    let mut lock = SNIFFER.lock();
    if lock.is_none() {
        match handle::SniffingHandle::open() {
            Ok(h) => {
                *lock = Some(h);
                Ok(())
            }
            Err(e) => Err(e),
        }
    } else {
        Ok(())
    }
}

/// Checks if the kernel-mode driver is currently linked.
pub fn is_sniffer_active() -> bool {
    SNIFFER.lock().is_some()
}
