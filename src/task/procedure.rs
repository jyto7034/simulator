use crate::enums::constant::*;
use crate::exception::exception::Exception;
use crate::game::*;
use crate::task::task::Task;

pub struct Procedure<'a> {
    pub task_queue: TaskQueue,
    pub event_listen_queue: Vec<(Task, (Behavior, Behavior))>,
    game: Option<&'a mut Game<'a>>,
    id: usize,
}

impl<'a> Procedure<'a> {
    pub fn new(game: Option<&'a mut Game<'a>>) -> Procedure<'a> {
        Procedure {
            task_queue: vec![],
            event_listen_queue: vec![],
            game,
            id: 0,
        }
    }

    fn generate_id(&mut self) -> usize {
        self.id += 1;
        self.id
    }

    pub fn add_task(&mut self, task: &Task) {
        let mut task = task.clone();
        task.set_task_id(self.generate_id());
        self.task_queue.push(task);
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
        Ok(())
    }
}
