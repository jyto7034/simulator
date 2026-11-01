use super::*;
use crate::game::ability::{
    authoring::AuthoredSkillEffect, flow::*, sustained::SustainedEffectDef, SkillDef,
    SkillEffectDef, SkillPresentationDef, SkillStepCondition, SkillStepDef, SkillStepRepeat,
    SkillUseLifetime, StepTargetingMode,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
mod action_modifiers;
mod draft;
mod origins;
mod overlay;

#[derive(Clone)]
struct Resolved<'a> {
    value: &'a Value,
    value_path: String,
    sources: Vec<String>,
}
type Environment<'a> = BTreeMap<&'a str, Resolved<'a>>;
type Types<'a> = BTreeMap<&'a str, ValueType>;

struct Compiler<'a> {
    module: &'a Module,
    predicate_origins: super::predicates::Origins,
    artifact: Artifact,
    values: BTreeMap<&'a str, (usize, &'a Value)>,
    definitions: BTreeMap<&'a str, (usize, &'a Definition)>,
    nodes: usize,
    condition_nodes: usize,
    copied_bytes: usize,
    action_modifiers: Vec<crate::game::ability::action::ActionModifier>,
}

pub fn compile_json(input: &str) -> Result<Artifact, Diagnostic> {
    let mut artifact = Artifact {
        // This object is owned by the compiler until every validation succeeds.
        // Its in-progress diagnostic map comes from the supplied original.
        original_verified: true,
        version: VERSION,
        format: "skill-composition-native".into(),
        document: String::new(),
        source_digest: String::new(),
        native_ron: String::new(),
        resources: Vec::new(),
        equipment_attacks: Vec::new(),
        equipment_defenses: Vec::new(),
        equipment_supports: Vec::new(),
        equipment_states: Vec::new(),
        sources: BTreeMap::new(),
    };
    if input.len() > MAX_INPUT_BYTES {
        return Err(artifact.diagnostic(DiagnosticCode::Limit, "$", "module byte limit exceeded"));
    }
    artifact.source_digest = format!("{:x}", Sha256::digest(input.as_bytes()));
    let mut decoder = serde_json::Deserializer::from_str(input);
    let mut module: Module = serde_path_to_error::deserialize(&mut decoder).map_err(|error| {
        if let Some((document, reason, path)) = draft::unresolved_at(input, error.path()) {
            let mut diagnostic = artifact.diagnostic(
                DiagnosticCode::Unresolved,
                &path,
                reason,
            );
            diagnostic.document = document;
            diagnostic
        } else {
            artifact.diagnostic(
                DiagnosticCode::Parse,
                &error.path().to_string(),
                error.to_string(),
            )
        }
    })?;
    decoder
        .end()
        .map_err(|error| artifact.diagnostic(DiagnosticCode::Parse, "$", error.to_string()))?;
    artifact.document = module.id.clone();
    if !(1..=VERSION).contains(&module.version) {
        return Err(artifact.diagnostic(
            DiagnosticCode::Version,
            "version",
            "unsupported composition model version",
        ));
    }
    artifact.version = module.version;
    let predicate_origins = super::predicates::lower(&mut module, &artifact)?;
    artifact.resources = module.resources.clone();
    let mut compiler = Compiler {
        module: &module,
        predicate_origins,
        artifact,
        values: BTreeMap::new(),
        definitions: BTreeMap::new(),
        nodes: 0,
        condition_nodes: 0,
        copied_bytes: 0,
        action_modifiers: Vec::new(),
    };
    compiler
        .check()
        .map_err(|error| compiler.contextualize(error))?;
    for (index, binding) in module.equipment_attacks.iter().enumerate() {
        let path = format!("equipment_attacks[{index}]");
        let value = compiler.resolve(
            &binding.program,
            &format!("{path}.program"),
            &Environment::new(),
            &[path.clone()],
        )?;
        let Value::AttackProgram(program) = value.value else {
            unreachable!("checked argument type");
        };
        compiler.record(&path, &path, &[], vec![])?;
        compiler.record(
            &format!("{path}.program"),
            &format!("{}.AttackProgram", value.value_path),
            &[path.clone()],
            value.sources,
        )?;
        compiler.artifact.equipment_attacks.push(
            crate::game::ability::equipment_attack::EquipmentAttackBinding {
                equipment: binding.equipment.clone(),
                program: program.clone(),
                source: None,
            },
        );
    }
    for (index, binding) in module.equipment_defenses.iter().enumerate() {
        let path = format!("equipment_defenses[{index}]");
        let value = compiler.resolve(
            &binding.program,
            &format!("{path}.program"),
            &Environment::new(),
            &[path.clone()],
        )?;
        let Value::DefenseProgram(program) = value.value else {
            unreachable!("checked argument type");
        };
        compiler.record(&path, &path, &[], vec![])?;
        compiler.record(
            &format!("{path}.program"),
            &format!("{}.DefenseProgram", value.value_path),
            &[path.clone()],
            value.sources,
        )?;
        compiler.artifact.equipment_defenses.push(
            crate::game::ability::equipment_defense::EquipmentDefenseBinding {
                equipment: binding.equipment.clone(),
                program: program.clone(),
                source: None,
            },
        );
    }
    for (index, binding) in module.equipment_supports.iter().enumerate() {
        let path = format!("equipment_supports[{index}]");
        let value = compiler.resolve(
            &binding.program,
            &format!("{path}.program"),
            &Environment::new(),
            &[path.clone()],
        )?;
        let Value::SupportProgram(program) = value.value else {
            unreachable!("checked argument type");
        };
        compiler.record(&path, &path, &[], vec![])?;
        compiler.record(
            &format!("{path}.program"),
            &format!("{}.SupportProgram", value.value_path),
            &[path.clone()],
            value.sources,
        )?;
        compiler.artifact.equipment_supports.push(
            crate::game::ability::equipment_support::EquipmentSupportBinding {
                equipment: binding.equipment.clone(),
                program: program.clone(),
                source: None,
            },
        );
    }
    for (index, binding) in module.equipment_states.iter().enumerate() {
        let path = format!("equipment_states[{index}]");
        let value = compiler.resolve(
            &binding.program,
            &format!("{path}.program"),
            &Environment::new(),
            &[path.clone()],
        )?;
        let Value::StateProgram(program) = value.value else {
            unreachable!("checked argument type");
        };
        compiler.record(&path, &path, &[], vec![])?;
        compiler.record(
            &format!("{path}.program"),
            &format!("{}.StateProgram", value.value_path),
            &[path.clone()],
            value.sources,
        )?;
        compiler.artifact.equipment_states.push(
            crate::game::ability::equipment_state::EquipmentStateBinding {
                equipment: binding.equipment.clone(),
                program: program.clone(),
                source: None,
            },
        );
    }
    let mut skills = Vec::new();
    for (index, entry) in module.skills.iter().enumerate() {
        let path = format!("skills[{index}]");
        let calls = vec![path.clone()];
        compiler.expand_skill_modifiers(&entry.modifiers, &path, index)?;
        let mut steps = Vec::new();
        let flow = compiler.scope(
            &entry.body,
            &format!("{path}.body"),
            &format!("{path}.flow"),
            &Environment::new(),
            &calls,
            &mut steps,
            index,
            0,
        )?;
        compiler.bind_skill_multihit(&entry.modifiers, &path, index, &steps)?;
        if steps.is_empty() && !entry.modifiers.is_empty() {
            return Err(compiler.error(
                DiagnosticCode::Native,
                &format!("{path}.modifiers"),
                &calls,
                "skill modifiers require at least one authored Action",
            ));
        }
        compiler.record(&path, &path, &calls, vec![])?;
        // A whole-skill policy is an input to every authored action, including
        // branches. Preserve the declaration when navigating an actual result.
        if !entry.modifiers.is_empty() {
            let prefix = format!("{path}.steps[");
            for (key, origin) in &mut compiler.artifact.sources {
                if key.starts_with(&prefix) {
                    for i in 0..entry.modifiers.len() {
                        let input = format!("{path}.modifiers[{i}]");
                        if !origin.inputs.contains(&input) {
                            origin.inputs.push(input);
                        }
                    }
                }
            }
        }
        let owned_effects: Vec<_> = steps.iter().flat_map(|step| step.effects.iter().filter_map(|effect| {
            (matches!(&effect.op, SkillEffectDef::Sustained { definition: SustainedEffectDef::State { program, .. } } if program.owns_use)
                || matches!(&effect.op, SkillEffectDef::Sustained { definition: SustainedEffectDef::TargetState { .. } | SustainedEffectDef::ChannelAura { .. } | SustainedEffectDef::BasicAttackOverlay { .. } })
                || matches!(&effect.op, SkillEffectDef::RelationAction { .. } | SkillEffectDef::CounterChannel { .. }
                    | SkillEffectDef::PreparedVolley { .. } | SkillEffectDef::InputPlan { .. }))
                .then(|| crate::game::ability::SkillUseEffectRef { step: step.id.clone(), effect: effect.id.clone() })
        })).collect();
        skills.push(SkillDef {
            id: entry.id.clone(),
            name: entry.name.clone(),
            kind: entry.kind.clone(),
            cast_targeting: entry.cast_targeting.clone(),
            focus_time_ms: entry.prepare_ms,
            focus_permissions: entry.focus_permissions.clone(),
            resonance_admission: entry.admission,
            action_policy: if compiler.action_modifiers.is_empty() {
                None
            } else {
                Some(
                    crate::game::ability::action::ActionPolicy {
                        external_interrupt_protected: false,
                        modifiers: std::mem::take(&mut compiler.action_modifiers),
                    }
                    .into(),
                )
            },
            use_lifetime: if owned_effects.is_empty() {
                SkillUseLifetime::Action
            } else {
                SkillUseLifetime::ActionAndEffects {
                    effects: owned_effects,
                }
            },
            flow: Some(flow.into()),
            steps,
        });
    }
    compiler.lower_overlay_children(&mut skills)?;
    #[derive(Serialize)]
    struct Native {
        skills: Vec<SkillDef>,
    }
    compiler.artifact.native_ron = ron::ser::to_string(&Native { skills })
        .map_err(|error| compiler.error(DiagnosticCode::Native, "$", &[], error.to_string()))?;
    // Require every successfully compiled result to be portable within the
    // declared bound, rather than discovering an unusable artifact on save.
    compiler.artifact.to_ron()?;
    compiler.artifact.original_verified = true;
    Ok(compiler.artifact)
}

impl<'a> Compiler<'a> {
    fn contextualize(&self, mut error: Diagnostic) -> Diagnostic {
        if let Some(trace) = self.input_error_context(&error.source.definition) {
            error.source.calls = trace.calls;
            for input in trace.inputs {
                if !error.source.inputs.contains(&input) {
                    error.source.inputs.push(input);
                }
            }
            return error;
        }
        let path = if error.source.definition.starts_with("predicates[") {
            error
                .source
                .calls
                .first()
                .unwrap_or(&error.source.definition)
        } else {
            &error.source.definition
        };
        let context = if let Some(index) = path
            .strip_prefix("definitions[")
            .and_then(|s| s.split_once(']'))
            .and_then(|(n, _)| n.parse::<usize>().ok())
        {
            self.module.definitions.get(index).and_then(|definition| {
                let mut budget = MAX_FLOW_NODES;
                self.module
                    .skills
                    .iter()
                    .enumerate()
                    .find_map(|(i, skill)| {
                        self.find_call_context(
                            &skill.body,
                            &format!("skills[{i}].body"),
                            &definition.name,
                            &vec![format!("skills[{i}]")],
                            &mut BTreeSet::new(),
                            &mut budget,
                        )
                    })
            })
        } else {
            path.strip_prefix("skills[")
                .and_then(|s| s.split_once(']'))
                .map(|(index, _)| vec![format!("skills[{index}]")])
        };
        if let Some(mut context) = context {
            for call in &error.source.calls {
                if context.last() != Some(call) {
                    context.push(call.clone());
                }
            }
            error.source.calls = context;
        }
        error
    }
    fn find_call_context(
        &self,
        flow: &Flow,
        path: &str,
        target: &str,
        calls: &[String],
        visited: &mut BTreeSet<String>,
        budget: &mut usize,
    ) -> Option<Vec<String>> {
        if *budget == 0 || calls.len() > MAX_CALL_DEPTH {
            return None;
        }
        *budget -= 1;
        if let Flow::Call { definition, .. } = flow {
            let mut chain = calls.to_vec();
            chain.push(path.into());
            if definition == target {
                return Some(chain);
            }
            if visited.insert(definition.clone()) {
                if let Some((index, definition)) = self
                    .module
                    .definitions
                    .iter()
                    .enumerate()
                    .find(|(_, d)| d.name == *definition)
                {
                    if let Some(result) = self.find_call_context(
                        &definition.body,
                        &format!("definitions[{index}].body"),
                        target,
                        &chain,
                        visited,
                        budget,
                    ) {
                        return Some(result);
                    }
                }
            }
        }
        children(flow).into_iter().find_map(|(child, suffix)| {
            self.find_call_context(
                child,
                &format!("{path}.{suffix}"),
                target,
                calls,
                visited,
                budget,
            )
        })
    }
    fn error(
        &self,
        code: DiagnosticCode,
        path: &str,
        calls: &[String],
        reason: impl Into<String>,
    ) -> Diagnostic {
        let mut error = self.artifact.diagnostic(code, path, reason);
        error.source = SourceTrace {
            definition: path.into(),
            calls: calls.to_vec(),
            inputs: vec![],
        };
        if let Some((lowered, origin)) = self
            .predicate_origins
            .iter()
            .filter(|(lowered, _)| {
                path == lowered.as_str()
                    || path
                        .strip_prefix(lowered.as_str())
                        .is_some_and(|tail| tail.starts_with('.') || tail.starts_with('['))
            })
            .max_by_key(|(lowered, _)| lowered.len())
        {
            error.source.definition = format!("{}{}", origin.definition, &path[lowered.len()..]);
            error.source.calls.extend(origin.calls.iter().cloned());
            error.source.inputs.extend(origin.inputs.iter().cloned());
        }
        error
    }
    fn identifier(&self, value: &str, path: &str) -> Result<(), Diagnostic> {
        if value.is_empty()
            || value.len() > 128
            || value.trim() != value
            || value.chars().any(char::is_control)
        {
            return Err(self.error(
                DiagnosticCode::Reference,
                path,
                &[],
                "identifier must be nonempty, trimmed, control-free, at most 128 bytes",
            ));
        }
        Ok(())
    }
    fn check(&mut self) -> Result<(), Diagnostic> {
        self.identifier(&self.module.id, "id")?;
        if !(1..=VERSION).contains(&self.module.version) {
            return Err(self.error(
                DiagnosticCode::Version,
                "version",
                &[],
                "unsupported composition model version",
            ));
        }
        if (self.module.skills.is_empty()
            && self.module.equipment_attacks.is_empty()
            && self.module.equipment_defenses.is_empty()
            && self.module.equipment_supports.is_empty()
            && self.module.equipment_states.is_empty())
            || self.module.skills.len() > 128
            || self.module.equipment_attacks.len() > 128
            || self.module.equipment_defenses.len() > 128
            || self.module.equipment_supports.len() > 128
            || self.module.equipment_states.len() > 128
            || self.module.definitions.len() > 1024
            || self.module.values.len() > 4096
        {
            return Err(self.error(
                DiagnosticCode::Limit,
                "$",
                &[],
                "module declaration limit exceeded or no executable entries",
            ));
        }
        for (i, item) in self.module.values.iter().enumerate() {
            let path = format!("values[{i}]");
            self.identifier(&item.name, &path)?;
            if self.values.insert(&item.name, (i, &item.value)).is_some() {
                return Err(self.error(
                    DiagnosticCode::Reference,
                    &path,
                    &[],
                    "duplicate shared value",
                ));
            }
        }
        for (i, definition) in self.module.definitions.iter().enumerate() {
            let path = format!("definitions[{i}]");
            self.identifier(&definition.name, &path)?;
            if self
                .definitions
                .insert(&definition.name, (i, definition))
                .is_some()
            {
                return Err(self.error(
                    DiagnosticCode::Reference,
                    &path,
                    &[],
                    "duplicate definition",
                ));
            }
        }
        for (i, definition) in self.module.definitions.iter().enumerate() {
            let path = format!("definitions[{i}]");
            let mut types = Types::new();
            if definition.parameters.len() > 64 {
                return Err(self.error(
                    DiagnosticCode::Limit,
                    &path,
                    &[],
                    "parameter limit exceeded",
                ));
            }
            for parameter in &definition.parameters {
                self.identifier(&parameter.name, &format!("{path}.parameters"))?;
                if types
                    .insert(&parameter.name, parameter.value_type)
                    .is_some()
                {
                    return Err(self.error(
                        DiagnosticCode::Reference,
                        &path,
                        &[],
                        "duplicate parameter",
                    ));
                }
            }
            let mut ids = BTreeSet::new();
            self.check_flow(
                &definition.body,
                &format!("{path}.body"),
                &types,
                &mut ids,
                0,
            )?;
            self.check_conditions(&definition.body, &format!("{path}.body"), &ids)?;
        }
        // All references must be checked before traversing another definition.
        for (i, definition) in self.module.definitions.iter().enumerate() {
            self.check_cycles(
                &definition.body,
                &format!("definitions[{i}].body"),
                &mut vec![definition.name.as_str()],
                &mut BTreeSet::new(),
            )?;
        }
        let mut ids = BTreeSet::new();
        for (i, binding) in self.module.equipment_attacks.iter().enumerate() {
            let path = format!("equipment_attacks[{i}]");
            self.identifier(&binding.equipment, &format!("{path}.equipment"))?;
            self.expect_type(
                &binding.program,
                &format!("{path}.program"),
                &Types::new(),
                ValueType::AttackProgram,
            )?;
        }
        for (i, binding) in self.module.equipment_defenses.iter().enumerate() {
            let path = format!("equipment_defenses[{i}]");
            self.identifier(&binding.equipment, &format!("{path}.equipment"))?;
            self.expect_type(
                &binding.program,
                &format!("{path}.program"),
                &Types::new(),
                ValueType::DefenseProgram,
            )?;
        }
        for (i, binding) in self.module.equipment_supports.iter().enumerate() {
            let path = format!("equipment_supports[{i}]");
            self.identifier(&binding.equipment, &format!("{path}.equipment"))?;
            self.expect_type(
                &binding.program,
                &format!("{path}.program"),
                &Types::new(),
                ValueType::SupportProgram,
            )?;
        }
        for (i, binding) in self.module.equipment_states.iter().enumerate() {
            let path = format!("equipment_states[{i}]");
            self.identifier(&binding.equipment, &format!("{path}.equipment"))?;
            self.expect_type(
                &binding.program,
                &format!("{path}.program"),
                &Types::new(),
                ValueType::StateProgram,
            )?;
        }
        for (i, skill) in self.module.skills.iter().enumerate() {
            let path = format!("skills[{i}]");
            self.identifier(skill.id.as_str(), &path)?;
            if !ids.insert(&skill.id) {
                return Err(self.error(
                    DiagnosticCode::Reference,
                    &path,
                    &[],
                    "duplicate skill identity",
                ));
            }
            self.check_skill_modifiers(&skill.modifiers, &path)?;
            let mut local_ids = BTreeSet::new();
            self.check_flow(
                &skill.body,
                &format!("{path}.body"),
                &Types::new(),
                &mut local_ids,
                0,
            )?;
            self.check_conditions(&skill.body, &format!("{path}.body"), &local_ids)?;
        }
        // Used values are checked at their reference, so diagnostics include
        // the caller. Unused library values still cannot hide unsupported data.
        for (i, item) in self.module.values.iter().enumerate() {
            self.check_value(&item.value, &format!("values[{i}].value"), &[])?;
        }
        Ok(())
    }
    fn check_value(&self, value: &Value, path: &str, calls: &[String]) -> Result<(), Diagnostic> {
        match value {
            Value::TilePattern(pattern) => {
                pattern.validate().map_err(|reason| {
                    self.error(
                        DiagnosticCode::Native,
                        &format!("{path}.TilePattern"),
                        calls,
                        reason,
                    )
                })?;
            }
            Value::StateProgram(program) => {
                program
                    .compile_equipped(&crate::game::resource::ResourceCatalog { resources: self.module.resources.clone() }, None, &crate::game::value_expr::ExprLimits::default())
                    .map_err(|reason| {
                        self.error(
                            DiagnosticCode::Native,
                            &format!("{path}.StateProgram"),
                            calls,
                            reason,
                        )
                    })?;
            }
            Value::DefenseProgram(program) => {
                crate::game::ability::equipment_defense::EquipmentDefenseBinding {
                    equipment: "unbound".into(), program: program.clone(), source: None,
                }.compile_conditional_reduction(&crate::game::value_expr::ExprLimits::default())
                    .map_err(|(p, reason)| self.error(DiagnosticCode::Native,
                        &format!("{path}.DefenseProgram{}", p.strip_prefix("program").unwrap_or(&p)), calls, reason))?;
                crate::game::ability::equipment_defense::EquipmentDefenseBinding {
                    equipment: "unbound".into(), program: program.clone(), source: None,
                }.compile_wearer_amounts(&crate::game::value_expr::ExprLimits::default())
                    .map_err(|(p, reason)| self.error(DiagnosticCode::Native,
                        &format!("{path}.DefenseProgram{}", p.strip_prefix("program").unwrap_or(&p)), calls, reason))?;
                program.validate().map_err(|(p, reason)| {
                    self.error(
                        DiagnosticCode::Native,
                        &format!("{path}.DefenseProgram.{p}"),
                        calls,
                        reason,
                    )
                })?;
            }
            Value::SupportProgram(program) => {
                crate::game::ability::equipment_support::EquipmentSupportBinding { equipment:"unbound".into(),program:program.clone(),source:None }
                    .compile_received_heal(&crate::game::value_expr::ExprLimits::default())
                    .map_err(|(p,reason)|self.error(DiagnosticCode::Native, &format!("{path}.SupportProgram{}",p.strip_prefix("program").unwrap_or(&p)),calls,reason))?;
                program.validate().map_err(|(p, reason)| {
                    self.error(
                        DiagnosticCode::Native,
                        &format!("{path}.SupportProgram.{p}"),
                        calls,
                        reason,
                    )
                })?;
            }
            Value::AttackProgram(program) => {
                let binding = crate::game::ability::equipment_attack::EquipmentAttackBinding {
                    equipment: "unbound".into(),
                    program: program.clone(),
                    source: None,
                };
                let resources = crate::game::resource::ResourceCatalog {
                    resources: self.module.resources.clone(),
                };
                binding
                    .compile(&resources, &crate::game::value_expr::ExprLimits::default())
                    .map_err(|(p, reason)| {
                        self.error(
                            DiagnosticCode::Native,
                            &format!(
                                "{path}.AttackProgram{}",
                                p.strip_prefix("program").unwrap_or(&p)
                            ),
                            calls,
                            reason,
                        )
                    })?;
            }
            Value::Effects(effects) => {
                for (index, effect) in effects.iter().enumerate() {
                    if let SkillEffectDef::Sustained { definition } = &effect.op {
                        definition.validate_authored_provenance().map_err(|reason| {
                            self.error(DiagnosticCode::Native,
                                &format!("{path}.Effects[{index}].op.Sustained.definition"), calls, reason)
                        })?;
                    }
                    if !matches!(
                        &effect.op,
                        SkillEffectDef::Pull { .. }
                            | SkillEffectDef::Resources { .. }
                            | SkillEffectDef::PersonalShieldGrant { .. }
                            | SkillEffectDef::RelationAction { .. }
                            | SkillEffectDef::CounterChannel { .. }
                            | SkillEffectDef::PreparedVolley { .. }
                            | SkillEffectDef::InputPlan { .. }
                            | SkillEffectDef::PairedAreas { .. }
                            | SkillEffectDef::TileBurst { .. }
                            | SkillEffectDef::PresenceBurst { .. }
                            | SkillEffectDef::Sustained {
                                definition: SustainedEffectDef::HitStatus { .. }
                                    | SustainedEffectDef::Control { .. }
                                    | SustainedEffectDef::State { .. }
                                    | SustainedEffectDef::PreparedState { .. }
                                    | SustainedEffectDef::TargetState { .. }
                                    | SustainedEffectDef::ChannelAura { .. }
                                    | SustainedEffectDef::BasicAttackOverlay { .. }
                            }
                    ) {
                        return Err(self.error(
                            DiagnosticCode::Unsupported,
                            path,
                            calls,
                            "attachment input supports Resources, Pull, PersonalShieldGrant, finite HitStatus/Control/TargetState, ChannelAura, BasicAttackOverlay, authored State/PreparedState, paired area installation, released tile bursts, deployment-owned presence bursts, and owned relation/counter/input/volley programs",
                        ));
                    }
                    let program_check = match &effect.op {
                        SkillEffectDef::RelationAction { program } => Some(("RelationAction", program.validate())),
                        SkillEffectDef::CounterChannel { program, .. } => Some(("CounterChannel", program.validate())),
                        SkillEffectDef::PreparedVolley { program } => Some(("PreparedVolley", program.validate())),
                        SkillEffectDef::InputPlan { program } => Some(("InputPlan", program.validate())),
                        SkillEffectDef::PairedAreas { program } => Some(("PairedAreas", program.validate())),
                        SkillEffectDef::TileBurst { program } => Some(("TileBurst", program.validate())),
                        SkillEffectDef::PresenceBurst { program } => Some(("PresenceBurst", program.validate())),
                        _ => None,
                    };
                    if let Some((kind, result)) = program_check {
                        result.map_err(|reason| self.error(DiagnosticCode::Native,
                            &format!("{path}.Effects[{index}].op.{kind}.program"), calls, reason))?;
                    }
                    if let SkillEffectDef::Sustained {
                        definition:
                            definition @ (SustainedEffectDef::TargetState { .. }
                            | SustainedEffectDef::ChannelAura { .. }
                            | SustainedEffectDef::BasicAttackOverlay { .. }
                            | SustainedEffectDef::Control { .. }
                            | SustainedEffectDef::State { .. }
                            | SustainedEffectDef::PreparedState { .. }),
                    } = &effect.op
                    {
                        let kind = match definition {
                            SustainedEffectDef::State { .. } => "State",
                            SustainedEffectDef::PreparedState { .. } => "PreparedState",
                            SustainedEffectDef::ChannelAura { .. } => "ChannelAura",
                            SustainedEffectDef::BasicAttackOverlay { .. } => "BasicAttackOverlay",
                            SustainedEffectDef::Control { duration_ms: 0, .. } => {
                                "Control.duration_ms"
                            }
                            SustainedEffectDef::Control { .. } => "Control.kind",
                            _ => "TargetState",
                        };
                        definition.validate().map_err(|reason| {
                            self.error(
                                DiagnosticCode::Native,
                                &format!("{path}.Effects[{index}].op.Sustained.definition.{kind}"),
                                calls,
                                reason,
                            )
                        })?;
                    }
                }
            }
            Value::Delivery(delivery) => {
                delivery.validate_authored_provenance().map_err(|reason| {
                    self.error(DiagnosticCode::Native, &format!("{path}.Delivery"), calls, reason)
                })?;
                if let crate::game::ability::DeliveryDef::TileArea { footprint, area } = delivery {
                    footprint.validate().map_err(|reason| {
                        self.error(
                            DiagnosticCode::Native,
                            &format!("{path}.Delivery.TileArea.footprint"),
                            calls,
                            reason,
                        )
                    })?;
                    area.try_validate_runtime_contract().map_err(|reason| {
                        self.error(
                            DiagnosticCode::Native,
                            &format!("{path}.Delivery.TileArea.area"),
                            calls,
                            reason,
                        )
                    })?;
                }
                if !matches!(
                    delivery,
                    crate::game::ability::DeliveryDef::Instant
                        | crate::game::ability::DeliveryDef::TileArea { .. }
                ) {
                    return Err(self.error(
                        DiagnosticCode::Unsupported,
                        path,
                        calls,
                        "composition target supports Instant and TileArea delivery",
                    ));
                }
            }
            _ => (),
        }
        Ok(())
    }
    fn argument_type(
        &self,
        argument: &Argument,
        path: &str,
        types: &Types<'_>,
    ) -> Result<ValueType, Diagnostic> {
        match argument {
            Argument::Literal(value) => {
                self.check_value(value, &format!("{path}.Literal"), &[])?;
                Ok(value.value_type())
            }
            Argument::Shared(name) => {
                let (index, value) = self.values.get(name.as_str()).ok_or_else(|| {
                    self.error(
                        DiagnosticCode::Reference,
                        path,
                        &[],
                        format!("unknown shared value '{name}'"),
                    )
                })?;
                self.check_value(value, &format!("values[{index}].value"), &[path.to_owned()])
                    .map_err(|mut error| {
                        error.source.inputs.push(format!("values[{index}].value"));
                        error
                    })?;
                Ok(value.value_type())
            }
            Argument::Parameter(name) => types.get(name.as_str()).copied().ok_or_else(|| {
                self.error(
                    DiagnosticCode::Reference,
                    path,
                    &[],
                    format!("unknown parameter '{name}'"),
                )
            }),
            Argument::Unresolved(reason) => {
                Err(self.error(DiagnosticCode::Unresolved, path, &[], reason))
            }
        }
    }
    fn expect_type(
        &self,
        argument: &Argument,
        path: &str,
        types: &Types<'_>,
        expected: ValueType,
    ) -> Result<(), Diagnostic> {
        let actual = self.argument_type(argument, path, types)?;
        if actual != expected {
            return Err(self.error(
                DiagnosticCode::Type,
                path,
                &[],
                format!("expected {expected:?}, got {actual:?}"),
            ));
        }
        Ok(())
    }
    fn check_flow(
        &self,
        flow: &Flow,
        path: &str,
        types: &Types<'_>,
        ids: &mut BTreeSet<String>,
        depth: usize,
    ) -> Result<(), Diagnostic> {
        if depth >= MAX_FLOW_DEPTH || ids.len() >= MAX_FLOW_NODES {
            return Err(self.error(
                DiagnosticCode::Limit,
                path,
                &[],
                "flow syntax depth/node limit exceeded",
            ));
        }
        self.identifier(flow.id(), path)?;
        if !ids.insert(flow.id().into()) {
            return Err(self.error(
                DiagnosticCode::Reference,
                path,
                &[],
                "duplicate local flow ID",
            ));
        }
        match flow {
            Flow::Action {
                target,
                delivery,
                effects,
                modifiers,
                ..
            } => {
                self.check_action_modifiers(modifiers, path, types)?;
                self.expect_type(target, &format!("{path}.target"), types, ValueType::Target)?;
                self.expect_type(
                    delivery,
                    &format!("{path}.delivery"),
                    types,
                    ValueType::Delivery,
                )?;
                for (i, effect) in effects.iter().enumerate() {
                    let p = format!("{path}.effects[{i}]");
                    match effect {
                        Effect::Damage { amount, .. } | Effect::Heal { amount, .. } => self
                            .expect_type(
                                amount,
                                &format!("{p}.amount"),
                                types,
                                ValueType::Amount,
                            )?,
                        Effect::Append { effects } => self.expect_type(
                            effects,
                            &format!("{p}.effects"),
                            types,
                            ValueType::Effects,
                        )?,
                        Effect::PersonalShieldGrant {
                            amount, duration, ..
                        } => {
                            self.expect_type(
                                amount,
                                &format!("{p}.amount"),
                                types,
                                ValueType::Amount,
                            )?;
                            self.expect_type(
                                duration,
                                &format!("{p}.duration"),
                                types,
                                ValueType::Milliseconds,
                            )?;
                        }
                    }
                }
            }
            Flow::Wait { duration, .. } => self.expect_type(
                duration,
                &format!("{path}.duration"),
                types,
                ValueType::Milliseconds,
            )?,
            Flow::Repeat {
                count, interval, ..
            } => {
                self.expect_type(count, &format!("{path}.count"), types, ValueType::Count)?;
                self.expect_type(
                    interval,
                    &format!("{path}.interval"),
                    types,
                    ValueType::Milliseconds,
                )?;
            }
            Flow::Call {
                definition,
                arguments,
                ..
            } => {
                let (definition_index, target) =
                    self.definitions.get(definition.as_str()).ok_or_else(|| {
                        self.error(
                            DiagnosticCode::Reference,
                            path,
                            &[],
                            format!("unknown definition '{definition}'"),
                        )
                    })?;
                let mut names = BTreeSet::new();
                for (i, arg) in arguments.iter().enumerate() {
                    let p = format!("{path}.arguments[{i}]");
                    let parameter = target
                        .parameters
                        .iter()
                        .find(|p| p.name == arg.name)
                        .ok_or_else(|| {
                            self.error(
                                DiagnosticCode::Reference,
                                &p,
                                &[],
                                format!("unknown input '{}' of '{definition}'", arg.name),
                            )
                        })?;
                    if !names.insert(&arg.name) {
                        return Err(self.error(
                            DiagnosticCode::Reference,
                            &p,
                            &[],
                            "duplicate call argument",
                        ));
                    }
                    self.expect_type(
                        &arg.value,
                        &format!("{p}.value"),
                        types,
                        parameter.value_type,
                    )
                    .map_err(|mut error| {
                        error.source.calls.push(path.into());
                        error
                            .source
                            .inputs
                            .push(format!("definitions[{definition_index}].parameters"));
                        error
                    })?;
                }
                if names.len() != target.parameters.len() {
                    return Err(self.error(
                        DiagnosticCode::Reference,
                        path,
                        &[],
                        format!("missing input of '{definition}'"),
                    ));
                }
            }
            Flow::Unsupported { operation, .. } => {
                return Err(self.error(DiagnosticCode::Unsupported, path, &[], operation))
            }
            _ => (),
        }
        for (child, suffix) in children(flow) {
            self.check_flow(child, &format!("{path}.{suffix}"), types, ids, depth + 1)?;
        }
        Ok(())
    }
    // These checks need no bound arguments and apply even to unused library
    // definitions. Settlement/order and effect bindings remain native checks.
    fn check_condition(
        &self,
        condition: &FlowCondition,
        path: &str,
        ids: &BTreeSet<String>,
    ) -> Result<(), Diagnostic> {
        if let FlowCondition::Logic(logic) = condition {
            if logic.children().is_empty() {
                return Err(self.error(
                    DiagnosticCode::Type,
                    path,
                    &[],
                    "boolean group must be nonempty",
                ));
            }
            for (i, child) in logic.children().iter().enumerate() {
                self.check_condition(
                    child,
                    &format!("{path}.Logic{}", logic.child_suffix(i)),
                    ids,
                )?;
            }
        }
        if let FlowCondition::Compare {
            node_id,
            metric,
            value,
            ..
        }
        | FlowCondition::EffectCompare {
            node_id,
            metric,
            value,
            ..
        } = condition
        {
            if !ids.contains(node_id) {
                return Err(self.error(
                    DiagnosticCode::Reference,
                    &format!("{path}.node_id"),
                    &[],
                    "result reference is outside this definition's local scope",
                ));
            }
            let units_match = match metric {
                FlowMetric::HpLost => matches!(value, FlowLiteral::Hp(_)),
                FlowMetric::ResolvedTargets
                | FlowMetric::DamagedTargets
                | FlowMetric::KilledTargets => matches!(value, FlowLiteral::Count(_)),
            };
            if !units_match {
                return Err(self.error(
                    DiagnosticCode::Type,
                    &format!("{path}.value"),
                    &[],
                    "result/literal units do not match",
                ));
            }
        }
        Ok(())
    }
    fn rename_condition(
        &self,
        condition: &mut FlowCondition,
        names: &BTreeMap<String, String>,
        path: &str,
        calls: &[String],
    ) -> Result<(), Diagnostic> {
        match condition {
            FlowCondition::Compare { node_id, .. }
            | FlowCondition::EffectCompare { node_id, .. } => {
                *node_id = names.get(node_id).cloned().ok_or_else(|| {
                    self.error(
                        DiagnosticCode::Reference,
                        &format!("{path}"),
                        calls,
                        "result reference is outside this definition's local scope",
                    )
                })?;
            }
            FlowCondition::Logic(logic) => {
                let suffixes: Vec<_> = (0..logic.children().len())
                    .map(|i| logic.child_suffix(i))
                    .collect();
                for (suffix, child) in suffixes.iter().zip(logic.children_mut()) {
                    self.rename_condition(child, names, &format!("{path}.Logic{suffix}"), calls)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    fn check_conditions(
        &self,
        flow: &Flow,
        path: &str,
        ids: &BTreeSet<String>,
    ) -> Result<(), Diagnostic> {
        if let Flow::Branch { condition, .. } = flow {
            self.check_condition(condition, &format!("{path}.condition"), ids)?;
        }
        for (child, suffix) in children(flow) {
            self.check_conditions(child, &format!("{path}.{suffix}"), ids)?;
        }
        Ok(())
    }
    fn check_cycles(
        &self,
        flow: &'a Flow,
        path: &str,
        stack: &mut Vec<&'a str>,
        visited: &mut BTreeSet<&'a str>,
    ) -> Result<(), Diagnostic> {
        if let Flow::Call { definition, .. } = flow {
            if stack.contains(&definition.as_str()) || stack.len() >= MAX_CALL_DEPTH {
                return Err(self.error(
                    DiagnosticCode::Limit,
                    path,
                    &[],
                    "recursive or excessive definition call depth",
                ));
            }
            if visited.contains(definition.as_str()) {
                return Ok(());
            }
            let (index, target) = self.definitions[definition.as_str()];
            stack.push(&target.name);
            self.check_cycles(
                &target.body,
                &format!("definitions[{index}].body"),
                stack,
                visited,
            )?;
            stack.pop();
            visited.insert(definition);
        }
        for (child, suffix) in children(flow) {
            self.check_cycles(child, &format!("{path}.{suffix}"), stack, visited)?;
        }
        Ok(())
    }
    fn resolve(
        &mut self,
        argument: &'a Argument,
        path: &str,
        env: &Environment<'a>,
        calls: &[String],
    ) -> Result<Resolved<'a>, Diagnostic> {
        let result = self.reference(argument, path, env, calls)?;
        // Charge each substitution before any owned native payload is cloned.
        self.charge_conditions(super::predicates::value_nodes(result.value), path, calls)?;
        self.charge_copy(result.value, path, calls)?;
        Ok(result)
    }
    fn reference(
        &self,
        argument: &'a Argument,
        path: &str,
        env: &Environment<'a>,
        calls: &[String],
    ) -> Result<Resolved<'a>, Diagnostic> {
        let mut result = match argument {
            Argument::Literal(value) => Resolved {
                value,
                value_path: format!("{path}.Literal"),
                sources: vec![],
            },
            Argument::Shared(name) => {
                let (index, value) = self.values.get(name.as_str()).ok_or_else(|| {
                    self.error(
                        DiagnosticCode::Reference,
                        path,
                        calls,
                        "unknown shared value",
                    )
                })?;
                Resolved {
                    value,
                    value_path: format!("values[{index}].value"),
                    sources: vec![format!("values[{index}].value")],
                }
            }
            Argument::Parameter(name) => env.get(name.as_str()).cloned().ok_or_else(|| {
                self.error(DiagnosticCode::Reference, path, calls, "unknown parameter")
            })?,
            Argument::Unresolved(reason) => {
                return Err(self.error(DiagnosticCode::Unresolved, path, calls, reason))
            }
        };
        result.sources.push(path.into());
        Ok(result)
    }

    fn charge_conditions(
        &mut self,
        count: usize,
        path: &str,
        calls: &[String],
    ) -> Result<(), Diagnostic> {
        if self.module.version >= 2 {
            self.condition_nodes = self.condition_nodes.saturating_add(count);
            if self.condition_nodes > MAX_FLOW_NODES {
                return Err(self.error(
                    DiagnosticCode::Limit,
                    path,
                    calls,
                    "expanded module condition node limit exceeded",
                ));
            }
        }
        crate::authoring::control::charge(count as u64, "composition_condition_copy")
            .map_err(|e| self.error(DiagnosticCode::Limit, path, calls, format!("{e:?}")))
    }
    fn charge_copy(
        &mut self,
        value: &impl Serialize,
        path: &str,
        calls: &[String],
    ) -> Result<(), Diagnostic> {
        super::budget::payload(value, &mut self.copied_bytes, MAX_ARTIFACT_BYTES)
            .map_err(|reason| self.error(DiagnosticCode::Limit, path, calls, reason))
    }

    fn record(
        &mut self,
        native: &str,
        source: &str,
        calls: &[String],
        inputs: Vec<String>,
    ) -> Result<(), Diagnostic> {
        self.charge_copy(&(native, source, calls, &inputs), source, calls)?;
        if self.artifact.sources.len() >= 65_536 {
            return Err(self.error(
                DiagnosticCode::Limit,
                source,
                calls,
                "source map entry limit exceeded",
            ));
        }
        let base = SourceTrace {
            definition: source.into(),
            calls: calls.to_vec(),
            inputs: inputs.clone(),
        };
        self.artifact.sources.insert(native.into(), base);
        // Range only over this source subtree, not every predicate in the module.
        for (lowered, origin) in self.predicate_origins.range(source.to_owned()..) {
            if !lowered.starts_with(source) {
                break;
            }
            crate::authoring::control::charge(1, "predicate_source_projection")
                .map_err(|e| self.error(DiagnosticCode::Limit, source, calls, format!("{e:?}")))?;
            if lowered != source && !lowered[source.len()..].starts_with(['.', '[']) {
                continue;
            }
            if calls.len() + origin.calls.len() > MAX_CALL_DEPTH + 1
                || inputs.len() + origin.inputs.len() > 256
                || self.artifact.sources.len() >= 65_536
            {
                return Err(self.error(
                    DiagnosticCode::Limit,
                    source,
                    calls,
                    "expanded predicate source map limit exceeded",
                ));
            }
            super::budget::payload(
                &(native, lowered, origin, calls, &inputs),
                &mut self.copied_bytes,
                MAX_ARTIFACT_BYTES,
            )
            .map_err(|reason| self.error(DiagnosticCode::Limit, source, calls, reason))?;
            let mut origin = origin.clone();
            let mut full_calls = calls.to_vec();
            full_calls.extend(origin.calls);
            origin.calls = full_calls;
            for input in &inputs {
                if !origin.inputs.contains(input) {
                    origin.inputs.push(input.clone());
                }
            }
            self.artifact
                .sources
                .insert(format!("{native}{}", &lowered[source.len()..]), origin);
        }
        Ok(())
    }

    fn scope(
        &mut self,
        flow: &'a Flow,
        path: &str,
        native: &str,
        env: &Environment<'a>,
        calls: &[String],
        steps: &mut Vec<SkillStepDef>,
        skill: usize,
        depth: usize,
    ) -> Result<SkillFlowDef, Diagnostic> {
        let mut names = BTreeMap::new();
        self.allocate(flow, path, calls, &mut names)?;
        self.expand(flow, path, native, env, calls, &names, steps, skill, depth)
    }
    fn allocate(
        &mut self,
        flow: &Flow,
        path: &str,
        calls: &[String],
        names: &mut BTreeMap<String, String>,
    ) -> Result<(), Diagnostic> {
        if self.nodes >= MAX_FLOW_NODES {
            return Err(self.error(
                DiagnosticCode::Limit,
                path,
                calls,
                "expanded flow node limit exceeded",
            ));
        }
        names.insert(flow.id().into(), format!("n{}", self.nodes));
        self.nodes += 1;
        for (child, suffix) in children(flow) {
            self.allocate(child, &format!("{path}.{suffix}"), calls, names)?;
        }
        Ok(())
    }
    fn expand(
        &mut self,
        flow: &'a Flow,
        path: &str,
        native: &str,
        env: &Environment<'a>,
        calls: &[String],
        names: &BTreeMap<String, String>,
        steps: &mut Vec<SkillStepDef>,
        skill: usize,
        depth: usize,
    ) -> Result<SkillFlowDef, Diagnostic> {
        if depth >= MAX_FLOW_DEPTH || calls.len() > MAX_CALL_DEPTH {
            return Err(self.error(
                DiagnosticCode::Limit,
                path,
                calls,
                "expanded flow/call depth exceeded",
            ));
        }
        let id = names[flow.id()].clone();
        self.record(native, path, calls, vec![])?;
        Ok(match flow {
            Flow::Action {
                target,
                delivery,
                effects,
                modifiers,
                wait,
                ..
            } => {
                let target = self.resolve(target, &format!("{path}.target"), env, calls)?;
                let delivery = self.resolve(delivery, &format!("{path}.delivery"), env, calls)?;
                let step_path = format!("skills[{skill}].steps[{}]", steps.len());
                self.record(&step_path, path, calls, vec![])?;
                self.record(
                    &format!("{step_path}.target"),
                    &format!("{path}.target"),
                    calls,
                    target.sources,
                )?;
                self.record(
                    &format!("{step_path}.delivery"),
                    &format!("{path}.delivery"),
                    calls,
                    delivery.sources,
                )?;
                let mut compiled_effects = Vec::new();
                for (i, effect) in effects.iter().enumerate() {
                    let p = format!("{path}.effects[{i}]");
                    match effect {
                        Effect::Damage {
                            id, tags, amount, ..
                        }
                        | Effect::Heal { id, tags, amount } => {
                            self.charge_copy(&(id, tags), &p, calls)?;
                            let amount =
                                self.resolve(amount, &format!("{p}.amount"), env, calls)?;
                            let Value::Amount(value) = amount.value else {
                                unreachable!("checked argument type")
                            };
                            let op = match effect {
                                Effect::Damage { damage_type, .. } => SkillEffectDef::DamageExpr {
                                    amount: value.clone(),
                                    damage_type: *damage_type,
                                },
                                _ => SkillEffectDef::HealExpr {
                                    amount: value.clone(),
                                },
                            };
                            let mut item = AuthoredSkillEffect::new(id, op);
                            item.tags = tags.clone();
                            self.record(
                                &format!("{step_path}.effects[{}]", compiled_effects.len()),
                                &p,
                                calls,
                                amount.sources,
                            )?;
                            compiled_effects.push(item);
                        }
                        Effect::PersonalShieldGrant {
                            id,
                            tags,
                            amount,
                            duration,
                            priority,
                            recipients,
                            support,
                            replace_only_recipients,
                            source_creation,
                        } => {
                            self.charge_copy(&(id, tags, support, source_creation), &p, calls)?;
                            let amount =
                                self.resolve(amount, &format!("{p}.amount"), env, calls)?;
                            let duration =
                                self.resolve(duration, &format!("{p}.duration"), env, calls)?;
                            let Value::Amount(value) = amount.value else {
                                unreachable!("checked argument type")
                            };
                            let Value::Milliseconds(duration_ms) = duration.value else {
                                unreachable!("checked argument type")
                            };
                            let duration_ms = u32::try_from(*duration_ms).map_err(|_| {
                                let mut error = self.error(
                                    DiagnosticCode::Type,
                                    &format!("{p}.duration"),
                                    calls,
                                    "personal shield duration exceeds u32 milliseconds",
                                );
                                error.source.inputs = duration.sources.clone();
                                error
                            })?;
                            let mut item = AuthoredSkillEffect::new(id, SkillEffectDef::PersonalShieldGrant {
                                definition: crate::game::relic::structural::authoring::PersonalShieldGrantDef {
                                    amount: std::sync::Arc::new(value.clone()), duration_ms,
                                    priority: *priority, recipients: *recipients, support: support.clone(),
                                    replace_only_recipients: *replace_only_recipients,
                                    source_creation: source_creation.clone(),
                                },
                            });
                            item.tags = tags.clone();
                            let mut inputs = amount.sources;
                            inputs.extend(duration.sources);
                            self.record(
                                &format!("{step_path}.effects[{}]", compiled_effects.len()),
                                &p,
                                calls,
                                inputs,
                            )?;
                            compiled_effects.push(item);
                        }
                        Effect::Append { effects } => {
                            let value =
                                self.resolve(effects, &format!("{p}.effects"), env, calls)?;
                            let Value::Effects(items) = value.value else {
                                unreachable!("checked argument type")
                            };
                            for (index, item) in items.iter().enumerate() {
                                let native =
                                    format!("{step_path}.effects[{}]", compiled_effects.len());
                                self.record(&native, &p, calls, value.sources.clone())?;
                                // Keep the producing definition as the runtime origin,
                                // but native payload errors should select the literal
                                // supplied through shared values/parameters/inline input.
                                let mut inputs = value.sources.clone();
                                inputs.push(p.clone());
                                self.record(
                                    &format!("{native}.op"),
                                    &format!("{}.Effects[{index}].op", value.value_path),
                                    calls,
                                    inputs,
                                )?;
                                if matches!(
                                    &item.op,
                                    SkillEffectDef::Sustained {
                                        definition: SustainedEffectDef::State { .. }
                                            | SustainedEffectDef::PreparedState { .. }
                                    }
                                ) {
                                    self.record(
                                        &format!("{native}.state"),
                                        &format!(
                                            "{}.Effects[{index}].op.Sustained.definition.program",
                                            value.value_path
                                        ),
                                        calls,
                                        value.sources.clone(),
                                    )?;
                                }
                                compiled_effects.push(item.clone());
                            }
                        }
                    }
                }
                let Value::Target(target) = target.value else {
                    unreachable!("checked argument type")
                };
                let Value::Delivery(delivery) = delivery.value else {
                    unreachable!("checked argument type")
                };
                self.expand_action_modifiers(
                    modifiers,
                    path,
                    env,
                    calls,
                    &id,
                    &step_path,
                    delivery,
                    &compiled_effects,
                    skill,
                )?;
                steps.push(SkillStepDef {
                    id: id.clone(),
                    delay_ms: 0,
                    target: target.clone(),
                    delivery: delivery.clone(),
                    targeting: StepTargetingMode::default(),
                    when: SkillStepCondition::default(),
                    repeat: SkillStepRepeat::default(),
                    effects: compiled_effects,
                    presentation: SkillPresentationDef::default(),
                });
                SkillFlowDef::Action {
                    id: id.clone(),
                    step_id: id,
                    wait: *wait,
                }
            }
            Flow::Wait { duration, .. } => {
                let value = self.resolve(duration, &format!("{path}.duration"), env, calls)?;
                let Value::Milliseconds(duration_ms) = value.value else {
                    unreachable!("checked argument type")
                };
                self.record(native, path, calls, value.sources)?;
                SkillFlowDef::Wait {
                    id,
                    duration_ms: *duration_ms,
                }
            }
            Flow::Sequence { children, .. }
            | Flow::Parallel {
                branches: children, ..
            } => {
                let field = if matches!(flow, Flow::Sequence { .. }) {
                    "children"
                } else {
                    "branches"
                };
                let mut result = Vec::new();
                for (i, child) in children.iter().enumerate() {
                    result.push(self.expand(
                        child,
                        &format!("{path}.{field}[{i}]"),
                        &format!("{native}.{field}[{i}]"),
                        env,
                        calls,
                        names,
                        steps,
                        skill,
                        depth + 1,
                    )?);
                }
                if field == "children" {
                    SkillFlowDef::Sequence {
                        id,
                        children: result,
                    }
                } else {
                    SkillFlowDef::Parallel {
                        id,
                        branches: result,
                    }
                }
            }
            Flow::Branch {
                condition,
                then_node,
                else_node,
                ..
            } => {
                self.charge_conditions(
                    super::predicates::flow_nodes(condition),
                    &format!("{path}.condition"),
                    calls,
                )?;
                self.charge_copy(condition, &format!("{path}.condition"), calls)?;
                let mut condition = condition.clone();
                self.rename_condition(&mut condition, names, &format!("{path}.condition"), calls)?;
                SkillFlowDef::Branch {
                    id,
                    condition,
                    then_node: Box::new(self.expand(
                        then_node,
                        &format!("{path}.then_node"),
                        &format!("{native}.then_node"),
                        env,
                        calls,
                        names,
                        steps,
                        skill,
                        depth + 1,
                    )?),
                    else_node: Box::new(self.expand(
                        else_node,
                        &format!("{path}.else_node"),
                        &format!("{native}.else_node"),
                        env,
                        calls,
                        names,
                        steps,
                        skill,
                        depth + 1,
                    )?),
                }
            }
            Flow::Repeat {
                count,
                interval,
                body,
                ..
            } => {
                let count = self.resolve(count, &format!("{path}.count"), env, calls)?;
                let interval = self.resolve(interval, &format!("{path}.interval"), env, calls)?;
                self.record(
                    &format!("{native}.count"),
                    &format!("{path}.count"),
                    calls,
                    count.sources,
                )?;
                self.record(
                    &format!("{native}.interval_ms"),
                    &format!("{path}.interval"),
                    calls,
                    interval.sources,
                )?;
                let Value::Count(count) = count.value else {
                    unreachable!("checked argument type")
                };
                let Value::Milliseconds(interval_ms) = interval.value else {
                    unreachable!("checked argument type")
                };
                SkillFlowDef::Repeat {
                    id,
                    count: *count,
                    interval_ms: *interval_ms,
                    body: Box::new(self.expand(
                        body,
                        &format!("{path}.body"),
                        &format!("{native}.body"),
                        env,
                        calls,
                        names,
                        steps,
                        skill,
                        depth + 1,
                    )?),
                }
            }
            Flow::Call {
                definition,
                arguments,
                ..
            } => {
                let (index, definition) = self.definitions[definition.as_str()];
                let mut inner = Environment::new();
                for (i, argument) in arguments.iter().enumerate() {
                    inner.insert(
                        argument.name.as_str(),
                        self.resolve(
                            &argument.value,
                            &format!("{path}.arguments[{i}].value"),
                            env,
                            calls,
                        )?,
                    );
                }
                let mut calls = calls.to_vec();
                calls.push(path.into());
                let body = self.scope(
                    &definition.body,
                    &format!("definitions[{index}].body"),
                    &format!("{native}.children[0]"),
                    &inner,
                    &calls,
                    steps,
                    skill,
                    depth + 1,
                )?;
                SkillFlowDef::Sequence {
                    id,
                    children: vec![body],
                }
            }
            Flow::Unsupported { operation, .. } => {
                return Err(self.error(DiagnosticCode::Unsupported, path, calls, operation))
            }
        })
    }
}

fn children(flow: &Flow) -> Vec<(&Flow, String)> {
    match flow {
        Flow::Sequence { children, .. } => children
            .iter()
            .enumerate()
            .map(|(i, f)| (f, format!("children[{i}]")))
            .collect(),
        Flow::Parallel { branches, .. } => branches
            .iter()
            .enumerate()
            .map(|(i, f)| (f, format!("branches[{i}]")))
            .collect(),
        Flow::Branch {
            then_node,
            else_node,
            ..
        } => vec![
            (then_node, "then_node".into()),
            (else_node, "else_node".into()),
        ],
        Flow::Repeat { body, .. } => vec![(body, "body".into())],
        _ => vec![],
    }
}
