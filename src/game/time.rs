use crate::enums::TimeType;

pub struct TimeManager {
    time_state: TimeType,
}

impl TimeManager {
    pub fn new() -> TimeManager {
        TimeManager {
            time_state: TimeType::None,
        }
    }

    pub fn set_to_day(&mut self) {
        self.time_state = TimeType::Day;
    }

    pub fn set_to_night(&mut self) {
        self.time_state = TimeType::Night;
    }

    pub fn set_to(&mut self, time_state: TimeType) {
        self.time_state = time_state;
    }

    pub fn get_state(&self) -> &TimeType {
        &self.time_state
    }
}
