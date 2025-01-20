use crate::{
    card::Card,
    enums::{CardAttribute, InsertType, PlayerType, SpellType, TargetCard, ZoneType},
    exception::Exception,
    game::Game,
    server::{respone::RESPONE_QUEUE, schema::Respones},
};

use super::Procedure;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Behavior {
    EndGame,

    CastingSpell(PlayerType, PlayerType, SpellType),

    InterruptSpell,

    BeUnderSpell,

    GiveDamageTo,

    DrawCardFromDeck,

    RemoveCardFromZone(ZoneType, PlayerType, Card),

    AddCardToZone(ZoneType, PlayerType, Card, i32),

    ChoiceCard(i32),

    ModifyCardSpec(CardAttribute, TargetCard),

    None,
}

/// 카드에 담긴 behavior 을 처리함.
pub fn execution(card: Card, game: &mut Game) -> Result<(), Exception> {
    for behavior_type in card.get_behavior_table() {
        match behavior_type {
            Behavior::EndGame => todo!(),
            Behavior::InterruptSpell => todo!(),
            Behavior::BeUnderSpell => todo!(),
            Behavior::GiveDamageTo => todo!(),
            Behavior::DrawCardFromDeck => todo!(),
            Behavior::ChoiceCard(_) => todo!(),
            Behavior::ModifyCardSpec(data, target) => {
                modify_card_spec(&card, game, data, &target);
            }
            Behavior::None => todo!(),
            Behavior::CastingSpell(_, _, _) => todo!(),
            Behavior::AddCardToZone(zone_type, player_type, card, slot_id) => {
                add_card_to_zone(game, zone_type, player_type, card, slot_id);
            }
            Behavior::RemoveCardFromZone(zone_type, player_type, card) => todo!(),
        }
    }
    Ok(())
}

use std::collections::HashSet;

fn has_intersection<T: Eq + std::hash::Hash>(vec1: &Vec<T>, vec2: &Vec<T>) -> bool {
    let set1: HashSet<_> = vec1.iter().collect();
    vec2.iter().any(|item| set1.contains(item))
}

/// 현재 처리 중인 card 에 trigger behavior 가 있는지 확인.
pub fn check_trigger(current_task_card: Card, app: &mut Procedure) -> Result<(), Exception> {
    for trigger in app.trigger_tasks.iter() {
        let trigger_beheviors = trigger.get_task().get_data_as_behavior();

        // trigger task 가 존재할 때,
        if has_intersection(&trigger_beheviors, &current_task_card.get_behavior_table()) {
            // tasks 에서 card 의 위치를 찾은 뒤, 해당 card 앞에 trigger card 를 삽입함.

            let pos = app
                .tasks
                .iter()
                .position(|it| it.get_task().get_data_as_card() == current_task_card);
            if let Some(pos) = pos {
                app.tasks.insert(pos, trigger.clone());
            } else {
                // TODO: 적당한 Exception 만들어서 리턴해야함.
                return Err(Exception::CardsNotFound);
            }

            // 처리 결과를 ResponeQueue 에 추가.
            RESPONE_QUEUE.lock().unwrap().push(Respones::AttackTo);
        }
    }
    Ok(())
}

// ModifyCardSpec 처리 함수
fn modify_card_spec(card: &Card, game: &mut Game, data: &CardAttribute, target: &TargetCard) {
    let _player = game.get_player(card.get_player_type());
}

// Behavior::RemoveCardFromDeck => todo!(),
fn remove_card_from_zone(
    game: &mut Game,
    zone_type: &ZoneType,
    player_type: &PlayerType,
    card: &Card,
) {
    let player = game.get_player(player_type.clone());
    player
        .get_mut()
        .get_zone(zone_type.clone())
        .remove_card(card.get_uuid().clone());
}

// AddCardToZone
fn add_card_to_zone(
    game: &mut Game,
    zone_type: &ZoneType,
    player_type: &PlayerType,
    card: &Card,
    slot_id: &i32,
) {
    let player = game.get_player(player_type.clone());
    player
        .get_mut()
        .get_zone(zone_type.clone())
        .add_card(card.clone(), InsertType::Slot(slot_id.clone()));
}
