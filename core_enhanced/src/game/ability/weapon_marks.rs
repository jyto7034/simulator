//! Weapon-copy mark subscriptions and permanent, explicitly selected evolution.
use super::marks::{MarkFilter, MarkRatio};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponStage {
    #[default]
    Base,
    Upgraded,
}
crate::authoring::clone_shape::copy_shape!(WeaponStage);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponEvolutionDef {
    pub fragment: String,
    pub form: String,
    pub base: String,
    pub upgraded: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponMarkHit {
    pub chance_percent: MarkRatio,
    pub amount: u32,
    pub multiplier: MarkRatio,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponMarkMultiplier {
    pub relic_definition: String,
    pub multiplier: MarkRatio,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkWeaponDef {
    pub evolution: WeaponEvolutionDef,
    pub key: String,
    pub filter: MarkFilter,
    pub base: WeaponMarkHit,
    pub upgraded: WeaponMarkHit,
    pub modifiers: Vec<WeaponMarkMultiplier>,
}
pub(crate) fn valid_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 128 && s.trim() == s
}
impl MarkWeaponDef {
    pub(crate) fn retained_dynamic_extent(&self) -> Option<usize> {
        let e = &self.evolution;
        let mut n = 0usize;
        for s in [&e.fragment, &e.form, &e.base, &e.upgraded, &self.key] {
            n = n.checked_add(s.capacity().checked_add(32)?)?;
        }
        n = n.checked_add(
            self.modifiers
                .capacity()
                .checked_mul(std::mem::size_of::<WeaponMarkMultiplier>())?,
        )?;
        for m in &self.modifiers {
            n = n.checked_add(m.relic_definition.capacity().checked_add(32)?)?;
        }
        Some(n)
    }
    pub fn validate(&self) -> Result<(), String> {
        let e = &self.evolution;
        if ![&e.fragment, &e.form, &e.base, &e.upgraded, &self.key]
            .into_iter()
            .all(|s| valid_id(s))
            || e.base == e.upgraded
            || e.form == "base"
        {
            return Err("Invalid weapon evolution identity".into());
        }
        for hit in [&self.base, &self.upgraded] {
            hit.chance_percent.validate(true)?;
            hit.multiplier.validate(false)?;
            if !hit.chance_percent.at_most(100) || hit.amount == 0 {
                return Err("Invalid weapon mark grant".into());
            }
        }
        if self.modifiers.len() > 64 {
            return Err("Too many weapon mark modifiers".into());
        }
        let mut relics = std::collections::BTreeSet::new();
        for m in &self.modifiers {
            m.multiplier.validate(false)?;
            if !valid_id(&m.relic_definition) || !relics.insert(&m.relic_definition) {
                return Err("Invalid or duplicate weapon mark modifier".into());
            }
        }
        Ok(())
    }
    pub fn hit(&self, stage: WeaponStage) -> &WeaponMarkHit {
        match stage {
            WeaponStage::Base => &self.base,
            WeaponStage::Upgraded => &self.upgraded,
        }
    }
}
crate::authoring::clone_shape::struct_shape!(WeaponEvolutionDef {
    fragment,
    form,
    base,
    upgraded
});
crate::authoring::clone_shape::struct_shape!(WeaponMarkHit {
    chance_percent,
    amount,
    multiplier
});
crate::authoring::clone_shape::struct_shape!(WeaponMarkMultiplier {
    relic_definition,
    multiplier
});
crate::authoring::clone_shape::struct_shape!(MarkWeaponDef {
    evolution,
    key,
    filter,
    base,
    upgraded,
    modifiers
});

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponMarksSnapshot {
    pub stores: Vec<WeaponMarkStoreSnapshot>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponMarkStoreSnapshot {
    pub weapon_copy: uuid::Uuid,
    pub key: String,
    pub units: Vec<(crate::game::battle::ids::UnitInstanceId, u64)>,
    pub tiles: Vec<(crate::game::resources::Position, u64)>,
}
