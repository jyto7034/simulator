use crate::{
    card::{types::PlayerType, Card},
    effect::{
        effects::EffectTiming,
        types::{EffectResult, EffectSpeed, HandlerType},
    },
    exception::GameError,
    server::input_handler::{InputAnswer, InputRequest},
};
use std::collections::HashSet;
use tracing::info;
use uuid::Uuid;

use super::{game_step::PlayCardResult, Game};

// 체인 처리 단계
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChainPhase {
    Idle,      // 대기 중 (새로운 체인 생서 가능)
    Building,  // 체인 구성 중 (효과 추가 가능)
    Resolving, // 체인 해결 중 (효과 실행 단계)
    Waiting,   // 사용자 입력 대기 중
    Completed, // 체인 처리 완료
}

impl Default for ChainPhase {
    fn default() -> Self {
        todo!()
    }
}

// 체인 아이템 구조체 (효과와 소스 카드 연결)
#[derive(Clone)]
pub struct ChainLink {
    effect: EffectTiming,
    source_card: Card,
}

#[derive(Clone, Default)]
pub struct Chain {
    // 체인 큐 (LIFO 방식으로 처리)
    links: Vec<ChainLink>,

    // 현재 체인 처리 단계
    current_phase: ChainPhase,

    // 이미 처리된 효과 ID
    processed_effect_ids: HashSet<Uuid>,
}

impl Chain {
    pub fn new() -> Self {
        Self {
            links: Vec::new(),
            current_phase: ChainPhase::Completed, // 초기 상태는 완료
            processed_effect_ids: HashSet::new(),
        }
    }

    /// 카드의 모든 효과를 처리합니다
    pub async fn process_card_effects(
        &mut self,
        game: &mut Game,
        player_type: PlayerType,
        card: Card,
    ) -> Result<PlayCardResult, GameError> {
        info!(
            "카드 효과 처리: player={:?}, card={:?}",
            player_type,
            card.get_uuid()
        );

        // 카드는 효과를 여러개 가질 수 있음.
        // 1. 카드의 효과는 각 게임 상태에 따라 발동 여부가 결정됨
        // 2. 개중에는 동시에 발동할 수 있는 효과도 있음 ( 이런 경우, 사용자가 무슨 효과를 발동할 지 선택함. )
        // 효과 중 이미 처리된 효과는 제외 해야함.
        // 이 때 이미 사용되었다고 해서 무조건 GameError 을 발생시키면 안됨.

        // 현재 입력된 카드의 효과를 가져옵니다.
        let effects = card.get_prioritized_effect().clone();

        // 발동 가능한 효과를 필터링 합니다.
        let activable_effects = effects
            .iter()
            .filter(|e| e.get_effect().can_activate(game, &card))
            .collect::<Vec<_>>();

        if activable_effects.is_empty() {
            // 발동 가능한 효과가 없으면 종료합니다.
            return Err(GameError::NoActivatableEffect);
        }

        // 발동 가능한 효과가 두 개 이상일 경우, 사용자로부터 선택을 받아야 합니다.
        if activable_effects.len() >= 2 {
            let rx = game
                .get_input_waiter_mut()
                .wait_for_input(InputRequest::SelectEffect {
                    source_card: card.get_uuid(),
                    potential_effects: activable_effects
                        .iter()
                        .map(|e| e.get_effect().get_id())
                        .collect(),
                })
                .await?;
            return Ok(PlayCardResult::NeedInput(
                rx,
                HandlerType::General(Box::new(
                    move |game: &mut Game,
                          source: &Card,
                          input: InputAnswer|
                          -> Result<EffectResult, GameError> {
                        Ok(EffectResult::Completed)
                    },
                )),
            ));
            // TODO: 무슨 효과를 발동할 지 선택 받아야함.
        }

        // 최종적으로 선택된 효과 하나를 가져옵니다.
        let activable_effect = *activable_effects.last().unwrap();

        match self.current_phase {
            // 체인이 대기 상태일 때
            ChainPhase::Idle => {
                // 바로 체인에 추가합니다.
                self.add_link_to_chain(activable_effect.clone(), card.clone());

                self.current_phase = ChainPhase::Building; // 체인 상태를 변경합니다.
            }
            // 체인이 구성 중일 때.
            ChainPhase::Building => {
                // 이전 체인 스피드를 확인하여 체인을 이어갈 수 있는지 확인합니다.
                // 체인 스피드가 같거나 빠른 경우에만 체인에 추가합니다.
                let last_effect = self.links.last().unwrap().effect.clone();

                // 현재 처리중인 효과의 이펙트 스피드가 체인보다 느릴 경우,
                if last_effect
                    .get_speed()
                    .can_it_chain(activable_effect.get_speed())
                    == false
                {
                    return Err(GameError::InvalidChainSpeed);
                }

                self.add_link_to_chain(activable_effect.clone(), card.clone());
            }
            _ => {
                // 체인 처리 중일 때는 추가할 수 없습니다.
                return Err(GameError::InvalidChainPhase);
            }
        }

        // 카드의 효과 중 발동될 수 있는 효과를 필터링 합니다.
        // 이 때 확인 사항은
        // 1. 효과가 발동될 수 있는 게임 상태인가?
        // 2. 아직 사용되지 않은 효과인가?

        todo!()
    }

    pub fn add_link_to_chain(&mut self, effect: EffectTiming, card: Card) {
        // 체인에 링크를 추가합니다.
        self.links.push(ChainLink {
            effect,
            source_card: card,
        });
    }

    pub fn can_add_to_chain(&self, effect_speed: EffectSpeed) -> bool {
        match self.current_phase {
            ChainPhase::Idle => true,
            ChainPhase::Building => {
                if let Some(last_link) = self.links.last() {
                    // 마지막 링크의 스피드와 비교하여 체인 가능 여부를 판단합니다.
                    return last_link.effect.get_speed().can_it_chain(effect_speed);
                } else {
                    // 체인이 비어있다면 추가 가능
                    return true;
                }
            }
            _ => false,
        }
    }
}

// 체인 해결 결과
pub enum ChainResolutionResult {
    Completed,                       // 모든 효과 처리 완료
    WaitingForInput(PlayCardResult), // 사용자 입력 대기
}
