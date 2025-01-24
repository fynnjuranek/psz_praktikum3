use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const LEVELS: usize = 4;
const ELEVATORS: usize = 3;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Direction {
    Up,
    Down,
}

enum DoorState {
    Closed,
    Closing,
    Open,
    Opening,
}

#[derive(Debug, Clone)]
struct Passenger {
    id: usize,
    direction: Direction,
    destination: usize,
    idle_on_level: bool,
    entering_elevator: bool,
    in_elevator: bool,
    leaving_elevator: bool,
    current_level: usize,
}

impl Passenger {
    fn update_current_level(&mut self, new_level: usize) {
        self.current_level = new_level;
    }

    fn set_idle_on_level(&mut self, idle: bool) {
        self.idle_on_level = idle;
    }

    fn set_entering_elevator(&mut self, entering: bool) {
        self.entering_elevator = entering;
    }

    fn set_in_elevator(&mut self, in_elevator: bool) {
        self.in_elevator = in_elevator;
    }

    fn set_leaving_elevator(&mut self, leaving: bool) {
        self.leaving_elevator = leaving;
    }
}

struct LevelQueue {
    up_queue: Vec<Passenger>,
    down_queue: Vec<Passenger>,
}

impl LevelQueue {
    fn new() -> Self {
        Self {
            up_queue: Vec::new(),
            down_queue: Vec::new(),
        }
    }

    fn add_passenger(&mut self, passenger: Passenger) {
        match passenger.direction {
            Direction::Up => self.up_queue.push(passenger),
            Direction::Down => self.down_queue.push(passenger),
        }
    }

    fn get_passenger(&mut self, direction: Direction) -> Option<Passenger> {
        match direction {
            Direction::Up => self.up_queue.pop(),
            Direction::Down => self.down_queue.pop(),
        }
    }
}

struct PendingRequestQueue {
    requests: Vec<Passenger>,
}

impl PendingRequestQueue {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    fn add_request(&mut self, passenger: Passenger) {
        self.requests.push(passenger);
    }

    fn get_request(&mut self) -> Option<Passenger> {
        self.requests.pop()
    }
}

struct Elevator {
    id: usize,
    current_level: usize,
    passengers: Vec<Passenger>,
    direction: Option<Direction>,
    door_state: DoorState,
    max_capacity: usize,
}

impl Elevator {
    fn new(id: usize) -> Self {
        Self {
            id,
            current_level: 0,
            passengers: Vec::new(),
            direction: Some(Direction::Up),
            door_state: DoorState::Closed,
            max_capacity: 2,
        }
    }

    fn close_door(&mut self) {
        self.door_state = DoorState::Closing;
        thread::sleep(Duration::from_millis(100));
        self.door_state = DoorState::Closed;
    }

    fn open_door(&mut self) {
        self.door_state = DoorState::Opening;
        thread::sleep(Duration::from_millis(100));
        self.door_state = DoorState::Open
    }

    fn move_and_handle_passengers(
        &mut self,
        levels: &Arc<Vec<Mutex<LevelQueue>>>,
        pending_requests: &Arc<Mutex<PendingRequestQueue>>,
    ) {
        loop {
            // Drop off passengers
            self.open_door();

            self.passengers.retain_mut(|passenger: &mut Passenger| {
                if passenger.destination == self.current_level {
                    println!(
                        "Elevator {}: Passenger {} dropped off at level {}",
                        self.id, passenger.id, self.current_level
                    );
                    false
                } else {
                    passenger.set_leaving_elevator(true);
                    passenger.set_in_elevator(false);
                    passenger.set_leaving_elevator(false);
                    true
                }
            });

            // Pick up passengers from the current level's queue
            if let Some(direction) = self.direction {
                let level_queue = &levels[self.current_level];
                let mut queue = level_queue.lock().unwrap();

                println!(
                    "Elevator {}: Current Level {}. Currently {} passengers: {:?}\n   Level {} Queue: Up: {:?}, Down: {:?}",
                    self.id, self.current_level, self.passengers.len(), self.passengers, self.current_level, queue.up_queue, queue.down_queue
                );

                while self.passengers.len() < self.max_capacity {
                    if let Some(mut passenger) = queue.get_passenger(direction) {
                        passenger.set_idle_on_level(false);
                        passenger.set_entering_elevator(true);
                        passenger.set_in_elevator(true);
                        passenger.set_entering_elevator(false);

                        println!(
                            "Elevator {}: Passenger {} picked up at level {} going {:?} to level {}",
                            self.id, passenger.id, self.current_level, passenger.direction, passenger.destination
                        );
                        self.passengers.push(passenger);
                    } else {
                        break;
                    }
                    if self.passengers.len() == self.max_capacity {
                        println!(
                            "Elevator {} is full. Currently {} passengers: {:?}",
                            self.id,
                            self.passengers.len(),
                            self.passengers
                        );
                    }
                }
                self.close_door();
                println!(
                    "Elevator {} @ level {}. Currently {} passengers: {:?}",
                    self.id,
                    self.current_level,
                    self.passengers.len(),
                    self.passengers
                );
            }

            // If no passengers onboard, check the pending request queue
            if self.passengers.is_empty() {
                if let Some(passenger) = pending_requests.lock().unwrap().get_request() {
                    // Move elevator to the requested level
                    println!(
                        "Elevator {}: Picking up pending request of Passenger {}. Heading to level {}...",
                        self.id, passenger.id, passenger.current_level
                    );

                    // Move the elevator to the level the passenger is waiting at
                    self.move_to_level(&passenger);
                } else {
                    println!("Elevator {}: No requests, idle.", self.id);
                    // Simulate wait time
                    thread::sleep(Duration::from_millis(1000));
                }
            } else {
                // Move the elevator if it's not idle
                self.move_elevator();
            }

            println!(
                "Elevator {} @ level {}. Currently {} passengers: {:?}",
                self.id,
                self.current_level,
                self.passengers.len(),
                self.passengers
            );
        }
    }

    // go straight to the prending request
    fn move_to_level(&mut self, passenger: &Passenger) {
        let target_level = passenger.current_level;

        // Move the elevator towards the target level
        while self.current_level != target_level {
            if self.current_level < target_level {
                self.direction = Some(Direction::Up);
                self.move_elevator();
            } else {
                self.direction = Some(Direction::Down);
                self.move_elevator();
            }
            // Simulate travel time
            thread::sleep(Duration::from_millis(1000));
            println!("Elevator {} moved to level {}", self.id, self.current_level);
        }
        self.direction = Some(passenger.direction);

        println!(
            "Elevator {}: Arrived at level {} to pick up Passenger {}",
            self.id, self.current_level, passenger.id,
        );
    }

    fn move_elevator(&mut self) {
        // Simulate travel time
        thread::sleep(Duration::from_millis(1000));

        // If there's only one passenger, move towards their destination
        if self.passengers.len() == 1 {
            let passenger = &self.passengers[0];
            self.direction = Some(passenger.direction); // Set direction based on the passenger
        }

        match self.direction {
            Some(Direction::Up) => {
                if self.current_level + 1 < LEVELS {
                    self.current_level += 1;
                } else {
                    self.direction = Some(Direction::Down);
                    self.current_level -= 1;
                }
            }
            Some(Direction::Down) => {
                if self.current_level > 0 {
                    self.current_level -= 1;
                } else {
                    self.direction = Some(Direction::Up);
                    self.current_level += 1;
                }
            }
            None => {}
        }
        // Update the passenger levels after the elevator moves
        self.update_passenger_levels();
    }

    // Method to update passengers' current level to match the elevator's current level
    fn update_passenger_levels(&mut self) {
        for passenger in self.passengers.iter_mut() {
            passenger.update_current_level(self.current_level);
        }
    }
}

fn spawn_passengers(
    levels: Arc<Vec<Mutex<LevelQueue>>>,
    pending_requests: Arc<Mutex<PendingRequestQueue>>,
    passenger_count: usize,
) {
    thread::spawn(move || {
        let mut rng = rand::thread_rng();
        for id in 0..passenger_count {
            let level = rng.gen_range(0..LEVELS);
            let direction = if level == 0 {
                Direction::Up
            } else if level == LEVELS - 1 {
                Direction::Down
            } else if rng.gen_bool(0.5) {
                Direction::Up
            } else {
                Direction::Down
            };

            let destination = match direction {
                Direction::Up => rng.gen_range(level + 1..LEVELS),
                Direction::Down => rng.gen_range(0..level),
            };

            let passenger = Passenger {
                id,
                direction,
                destination,
                idle_on_level: true,
                entering_elevator: false,
                in_elevator: false,
                leaving_elevator: false,
                current_level: level,
            };

            thread::sleep(Duration::from_millis(rng.gen_range(100..1000)));
            levels[level]
                .lock()
                .unwrap()
                .add_passenger(passenger.clone()); // Clone here

            println!(
                "Passenger {} spawned at level {} going {:?} to level {}",
                id, level, direction, destination
            );

            // Add passenger to pending requests
            pending_requests
                .lock()
                .unwrap()
                .add_request(passenger.clone()); // Clone here
        }
    });
}

fn main() {
    let levels = Arc::new(
        (0..LEVELS)
            .map(|_| Mutex::new(LevelQueue::new()))
            .collect::<Vec<_>>(),
    );
    let pending_requests = Arc::new(Mutex::new(PendingRequestQueue::new()));

    let passenger_count = 30;
    spawn_passengers(
        Arc::clone(&levels),
        Arc::clone(&pending_requests),
        passenger_count,
    );

    let mut threads: Vec<thread::JoinHandle<()>> = vec![];

    for i in 0..ELEVATORS {
        let levels = Arc::clone(&levels);
        let pending_requests = Arc::clone(&pending_requests);
        threads.push(thread::spawn(move || {
            let mut elevator = Elevator::new(i);
            elevator.move_and_handle_passengers(&levels, &pending_requests);
        }));
    }

    for thread in threads {
        thread.join().unwrap();
    }
}
