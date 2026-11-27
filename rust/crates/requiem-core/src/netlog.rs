use crate::utils::time;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

const LIMIT: usize = 300;

#[derive(Clone, Debug)]
pub struct NetEntry {
    pub seq: u64,

    pub at_ms: u64,
    pub method: String,

    pub path: String,

    pub status: Option<u16>,
    pub error: Option<String>,
    pub duration_ms: u64,

    pub bytes: Option<u64>,
}

static LOG: Mutex<VecDeque<NetEntry>> = Mutex::new(VecDeque::new());
static VERSION: AtomicU64 = AtomicU64::new(0);

pub fn record(mut entry: NetEntry) {
    entry.seq = VERSION.fetch_add(1, Ordering::AcqRel);
    entry.at_ms = time::time_of_day_ms();
    let mut log = LOG.lock().expect("net log poisoned");
    log.push_back(entry);
    if log.len() > LIMIT {
        log.pop_front();
    }
}

pub fn version() -> u64 {
    VERSION.load(Ordering::Acquire)
}

pub fn entries() -> Vec<NetEntry> {
    LOG.lock()
        .expect("net log poisoned")
        .iter()
        .cloned()
        .collect()
}

pub fn clear() {
    LOG.lock().expect("net log poisoned").clear();
    VERSION.fetch_add(1, Ordering::AcqRel);
}
