use crate::enums::constant::TaskType;
use crate::exception::exception::Exception;
use crate::utils::utils;

pub struct Task {
    task_type: TaskType,
    task_uuid: String,
}

impl Task {
    pub fn new(task_type: TaskType) -> Result<Task, Exception> {
        let uuid = match utils::generate_uuid() {
            Ok(ans) => ans,
            Err(_) => "".to_string(),
        };
        Ok(Task {
            task_type,
            task_uuid: uuid,
        })
    }

    pub fn get_type(&self) -> &TaskType {
        &self.task_type
    }

    pub fn set_type(&mut self, task_type: TaskType) {
        self.task_type = task_type;
    }
}
