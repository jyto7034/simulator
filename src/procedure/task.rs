use std::ops::Deref;

use crate::{enums::TaskType, server::schema::MessageInfo, utils::utils};

/// Trigger Card, 혹은 Trap Card 에 의해서 Task 의 내용이 수정되거나 사라지는 경우 존재함.
/// Tasks 의 Add 함수 호출 시 들어오는 Task 를 수정함.

/// Card, task id 등의 정보를 담는 구조체
#[derive(Clone)]
pub struct Task {
    task: TaskType,
    task_uuid: String,
    id: Option<i32>,
    info: Option<MessageInfo>,
}

impl Task {
    pub fn dummy() -> Task {
        Task {
            task: TaskType::None,
            task_uuid: "".to_string(),
            id: Some(0),
            info: None,
        }
    }

    pub fn new(task_type: TaskType, info: MessageInfo) -> Task {
        let uuid = match utils::generate_uuid() {
            Ok(ans) => ans,
            Err(_) => panic!(),
        };
        Task {
            task: task_type,
            task_uuid: uuid,
            id: None,
            info: Some(info),
        }
    }

    pub fn get_task_uuid(&self) -> &String {
        &self.task_uuid
    }

    pub fn set_task_id(&mut self, id: i32) {
        self.id = Some(id);
    }

    pub fn get_task_id(&self) -> Option<i32> {
        self.id
    }

    pub fn get_task(&self) -> &TaskType {
        &self.task
    }

    pub fn get_info(&self) -> &MessageInfo {
        self.info.as_ref().unwrap()
    }
}
