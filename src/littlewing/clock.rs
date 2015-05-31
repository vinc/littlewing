extern crate time;

pub struct Clock {
    pub moves: u8,
    pub time: u16,
    pub started_at: f64
}

impl Clock {
    pub fn new(moves: u8, time: u16) -> Clock {
        Clock {
            moves: moves,
            time: time,
            started_at: 0.0
        }
    }
    pub fn start(&mut self) {
        self.started_at = time::precise_time_s();
    }
    pub fn allocated_time(&self) -> f64 {
        self.time as f64 / self.moves as f64
    }
    pub fn elapsed_time(&self) -> f64 {
        time::precise_time_s() - self.started_at
    }
    pub fn poll(&self) -> bool {
        self.elapsed_time() > self.allocated_time()
    }
}
