use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

#[derive(Clone)]
pub struct Clock {
    pub polling_nodes_count: u64,
    pub started_at: Instant,
    moves_level: u16,
    moves_remaining: u16,
    time_remaining: u64,
    last_nodes_count: u64,
    is_finished: Arc<AtomicBool>,
    is_level: bool // TODO: find a better name
}

impl Clock {
    pub fn new(moves: u16, time: u64) -> Clock {
        Clock {
            polling_nodes_count: 100,
            started_at: Instant::now(),
            moves_level: moves,
            moves_remaining: moves,
            time_remaining: time,
            last_nodes_count: 0,
            is_finished: Arc::new(AtomicBool::new(false)),
            is_level: true
        }
    }

    pub fn start(&mut self, ply: usize) {
        self.is_finished.store(false, Ordering::Relaxed);
        self.last_nodes_count = 0;
        self.started_at = Instant::now();

        // The UCI protocol gives the number of remaining moves before each
        // search but XBoard doesn't so we need to calculate it based on moves
        // history and the level command.
        if self.is_level {
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
        self.started_at.elapsed().as_millis() as u64
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
