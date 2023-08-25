use crate::enums::constant::*;
use crate::task::task::Task;

pub struct Procedure {
    pub task_queue: TaskQueue,
}

impl Procedure {
    pub fn add_task(&mut self, task: Task) {
        self.task_queue.push(task);
    }

    pub fn find_task() {}

    pub fn remove_task() {}
}
