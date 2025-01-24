pub mod passenger {
    use crate::Direction;
    use std::thread;
    use std::time::Duration;

    #[derive(Debug, Clone)]
    pub enum PassengerState {
        Idle,
        InElevator,
        EnteringElevator,
        ExitingElevator,
    }

    #[derive(Debug, Clone)]
    pub struct Passenger {
        id: usize,
        direction: Direction,
        destination: usize,
        passenger_state: PassengerState,
        current_level: usize,
    }

    impl Passenger {
        pub fn new(
            id: usize,
            direction: Direction,
            destination: usize,
            current_level: usize,
        ) -> Self {
            Self {
                id,
                direction,
                destination,
                passenger_state: PassengerState::Idle,
                current_level,
            }
        }

        pub fn update_current_level(&mut self, new_level: usize) {
            self.current_level = new_level;
        }

        pub fn enter_elevator(&mut self) {
            self.passenger_state = PassengerState::EnteringElevator;
            thread::sleep(Duration::from_micros(100));
            self.passenger_state = PassengerState::InElevator;
        }

        pub fn wait_for_elevator(&mut self) {
            self.passenger_state = PassengerState::Idle;
        }

        pub fn exit_elevator(&mut self) {
            self.passenger_state = PassengerState::ExitingElevator;
            thread::sleep(Duration::from_micros(100));
            // When not in elevator just be idle on floor
            self.passenger_state = PassengerState::Idle;
        }

        pub fn get_direction(&self) -> &Direction {
            &self.direction
        }

        pub fn get_current_level(&self) -> &usize {
            &self.current_level
        }

        pub fn get_destination(&self) -> &usize {
            &self.destination
        }

        pub fn get_id(&self) -> &usize {
            &self.id
        }
    }
}
