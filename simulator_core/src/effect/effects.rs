use actix::Addr;

use crate::{card::Card, game::GameActor};

use super::{types::EffectSpeed, Effect};

/// `Effects` 구조체는 게임 내에서 발생하는 효과들을 관리합니다.
/// 효과는 우선순위에 따라 정렬되어 저장되며, 다양한 필터링 및 접근 방법을 제공합니다.
///
/// # Examples
///
/// ```
/// use simulator_core::effect::{Effects, EffectTiming, Effect};
/// use simulator_core::effect::types::EffectSpeed;
///
/// // Example Effect (replace with a real implementation)
/// struct ExampleEffect {}
/// impl Effect for ExampleEffect {
///     fn apply(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> Result<(), String> {
///         Ok(())
///     }
///     fn can_activate(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> bool {
///         true
///     }
///     fn get_speed(&self) -> EffectSpeed {
///         EffectSpeed::Normal
///     }
/// }
///
/// let mut effects = Effects::new();
/// let effect = Box::new(ExampleEffect {});
/// let effect_timing = EffectTiming::new(1, EffectSpeed::Fast, effect);
/// effects.add_effect(effect_timing);
///
/// assert_eq!(effects.get_effects().len(), 1);
/// ```
// Effect 에 우선순위를 부여하는건 장기적으로 좋음.
pub struct Effects {
    // 항상 정렬 상태를 보장해야함.
    prioritized_effects: Vec<EffectTiming>,
}

impl Effects {
    /// `Effects` 구조체의 새로운 인스턴스를 생성합니다.
    ///
    /// # Returns
    ///
    /// - `Effects`: 비어있는 효과 목록을 가진 새로운 `Effects` 인스턴스.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::effect::Effects;
    ///
    /// let effects = Effects::new();
    /// assert_eq!(effects.get_effects().len(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            prioritized_effects: vec![],
        }
    }

    /// 효과를 목록에 추가하고, 우선순위에 따라 정렬합니다.
    ///
    /// # Arguments
    ///
    /// * `effect` - 추가할 `EffectTiming` 객체.
    ///
    /// # Returns
    ///
    /// 없음.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::effect::{Effects, EffectTiming, Effect};
    /// use simulator_core::effect::types::EffectSpeed;
    ///
    /// // Example Effect (replace with a real implementation)
    /// struct ExampleEffect {}
    /// impl Effect for ExampleEffect {
    ///     fn apply(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> Result<(), String> {
    ///         Ok(())
    ///     }
    ///     fn can_activate(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> bool {
    ///         true
    ///     }
    ///     fn get_speed(&self) -> EffectSpeed {
    ///         EffectSpeed::Normal
    ///     }
    /// }
    ///
    /// let mut effects = Effects::new();
    /// let effect = Box::new(ExampleEffect {});
    /// let effect_timing = EffectTiming::new(1, EffectSpeed::Fast, effect);
    /// effects.add_effect(effect_timing);
    ///
    /// assert_eq!(effects.get_effects().len(), 1);
    /// ```
    pub fn add_effect(&mut self, effect: EffectTiming) {
        self.prioritized_effects.push(effect);
        self.prioritized_effects.sort_by_key(|e| e.get_priority());
    }

    /// 효과 목록에 있는 모든 효과에 대한 불변 참조 벡터를 반환합니다.
    ///
    /// # Returns
    ///
    /// - `Vec<&Box<dyn Effect>>`: 효과 목록에 있는 모든 효과에 대한 불변 참조 벡터.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::effect::{Effects, EffectTiming, Effect};
    /// use simulator_core::effect::types::EffectSpeed;
    ///
    /// // Example Effect (replace with a real implementation)
    /// struct ExampleEffect {}
    /// impl Effect for ExampleEffect {
    ///     fn apply(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> Result<(), String> {
    ///         Ok(())
    ///     }
    ///     fn can_activate(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> bool {
    ///         true
    ///     }
    ///     fn get_speed(&self) -> EffectSpeed {
    ///         EffectSpeed::Normal
    ///     }
    /// }
    ///
    /// let mut effects = Effects::new();
    /// let effect = Box::new(ExampleEffect {});
    /// let effect_timing = EffectTiming::new(1, EffectSpeed::Fast, effect);
    /// effects.add_effect(effect_timing);
    ///
    /// let retrieved_effects = effects.get_effects();
    /// assert_eq!(retrieved_effects.len(), 1);
    /// ```
    pub fn get_effects(&self) -> Vec<&Box<dyn Effect>> {
        self.prioritized_effects
            .iter()
            .map(|e| e.get_effect())
            .collect::<Vec<_>>()
    }

    /// 효과 목록에 있는 모든 효과에 대한 가변 참조 벡터를 반환합니다.
    ///
    /// # Returns
    ///
    /// - `Vec<&mut Box<dyn Effect>>`: 효과 목록에 있는 모든 효과에 대한 가변 참조 벡터.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::effect::{Effects, EffectTiming, Effect};
    /// use simulator_core::effect::types::EffectSpeed;
    ///
    /// // Example Effect (replace with a real implementation)
    /// struct ExampleEffect {}
    /// impl Effect for ExampleEffect {
    ///     fn apply(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> Result<(), String> {
    ///         Ok(())
    ///     }
    ///     fn can_activate(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> bool {
    ///         true
    ///     }
    ///     fn get_speed(&self) -> EffectSpeed {
    ///         EffectSpeed::Normal
    ///     }
    /// }
    ///
    /// let mut effects = Effects::new();
    /// let effect = Box::new(ExampleEffect {});
    /// let effect_timing = EffectTiming::new(1, EffectSpeed::Fast, effect);
    /// effects.add_effect(effect_timing);
    ///
    /// let retrieved_effects = effects.get_effects_mut();
    /// assert_eq!(retrieved_effects.len(), 1);
    /// ```
    pub fn get_effects_mut(&mut self) -> Vec<&mut Box<dyn Effect>> {
        self.prioritized_effects
            .iter_mut()
            .map(|e| e.get_effect_mut())
            .collect::<Vec<_>>()
    }

    /// 특정 조건을 만족하는 효과만 필터링합니다.
    ///
    /// # Arguments
    ///
    /// * `filter` - `EffectTiming` 객체를 인자로 받아 bool 값을 반환하는 클로저.
    ///
    /// # Returns
    ///
    /// - `Vec<&EffectTiming>`: 조건을 만족하는 효과들의 `EffectTiming` 객체에 대한 불변 참조 벡터.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::effect::{Effects, EffectTiming, Effect};
    /// use simulator_core::effect::types::EffectSpeed;
    ///
    /// // Example Effect (replace with a real implementation)
    /// struct ExampleEffect {}
    /// impl Effect for ExampleEffect {
    ///     fn apply(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> Result<(), String> {
    ///         Ok(())
    ///     }
    ///     fn can_activate(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> bool {
    ///         true
    ///     }
    ///     fn get_speed(&self) -> EffectSpeed {
    ///         EffectSpeed::Normal
    ///     }
    /// }
    ///
    /// let mut effects = Effects::new();
    /// let effect = Box::new(ExampleEffect {});
    /// let effect_timing = EffectTiming::new(1, EffectSpeed::Fast, effect);
    /// effects.add_effect(effect_timing);
    ///
    /// let filtered_effects = effects.filter_effects(|e| e.get_priority() == 1);
    /// assert_eq!(filtered_effects.len(), 1);
    /// ```
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

    /// 체인 상태에서 추가 가능한 효과만 필터링합니다.
    ///
    /// # Arguments
    ///
    /// * `current_chain_level` - 현재 체인 레벨을 나타내는 `EffectSpeed` 값.
    ///
    /// # Returns
    ///
    /// - `Vec<&EffectTiming>`: 현재 체인 레벨보다 빠르거나 같은 속도를 가진 효과들의 `EffectTiming` 객체에 대한 불변 참조 벡터.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::effect::{Effects, EffectTiming, Effect};
    /// use simulator_core::effect::types::EffectSpeed;
    ///
    /// // Example Effect (replace with a real implementation)
    /// struct ExampleEffect {}
    /// impl Effect for ExampleEffect {
    ///     fn apply(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> Result<(), String> {
    ///         Ok(())
    ///     }
    ///     fn can_activate(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> bool {
    ///         true
    ///     }
    ///     fn get_speed(&self) -> EffectSpeed {
    ///         EffectSpeed::Normal
    ///     }
    /// }
    ///
    /// let mut effects = Effects::new();
    /// let effect = Box::new(ExampleEffect {});
    /// let effect_timing = EffectTiming::new(1, EffectSpeed::Fast, effect);
    /// effects.add_effect(effect_timing);
    ///
    /// let chainable_effects = effects.get_chainable_effects(EffectSpeed::Normal);
    /// assert_eq!(chainable_effects.len(), 1);
    /// ```
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

    /// 발동 가능한 효과만 필터링합니다.
    ///
    /// # Arguments
    ///
    /// * `game` - `GameActor`의 주소.
    /// * `source` - 효과의 발동 주체인 `Card` 객체에 대한 참조.
    ///
    /// # Returns
    ///
    /// - `Vec<&EffectTiming>`: 발동 가능한 효과들의 `EffectTiming` 객체에 대한 불변 참조 벡터.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // TODO: 이 함수를 위한 테스트 케이스를 추가하세요. GameActor와 Card 의존성 때문에 현재는 불가능합니다.
    /// ```
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

/// `EffectTiming` 구조체는 효과의 실행 시점과 우선순위, 속도 등의 정보를 담고 있습니다.
/// 효과의 발동 순서를 결정하는 데 사용됩니다.
#[derive(Clone)]
pub struct EffectTiming {
    priority: u8, // 낮을수록 높은 우선순위
    speed: EffectSpeed,
    is_used: bool, // 효과가 사용되었는지 여부
    effect: Box<dyn Effect>,
}

impl EffectTiming {
    /// `EffectTiming` 구조체의 새로운 인스턴스를 생성합니다.
    ///
    /// # Arguments
    ///
    /// * `priority` - 효과의 우선순위. 낮을수록 높은 우선순위를 가집니다.
    /// * `speed` - 효과의 발동 속도.
    /// * `effect` - 실행할 효과 객체. `Effect` 트레잇을 구현해야 합니다.
    ///
    /// # Returns
    ///
    /// - `EffectTiming`: 주어진 인자들을 사용하여 초기화된 새로운 `EffectTiming` 인스턴스.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::effect::{EffectTiming, Effect};
    /// use simulator_core::effect::types::EffectSpeed;
    ///
    /// // Example Effect (replace with a real implementation)
    /// struct ExampleEffect {}
    /// impl Effect for ExampleEffect {
    ///     fn apply(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> Result<(), String> {
    ///         Ok(())
    ///     }
    ///     fn can_activate(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> bool {
    ///         true
    ///     }
    ///     fn get_speed(&self) -> EffectSpeed {
    ///         EffectSpeed::Normal
    ///     }
    /// }
    ///
    /// let effect = Box::new(ExampleEffect {});
    /// let effect_timing = EffectTiming::new(1, EffectSpeed::Fast, effect);
    ///
    /// assert_eq!(effect_timing.get_priority(), 1);
    /// assert_eq!(effect_timing.get_speed(), EffectSpeed::Fast);
    /// ```
    pub fn new(priority: u8, speed: EffectSpeed, effect: Box<dyn Effect>) -> Self {
        Self {
            priority,
            effect,
            speed,
            is_used: false,
        }
    }

    /// 효과의 우선순위를 반환합니다.
    ///
    /// # Returns
    ///
    /// - `u8`: 효과의 우선순위 값. 낮을수록 높은 우선순위를 가집니다.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::effect::{EffectTiming, Effect};
    /// use simulator_core::effect::types::EffectSpeed;
    ///
    /// // Example Effect (replace with a real implementation)
    /// struct ExampleEffect {}
    /// impl Effect for ExampleEffect {
    ///     fn apply(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> Result<(), String> {
    ///         Ok(())
    ///     }
    ///     fn can_activate(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> bool {
    ///         true
    ///     }
    ///     fn get_speed(&self) -> EffectSpeed {
    ///         EffectSpeed::Normal
    ///     }
    /// }
    ///
    /// let effect = Box::new(ExampleEffect {});
    /// let effect_timing = EffectTiming::new(1, EffectSpeed::Fast, effect);
    ///
    /// assert_eq!(effect_timing.get_priority(), 1);
    /// ```
    pub fn get_priority(&self) -> u8 {
        self.priority
    }

    /// 효과의 발동 속도를 반환합니다.
    ///
    /// # Returns
    ///
    /// - `EffectSpeed`: 효과의 발동 속도.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::effect::{EffectTiming, Effect};
    /// use simulator_core::effect::types::EffectSpeed;
    ///
    /// // Example Effect (replace with a real implementation)
    /// struct ExampleEffect {}
    /// impl Effect for ExampleEffect {
    ///     fn apply(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> Result<(), String> {
    ///         Ok(())
    ///     }
    ///     fn can_activate(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> bool {
    ///         true
    ///     }
    ///     fn get_speed(&self) -> EffectSpeed {
    ///         EffectSpeed::Normal
    ///     }
    /// }
    ///
    /// let effect = Box::new(ExampleEffect {});
    /// let effect_timing = EffectTiming::new(1, EffectSpeed::Fast, effect);
    ///
    /// assert_eq!(effect_timing.get_speed(), EffectSpeed::Fast);
    /// ```
    pub fn get_speed(&self) -> EffectSpeed {
        self.speed
    }

    /// 효과 객체에 대한 불변 참조를 반환합니다.
    ///
    /// # Returns
    ///
    /// - `&Box<dyn Effect>`: 효과 객체에 대한 불변 참조.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::effect::{EffectTiming, Effect};
    /// use simulator_core::effect::types::EffectSpeed;
    ///
    /// // Example Effect (replace with a real implementation)
    /// struct ExampleEffect {}
    /// impl Effect for ExampleEffect {
    ///     fn apply(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> Result<(), String> {
    ///         Ok(())
    ///     }
    ///     fn can_activate(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> bool {
    ///         true
    ///     }
    ///     fn get_speed(&self) -> EffectSpeed {
    ///         EffectSpeed::Normal
    ///     }
    /// }
    ///
    /// let effect = Box::new(ExampleEffect {});
    /// let effect_timing = EffectTiming::new(1, EffectSpeed::Fast, effect);
    ///
    /// let retrieved_effect = effect_timing.get_effect();
    /// // You can't directly assert equality on trait objects without more setup.
    /// ```
    pub fn get_effect(&self) -> &Box<dyn Effect> {
        &self.effect
    }

    /// 효과 객체에 대한 가변 참조를 반환합니다.
    ///
    /// # Returns
    ///
    /// - `&mut Box<dyn Effect>`: 효과 객체에 대한 가변 참조.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::effect::{EffectTiming, Effect};
    /// use simulator_core::effect::types::EffectSpeed;
    ///
    /// // Example Effect (replace with a real implementation)
    /// struct ExampleEffect {}
    /// impl Effect for ExampleEffect {
    ///     fn apply(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> Result<(), String> {
    ///         Ok(())
    ///     }
    ///     fn can_activate(&self, _game: actix::Addr<simulator_core::game::GameActor>, _card: &simulator_core::card::Card) -> bool {
    ///         true
    ///     }
    ///     fn get_speed(&self) -> EffectSpeed {
    ///         EffectSpeed::Normal
    ///     }
    /// }
    ///
    /// let mut effect = Box::new(ExampleEffect {});
    /// let mut effect_timing = EffectTiming::new(1, EffectSpeed::Fast, effect);
    ///
    /// let retrieved_effect = effect_timing.get_effect_mut();
    /// // You can't directly assert equality on trait objects without more setup.
    /// ```
    pub fn get_effect_mut(&mut self) -> &mut Box<dyn Effect> {
        &mut self.effect
    }
}