use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use elevator::{Direction, Elevator};
use passenger::Passenger;
use queues::{LevelQueue, PendingRequestQueue};

use rand::Rng;

mod elevator;
mod passenger;
mod queues;

const LEVELS: usize = 3;
const ELEVATORS: usize = 3;

fn spawn_passengers(
    levels: Arc<Vec<Mutex<LevelQueue>>>,
    pending_requests: Arc<Mutex<PendingRequestQueue>>,
    passenger_count: usize,
) {
    thread::spawn(move || {
        let mut rng = rand::thread_rng();
        for id in 0..=passenger_count {
            let level = rng.gen_range(0..=LEVELS);
            let direction = if level == 0 {
                Direction::Up
            } else if level == LEVELS {
                Direction::Down
            } else if rng.gen_bool(0.5) {
                Direction::Up
            } else {
                Direction::Down
            };

            let destination = match direction {
                Direction::Up => rng.gen_range(level + 1..=LEVELS),
                Direction::Down => rng.gen_range(0..level),
            };

            let mut passenger = Passenger::new(id, direction, destination, level);

            // after intialization of passenger, he waits for elevator on the floor.
            passenger.wait_for_elevator();

            // after intialization of passenger, he waits for elevator on the floor.
            passenger.wait_for_elevator();

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
        (0..=LEVELS)
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

    for i in 0..=ELEVATORS {
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
