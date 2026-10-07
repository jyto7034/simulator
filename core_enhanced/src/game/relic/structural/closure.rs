//! Finite candidate worklist with signed/structural dependency stratification.
use super::{
    authoring::*, compiler::Binding, EffectiveEffect, FaultCode, ProspectiveProvider,
    RelicPlanError, Work,
};

pub(crate) fn provider_matches(p: &ProviderPredicateDef, actual: &ProspectiveProvider) -> bool {
    let (equipment, fragment, form) = match actual {
        ProspectiveProvider::Fragment(f) => (
            None,
            Some(f.definition_id.as_str()),
            Some(f.form_id.as_str()),
        ),
        ProspectiveProvider::Equipment { definition_id, .. } => {
            (Some(definition_id.as_str()), None, None)
        }
    };
    p.equipment_id
        .as_deref()
        .is_none_or(|id| equipment == Some(id))
        && p.fragment_id
            .as_deref()
            .is_none_or(|id| fragment == Some(id))
        && p.form_id.as_deref().is_none_or(|id| form == Some(id))
}
pub(crate) fn matches(
    selector: &StructuralSelectorDef,
    node: &EffectiveEffect,
    providers: &[ProspectiveProvider],
    presence: bool,
    work: &mut Work,
) -> Result<bool, RelicPlanError> {
    let bytes = selector
        .provider
        .equipment_id
        .as_ref()
        .map_or(0, String::len)
        + selector
            .provider
            .fragment_id
            .as_ref()
            .map_or(0, String::len)
        + selector.provider.form_id.as_ref().map_or(0, String::len);
    work.charge(
        bytes.checked_add(1).ok_or_else(overflow)?,
        "relic_structural_provider_match",
    )?;
    if !provider_matches(&selector.provider, &providers[node.provider]) {
        return Ok(false);
    }
    effect_matches(&selector.effect, node, presence, work)
}
fn effect_matches(
    p: &EffectPredicateDef,
    node: &EffectiveEffect,
    presence: bool,
    work: &mut Work,
) -> Result<bool, RelicPlanError> {
    work.charge(1, "relic_structural_predicate")?;
    if p.op.is_some_and(|op| op != node.operation) {
        return Ok(false);
    }
    for tag in &p.all_tags {
        let bytes = node
            .tags
            .iter()
            .try_fold(tag.len(), |n, s| n.checked_add(s.len()))
            .ok_or_else(overflow)?;
        work.charge(bytes, "relic_structural_tag_match")?;
        if !node.tags.contains(tag) {
            return Ok(false);
        }
    }
    Ok(!presence || p.has_property.is_none_or(|p| node.has_property(p)))
}
fn condition(
    c: &RelicConditionDef,
    target: Option<usize>,
    nodes: &[EffectiveEffect],
    providers: &[ProspectiveProvider],
    work: &mut Work,
) -> Result<bool, RelicPlanError> {
    work.charge(1, "relic_structural_condition")?;
    match c {
        RelicConditionDef::Always => Ok(true),
        RelicConditionDef::SameEffect(p) => match target {
            Some(n) => effect_matches(p, &nodes[n], true, work),
            None => Ok(false),
        },
        RelicConditionDef::Exists(s) => {
            for n in nodes.iter().filter(|n| n.present) {
                if matches(s, n, providers, true, work)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        RelicConditionDef::All(v) => {
            for c in v {
                if !condition(c, target, nodes, providers, work)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        RelicConditionDef::Any(v) => {
            for c in v {
                if condition(c, target, nodes, providers, work)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        RelicConditionDef::Not(c) => Ok(!condition(c, target, nodes, providers, work)?),
    }
}
fn enabled(
    b: &Binding<'_>,
    nodes: &[EffectiveEffect],
    providers: &[ProspectiveProvider],
    work: &mut Work,
) -> Result<bool, RelicPlanError> {
    if let Some(target) = b.target {
        return Ok(nodes[target].present
            && matches(&b.rule.selector, &nodes[target], providers, true, work)?
            && condition(&b.rule.when, Some(target), nodes, providers, work)?);
    }
    fn same(c: &RelicConditionDef) -> bool {
        match c {
            RelicConditionDef::SameEffect(_) => true,
            RelicConditionDef::All(v) | RelicConditionDef::Any(v) => v.iter().any(same),
            RelicConditionDef::Not(c) => same(c),
            _ => false,
        }
    }
    if b.rule.selector.effect == EffectPredicateDef::default()
        && !same(&b.rule.when)
        && condition(&b.rule.when, None, nodes, providers, work)?
    {
        return Ok(true);
    }
    for (i, node) in nodes.iter().enumerate() {
        if node.present
            && node.provider == b.provider
            && node.anchor == b.anchor
            && matches(&b.rule.selector, node, providers, true, work)?
            && condition(&b.rule.when, Some(i), nodes, providers, work)?
        {
            return Ok(true);
        }
    }
    Ok(false)
}
#[derive(Clone, Copy)]
struct Read<'a> {
    selector: Option<&'a StructuralSelectorDef>,
    effect: &'a EffectPredicateDef,
    local: bool,
    negative: bool,
}
fn reads<'a>(c: &'a RelicConditionDef, negative: bool, out: &mut Vec<Read<'a>>) {
    match c {
        RelicConditionDef::SameEffect(p) => out.push(Read {
            selector: None,
            effect: p,
            local: true,
            negative,
        }),
        RelicConditionDef::Exists(s) => out.push(Read {
            selector: Some(s),
            effect: &s.effect,
            local: false,
            negative,
        }),
        RelicConditionDef::All(v) | RelicConditionDef::Any(v) => {
            for c in v {
                reads(c, negative, out);
            }
        }
        RelicConditionDef::Not(c) => reads(c, !negative, out),
        RelicConditionDef::Always => {}
    }
}
fn changes(recipe: &StructuralRecipeDef, p: RelicProperty) -> bool {
    match recipe {
        StructuralRecipeDef::FixedRecipientSupport
        | StructuralRecipeDef::PersonalShieldSingle { .. } => p == RelicProperty::SupportAreaRadius,
        StructuralRecipeDef::RegularBasicSupport { .. } => matches!(
            p,
            RelicProperty::RegularBasicDamagePercent | RelicProperty::RegularBasicSpeedPoints
        ),
        StructuralRecipeDef::DamageFootprint { .. } => p == RelicProperty::DamageFootprint,
    }
}
/// Edge direction: reader -> producer. Strict edges require an earlier stratum.
fn strata(
    bindings: &[Binding<'_>],
    nodes: &[EffectiveEffect],
    providers: &[ProspectiveProvider],
    work: &mut Work,
) -> Result<Vec<usize>, RelicPlanError> {
    let count = bindings.len();
    work.reserve(
        count.checked_mul(std::mem::size_of::<Vec<(usize, bool, bool)>>()),
        "relic_structural_dependency_rows",
    )?;
    let mut edges = vec![Vec::<(usize, bool, bool)>::new(); count];
    for (reader, b) in bindings.iter().enumerate() {
        let bytes = super::compiler::serialized_size(&b.rule.when, work)?;
        work.reserve(
            bytes.checked_mul(std::mem::size_of::<Read<'_>>()),
            "relic_structural_dependency_reads",
        )?;
        work.charge(bytes, "relic_structural_dependency_reads")?;
        let mut selected_reads = vec![Read {
            selector: Some(&b.rule.selector),
            effect: &b.rule.selector.effect,
            local: true,
            negative: false,
        }];
        reads(&b.rule.when, false, &mut selected_reads);
        for read in selected_reads {
            for (ni, node) in nodes.iter().enumerate() {
                if read.local
                    && (b.target.is_some_and(|n| n != ni)
                        || (b.target.is_none()
                            && (node.provider != b.provider || node.anchor != b.anchor)))
                {
                    continue;
                }
                let matched = match read.selector {
                    Some(s) => matches(s, node, providers, false, work)?,
                    None => effect_matches(read.effect, node, false, work)?,
                };
                if !matched {
                    continue;
                }
                work.charge(count, "relic_structural_dependency_producers")?;
                for (producer, writer) in bindings.iter().enumerate() {
                    let granted = writer.granted.contains(&ni);
                    let structural = writer.target == Some(ni)
                        && matches!(&writer.rule.apply, RelicRuleActionDef::Recipe(r) if read.effect.has_property.is_some_and(|p| changes(r,p)));
                    if granted || structural {
                        work.reserve(
                            Some(std::mem::size_of::<(usize, bool, bool)>()),
                            "relic_structural_dependency_edge",
                        )?;
                        edges[reader].push((producer, read.negative || structural, structural));
                    }
                }
            }
        }
    }
    work.reserve(
        count.checked_mul(std::mem::size_of::<usize>()),
        "relic_structural_strata",
    )?;
    let mut result = vec![0usize; count];
    // Longest strict path. A changing path after count rounds witnesses a
    // negative/nonmonotone SCC; positive SCCs retain the same stratum.
    for iteration in 0..count {
        let mut changed = false;
        for (reader, deps) in edges.iter().enumerate() {
            work.charge(deps.len(), "relic_structural_stratification")?;
            for &(producer, strict, structural) in deps {
                let required = result[producer]
                    .checked_add(usize::from(strict))
                    .ok_or_else(overflow)?;
                if required > result[reader] {
                    if iteration + 1 == count {
                        return Err(RelicPlanError::new(
                            if structural {
                                FaultCode::UnprovenStructuralDependency
                            } else {
                                FaultCode::NegativeDependencyCycle
                            },
                            format!(
                                "dependency cycle reader={} producer={}",
                                bindings[reader].rule.id, bindings[producer].rule.id
                            ),
                        ));
                    }
                    result[reader] = required;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    Ok(result)
}
pub(crate) fn solve(
    bindings: &mut [Binding<'_>],
    nodes: &mut [EffectiveEffect],
    providers: &[ProspectiveProvider],
    work: &mut Work,
) -> Result<(), RelicPlanError> {
    let levels = strata(bindings, nodes, providers, work)?;
    let maximum = levels.iter().copied().max().unwrap_or(0);
    for level in 0..=maximum {
        loop {
            let mut changed = false;
            for (i, b) in bindings.iter_mut().enumerate() {
                work.charge(1, "relic_structural_frontier")?;
                if levels[i] != level || b.active || !enabled(b, nodes, providers, work)? {
                    continue;
                }
                b.active = true;
                changed = true;
                for &n in &b.granted {
                    nodes[n].present = true;
                }
            }
            if !changed {
                break;
            }
        }
        for (i, b) in bindings.iter().enumerate() {
            if levels[i] == level && b.active {
                if let (Some(target), RelicRuleActionDef::Recipe(recipe)) =
                    (b.target, &b.rule.apply)
                {
                    nodes[target].include_recipe(recipe, &b.contribution, work)?;
                }
            }
        }
    }
    Ok(())
}
fn overflow() -> RelicPlanError {
    RelicPlanError::new(FaultCode::ArithmeticOverflow, "dependency bound")
}
