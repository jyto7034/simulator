use crate::deck::Card;
use crate::enums::{TaskPriority, PlayerType};
use crate::exception::exception::Exception;
use crate::game::Behavior;
use crate::utils::utils;

#[derive(Clone)]
pub struct Task {
    player_type: PlayerType,
    task_uuid: String,
    card: Card,
    priority: TaskPriority,
    id: Option<usize>,
}

impl Task {
    pub fn dummy() -> Task {
        Task {
            player_type: PlayerType::None,
            task_uuid: "0".to_string(),
            card: Card::dummy(),
            priority: TaskPriority::None,
            id: Some(0 as usize),
        }
    }

    pub fn new(player_type: PlayerType, card: &Card, behavior: Behavior, priority: TaskPriority) -> Result<Task, Exception> {
        let uuid = match utils::generate_uuid() {
            Ok(ans) => ans,
            Err(_) => "".to_string(),
        };
        Ok(Task {
            player_type,
            task_uuid: uuid,
            card: card.clone(),
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
