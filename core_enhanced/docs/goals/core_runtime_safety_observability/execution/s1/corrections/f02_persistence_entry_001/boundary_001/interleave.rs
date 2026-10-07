use game_core::application::*;
fn check(a: &mut CoreApplication, c: ApplicationCommand) { let p = a.prepare_non_battle_command(uuid::Uuid::nil(), c.clone(), ApplicationExecutionContext::default()).unwrap(); let _ = a.execute(uuid::Uuid::nil(), c, ApplicationExecutionContext::default()); p.commit(); }
fn main() {}
