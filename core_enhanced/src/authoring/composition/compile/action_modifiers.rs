use super::*;
use crate::game::ability::action as native;

impl<'a> Compiler<'a> {
    pub(super) fn check_skill_modifiers(
        &self,
        modifiers: &[SkillModifier],
        path: &str,
    ) -> Result<(), Diagnostic> {
        if modifiers.len() > 128 {
            return Err(self.error(
                DiagnosticCode::Limit,
                &format!("{path}.modifiers"),
                &[],
                "skill modifier limit exceeded",
            ));
        }
        let mut ids = BTreeSet::new();
        for (index, modifier) in modifiers.iter().enumerate() {
            let p = format!("{path}.modifiers[{index}]");
            self.identifier(&modifier.id, &format!("{p}.id"))?;
            self.identifier(&modifier.relic_definition, &format!("{p}.relic_definition"))?;
            if !ids.insert(&modifier.id) {
                return Err(self.error(
                    DiagnosticCode::Reference,
                    &format!("{p}.id"),
                    &[],
                    "duplicate skill modifier ID",
                ));
            }
            if let SkillChange::Multihit {
                increment_percent,
                cap_percent,
            } = &modifier.change
            {
                for (field, argument) in [
                    ("increment_percent", increment_percent),
                    ("cap_percent", cap_percent),
                ] {
                    self.expect_type(
                        argument,
                        &format!("{p}.change.Multihit.{field}"),
                        &Types::new(),
                        ValueType::Count,
                    )?;
                }
            }
        }
        Ok(())
    }

    pub(super) fn expand_skill_modifiers(
        &mut self,
        modifiers: &'a [SkillModifier],
        path: &str,
        skill: usize,
    ) -> Result<(), Diagnostic> {
        for (index, modifier) in modifiers.iter().enumerate() {
            let p = format!("{path}.modifiers[{index}]");
            let calls = [path.to_string()];
            self.charge_copy(modifier, &p, &calls)?;
            let output = format!(
                "skills[{skill}].action_policy.modifiers[{}]",
                self.action_modifiers.len()
            );
            self.record(&output, &p, &calls, vec![])?;
            self.record(
                &format!("{output}.change"),
                &format!("{p}.change"),
                &calls,
                vec![],
            )?;
            let change = match &modifier.change {
                SkillChange::ProtectInterruption => native::ActionChange::ProtectInterruption,
                SkillChange::Multihit {
                    increment_percent,
                    cap_percent,
                } => {
                    let mut rates = [0; 2];
                    for (i, (field, argument)) in [
                        ("increment_percent", increment_percent),
                        ("cap_percent", cap_percent),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        let input = format!("{p}.change.Multihit.{field}");
                        let resolved =
                            self.resolve(argument, &input, &Environment::new(), &calls)?;
                        let Value::Count(rate) = resolved.value else {
                            unreachable!("checked skill modifier type")
                        };
                        self.check_modifier_percent(
                            *rate,
                            &format!("{}.Count", resolved.value_path),
                            &calls,
                            "Multihit",
                        )?;
                        rates[i] = *rate;
                        self.record(
                            &format!("{output}.change.Multihit.{field}"),
                            &format!("{}.Count", resolved.value_path),
                            &calls,
                            resolved.sources,
                        )?;
                    }
                    native::ActionChange::Multihit {
                        steps: Vec::new(),
                        increment_percent: rates[0],
                        cap_percent: rates[1],
                    }
                }
            };
            self.action_modifiers.push(native::ActionModifier {
                id: format!("skill:{}", modifier.id),
                relic_definition: modifier.relic_definition.clone(),
                variant: modifier.variant,
                change,
            });
        }
        Ok(())
    }

    /// Expansion supplies stable native action IDs. Bind one whole-skill rule
    /// after expansion, rather than giving each call an independent counter.
    pub(super) fn bind_skill_multihit(
        &mut self,
        modifiers: &[SkillModifier],
        path: &str,
        skill: usize,
        steps: &[SkillStepDef],
    ) -> Result<(), Diagnostic> {
        for (index, modifier) in modifiers.iter().enumerate() {
            if !matches!(modifier.change, SkillChange::Multihit { .. }) {
                continue;
            }
            let input = format!("{path}.modifiers[{index}].change.Multihit");
            let calls = [path.to_string()];
            let mut selected = Vec::new();
            let mut origins = Vec::new();
            for (i, step) in steps.iter().enumerate() {
                if step.effects.iter().any(|effect| {
                    matches!(
                        effect.op,
                        SkillEffectDef::Damage { .. } | SkillEffectDef::DamageExpr { .. }
                    )
                }) {
                    if selected.len() == 64 {
                        return Err(self.error(
                            DiagnosticCode::Limit,
                            &input,
                            &calls,
                            "Multihit supports at most 64 expanded damage actions",
                        ));
                    }
                    self.charge_copy(&step.id, &input, &calls)?;
                    selected.push(step.id.clone());
                    let origin = &self.artifact.sources[&format!("skills[{skill}].steps[{i}]")];
                    origins.push(origin.definition.clone());
                    origins.extend(origin.calls.iter().cloned());
                }
            }
            if selected.is_empty() {
                return Err(self.error(
                    DiagnosticCode::Native,
                    &input,
                    &calls,
                    "Multihit requires a direct damage action",
                ));
            }
            self.record(
                &format!("skills[{skill}].action_policy.modifiers[{index}].change.Multihit.steps"),
                &input,
                &calls,
                origins,
            )?;
            let native::ActionChange::Multihit { steps, .. } =
                &mut self.action_modifiers[index].change
            else {
                unreachable!("skill policies precede local action policies")
            };
            *steps = selected;
        }
        Ok(())
    }

    fn check_ramp_component(
        &self,
        value: u32,
        path: &str,
        calls: &[String],
    ) -> Result<(), Diagnostic> {
        if value == 0 {
            return Err(self.error(
                DiagnosticCode::Native,
                path,
                calls,
                "ramp ratio component must be positive",
            ));
        }
        Ok(())
    }

    pub(super) fn check_action_modifiers(
        &self,
        modifiers: &[ActionModifier],
        path: &str,
        types: &Types<'_>,
    ) -> Result<(), Diagnostic> {
        if modifiers.len() > 128 {
            return Err(self.error(
                DiagnosticCode::Limit,
                path,
                &[],
                "action modifier limit exceeded",
            ));
        }
        let mut ids = BTreeSet::new();
        for (index, modifier) in modifiers.iter().enumerate() {
            let p = format!("{path}.modifiers[{index}]");
            self.identifier(&modifier.id, &format!("{p}.id"))?;
            self.identifier(&modifier.relic_definition, &format!("{p}.relic_definition"))?;
            if !ids.insert(&modifier.id) {
                return Err(self.error(
                    DiagnosticCode::Reference,
                    &format!("{p}.id"),
                    &[],
                    "duplicate local action modifier ID",
                ));
            }
            match &modifier.change {
                ActionChange::AreaControlVulnerability {
                    percent, duration, ..
                }
                | ActionChange::ControlVulnerability {
                    percent, duration, ..
                } => {
                    let kind = modifier.change.kind();
                    let p = format!("{p}.change.{kind}");
                    if let ActionChange::ControlVulnerability { effect, .. } = &modifier.change {
                        self.identifier(effect, &format!("{p}.effect"))?;
                    }
                    for (field, arg, ty) in [
                        ("percent", percent, ValueType::Count),
                        ("duration", duration, ValueType::Milliseconds),
                    ] {
                        let field_path = format!("{p}.{field}");
                        self.expect_type(arg, &field_path, types, ty)?;
                        let concrete = match arg {
                            Argument::Literal(v) => Some((v, format!("{field_path}.Literal"))),
                            Argument::Shared(name) => {
                                let (i, v) = self.values[name.as_str()];
                                Some((v, format!("values[{i}].value")))
                            }
                            _ => None,
                        };
                        if let Some((v, value_path)) = concrete {
                            match v {
                                Value::Count(n) => self.check_modifier_percent(
                                    *n,
                                    &format!("{value_path}.Count"),
                                    &[],
                                    kind,
                                )?,
                                Value::Milliseconds(n) => {
                                    self.vulnerability_duration(
                                        *n,
                                        &format!("{value_path}.Milliseconds"),
                                        &[],
                                    )?;
                                }
                                _ => unreachable!("checked argument type"),
                            }
                        }
                    }
                }
                ActionChange::AreaRampRate {
                    reached_rules,
                    numerator,
                    denominator,
                    ..
                } => {
                    let p = format!("{p}.change.AreaRampRate");
                    let mut unique = BTreeSet::new();
                    if reached_rules.len() > 128 || reached_rules.iter().any(|r| !unique.insert(r))
                    {
                        return Err(self.error(
                            DiagnosticCode::Native,
                            &format!("{p}.reached_rules"),
                            &[],
                            "ramp arrival rules must be unique and at most 128",
                        ));
                    }
                    let mut known = [None, None];
                    for (i, (field, arg)) in
                        [("numerator", numerator), ("denominator", denominator)]
                            .into_iter()
                            .enumerate()
                    {
                        let field_path = format!("{p}.{field}");
                        self.expect_type(arg, &field_path, types, ValueType::Count)?;
                        let concrete = match arg {
                            Argument::Literal(v) => Some((v, format!("{field_path}.Literal"))),
                            Argument::Shared(name) => {
                                let (i, v) = self.values[name.as_str()];
                                Some((v, format!("values[{i}].value")))
                            }
                            _ => None,
                        };
                        if let Some((Value::Count(value), value_path)) = concrete {
                            self.check_ramp_component(*value, &format!("{value_path}.Count"), &[])?;
                            known[i] = Some(*value);
                        }
                    }
                    if matches!(known, [Some(n), Some(d)] if n < d) {
                        return Err(self.error(
                            DiagnosticCode::Native,
                            &p,
                            &[],
                            "ramp progress ratio must be at least one",
                        ));
                    }
                }
                ActionChange::Range { pattern } => {
                    self.expect_type(
                        pattern,
                        &format!("{p}.change.Range.pattern"),
                        types,
                        ValueType::TilePattern,
                    )?;
                }
                ActionChange::TauntDamage {
                    effect,
                    taken_percent,
                    basic_percent,
                } => {
                    let p = format!("{p}.change.TauntDamage");
                    self.identifier(effect, &format!("{p}.effect"))?;
                    let mut known = [None, None];
                    for (i, (field, argument, max)) in [
                        ("taken_percent", taken_percent, 100),
                        ("basic_percent", basic_percent, i32::MAX as u32),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        let field_path = format!("{p}.{field}");
                        self.expect_type(argument, &field_path, types, ValueType::Count)?;
                        let concrete = match argument {
                            Argument::Literal(v) => Some((v, format!("{field_path}.Literal"))),
                            Argument::Shared(name) => {
                                let (i, v) = self.values[name.as_str()];
                                Some((v, format!("values[{i}].value")))
                            }
                            _ => None,
                        };
                        if let Some((Value::Count(v), value_path)) = concrete {
                            self.check_taunt_coefficient(
                                *v,
                                max,
                                &format!("{value_path}.Count"),
                                &[],
                            )?;
                            known[i] = Some(*v);
                        }
                    }
                    if known == [Some(0), Some(0)] {
                        return Err(self.error(
                            DiagnosticCode::Native,
                            &p,
                            &[],
                            "taunt modifier requires at least one positive coefficient",
                        ));
                    }
                }
                ActionChange::AreaDuration { percent }
                | ActionChange::Duration { percent, .. }
                | ActionChange::SupportRadius { percent, .. } => {
                    let kind = modifier.change.kind();
                    if let ActionChange::Duration { effect, .. }
                    | ActionChange::SupportRadius { effect, .. } = &modifier.change
                    {
                        self.identifier(effect, &format!("{p}.change.{kind}.effect"))?;
                    }
                    let p = format!("{p}.change.{kind}.percent");
                    self.expect_type(percent, &p, types, ValueType::Count)?;
                    // Validate concrete values even in an unused definition;
                    // parameter values are checked at each expanded call.
                    let known = match percent {
                        Argument::Literal(value) => Some((value, format!("{p}.Literal"))),
                        Argument::Shared(name) => {
                            let (i, value) = self.values[name.as_str()];
                            Some((value, format!("values[{i}].value")))
                        }
                        _ => None,
                    };
                    if let Some((Value::Count(value), value_path)) = known {
                        self.check_modifier_percent(
                            *value,
                            &format!("{value_path}.Count"),
                            &[],
                            kind,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    fn vulnerability_duration(
        &self,
        duration: u64,
        path: &str,
        calls: &[String],
    ) -> Result<u32, Diagnostic> {
        if duration == 0 || duration > u64::from(u32::MAX) {
            return Err(self.error(
                DiagnosticCode::Native,
                path,
                calls,
                "vulnerability duration must be positive and at most u32::MAX milliseconds",
            ));
        }
        Ok(duration as u32)
    }

    fn check_modifier_percent(
        &self,
        percent: u32,
        path: &str,
        calls: &[String],
        kind: &str,
    ) -> Result<(), Diagnostic> {
        if percent == 0 || percent > i32::MAX as u32 {
            return Err(self.error(
                DiagnosticCode::Native,
                path,
                calls,
                &format!("{kind} modifier percent must be positive and at most i32::MAX"),
            ));
        }
        Ok(())
    }

    fn check_taunt_coefficient(
        &self,
        value: u32,
        max: u32,
        path: &str,
        calls: &[String],
    ) -> Result<(), Diagnostic> {
        if value > max {
            return Err(self.error(
                DiagnosticCode::Native,
                path,
                calls,
                &format!("taunt coefficient must be at most {max}"),
            ));
        }
        Ok(())
    }

    pub(super) fn expand_action_modifiers(
        &mut self,
        modifiers: &'a [ActionModifier],
        path: &str,
        env: &Environment<'a>,
        calls: &[String],
        step: &str,
        step_path: &str,
        delivery: &crate::game::ability::DeliveryDef,
        effects: &[AuthoredSkillEffect],
        skill: usize,
    ) -> Result<(), Diagnostic> {
        for (index, modifier) in modifiers.iter().enumerate() {
            let p = format!("{path}.modifiers[{index}]");
            if self.action_modifiers.len() >= 128 {
                return Err(self.error(
                    DiagnosticCode::Limit,
                    &p,
                    calls,
                    "expanded skill action modifier limit exceeded",
                ));
            }
            self.charge_copy(modifier, &p, calls)?;
            let output = format!(
                "skills[{skill}].action_policy.modifiers[{}]",
                self.action_modifiers.len()
            );
            self.record(&output, &p, calls, vec![])?;
            let change = match &modifier.change {
                ActionChange::AreaControlVulnerability {
                    percent, duration, ..
                }
                | ActionChange::ControlVulnerability {
                    percent, duration, ..
                } => {
                    let kind = modifier.change.kind();
                    let p = format!("{p}.change.{kind}");
                    let percent = self.resolve(percent, &format!("{p}.percent"), env, calls)?;
                    let duration = self.resolve(duration, &format!("{p}.duration"), env, calls)?;
                    let Value::Count(percent_value) = percent.value else {
                        unreachable!("checked argument type")
                    };
                    let Value::Milliseconds(duration_value) = duration.value else {
                        unreachable!("checked argument type")
                    };
                    self.check_modifier_percent(
                        *percent_value,
                        &format!("{}.Count", percent.value_path),
                        calls,
                        kind,
                    )?;
                    let duration_ms = self.vulnerability_duration(
                        *duration_value,
                        &format!("{}.Milliseconds", duration.value_path),
                        calls,
                    )?;
                    let (change, destination, origin_key) = match &modifier.change {
                        ActionChange::AreaControlVulnerability { status, .. } => {
                            let crate::game::ability::DeliveryDef::TileArea { area, .. } = delivery
                            else {
                                return Err(self.error(
                                    DiagnosticCode::Native,
                                    &p,
                                    calls,
                                    "AreaControlVulnerability requires a channel-owned TileArea",
                                ));
                            };
                            let change = native::AreaChange::ControlVulnerability {
                                status: *status,
                                percent: *percent_value,
                                duration_ms,
                            };
                            change.validate(area).map_err(|reason| {
                                self.error(DiagnosticCode::Native, &p, calls, reason)
                            })?;
                            let destination =
                                format!("{output}.change.Area.change.ControlVulnerability");
                            self.record(
                                &format!("{destination}.status"),
                                &format!("{p}.status"),
                                calls,
                                vec![],
                            )?;
                            (
                                native::ActionChange::Area {
                                    step: step.into(),
                                    change,
                                },
                                destination,
                                format!("{step_path}.delivery"),
                            )
                        }
                        ActionChange::ControlVulnerability { effect, .. } => {
                            let mut matching =
                                effects.iter().enumerate().filter(|(_, e)| e.id == *effect);
                            let selected = matching.next();
                            if selected.is_none() || matching.next().is_some() {
                                return Err(self.error(DiagnosticCode::Reference, &format!("{p}.effect"), calls, "modifier requires exactly one effect with this ID in its Action"));
                            }
                            let (index, selected) = selected.unwrap();
                            if !matches!(&selected.op, SkillEffectDef::Sustained { definition: SustainedEffectDef::HitStatus { profile, .. } } if profile.kind.is_hard_control() && profile.vulnerability.is_none())
                            {
                                return Err(self.error(DiagnosticCode::Native, &format!("{p}.effect"), calls, "ControlVulnerability requires a stun or freeze without an existing vulnerability"));
                            }
                            let destination = format!("{output}.change.ControlVulnerability");
                            self.record(
                                &format!("{destination}.effect"),
                                &format!("{p}.effect"),
                                calls,
                                vec![],
                            )?;
                            (
                                native::ActionChange::ControlVulnerability {
                                    step: step.into(),
                                    effect: effect.clone(),
                                    percent: *percent_value,
                                    duration_ms,
                                },
                                destination,
                                format!("{step_path}.effects[{index}]"),
                            )
                        }
                        _ => unreachable!(),
                    };
                    self.record(
                        &format!("{destination}.percent"),
                        &format!("{}.Count", percent.value_path),
                        calls,
                        percent.sources.clone(),
                    )?;
                    self.record(
                        &format!("{destination}.duration_ms"),
                        &format!("{}.Milliseconds", duration.value_path),
                        calls,
                        duration.sources.clone(),
                    )?;
                    if let Some(origin) = self.artifact.sources.get_mut(&origin_key) {
                        for input in std::iter::once(p)
                            .chain(percent.sources.into_iter())
                            .chain(duration.sources.into_iter())
                        {
                            if !origin.inputs.contains(&input) {
                                origin.inputs.push(input);
                            }
                        }
                    }
                    change
                }
                ActionChange::AreaRampRate {
                    status,
                    reached_rules,
                    numerator,
                    denominator,
                } => {
                    let p = format!("{p}.change.AreaRampRate");
                    let mut ratio = [0u64; 2];
                    let mut inputs = vec![p.clone()];
                    for (i, (field, arg)) in
                        [("numerator", numerator), ("denominator", denominator)]
                            .into_iter()
                            .enumerate()
                    {
                        let value = self.resolve(arg, &format!("{p}.{field}"), env, calls)?;
                        let Value::Count(component) = value.value else {
                            unreachable!("checked argument type")
                        };
                        self.check_ramp_component(
                            *component,
                            &format!("{}.Count", value.value_path),
                            calls,
                        )?;
                        ratio[i] = u64::from(*component);
                        inputs.extend(value.sources.iter().cloned());
                        self.record(
                            &format!("{output}.change.Area.change.RampRate.{field}"),
                            &format!("{}.Count", value.value_path),
                            calls,
                            value.sources,
                        )?;
                    }
                    let crate::game::ability::DeliveryDef::TileArea { area, .. } = delivery else {
                        return Err(self.error(
                            DiagnosticCode::Native,
                            &p,
                            calls,
                            "AreaRampRate requires a channel-owned TileArea",
                        ));
                    };
                    let change = native::AreaChange::RampRate {
                        status: *status,
                        reached_rules: reached_rules.clone(),
                        numerator: ratio[0],
                        denominator: ratio[1],
                    };
                    change
                        .validate(area)
                        .map_err(|reason| self.error(DiagnosticCode::Native, &p, calls, reason))?;
                    for field in ["status", "reached_rules"] {
                        self.record(
                            &format!("{output}.change.Area.change.RampRate.{field}"),
                            &format!("{p}.{field}"),
                            calls,
                            vec![],
                        )?;
                    }
                    if let Some(origin) = self
                        .artifact
                        .sources
                        .get_mut(&format!("{step_path}.delivery"))
                    {
                        for input in inputs {
                            if !origin.inputs.contains(&input) {
                                origin.inputs.push(input);
                            }
                        }
                    }
                    native::ActionChange::Area {
                        step: step.into(),
                        change,
                    }
                }
                ActionChange::AreaDuration { percent } => {
                    let p = format!("{p}.change.AreaDuration.percent");
                    let value = self.resolve(percent, &p, env, calls)?;
                    let Value::Count(percent) = value.value else {
                        unreachable!("checked argument type")
                    };
                    self.check_modifier_percent(
                        *percent,
                        &format!("{}.Count", value.value_path),
                        calls,
                        "AreaDuration",
                    )?;
                    let crate::game::ability::DeliveryDef::TileArea { area, .. } = delivery else {
                        return Err(self.error(
                            DiagnosticCode::Native,
                            &p,
                            calls,
                            "AreaDuration requires a channel-owned TileArea",
                        ));
                    };
                    let change = native::AreaChange::Duration { percent: *percent };
                    change
                        .validate(area)
                        .map_err(|reason| self.error(DiagnosticCode::Native, &p, calls, reason))?;
                    self.record(
                        &format!("{output}.change.Area.change.Duration.percent"),
                        &format!("{}.Count", value.value_path),
                        calls,
                        value.sources.clone(),
                    )?;
                    // The area, rather than an optional hit effect, owns this
                    // lifetime. Preserve the conditional input at its source.
                    if let Some(origin) = self
                        .artifact
                        .sources
                        .get_mut(&format!("{step_path}.delivery"))
                    {
                        for input in std::iter::once(p).chain(value.sources.into_iter()) {
                            if !origin.inputs.contains(&input) {
                                origin.inputs.push(input);
                            }
                        }
                    }
                    native::ActionChange::Area {
                        step: step.into(),
                        change,
                    }
                }
                ActionChange::Range { pattern } => {
                    let p = format!("{p}.change.Range.pattern");
                    let value = self.resolve(pattern, &p, env, calls)?;
                    let Value::TilePattern(pattern) = value.value else {
                        unreachable!("checked argument type")
                    };
                    if !matches!(delivery, crate::game::ability::DeliveryDef::TileArea { .. }) {
                        return Err(self.error(
                            DiagnosticCode::Native,
                            &p,
                            calls,
                            "Range modifier requires TileArea delivery",
                        ));
                    }
                    self.record(
                        &format!("{output}.change.Range.pattern"),
                        &format!("{}.TilePattern", value.value_path),
                        calls,
                        value.sources.clone(),
                    )?;
                    // Every recipient effect and the delivery itself depend on
                    // this mask. Keep that authored input when tracing a hit.
                    for key in std::iter::once(format!("{step_path}.delivery"))
                        .chain((0..effects.len()).map(|i| format!("{step_path}.effects[{i}]")))
                    {
                        if let Some(origin) = self.artifact.sources.get_mut(&key) {
                            for input in &value.sources {
                                if !origin.inputs.contains(input) {
                                    origin.inputs.push(input.clone());
                                }
                            }
                        }
                    }
                    native::ActionChange::Range {
                        steps: vec![step.into()],
                        pattern: pattern.clone(),
                    }
                }
                ActionChange::TauntDamage {
                    effect,
                    taken_percent,
                    basic_percent,
                } => {
                    let p = format!("{p}.change.TauntDamage");
                    let mut coefficients = [0, 0];
                    let mut inputs = vec![p.clone()];
                    for (i, (field, argument, max)) in [
                        ("taken_percent", taken_percent, 100),
                        ("basic_percent", basic_percent, i32::MAX as u32),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        let value = self.resolve(argument, &format!("{p}.{field}"), env, calls)?;
                        let Value::Count(coefficient) = value.value else {
                            unreachable!("checked argument type")
                        };
                        self.check_taunt_coefficient(
                            *coefficient,
                            max,
                            &format!("{}.Count", value.value_path),
                            calls,
                        )?;
                        coefficients[i] = *coefficient;
                        inputs.extend(value.sources.iter().cloned());
                        self.record(
                            &format!("{output}.change.TauntDamage.{field}"),
                            &format!("{}.Count", value.value_path),
                            calls,
                            value.sources,
                        )?;
                    }
                    if coefficients == [0, 0] {
                        return Err(self.error(
                            DiagnosticCode::Native,
                            &p,
                            calls,
                            "taunt modifier requires at least one positive coefficient",
                        ));
                    }
                    let mut matching = effects.iter().enumerate().filter(|(_, e)| e.id == *effect);
                    let selected = matching.next();
                    if selected.is_none() || matching.next().is_some() {
                        return Err(self.error(
                            DiagnosticCode::Reference,
                            &format!("{p}.effect"),
                            calls,
                            "modifier requires exactly one effect with this ID in its Action",
                        ));
                    }
                    let (effect_index, selected) = selected.unwrap();
                    if !matches!(
                        &selected.op,
                        SkillEffectDef::Sustained {
                            definition: SustainedEffectDef::Control {
                                kind: crate::game::ability::sustained::ControlKind::ApproachTaunt { .. },
                                ..
                            }
                        }
                    ) {
                        return Err(self.error(
                            DiagnosticCode::Native,
                            &format!("{p}.effect"),
                            calls,
                            "taunt modifier requires an approach taunt effect",
                        ));
                    }
                    self.record(
                        &format!("{output}.change.TauntDamage.effect"),
                        &format!("{p}.effect"),
                        calls,
                        vec![],
                    )?;
                    if let Some(origin) = self
                        .artifact
                        .sources
                        .get_mut(&format!("{step_path}.effects[{effect_index}]"))
                    {
                        for input in inputs {
                            if !origin.inputs.contains(&input) {
                                origin.inputs.push(input);
                            }
                        }
                    }
                    native::ActionChange::TauntDamage {
                        step: step.into(),
                        effect: effect.clone(),
                        taken_percent: coefficients[0],
                        basic_percent: coefficients[1],
                    }
                }
                ActionChange::Duration { effect, percent }
                | ActionChange::SupportRadius { effect, percent } => {
                    let kind = modifier.change.kind();
                    let p = format!("{p}.change.{kind}");
                    let value = self.resolve(percent, &format!("{p}.percent"), env, calls)?;
                    let Value::Count(percent) = value.value else {
                        unreachable!("checked argument type")
                    };
                    self.check_modifier_percent(
                        *percent,
                        &format!("{}.Count", value.value_path),
                        calls,
                        kind,
                    )?;
                    let mut matching = effects.iter().enumerate().filter(|(_, e)| e.id == *effect);
                    let selected = matching.next();
                    if selected.is_none() || matching.next().is_some() {
                        return Err(self.error(
                            DiagnosticCode::Reference,
                            &format!("{p}.effect"),
                            calls,
                            "modifier requires exactly one effect with this ID in its Action",
                        ));
                    }
                    let (effect_index, selected) = selected.unwrap();
                    let supported = match &modifier.change {
                        ActionChange::Duration { .. } => matches!(
                            &selected.op,
                            SkillEffectDef::Sustained {
                                definition: SustainedEffectDef::HitStatus { .. }
                                    | SustainedEffectDef::Control { .. }
                                    | SustainedEffectDef::ChannelAura { .. }
                            }
                        ),
                        ActionChange::SupportRadius { .. } => {
                            native::support_radius(&selected.op).is_some()
                        }
                        ActionChange::Range { .. }
                        | ActionChange::AreaDuration { .. }
                        | ActionChange::AreaRampRate { .. }
                        | ActionChange::AreaControlVulnerability { .. }
                        | ActionChange::ControlVulnerability { .. }
                        | ActionChange::TauntDamage { .. } => {
                            unreachable!()
                        }
                    };
                    if !supported {
                        return Err(self.error(
                            DiagnosticCode::Native,
                            &format!("{p}.effect"),
                            calls,
                            match &modifier.change {
                                ActionChange::Duration { .. } => "duration modifier requires a supported finite sustained effect",
                                ActionChange::SupportRadius { .. } => "support radius modifier requires an authored circular group support effect",
                                ActionChange::Range { .. } | ActionChange::AreaDuration { .. } | ActionChange::AreaRampRate { .. } | ActionChange::AreaControlVulnerability { .. } | ActionChange::ControlVulnerability { .. } | ActionChange::TauntDamage { .. } => unreachable!(),
                            },
                        ));
                    }
                    self.record(
                        &format!("{output}.change.{kind}.effect"),
                        &format!("{p}.effect"),
                        calls,
                        vec![],
                    )?;
                    // The runtime effect can navigate the declared conditional
                    // input as well as its original payload. This is provenance,
                    // not a receipt claiming that the relic was equipped.
                    if let Some(origin) = self
                        .artifact
                        .sources
                        .get_mut(&format!("{step_path}.effects[{effect_index}]"))
                    {
                        for input in std::iter::once(p.clone()).chain(value.sources.iter().cloned())
                        {
                            if !origin.inputs.contains(&input) {
                                origin.inputs.push(input);
                            }
                        }
                    }
                    self.record(
                        &format!("{output}.change.{kind}.percent"),
                        &format!("{}.Count", value.value_path),
                        calls,
                        value.sources,
                    )?;
                    match &modifier.change {
                        ActionChange::Duration { .. } => native::ActionChange::Duration {
                            step: step.into(),
                            effect: effect.clone(),
                            percent: *percent,
                        },
                        ActionChange::SupportRadius { .. } => native::ActionChange::SupportRadius {
                            step: step.into(),
                            effect: effect.clone(),
                            percent: *percent,
                        },
                        ActionChange::Range { .. }
                        | ActionChange::AreaDuration { .. }
                        | ActionChange::AreaRampRate { .. }
                        | ActionChange::AreaControlVulnerability { .. }
                        | ActionChange::ControlVulnerability { .. }
                        | ActionChange::TauntDamage { .. } => {
                            unreachable!()
                        }
                    }
                }
            };
            self.action_modifiers.push(native::ActionModifier {
                id: format!("{step}:{}", modifier.id),
                relic_definition: modifier.relic_definition.clone(),
                variant: modifier.variant,
                change,
            });
        }
        Ok(())
    }
}
