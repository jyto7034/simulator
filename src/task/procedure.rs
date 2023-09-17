use crate::enums::constant::*;
use crate::exception::exception::Exception;
use crate::game::*;
use crate::task::task::Task;

pub struct Procedure {
    pub task_queue: TaskQueue,
}

impl Procedure {
    pub fn new() -> Procedure {
        Procedure { task_queue: vec![] }
    }
    pub fn add_task(&mut self, task: &Task) {
        self.task_queue.push(task.clone());
    }

    pub fn find_task_by_uuid(&self, uuid: &UUID) -> Option<&Task> {
        let result: Vec<&Task> = self
            .task_queue
            .iter()
            .filter(|item| item.get_task_uuid() == uuid)
            .collect();

        result.first().copied()
    }

    pub fn find_task_by_ref(&self, task: &Task) -> Option<&Task> {
        self.find_task_by_uuid(task.get_task_uuid())
    }

    pub fn remove_task_by_uuid(&mut self, uuid: &UUID) -> Result<(), Exception> {
        let prev_len = self.task_queue.len();
        self.task_queue.retain(|item| item.get_task_uuid() != uuid);
        if self.task_queue.len() != prev_len {
            Ok(())
        } else {
            Err(Exception::NothingToRemove)
        }
    }

    pub fn remove_task_by_ref(&mut self, task: &Task) -> Result<(), Exception> {
        self.remove_task_by_uuid(task.get_task_uuid())
    }

    /// 후입선출 방식으로 uuid 를 집계하여 리턴한다.
    pub fn get_task_list(&self) -> Vec<&String> {
        let mut res = vec![];
        for item in self.task_queue.iter().rev() {
            res.push(item.get_task_uuid());
        }
        res
    }

    /// queue 에 있는 task 를 처리하는 함수.
    /// 후입선출로 우선순위에 따라 순서대로 처리한다.
    pub fn execuiton(&mut self) -> Result<(), Exception> {
        let cloned_tasks: Vec<Task> = self.task_queue.iter().rev().cloned().collect();
        for task in cloned_tasks {
            match task.get_priority_type() {
                TaskPriority::Immediately => {
                    execution(task.get_behavior_type())?;
                }
                TaskPriority::RoundEnd => {
                    execution(task.get_behavior_type())?;
                }
                TaskPriority::RoundStart => {
                    execution(task.get_behavior_type())?;
                }
                TaskPriority::AttackTurn => {
                    execution(task.get_behavior_type())?;
                }
                TaskPriority::DefenseTurn => {
                    execution(task.get_behavior_type())?;
                }
                TaskPriority::None => {}
            }
            if let Err(err) = self.remove_task_by_uuid(task.get_task_uuid()) {
                if err == Exception::NothingToRemove {
                    todo!();
                }
            }
        }
        Ok(())
    }
}
