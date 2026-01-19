use std::prelude::v1::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[cfg(feature = "std")]
fn default_system_time() -> u128 {
    use std::time::SystemTime;
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
}

#[cfg(not(feature = "std"))]
fn default_system_time() -> u128 {
    0 // NOTE: Must be overrided in Clock by external crates
}

#[derive(Clone)]
pub struct Clock {
    pub system_time: Arc<dyn Fn() -> u128 + Send + Sync + 'static>,
    pub polling_nodes_count: u64,
    pub started_at: u128,
    moves_level: u16,
    moves_remaining: u16,
    time_remaining: u64,
    last_nodes_count: u64,
    is_finished: Arc<AtomicBool>,
    is_level: bool // TODO: find a better name
}

impl Clock {
    pub fn new(moves: u16, time: u64) -> Clock {
        let system_time = Arc::new(default_system_time);
        Clock {
            system_time: system_time,
            polling_nodes_count: 100,
            started_at: 0,
            moves_level: moves,
            moves_remaining: if moves > 0 { moves } else { 20 },
            time_remaining: time,
            last_nodes_count: 0,
            is_finished: Arc::new(AtomicBool::new(false)),
            is_level: true
        }
    }

    pub fn start(&mut self, ply: usize) {
        self.is_finished.store(false, Ordering::Relaxed);
        self.last_nodes_count = 0;
        self.started_at = (self.system_time)();

        // The UCI protocol gives the number of remaining moves before each
        // search but XBoard doesn't so we need to calculate it based on moves
        // history and the level command.
        if self.is_level && self.moves_level > 0 {
            assert!(ply > 0);
            let moves_done = (((ply - 1) / 2) as u16) % self.moves_level;
            self.moves_remaining = self.moves_level - moves_done;
        }
    }

    pub fn stop(&mut self) {
        self.is_finished.store(true, Ordering::Relaxed);
    }

    pub fn disable_level(&mut self) {
        self.is_level = false;
    }

    pub fn set_time(&mut self, time: u64) {
        self.time_remaining = time;
    }

    pub fn allocated_time(&self) -> u64 {
        self.time_remaining / self.moves_remaining as u64
    }

    pub fn elapsed_time(&self) -> u64 {
        ((self.system_time)() - self.started_at) as u64
    }

    pub fn poll(&mut self, nodes_count: u64) -> bool {
        // We do the real computation only every `polling_nodes_count` nodes
        // TODO: do we need this?
        if nodes_count - self.last_nodes_count > self.polling_nodes_count {
            self.last_nodes_count = nodes_count;

            // A certain amount of time pass between two polls,
            // and after the end of the search.
            let time_between_polls = self.polling_nodes_count / 4;
            let time_to_play = 25;
            let delta = time_between_polls + time_to_play;

            if delta + self.elapsed_time() > self.allocated_time() {
                self.is_finished.store(true, Ordering::Relaxed);
            }
        }

        self.is_finished.load(Ordering::Relaxed)
    }
}
