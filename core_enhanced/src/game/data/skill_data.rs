use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use crate::game::ability::authoring::{
    effect_support, ActionSite, AuthoredSkillEffect, AuthoringDiagnostic, EffectOrigin,
    EffectProperty, EffectSelection, EffectSite, EffectSource,
};

use crate::game::{
    ability::{
        DeliveryDef, FocusPermissions, ProjectileHitPolicy, SkillAreaAnchorSource,
        SkillCastSelection, SkillCastTargetingDef, SkillDef, SkillEffectDef, SkillKind,
        SkillPresentationDef, SkillStepCondition, SkillStepDef, SkillStepRepeat, SkillTarget,
        SkillTargetReach, StepTargetingMode,
    },
    battle::buffs::BuffDatabase,
    battle::tile_range::TileRangePattern,
    data::build_string_index,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SkillRangePresetDef {
    pub id: String,
    pub range: TileRangePattern,
}

fn path_index(path: &str, name: &str) -> Option<usize> {
    let (_, tail) = path.split_once(&format!("{name}["))?;
    tail.split_once(']')?.0.parse().ok()
}

// Best-effort identities for a failed primary parse. This scanner never builds
// a second RON tree: it walks bounded lexical ranges and allocates only the
// three selected ID strings named by the primary serde path.
fn enrich_parse_error(
    input: &str,
    mut error: super::live_ron::LiveRonError,
) -> super::live_ron::LiveRonError {
    let Some(path) = error.field_path.as_deref() else {
        return error;
    };
    let Some(definition_index) = path_index(path, "skills") else {
        return error;
    };
    let Some(root) = find_nth_direct_tuple(input, 0..input.len(), 0) else {
        return error;
    };
    let Some(definition) = find_direct_container(input, root, "skills", b'[')
        .and_then(|skills| find_nth_direct_tuple(input, skills, definition_index))
    else {
        return error;
    };
    error.object_id = find_direct_string_field(input, definition.clone(), "id");
    let Some(action_index) = path_index(path, "steps") else {
        return error;
    };
    let Some(action) = find_direct_container(input, definition, "steps", b'[')
        .and_then(|steps| find_nth_direct_tuple(input, steps, action_index))
    else {
        return error;
    };
    error.action_id = find_direct_string_field(input, action.clone(), "id");
    if let Some(effect_index) = path_index(path, "effects") {
        error.effect_id = find_direct_container(input, action, "effects", b'[')
            .and_then(|effects| find_nth_direct_tuple(input, effects, effect_index))
            .and_then(|effect| find_direct_string_field(input, effect, "id"));
    }
    error
}

#[derive(Clone, Copy)]
enum LexToken<'a> {
    Ident(&'a str),
    String(usize, usize),
    Open(u8, usize),
    Close(u8),
    Colon,
    Other,
}

struct Lexer<'a> {
    input: &'a str,
    index: usize,
    end: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str, range: std::ops::Range<usize>) -> Self {
        Self {
            input,
            index: range.start,
            end: range.end.min(input.len()),
        }
    }

    fn next(&mut self) -> Option<LexToken<'a>> {
        let bytes = self.input.as_bytes();
        while self.index < self.end {
            let start = self.index;
            match bytes[start] {
                byte if byte.is_ascii_whitespace() => self.index += 1,
                b'/' if bytes.get(start + 1) == Some(&b'/') => {
                    self.index += 2;
                    while self.index < self.end && bytes[self.index] != b'\n' {
                        self.index += 1;
                    }
                }
                b'/' if bytes.get(start + 1) == Some(&b'*') => {
                    self.index += 2;
                    let mut depth = 1_usize;
                    while self.index < self.end && depth != 0 {
                        if bytes.get(self.index..self.index + 2) == Some(b"/*") {
                            depth = depth.saturating_add(1);
                            self.index += 2;
                        } else if bytes.get(self.index..self.index + 2) == Some(b"*/") {
                            depth -= 1;
                            self.index += 2;
                        } else {
                            self.index += 1;
                        }
                    }
                }
                b'r' if matches!(bytes.get(start + 1), Some(b'#' | b'"')) => {
                    let mut quote = start + 1;
                    while bytes.get(quote) == Some(&b'#') {
                        quote += 1;
                    }
                    if bytes.get(quote) != Some(&b'"') {
                        self.index = quote;
                        continue;
                    }
                    let hashes = quote - start - 1;
                    self.index = quote + 1;
                    while self.index < self.end {
                        if bytes[self.index] == b'"'
                            && bytes
                                .get(self.index + 1..self.index + 1 + hashes)
                                .is_some_and(|tail| tail.iter().all(|byte| *byte == b'#'))
                        {
                            self.index += 1 + hashes;
                            return Some(LexToken::String(start, self.index));
                        }
                        self.index += 1;
                    }
                    return None;
                }
                b'"' => {
                    self.index += 1;
                    while self.index < self.end {
                        let byte = bytes[self.index];
                        self.index += 1;
                        if byte == b'"' {
                            return Some(LexToken::String(start, self.index));
                        }
                        if byte == b'\\' && self.index < self.end {
                            self.index += 1;
                        }
                    }
                    return None;
                }
                b'\'' => {
                    self.index += 1;
                    while self.index < self.end {
                        let byte = bytes[self.index];
                        self.index += 1;
                        if byte == b'\'' {
                            break;
                        }
                        if byte == b'\\' && self.index < self.end {
                            self.index += 1;
                        }
                    }
                    return Some(LexToken::Other);
                }
                byte if byte.is_ascii_alphabetic() || byte == b'_' => {
                    self.index += 1;
                    while self.index < self.end
                        && bytes
                            .get(self.index)
                            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                    {
                        self.index += 1;
                    }
                    return Some(LexToken::Ident(&self.input[start..self.index]));
                }
                b'(' | b'[' | b'{' => {
                    self.index += 1;
                    return Some(LexToken::Open(bytes[start], start));
                }
                b')' | b']' | b'}' => {
                    self.index += 1;
                    return Some(LexToken::Close(bytes[start]));
                }
                b':' => {
                    self.index += 1;
                    return Some(LexToken::Colon);
                }
                _ => {
                    self.index += 1;
                    return Some(LexToken::Other);
                }
            }
        }
        None
    }
}

fn matching_close(input: &str, opener_at: usize, end: usize) -> Option<usize> {
    let opener = *input.as_bytes().get(opener_at)?;
    let expected = match opener {
        b'(' => b')',
        b'[' => b']',
        b'{' => b'}',
        _ => return None,
    };
    let mut lexer = Lexer::new(input, opener_at + 1..end);
    let mut depth = 1_usize;
    while let Some(token) = lexer.next() {
        match token {
            LexToken::Open(_, _) => depth = depth.checked_add(1)?,
            LexToken::Close(close) => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return (close == expected).then_some(lexer.index - 1);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_direct_string_field(
    input: &str,
    range: std::ops::Range<usize>,
    field: &str,
) -> Option<String> {
    let mut lexer = Lexer::new(input, range);
    let mut depth = 0_usize;
    while let Some(token) = lexer.next() {
        match token {
            LexToken::Open(_, _) => depth = depth.checked_add(1)?,
            LexToken::Close(_) => depth = depth.checked_sub(1)?,
            LexToken::Ident(value) if depth == 0 && value == field => {
                if !matches!(lexer.next(), Some(LexToken::Colon)) {
                    continue;
                }
                let Some(LexToken::String(start, end)) = lexer.next() else {
                    continue;
                };
                return ron::de::from_str(&input[start..end]).ok();
            }
            _ => {}
        }
    }
    None
}

fn find_direct_container(
    input: &str,
    range: std::ops::Range<usize>,
    field: &str,
    opener: u8,
) -> Option<std::ops::Range<usize>> {
    let range_end = range.end;
    let mut lexer = Lexer::new(input, range);
    let mut depth = 0_usize;
    while let Some(token) = lexer.next() {
        match token {
            LexToken::Open(_, _) => depth = depth.checked_add(1)?,
            LexToken::Close(_) => depth = depth.checked_sub(1)?,
            LexToken::Ident(value) if depth == 0 && value == field => {
                if !matches!(lexer.next(), Some(LexToken::Colon)) {
                    continue;
                }
                let Some(LexToken::Open(actual, at)) = lexer.next() else {
                    continue;
                };
                if actual != opener {
                    continue;
                }
                return matching_close(input, at, range_end).map(|close| at + 1..close);
            }
            _ => {}
        }
    }
    None
}

fn find_nth_direct_tuple(
    input: &str,
    range: std::ops::Range<usize>,
    ordinal: usize,
) -> Option<std::ops::Range<usize>> {
    let range_end = range.end;
    let mut lexer = Lexer::new(input, range);
    let mut depth = 0_usize;
    let mut found = 0_usize;
    while let Some(token) = lexer.next() {
        match token {
            LexToken::Open(b'(', at) if depth == 0 => {
                if found == ordinal {
                    return matching_close(input, at, range_end).map(|close| at + 1..close);
                }
                found = found.checked_add(1)?;
                depth = 1;
            }
            LexToken::Open(_, _) => depth = depth.checked_add(1)?,
            LexToken::Close(_) => depth = depth.checked_sub(1)?,
            _ => {}
        }
    }
    None
}

fn parse_value_expr_bounded<T: serde::de::DeserializeOwned>(
    source_file: &str,
    input: &str,
    limits: &crate::game::value_expr::ExprParseLimits,
    max_nesting: usize,
) -> Result<T, super::live_ron::LiveRonError> {
    let limits = limits.validate().map_err(|cause| {
        super::live_ron::LiveRonError::validation(
            source_file,
            "skills",
            None::<String>,
            Some("value_expr_limits"),
            cause,
        )
        .with_phase(super::live_ron::LiveRonPhase::Structure)
        .with_code("value_expr_budget_exceeded")
    })?;
    crate::game::value_expr::validate_expression_input(input, limits.max_input_bytes, max_nesting)
        .map_err(|error| {
            super::live_ron::LiveRonError::validation(
                source_file,
                "skills",
                None::<String>,
                Some(format!("byte[{}]", error.byte_offset)),
                error.cause,
            )
            .with_phase(super::live_ron::LiveRonPhase::Structure)
            .with_code("value_expr_input_limit")
        })?;
    crate::game::value_expr::with_parse_limits(limits, || {
        super::live_ron::parse_live_ron(source_file, "skills", input)
    })
    .map_err(classify_value_expr_parse_error)
}

fn classify_value_expr_parse_error(
    mut error: super::live_ron::LiveRonError,
) -> super::live_ron::LiveRonError {
    error.code = if error
        .cause
        .contains("value expression depth limit exceeded")
    {
        "value_expr_depth_limit"
    } else if error.cause.contains("value expression node limit exceeded") {
        "value_expr_node_limit"
    } else if error
        .cause
        .contains("authored aggregate member limit exceeded")
    {
        "value_expr_member_limit"
    } else if error.cause.contains("missing field `rounding`") {
        "value_expr_missing_rounding"
    } else {
        return error;
    };
    error
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillDatabase {
    #[serde(skip)]
    flow_plans: OnceLock<Vec<std::sync::Arc<crate::game::ability::flow::CompiledSkillFlow>>>,
    #[serde(default)]
    pub range_presets: Vec<SkillRangePresetDef>,
    pub skills: Vec<SkillDef>,

    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
}

impl SkillDatabase {
    pub fn new(skills: Vec<SkillDef>) -> Self {
        let mut database = Self {
            range_presets: vec![],
            skills,
            by_id: OnceLock::new(),
            flow_plans: OnceLock::new(),
        };
        database.bind_sources("typed:skills");
        database
            .validate_contract()
            .expect("invalid Skill authoring contract");
        database
    }

    pub(crate) fn from_bound_effective_plan(
        skill: SkillDef,
        flow: std::sync::Arc<crate::game::ability::flow::CompiledSkillFlow>,
        canonical: &SkillDatabase,
        single_recipe_sites: &[EffectSite],
    ) -> Result<Self, super::DataValidationError> {
        validate_skill_contracts_with_context(
            std::slice::from_ref(&skill),
            &canonical.skills,
            single_recipe_sites,
        )?;
        if !flow.matches_source(skill.flow.as_ref(), &skill.steps) {
            return Err(super::DataValidationError::at(
                skill.id.as_str(),
                "flow",
                "effective source/flow mismatch",
            ));
        }
        let plans = OnceLock::new();
        let _ = plans.set(vec![flow]);
        Ok(Self {
            range_presets: Vec::new(),
            skills: vec![skill],
            by_id: OnceLock::new(),
            flow_plans: plans,
        })
    }
    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id
            .get_or_init(|| build_string_index(&self.skills, "skill id", |skill| &skill.id))
    }

    pub(crate) fn validate_indexes(&self) {
        // Index validation must not repeat charged compilation through an
        // infallible expect. Public raw publication compiles fallibly, and the
        // amount catalog calls compiled_flow again for typed candidate changes.
        validate_skill_contracts(&self.skills).expect("invalid Skill authoring contract");
        let _ = self.by_id();
    }

    pub(crate) fn validate_contract(&self) -> Result<(), crate::game::data::DataValidationError> {
        validate_skill_contracts(&self.skills)?;
        let mut plans = Vec::with_capacity(self.skills.len());
        for (index, skill) in self.skills.iter().enumerate() {
            let plan = crate::game::ability::flow::CompiledSkillFlow::compile(
                skill.flow.as_ref(),
                &skill.steps,
            )
            .map_err(|error| {
                super::DataValidationError::at(
                    skill.id.as_str(),
                    format!("skills[{index}].{}", error.path),
                    error.cause,
                )
            })?;
            if plan
                .authored_delay_bound_ms
                .checked_add(u64::from(skill.focus_time_ms))
                .is_none()
            {
                return Err(super::DataValidationError::at(
                    skill.id.as_str(),
                    format!("skills[{index}].flow"),
                    "focus plus flow horizon overflows time",
                ));
            }
            plans.push(std::sync::Arc::new(plan));
        }
        let _ = self.flow_plans.set(plans);
        Ok(())
    }

    pub fn compiled_flow(
        &self,
        id: &str,
    ) -> Result<std::sync::Arc<crate::game::ability::flow::CompiledSkillFlow>, String> {
        let index = *self
            .by_id()
            .get(id)
            .ok_or_else(|| format!("unknown skill '{id}'"))?;
        let skill = &self.skills[index];
        if let Some(plan) = self
            .flow_plans
            .get()
            .and_then(|plans| plans.get(index))
            .filter(|plan| plan.matches_source(skill.flow.as_ref(), &skill.steps))
        {
            return Ok(plan.clone());
        }
        // Typed candidate mutation may replace public authoring fields before
        // validation. Never execute an index plan from a different definition.
        crate::game::ability::flow::CompiledSkillFlow::compile(skill.flow.as_ref(), &skill.steps)
            .map(std::sync::Arc::new)
            .map_err(|error| format!("{}: {}", error.path, error.cause))
    }

    pub(crate) fn validate_buff_references(
        &self,
        buff_data: &BuffDatabase,
    ) -> Result<(), crate::game::data::DataValidationError> {
        validate_skill_buff_references(&self.skills, buff_data)
    }

    pub(crate) fn validate_resource_references(
        &self,
        catalog: &crate::game::resource::ResourceCatalog,
    ) -> Result<(), crate::game::data::DataValidationError> {
        for (skill_index, skill) in self.skills.iter().enumerate() {
            let charge = |work, site| {
                crate::authoring::control::charge(work, site).map_err(|error| {
                    super::DataValidationError::at(skill.id.as_str(), "steps", error.code())
                })
            };
            charge(1, "resource_reference_skill")?;
            for (step_index, step) in skill.steps.iter().enumerate() {
                charge(1, "resource_reference_step")?;
                for (effect_index, effect) in step.effects.iter().enumerate() {
                    charge(1, "resource_reference_effect")?;
                    if let SkillEffectDef::Resources { operations } = &effect.op {
                        for (index, operation) in operations.iter().enumerate() {
                            charge(
                                catalog.resources.len() as u64 + 1,
                                "resource_reference_lookup",
                            )?;
                            let path = format!("skills[{skill_index}].steps[{step_index}].effects[{effect_index}].op.Resources.operations[{index}]");
                            let definition = catalog
                                .resources
                                .iter()
                                .find(|def| def.id == operation.resource)
                                .ok_or_else(|| {
                                    super::DataValidationError::at(
                                        skill.id.as_str(),
                                        format!("{path}.resource"),
                                        format!(
                                            "unknown resource '{}'",
                                            operation.resource.as_str()
                                        ),
                                    )
                                })?;
                            definition
                                .validate_selector(operation.owner)
                                .map_err(|error| {
                                    super::DataValidationError::at(
                                        skill.id.as_str(),
                                        format!("{path}.owner"),
                                        error.to_string(),
                                    )
                                })?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_by_id(&self, id: impl AsRef<str>) -> Option<&SkillDef> {
        self.by_id()
            .get(id.as_ref())
            .and_then(|&index| self.skills.get(index))
    }

    pub fn from_ron(source_file: &str, input: &str) -> Result<Self, super::live_ron::LiveRonError> {
        let limits = crate::game::value_expr::current_parse_limits();
        let raw = RawSkillDatabase::from_ron_with_limits(source_file, input, &limits, 128)?;
        let mut database = raw.try_into_unvalidated_database().map_err(|error| {
            super::live_ron::LiveRonError::validation(
                source_file,
                "skills",
                error.object_id,
                error.field_path,
                error.cause,
            )
        })?;
        database.bind_sources(source_file);
        database
            .validate_contract()
            .map_err(|error| database.live_error(source_file, error))?;
        Ok(database)
    }

    pub(crate) fn live_error(
        &self,
        source_file: &str,
        error: super::DataValidationError,
    ) -> super::live_ron::LiveRonError {
        let mut result = super::live_ron::LiveRonError::validation(
            source_file,
            "skills",
            error.object_id,
            error.field_path,
            error.cause,
        );
        if let Some(path) = &result.field_path {
            if let Some(skill) = path_index(path, "skills").and_then(|i| self.skills.get(i)) {
                result.object_id = Some(skill.id.to_string());
                if let Some(step) = path_index(path, "steps").and_then(|i| skill.steps.get(i)) {
                    result.action_id = Some(step.id.clone());
                    result.effect_id = path_index(path, "effects")
                        .and_then(|i| step.effects.get(i))
                        .map(|effect| effect.id.clone());
                }
            }
        }
        result
    }

    pub(crate) fn bind_sources(&mut self, source_file: &str) {
        self.bind_definition_sources(|index, _| {
            (source_file.to_owned(), format!("skills[{index}]"))
        });
    }

    pub(crate) fn bind_definition_sources(
        &mut self,
        mut source: impl FnMut(usize, &crate::game::ability::SkillId) -> (String, String),
    ) {
        self.flow_plans = OnceLock::new();
        for (skill_index, skill) in self.skills.iter_mut().enumerate() {
            let (source_file, field_path) = source(skill_index, &skill.id);
            for (step_index, step) in skill.steps.iter_mut().enumerate() {
                for (effect_index, effect) in step.effects.iter_mut().enumerate() {
                    effect.source = Some(std::sync::Arc::new(EffectSource {
                        source_file: source_file.clone(),
                        site: EffectSite {
                            action: ActionSite {
                                definition_id: skill.id.clone(),
                                action_id: step.id.clone(),
                            },
                            effect_id: effect.id.clone(),
                        },
                        field_path: format!(
                            "{field_path}.steps[{step_index}].effects[{effect_index}]"
                        ),
                        origin_definition_id: match &effect.origin {
                            EffectOrigin::Original => skill.id.clone(),
                            EffectOrigin::Added {
                                source_definition_id,
                            } => source_definition_id.clone(),
                        },
                        origin: effect.origin.clone(),
                    }));
                }
            }
        }
    }

    pub fn effect(&self, site: &EffectSite) -> Option<&AuthoredSkillEffect> {
        self.get_by_id(&site.action.definition_id)?
            .steps
            .iter()
            .find(|step| step.id == site.action.action_id)?
            .effects
            .iter()
            .find(|effect| effect.id == site.effect_id)
    }

    pub fn matches_effects(
        &self,
        definition_id: &crate::game::ability::SkillId,
        selection: &EffectSelection,
    ) -> bool {
        let Some(skill) = self.get_by_id(definition_id) else {
            return false;
        };
        let exists = |predicate: &crate::game::ability::authoring::EffectPredicate| {
            skill
                .steps
                .iter()
                .flat_map(|step| &step.effects)
                .any(|effect| predicate.matches(effect))
        };
        match selection {
            EffectSelection::SameEffect(predicate) => exists(predicate),
            EffectSelection::EachExists(predicates) => predicates.iter().all(exists),
        }
    }

    /// Edit a candidate definition, before passing it through validated publication.
    /// An action-owned property is never silently applied to one effect.
    pub fn set_effect_amount(
        &mut self,
        site: &EffectSite,
        property: EffectProperty,
        amount: i32,
    ) -> Result<(), AuthoringDiagnostic> {
        let skill_index = self
            .skills
            .iter()
            .position(|skill| skill.id == site.action.definition_id);
        let step_index = skill_index.and_then(|i| {
            self.skills[i]
                .steps
                .iter()
                .position(|step| step.id == site.action.action_id)
        });
        let effect_index = skill_index.zip(step_index).and_then(|(i, j)| {
            self.skills[i].steps[j]
                .effects
                .iter()
                .position(|effect| effect.id == site.effect_id)
        });
        let diagnostic = |cause: &str| {
            let source = self
                .effect(site)
                .and_then(|effect| effect.source().cloned());
            let affected_effects = skill_index
                .zip(step_index)
                .map(|(i, j)| {
                    self.skills[i].steps[j]
                        .effects
                        .iter()
                        .map(|effect| EffectSite {
                            action: site.action.clone(),
                            effect_id: effect.id.clone(),
                        })
                        .collect()
                })
                .unwrap_or_default();
            AuthoringDiagnostic {
                source,
                site: site.clone(),
                property,
                affected_effects,
                cause: cause.to_owned(),
            }
        };
        let Some(((i, j), k)) = skill_index.zip(step_index).zip(effect_index) else {
            return Err(diagnostic("unknown definition/action/effect address"));
        };
        if property != EffectProperty::Amount {
            return Err(diagnostic(
                "unsupported effect edit; target/delivery are shared action properties",
            ));
        }
        let step = &self.skills[i].steps[j];
        if effect_support(&step.effects[k].op, &step.delivery).is_none() {
            return Err(diagnostic(
                "unsupported operation/delivery for literal effect editing",
            ));
        }
        if matches!(step.effects[k].op, SkillEffectDef::Damage { .. }) && amount < 0 {
            return Err(diagnostic("Damage amount must not be negative"));
        }
        match &mut self.skills[i].steps[j].effects[k].op {
            SkillEffectDef::Damage { amount: value, .. }
            | SkillEffectDef::Heal { amount: value } => *value = amount,
            _ => unreachable!("effect_support restricts editing to typed literal operations"),
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for SkillDatabase {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawSkillDatabase::deserialize(deserializer)?;
        let database = raw
            .try_into_unvalidated_database()
            .map_err(|error| D::Error::custom(error.cause))?;
        database
            .validate_contract()
            .map_err(|error| D::Error::custom(format!("invalid Skill data: {error:?}")))?;
        Ok(database)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename = "SkillDatabase")]
pub(crate) struct RawSkillDatabase {
    #[serde(default)]
    pub(crate) range_presets: Vec<SkillRangePresetDef>,
    pub(crate) skills: Vec<RawSkillDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename = "SkillDef")]
pub(crate) struct RawSkillDef {
    #[serde(default)]
    action_policy: Option<std::sync::Arc<crate::game::ability::action::ActionPolicy>>,
    #[serde(default)]
    resonance_admission: Option<crate::game::ability::resonance::ResonanceAdmission>,
    use_lifetime: crate::game::ability::SkillUseLifetime,
    #[serde(default)]
    flow: Option<std::sync::Arc<crate::game::ability::flow::SkillFlowDef>>,
    pub(crate) id: crate::game::ability::SkillId,
    name: String,
    kind: SkillKind,
    cast_targeting: RawSkillCastTargetingDef,
    focus_time_ms: u32,
    #[serde(default)]
    focus_permissions: FocusPermissions,
    steps: Vec<RawSkillStepDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum RawSkillCastTargetingDef {
    Explicit {
        selection: SkillCastSelection,
        target: SkillTarget,
        reach: SkillTargetReach,
        #[serde(default)]
        target_range: Option<TileRangePattern>,
        #[serde(default)]
        target_range_preset: Option<String>,
        air_capable: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename = "SkillStepDef")]
struct RawSkillStepDef {
    id: String,
    delay_ms: u32,
    target: SkillTarget,
    #[serde(default)]
    targeting: StepTargetingMode,
    #[serde(default)]
    when: SkillStepCondition,
    #[serde(default)]
    repeat: SkillStepRepeat,
    delivery: DeliveryDef,
    effects: Vec<AuthoredSkillEffect>,
    #[serde(default)]
    presentation: SkillPresentationDef,
}

impl RawSkillDatabase {
    pub(crate) fn from_ron(
        source_file: &str,
        input: &str,
    ) -> Result<Self, super::live_ron::LiveRonError> {
        Self::from_ron_with_limits(
            source_file,
            input,
            &crate::game::value_expr::current_parse_limits(),
            128,
        )
    }

    pub(crate) fn from_ron_with_limits(
        source_file: &str,
        input: &str,
        limits: &crate::game::value_expr::ExprParseLimits,
        max_nesting: usize,
    ) -> Result<Self, super::live_ron::LiveRonError> {
        // The extension is an explicit format discriminator; malformed graph IR
        // must never fall back to the ordinary skill catalog parser.
        let lowered;
        let input = if source_file.ends_with(".skill-graph.ron") {
            let graph: crate::authoring::node_graph::PortableGraph =
                parse_value_expr_bounded(source_file, input, limits, max_nesting)?;
            lowered = graph.native_ron()
                .map_err(|cause| super::live_ron::LiveRonError::validation(
                    source_file, "skills", None::<String>, Some("graph"), cause,
                ).with_phase(super::live_ron::LiveRonPhase::Compile)
                 .with_code("graph_artifact_invalid"))?;
            lowered.as_str()
        } else { input };
        parse_value_expr_bounded(source_file, input, limits, max_nesting)
            .map_err(|error| enrich_parse_error(input, error))
    }

    pub(crate) fn try_into_unvalidated_database(
        self,
    ) -> Result<SkillDatabase, super::DataValidationError> {
        let mut preset_index = HashMap::new();
        for (index, preset) in self.range_presets.iter().enumerate() {
            if preset_index.insert(preset.id.clone(), index).is_some() {
                return Err(super::DataValidationError::at(
                    &preset.id,
                    format!("range_presets[{index}].id"),
                    format!("duplicate skill range preset id '{}'", preset.id),
                ));
            }
        }
        for (index, preset) in self.range_presets.iter().enumerate() {
            preset.range.validate().map_err(|error| {
                super::DataValidationError::at(
                    &preset.id,
                    format!("range_presets[{index}].range"),
                    format!(
                        "skill range preset '{}' has invalid defense_tile_range: {error}",
                        preset.id
                    ),
                )
            })?;
        }

        let skills = self
            .skills
            .into_iter()
            .enumerate()
            .map(|(index, skill)| {
                let id = skill.id.to_string();
                skill
                    .try_into_skill_def(&self.range_presets, &preset_index)
                    .map_err(|cause| {
                        super::DataValidationError::at(
                            id,
                            format!("skills[{index}].cast_targeting"),
                            cause,
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut database = SkillDatabase {
            range_presets: self.range_presets,
            skills,
            by_id: OnceLock::new(),
            flow_plans: OnceLock::new(),
        };
        database.bind_sources("data/skills/base.ron");
        Ok(database)
    }
}

impl RawSkillDef {
    fn try_into_skill_def(
        self,
        presets: &[SkillRangePresetDef],
        preset_index: &HashMap<String, usize>,
    ) -> Result<SkillDef, String> {
        Ok(SkillDef {
            action_policy: self.action_policy,
            resonance_admission: self.resonance_admission,
            use_lifetime: self.use_lifetime,
            flow: self.flow,
            id: self.id,
            name: self.name,
            kind: self.kind,
            cast_targeting: self
                .cast_targeting
                .try_into_cast_targeting(presets, preset_index)?,
            focus_time_ms: self.focus_time_ms,
            focus_permissions: self.focus_permissions,
            steps: self
                .steps
                .into_iter()
                .map(|step| step.try_into_step(presets, preset_index))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl RawSkillCastTargetingDef {
    fn try_into_cast_targeting(
        self,
        presets: &[SkillRangePresetDef],
        preset_index: &HashMap<String, usize>,
    ) -> Result<SkillCastTargetingDef, String> {
        Ok(match self {
            RawSkillCastTargetingDef::Explicit {
                selection,
                target,
                reach,
                target_range,
                target_range_preset,
                air_capable,
            } => SkillCastTargetingDef::Explicit {
                selection,
                target,
                reach,
                target_range: try_resolve_skill_range_source(
                    "explicit cast_targeting",
                    target_range,
                    target_range_preset,
                    presets,
                    preset_index,
                )?,
                air_capable,
            },
        })
    }
}

impl RawSkillStepDef {
    fn try_into_step(
        self,
        _presets: &[SkillRangePresetDef],
        _preset_index: &HashMap<String, usize>,
    ) -> Result<SkillStepDef, String> {
        let step_id = self.id;
        Ok(SkillStepDef {
            id: step_id.clone(),
            delay_ms: self.delay_ms,
            target: self.target,
            targeting: self.targeting,
            when: self.when,
            repeat: self.repeat,
            delivery: self.delivery,
            effects: self.effects,
            presentation: self.presentation,
        })
    }
}

fn try_resolve_skill_range_source(
    context: &str,
    inline_range: Option<TileRangePattern>,
    preset_id: Option<String>,
    presets: &[SkillRangePresetDef],
    preset_index: &HashMap<String, usize>,
) -> Result<Option<TileRangePattern>, String> {
    match (inline_range, preset_id) {
        (Some(_), Some(preset_id)) => Err(format!(
            "{context} defines both target_range and target_range_preset '{preset_id}'"
        )),
        (Some(range), None) => Ok(Some(range)),
        (None, Some(preset_id)) => {
            let Some(index) = preset_index.get(&preset_id) else {
                return Err(format!(
                    "{context} references unknown target_range_preset '{preset_id}'"
                ));
            };
            Ok(Some(presets[*index].range.clone()))
        }
        (None, None) => Ok(None),
    }
}

fn validate_skill_buff_references(
    skills: &[SkillDef],
    buff_data: &BuffDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    for (skill_index, skill) in skills.iter().enumerate() {
        let skill_id = skill.id.to_string();
        for (step_index, step) in skill.steps.iter().enumerate() {
            let step_path = format!("skills[{skill_index}].steps[{step_index}]");
            let validate_buff = |field_path: String,
                                 buff_id: &str|
             -> Result<(), crate::game::data::DataValidationError> {
                if buff_data.contains_name(buff_id) {
                    Ok(())
                } else {
                    Err(crate::game::data::DataValidationError::at(
                        &skill_id,
                        field_path,
                        format!("references unknown buff '{buff_id}'"),
                    ))
                }
            };
            match &step.when {
                crate::game::ability::SkillStepCondition::Always
                | crate::game::ability::SkillStepCondition::IfPreviousStepDealtDamage => {}
                crate::game::ability::SkillStepCondition::IfCasterHasBuff { buff_id, .. } => {
                    validate_buff(format!("{step_path}.when.IfCasterHasBuff.buff_id"), buff_id)?;
                }
            }

            if let crate::game::ability::SkillStepRepeat::ByBuffStacks { buff_id, .. } =
                &step.repeat
            {
                validate_buff(format!("{step_path}.repeat.ByBuffStacks.buff_id"), buff_id)?;
            }

            for (effect_index, effect) in step.effects.iter().enumerate() {
                if let crate::game::ability::SkillEffectDef::ApplyBuff { buff_id, .. } = &effect.op
                {
                    validate_buff(
                        format!("{step_path}.effects[{effect_index}].op.ApplyBuff.buff_id"),
                        buff_id,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn validate_skill_contracts(
    skills: &[SkillDef],
) -> Result<(), crate::game::data::DataValidationError> {
    validate_skill_contracts_with_context(skills, skills, &[])
}
fn validate_skill_contracts_with_context(
    skills: &[SkillDef],
    references: &[SkillDef],
    single_recipe_sites: &[EffectSite],
) -> Result<(), crate::game::data::DataValidationError> {
    let mut definition_ids = HashSet::new();
    for (skill_index, skill) in skills.iter().enumerate() {
        let skill_id = skill.id.to_string();
        let skill_path = format!("skills[{skill_index}]");
        let error = |path: String, cause: String| {
            crate::game::data::DataValidationError::at(&skill_id, path, cause)
        };
        if skill.id.as_str().trim().is_empty() || !definition_ids.insert(&skill.id) {
            return Err(error(
                format!("{skill_path}.id"),
                format!("empty or duplicate skill id '{}'", skill.id),
            ));
        }
        if let Some(policy) = &skill.action_policy { policy.validate(skill).map_err(|cause| error(format!("{skill_path}.action_policy"),cause))?; }
        if let Some(policy) = skill.resonance_admission {
            policy.validate().map_err(|cause| error(format!("{skill_path}.resonance_admission"), cause.into()))?;
        }
        if skill.steps.is_empty() {
            return Err(error(
                format!("{skill_path}.steps"),
                "must define at least one step".to_string(),
            ));
        }
        let crate::game::ability::SkillCastTargetingDef::Explicit {
            selection,
            reach,
            target,
            target_range,
            ..
        } = &skill.cast_targeting;
        if !matches!(
            (selection, target),
            (SkillCastSelection::SelfUnit, SkillTarget::SelfUnit)
                | (
                    SkillCastSelection::AutomaticUnit | SkillCastSelection::PlayerUnit,
                    SkillTarget::EnemySingle { .. } | SkillTarget::AllySingle
                )
                | (SkillCastSelection::PlayerTile | SkillCastSelection::PlayerTilePair | SkillCastSelection::PlayerDirection, SkillTarget::CastTarget)
        ) {
            return Err(error(
                format!("{skill_path}.cast_targeting.selection"),
                "selection is incompatible with cast target semantics".to_string(),
            ));
        }
        if *selection==SkillCastSelection::PlayerDirection && !skill.steps.iter().flat_map(|s| &s.effects).any(|e| matches!(e.op,SkillEffectDef::StatusStrike{..})) {return Err(error(format!("{skill_path}.cast_targeting.selection"),"Direction selection requires an explicit directional execution profile".into()));}
        validate_skill_target_reach(*reach, target_range.as_ref())
            .map_err(|cause| error(format!("{skill_path}.cast_targeting"), cause))?;
        if matches!(reach, SkillTargetReach::Global) && matches!(target, SkillTarget::SelfUnit) {
            return Err(error(
                format!("{skill_path}.cast_targeting.target"),
                "global reach requires an explicit non-self target".to_string(),
            ));
        }
        if matches!(
            target,
            SkillTarget::EnemySingle { .. } | SkillTarget::AllySingle
        ) && matches!(reach, SkillTargetReach::Pattern)
            && target_range.is_none()
        {
            return Err(error(
                format!("{skill_path}.cast_targeting.target_range"),
                "pattern reach for a single unit target requires target_range".to_string(),
            ));
        }
        if matches!(
            selection,
            SkillCastSelection::PlayerUnit | SkillCastSelection::PlayerTile | SkillCastSelection::PlayerTilePair | SkillCastSelection::PlayerDirection
        ) && skill
            .steps
            .iter()
            .any(|step| matches!(step.targeting, StepTargetingMode::RetargetOnStep))
        {
            return Err(error(
                format!("{skill_path}.steps"),
                "player-selected skills cannot use RetargetOnStep".to_string(),
            ));
        }
        let mut step_ids = HashSet::new();
        let mut segmented_pools = HashSet::new();

        for (step_index, step) in skill.steps.iter().enumerate() {
            let step_path = format!("{skill_path}.steps[{step_index}]");
            if step.id.trim().is_empty() || !step_ids.insert(step.id.clone()) {
                return Err(error(
                    format!("{step_path}.id"),
                    format!("duplicate step id '{}'", step.id),
                ));
            }

            if matches!(step.target, SkillTarget::AllySingle)
                && !matches!(step.delivery, DeliveryDef::Instant)
            {
                return Err(error(format!("{step_path}.delivery"), "AllySingle currently requires Instant delivery; ally areas use CastTarget with TileArea Allies".into()));
            }
            let mut has_damage_effect = false;
            let mut has_modify_damage_effect = false;
            let mut effect_ids = HashSet::new();
            for (effect_index, effect) in step.effects.iter().enumerate() {
                let effect_path = format!("{step_path}.effects[{effect_index}]");
                if effect.id.trim().is_empty() || !effect_ids.insert(&effect.id) {
                    return Err(error(
                        format!("{effect_path}.id"),
                        format!(
                            "empty or duplicate effect id '{}' in action '{}'",
                            effect.id, step.id
                        ),
                    ));
                }
                if let Some(damage_id) = &effect.attached_to {
                    let prior_damage = step.effects[..effect_index].iter().any(|e| {
                        &e.id == damage_id
                            && matches!(
                                e.op,
                                SkillEffectDef::Damage { .. } | SkillEffectDef::DamageExpr { .. }
                            )
                    });
                    let supported = matches!(
                        &effect.op,
                        SkillEffectDef::ApplyBuff { .. }
                            | SkillEffectDef::ModifyStats { .. }
                            | SkillEffectDef::InterruptCast
                            | SkillEffectDef::Pull { .. }
                            | SkillEffectDef::Sustained {
                                definition:
                                    crate::game::ability::sustained::SustainedEffectDef::Control { .. }
                                    | crate::game::ability::sustained::SustainedEffectDef::HitStatus { .. }
                            }
                    );
                    if !prior_damage || !supported {
                        return Err(error(format!("{effect_path}.attached_to"), "attachment requires an earlier damage effect in the same action and a supported target-local operation".into()));
                    }
                }
                if let EffectOrigin::Added {
                    source_definition_id,
                } = &effect.origin
                {
                    if !skills
                        .iter()
                        .any(|definition| &definition.id == source_definition_id)
                    {
                        return Err(error(
                            format!("{effect_path}.origin.Added.source_definition_id"),
                            format!(
                                "effect '{}' references unknown source definition '{}'",
                                effect.id, source_definition_id
                            ),
                        ));
                    }
                }
                match &effect.op {
                    SkillEffectDef::GuardCounter { program } => {
                        program.validate().map_err(|message|error(format!("{effect_path}.op.GuardCounter"),message))?;
                        let bound=references.iter().flat_map(|s|&s.steps).flat_map(|s|&s.effects).any(|e|matches!(&e.op,SkillEffectDef::Sustained{definition:crate::game::ability::sustained::SustainedEffectDef::Parry{counter_skill,guard_charge:Some(charge),..}} if counter_skill==&skill.id && charge.as_ref()==program.charge.as_ref()));
                        if !bound { return Err(error(effect_path.clone(),"Guard counter must match an authored parry's charge definition".into())); }

                        if skill.flow.is_some() || skill.steps.len()!=1 || step.effects.len()!=1 || step.delay_ms!=0 || skill.focus_time_ms!=0 || !matches!(step.delivery,DeliveryDef::Instant) || !matches!(step.target,SkillTarget::CastTarget) || !matches!(step.repeat,crate::game::ability::SkillStepRepeat::Once) || effect.attached_to.is_some() { return Err(error(effect_path.clone(),"Guard counter requires one immediate cast-target strike".into())); }
                    }
                    SkillEffectDef::StatusStrike { program } => {
                        program.validate().map_err(|message|error(format!("{effect_path}.op.StatusStrike"),message))?;
                        let selection=matches!(&skill.cast_targeting,crate::game::ability::SkillCastTargetingDef::Explicit{selection:crate::game::ability::SkillCastSelection::PlayerDirection,target:SkillTarget::CastTarget,reach:SkillTargetReach::Pattern,target_range:Some(r),..} if r==&program.range);
                        let immediate=matches!(skill.flow.as_deref(),Some(crate::game::ability::flow::SkillFlowDef::Action{step_id,wait:crate::game::ability::flow::FlowMilestone::Released,..}) if step_id==&step.id);
                        if !selection || !immediate || skill.resonance_admission!=Some(program.admission) || skill.steps.len()!=1 || step.effects.len()!=1 || step.delay_ms!=0 || skill.focus_time_ms!=0 || !matches!(skill.use_lifetime,crate::game::ability::SkillUseLifetime::Action) || !matches!(step.target,SkillTarget::SelfUnit) || !matches!(step.delivery,DeliveryDef::Instant) || !matches!(step.repeat,crate::game::ability::SkillStepRepeat::Once) || effect.attached_to.is_some(){return Err(error(effect_path.clone(),"Status strike requires atomic direction selection and one immediate transition strike".into()));}
                    }
                    SkillEffectDef::FusionPlacement { program } => {
                        program.validate().map_err(|message|error(format!("{effect_path}.op.FusionPlacement"),message))?;
                        let selection=matches!(&skill.cast_targeting,crate::game::ability::SkillCastTargetingDef::Explicit{selection:crate::game::ability::SkillCastSelection::PlayerTile,target:SkillTarget::CastTarget,reach:SkillTargetReach::Pattern,target_range:Some(r),..} if r==&program.selection_range);
                        let immediate=matches!(skill.flow.as_deref(),Some(crate::game::ability::flow::SkillFlowDef::Action{step_id,wait:crate::game::ability::flow::FlowMilestone::Released,..}) if step_id==&step.id);
                        if !selection || !immediate || skill.resonance_admission!=Some(program.admission) || skill.steps.len()!=1 || step.effects.len()!=1 || step.delay_ms!=0 || skill.focus_time_ms!=0 || !matches!(skill.use_lifetime,crate::game::ability::SkillUseLifetime::Action) || !matches!(step.target,SkillTarget::SelfUnit) || !matches!(step.delivery,DeliveryDef::Instant) || !matches!(step.repeat,crate::game::ability::SkillStepRepeat::Once) || effect.attached_to.is_some(){return Err(error(effect_path.clone(),"Fusion placement requires atomic tile selection and one immediate independent installation".into()));}
                    }
                    SkillEffectDef::PairedAreas { program } => {
                        program.validate().map_err(|message|error(format!("{effect_path}.op.PairedAreas"),message))?;
                        let selection=matches!(&skill.cast_targeting,crate::game::ability::SkillCastTargetingDef::Explicit{selection:crate::game::ability::SkillCastSelection::PlayerTilePair,target:SkillTarget::CastTarget,reach:SkillTargetReach::Pattern,target_range:Some(r),..} if r==&program.selection_range);
                        let immediate=matches!(skill.flow.as_deref(),Some(crate::game::ability::flow::SkillFlowDef::Action{step_id,wait:crate::game::ability::flow::FlowMilestone::Released,..}) if step_id==&step.id);
                        if !selection || !immediate || skill.resonance_admission!=Some(program.admission) || skill.steps.len()!=1 || step.effects.len()!=1 || step.delay_ms!=0 || skill.focus_time_ms!=0 || !matches!(skill.use_lifetime,crate::game::ability::SkillUseLifetime::Action) || !matches!(step.target,SkillTarget::SelfUnit) || !matches!(step.delivery,DeliveryDef::Instant) || !matches!(step.repeat,crate::game::ability::SkillStepRepeat::Once) || effect.attached_to.is_some(){return Err(error(effect_path.clone(),"Paired areas require atomic pair selection and one immediate independent installation".into()));}
                    }
                    SkillEffectDef::PreparedVolley { program } => {
                        program.validate().map_err(|message|error(format!("{effect_path}.op.PreparedVolley"),message))?;
                        let immediate_flow=matches!(skill.flow.as_deref(),Some(crate::game::ability::flow::SkillFlowDef::Action{step_id,wait:crate::game::ability::flow::FlowMilestone::Released,..}) if step_id==&step.id);
                        if skill.resonance_admission!=Some(program.admission)||skill.steps.len()!=1||step.effects.len()!=1||step.delay_ms!=0||skill.focus_time_ms!=0||!immediate_flow||skill.cast_selection()!=crate::game::ability::SkillCastSelection::SelfUnit
                            ||!matches!(step.target,SkillTarget::SelfUnit)||!matches!(step.delivery,DeliveryDef::Instant)||!matches!(step.repeat,crate::game::ability::SkillStepRepeat::Once)||effect.attached_to.is_some()||!skill.use_lifetime.owns_effect(&step.id,&effect.id){
                            return Err(error(effect_path.clone(),"Prepared volley requires its declared admission and one owned immediate self action".into()));
                        }
                    }
                    SkillEffectDef::InputPlan { program } => {
                        program.validate().map_err(|message|error(format!("{effect_path}.op.InputPlan"),message))?;
                        let immediate_flow=matches!(skill.flow.as_deref(),Some(crate::game::ability::flow::SkillFlowDef::Action{step_id,wait:crate::game::ability::flow::FlowMilestone::Released,..}) if step_id==&step.id);
                        if skill.resonance_admission!=Some(program.admission)||skill.steps.len()!=1||step.effects.len()!=1||step.delay_ms!=0||skill.focus_time_ms!=0||!immediate_flow||skill.cast_selection()!=crate::game::ability::SkillCastSelection::SelfUnit
                            ||!matches!(step.target,SkillTarget::SelfUnit)||!matches!(step.delivery,DeliveryDef::Instant)||!matches!(step.repeat,crate::game::ability::SkillStepRepeat::Once)||effect.attached_to.is_some()||!skill.use_lifetime.owns_effect(&step.id,&effect.id){
                            return Err(error(effect_path.clone(),"Input plan requires its declared admission and one owned immediate self action".into()));
                        }
                    }
                    SkillEffectDef::RelationAction { program } => {
                        program.validate().map_err(|message|error(format!("{effect_path}.op.RelationAction"),message))?;
                        let immediate_flow=matches!(skill.flow.as_deref(),Some(crate::game::ability::flow::SkillFlowDef::Action{step_id,wait:crate::game::ability::flow::FlowMilestone::Released,..}) if step_id==&step.id);
                        let selection=if let Some(range)=&program.selection_range {
                            matches!(&skill.cast_targeting,crate::game::ability::SkillCastTargetingDef::Explicit{selection:crate::game::ability::SkillCastSelection::PlayerTile,target:SkillTarget::CastTarget,reach:SkillTargetReach::Pattern,target_range:Some(r),air_capable:false} if r==range)
                        }else{skill.cast_selection()==crate::game::ability::SkillCastSelection::SelfUnit};
                        if skill.resonance_admission!=Some(program.admission)||skill.steps.len()!=1||step.effects.len()!=1||step.delay_ms!=0||skill.focus_time_ms!=0||!immediate_flow||!selection
                            ||!matches!(step.target,SkillTarget::SelfUnit)||!matches!(step.delivery,DeliveryDef::Instant)||!matches!(step.repeat,crate::game::ability::SkillStepRepeat::Once)||effect.attached_to.is_some()||!skill.use_lifetime.owns_effect(&step.id,&effect.id){
                            return Err(error(effect_path.clone(),"Relationship action requires its declared admission/selection and one owned immediate self action".into()));
                        }
                    }
                    SkillEffectDef::CounterChannel { program, root, entry } => {
                        program.validate().map_err(|message| error(format!("{effect_path}.op.CounterChannel"), message))?;
                        let e=program.entries.get(*entry).ok_or_else(||error(effect_path.clone(),"Unknown counter menu entry".into()))?;
                        let immediate_flow = matches!(skill.flow.as_deref(),Some(crate::game::ability::flow::SkillFlowDef::Action {step_id, wait:crate::game::ability::flow::FlowMilestone::Released,..}) if step_id==&step.id);
                        let declared_selection = matches!(&skill.cast_targeting, crate::game::ability::SkillCastTargetingDef::Explicit {
                            selection: crate::game::ability::SkillCastSelection::PlayerTile, target: SkillTarget::CastTarget, reach, target_range, air_capable: true
                        } if *reach == (if e.automatic { SkillTargetReach::Global } else { SkillTargetReach::Pattern }) && *target_range == e.selection_range);
                        if skill.id!=program.entry_skill_id(root,*entry) || skill.resonance_admission!=Some(e.admission)
                            || skill.steps.len()!=1 || step.effects.len()!=1 || step.delay_ms!=0 || skill.focus_time_ms!=0 || !immediate_flow || !declared_selection
                            || skill.cast_selection()!=crate::game::ability::SkillCastSelection::PlayerTile
                            || !matches!(step.target,SkillTarget::SelfUnit) || !matches!(step.delivery,DeliveryDef::Instant)
                            || !matches!(step.repeat,crate::game::ability::SkillStepRepeat::Once) || effect.attached_to.is_some()
                            || !skill.use_lifetime.owns_effect(&step.id,&effect.id) {
                            return Err(error(effect_path.clone(),"Counter channel requires its declared menu admission and one owned immediate self effect".into()));
                        }
                        for i in 0..program.entries.len() {
                            let id = program.entry_skill_id(root, i);
                            if !references.iter().find(|s| s.id == id).is_some_and(|s| s.steps.iter().flat_map(|step| &step.effects).any(|effect|
                                matches!(&effect.op, SkillEffectDef::CounterChannel { program: p, root: r, entry: e } if p == program && r == root && *e == i))) {
                                return Err(error(effect_path.clone(), format!("Counter menu is missing its matching declared entry '{id}'")));
                            }
                        }
                    }
                    SkillEffectDef::TileBurst { program } => {
                        program.validate().map_err(|message| error(format!("{effect_path}.op.TileBurst"), message))?;
                        let immediate_flow = matches!(skill.flow.as_deref(),Some(crate::game::ability::flow::SkillFlowDef::Action {step_id, wait:crate::game::ability::flow::FlowMilestone::Released,..}) if step_id==&step.id);
                        if skill.steps.len()!=1 || step.effects.len()!=1 || step.delay_ms!=0 || skill.focus_time_ms!=0 || !immediate_flow
                            || skill.cast_selection() != crate::game::ability::SkillCastSelection::SelfUnit
                            || !matches!(step.target, SkillTarget::SelfUnit)
                            || !matches!(step.delivery, DeliveryDef::Instant)
                            || !matches!(step.repeat, crate::game::ability::SkillStepRepeat::Once)
                            || effect.attached_to.is_some() {
                            return Err(error(format!("{effect_path}.op.TileBurst"), "Tile burst requires one immediate released SelfUnit Instant/Once action without damage attachment".into()));
                        }
                    }
                    SkillEffectDef::PresenceBurst { program } => {
                        program.validate().map_err(|message| error(format!("{effect_path}.op.PresenceBurst"), message))?;
                        let immediate_flow = matches!(skill.flow.as_deref(),Some(crate::game::ability::flow::SkillFlowDef::Action {step_id, wait:crate::game::ability::flow::FlowMilestone::Released,..}) if step_id==&step.id);
                        if skill.steps.len()!=1 || step.effects.len()!=1 || step.delay_ms!=0 || skill.focus_time_ms!=0 || !immediate_flow
                            || skill.cast_selection() != crate::game::ability::SkillCastSelection::SelfUnit
                            || !matches!(step.target, SkillTarget::SelfUnit)
                            || !matches!(step.delivery, DeliveryDef::Instant)
                            || !matches!(step.repeat, crate::game::ability::SkillStepRepeat::Once)
                            || effect.attached_to.is_some() {
                            return Err(error(format!("{effect_path}.op.PresenceBurst"), "Presence burst requires one immediate released SelfUnit Instant/Once action without damage attachment".into()));
                        }
                    }
                    SkillEffectDef::CreateObject { definition } => {
                        definition.validate().map_err(|message| {
                            error(format!("{effect_path}.op.CreateObject"), message)
                        })?;
                        let valid_anchor = match definition.anchor {
                            crate::game::ability::objects::ObjectPlacementAnchor::CastTile => {
                                skill.cast_selection()
                                    == crate::game::ability::SkillCastSelection::PlayerTile
                                    && matches!(step.target, SkillTarget::CastTarget)
                            }
                            crate::game::ability::objects::ObjectPlacementAnchor::SourceTile => {
                                skill.cast_selection()
                                    == crate::game::ability::SkillCastSelection::SelfUnit
                                    && matches!(step.target, SkillTarget::SelfUnit)
                            }
                        };
                        if !valid_anchor
                            || !matches!(step.delivery, DeliveryDef::Instant)
                            || !matches!(step.repeat, crate::game::ability::SkillStepRepeat::Once)
                            || skill.flow.is_some()
                            || effect.attached_to.is_some()
                        {
                            return Err(error(format!("{effect_path}.op.CreateObject"), "CreateObject requires Instant/Once without flow or damage attachment, and matching PlayerTile/CastTarget or SelfUnit/SourceTile anchor".into()));
                        }
                    }
                    SkillEffectDef::Damage { amount, .. } => {
                        has_damage_effect = true;
                        if *amount < 0 {
                            return Err(error(
                                format!("{step_path}.effects[{effect_index}].op.Damage.amount"),
                                format!("Damage amount must not be negative: {amount}"),
                            ));
                        }
                    }
                    SkillEffectDef::DamageExpr { .. } => {
                        has_damage_effect = true;
                    }
                    SkillEffectDef::ModifyDamage { .. } => {
                        has_modify_damage_effect = true;
                    }
                    SkillEffectDef::Resources { operations } => {
                        if operations.is_empty()
                            || operations.len()
                                > crate::game::resource::store::ResourceLimits::default().batch
                        {
                            return Err(error(
                                format!(
                                    "{step_path}.effects[{effect_index}].op.Resources.operations"
                                ),
                                "Resources requires a nonempty bounded operation batch".to_owned(),
                            ));
                        }
                    }
                    SkillEffectDef::Pull { max_distance_tiles } => {
                        if *max_distance_tiles == 0
                            || *max_distance_tiles > 64
                            || matches!(step.target, crate::game::ability::SkillTarget::SelfUnit)
                        {
                            return Err(error(format!("{effect_path}.op.Pull"), "Pull requires 1..64 tiles and an enemy target; runtime actor support is opponent combatants".into()));
                        }
                    }
                    SkillEffectDef::HealthCost { amount, .. } => {
                        if *amount == 0
                            || *amount > i32::MAX as u32
                            || !matches!(step.target, crate::game::ability::SkillTarget::SelfUnit)
                            || !matches!(step.delivery, DeliveryDef::Instant)
                        {
                            return Err(error(format!("{effect_path}.op.HealthCost"), "HealthCost requires a positive signed-range amount and an Instant SelfUnit producer".into()));
                        }
                    }
                    SkillEffectDef::RechargeShield {
                        source_skill,
                        amount,
                    } => {
                        let exists = references.iter().find(|s| &s.id == source_skill).is_some_and(|s| s.steps.iter().any(|step| step.effects.iter().any(|e| matches!(e.op,
                            SkillEffectDef::ShieldExpr { .. } | SkillEffectDef::Sustained { definition: crate::game::ability::sustained::SustainedEffectDef::Shield { .. } }
                        ))));
                        if *amount == 0
                            || !exists
                            || !matches!(step.target, crate::game::ability::SkillTarget::SelfUnit)
                            || !matches!(step.delivery, DeliveryDef::Instant)
                        {
                            return Err(error(format!("{effect_path}.op.RechargeShield"), "RechargeShield requires a positive amount, a shield-producing source skill, and Instant SelfUnit".into()));
                        }
                    }
                    SkillEffectDef::PersonalShieldGrant { definition } => {
                        crate::game::relic::structural::validate_personal_shield_definition(
                            definition,
                        )
                        .map_err(|e| {
                            error(
                                format!("{effect_path}.op.PersonalShieldGrant"),
                                e.to_string(),
                            )
                        })?;
                        let valid_target = match definition.recipients {
                            crate::game::relic::structural::authoring::PersonalShieldRecipientsDef::PlayerEmployeesAtRelease { .. } =>
                                matches!(step.target, crate::game::ability::SkillTarget::SelfUnit),
                            crate::game::relic::structural::authoring::PersonalShieldRecipientsDef::SelectedPlayerEmployeeAtRelease { .. } =>
                                (matches!(step.target, crate::game::ability::SkillTarget::AllySingle) ||
                                    (matches!(step.target, crate::game::ability::SkillTarget::SelfUnit) &&
                                        effect.source().is_some_and(|source| single_recipe_sites.contains(&source.site))))
                                    && skill.cast_selection() == SkillCastSelection::PlayerUnit,
                        };
                        if !valid_target || !matches!(step.delivery, DeliveryDef::Instant) {
                            return Err(error(format!("{effect_path}.op.PersonalShieldGrant"),
                                "personal grant requires Instant with its explicit self/selected-player carrier".into()));
                        }
                    }
                    SkillEffectDef::ShieldExpr {
                        duration_ms,
                        priority,
                        shared_radius_units,
                        ..
                    } => {
                        crate::game::ability::sustained::SustainedEffectDef::Shield {
                            duration_ms: *duration_ms,
                            amount: 1,
                            priority: *priority,
                            shared_radius_units: *shared_radius_units,
                        }
                        .validate()
                        .map_err(|message| {
                            error(format!("{effect_path}.op.ShieldExpr"), message)
                        })?;
                        if shared_radius_units.is_some()
                            && (!matches!(step.target, crate::game::ability::SkillTarget::SelfUnit)
                                || !matches!(step.delivery, DeliveryDef::Instant))
                        {
                            return Err(error(
                                format!("{effect_path}.op.ShieldExpr"),
                                "Shared shield requires one Instant SelfUnit producer".into(),
                            ));
                        }
                    }
                    SkillEffectDef::Sustained { definition } => {
                        definition.validate().map_err(|message| {
                            error(format!("{effect_path}.op.Sustained"), message)
                        })?;
                        if let crate::game::ability::sustained::SustainedEffectDef::SegmentedShield { pool, .. } = definition {
                            if skill.flow.is_some()
                                || !matches!(step.repeat, crate::game::ability::SkillStepRepeat::Once)
                                || effect.attached_to.is_some()
                                || !segmented_pools.insert(pool.pool_id.as_str())
                            {
                                return Err(error(format!("{effect_path}.op.Sustained"), "SegmentedShield requires a unique pool producer, Once, without flow or damage attachment".into()));
                            }
                        }
                        if let crate::game::ability::sustained::SustainedEffectDef::Parry { fragment_guard: Some(_), .. } = definition {
                            let declarations = skill.steps.iter().flat_map(|s| &s.effects).filter(|e| matches!(&e.op, SkillEffectDef::Sustained { definition: crate::game::ability::sustained::SustainedEffectDef::Parry { fragment_guard: Some(_), .. } })).count();
                            if declarations != 1 || skill.cast_selection() != crate::game::ability::SkillCastSelection::SelfUnit {
                                return Err(error(format!("{effect_path}.op.Sustained"), "compiled fragment guard requires exactly one SelfUnit parry declaration".into()));
                            }
                        }
                        if let crate::game::ability::sustained::SustainedEffectDef::Parry {
                            counter_skill,
                            guard_charge,
                            ..
                        } = definition
                        {
                            let child = references
                                .iter()
                                .find(|s| &s.id == counter_skill)
                                .ok_or_else(|| {
                                    error(
                                        format!("{effect_path}.op.Sustained.counter_skill"),
                                        "missing parry counter".into(),
                                    )
                                })?;
                            let valid = counter_skill != &skill.id
                                && child.focus_time_ms == 0
                                && child.flow.is_none()
                                && child.steps.len() == 1
                                && child.steps[0].delay_ms == 0
                                && matches!(child.steps[0].delivery, DeliveryDef::Instant)
                                && matches!(
                                    child.steps[0].target,
                                    crate::game::ability::SkillTarget::CastTarget
                                )
                                && if let Some(charge) = guard_charge {
                                    child.steps[0].effects.len() == 1 && matches!(
                                        &child.steps[0].effects[0].op,
                                        SkillEffectDef::GuardCounter { program } if program.charge.as_ref() == charge.as_ref()
                                    )
                                } else {
                                    child.steps[0].effects.iter().all(|e| matches!(
                                        e.op, SkillEffectDef::Damage { .. }
                                            | SkillEffectDef::DamageExpr { .. }
                                            | SkillEffectDef::ModifyDamage { .. }
                                    ))
                                };
                            if !valid {
                                return Err(error(format!("{effect_path}.op.Sustained.counter_skill"), "counter requires a distinct zero-focus immediate CastTarget damage leaf without flow".into()));
                            }
                        }
                        if let crate::game::ability::sustained::SustainedEffectDef::BasicAttackOverlay { goodbye_skill: Some(id), .. } = definition {
                            let child = references.iter().find(|s| &s.id == id).ok_or_else(|| error(format!("{effect_path}.op.Sustained.goodbye_skill"), format!("missing Goodbye skill '{id}'")))?;
                            let immediate = child.steps.len() == 1 && child.steps[0].delay_ms == 0
                                && matches!(&child.steps[0].delivery, DeliveryDef::Instant | DeliveryDef::TileArea { area: crate::game::ability::SkillTileAreaDeliveryDef { duration_ms: 0, emission: None, first_tick_delay_ms: 0, .. }, .. });
                            if id == &skill.id || child.focus_time_ms != 0 || child.flow.is_some() || !immediate {
                                return Err(error(format!("{effect_path}.op.Sustained.goodbye_skill"), "Goodbye requires a distinct zero-focus skill with one immediate zero-delay step and no flow; reinput executes it atomically".into()));
                            }
                        }
                        if matches!(
                            definition,
                            crate::game::ability::sustained::SustainedEffectDef::PreparedBasic { .. }
                            | crate::game::ability::sustained::SustainedEffectDef::Parry { .. }
                            | crate::game::ability::sustained::SustainedEffectDef::ChannelAura { .. }
                            | crate::game::ability::sustained::SustainedEffectDef::BasicAttackOverlay { .. }
                            | crate::game::ability::sustained::SustainedEffectDef::Shield { shared_radius_units: Some(_), .. }
                            | crate::game::ability::sustained::SustainedEffectDef::SegmentedShield { .. }
                        ) && (!matches!(
                            step.target,
                            crate::game::ability::SkillTarget::SelfUnit
                        ) || !matches!(
                            step.delivery,
                            crate::game::ability::DeliveryDef::Instant
                        )) {
                            return Err(error(
                                format!("{effect_path}.op.Sustained"),
                                "ChannelAura/BasicAttackOverlay/shared Shield requires an Instant SelfUnit producer".to_owned(),
                            ));
                        }
                    }
                    SkillEffectDef::Heal { .. }
                    | SkillEffectDef::HealExpr { .. }
                    | SkillEffectDef::ModifyResonance { .. }
                    | SkillEffectDef::ModifyTacticalResource { .. }
                    | SkillEffectDef::ModifyStats { .. }
                    | SkillEffectDef::ApplyBuff { .. }
                    | SkillEffectDef::InterruptCast
                    | SkillEffectDef::ExtraAttack { .. } => {}
                }
            }
            if has_modify_damage_effect && !has_damage_effect {
                return Err(error(
                    format!("{step_path}.effects"),
                    "ModifyDamage requires a Damage effect in the same step".to_string(),
                ));
            }

            step.delivery
                .try_validate_projectile_scalar_contract()
                .map_err(|delivery_error| {
                    error(
                        format!("{step_path}.delivery.{}", delivery_error.field_suffix),
                        delivery_error.cause.to_string(),
                    )
                })?;

            match &step.delivery {
                DeliveryDef::Projectile {
                    hit_policy,
                    allow_targetless_cast,
                    max_range_tiles,
                    max_lifetime_ms,
                    max_kills,
                    max_pierces,
                    collision,
                    ..
                } => {
                    collision.try_validate_runtime_contract().map_err(|cause| {
                        error(format!("{step_path}.delivery.Projectile.collision"), cause)
                    })?;
                    match hit_policy {
                        ProjectileHitPolicy::TargetLocked => {
                            if *allow_targetless_cast {
                                return Err(error(
                                    format!(
                                        "{step_path}.delivery.Projectile.allow_targetless_cast"
                                    ),
                                    "target-locked projectile must not allow targetless cast"
                                        .to_string(),
                                ));
                            }
                            if max_range_tiles.is_some()
                                || max_lifetime_ms.is_some()
                                || max_kills.is_some()
                                || max_pierces.is_some()
                            {
                                return Err(error(
                                    format!("{step_path}.delivery.Projectile"),
                                    "target-locked projectile must not define directional stop policy fields".to_string(),
                                ));
                            }
                            collision
                                .try_validate_homing_runtime_contract()
                                .map_err(|cause| {
                                    error(
                                        format!("{step_path}.delivery.Projectile.collision"),
                                        cause,
                                    )
                                })?;
                        }
                        ProjectileHitPolicy::DirectionalCollision => {
                            if max_range_tiles.is_none() && max_lifetime_ms.is_none() {
                                return Err(error(
                                    format!("{step_path}.delivery.Projectile"),
                                    "DirectionalCollision requires max_range_tiles or max_lifetime_ms".to_string(),
                                ));
                            }
                            if max_kills.is_none()
                                && max_pierces.is_none()
                                && collision.max_hits.is_none()
                                && max_range_tiles.is_none()
                                && max_lifetime_ms.is_none()
                            {
                                return Err(error(
                                    format!("{step_path}.delivery.Projectile"),
                                    "DirectionalCollision requires at least one stop policy"
                                        .to_string(),
                                ));
                            }
                        }
                    }
                }
                DeliveryDef::TileArea { footprint, area } => {
                    footprint.validate().map_err(|cause| {
                        error(format!("{step_path}.delivery.TileArea.footprint"), cause)
                    })?;
                    area.try_validate_runtime_contract().map_err(|cause| {
                        error(format!("{step_path}.delivery.TileArea.area"), cause)
                    })?;
                    if area.ground_status.is_some() && !step.effects.is_empty() {
                        return Err(error(format!("{step_path}.effects"), "ground status subscription delivery must not discard ordinary tick effects".into()));
                    }
                    if area.mode == crate::game::ability::SkillTileAreaMode::SelectedGround
                        && (*selection != SkillCastSelection::PlayerTile || step.target != SkillTarget::CastTarget) {
                        return Err(error(format!("{step_path}.delivery.TileArea.area.mode"),
                            "selected ground requires PlayerTile selection and CastTarget action".into()));
                    }
                    if matches!(area.mode, crate::game::ability::SkillTileAreaMode::ChannelGround { .. })
                        && (*selection != SkillCastSelection::SelfUnit || step.target != SkillTarget::SelfUnit) {
                        return Err(error(format!("{step_path}.delivery.TileArea.area.mode"), "channel ground requires self selection and self action".into()));
                    }
                    let expected_origin = match area.anchor {
                        SkillAreaAnchorSource::Caster => {
                            crate::game::ability::SkillTileAreaOrigin::Caster
                        }
                        SkillAreaAnchorSource::AcceptedCastTarget
            | SkillAreaAnchorSource::CastTarget
                        | SkillAreaAnchorSource::CastTargetStart
                        | SkillAreaAnchorSource::ImpactContext
                        | SkillAreaAnchorSource::ImpactContextStart => {
                            crate::game::ability::SkillTileAreaOrigin::Anchor
                        }
                    };
                    if area.tile_origin != expected_origin {
                        return Err(error(
                            format!("{step_path}.delivery.TileArea.area.tile_origin"),
                            format!(
                                "{:?} area requires {:?} tile_origin",
                                area.anchor, expected_origin
                            ),
                        ));
                    }
                    if area.anchor != SkillAreaAnchorSource::Caster
                        && !matches!(area.mode, crate::game::ability::SkillTileAreaMode::SelectedGround | crate::game::ability::SkillTileAreaMode::ChannelGround { .. })
                        && !footprint.is_rotation_invariant()
                    {
                        return Err(error(
                            format!("{step_path}.delivery.TileArea.footprint"),
                            "target/impact-centered footprint must be rotation-invariant until an explicit orientation policy exists".to_string(),
                        ));
                    }
                    if matches!(
                        area.anchor,
                        SkillAreaAnchorSource::ImpactContext
                            | SkillAreaAnchorSource::ImpactContextStart
                    ) {
                        let has_prior_spatial_delivery =
                            skill.steps[..step_index].iter().any(|prior_step| {
                                matches!(
                                    prior_step.delivery,
                                    DeliveryDef::Projectile { .. } | DeliveryDef::TileArea { .. }
                                )
                            });
                        if !has_prior_spatial_delivery {
                            return Err(error(
                                format!("{step_path}.delivery.TileArea.area.anchor"),
                                format!("{:?} requires a prior spatial delivery step", area.anchor),
                            ));
                        }
                    }
                }
                DeliveryDef::Instant => {}
            }
        }
        // A finite beneficial effect or owned channel is part of the skill use. Hostile statuses,
        // periodic damage and permanent changes retain their independent owners.
        let finite_use_effect = |effect: &SkillEffectDef| match effect {
            SkillEffectDef::ShieldExpr { .. } | SkillEffectDef::PreparedVolley { .. } | SkillEffectDef::InputPlan { .. } | SkillEffectDef::RelationAction { .. } | SkillEffectDef::CounterChannel { .. } => true,
            SkillEffectDef::Sustained { definition } => {
                !definition.is_hostile()
                    && !matches!(
                        definition,
                        crate::game::ability::sustained::SustainedEffectDef::SegmentedShield { .. }
                        | crate::game::ability::sustained::SustainedEffectDef::PreparedBasic { .. }
                    )
            }
            _ => false,
        };
        let mut selected = HashSet::new();
        if matches!(&skill.use_lifetime, crate::game::ability::SkillUseLifetime::ActionAndEffects { effects } if effects.is_empty())
        {
            return Err(error(
                format!("{skill_path}.use_lifetime"),
                "ActionAndEffects requires at least one finite effect".into(),
            ));
        }
        for reference in skill.use_lifetime.effects() {
            let step = skill.steps.iter().find(|step| step.id == reference.step);
            let effect = step.and_then(|step| {
                step.effects
                    .iter()
                    .find(|effect| effect.id == reference.effect)
            });
            if !selected.insert((&reference.step, &reference.effect))
                || !effect.is_some_and(|effect| finite_use_effect(&effect.op))
            {
                return Err(error(
                    format!("{skill_path}.use_lifetime"),
                    format!(
                        "{}.{} must uniquely reference a finite use-owned effect",
                        reference.step, reference.effect
                    ),
                ));
            }
        }
        for step in &skill.steps {
            for effect in &step.effects {
                if !finite_use_effect(&effect.op) {
                    continue;
                }
                // A recurring independent producer could apply a new lease after
                // this use ended. This combination needs a separate authoring
                // model and is rejected rather than silently extending the use.
                if matches!(&step.delivery, DeliveryDef::TileArea { area, .. } if area.is_independent())
                {
                    return Err(error(format!("{skill_path}.use_lifetime"),
                        format!("{}.{} finite use effect cannot be produced by an independent persistent area", step.id, effect.id)));
                }
                if !selected.contains(&(&step.id, &effect.id)) {
                    return Err(error(
                        format!("{skill_path}.use_lifetime"),
                        format!(
                            "{}.{} requires an explicit finite use effect reference",
                            step.id, effect.id
                        ),
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_skill_target_reach(
    reach: SkillTargetReach,
    target_range: Option<&TileRangePattern>,
) -> Result<(), String> {
    match reach {
        SkillTargetReach::Pattern => {
            if let Some(pattern) = target_range {
                pattern
                    .validate()
                    .map_err(|error| format!("invalid target_range: {error}"))?;
            }
        }
        SkillTargetReach::Global => {
            if target_range.is_some() {
                return Err("global reach must not also define target_range".to_string());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::ability::{
        DeliveryDef, ProjectileHitPolicy, SkillAreaAnchorSource, SkillCastTargetingDef,
        SkillEffectDef, SkillId, SkillProjectileCollisionDef, SkillStepDef,
        SkillTileAreaDeliveryDef, StepTargetingMode, UnitTargetRule,
    };
    use crate::game::battle::buffs::BuffDatabase;
    use crate::game::battle::damage::{DamageModifiers, DamageType};
    use crate::game::battle::tile_range::TileRangePattern;

    #[test]
    fn dynamic_source_path_survives_input_drop_in_compiled_effect_and_error() {
        let (database, error) = {
            let source = format!("packs/{}/skills/custom.ron", "user");
            let input =
                include_str!("../../../tests/fixtures/content_authoring/literal_effects.ron");
            let database = SkillDatabase::from_ron(&source, input)
                .expect("valid authored skills should compile with an owned input path");
            let invalid = input.replacen("Damage(amount: 17", "Unsupported(amount: 17", 1);
            let error = SkillDatabase::from_ron(&source, &invalid)
                .expect_err("unsupported operation must retain the original source");
            (database, error)
        };
        let effect = &database.skills[0].steps[0].effects[0];
        let source = effect.source.as_ref().expect("compiled effect source");
        assert_eq!(source.source_file, "packs/user/skills/custom.ron");
        assert_eq!(source.field_path, "skills[0].steps[0].effects[0]");
        assert_eq!(
            source.site.action.definition_id,
            SkillId::from("authoring_alpha")
        );
        assert_eq!(error.source_file, "packs/user/skills/custom.ron");
        assert_eq!(error.object_id.as_deref(), Some("authoring_alpha"));
        assert_eq!(error.action_id.as_deref(), Some("strike"));
        assert_eq!(error.effect_id.as_deref(), Some("damage"));
        assert!(error.field_path.as_deref().unwrap().contains("effects[0]"));
    }

    fn explicit_cast_targeting(target: SkillTarget) -> SkillCastTargetingDef {
        SkillCastTargetingDef::explicit(target, Default::default(), None, false)
    }

    fn explicit_enemy_cast_targeting() -> SkillCastTargetingDef {
        SkillCastTargetingDef::explicit(
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            SkillTargetReach::Pattern,
            Some(TileRangePattern {
                include_anchor_tile: false,
                rows: vec![".@X".to_string()],
            }),
            false,
        )
    }

    fn explicit_self_cast_targeting() -> SkillCastTargetingDef {
        explicit_cast_targeting(SkillTarget::SelfUnit)
    }

    fn projectile_step(collision: SkillProjectileCollisionDef) -> SkillStepDef {
        SkillStepDef {
            id: "shot".to_string(),
            delay_ms: 0,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            targeting: Default::default(),
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 1_000,
                hit_policy: ProjectileHitPolicy::TargetLocked,
                allow_targetless_cast: false,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision,
            },
            effects: vec![],
            presentation: Default::default(),
        }
    }

    fn tile_area_step(id: &str, anchor: SkillAreaAnchorSource) -> SkillStepDef {
        SkillStepDef {
            id: id.to_string(),
            delay_ms: 0,
            target: SkillTarget::CastTarget,
            targeting: Default::default(),
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::TileArea {
                footprint: TileRangePattern {
                    include_anchor_tile: true,
                    rows: vec![".@.".to_string()],
                },
                area: SkillTileAreaDeliveryDef {
                    anchor,
                    tile_origin: if anchor == SkillAreaAnchorSource::Caster {
                        crate::game::ability::SkillTileAreaOrigin::Caster
                    } else {
                        crate::game::ability::SkillTileAreaOrigin::Anchor
                    },
                    ..Default::default()
                },
            },
            effects: vec![],
            presentation: Default::default(),
        }
    }

    fn assert_validation_error(
        error: crate::game::data::DataValidationError,
        object_id: &str,
        field_path: &str,
        cause: &str,
    ) {
        assert_eq!(
            error,
            crate::game::data::DataValidationError::at(object_id, field_path, cause)
        );
    }

    #[test]
    fn skill_database_rejects_empty_step_skills() {
        let skill = SkillDef {
            action_policy: None,
            resonance_admission: None,
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
            id: SkillId::from("empty"),
            name: "empty".to_string(),
            kind: SkillKind::Untargeted,
            cast_targeting: explicit_self_cast_targeting(),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![],
        };
        let error = validate_skill_contracts(&[skill]).expect_err("empty steps must reject");
        assert_validation_error(
            error,
            "empty",
            "skills[0].steps",
            "must define at least one step",
        );
    }

    #[test]
    fn skill_database_rejects_nondefault_homing_collision_contracts() {
        let skill = SkillDef {
            action_policy: None,
            resonance_admission: None,
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
            id: SkillId::from("homing"),
            name: "homing".to_string(),
            kind: SkillKind::Targeted,
            cast_targeting: explicit_enemy_cast_targeting(),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![projectile_step(SkillProjectileCollisionDef {
                piercing: true,
                ..Default::default()
            })],
        };
        let error = validate_skill_contracts(&[skill])
            .expect_err("nondefault homing collision must reject");
        assert_validation_error(
            error,
            "homing",
            "skills[0].steps[0].delivery.Projectile.collision",
            "targeted homing projectile does not support piercing",
        );
    }

    #[test]
    fn skill_database_rejects_global_reach_with_authored_target_pattern() {
        let skill = SkillDef {
            action_policy: None,
            resonance_admission: None,
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
            id: SkillId::from("ambiguous_whole_field"),
            name: "ambiguous_whole_field".to_string(),
            kind: SkillKind::Targeted,
            cast_targeting: SkillCastTargetingDef::explicit(
                SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                SkillTargetReach::Global,
                Some(TileRangePattern {
                    include_anchor_tile: false,
                    rows: vec![".@X".to_string()],
                }),
                false,
            ),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "hit".to_string(),
                delay_ms: 0,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: Default::default(),
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![],
                presentation: Default::default(),
            }],
        };
        let error = validate_skill_contracts(&[skill])
            .expect_err("global reach plus target range must reject");
        assert_validation_error(
            error,
            "ambiguous_whole_field",
            "skills[0].cast_targeting",
            "global reach must not also define target_range",
        );
    }

    #[test]
    fn skill_database_rejects_incompatible_selection_and_player_retargeting() {
        let incompatible = SkillDef {
            action_policy: None,
            resonance_admission: None,
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
            id: SkillId::from("incompatible_player_unit"),
            name: "incompatible_player_unit".to_string(),
            kind: SkillKind::Targeted,
            cast_targeting: SkillCastTargetingDef::explicit_with_selection(
                SkillCastSelection::PlayerUnit,
                SkillTarget::SelfUnit,
                SkillTargetReach::Pattern,
                None,
                false,
            ),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "self".to_string(),
                delay_ms: 0,
                target: SkillTarget::SelfUnit,
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![],
                presentation: Default::default(),
            }],
        };
        let error = validate_skill_contracts(&[incompatible]).unwrap_err();
        assert_eq!(
            error.cause,
            "selection is incompatible with cast target semantics"
        );

        let retargeting = SkillDef {
            action_policy: None,
            resonance_admission: None,
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
            id: SkillId::from("player_unit_retargeting"),
            name: "player_unit_retargeting".to_string(),
            kind: SkillKind::Targeted,
            cast_targeting: SkillCastTargetingDef::explicit_with_selection(
                SkillCastSelection::PlayerUnit,
                SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                SkillTargetReach::Pattern,
                Some(TileRangePattern {
                    include_anchor_tile: false,
                    rows: vec![".@X".to_string()],
                }),
                false,
            ),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "retarget".to_string(),
                delay_ms: 0,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::RetargetOnStep,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![],
                presentation: Default::default(),
            }],
        };
        let error = validate_skill_contracts(&[retargeting]).unwrap_err();
        assert_eq!(
            error.cause,
            "player-selected skills cannot use RetargetOnStep"
        );
    }

    #[test]
    fn skill_database_rejects_duplicate_step_ids() {
        let skill = SkillDef {
            action_policy: None,
            resonance_admission: None,
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
            id: SkillId::from("duplicate_steps"),
            name: "duplicate_steps".to_string(),
            kind: SkillKind::Untargeted,
            cast_targeting: explicit_enemy_cast_targeting(),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![
                projectile_step(Default::default()),
                projectile_step(Default::default()),
            ],
        };
        let error = validate_skill_contracts(&[skill]).expect_err("duplicate step ids must reject");
        assert_validation_error(
            error,
            "duplicate_steps",
            "skills[0].steps[1].id",
            "duplicate step id 'shot'",
        );
    }

    #[test]
    fn skill_database_rejects_unknown_buff_references() {
        let skill = SkillDef {
            action_policy: None,
            resonance_admission: None,
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
            id: SkillId::from("unknown_buff"),
            name: "unknown_buff".to_string(),
            kind: SkillKind::Untargeted,
            cast_targeting: explicit_self_cast_targeting(),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "step".to_string(),
                delay_ms: 0,
                target: SkillTarget::SelfUnit,
                targeting: Default::default(),
                when: crate::game::ability::SkillStepCondition::IfCasterHasBuff {
                    buff_id: "does_not_exist".to_string(),
                    min_stacks: 1,
                },
                repeat: crate::game::ability::SkillStepRepeat::ByBuffStacks {
                    unit: crate::game::ability::SkillUnitReference::SelfUnit,
                    buff_id: "does_not_exist".to_string(),
                    max: None,
                },
                delivery: DeliveryDef::Instant,
                effects: vec![crate::game::ability::SkillEffectDef::ApplyBuff {
                    buff_id: "does_not_exist".to_string(),
                    duration_ms: 100,
                }
                .authored("effect_01")],
                presentation: Default::default(),
            }],
        };
        let database = SkillDatabase::new(vec![skill]);
        let error = database
            .validate_buff_references(&BuffDatabase::new(vec![]))
            .expect_err("unknown buff references must be rejected");
        assert_eq!(error.object_id.as_deref(), Some("unknown_buff"));
        assert_eq!(
            error.field_path.as_deref(),
            Some("skills[0].steps[0].when.IfCasterHasBuff.buff_id")
        );
        assert_eq!(error.cause, "references unknown buff 'does_not_exist'");
    }

    #[test]
    fn skill_database_rejects_negative_damage_amounts() {
        let skill = SkillDef {
            action_policy: None,
            resonance_admission: None,
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
            id: SkillId::from("negative_damage"),
            name: "negative_damage".to_string(),
            kind: SkillKind::Untargeted,
            cast_targeting: explicit_enemy_cast_targeting(),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "hit".to_string(),
                delay_ms: 0,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: Default::default(),
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::Damage {
                    amount: -1,
                    damage_type: DamageType::Magic,
                }
                .authored("effect_01")],
                presentation: Default::default(),
            }],
        };
        let error = validate_skill_contracts(&[skill]).expect_err("negative Damage must reject");
        assert_validation_error(
            error,
            "negative_damage",
            "skills[0].steps[0].effects[0].op.Damage.amount",
            "Damage amount must not be negative: -1",
        );
    }

    #[test]
    fn skill_database_rejects_modify_damage_without_damage_effect() {
        let skill = SkillDef {
            action_policy: None,
            resonance_admission: None,
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
            id: SkillId::from("dangling_modify_damage"),
            name: "dangling_modify_damage".to_string(),
            kind: SkillKind::Untargeted,
            cast_targeting: explicit_enemy_cast_targeting(),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "modifier_only".to_string(),
                delay_ms: 0,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: Default::default(),
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::ModifyDamage {
                    modifiers: DamageModifiers {
                        damage_amp_percent: 25,
                        ..Default::default()
                    },
                }
                .authored("effect_01")],
                presentation: Default::default(),
            }],
        };
        let error = validate_skill_contracts(&[skill])
            .expect_err("ModifyDamage without Damage must reject");
        assert_validation_error(
            error,
            "dangling_modify_damage",
            "skills[0].steps[0].effects",
            "ModifyDamage requires a Damage effect in the same step",
        );
    }

    #[test]
    fn skill_database_rejects_impact_context_anchor_without_prior_spatial_delivery() {
        let skill = SkillDef {
            action_policy: None,
            resonance_admission: None,
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
            id: SkillId::from("dangling_impact_context"),
            name: "dangling_impact_context".to_string(),
            kind: SkillKind::Untargeted,
            cast_targeting: explicit_cast_targeting(SkillTarget::CastTarget),
            focus_time_ms: 0,
            focus_permissions: Default::default(),
            steps: vec![tile_area_step("area", SkillAreaAnchorSource::ImpactContext)],
        };
        let error = validate_skill_contracts(&[skill])
            .expect_err("ImpactContext without prior spatial delivery must reject");
        assert_validation_error(
            error,
            "dangling_impact_context",
            "skills[0].steps[0].delivery.TileArea.area.anchor",
            "ImpactContext requires a prior spatial delivery step",
        );
    }

    #[test]
    fn skill_database_rejects_noncentered_or_oriented_target_area() {
        for defect in ["caster_origin", "implicit_orientation"] {
            let mut step = tile_area_step("area", SkillAreaAnchorSource::CastTarget);
            let DeliveryDef::TileArea { footprint, area } = &mut step.delivery else {
                unreachable!();
            };
            match defect {
                "caster_origin" => {
                    area.tile_origin = crate::game::ability::SkillTileAreaOrigin::Caster;
                }
                "implicit_orientation" => {
                    *footprint = TileRangePattern {
                        include_anchor_tile: false,
                        rows: vec![".@X".to_string()],
                    };
                }
                _ => unreachable!(),
            }
            let skill = SkillDef {
                action_policy: None,
                resonance_admission: None,
                use_lifetime: crate::game::ability::SkillUseLifetime::Action,
                flow: None,
                id: SkillId::from(defect),
                name: defect.to_string(),
                kind: SkillKind::Targeted,
                cast_targeting: explicit_cast_targeting(SkillTarget::CastTarget),
                focus_time_ms: 0,
                focus_permissions: Default::default(),
                steps: vec![step],
            };
            let error =
                validate_skill_contracts(&[skill]).expect_err("target-area defect must reject");
            let (field_path, cause) = match defect {
                "caster_origin" => (
                    "skills[0].steps[0].delivery.TileArea.area.tile_origin",
                    "CastTarget area requires Anchor tile_origin",
                ),
                "implicit_orientation" => (
                    "skills[0].steps[0].delivery.TileArea.footprint",
                    "target/impact-centered footprint must be rotation-invariant until an explicit orientation policy exists",
                ),
                _ => unreachable!(),
            };
            assert_validation_error(error, defect, field_path, cause);
        }
    }

    fn complete_skill_database_ron(
        skill_id: &str,
        cast_targeting: &str,
        step_target: &str,
        step_extra: &str,
        delivery: &str,
    ) -> String {
        format!(
            r#"
            SkillDatabase(
                skills: [
                    SkillDef(
                        id: "{skill_id}",
                        name: "{skill_id}",
                        kind: Targeted,
                        focus_time_ms: 0,
                        use_lifetime: Action,
                        cast_targeting: {cast_targeting},
                        steps: [
                            SkillStepDef(
                                id: "hit",
                                delay_ms: 0,
                                {step_extra}
                                target: {step_target},
                                delivery: {delivery},
                                effects: [],
                            ),
                        ],
                    ),
                ],
            )
            "#
        )
    }

    fn complete_self_skill_database_ron(skill_id: &str) -> String {
        complete_skill_database_ron(
            skill_id,
            "Explicit(selection: self, target: SelfUnit, reach: pattern, air_capable: false)",
            "SelfUnit",
            "",
            "Instant",
        )
    }

    fn complete_enemy_skill_database_ron(skill_id: &str) -> String {
        complete_skill_database_ron(
            skill_id,
            "Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: pattern, target_range: Some((include_anchor_tile: false, rows: [\".@X\"])), air_capable: false)",
            "EnemySingle(rule: Nearest)",
            "",
            "Instant",
        )
    }

    fn replace_exactly_once(authored: &str, from: &str, to: &str) -> String {
        assert_eq!(
            authored.matches(from).count(),
            1,
            "fixture replacement source must occur exactly once: {from}"
        );
        authored.replacen(from, to, 1)
    }

    fn assert_single_ron_defect(valid: &str, invalid: &str, expected_error: &str) {
        ron::de::from_str::<SkillDatabase>(valid)
            .unwrap_or_else(|error| panic!("repaired control must parse: {error:?}"));
        let error = ron::de::from_str::<SkillDatabase>(invalid)
            .expect_err("restored single defect must reject");
        assert_eq!(error.to_string(), expected_error);
    }

    #[test]
    fn skill_database_resolves_range_preset_from_ron() {
        let database: SkillDatabase = ron::de::from_str(
            r#"
            SkillDatabase(
                range_presets: [
                    SkillRangePresetDef(
                        id: "front_1",
                        range: (include_anchor_tile: false, rows: [".@X"]),
                    ),
                ],
                skills: [
                    SkillDef(
                        id: "preset_skill",
                        name: "Preset Skill",
                        kind: Targeted,
                        focus_time_ms: 0,
                        use_lifetime: Action,
                        cast_targeting: Explicit(
                            selection: automatic_unit,
                            target: EnemySingle(rule: Nearest),
                            reach: pattern,
                            target_range_preset: Some("front_1"),
                            air_capable: false,
                        ),
                        steps: [
                            SkillStepDef(
                                id: "hit",
                                delay_ms: 0,
                                target: EnemySingle(rule: Nearest),
                                delivery: Instant,
                                effects: [],
                            ),
                        ],
                    ),
                ],
            )
            "#,
        )
        .unwrap();

        let skill = database.get_by_id("preset_skill").unwrap();
        let SkillCastTargetingDef::Explicit { target_range, .. } = &skill.cast_targeting;
        let range = target_range.as_ref().unwrap();
        assert_eq!(range.rows, vec![".@X"]);
        assert_eq!(database.range_presets.len(), 1);
    }

    #[test]
    fn skill_database_rejects_implicit_and_legacy_targeting_authority() {
        let self_control = complete_self_skill_database_ron("implicit_skill");
        let missing_cast_targeting = replace_exactly_once(
            &self_control,
            "cast_targeting: Explicit(selection: self, target: SelfUnit, reach: pattern, air_capable: false),",
            "",
        );
        assert_single_ron_defect(
            &self_control,
            &missing_cast_targeting,
            "missing field `cast_targeting`",
        );

        let missing_selection_control = complete_self_skill_database_ron("missing_cast_selection");
        let missing_cast_selection =
            replace_exactly_once(&missing_selection_control, "selection: self, ", "");
        assert_single_ron_defect(
            &missing_selection_control,
            &missing_cast_selection,
            "missing field `selection`",
        );

        let unknown_selection_control = complete_enemy_skill_database_ron("unknown_cast_selection");
        let unknown_cast_selection = replace_exactly_once(
            &unknown_selection_control,
            "selection: automatic_unit",
            "selection: player_cursor_unit",
        );
        assert_single_ron_defect(
            &unknown_selection_control,
            &unknown_cast_selection,
            "unknown variant `player_cursor_unit`, expected one of `self`, `automatic_unit`, `player_unit`, `player_tile`",
        );

        let missing_rule_control = complete_enemy_skill_database_ron("missing_selection_rule");
        let missing_selection_rule = replace_exactly_once(
            &missing_rule_control,
            "selection: automatic_unit, target: EnemySingle(rule: Nearest)",
            "selection: automatic_unit, target: EnemySingle()",
        );
        assert_single_ron_defect(
            &missing_rule_control,
            &missing_selection_rule,
            "missing field `rule`",
        );

        let missing_reach_control = complete_self_skill_database_ron("missing_reach");
        let missing_reach = replace_exactly_once(&missing_reach_control, "reach: pattern, ", "");
        assert_single_ron_defect(
            &missing_reach_control,
            &missing_reach,
            "missing field `reach`",
        );

        let missing_air_control = complete_self_skill_database_ron("missing_air_capable");
        let missing_air_capable =
            replace_exactly_once(&missing_air_control, ", air_capable: false)", ")");
        assert_single_ron_defect(
            &missing_air_control,
            &missing_air_capable,
            "missing field `air_capable`",
        );

        let missing_range_control =
            complete_enemy_skill_database_ron("missing_pattern_target_range");
        let missing_pattern_target_range = replace_exactly_once(
            &missing_range_control,
            "target_range: Some((include_anchor_tile: false, rows: [\".@X\"])), ",
            "",
        );
        assert_single_ron_defect(
            &missing_range_control,
            &missing_pattern_target_range,
            "invalid Skill data: DataValidationError { object_id: Some(\"missing_pattern_target_range\"), field_path: Some(\"skills[0].cast_targeting.target_range\"), cause: \"pattern reach for a single unit target requires target_range\" }",
        );

        let legacy_weapon_control = complete_enemy_skill_database_ron("legacy_weapon_authority");
        let legacy_weapon_authority = replace_exactly_once(
            &legacy_weapon_control,
            "Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: pattern, target_range: Some((include_anchor_tile: false, rows: [\".@X\"])), air_capable: false)",
            "WeaponTargetingProfile",
        );
        assert_single_ron_defect(
            &legacy_weapon_control,
            &legacy_weapon_authority,
            "unknown variant `WeaponTargetingProfile`, expected `Explicit`",
        );

        let legacy_range_control = complete_enemy_skill_database_ron("legacy_range_skill");
        let legacy_range_units = replace_exactly_once(
            &legacy_range_control,
            "delay_ms: 0,",
            "delay_ms: 0, range_units: 3,",
        );
        assert_single_ron_defect(
            &legacy_range_control,
            &legacy_range_units,
            "unknown field `range_units`, expected one of `id`, `delay_ms`, `target`, `targeting`, `when`, `repeat`, `delivery`, `effects`, `presentation`",
        );

        for (legacy_step_field, field_name) in [
            ("range_policy: whole_field_valid_tiles,", "range_policy"),
            (
                "defense_tile_range: Some((include_anchor_tile: false, rows: [\".@X\"])),",
                "defense_tile_range",
            ),
            ("air_capable: true,", "air_capable"),
        ] {
            let control = complete_self_skill_database_ron("legacy_step_authority");
            let authored = replace_exactly_once(
                &control,
                "delay_ms: 0,",
                &format!("delay_ms: 0, {legacy_step_field}"),
            );
            assert_single_ron_defect(
                &control,
                &authored,
                &format!(
                    "unknown field `{field_name}`, expected one of `id`, `delay_ms`, `target`, `targeting`, `when`, `repeat`, `delivery`, `effects`, `presentation`"
                ),
            );
        }

        let footprint_control = complete_skill_database_ron(
            "missing_footprint",
            "Explicit(selection: self, target: SelfUnit, reach: pattern, air_capable: false)",
            "SelfUnit",
            "",
            "TileArea(footprint: (include_anchor_tile: true, rows: [\".@.\"]), area: (anchor: Caster))",
        );
        let missing_tile_area_footprint = replace_exactly_once(
            &footprint_control,
            "footprint: (include_anchor_tile: true, rows: [\".@.\"]), ",
            "",
        );
        assert_single_ron_defect(
            &footprint_control,
            &missing_tile_area_footprint,
            "missing field `footprint`",
        );
    }

    #[test]
    fn skill_database_rejects_inline_target_range_and_preset_conflict() {
        let control = replace_exactly_once(
            &complete_enemy_skill_database_ron("conflict_skill"),
            "SkillDatabase(",
            r#"SkillDatabase(
                range_presets: [
                    SkillRangePresetDef(
                        id: "front_1",
                        range: (include_anchor_tile: false, rows: [".@X"]),
                    ),
                ],"#,
        );
        let conflict = replace_exactly_once(
            &control,
            "target_range: Some((include_anchor_tile: false, rows: [\".@X\"])), ",
            "target_range: Some((include_anchor_tile: false, rows: [\".@X\"])), target_range_preset: Some(\"front_1\"), ",
        );
        assert_single_ron_defect(
            &control,
            &conflict,
            "explicit cast_targeting defines both target_range and target_range_preset 'front_1'",
        );
    }
}
