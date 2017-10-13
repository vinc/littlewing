extern crate time;

#[derive(Clone)]
pub struct Clock {
    moves_level: u16,
    //time_level: u64, // TODO: check that we really don't need it

    moves_remaining: u16,
    time_remaining: u64,

    pub started_at: u64,
    pub polling_nodes_count: u64,
    last_nodes_count: u64,
    is_finished: bool
}

impl Clock {
    pub fn new(moves: u16, time: u64) -> Clock {
        Clock {
            moves_level: moves,
            //time_level: time,
            moves_remaining: moves,
            time_remaining: time,
            started_at: 0,
            polling_nodes_count: 100,
            last_nodes_count: 0,
            is_finished: false,
        }
    }

    pub fn start(&mut self, ply: usize) {
        self.is_finished = false;
        self.last_nodes_count = 0;
        self.started_at = (time::precise_time_s() * 1000.0) as u64;

        assert!(ply > 0);
        let moves_done = (((ply - 1) / 2) as u16) % self.moves_level;
        self.moves_remaining = self.moves_level - moves_done;
    }

    pub fn set_time(&mut self, time: u64) {
        self.time_remaining = time;
    }

    pub fn allocated_time(&self) -> u64 {
        self.time_remaining / self.moves_remaining as u64
    }

    pub fn elapsed_time(&self) -> u64 {
        let now = (time::precise_time_s() * 1000.0) as u64;

        now - self.started_at
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

            self.is_finished = delta + self.elapsed_time() > self.allocated_time();
        }

        self.is_finished
    }
}
