use crate::enums::TaskPriority;
use crate::exception::exception::Exception;
use crate::game::Behavior;
use crate::utils::utils;

#[derive(Clone)]
pub struct Task {
    task_uuid: String,
    behavior: Behavior,
    priority: TaskPriority,
}

impl Task {
    pub fn dummy() -> Task {
        Task {
            task_uuid: "0".to_string(),
            behavior: Behavior::None,
            priority: TaskPriority::None,
        }
    }

    pub fn new(behavior_type: Behavior, priority: TaskPriority) -> Result<Task, Exception> {
        let uuid = match utils::generate_uuid() {
            Ok(ans) => ans,
            Err(_) => "".to_string(),
        };
        Ok(Task {
            task_uuid: uuid,
            behavior: behavior_type,
            priority,
        })
    }

    pub fn get_behavior_type(&self) -> &Behavior {
        &self.behavior
    }

    pub fn set_behavior_type(&mut self, behavior_type: Behavior) {
        self.behavior = behavior_type;
    }

    pub fn get_priority_type(&self) -> &TaskPriority {
        &self.priority
    }

    pub fn set_priority_type(&mut self, priority_type: TaskPriority) {
        self.priority = priority_type;
    }

    pub fn get_task_uuid(&self) -> &String {
        &self.task_uuid
    }
}
