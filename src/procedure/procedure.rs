use crate::{exception::exception::Exception, game::game::Game};

use super::{behavior::{check_trigger, execution}, task::Task};

pub struct Procedure {
    pub tasks: Vec<Task>,
    pub trigger_tasks: Vec<Task>
}

impl Procedure{
    pub fn generate_id(&self) -> i32{
        (self.tasks.len() + 1) as i32
    }

    pub fn add_task(&mut self, mut task: Task){
        task.set_task_id(self.generate_id());
        self.tasks.push(task);
    }

    /// 객체 복사 후 Procedure 수행 후 리턴
    pub fn simulate(&mut self, game: Game) -> Result<(), Exception>{
        let mut _game = game.clone();
        self.run(&mut _game)?;
        Ok(())
    }

    pub fn run(&mut self, game: &mut Game) -> Result<(), Exception>{
        for task in &self.tasks.clone(){
            let card = task.get_task().get_data_as_card();
            check_trigger(card.clone(), self)?;
        }
        for task in &self.tasks{
            let card = task.get_task().get_data_as_card();
            execution(card.clone(), game)?;
        }
        Ok(())
    }
}