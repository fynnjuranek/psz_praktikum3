use crate::{elevator::Direction, passenger::Passenger};

pub struct LevelQueue {
    up_queue: Vec<Passenger>,
    down_queue: Vec<Passenger>,
}

impl LevelQueue {
    pub fn new() -> Self {
        Self {
            up_queue: Vec::new(),
            down_queue: Vec::new(),
        }
    }

    pub fn add_passenger(&mut self, passenger: Passenger) {
        match passenger.clone().get_direction() {
            Direction::Up => self.up_queue.push(passenger),
            Direction::Down => self.down_queue.push(passenger),
        }
    }

    pub fn get_passenger(&mut self, direction: Direction) -> Option<Passenger> {
        match direction {
            Direction::Up => self.up_queue.pop(),
            Direction::Down => self.down_queue.pop(),
        }
    }

    pub fn get_up_queue(&self) -> &Vec<Passenger> {
        &self.down_queue
    }

    pub fn get_down_queue(&self) -> &Vec<Passenger> {
        &self.down_queue
    }
}

pub struct PendingRequestQueue {
    requests: Vec<Passenger>,
}

impl PendingRequestQueue {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    pub fn add_request(&mut self, passenger: Passenger) {
        self.requests.push(passenger);
    }

    pub fn get_request(&mut self) -> Option<Passenger> {
        self.requests.pop()
    }
}
