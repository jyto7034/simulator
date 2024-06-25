use crate::{app::app::App, enums::TaskType, exception::exception::Exception, procedure::{behavior::Behavior, task::Task}, server::schema::Respones};
use super::schema::MessageInfo;
use std::sync::Mutex;

lazy_static! {
    pub static ref RESPONE_QUEUE: Mutex<Vec<Respones>> = {
        let queue = Mutex::new(vec![]);
        queue
    };
}

pub struct Respone{

}

/// server 가 client 에게 보내는 ResponeMsg
impl Respones{
    // respone GetMulligunCards
    pub fn get_mulligun_cards(app: &mut  App, info: MessageInfo, data: i32) -> Result<(), Exception>{
        let task = Task::new(TaskType::Behavior(Behavior::ChoiceCard(data)), info);
        app.procedure.add_task(task);
        app.procedure.run(&mut app.game)?;
        Ok(())
    }

    pub fn play_card_with_target(app: &mut  App, info: MessageInfo) -> Result<(), Exception>{
        Ok(())
    }
}