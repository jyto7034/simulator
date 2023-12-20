use std::cell::RefCell;
use std::rc::Weak;

use crate::enums::constant::*;
use crate::exception::exception::Exception;
use crate::game::*;
use crate::task::task::Task;

#[derive(Debug)]
pub struct Procedure {
    pub task_queue: TaskQueue,
    game: Option<Weak<RefCell<Game>>>,
    id: usize,
}

impl Procedure {
    pub fn new(game: Option<Weak<RefCell<Game>>>) -> Procedure {
        Procedure {
            task_queue: vec![],
            game,
            id: 0,
        }
    }

    fn generate_id(&mut self) -> usize {
        self.id += 1;
        self.id
    }

    pub fn add_task(&mut self, task: Task) {
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
    pub fn get_task_list(&self) -> Vec<String> {
        self.task_queue
            .iter()
            .map(|task| task.get_task_uuid().clone())
            .collect()
    }

    /// 유닛 카드, 스펠 카드, 효과 카드가 존재한다.
    /// 
    pub fn process_task(&mut self) -> Result<(), Exception> {
        let _task_queue = self.task_queue.clone();


        // 먼저 해당 listen event 를 발동시킨 card 의 정보를 담고 있는 task 를 가져온다.
        // 그리고 to_find 이라는 변수로 무슨 행동을 감시할 것인지 설정하고 만약 detected 되면
        // 카드의 run 함수를 실행함.

        // player 의 HandZone 에서 Draw 함수를 통해 카드를 전개합니다.
        // 

        // 그런 뒤 남은 task 들을 처리한다.
        Ok(())
    }
}
