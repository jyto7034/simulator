use super::{Candidate, ModValidationError};
use crate::{
    authoring::{clone_shape::CloneShape, control},
    game::{
        battle::{runtime_facts::RuntimeFactSource, skill_source::SkillEffectExecutionSource},
        behavior::GameError,
    },
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const MAX_KEYS: usize = 8192;
pub const COVERAGE_SUPPORT_REVISION: &str = "cmva.coverage-support/v1";
#[derive(Debug, Clone, Serialize)]
pub struct DefinitionCoverageSupport {
    pub kind: crate::game::data::content_pack::DefinitionKind,
    pub add_or_replace: bool,
    pub remove: bool,
    pub consumer: &'static str,
}
#[derive(Debug, Clone, Serialize)]
pub struct CoverageSupport {
    pub revision: &'static str,
    pub definitions: Vec<DefinitionCoverageSupport>,
    pub limitations: Vec<&'static str>,
}
/// Bounded versioned suite support, not a claim that every valid mod has an
/// exercised consumer. Unsupported shapes/operations remain Incomplete.
pub fn coverage_support() -> CoverageSupport {
    use crate::game::data::content_pack::DefinitionKind as K;
    CoverageSupport {
        revision: COVERAGE_SUPPORT_REVISION,
        definitions: K::ALL.iter().map(|&kind| {
            let consumer = match kind {
                K::Employee => "normal employee loadout preparation",
                K::Equipment => "normal equipment owner, activation and triggered effects",
                K::SkillFragment => "normal basic profile or actual active fragment effects",
                K::Relic => "actual numeric, structural grant/edit/recipe/wedge and causal effects",
                K::PveEncounter => "normal encounter planning and battle commit",
                K::RunPolicy => "normal run start",
                K::Skill => "actual authored skill effect sites",
                K::Buff => "actual buff consumption",
                K::IndependentObject => "actual object spawn and terminal/reclamation",
                _ => "unsupported normal-consumer witness",
            };
            DefinitionCoverageSupport { kind, add_or_replace: matches!(kind,K::Employee|K::Equipment|K::SkillFragment|K::Relic|K::PveEncounter|K::RunPolicy|K::Skill|K::Buff|K::IndependentObject), remove: matches!(kind,K::Employee|K::Equipment|K::SkillFragment|K::Relic|K::PveEncounter), consumer }
        }).collect(),
        limitations: vec!["every changed definition/site is required; trusted suites may only strengthen this union", "legal empty Skill steps, Relic grant/causal sites, or definitions with zero known sites remain Incomplete", "Remove requires frozen trusted-base positive preparation, exact removed-reference refusal, and actual same-kind surviving preparation; mod-only introduced definitions lack this counterfactual and remain Incomplete", "missing eligible target/trigger/required witness is Incomplete; complete measured limit exceedance is ContentFailed"],
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageKey {
    pub generation: String,
    pub definition_kind: String,
    pub definition_id: String,
    pub action_id: String,
    pub effect_id: String,
    pub producer_family: String,
}
crate::authoring::clone_shape::struct_shape!(CoverageKey {
    generation,
    definition_kind,
    definition_id,
    action_id,
    effect_id,
    producer_family
});
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStage {
    CatalogValidated,
    Triggered,
    Ineligible,
    LegalNoop,
    ConsumedOrEffect,
    TerminatedOrReclaimed,
}
crate::authoring::clone_shape::copy_shape!(CoverageStage);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageRequirement {
    pub key: CoverageKey,
    pub required_stage: CoverageStage,
}
crate::authoring::clone_shape::struct_shape!(CoverageRequirement {
    key,
    required_stage
});
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageRow {
    pub requirement: CoverageRequirement,
    pub counts: [u64; 6],
}
impl CloneShape for CoverageRow {
    fn visit_clone_shape(&self, meter: &mut control::CloneMeter) -> Result<(), GameError> {
        self.requirement.visit_clone_shape(meter)?;
        meter.scan_storage(Some(6))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageReport {
    pub rows: Vec<CoverageRow>,
    pub overflow: bool,
    pub unsupported: Vec<String>,
}
crate::authoring::clone_shape::struct_shape!(CoverageReport {
    rows,
    overflow,
    unsupported
});
impl CoverageReport {
    pub fn is_complete(&self) -> bool {
        !self.overflow
            && self.unsupported.is_empty()
            && self
                .rows
                .iter()
                .all(|r| r.counts[r.requirement.required_stage as usize] > 0)
    }
    pub(crate) fn merge(&mut self, other: &Self) -> Result<(), GameError> {
        if self.rows.len() != other.rows.len()
            || self
                .rows
                .iter()
                .zip(&other.rows)
                .any(|(a, b)| a.requirement != b.requirement)
        {
            return Err(invalid());
        }
        control::charge(
            (self.rows.len() as u64).saturating_mul(6),
            "lab_coverage_merge",
        )?;
        for (a, b) in self.rows.iter_mut().zip(&other.rows) {
            for (left, right) in a.counts.iter_mut().zip(b.counts) {
                *left = left.checked_add(right).ok_or_else(invalid)?;
            }
        }
        self.overflow |= other.overflow;
        Ok(())
    }
    pub(crate) fn record_definition(
        &mut self,
        kind: &str,
        id: &str,
        family: &str,
        stage: CoverageStage,
    ) -> Result<(), GameError> {
        control::charge(self.rows.len() as u64 + 1, "lab_coverage_world_consumer")?;
        for row in &mut self.rows {
            let key = &row.requirement.key;
            if key.definition_kind == kind
                && key.definition_id == id
                && key.producer_family == family
            {
                row.counts[stage as usize] = row.counts[stage as usize]
                    .checked_add(1)
                    .ok_or_else(invalid)?;
            }
        }
        Ok(())
    }
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut n = std::mem::size_of::<Self>().checked_add(
            self.rows
                .capacity()
                .checked_mul(std::mem::size_of::<CoverageRow>())?,
        )?;
        for r in &self.rows {
            let k = &r.requirement.key;
            for s in [
                &k.generation,
                &k.definition_kind,
                &k.definition_id,
                &k.action_id,
                &k.effect_id,
                &k.producer_family,
            ] {
                n = n.checked_add(s.capacity())?;
            }
        }
        n = n.checked_add(
            self.unsupported
                .capacity()
                .checked_mul(std::mem::size_of::<String>())?,
        )?;
        for s in &self.unsupported {
            n = n.checked_add(s.capacity())?;
        }
        Some(n)
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct CoveragePlan {
    pub(crate) report: CoverageReport,
}
crate::authoring::clone_shape::struct_shape!(CoveragePlan { report });
impl CoveragePlan {
    pub(crate) fn for_candidate(candidate: &Candidate) -> Result<Arc<Self>, ModValidationError> {
        Self::for_suite(candidate, &[])
    }
    pub(crate) fn for_suite(
        candidate: &Candidate,
        required: &[crate::game::data::content_pack::DefinitionKey],
    ) -> Result<Arc<Self>, ModValidationError> {
        let mut rows = Vec::new();
        let mut unsupported = Vec::new();
        let mut obligations: std::collections::BTreeSet<_> =
            candidate.view.changed_definitions.iter().cloned().collect();
        obligations.extend(required.iter().cloned());
        for definition in &candidate.publication.definitions {
            if !obligations.remove(&definition.key) {
                continue;
            }
            let kind = format!("{:?}", definition.key.kind);
            let rows_before = rows.len();
            let mut push = |action: String,
                            effect: String,
                            family: &str,
                            stage|
             -> Result<(), ModValidationError> {
                if rows.len() >= MAX_KEYS {
                    return Err(ModValidationError::incomplete(
                        "coverage.keys",
                        "finite coverage key limit",
                    ));
                }
                let key = CoverageKey {
                    generation: candidate
                        .view
                        .required
                        .content_identity
                        .semantic_sha256
                        .clone(),
                    definition_kind: kind.clone(),
                    definition_id: definition.key.id.clone(),
                    action_id: action,
                    effect_id: effect,
                    producer_family: family.into(),
                };
                rows.push(CoverageRow {
                    requirement: CoverageRequirement {
                        key,
                        required_stage: stage,
                    },
                    counts: [0; 6],
                });
                Ok(())
            };
            match kind.as_str() {
                "Employee" => push(
                    String::new(),
                    String::new(),
                    "employee_profile",
                    CoverageStage::ConsumedOrEffect,
                )?,
                "PveEncounter" => push(
                    String::new(),
                    String::new(),
                    "encounter_plan",
                    CoverageStage::ConsumedOrEffect,
                )?,
                "RunPolicy" => push(
                    String::new(),
                    String::new(),
                    "run_start_policy",
                    CoverageStage::ConsumedOrEffect,
                )?,
                "SkillFragment" => {
                    let fragment = candidate
                        .generation()
                        .database()
                        .skill_fragment_data
                        .fragments
                        .iter()
                        .find(|f| f.id.as_str() == definition.key.id);
                    if let Some(fragment) = fragment {
                        match &fragment.effect {
                            crate::game::data::skill_fragment_data::SkillFragmentEffectDef::BasicAttackModifier { .. } => push(String::new(), String::new(), "fragment_profile", CoverageStage::ConsumedOrEffect)?,
                            crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill { forms, .. } => {
                                for (id, form) in forms { push(id.clone(), form.skill_id.as_str().into(), "fragment_effect", CoverageStage::ConsumedOrEffect)?; }
                            }
                        }
                    } else {
                        unsupported.push(format!("{kind}:{}", definition.key.id));
                    }
                }
                "Skill" => {
                    let skill = candidate
                        .generation()
                        .database()
                        .skill_data
                        .skills
                        .iter()
                        .find(|s| s.id.as_str() == definition.key.id);
                    if let Some(skill) = skill {
                        for step in &skill.steps {
                            if step.effects.is_empty() {
                                unsupported.push(format!(
                                    "zero_site:{kind}:{}:step:{}",
                                    definition.key.id, step.id
                                ));
                            }
                            for effect in &step.effects {
                                push(
                                    step.id.clone(),
                                    effect.id.clone(),
                                    "skill_effect",
                                    if matches!(
                                        effect.op,
                                        crate::game::ability::SkillEffectDef::Damage {
                                            amount: 0,
                                            ..
                                        } | crate::game::ability::SkillEffectDef::Heal {
                                            amount: 0
                                        }
                                    ) {
                                        CoverageStage::LegalNoop
                                    } else {
                                        CoverageStage::ConsumedOrEffect
                                    },
                                )?;
                            }
                        }
                    } else {
                        unsupported.push(format!("{kind}:{}", definition.key.id));
                    }
                }
                "Equipment" => {
                    let equipment = candidate
                        .generation()
                        .database()
                        .equipment_data
                        .items
                        .iter()
                        .find(|e| e.id == definition.key.id);
                    if let Some(equipment) = equipment {
                        push(
                            String::new(),
                            String::new(),
                            &format!("equipment_owner/{}", equipment.uuid),
                            CoverageStage::ConsumedOrEffect,
                        )?;
                        for (index, binding) in equipment.ability_activations.iter().enumerate() {
                            push(
                                binding.ability_id.as_str().into(),
                                index.to_string(),
                                &format!("equipment_activation/{}", equipment.uuid),
                                CoverageStage::ConsumedOrEffect,
                            )?;
                        }
                        for (trigger, effects) in &equipment.triggered_effects {
                            for index in 0..effects.len() {
                                push(
                                    format!("{trigger:?}"),
                                    index.to_string(),
                                    &format!("equipment_trigger/{}", equipment.uuid),
                                    CoverageStage::ConsumedOrEffect,
                                )?;
                            }
                        }
                    } else {
                        unsupported.push(format!("{kind}:{}", definition.key.id));
                    }
                }
                "Relic" => {
                    let relic = candidate
                        .generation()
                        .database()
                        .relic_data
                        .relics
                        .iter()
                        .find(|r| r.id == definition.key.id);
                    if let Some(relic) = relic {
                        let mut forms = vec![(
                            crate::game::relic::RelicVariant::Normal,
                            &relic.rules,
                            &relic.effects,
                        )];
                        if let Some(corrupt) = &relic.corruption {
                            forms.push((
                                crate::game::relic::RelicVariant::Corrupt,
                                &corrupt.rules,
                                &corrupt.effects,
                            ));
                        }
                        for (variant, rules, effects) in forms {
                            for effect in effects {
                                push(
                                    effect.id.clone(),
                                    String::new(),
                                    &format!("relic_rule/{variant:?}"),
                                    CoverageStage::ConsumedOrEffect,
                                )?;
                            }
                            for rule in rules {
                                if empty_relic_consumer(rule) {
                                    unsupported.push(format!(
                                        "zero_site:{kind}:{}:rule:{}",
                                        definition.key.id, rule.id
                                    ));
                                }
                                relic_requirements(rule, variant, &mut push)?;
                            }
                        }
                    } else {
                        unsupported.push(format!("{kind}:{}", definition.key.id));
                    }
                }
                "Buff" => push(
                    String::new(),
                    String::new(),
                    "buff",
                    CoverageStage::ConsumedOrEffect,
                )?,
                "IndependentObject" => {
                    push(
                        String::new(),
                        String::new(),
                        "object_spawn",
                        CoverageStage::ConsumedOrEffect,
                    )?;
                    push(
                        String::new(),
                        String::new(),
                        "object_terminal",
                        CoverageStage::TerminatedOrReclaimed,
                    )?;
                }
                // A trusted suite must provide a concrete normal-consumer oracle
                // for these catalog kinds. Static publication alone is no witness.
                _ => unsupported.push(format!("{kind}:{}", definition.key.id)),
            }
            if rows.len() == rows_before {
                unsupported.push(format!("zero_site:{kind}:{}", definition.key.id));
            }
        }
        for removed in &candidate.view.removed_definitions {
            obligations.remove(removed);
            if matches!(
                removed.kind,
                crate::game::data::content_pack::DefinitionKind::Employee
                    | crate::game::data::content_pack::DefinitionKind::Equipment
                    | crate::game::data::content_pack::DefinitionKind::SkillFragment
                    | crate::game::data::content_pack::DefinitionKind::Relic
                    | crate::game::data::content_pack::DefinitionKind::PveEncounter
            ) {
                for (family, stage) in [
                    ("removed_absence", CoverageStage::CatalogValidated),
                    ("removed_reference", CoverageStage::Ineligible),
                    ("removed_surviving", CoverageStage::ConsumedOrEffect),
                ] {
                    if rows.len() >= MAX_KEYS {
                        return Err(ModValidationError::incomplete(
                            "coverage.keys",
                            "finite coverage key limit",
                        ));
                    }
                    rows.push(CoverageRow {
                        requirement: CoverageRequirement {
                            key: CoverageKey {
                                generation: candidate
                                    .view
                                    .required
                                    .content_identity
                                    .semantic_sha256
                                    .clone(),
                                definition_kind: format!("{:?}", removed.kind),
                                definition_id: removed.id.clone(),
                                action_id: String::new(),
                                effect_id: String::new(),
                                producer_family: family.into(),
                            },
                            required_stage: stage,
                        },
                        counts: [0; 6],
                    });
                }
            } else {
                unsupported.push(format!("removed:{:?}:{}", removed.kind, removed.id));
            }
        }
        for missing in obligations {
            unsupported.push(format!(
                "required_missing:{:?}:{}",
                missing.kind, missing.id
            ));
        }
        Ok(Arc::new(Self {
            report: CoverageReport {
                rows,
                overflow: false,
                unsupported,
            },
        }))
    }
}
#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoverageCollector {
    pub report: CoverageReport,
}
crate::authoring::clone_shape::struct_shape!(CoverageCollector { report });
impl CoverageCollector {
    pub(crate) fn new(plan: &CoveragePlan) -> Result<Box<Self>, GameError> {
        control::metadata_size(&plan.report)?;
        Ok(Box::new(Self {
            report: crate::authoring::clone_shape::controlled_clone(&plan.report)?,
        }))
    }
    pub(crate) fn price_record(&self) -> Result<(), GameError> {
        control::charge(
            self.report.rows.len() as u64 + 1,
            "lab_coverage_actual_producer",
        )?;
        if self.report.overflow
            || self
                .report
                .rows
                .iter()
                .any(|r| r.counts.iter().any(|c| *c == u64::MAX))
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn record_where(
        &mut self,
        stage: CoverageStage,
        mut matches: impl FnMut(&CoverageKey) -> bool,
    ) {
        for row in &mut self.report.rows {
            if matches(&row.requirement.key) {
                let c = &mut row.counts[stage as usize];
                if let Some(next) = c.checked_add(1) {
                    *c = next;
                } else {
                    self.report.overflow = true;
                }
            }
        }
    }
    pub(crate) fn record_effect(
        &mut self,
        source: Option<&SkillEffectExecutionSource>,
        stage: CoverageStage,
    ) {
        let Some(source) = source else {
            return;
        };
        let site = &source.authored.site;
        self.record_where(stage, |k| {
            k.definition_kind == "Skill"
                && k.definition_id == site.action.definition_id.as_str()
                && k.action_id == site.action.action_id
                && k.effect_id == site.effect_id
        });
    }
    pub(crate) fn record_source(&mut self, source: &RuntimeFactSource, stage: CoverageStage) {
        self.record_source_at(source, source.authored.as_ref().map(|a| &a.site), stage);
    }
    pub(crate) fn record_source_at(
        &mut self,
        source: &RuntimeFactSource,
        site: Option<&crate::game::ability::authoring::EffectSite>,
        stage: CoverageStage,
    ) {
        if let Some(fragment) = &source.fragment {
            self.record_where(stage, |k| {
                k.definition_kind == "SkillFragment"
                    && k.definition_id == fragment.definition_id.as_str()
                    && k.producer_family == "fragment_effect"
                    && k.action_id == fragment.form_id
                    && k.effect_id == fragment.skill_id.as_str()
            });
        }
        if let Some(site) = site {
            self.record_where(stage, |k| {
                k.definition_kind == "Skill"
                    && k.definition_id == site.action.definition_id.as_str()
                    && k.action_id == site.action.action_id
                    && k.effect_id == site.effect_id
            });
        }
        if let Some(equipment) = &source.equipment {
            self.record_where(stage, |k| {
                if k.definition_kind != "Equipment" {
                    return false;
                }
                let uuid = k
                    .producer_family
                    .split('/')
                    .nth(1)
                    .and_then(|id| uuid::Uuid::parse_str(id).ok());
                if uuid != Some(equipment.definition_uuid) {
                    return false;
                }
                if k.producer_family.starts_with("equipment_owner/") {
                    return true;
                }
                k.producer_family.starts_with("equipment_activation/")
                    && source.cast.as_ref().is_some_and(|cast| {
                        cast.skill_id.as_str() == k.action_id
                            && cast.proc_origin.as_ref().is_some_and(|p| {
                                k.effect_id.parse::<usize>().ok() == Some(p.binding_index)
                            })
                    })
            });
        }
        if let Some(program) = source
            .causal()
            .and_then(|c| c.program.as_ref())
            .map(|b| &b.selected_program)
        {
            let contribution = &program.contribution;
            if let Some(site) = site {
                self.record_where(stage, |k| {
                    let mut parts = k.producer_family.split('/');
                    k.definition_kind == "Relic"
                        && k.definition_id == contribution.definition_id
                        && parts.next() == Some("causal")
                        && parts.next() == Some(variant_name(contribution.variant))
                        && parts.next() == Some(contribution.rule_id.as_str())
                        && parts.next().and_then(|x| x.parse::<u8>().ok())
                            == Some(program.site_kind)
                        && parts.next().and_then(|x| x.parse::<usize>().ok())
                            == Some(program.site_index)
                        && parts.next() == Some(program.id.as_str())
                        && k.action_id == site.action.action_id
                        && k.effect_id == site.effect_id
                });
            }
        }
        if let Some(binding) = source.relic_effective() {
            if let Some(effect) = site.and_then(|s| binding.effect(s)) {
                if let crate::game::relic::structural::EffectiveEffectKey::Generated {
                    relic_copy_uuid,
                    grant_rule_id,
                    local_action_id,
                    local_effect_id,
                    ..
                } = &effect.key
                {
                    if let Some(grant) = binding.plan.grants.iter().find(|g| {
                        g.contribution.copy_uuid == *relic_copy_uuid
                            && g.contribution.rule_id == *grant_rule_id
                    }) {
                        let c = &grant.contribution;
                        self.record_where(stage, |k| {
                            let mut parts = k.producer_family.split('/');
                            k.definition_kind == "Relic"
                                && k.definition_id == c.definition_id
                                && parts.next() == Some("grant")
                                && parts.next() == Some(variant_name(c.variant))
                                && parts.next() == Some(c.rule_id.as_str())
                                && k.action_id == *local_action_id
                                && k.effect_id == *local_effect_id
                        });
                    }
                }
                for contribution in effect
                    .properties
                    .iter()
                    .flat_map(|p| &p.contributions)
                    .chain(&effect.recipe_contributions)
                {
                    self.record_contribution(contribution, stage, "relic_rule");
                }
            }
        }
        if let Some(binding) = source.relic_numeric() {
            for c in &binding.contributions {
                self.record_where(stage, |k| {
                    k.definition_kind == "Relic"
                        && k.definition_id == c.definition_id
                        && k.action_id == c.effect_id
                        && k.producer_family.strip_prefix("relic_rule/")
                            == Some(variant_name(c.variant))
                });
            }
        }
    }
    pub(crate) fn record_contribution(
        &mut self,
        c: &crate::game::relic::structural::PlanContribution,
        stage: CoverageStage,
        family: &str,
    ) {
        self.record_where(stage, |k| {
            k.definition_kind == "Relic"
                && k.definition_id == c.definition_id
                && k.action_id == c.rule_id
                && k.producer_family
                    .strip_prefix(family)
                    .and_then(|s| s.strip_prefix('/'))
                    == Some(variant_name(c.variant))
        });
    }
    pub(crate) fn record_object(&mut self, id: &str, terminal: bool) {
        self.record_where(
            if terminal {
                CoverageStage::TerminatedOrReclaimed
            } else {
                CoverageStage::ConsumedOrEffect
            },
            |k| {
                k.definition_kind == "IndependentObject"
                    && k.definition_id == id
                    && k.producer_family
                        == if terminal {
                            "object_terminal"
                        } else {
                            "object_spawn"
                        }
            },
        );
    }
    pub(crate) fn record_buff(&mut self, id: crate::game::battle::buffs::BuffId) {
        self.record_where(CoverageStage::ConsumedOrEffect, |k| {
            k.definition_kind == "Buff"
                && crate::game::battle::buffs::BuffId::from_name(&k.definition_id) == id
        });
    }
    pub(crate) fn price_source(&self, source: &RuntimeFactSource) -> Result<(), GameError> {
        self.price_record()?;
        let effective = source.relic_effective().map_or(0, |b| {
            b.plan
                .effects
                .len()
                .saturating_add(b.plan.grants.len())
                .saturating_add(
                    b.plan
                        .effects
                        .iter()
                        .map(|e| {
                            e.properties
                                .iter()
                                .map(|p| p.contributions.len())
                                .sum::<usize>()
                                + e.recipe_contributions.len()
                        })
                        .sum::<usize>(),
                )
        });
        let numeric = source.relic_numeric().map_or(0, |b| b.contributions.len());
        let text = self
            .report
            .rows
            .iter()
            .try_fold(0_u64, |n, r| {
                let k = &r.requirement.key;
                n.checked_add(
                    (k.definition_id.len()
                        + k.action_id.len()
                        + k.effect_id.len()
                        + k.producer_family.len()) as u64,
                )
            })
            .ok_or_else(invalid)?;
        control::charge(
            text.saturating_mul((effective + numeric + 2) as u64),
            "lab_coverage_source_scan",
        )
    }
}
fn invalid() -> GameError {
    control::reject("lab_coverage_invalid", "lab_coverage_table")
}
fn empty_relic_consumer(rule: &crate::game::relic::structural::authoring::RelicRuleDef) -> bool {
    use crate::game::relic::structural::authoring::*;
    let program_empty = |program: &CausalProgramDef| {
        program.steps.is_empty() || program.steps.iter().any(|step| step.effects.is_empty())
    };
    let payload_empty = |payload: &CausalPayloadDef| match payload {
        CausalPayloadDef::Program(program) => program_empty(program),
        CausalPayloadDef::Fork { branches } => {
            branches.is_empty() || branches.iter().any(|b| program_empty(&b.program))
        }
        CausalPayloadDef::Join { branches, then, .. } => {
            branches.is_empty()
                || branches.iter().any(|b| program_empty(&b.program))
                || program_empty(then)
        }
    };
    match &rule.apply {
        RelicRuleActionDef::Grant(GrantBodyDef::InlineEffects { effects, .. }) => {
            effects.is_empty()
        }
        RelicRuleActionDef::Grant(GrantBodyDef::FiniteFlow { steps, .. }) => {
            steps.is_empty() || steps.iter().any(|step| step.effects.is_empty())
        }
        RelicRuleActionDef::Subscribe(subscription) => payload_empty(&subscription.payload),
        RelicRuleActionDef::Periodic { payload, .. } => payload_empty(payload),
        _ => false,
    }
}
fn relic_requirements(
    rule: &crate::game::relic::structural::authoring::RelicRuleDef,
    variant: crate::game::relic::RelicVariant,
    push: &mut impl FnMut(String, String, &str, CoverageStage) -> Result<(), ModValidationError>,
) -> Result<(), ModValidationError> {
    use crate::game::relic::structural::authoring::*;
    match &rule.apply {
        RelicRuleActionDef::Subscribe(s) => {
            payload_requirements(&rule.id, variant, &s.payload, push)
        }
        RelicRuleActionDef::Periodic { payload, .. } => {
            payload_requirements(&rule.id, variant, payload, push)
        }
        RelicRuleActionDef::Wedge { .. } => push(
            rule.id.clone(),
            String::new(),
            &format!("wedge_positive/{variant:?}"),
            CoverageStage::ConsumedOrEffect,
        ),
        RelicRuleActionDef::Grant(GrantBodyDef::InlineEffects { action_id, effects }) => {
            for effect in effects {
                push(
                    action_id.clone(),
                    effect.id.clone(),
                    &format!("grant/{variant:?}/{}", rule.id),
                    CoverageStage::ConsumedOrEffect,
                )?;
            }
            Ok(())
        }
        RelicRuleActionDef::Grant(GrantBodyDef::FiniteFlow { steps, .. }) => {
            for step in steps {
                for effect in &step.effects {
                    push(
                        step.id.clone(),
                        effect.id.clone(),
                        &format!("grant/{variant:?}/{}", rule.id),
                        CoverageStage::ConsumedOrEffect,
                    )?;
                }
            }
            Ok(())
        }
        RelicRuleActionDef::Edit { .. } | RelicRuleActionDef::Recipe(_) => push(
            rule.id.clone(),
            String::new(),
            &format!("relic_rule/{variant:?}"),
            CoverageStage::ConsumedOrEffect,
        ),
    }
}
fn payload_requirements(
    rule: &str,
    variant: crate::game::relic::RelicVariant,
    payload: &crate::game::relic::structural::authoring::CausalPayloadDef,
    push: &mut impl FnMut(String, String, &str, CoverageStage) -> Result<(), ModValidationError>,
) -> Result<(), ModValidationError> {
    use crate::game::relic::structural::authoring::*;
    let mut program = |kind: u8, index: usize, id: &str, p: &CausalProgramDef| {
        let family = format!("causal/{variant:?}/{rule}/{kind}/{index}/{id}");
        for step in &p.steps {
            for effect in &step.effects {
                push(
                    step.id.clone(),
                    effect.id.clone(),
                    &family,
                    if matches!(
                        effect.op,
                        CausalPrimitiveDef::Damage { amount: 0, .. }
                            | CausalPrimitiveDef::Heal { amount: 0 }
                    ) {
                        CoverageStage::LegalNoop
                    } else {
                        CoverageStage::ConsumedOrEffect
                    },
                )?;
            }
        }
        Ok::<(), ModValidationError>(())
    };
    match payload {
        CausalPayloadDef::Program(p) => program(0, 0, "program", p),
        CausalPayloadDef::Fork { branches } | CausalPayloadDef::Join { branches, .. } => {
            for (index, branch) in branches.iter().enumerate() {
                program(1, index, &branch.id, &branch.program)?;
            }
            if let CausalPayloadDef::Join { then, .. } = payload {
                program(2, branches.len(), "then", then)?;
            }
            Ok(())
        }
    }
}

fn variant_name(variant: crate::game::relic::RelicVariant) -> &'static str {
    match variant {
        crate::game::relic::RelicVariant::Normal => "Normal",
        crate::game::relic::RelicVariant::Corrupt => "Corrupt",
    }
}
