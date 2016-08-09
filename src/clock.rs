extern crate time;

pub struct Clock {
    pub moves: u8,
    pub time: u16,
    pub started_at: f64,
    pub polling_nodes_count: u64,
    last_nodes_count: u64,
    is_finished: bool
}

impl Clock {
    pub fn new(moves: u8, time: u16) -> Clock {
        Clock {
            moves: moves,
            time: time,
            started_at: 0.0,
            polling_nodes_count: 1000,
            last_nodes_count: 0,
            is_finished: false
        }
    }
    pub fn start(&mut self) {
        self.is_finished = false;
        self.last_nodes_count = 0;
        self.started_at = time::precise_time_s();
    }
    pub fn allocated_time(&self) -> f64 {
        self.time as f64 / self.moves as f64
    }
    pub fn elapsed_time(&self) -> f64 {
        time::precise_time_s() - self.started_at
    }
    pub fn poll(&mut self, nodes_count: u64) -> bool {
        if nodes_count - self.last_nodes_count > self.polling_nodes_count {
            self.last_nodes_count = nodes_count;
            self.is_finished = self.elapsed_time() > self.allocated_time();
        }

        self.is_finished
    }
}
