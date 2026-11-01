//! Pure, typed authoring definitions lowered before native validation.
use super::*;
use crate::game::ability::{
    condition::*, flow::*, state_program::*, sustained::SustainedEffectDef, SkillEffectDef,
};
use crate::game::value_expr::{self as expr, ValueUnit};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredicateContext {
    State,
    Flow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredicateType {
    Amount(ValueUnit),
    BoolField,
    FlowResult(ValueUnit),
    FlowLiteral(ValueUnit),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredicateParameter {
    pub name: String,
    pub value_type: PredicateType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<PredicateRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredicateRange {
    pub min: i64,
    pub max: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredicateDefinition {
    pub name: String,
    pub context: PredicateContext,
    pub parameters: Vec<PredicateParameter>,
    pub body: PredicateBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PredicateBody {
    Always(bool),
    EventVictimHadOwnerDot,
    Logic(ConditionLogic<PredicateBody>),
    Call(PredicateCall),
    Compare {
        left: PredicateArgument,
        comparison: FlowComparison,
        right: PredicateArgument,
    },
    Bool {
        field: PredicateArgument,
        equals: bool,
    },
    Result {
        result: PredicateArgument,
        comparison: FlowComparison,
        value: PredicateArgument,
    },
}

pub(super) type Origins = BTreeMap<String, SourceTrace>;
type Types = BTreeMap<String, (PredicateType, Option<PredicateRange>)>;
type Bindings = BTreeMap<String, (PredicateValue, Vec<String>)>;
enum Lowered {
    State(StateCondition),
    Flow(FlowCondition),
}

struct Resolver<'a> {
    artifact: &'a Artifact,
    definitions: &'a [PredicateDefinition],
    names: BTreeMap<&'a str, usize>,
    origins: Origins,
    nodes: usize,
    expanded: usize,
    bytes: usize,
    version: u32,
}

pub(super) fn lower(module: &mut Module, artifact: &Artifact) -> Result<Origins, Diagnostic> {
    let definitions = std::mem::take(&mut module.predicates);
    let mut resolver = Resolver {
        artifact,
        definitions: &definitions,
        names: BTreeMap::new(),
        origins: BTreeMap::new(),
        nodes: 0,
        expanded: 0,
        bytes: 0,
        version: module.version,
    };
    if !definitions.is_empty() {
        resolver.feature("predicates")?;
    }
    if definitions.len() > 256 {
        return Err(resolver.error(
            DiagnosticCode::Limit,
            "predicates",
            "predicate definition limit exceeded",
        ));
    }
    for (i, definition) in definitions.iter().enumerate() {
        resolver.identifier(&definition.name, &format!("predicates[{i}].name"))?;
        if resolver.names.insert(&definition.name, i).is_some() {
            return Err(resolver.error(
                DiagnosticCode::Reference,
                &format!("predicates[{i}].name"),
                "duplicate predicate",
            ));
        }
    }
    for (i, definition) in definitions.iter().enumerate() {
        let path = format!("predicates[{i}]");
        let mut types = Types::new();
        if definition.parameters.len() > 32 {
            return Err(resolver.error(
                DiagnosticCode::Limit,
                &path,
                "predicate parameter limit exceeded",
            ));
        }
        for (j, parameter) in definition.parameters.iter().enumerate() {
            resolver.identifier(&parameter.name, &format!("{path}.parameters[{j}].name"))?;
            if matches!(&parameter.value_type, PredicateType::FlowResult(unit) | PredicateType::FlowLiteral(unit) if !matches!(unit, ValueUnit::Count | ValueUnit::Magnitude))
            {
                return Err(resolver.error(
                    DiagnosticCode::Type,
                    &format!("{path}.parameters[{j}].value_type"),
                    "Flow result/literal inputs support Count or Magnitude",
                ));
            }
            if let Some(range) = parameter.range {
                if range.min > range.max
                    || !matches!(
                        parameter.value_type,
                        PredicateType::Amount(_) | PredicateType::FlowLiteral(_)
                    )
                {
                    return Err(resolver.error(
                        DiagnosticCode::Type,
                        &format!("{path}.parameters[{j}].range"),
                        "range requires an ordered numeric Amount/FlowLiteral interval",
                    ));
                }
            }
            if types
                .insert(
                    parameter.name.clone(),
                    (parameter.value_type.clone(), parameter.range),
                )
                .is_some()
            {
                return Err(resolver.error(
                    DiagnosticCode::Reference,
                    &path,
                    "duplicate predicate parameter",
                ));
            }
        }
        resolver.check_body(
            &definition.body,
            definition.context,
            &types,
            &format!("{path}.body"),
            1,
        )?;
    }
    let mut done = BTreeMap::new();
    for i in 0..definitions.len() {
        resolver.cycles(i, &mut Vec::new(), &mut done)?;
    }
    for (i, value) in module.values.iter_mut().enumerate() {
        resolver.value(&mut value.value, &format!("values[{i}].value"), 1)?;
    }
    for (i, definition) in module.definitions.iter_mut().enumerate() {
        resolver.flow(&mut definition.body, &format!("definitions[{i}].body"), 1)?;
    }
    for (i, skill) in module.skills.iter_mut().enumerate() {
        resolver.flow(&mut skill.body, &format!("skills[{i}].body"), 1)?;
    }
    for (i, binding) in module.equipment_attacks.iter_mut().enumerate() {
        resolver.argument(
            &mut binding.program,
            &format!("equipment_attacks[{i}].program"),
            1,
        )?;
    }
    for (i, binding) in module.equipment_defenses.iter_mut().enumerate() {
        resolver.argument(
            &mut binding.program,
            &format!("equipment_defenses[{i}].program"),
            1,
        )?;
    }
    for (i, binding) in module.equipment_supports.iter_mut().enumerate() {
        resolver.argument(
            &mut binding.program,
            &format!("equipment_supports[{i}].program"),
            1,
        )?;
    }
    for (i, binding) in module.equipment_states.iter_mut().enumerate() {
        resolver.argument(
            &mut binding.program,
            &format!("equipment_states[{i}].program"),
            1,
        )?;
    }
    let origins = resolver.origins;
    module.predicates = definitions;
    Ok(origins)
}

impl Resolver<'_> {
    fn bind(
        &mut self,
        argument: &PredicateArgument,
        env: &Bindings,
        path: &str,
    ) -> Result<(PredicateValue, Vec<String>), Diagnostic> {
        match argument {
            PredicateArgument::Literal(value) => {
                self.retain(value, path)?;
                Ok((value.clone(), vec![path.into()]))
            }
            PredicateArgument::Parameter(name) => {
                let (value, sources) = env.get(name).ok_or_else(|| {
                    self.error(
                        DiagnosticCode::Reference,
                        path,
                        format!("unbound predicate parameter '{name}'"),
                    )
                })?;
                self.retain(value, path)?;
                let mut sources = sources.clone();
                sources.push(path.into());
                Ok((value.clone(), sources))
            }
            PredicateArgument::Unresolved(reason) => {
                Err(self.error(DiagnosticCode::Unresolved, path, reason))
            }
        }
    }
    fn expand_call(
        &mut self,
        call: &PredicateCall,
        context: PredicateContext,
        env: &Bindings,
        path: &str,
        output: &str,
        calls: &[String],
        depth: usize,
    ) -> Result<Lowered, Diagnostic> {
        self.feature(path)?;
        self.charge(path, depth)?;
        if calls.len() >= MAX_CALL_DEPTH {
            return Err(self.error(DiagnosticCode::Limit, path, "predicate call depth exceeded"));
        }
        let mut types = Types::new();
        for (name, (value, _)) in env {
            types.insert(
                name.clone(),
                (self.value_type(value, path)?, literal_range(value)),
            );
        }
        let index = self.target(call, context, &types, path)?;
        let mut bindings = Bindings::new();
        for (i, argument) in call.arguments.iter().enumerate() {
            let value = self.bind(
                &argument.value,
                env,
                &format!("{path}.arguments[{i}].value"),
            )?;
            bindings.insert(argument.name.clone(), value);
        }
        let mut calls = calls.to_vec();
        calls.push(path.into());
        let body = &self.definitions[index].body;
        self.expand_body(
            body,
            context,
            &bindings,
            &format!("predicates[{index}].body"),
            output,
            &calls,
            depth + 1,
        )
        .map_err(|mut error| {
            if error.source.calls.is_empty() {
                error.source.calls = calls;
            }
            error
        })
    }
    fn expand_body(
        &mut self,
        body: &PredicateBody,
        context: PredicateContext,
        env: &Bindings,
        path: &str,
        output: &str,
        calls: &[String],
        depth: usize,
    ) -> Result<Lowered, Diagnostic> {
        self.charge(path, depth)?;
        self.expanded += 1;
        if self.expanded > MAX_FLOW_NODES {
            return Err(self.error(
                DiagnosticCode::Limit,
                path,
                "expanded predicate node limit exceeded",
            ));
        }
        let mut inputs = Vec::new();
        for (_, sources) in env.values() {
            for source in sources {
                if !inputs.contains(source) {
                    inputs.push(source.clone());
                }
            }
        }
        let trace = SourceTrace {
            definition: authored_path(path),
            calls: calls.iter().map(|p| authored_path(p)).collect(),
            inputs: inputs.iter().map(|p| authored_path(p)).collect(),
        };
        self.retain(&trace, path)?;
        self.origins.insert(output.into(), trace);
        Ok(match body {
            PredicateBody::Always(value) => match context {
                PredicateContext::Flow => Lowered::Flow(FlowCondition::Always(*value)),
                PredicateContext::State => Lowered::State(StateCondition::Always(*value)),
            },
            PredicateBody::EventVictimHadOwnerDot => {
                if context != PredicateContext::State {
                    return Err(self.error(
                        DiagnosticCode::Type,
                        path,
                        "kill event input requires State context",
                    ));
                }
                Lowered::State(StateCondition::EventVictimHadOwnerDot)
            }
            PredicateBody::Call(call) => {
                return self.expand_call(call, context, env, path, output, calls, depth + 1)
            }
            PredicateBody::Logic(logic) => match context {
                PredicateContext::State => {
                    Lowered::State(StateCondition::Logic(logic.try_map(|i, child| {
                        match self.expand_body(
                            child,
                            context,
                            env,
                            &format!("{path}.Logic{}", logic.child_suffix(i)),
                            &format!("{output}.Logic{}", logic.child_suffix(i)),
                            calls,
                            depth + 1,
                        )? {
                            Lowered::State(value) => Ok(value),
                            _ => Err(self.error(
                                DiagnosticCode::Type,
                                path,
                                "Flow leaf in State predicate",
                            )),
                        }
                    })?))
                }
                PredicateContext::Flow => {
                    Lowered::Flow(FlowCondition::Logic(logic.try_map(|i, child| {
                        match self.expand_body(
                            child,
                            context,
                            env,
                            &format!("{path}.Logic{}", logic.child_suffix(i)),
                            &format!("{output}.Logic{}", logic.child_suffix(i)),
                            calls,
                            depth + 1,
                        )? {
                            Lowered::Flow(value) => Ok(value),
                            _ => Err(self.error(
                                DiagnosticCode::Type,
                                path,
                                "State leaf in Flow predicate",
                            )),
                        }
                    })?))
                }
            },
            PredicateBody::Compare {
                left,
                comparison,
                right,
            } => {
                let (PredicateValue::Amount(left), _) =
                    self.bind(left, env, &format!("{path}.Compare.left"))?
                else {
                    return Err(self.error(DiagnosticCode::Type, path, "expected Amount input"));
                };
                let (PredicateValue::Amount(right), _) =
                    self.bind(right, env, &format!("{path}.Compare.right"))?
                else {
                    return Err(self.error(DiagnosticCode::Type, path, "expected Amount input"));
                };
                Lowered::State(StateCondition::Compare {
                    left,
                    comparison: *comparison,
                    right,
                })
            }
            PredicateBody::Bool { field, equals } => {
                let (PredicateValue::BoolField(field), _) =
                    self.bind(field, env, &format!("{path}.Bool.field"))?
                else {
                    return Err(self.error(DiagnosticCode::Type, path, "expected BoolField input"));
                };
                Lowered::State(StateCondition::Bool {
                    field,
                    equals: *equals,
                })
            }
            PredicateBody::Result {
                result,
                comparison,
                value,
            } => {
                let (PredicateValue::FlowResult(result), _) =
                    self.bind(result, env, &format!("{path}.Result.result"))?
                else {
                    return Err(self.error(
                        DiagnosticCode::Type,
                        path,
                        "expected FlowResult input",
                    ));
                };
                let (PredicateValue::FlowLiteral(value), _) =
                    self.bind(value, env, &format!("{path}.Result.value"))?
                else {
                    return Err(self.error(
                        DiagnosticCode::Type,
                        path,
                        "expected FlowLiteral input",
                    ));
                };
                Lowered::Flow(if let Some(effect_id) = result.effect_id {
                    FlowCondition::EffectCompare {
                        node_id: result.node_id,
                        effect_id,
                        metric: result.metric,
                        comparison: *comparison,
                        value,
                    }
                } else {
                    FlowCondition::Compare {
                        node_id: result.node_id,
                        metric: result.metric,
                        comparison: *comparison,
                        value,
                    }
                })
            }
        })
    }
    fn state_condition(
        &mut self,
        condition: &mut StateCondition,
        path: &str,
        depth: usize,
    ) -> Result<(), Diagnostic> {
        self.charge(path, depth)?;
        match condition {
            StateCondition::Always(_) => self.feature(path)?,
            StateCondition::Predicate(call) => {
                let Lowered::State(value) = self.expand_call(
                    call,
                    PredicateContext::State,
                    &Bindings::new(),
                    path,
                    path,
                    &[],
                    depth + 1,
                )?
                else {
                    return Err(self.error(DiagnosticCode::Type, path, "expected State predicate"));
                };
                *condition = value;
            }
            StateCondition::Logic(logic) => {
                self.feature(path)?;
                if logic.children().is_empty() {
                    return Err(self.error(
                        DiagnosticCode::Type,
                        path,
                        "boolean group must be nonempty",
                    ));
                }
                let suffixes: Vec<_> = (0..logic.children().len())
                    .map(|i| logic.child_suffix(i))
                    .collect();
                for (suffix, child) in suffixes.iter().zip(logic.children_mut()) {
                    self.state_condition(child, &format!("{path}.Logic{suffix}"), depth + 1)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    fn flow_condition(
        &mut self,
        condition: &mut FlowCondition,
        path: &str,
        depth: usize,
    ) -> Result<(), Diagnostic> {
        self.charge(path, depth)?;
        match condition {
            FlowCondition::Predicate(call) => {
                let Lowered::Flow(value) = self.expand_call(
                    call,
                    PredicateContext::Flow,
                    &Bindings::new(),
                    path,
                    path,
                    &[],
                    depth + 1,
                )?
                else {
                    return Err(self.error(DiagnosticCode::Type, path, "expected Flow predicate"));
                };
                *condition = value;
            }
            FlowCondition::Logic(logic) => {
                self.feature(path)?;
                if logic.children().is_empty() {
                    return Err(self.error(
                        DiagnosticCode::Type,
                        path,
                        "boolean group must be nonempty",
                    ));
                }
                let suffixes: Vec<_> = (0..logic.children().len())
                    .map(|i| logic.child_suffix(i))
                    .collect();
                for (suffix, child) in suffixes.iter().zip(logic.children_mut()) {
                    self.flow_condition(child, &format!("{path}.Logic{suffix}"), depth + 1)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    fn block(&mut self, ops: &mut [StateOp], path: &str, depth: usize) -> Result<(), Diagnostic> {
        for (i, op) in ops.iter_mut().enumerate() {
            let path = format!("{path}[{i}]");
            self.charge(&path, depth)?;
            if let StateOp::If {
                condition,
                then_ops,
                else_ops,
            } = op
            {
                self.state_condition(condition, &format!("{path}.condition"), depth + 1)?;
                self.block(then_ops, &format!("{path}.then"), depth + 1)?;
                self.block(else_ops, &format!("{path}.else"), depth + 1)?;
            } else if let StateOp::Chance { then_ops, else_ops, .. } = op {
                self.block(then_ops, &format!("{path}.then"), depth + 1)?;
                self.block(else_ops, &format!("{path}.else"), depth + 1)?;
            }
        }
        Ok(())
    }
    fn state(
        &mut self,
        program: &mut StateProgram,
        path: &str,
        depth: usize,
    ) -> Result<(), Diagnostic> {
        for (i, handler) in program.handlers.iter_mut().enumerate() {
            self.block(
                &mut handler.body,
                &format!("{path}.handlers[{i}].body"),
                depth + 1,
            )?;
        }
        Ok(())
    }
    fn value(&mut self, value: &mut Value, path: &str, depth: usize) -> Result<(), Diagnostic> {
        match value {
            Value::StateProgram(program) => {
                self.state(program, &format!("{path}.StateProgram"), depth + 1)?
            }
            Value::AttackProgram(program) => {
                for (i, reaction) in program.reactions.iter_mut().enumerate() {
                    self.block(
                        &mut reaction.body,
                        &format!("{path}.AttackProgram.reactions[{i}].body"),
                        depth + 1,
                    )?;
                }
            }
            Value::Effects(effects) => {
                for (i, effect) in effects.iter_mut().enumerate() {
                    if let SkillEffectDef::Sustained {
                        definition:
                            SustainedEffectDef::State { program, .. }
                            | SustainedEffectDef::PreparedState { program, .. },
                    } = &mut effect.op
                    {
                        self.state(
                            Arc::make_mut(program),
                            &format!("{path}.Effects[{i}].op.Sustained.definition.program"),
                            depth + 1,
                        )?;
                    }
                }
            }
            _ => (),
        }
        Ok(())
    }
    fn argument(
        &mut self,
        argument: &mut Argument,
        path: &str,
        depth: usize,
    ) -> Result<(), Diagnostic> {
        if let Argument::Literal(value) = argument {
            self.value(value, &format!("{path}.Literal"), depth + 1)?;
        }
        Ok(())
    }
    fn flow(&mut self, flow: &mut Flow, path: &str, depth: usize) -> Result<(), Diagnostic> {
        self.charge(path, depth)?;
        match flow {
            Flow::Action {
                target,
                delivery,
                effects,
                modifiers,
                ..
            } => {
                self.argument(target, &format!("{path}.target"), depth + 1)?;
                self.argument(delivery, &format!("{path}.delivery"), depth + 1)?;
                for (i, effect) in effects.iter_mut().enumerate() {
                    let p = format!("{path}.effects[{i}]");
                    match effect {
                        Effect::Damage { amount, .. } | Effect::Heal { amount, .. } => {
                            self.argument(amount, &format!("{p}.amount"), depth + 1)?
                        }
                        Effect::PersonalShieldGrant {
                            amount, duration, ..
                        } => {
                            self.argument(amount, &format!("{p}.amount"), depth + 1)?;
                            self.argument(duration, &format!("{p}.duration"), depth + 1)?;
                        }
                        Effect::Append { effects } => {
                            self.argument(effects, &format!("{p}.effects"), depth + 1)?
                        }
                    }
                }
                for (i, modifier) in modifiers.iter_mut().enumerate() {
                    let p = format!("{path}.modifiers[{i}].change");
                    let args: Vec<(&str, &mut Argument)> = match &mut modifier.change {
                        ActionChange::Range { pattern } => vec![("pattern", pattern)],
                        ActionChange::AreaDuration { percent }
                        | ActionChange::Duration { percent, .. }
                        | ActionChange::SupportRadius { percent, .. } => vec![("percent", percent)],
                        ActionChange::AreaRampRate {
                            numerator,
                            denominator,
                            ..
                        } => vec![("numerator", numerator), ("denominator", denominator)],
                        ActionChange::AreaControlVulnerability {
                            percent, duration, ..
                        }
                        | ActionChange::ControlVulnerability {
                            percent, duration, ..
                        } => vec![("percent", percent), ("duration", duration)],
                        ActionChange::TauntDamage {
                            taken_percent,
                            basic_percent,
                            ..
                        } => vec![
                            ("taken_percent", taken_percent),
                            ("basic_percent", basic_percent),
                        ],
                    };
                    for (field, argument) in args {
                        self.argument(argument, &format!("{p}.{field}"), depth + 1)?;
                    }
                }
            }
            Flow::Sequence { children, .. } => {
                for (i, child) in children.iter_mut().enumerate() {
                    self.flow(child, &format!("{path}.children[{i}]"), depth + 1)?;
                }
            }
            Flow::Parallel { branches, .. } => {
                for (i, child) in branches.iter_mut().enumerate() {
                    self.flow(child, &format!("{path}.branches[{i}]"), depth + 1)?;
                }
            }
            Flow::Branch {
                condition,
                then_node,
                else_node,
                ..
            } => {
                self.flow_condition(condition, &format!("{path}.condition"), depth + 1)?;
                self.flow(then_node, &format!("{path}.then_node"), depth + 1)?;
                self.flow(else_node, &format!("{path}.else_node"), depth + 1)?;
            }
            Flow::Wait { duration, .. } => {
                self.argument(duration, &format!("{path}.duration"), depth + 1)?
            }
            Flow::Repeat {
                count,
                interval,
                body,
                ..
            } => {
                self.argument(count, &format!("{path}.count"), depth + 1)?;
                self.argument(interval, &format!("{path}.interval"), depth + 1)?;
                self.flow(body, &format!("{path}.body"), depth + 1)?;
            }
            Flow::Call { arguments, .. } => {
                for (i, argument) in arguments.iter_mut().enumerate() {
                    self.argument(
                        &mut argument.value,
                        &format!("{path}.arguments[{i}].value"),
                        depth + 1,
                    )?;
                }
            }
            Flow::Unsupported { .. } => (),
        }
        Ok(())
    }
}

impl Resolver<'_> {
    fn error(&self, code: DiagnosticCode, path: &str, reason: impl Into<String>) -> Diagnostic {
        self.artifact.diagnostic(code, &authored_path(path), reason)
    }
    fn feature(&self, path: &str) -> Result<(), Diagnostic> {
        if self.version < 2 {
            return Err(self.error(
                DiagnosticCode::Version,
                path,
                "boolean composition and predicates require model version 2",
            ));
        }
        Ok(())
    }
    fn identifier(&self, name: &str, path: &str) -> Result<(), Diagnostic> {
        if name.is_empty()
            || name.len() > 128
            || name.trim() != name
            || name.chars().any(char::is_control)
        {
            return Err(self.error(
                DiagnosticCode::Reference,
                path,
                "predicate identifier must be nonempty, trimmed, control-free, at most 128 bytes",
            ));
        }
        Ok(())
    }
    fn charge(&mut self, path: &str, depth: usize) -> Result<(), Diagnostic> {
        self.nodes += 1;
        crate::authoring::control::charge(1, "predicate_visit")
            .map_err(|e| self.error(DiagnosticCode::Limit, path, format!("{e:?}")))?;
        if depth > MAX_FLOW_DEPTH || self.nodes > MAX_INPUT_BYTES {
            return Err(self.error(
                DiagnosticCode::Limit,
                path,
                "predicate depth/node limit exceeded",
            ));
        }
        Ok(())
    }
    fn retain<T: Serialize>(&mut self, value: &T, path: &str) -> Result<(), Diagnostic> {
        super::budget::payload(value, &mut self.bytes, MAX_INPUT_BYTES)
            .map_err(|reason| self.error(DiagnosticCode::Limit, path, reason))
    }

    fn value_type(&self, value: &PredicateValue, path: &str) -> Result<PredicateType, Diagnostic> {
        Ok(match value {
            PredicateValue::BoolField(name) => {
                self.identifier(name, path)?;
                PredicateType::BoolField
            }
            PredicateValue::FlowLiteral(value) => PredicateType::FlowLiteral(literal_unit(*value)),
            PredicateValue::FlowResult(result) => {
                self.identifier(&result.node_id, path)?;
                if let Some(effect) = &result.effect_id {
                    self.identifier(effect, path)?;
                }
                PredicateType::FlowResult(metric_unit(result.metric))
            }
            PredicateValue::Amount(amount) => {
                use crate::game::ability::authoring::*;
                let source = EffectSource {
                    source_file: self.artifact.document.clone(),
                    site: EffectSite {
                        action: ActionSite {
                            definition_id: "predicate".into(),
                            action_id: "condition".into(),
                        },
                        effect_id: "value".into(),
                    },
                    field_path: path.into(),
                    origin_definition_id: "predicate".into(),
                    origin: EffectOrigin::Original,
                };
                let mut capabilities = expr::DeliveryCapabilities::constant_only();
                capabilities.state_context = true;
                capabilities.target_at_release = true;
                capabilities.target_at_consume = true;
                capabilities.deadline = true;
                if amount.moment != expr::EvalMoment::Live {
                    return Err(self.error(
                        DiagnosticCode::Type,
                        path,
                        "predicate state amounts require explicit Live evaluation",
                    ));
                }
                let plan =
                    expr::compile(amount, &source, capabilities, &expr::ExprLimits::default())
                        .map_err(|e| self.error(DiagnosticCode::Type, path, e.to_string()))?;
                PredicateType::Amount(plan.result_unit().clone())
            }
        })
    }
    fn argument_type(
        &self,
        argument: &PredicateArgument,
        types: &Types,
        path: &str,
    ) -> Result<PredicateType, Diagnostic> {
        match argument {
            PredicateArgument::Literal(value) => self.value_type(value, path),
            PredicateArgument::Parameter(name) => types
                .get(name)
                .map(|(kind, _)| kind.clone())
                .ok_or_else(|| {
                    self.error(
                        DiagnosticCode::Reference,
                        path,
                        format!("unknown predicate parameter '{name}'"),
                    )
                }),
            PredicateArgument::Unresolved(reason) => {
                Err(self.error(DiagnosticCode::Unresolved, path, reason))
            }
        }
    }
    fn target(
        &self,
        call: &PredicateCall,
        context: PredicateContext,
        types: &Types,
        path: &str,
    ) -> Result<usize, Diagnostic> {
        let index = *self.names.get(call.name.as_str()).ok_or_else(|| {
            self.error(
                DiagnosticCode::Reference,
                path,
                format!("unknown predicate '{}'", call.name),
            )
        })?;
        let definition = &self.definitions[index];
        if definition.context != context {
            return Err(self.error(
                DiagnosticCode::Type,
                path,
                "predicate context does not match caller",
            ));
        }
        let mut names = BTreeSet::new();
        for (i, argument) in call.arguments.iter().enumerate() {
            let p = format!("{path}.arguments[{i}].value");
            if !names.insert(&argument.name) {
                return Err(self.error(
                    DiagnosticCode::Reference,
                    &p,
                    "duplicate predicate argument",
                ));
            }
            let parameter = definition
                .parameters
                .iter()
                .find(|p| p.name == argument.name)
                .ok_or_else(|| {
                    self.error(DiagnosticCode::Reference, &p, "unknown predicate input")
                })?;
            let actual = self.argument_type(&argument.value, types, &p)?;
            if actual != parameter.value_type {
                return Err(self.error(
                    DiagnosticCode::Type,
                    &p,
                    format!("expected {:?}, got {actual:?}", parameter.value_type),
                ));
            }
            if let Some(required) = parameter.range {
                let actual = match &argument.value {
                    PredicateArgument::Literal(value) => literal_range(value),
                    PredicateArgument::Parameter(name) => {
                        types.get(name).and_then(|(_, range)| *range)
                    }
                    PredicateArgument::Unresolved(_) => None,
                };
                if !actual
                    .is_some_and(|actual| actual.min >= required.min && actual.max <= required.max)
                {
                    return Err(self.error(DiagnosticCode::Type, &p, format!("input must prove range {}..{} using a numeric literal or a constrained forwarded parameter", required.min, required.max)));
                }
            }
        }
        if names.len() != definition.parameters.len() {
            return Err(self.error(DiagnosticCode::Reference, path, "missing predicate input"));
        }
        Ok(index)
    }
    fn check_body(
        &mut self,
        body: &PredicateBody,
        context: PredicateContext,
        types: &Types,
        path: &str,
        depth: usize,
    ) -> Result<(), Diagnostic> {
        self.charge(path, depth)?;
        match body {
            PredicateBody::Always(_) => (),
            PredicateBody::EventVictimHadOwnerDot => {
                if context != PredicateContext::State {
                    return Err(self.error(
                        DiagnosticCode::Type,
                        path,
                        "kill event input requires State context",
                    ));
                }
            }
            PredicateBody::Logic(logic) => {
                if logic.children().is_empty() {
                    return Err(self.error(
                        DiagnosticCode::Type,
                        path,
                        "boolean group must be nonempty",
                    ));
                }
                for (i, child) in logic.children().iter().enumerate() {
                    self.check_body(
                        child,
                        context,
                        types,
                        &format!("{path}.Logic{}", logic.child_suffix(i)),
                        depth + 1,
                    )?;
                }
            }
            PredicateBody::Call(call) => {
                self.target(call, context, types, path)?;
            }
            PredicateBody::Compare { left, right, .. } => {
                let left = self.argument_type(left, types, &format!("{path}.Compare.left"))?;
                let right = self.argument_type(right, types, &format!("{path}.Compare.right"))?;
                if context != PredicateContext::State
                    || !matches!(left, PredicateType::Amount(_))
                    || left != right
                {
                    return Err(self.error(
                        DiagnosticCode::Type,
                        path,
                        "State comparison requires matching Amount units",
                    ));
                }
            }
            PredicateBody::Bool { field, .. } => {
                if context != PredicateContext::State
                    || self.argument_type(field, types, &format!("{path}.Bool.field"))?
                        != PredicateType::BoolField
                {
                    return Err(self.error(
                        DiagnosticCode::Type,
                        path,
                        "State Bool requires a BoolField input",
                    ));
                }
            }
            PredicateBody::Result { result, value, .. } => {
                let result = self.argument_type(result, types, &format!("{path}.Result.result"))?;
                let value = self.argument_type(value, types, &format!("{path}.Result.value"))?;
                if context != PredicateContext::Flow
                    || !matches!((&result, &value), (PredicateType::FlowResult(a), PredicateType::FlowLiteral(b)) if a == b)
                {
                    return Err(self.error(
                        DiagnosticCode::Type,
                        path,
                        "Flow comparison requires matching result/literal units",
                    ));
                }
            }
        }
        Ok(())
    }
    fn cycles(
        &self,
        index: usize,
        stack: &mut Vec<usize>,
        done: &mut BTreeMap<usize, usize>,
    ) -> Result<usize, Diagnostic> {
        if stack.contains(&index) {
            return Err(self.error(
                DiagnosticCode::Reference,
                &format!("predicates[{index}]"),
                "recursive predicate call",
            ));
        }
        if stack.len() >= MAX_CALL_DEPTH {
            return Err(self.error(
                DiagnosticCode::Limit,
                &format!("predicates[{index}]"),
                "predicate call depth exceeded",
            ));
        }
        if let Some(depth) = done.get(&index) {
            return Ok(*depth);
        }
        stack.push(index);
        fn calls(body: &PredicateBody, targets: &mut Vec<String>) {
            match body {
                PredicateBody::Call(call) => targets.push(call.name.clone()),
                PredicateBody::Logic(logic) => {
                    for child in logic.children() {
                        calls(child, targets);
                    }
                }
                _ => (),
            }
        }
        let mut targets = Vec::new();
        calls(&self.definitions[index].body, &mut targets);
        let mut depth = 1;
        for target in targets {
            depth = depth.max(1 + self.cycles(self.names[target.as_str()], stack, done)?);
        }
        stack.pop();
        if depth > MAX_CALL_DEPTH {
            return Err(self.error(
                DiagnosticCode::Limit,
                &format!("predicates[{index}]"),
                "predicate call depth exceeded",
            ));
        }
        done.insert(index, depth);
        Ok(depth)
    }
}

fn metric_unit(metric: FlowMetric) -> ValueUnit {
    if metric == FlowMetric::HpLost {
        ValueUnit::Magnitude
    } else {
        ValueUnit::Count
    }
}
fn literal_unit(value: FlowLiteral) -> ValueUnit {
    match value {
        FlowLiteral::Hp(_) => ValueUnit::Magnitude,
        FlowLiteral::Count(_) => ValueUnit::Count,
    }
}

pub(super) fn flow_nodes(condition: &FlowCondition) -> usize {
    1 + match condition {
        FlowCondition::Logic(logic) => logic.children().iter().map(flow_nodes).sum(),
        _ => 0,
    }
}
fn state_nodes(condition: &StateCondition) -> usize {
    1 + match condition {
        StateCondition::Logic(logic) => logic.children().iter().map(state_nodes).sum(),
        _ => 0,
    }
}
fn block_nodes(ops: &[StateOp]) -> usize {
    ops.iter()
        .map(|op| match op {
            StateOp::If {
                condition,
                then_ops,
                else_ops,
            } => state_nodes(condition) + block_nodes(then_ops) + block_nodes(else_ops),
            StateOp::Chance { then_ops, else_ops, .. } => block_nodes(then_ops) + block_nodes(else_ops),
            _ => 0,
        })
        .sum()
}
fn program_nodes(program: &StateProgram) -> usize {
    program
        .handlers
        .iter()
        .map(|handler| block_nodes(&handler.body))
        .sum()
}
pub(super) fn value_nodes(value: &Value) -> usize {
    match value {
        Value::StateProgram(program) => program_nodes(program),
        Value::AttackProgram(program) => program
            .reactions
            .iter()
            .map(|reaction| block_nodes(&reaction.body))
            .sum(),
        Value::Effects(effects) => effects
            .iter()
            .map(|effect| match &effect.op {
                SkillEffectDef::Sustained {
                    definition:
                        SustainedEffectDef::State { program, .. }
                        | SustainedEffectDef::PreparedState { program, .. },
                } => program_nodes(program),
                _ => 0,
            })
            .sum(),
        _ => 0,
    }
}

fn literal_range(value: &PredicateValue) -> Option<PredicateRange> {
    let value = match value {
        PredicateValue::FlowLiteral(FlowLiteral::Count(value) | FlowLiteral::Hp(value)) => {
            i64::try_from(*value).ok()?
        }
        PredicateValue::Amount(amount) => {
            let expr::ValueExpr::Const(value) = &amount.root else {
                return None;
            };
            i64::try_from(value.value).ok()?
        }
        _ => return None,
    };
    Some(PredicateRange {
        min: value,
        max: value,
    })
}

fn authored_path(path: &str) -> String {
    path.replace(".then[", ".then_ops[")
        .replace(".else[", ".else_ops[")
}
