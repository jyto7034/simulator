//! Diagnostic-only use tracing for values rejected before native expansion.
//! Uses the same typed reference resolver as expansion; never clones payloads.
use super::*;

fn contains(parent: &str, path: &str) -> bool {
    parent == path
        || path
            .strip_prefix(parent)
            .is_some_and(|tail| tail.starts_with('.') || tail.starts_with('['))
}
impl<'a> Compiler<'a> {
    pub(super) fn input_error_context(&self, target: &str) -> Option<SourceTrace> {
        if !target.starts_with("values[") && !target.contains(".Literal.") {
            return None;
        }
        let mut budget = MAX_FLOW_NODES;
        for (i, skill) in self.module.skills.iter().enumerate() {
            if let Some(found) = self.input_use(
                &skill.body,
                &format!("skills[{i}].body"),
                target,
                &Environment::new(),
                &[format!("skills[{i}]")],
                &mut budget,
            ) {
                return Some(found);
            }
        }
        let bindings = self
            .module
            .equipment_attacks
            .iter()
            .enumerate()
            .map(|(i, b)| (format!("equipment_attacks[{i}]"), &b.program))
            .chain(
                self.module
                    .equipment_defenses
                    .iter()
                    .enumerate()
                    .map(|(i, b)| (format!("equipment_defenses[{i}]"), &b.program)),
            )
            .chain(
                self.module
                    .equipment_supports
                    .iter()
                    .enumerate()
                    .map(|(i, b)| (format!("equipment_supports[{i}]"), &b.program)),
            )
            .chain(
                self.module
                    .equipment_states
                    .iter()
                    .enumerate()
                    .map(|(i, b)| (format!("equipment_states[{i}]"), &b.program)),
            );
        for (path, argument) in bindings {
            let value = self
                .reference(
                    argument,
                    &format!("{path}.program"),
                    &Environment::new(),
                    &[],
                )
                .ok()?;
            if contains(&value.value_path, target) {
                return Some(SourceTrace {
                    definition: target.into(),
                    calls: vec![path],
                    inputs: value.sources,
                });
            }
        }
        None
    }
    fn input_use(
        &self,
        flow: &'a Flow,
        path: &str,
        target: &str,
        env: &Environment<'a>,
        calls: &[String],
        budget: &mut usize,
    ) -> Option<SourceTrace> {
        if *budget == 0 || calls.len() > MAX_CALL_DEPTH {
            return None;
        }
        *budget -= 1;
        match flow {
            Flow::Call {
                definition,
                arguments,
                ..
            } => {
                let (index, definition) = self.definitions.get(definition.as_str())?;
                let mut next = Environment::new();
                for (i, argument) in arguments.iter().enumerate() {
                    let value = self
                        .reference(
                            &argument.value,
                            &format!("{path}.arguments[{i}].value"),
                            env,
                            calls,
                        )
                        .ok()?;
                    next.insert(argument.name.as_str(), value);
                }
                let mut calls = calls.to_vec();
                calls.push(path.into());
                return self.input_use(
                    &definition.body,
                    &format!("definitions[{index}].body"),
                    target,
                    &next,
                    &calls,
                    budget,
                );
            }
            Flow::Action { effects, .. } => {
                for (i, effect) in effects.iter().enumerate() {
                    let (argument, field) = match effect {
                        Effect::Append { effects } => (effects, "effects"),
                        Effect::Damage { amount, .. }
                        | Effect::Heal { amount, .. }
                        | Effect::PersonalShieldGrant { amount, .. } => (amount, "amount"),
                    };
                    let site = format!("{path}.effects[{i}]");
                    let Some(value) = self
                        .reference(argument, &format!("{site}.{field}"), env, calls)
                        .ok()
                    else {
                        continue;
                    };
                    if contains(&value.value_path, target) {
                        let mut inputs = value.sources;
                        inputs.push(site);
                        return Some(SourceTrace {
                            definition: target.into(),
                            calls: calls.to_vec(),
                            inputs,
                        });
                    }
                }
            }
            _ => (),
        }
        for (child, suffix) in children(flow) {
            if let Some(found) = self.input_use(
                child,
                &format!("{path}.{suffix}"),
                target,
                env,
                calls,
                budget,
            ) {
                return Some(found);
            }
        }
        None
    }
}
