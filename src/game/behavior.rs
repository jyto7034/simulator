use crate::{enums::SpellType, exception::exception::Exception, unit::{Entity, Player}, deck::Card};

#[derive(Clone, Debug)]
pub enum Behavior {
    /// caster 는 시전자 opponent 는 피시전자입니다.

    /// 게임을 종료합니다.
    /// fn end_game( player ) -> Result
    /// 매개변수로 player 객체를 받으며, 해당 플레이어가 게임에서 승리합니다.
    EndGame,

    /// caster 가 defender 에게 spell 을 시전합니다.
    /// fn casting_spell( caster, opponent, spell_type )
    CastingSpell(SpellType),

    /// caster 가 현재 전개되고 있는 spell 을 취소합니다.
    /// fn Interrupt_spell( caster, opponent )
    InterruptSpell,

    /// caster 가 전개한 Spell 카드의 효과를 opponent 에게 적용합니다.
    /// fn be_under_spell( caster, opponent )
    BeUnderSpell,

    /// attacker 가 defender 에게 damage 만큼의 피해를 입힙니다.
    /// fn give_damage_to( attacker, defender, damage )
    /// 처음 두 매개 변수는 player 객체이며, damage 는 정수형 변수입니다.
    GiveDamageTo,

    /// player 가 Deck 에서 card_type 에 해당하는 카드를 card_num 만큼 꺼냅니다.
    /// fn draw_card_from_deck( player, card_type, card_num )
    DrawCardFromDeck,

    /// player 가 Deck 에 card 를 넣습니다.
    /// fn insert_card_to_deck( player, card )
    InsertCardToDeck,

    /// player 가 card 를 덱에서 제거합니다.
    /// fn insert_card_to_deck( player, card )
    DestroyCardFromDeck,

    /// player 가 자신의 손패에 card 를 추가합니다.
    /// fn insert_card_to_deck( player, card )
    AddCardToHand,

    /// player 가 자신의 덱에 card 를 추가합니다.
    /// fn insert_card_to_deck( player, card )
    AddCardToDeck,

    /// player 가 자신의 필드에 card 를 추가합니다.
    /// fn insert_card_to_deck( player, card )
    AddCardToField,

    /// player 가 자신의 필드에 카드를 전개합니다.
    /// fn play_card_to_field( player, card )
    PlayCardToField,

    /// 해당 Player 가 다른 이벤트가 발생하는 것을 기다립니다.
    /// fn listen_other_event( Player, target_behavior, result_behavior )
    /// 조건이 충족되면, 정해진 이벤트를 발생시킵니다.
    ListenOtherEvent,

    /// 초기화용
    None,
}

pub fn execution(behavior_type: &Behavior) -> Result<Exception, Exception> {
    match behavior_type {
        Behavior::EndGame => todo!(),
        Behavior::CastingSpell(_) => todo!(),
        Behavior::InterruptSpell => todo!(),
        Behavior::BeUnderSpell => todo!(),
        Behavior::GiveDamageTo => todo!(),
        Behavior::DrawCardFromDeck => todo!(),
        Behavior::InsertCardToDeck => todo!(),
        Behavior::DestroyCardFromDeck => todo!(),
        Behavior::AddCardToHand => todo!(),
        Behavior::AddCardToDeck => todo!(),
        Behavior::AddCardToField => todo!(),
        Behavior::PlayCardToField => todo!(),
        Behavior::ListenOtherEvent => todo!(),
        Behavior::None => todo!(),
    }
}

// 감시를 하는 방법에는 여러가지가 있을테지만, 당장 떠오르는건, 
fn listen_other_event(player: &Player, target_behavior: &Behavior, result_behavior: &Behavior) {
    
}

// 물류아저씨한테 가방 하나 더 달라고 하기.