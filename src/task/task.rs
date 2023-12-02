use crate::deck::Card;
use crate::enums::constant::*;
use crate::enums::{PlayerType, TaskPriority};
use crate::exception::exception::Exception;
use crate::game::Behavior;
use crate::utils::utils;

/// Card 구조체엔 run 이라는 함수가 존재하는데, 카드의 효과를 발동할 때,
/// 이 run 함수를 실행시킨다. 실행된 run 함수는 자신 카드의 uuid 를 task 객체로 만들어 procedure 의 task_list 에 추가한다.
/// 또한, procedure 의 execution 함수를 실행시켜, task 를 처리한다.
#[derive(Clone, Debug)]
pub struct Task {
    card: Card,
    task_uuid: String,
    priority: TaskPriority,
    id: Option<usize>,
}

impl Task {
    pub fn dummy() -> Task {
        Task {
            card: Card::dummy(),
            task_uuid: "".to_string(),
            priority: TaskPriority::None,
            id: Some(0),
        }
    }

    pub fn new(
        card: Card,
        priority: TaskPriority,
    ) -> Result<Task, Exception> {
        let uuid = match utils::generate_uuid() {
            Ok(ans) => ans,
            Err(_) => "".to_string(),
        };
        Ok(Task {
            card,
            task_uuid: uuid,
            priority,
            id: Some(0 as usize),
        })
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

    pub fn set_task_id(&mut self, id: usize) {
        self.id = Some(id);
    }

    pub fn get_task_id(&self) -> Option<usize> {
        self.id
    }
}
