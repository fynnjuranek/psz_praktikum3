use crate::queues::{LevelQueue, PendingRequestQueue};
use crate::Passenger;
use crate::LEVELS;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Down,
}

enum DoorState {
    Closed,
    Closing,
    Open,
    Opening,
}

pub struct Elevator {
    id: usize,
    current_level: usize,
    passengers: Vec<Passenger>,
    direction: Option<Direction>,
    door_state: DoorState,
    max_capacity: usize,
}

impl Elevator {
    pub fn new(id: usize) -> Self {
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

    pub fn move_and_handle_passengers(
        &mut self,
        levels: &Arc<Vec<Mutex<LevelQueue>>>,
        pending_requests: &Arc<Mutex<PendingRequestQueue>>,
    ) {
        loop {
            // Drop off passengers
            self.open_door();

            self.passengers.retain_mut(|passenger: &mut Passenger| {
                if passenger.get_destination() == &self.current_level {
                    println!(
                        "Elevator {}: Passenger {} dropped off at level {}",
                        self.id,
                        passenger.get_id(),
                        self.current_level
                    );
                    false
                } else {
                    passenger.exit_elevator();
                    true
                }
            });

            // Pick up passengers from the current level's queue
            if let Some(direction) = &self.direction {
                let level_queue = &levels[self.current_level];
                let mut queue = level_queue.lock().unwrap();

                println!(
                        "Elevator {}: Current Level {}. Currently {} passengers: {:?}\n   Level {} Queue: Up: {:?}, Down: {:?}",
                        self.id, self.current_level, self.passengers.len(), self.passengers, self.current_level, queue.get_up_queue(), queue.get_down_queue()
                    );

                while self.passengers.len() < self.max_capacity {
                    if let Some(mut passenger) = queue.get_passenger(*direction) {
                        passenger.enter_elevator();
                        println!(
                                "Elevator {}: Passenger {} picked up at level {} going {:?} to level {}",
                                self.id, passenger.get_id(), self.current_level, passenger.get_direction(), passenger.get_destination()
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
                            self.id, passenger.get_id(), passenger.get_current_level()                        );

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
        let target_level = passenger.get_current_level();

        // Move the elevator towards the target level
        while &self.current_level != target_level {
            if &self.current_level < target_level {
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
        self.direction = Some(*passenger.get_direction());

        println!(
            "Elevator {}: Arrived at level {} to pick up Passenger {}",
            self.id,
            self.current_level,
            passenger.get_id(),
        );
    }

    fn move_elevator(&mut self) {
        // Simulate travel time
        thread::sleep(Duration::from_millis(1000));

        // If there's only one passenger, move towards their destination
        if self.passengers.len() == 1 {
            let passenger = &self.passengers[0];
            self.direction = Some(*passenger.get_direction()); // Set direction based on the passenger
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
