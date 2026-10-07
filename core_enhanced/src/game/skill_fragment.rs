use crate::game::{
    ability::{SkillActivationMode, SkillId},
    battle::types::UnitCombatProfile,
    behavior::GameError,
    data::skill_fragment_data::{
        SkillFragmentDatabase, SkillFragmentEffectDef, SkillFragmentForm, SkillFragmentId,
        SkillFragmentMetadata, BASE_FRAGMENT_FORM, MAX_FRAGMENT_FORMS,
    },
    managers::uuid_manager::UuidManager,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;
pub(crate) mod bounded;
pub mod legacy;
mod transition;
use legacy::LegacyFragmentProgress;
pub const MAX_OWNED_FRAGMENTS: usize = 1024;

pub fn starter_basic_attack_fragment_id() -> SkillFragmentId {
    SkillFragmentId::from("starter_basic_attack_enhancement")
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedFragment {
    pub copy_uuid: Uuid,
    pub definition_id: SkillFragmentId,
    #[serde(deserialize_with = "bounded::forms")]
    pub unlocked_forms: BTreeSet<String>,
    pub selected_form: String,
    pub equipped_to: Option<Uuid>,
}
crate::authoring::clone_shape::struct_shape!(OwnedFragment {
    copy_uuid,
    definition_id,
    unlocked_forms,
    selected_form,
    equipped_to
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FragmentSourceSnapshot {
    pub copy_uuid: Uuid,
    pub definition_id: SkillFragmentId,
    pub form_id: String,
    pub skill_id: SkillId,
}
crate::authoring::clone_shape::struct_shape!(FragmentSourceSnapshot {
    copy_uuid,
    definition_id,
    form_id,
    skill_id
});
impl FragmentSourceSnapshot {
    pub(crate) fn into_shared(self) -> Result<std::sync::Arc<Self>, GameError> {
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.retained_extent()
                .and_then(|bytes| bytes.checked_add(64)),
            "fragment_source_owner",
        )?;
        Ok(std::sync::Arc::new(self))
    }
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        std::mem::size_of::<Self>()
            .checked_add(self.definition_id.as_str().len())?
            .checked_add(self.form_id.len())?
            .checked_add(self.skill_id.as_str().len())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillFragmentInventory {
    #[serde(deserialize_with = "bounded::map::<_, _, _, MAX_OWNED_FRAGMENTS>")]
    copies: BTreeMap<Uuid, OwnedFragment>,
    /// Existing reward credits are preserved; there is no dust upgrade/bloom consumer.
    fragment_dust: u32,
    /// Only the explicit V7 converter writes this provenance. Never gameplay truth.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    legacy_progress: BTreeMap<SkillFragmentId, LegacyFragmentProgress>,
}
crate::authoring::clone_shape::struct_shape!(SkillFragmentInventory {
    copies,
    fragment_dust,
    legacy_progress
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillFragmentPolicy {
    pub max_copies: usize,
}
impl SkillFragmentPolicy {
    pub fn default_run_policy() -> Self {
        Self {
            max_copies: MAX_OWNED_FRAGMENTS,
        }
    }
}
impl Default for SkillFragmentPolicy {
    fn default() -> Self {
        Self::default_run_policy()
    }
}

impl SkillFragmentInventory {
    pub(crate) fn import_legacy_history(
        &mut self,
        history: BTreeMap<SkillFragmentId, LegacyFragmentProgress>,
        dust: u32,
    ) {
        self.legacy_progress = history;
        self.fragment_dust = dust;
    }
    pub(crate) fn import_legacy_binding(
        &mut self,
        copy: Uuid,
        owner: Uuid,
    ) -> Result<(), GameError> {
        let copy = self.copies.get_mut(&copy).ok_or(GameError::InvalidAction)?;
        if owner.is_nil() || copy.equipped_to.is_some() {
            return Err(GameError::InvalidAction);
        }
        copy.equipped_to = Some(owner);
        Ok(())
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn copies(&self) -> impl Iterator<Item = &OwnedFragment> {
        self.copies.values()
    }
    pub fn get(&self, copy_uuid: &Uuid) -> Option<&OwnedFragment> {
        self.copies.get(copy_uuid)
    }
    pub fn owned_ids(&self) -> impl Iterator<Item = &SkillFragmentId> {
        self.copies
            .values()
            .map(|c| &c.definition_id)
            .collect::<BTreeSet<_>>()
            .into_iter()
    }
    pub fn count(&self, definition: &SkillFragmentId) -> u32 {
        self.copies
            .values()
            .filter(|c| &c.definition_id == definition)
            .count() as u32
    }
    pub fn contains(&self, definition: &SkillFragmentId) -> bool {
        self.count(definition) != 0
    }
    pub fn fragment_dust(&self) -> u32 {
        self.fragment_dust
    }
    pub fn add_fragment_dust(&mut self, amount: u32) -> Result<u32, GameError> {
        if amount == 0 {
            return Err(GameError::InvalidAction);
        }
        self.fragment_dust = self
            .fragment_dust
            .checked_add(amount)
            .ok_or(GameError::InvalidAction)?;
        Ok(self.fragment_dust)
    }
    pub fn grant(
        &mut self,
        metadata: &SkillFragmentMetadata,
        uuids: &mut UuidManager,
        policy: &SkillFragmentPolicy,
    ) -> Result<Uuid, GameError> {
        if self.copies.len() >= policy.max_copies.min(MAX_OWNED_FRAGMENTS)
            || metadata.id.as_str().is_empty()
        {
            return Err(GameError::InvalidAction);
        }
        let expected = uuids.peek(UuidManager::NS_OWNED_FRAGMENT);
        if expected.is_nil() || self.copies.contains_key(&expected) {
            return Err(GameError::InvalidAction);
        }
        let copy_uuid = uuids.next_owned_fragment()?;
        self.copies.insert(
            copy_uuid,
            OwnedFragment {
                copy_uuid,
                definition_id: metadata.id.clone(),
                unlocked_forms: BTreeSet::from([BASE_FRAGMENT_FORM.to_string()]),
                selected_form: BASE_FRAGMENT_FORM.to_string(),
                equipped_to: None,
            },
        );
        Ok(copy_uuid)
    }
    pub fn resolve<'a>(
        &'a self,
        copy_uuid: &Uuid,
        database: &'a SkillFragmentDatabase,
    ) -> Result<
        (
            &'a OwnedFragment,
            &'a SkillFragmentMetadata,
            &'a SkillFragmentForm,
        ),
        GameError,
    > {
        let copy = self.copies.get(copy_uuid).ok_or(GameError::InvalidAction)?;
        let metadata = database.get_by_id(&copy.definition_id).ok_or_else(|| {
            GameError::InvalidStaticData(format!(
                "fragment definition '{}' missing",
                copy.definition_id
            ))
        })?;
        let SkillFragmentEffectDef::ActiveSkill { forms, .. } = &metadata.effect else {
            return Err(GameError::InvalidAction);
        };
        if !copy.unlocked_forms.contains(&copy.selected_form) {
            return Err(GameError::InvalidAction);
        }
        let form = forms
            .get(&copy.selected_form)
            .ok_or(GameError::InvalidAction)?;
        Ok((copy, metadata, form))
    }
    pub(crate) fn unlock(
        &mut self,
        copy_uuid: Uuid,
        form_id: &str,
        database: &SkillFragmentDatabase,
    ) -> Result<bool, GameError> {
        let copy = self
            .copies
            .get_mut(&copy_uuid)
            .ok_or(GameError::InvalidAction)?;
        let metadata = database
            .get_by_id(&copy.definition_id)
            .ok_or(GameError::InvalidAction)?;
        let SkillFragmentEffectDef::ActiveSkill { forms, .. } = &metadata.effect else {
            return Err(GameError::InvalidAction);
        };
        if !forms.contains_key(form_id)
            || copy.unlocked_forms.len() >= MAX_FRAGMENT_FORMS
                && !copy.unlocked_forms.contains(form_id)
        {
            return Err(GameError::InvalidAction);
        }
        Ok(copy.unlocked_forms.insert(form_id.to_owned()))
    }
    pub(crate) fn select(
        &mut self,
        copy_uuid: Uuid,
        form_id: &str,
        database: &SkillFragmentDatabase,
    ) -> Result<bool, GameError> {
        let copy = self
            .copies
            .get_mut(&copy_uuid)
            .ok_or(GameError::InvalidAction)?;
        let metadata = database
            .get_by_id(&copy.definition_id)
            .ok_or(GameError::InvalidAction)?;
        let SkillFragmentEffectDef::ActiveSkill { forms, .. } = &metadata.effect else {
            return Err(GameError::InvalidAction);
        };
        if !forms.contains_key(form_id) || !copy.unlocked_forms.contains(form_id) {
            return Err(GameError::InvalidAction);
        }
        let changed = copy.selected_form != form_id;
        if changed {
            copy.selected_form = form_id.to_owned();
        }
        Ok(changed)
    }
    pub(crate) fn remove(&mut self, copy_uuid: Uuid) -> Result<OwnedFragment, GameError> {
        if self
            .copies
            .get(&copy_uuid)
            .ok_or(GameError::InvalidAction)?
            .equipped_to
            .is_some()
        {
            return Err(GameError::InvalidAction);
        }
        Ok(self
            .copies
            .remove(&copy_uuid)
            .expect("validated exact copy"))
    }
    pub(crate) fn validate_checkpoint_state(
        &self,
        database: &SkillFragmentDatabase,
        policy: &SkillFragmentPolicy,
    ) -> bool {
        self.copies.len() <= policy.max_copies.min(MAX_OWNED_FRAGMENTS)
            && self.copies.iter().all(|(key, copy)| {
                if key.is_nil()
                    || key != &copy.copy_uuid
                    || copy.equipped_to.is_some_and(|id| id.is_nil())
                    || !copy.unlocked_forms.contains(BASE_FRAGMENT_FORM)
                    || !copy.unlocked_forms.contains(&copy.selected_form)
                    || copy.unlocked_forms.len() > MAX_FRAGMENT_FORMS
                {
                    return false;
                }
                let Some(metadata) = database.get_by_id(&copy.definition_id) else {
                    return false;
                };
                match &metadata.effect {
                    SkillFragmentEffectDef::BasicAttackModifier { .. } => {
                        copy.selected_form == BASE_FRAGMENT_FORM
                            && copy.unlocked_forms.len() == 1
                            && copy.equipped_to.is_none()
                    }
                    SkillFragmentEffectDef::ActiveSkill { forms, .. } => {
                        copy.unlocked_forms.iter().all(|id| forms.contains_key(id))
                    }
                }
            })
            && self.legacy_progress.iter().all(|(id, old)| {
                database.get_by_id(id).is_some()
                    && old.upgrade_level <= 10
                    && old.awakening_progress == u32::from(old.upgrade_level)
                    && (!old.awakened || old.awakening_available)
            })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillFragmentLoadout {
    baseline_ids: Vec<SkillFragmentId>,
    active_copy_uuid: Option<Uuid>,
}
impl SkillFragmentLoadout {
    #[cfg(test)]
    pub fn starter() -> Self {
        Self::with_baseline_ids(vec![starter_basic_attack_fragment_id()])
    }
    pub fn with_baseline_ids(baseline_ids: Vec<SkillFragmentId>) -> Self {
        Self {
            baseline_ids,
            active_copy_uuid: None,
        }
    }
    pub fn baseline_ids(&self) -> &[SkillFragmentId] {
        &self.baseline_ids
    }
    pub fn active_copy_uuid(&self) -> Option<Uuid> {
        self.active_copy_uuid
    }
    pub(crate) fn set_copy(&mut self, copy_uuid: Option<Uuid>) {
        self.active_copy_uuid = copy_uuid;
    }
    pub fn apply_to_profile(
        &self,
        database: &SkillFragmentDatabase,
        inventory: &SkillFragmentInventory,
        owner: Uuid,
        base_profile: &UnitCombatProfile,
    ) -> Result<UnitCombatProfile, GameError> {
        if base_profile.fragment_source.is_some() || !base_profile.has_employee_base_resonance() {
            return Err(GameError::InvalidAction);
        }
        let mut profile = base_profile.clone();
        for fragment_id in &self.baseline_ids {
            let metadata = database.get_by_id(fragment_id).ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "employee loadout references missing skill fragment '{fragment_id}'"
                ))
            })?;

            match &metadata.effect {
                SkillFragmentEffectDef::BasicAttackModifier {
                    attack_bonus,
                    attack_interval_ms_reduction,
                } => {
                    profile
                        .stats
                        .add_attack((*attack_bonus).min(i32::MAX as u32) as i32);
                    if *attack_interval_ms_reduction > 0 {
                        let reduction = i64::from(*attack_interval_ms_reduction);
                        profile.stats.add_attack_interval_ms(-reduction);
                        profile.basic_attack.interval_ms = profile
                            .basic_attack
                            .interval_ms
                            .saturating_sub(*attack_interval_ms_reduction as u64)
                            .max(1);
                    }
                }
                SkillFragmentEffectDef::ActiveSkill { .. } => {
                    return Err(GameError::InvalidStaticData(format!(
                        "baseline skill fragment '{fragment_id}' cannot grant an active skill"
                    )));
                }
            }
        }

        if let Some(copy_uuid) = self.active_copy_uuid {
            let (copy, _metadata, form) = inventory.resolve(&copy_uuid, database)?;
            if copy.equipped_to != Some(owner) || profile.skill_id.is_some() {
                return Err(GameError::InvalidAction);
            }
            // Equipment compatibility is checked after the actual weapon profile is composed
            // by the world transition and final battle producer. This early stat pass has no weapon.
            profile.skill_id = Some(form.skill_id.clone());
            profile.resonance.max = form.resonance_max;
            profile.resonance.start = form.resonance_max / 5;
            profile.resonance.gain_lock_ms = 0;
            profile.skill_activation_mode = SkillActivationMode::Manual;
            profile.fragment_source = Some(FragmentSourceSnapshot {
                copy_uuid,
                definition_id: copy.definition_id.clone(),
                form_id: copy.selected_form.clone(),
                skill_id: form.skill_id.clone(),
            });
        }
        Ok(profile)
    }
}
