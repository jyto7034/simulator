use std::any::Any;

use uuid::Uuid;

use crate::{
    enums::ZoneType,
    exception::GameError,
    game::Game,
    selector::TargetSelector,
    server::input_handler::{InputAnswer, InputRequest},
    zone::zone::Zone,
};

use super::{insert::Insert, take::Take, types::StatType, Card};

pub struct EffectInfo {
    pub effect_id: Uuid,
    pub effect_type: EffectType,
    pub from_location: ZoneType,
    pub to_location: ZoneType,
}

impl EffectInfo {
    pub fn new(
        effect_id: Uuid,
        effect_type: EffectType,
        from_location: ZoneType,
        to_location: ZoneType,
    ) -> Self {
        Self {
            effect_id,
            effect_type,
            from_location,
            to_location,
        }
    }
}

pub enum EffectResult {
    // 효과가 완전히 실행됨
    Completed,

    // 사용자 입력이 필요함
    NeedsInput {
        inner: InputRequest,
        handler: Box<
            dyn FnOnce(&mut Game, &Card, InputAnswer) -> Result<EffectResult, GameError>
                + Send
                + Sync,
        >,
    },
}

#[derive(PartialEq, Eq)]
pub enum EffectLevel {
    Immediate,
    Chain,
}

pub trait EffectTiming {
    fn default_timing(&self) -> EffectLevel {
        EffectLevel::Chain
    }
    fn get_timing(&self) -> EffectLevel {
        self.default_timing()
    }
}

impl<T: Effect> EffectTiming for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectType {
    Dig,
    Draw,
    ModifyStat,
}

#[derive(Debug, Clone, Copy)]
pub enum EffectProcessPhase {
    ImmediatePhase, // 즉발 효과 처리 중
    ChainPhase,     // 체인 효과 처리 중
    InputWaiting,   // 사용자 입력 대기 중
}

// 이 카드명의 "카드"는 1턴에 1장밖에 "발동"할 수 없다.
// 이 카드명의 "효과"는 1턴에 1장밖에 "사용"할 수 없다.
pub trait Effect: Send + Sync + EffectTiming {
    /// 효과를 발동합니다.
    /// # Arguments
    /// * `game` - 게임 객체
    /// * `source` - 해당 효과를 발동시킨 카드
    /// # Returns
    /// * `Result<EffectResult, GameError>`
    /// # Errors
    /// * `GameError` - 효과 적용에 실패한 경우.
    fn begin_effect(&self, game: &mut Game, source: &Card) -> Result<EffectResult, GameError>;

    /// 효과를 발동할 수 있는지 확인합니다.
    /// # Arguments
    /// * `game` - 게임 객체
    /// * `source` - 해당 효과를 발동시킨 카드
    /// # Returns
    /// * `bool`
    fn can_activate(&self, game: &Game, source: &Card) -> bool;

    fn handle_input(
        &self,
        game: &mut Game,
        source: &Card,
        input: InputAnswer,
    ) -> Result<EffectResult, GameError> {
        Err(GameError::InputNotExpected)
    }

    fn clone_effect(&self) -> Result<Box<dyn Effect>, GameError>;

    fn get_effect_type(&self) -> EffectType;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn get_id(&self) -> Uuid;
}

pub struct DigEffect {
    pub selector: Box<dyn TargetSelector>,
    pub insert: Box<dyn Insert>,
    pub take: Box<dyn Take>,
    pub info: EffectInfo,
}

impl DigEffect {
    pub fn new(
        selector: Box<dyn TargetSelector>,
        insert: Box<dyn Insert>,
        take: Box<dyn Take>,
        info: EffectInfo,
    ) -> Self {
        Self {
            selector,
            insert,
            take,
            info,
        }
    }

    pub fn get_selector(&self) -> &Box<dyn TargetSelector> {
        &self.selector
    }

    pub fn get_selector_mut(&mut self) -> &mut Box<dyn TargetSelector> {
        &mut self.selector
    }

    pub fn get_effect_type(&self) -> EffectType {
        EffectType::Dig
    }
}

impl Effect for DigEffect {
    /// dig 효과를 발동합니다.
    /// # Arguments
    /// * `game` - 게임 객체
    /// * `source` - 해당 효과를 발동시킨 카드
    /// # Returns
    /// * `Result<EffectResult, GameError>`
    /// # Errors
    /// * `GameError` - 효과 적용에 실패한 경우.
    fn begin_effect(&self, game: &mut Game, source: &Card) -> Result<EffectResult, GameError> {
        // select_targets 으로 대상 카드를 가져옵니다.
        let potential_targets = self.selector.select_targets(game, source)?;

        if potential_targets.is_empty() {
            // 파낼 카드가 없으면 효과 종료 (또는 다른 처리)
            return Ok(EffectResult::Completed);
        }

        // Vec<Card> -> Vec<Uuid> 변환
        let potential_targets_uuids = potential_targets
            .iter()
            .map(|card| card.get_uuid())
            .collect::<Vec<Uuid>>();

        Ok(EffectResult::NeedsInput {
            inner: InputRequest::Dig {
                source_card: source.get_uuid(),
                source_effect_uuid: self.info.effect_id,
                potential_cards: potential_targets_uuids,
            },
            handler: Box::new(|game, source, input| Ok(EffectResult::Completed)),
        })
    }
    fn handle_input(
        &self,
        game: &mut Game,
        source: &Card,
        input: InputAnswer,
    ) -> Result<EffectResult, GameError> {
        todo!()
    }

    fn can_activate(&self, game: &Game, source: &Card) -> bool {
        todo!()
    }

    fn clone_effect(&self) -> Result<Box<dyn Effect>, GameError> {
        todo!()
    }

    fn get_effect_type(&self) -> EffectType {
        EffectType::Dig
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_id(&self) -> Uuid {
        self.info.effect_id
    }
}

pub struct DrawEffect {
    pub count: usize,
}

impl Effect for DrawEffect {
    fn begin_effect(&self, game: &mut Game, source: &Card) -> Result<EffectResult, GameError> {
        todo!()
    }

    fn can_activate(&self, game: &Game, source: &Card) -> bool {
        game.get_player_by_type(source.get_owner())
            .get()
            .get_deck()
            .len()
            >= self.count
    }

    fn clone_effect(&self) -> Result<Box<dyn Effect>, GameError> {
        Ok(Box::new(Self { count: self.count }))
    }

    fn get_effect_type(&self) -> EffectType {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        todo!()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        todo!()
    }

    fn get_id(&self) -> Uuid {
        todo!()
    }
}

pub struct ModifyStatEffect {
    pub stat_type: StatType,
    pub amount: i32,
    pub target_selector: Box<dyn TargetSelector>,
}

impl Effect for ModifyStatEffect {
    fn begin_effect(&self, game: &mut Game, source: &Card) -> Result<EffectResult, GameError> {
        todo!()
        // let targets = self.target_selector.select_targets(game, source)?;
        // for mut target in targets {
        //     target.modify_stat(self.stat_type, self.amount)?;
        // }
        // Ok(())
    }

    fn can_activate(&self, game: &Game, source: &Card) -> bool {
        self.target_selector.has_valid_targets(game, source)
    }

    fn clone_effect(&self) -> Result<Box<dyn Effect>, GameError> {
        Ok(Box::new(Self {
            stat_type: self.stat_type,
            amount: self.amount,
            target_selector: self.target_selector.clone_selector(),
        }))
    }

    fn get_effect_type(&self) -> EffectType {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        todo!()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        todo!()
    }

    fn get_id(&self) -> Uuid {
        todo!()
    }
}
