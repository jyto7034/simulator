use actix::Addr;

use crate::{card::Card, game::GameActor};

use super::{types::EffectSpeed, Effect};

// Effect 에 우선순위를 부여하는건 장기적으로 좋음.
pub struct Effects {
    // 항상 정렬 상태를 보장해야함.
    prioritized_effects: Vec<EffectTiming>,
}

impl Effects {
    pub fn new() -> Self {
        Self {
            prioritized_effects: vec![],
        }
    }

    pub fn add_effect(&mut self, effect: EffectTiming) {
        self.prioritized_effects.push(effect);
        self.prioritized_effects.sort_by_key(|e| e.get_priority());
    }

    pub fn get_effects(&self) -> Vec<&Box<dyn Effect>> {
        self.prioritized_effects
            .iter()
            .map(|e| e.get_effect())
            .collect::<Vec<_>>()
    }

    pub fn get_effects_mut(&mut self) -> Vec<&mut Box<dyn Effect>> {
        self.prioritized_effects
            .iter_mut()
            .map(|e| e.get_effect_mut())
            .collect::<Vec<_>>()
    }

    // 특정 조건을 만족하는 효과만 필터링 (조건을 클로저로 전달)
    pub fn filter_effects<F>(&self, filter: F) -> Vec<&EffectTiming>
    where
        F: Fn(&EffectTiming) -> bool,
    {
        self.prioritized_effects
            .iter()
            .filter(|e| filter(e))
            .collect()
    }

    // 체인 상태에서 추가 가능한 효과만 필터링
    pub fn get_chainable_effects(&self, current_chain_level: EffectSpeed) -> Vec<&EffectTiming> {
        self.prioritized_effects
            .iter()
            .filter(|e| {
                let timing = e.get_effect().get_speed();
                if timing >= current_chain_level {
                    return true;
                } else {
                    return false;
                }
            })
            .collect()
    }

    // 발동 가능한 효과만 필터링
    pub fn get_activatable_effects(
        &self,
        game: Addr<GameActor>,
        source: &Card,
    ) -> Vec<&EffectTiming> {
        todo!()
        // self.prioritized_effects
        //     .iter()
        //     .filter(|e| e.get_effect().can_activate(game, source))
        //     .collect()
    }
}

#[derive(Clone)]
pub struct EffectTiming {
    priority: u8, // 낮을수록 높은 우선순위
    speed: EffectSpeed,
    is_used: bool, // 효과가 사용되었는지 여부
    effect: Box<dyn Effect>,
}

impl EffectTiming {
    pub fn new(priority: u8, speed: EffectSpeed, effect: Box<dyn Effect>) -> Self {
        Self {
            priority,
            effect,
            speed,
            is_used: false,
        }
    }

    pub fn get_priority(&self) -> u8 {
        self.priority
    }

    pub fn get_speed(&self) -> EffectSpeed {
        self.speed
    }

    pub fn get_effect(&self) -> &Box<dyn Effect> {
        &self.effect
    }

    pub fn get_effect_mut(&mut self) -> &mut Box<dyn Effect> {
        &mut self.effect
    }
}
