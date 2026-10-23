//! Bounded authored deserialization for recursive value expressions.
//!
//! RON 0.5.1 delegates to Serde visitors but has no native recursion limit.
//! The acquisition scanner bounds the expression slice before snapshot clones
//! or typed allocations, and these visitors admit every node/member before it
//! is deserialized into the owned AST.

use std::{
    cell::{Cell, RefCell},
    fmt,
};

use serde::{
    de::{self, DeserializeSeed, Error as _, SeqAccess, Visitor},
    Deserialize, Deserializer,
};

use super::{
    AmountExpr, AmountProvenance, AuthoredMember, EvalMoment, FiniteSource,
    ResolvedTargetPredicate, ValueExpr,
};

// RON 0.5's deserialize_any parses signed magnitudes through i64, so its
// exact round-trip range excludes i64::MIN and unsigned values above i64::MAX.
// Wider formats retain their i128 path; RON rejects these instead of emitting
// an artifact that its loader cannot read.
pub(super) fn serialize_integer<S: serde::Serializer>(
    value: &i128,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let max = i128::from(i64::MAX);
    if (-max..=max).contains(value) {
        serializer.serialize_i64(*value as i64)
    } else {
        serializer.serialize_i128(*value)
    }
}

struct AuthoredI128(i128);

impl<'de> Deserialize<'de> for AuthoredI128 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IntegerVisitor;
        impl<'de> Visitor<'de> for IntegerVisitor {
            type Value = AuthoredI128;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a signed or unsigned integer representable by RON 0.5")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(AuthoredI128(i128::from(value)))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                Ok(AuthoredI128(i128::from(value)))
            }

            fn visit_i128<E>(self, value: i128) -> Result<Self::Value, E> {
                Ok(AuthoredI128(value))
            }

            fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                i128::try_from(value)
                    .map(AuthoredI128)
                    .map_err(|_| E::custom("authored integer exceeds signed i128"))
            }
        }

        deserializer.deserialize_any(IntegerVisitor)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthoredTypedInteger {
    value: AuthoredI128,
    unit: super::ValueUnit,
}

impl<'de> Deserialize<'de> for super::TypedInteger {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let authored = AuthoredTypedInteger::deserialize(deserializer)?;
        Ok(Self::new(authored.value.0, authored.unit))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExprParseLimits {
    pub max_input_bytes: usize,
    pub max_depth: usize,
    pub max_nodes: usize,
    pub max_authored_members: usize,
}

impl ExprParseLimits {
    pub const HARD_MAX: Self = Self {
        max_input_bytes: 64 * 1024,
        max_depth: 64,
        max_nodes: 1024,
        max_authored_members: 4096,
    };

    pub fn validate(self) -> Result<Self, &'static str> {
        if self.max_input_bytes == 0
            || self.max_depth == 0
            || self.max_nodes == 0
            || self.max_authored_members == 0
        {
            return Err("all expression parse limits must be non-zero");
        }
        if self.max_input_bytes > Self::HARD_MAX.max_input_bytes
            || self.max_depth > Self::HARD_MAX.max_depth
            || self.max_nodes > Self::HARD_MAX.max_nodes
            || self.max_authored_members > Self::HARD_MAX.max_authored_members
        {
            return Err("expression parse limits exceed the hard safety ceiling");
        }
        Ok(self)
    }
}

impl Default for ExprParseLimits {
    fn default() -> Self {
        crate::runtime_safety::RuntimeSafetySettings::embedded().expression_parse()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprInputError {
    pub byte_offset: usize,
    pub cause: &'static str,
}

impl fmt::Display for ExprInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at byte {}", self.cause, self.byte_offset)
    }
}

impl std::error::Error for ExprInputError {}

#[derive(Clone, Copy)]
struct ParseMeter {
    limits: ExprParseLimits,
    depth: usize,
    nodes: usize,
    authored_members: usize,
}

thread_local! {
    static CONFIGURED_LIMITS: Cell<Option<ExprParseLimits>> = const { Cell::new(None) };
    static ACTIVE_METER: RefCell<Option<ParseMeter>> = const { RefCell::new(None) };
}

pub(crate) fn current_parse_limits() -> ExprParseLimits {
    CONFIGURED_LIMITS.with(Cell::get).unwrap_or_default()
}

pub(crate) fn with_parse_limits<T>(limits: ExprParseLimits, operation: impl FnOnce() -> T) -> T {
    struct Restore(Option<ExprParseLimits>);
    impl Drop for Restore {
        fn drop(&mut self) {
            CONFIGURED_LIMITS.with(|configured| configured.set(self.0));
        }
    }

    let previous = CONFIGURED_LIMITS.with(|configured| {
        let previous = configured.get();
        configured.set(Some(limits));
        previous
    });
    let _restore = Restore(previous);
    operation()
}

struct NodeGuard {
    owns_meter: bool,
}

impl Drop for NodeGuard {
    fn drop(&mut self) {
        ACTIVE_METER.with(|active| {
            let mut active = active.borrow_mut();
            if let Some(meter) = active.as_mut() {
                meter.depth = meter.depth.saturating_sub(1);
            }
            if self.owns_meter {
                *active = None;
            }
        });
    }
}

fn enter_node<E: de::Error>() -> Result<NodeGuard, E> {
    ACTIVE_METER.with(|active| {
        let mut active = active.borrow_mut();
        let owns_meter = active.is_none();
        if owns_meter {
            let limits = current_parse_limits();
            *active = Some(ParseMeter {
                limits,
                depth: 0,
                nodes: 0,
                authored_members: 0,
            });
        }
        let meter = active.as_mut().expect("parse meter initialized");
        let next_depth = meter
            .depth
            .checked_add(1)
            .ok_or_else(|| E::custom("value expression depth counter overflowed"))?;
        if next_depth > meter.limits.max_depth {
            return Err(E::custom(format!(
                "value expression depth limit exceeded: {next_depth} > {}",
                meter.limits.max_depth
            )));
        }
        let next_nodes = meter
            .nodes
            .checked_add(1)
            .ok_or_else(|| E::custom("value expression node counter overflowed"))?;
        if next_nodes > meter.limits.max_nodes {
            return Err(E::custom(format!(
                "value expression node limit exceeded: {next_nodes} > {}",
                meter.limits.max_nodes
            )));
        }
        meter.depth = next_depth;
        meter.nodes = next_nodes;
        Ok(NodeGuard { owns_meter })
    })
}

fn nodes_at_limit() -> bool {
    ACTIVE_METER.with(|active| {
        active
            .borrow()
            .as_ref()
            .is_some_and(|meter| meter.nodes >= meter.limits.max_nodes)
    })
}

fn members_at_limit() -> bool {
    ACTIVE_METER.with(|active| {
        active
            .borrow()
            .as_ref()
            .is_some_and(|meter| meter.authored_members >= meter.limits.max_authored_members)
    })
}

struct RejectSeed(&'static str);

impl<'de> DeserializeSeed<'de> for RejectSeed {
    type Value = ();

    fn deserialize<D>(self, _deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        Err(D::Error::custom(self.0))
    }
}

struct ValueExprSeed;

impl<'de> DeserializeSeed<'de> for ValueExprSeed {
    type Value = ValueExpr;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        ValueExpr::deserialize(deserializer)
    }
}

struct AuthoredMemberSeed;

impl<'de> DeserializeSeed<'de> for AuthoredMemberSeed {
    type Value = AuthoredMember;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        ACTIVE_METER.with(|active| {
            let mut active = active.borrow_mut();
            let meter = active
                .as_mut()
                .ok_or_else(|| D::Error::custom("authored member outside a value expression"))?;
            meter.authored_members = meter
                .authored_members
                .checked_add(1)
                .ok_or_else(|| D::Error::custom("authored aggregate member counter overflowed"))?;
            if meter.authored_members > meter.limits.max_authored_members {
                return Err(D::Error::custom(format!(
                    "authored aggregate member limit exceeded: {} > {}",
                    meter.authored_members, meter.limits.max_authored_members
                )));
            }
            Ok(())
        })?;
        AuthoredMember::deserialize(deserializer)
    }
}

struct ResolvedPredicateSeed;

impl<'de> DeserializeSeed<'de> for ResolvedPredicateSeed {
    type Value = ResolvedTargetPredicate;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let _guard = enter_node::<D::Error>()?;
        ResolvedTargetPredicate::deserialize(deserializer)
    }
}

#[derive(Debug)]
struct BoundedExprVec(Vec<ValueExpr>);

impl<'de> Deserialize<'de> for BoundedExprVec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ExprVecVisitor;
        impl<'de> Visitor<'de> for ExprVecVisitor {
            type Value = BoundedExprVec;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a bounded sequence of value expressions")
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::new();
                loop {
                    if nodes_at_limit() {
                        match sequence.next_element_seed(RejectSeed(
                            "value expression node limit exceeded before sequence element allocation",
                        ))? {
                            None => break,
                            Some(()) => unreachable!("RejectSeed never accepts an element"),
                        }
                    }
                    match sequence.next_element_seed(ValueExprSeed)? {
                        Some(value) => values.push(value),
                        None => break,
                    }
                }
                Ok(BoundedExprVec(values))
            }
        }
        deserializer.deserialize_seq(ExprVecVisitor)
    }
}

#[derive(Debug)]
struct BoundedMembers(Vec<AuthoredMember>);

impl<'de> Deserialize<'de> for BoundedMembers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MembersVisitor;
        impl<'de> Visitor<'de> for MembersVisitor {
            type Value = BoundedMembers;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a bounded sequence of authored aggregate members")
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut members = Vec::new();
                loop {
                    if members_at_limit() {
                        match sequence.next_element_seed(RejectSeed(
                            "authored aggregate member limit exceeded before member allocation",
                        ))? {
                            None => break,
                            Some(()) => unreachable!("RejectSeed never accepts an element"),
                        }
                    }
                    match sequence.next_element_seed(AuthoredMemberSeed)? {
                        Some(member) => members.push(member),
                        None => break,
                    }
                }
                Ok(BoundedMembers(members))
            }
        }
        deserializer.deserialize_seq(MembersVisitor)
    }
}

#[derive(Debug)]
struct BoundedResolvedPredicates(Vec<ResolvedTargetPredicate>);

impl<'de> Deserialize<'de> for BoundedResolvedPredicates {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PredicatesVisitor;
        impl<'de> Visitor<'de> for PredicatesVisitor {
            type Value = BoundedResolvedPredicates;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a bounded sequence of resolved-target predicates")
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut predicates = Vec::new();
                loop {
                    if nodes_at_limit() {
                        match sequence.next_element_seed(RejectSeed(
                            "value expression node limit exceeded before predicate allocation",
                        ))? {
                            None => break,
                            Some(()) => unreachable!("RejectSeed never accepts an element"),
                        }
                    }
                    match sequence.next_element_seed(ResolvedPredicateSeed)? {
                        Some(predicate) => predicates.push(predicate),
                        None => break,
                    }
                }
                Ok(BoundedResolvedPredicates(predicates))
            }
        }
        deserializer.deserialize_seq(PredicatesVisitor)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthoredAmountExpr {
    moment: EvalMoment,
    root: ValueExpr,
}

impl<'de> Deserialize<'de> for AmountExpr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let authored = AuthoredAmountExpr::deserialize(deserializer)?;
        Ok(Self {
            moment: authored.moment,
            root: authored.root,
            provenance: AmountProvenance::AuthoredExpression,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
enum WireValueExpr {
    Const(super::TypedInteger),
    Read(super::ValueRef),
    Add(BoundedExprVec),
    Mul {
        value: Box<ValueExpr>,
        scalar: Box<ValueExpr>,
    },
    Ratio {
        numerator: Box<ValueExpr>,
        denominator: Box<ValueExpr>,
        rounding: super::Rounding,
    },
    Clamp {
        value: Box<ValueExpr>,
        min: Box<ValueExpr>,
        max: Box<ValueExpr>,
    },
    Aggregate {
        source: FiniteSource,
        grouping: super::Grouping,
        reduction: super::Reduction,
        outer_reduction: Option<super::Reduction>,
        member: Box<ValueExpr>,
    },
}

impl From<WireValueExpr> for ValueExpr {
    fn from(value: WireValueExpr) -> Self {
        match value {
            WireValueExpr::Const(value) => Self::Const(value),
            WireValueExpr::Read(value) => Self::Read(value),
            WireValueExpr::Add(values) => Self::Add(values.0),
            WireValueExpr::Mul { value, scalar } => Self::Mul { value, scalar },
            WireValueExpr::Ratio {
                numerator,
                denominator,
                rounding,
            } => Self::Ratio {
                numerator,
                denominator,
                rounding,
            },
            WireValueExpr::Clamp { value, min, max } => Self::Clamp { value, min, max },
            WireValueExpr::Aggregate {
                source,
                grouping,
                reduction,
                outer_reduction,
                member,
            } => Self::Aggregate {
                source,
                grouping,
                reduction,
                outer_reduction,
                member,
            },
        }
    }
}

impl<'de> Deserialize<'de> for ValueExpr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let _guard = enter_node::<D::Error>()?;
        WireValueExpr::deserialize(deserializer).map(Into::into)
    }
}

#[derive(Deserialize)]
enum WireFiniteSource {
    AuthoredMembers(BoundedMembers),
    ResolvedTargets,
    ResolvedTargetsWhere(BoundedResolvedPredicates),
}

impl<'de> Deserialize<'de> for FiniteSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match WireFiniteSource::deserialize(deserializer)? {
            WireFiniteSource::AuthoredMembers(members) => Self::AuthoredMembers(members.0),
            WireFiniteSource::ResolvedTargets => Self::ResolvedTargets,
            WireFiniteSource::ResolvedTargetsWhere(predicates) => {
                Self::ResolvedTargetsWhere(predicates.0)
            }
        })
    }
}

/// Scan skill RON without allocating expression payloads. `max_nesting` also
/// bounds the scanner's own delimiter stack for direct typed loading paths.
pub fn validate_expression_input(
    input: &str,
    max_expression_bytes: usize,
    max_nesting: usize,
) -> Result<(), ExprInputError> {
    if max_expression_bytes == 0 || max_nesting == 0 {
        return Err(ExprInputError {
            byte_offset: 0,
            cause: "expression input limits must be non-zero",
        });
    }
    #[derive(Clone, Copy)]
    struct Delimiter {
        opener: u8,
        expression_start: Option<usize>,
    }

    let bytes = input.as_bytes();
    let mut index = 0_usize;
    let mut stack = Vec::<Delimiter>::new();
    let mut pending_expression = None::<usize>;
    let mut active_expression = None::<usize>;

    let error = |byte_offset, cause| ExprInputError { byte_offset, cause };
    while index < bytes.len() {
        if let Some(start) = active_expression {
            if index.saturating_sub(start) >= max_expression_bytes {
                return Err(error(
                    start.saturating_add(max_expression_bytes),
                    "value expression input byte limit exceeded",
                ));
            }
        }
        match bytes[index] {
            byte if byte.is_ascii_whitespace() => index += 1,
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index += 2;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                let start = index;
                index += 2;
                let mut comment_depth = 1_usize;
                while comment_depth != 0 {
                    if index >= bytes.len() {
                        return Err(error(start, "unterminated block comment"));
                    }
                    if bytes.get(index..index + 2) == Some(b"/*") {
                        comment_depth = comment_depth
                            .checked_add(1)
                            .ok_or_else(|| error(index, "comment nesting overflowed"))?;
                        if comment_depth > max_nesting {
                            return Err(error(index, "comment nesting limit exceeded"));
                        }
                        index += 2;
                    } else if bytes.get(index..index + 2) == Some(b"*/") {
                        comment_depth -= 1;
                        index += 2;
                    } else {
                        index += 1;
                    }
                    if let Some(expression_start) = active_expression {
                        if index.saturating_sub(expression_start) > max_expression_bytes {
                            return Err(error(
                                expression_start.saturating_add(max_expression_bytes),
                                "value expression input byte limit exceeded",
                            ));
                        }
                    }
                }
            }
            b'"' | b'\'' => {
                pending_expression = None;
                let quote = bytes[index];
                let start = index;
                index += 1;
                loop {
                    let Some(&byte) = bytes.get(index) else {
                        return Err(error(start, "unterminated quoted value"));
                    };
                    index += 1;
                    if byte == quote {
                        break;
                    }
                    if byte == b'\\' {
                        if index == bytes.len() {
                            return Err(error(start, "unterminated quoted escape"));
                        }
                        index += 1;
                    }
                    if let Some(expression_start) = active_expression {
                        if index.saturating_sub(expression_start) > max_expression_bytes {
                            return Err(error(
                                expression_start.saturating_add(max_expression_bytes),
                                "value expression input byte limit exceeded",
                            ));
                        }
                    }
                }
            }
            b'r' if matches!(bytes.get(index + 1), Some(b'#' | b'"')) => {
                pending_expression = None;
                let start = index;
                let mut end = index + 1;
                while bytes.get(end) == Some(&b'#') {
                    end += 1;
                }
                if bytes.get(end) != Some(&b'"') {
                    index = end;
                    continue;
                }
                let hashes = end - index - 1;
                if hashes > max_nesting {
                    return Err(error(start, "raw string delimiter limit exceeded"));
                }
                index = end + 1;
                loop {
                    let Some(&byte) = bytes.get(index) else {
                        return Err(error(start, "unterminated raw string"));
                    };
                    index += 1;
                    if byte == b'"'
                        && bytes
                            .get(index..index + hashes)
                            .is_some_and(|tail| tail.iter().all(|byte| *byte == b'#'))
                    {
                        index += hashes;
                        break;
                    }
                    if let Some(expression_start) = active_expression {
                        if index.saturating_sub(expression_start) > max_expression_bytes {
                            return Err(error(
                                expression_start.saturating_add(max_expression_bytes),
                                "value expression input byte limit exceeded",
                            ));
                        }
                    }
                }
            }
            byte if byte.is_ascii_alphabetic() || byte == b'_' => {
                let start = index;
                index += 1;
                while bytes
                    .get(index)
                    .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                {
                    index += 1;
                }
                pending_expression =
                    matches!(&bytes[start..index], b"DamageExpr" | b"HealExpr").then_some(start);
            }
            b'(' | b'[' | b'{' => {
                let expression_start = pending_expression.take();
                if stack.len() >= max_nesting {
                    return Err(error(index, "parser nesting limit exceeded"));
                }
                if let Some(start) = expression_start {
                    if active_expression.is_some() {
                        return Err(error(start, "nested effect expression wrapper is invalid"));
                    }
                    active_expression = Some(start);
                }
                stack.push(Delimiter {
                    opener: bytes[index],
                    expression_start,
                });
                index += 1;
            }
            b')' | b']' | b'}' => {
                pending_expression = None;
                let expected = match bytes[index] {
                    b')' => b'(',
                    b']' => b'[',
                    _ => b'{',
                };
                let Some(delimiter) = stack.pop() else {
                    return Err(error(index, "unmatched closing delimiter"));
                };
                if delimiter.opener != expected {
                    return Err(error(index, "mismatched closing delimiter"));
                }
                index += 1;
                if let Some(start) = delimiter.expression_start {
                    let length = index.saturating_sub(start);
                    if length > max_expression_bytes {
                        return Err(error(
                            start.saturating_add(max_expression_bytes),
                            "value expression input byte limit exceeded",
                        ));
                    }
                    active_expression = None;
                }
            }
            _ => {
                pending_expression = None;
                index += 1;
            }
        }
    }
    if !stack.is_empty() {
        return Err(error(index, "unclosed delimiter"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::skill_data::RawSkillDatabase;

    fn skill_ron(op: &str) -> String {
        format!(
            r#"SkillDatabase(
                skills: [SkillDef(
                    id: "bounded_expr", name: "Bounded Expr", kind: Targeted,
                    focus_time_ms: 0,
                    use_lifetime: Action,
                    cast_targeting: Explicit(
                        selection: automatic_unit,
                        target: EnemySingle(rule: Nearest),
                        reach: global,
                        air_capable: false,
                    ),
                    steps: [SkillStepDef(
                        id: "strike", delay_ms: 0,
                        target: EnemySingle(rule: Nearest), delivery: Instant,
                        effects: [(id: "amount", op: {op})],
                    )],
                )],
            )"#
        )
    }

    fn damage_expr(root: &str, padding: usize) -> String {
        format!(
            "DamageExpr(/*{}*/ amount: (moment: Snapshot, root: {root}), damage_type: True)",
            "x".repeat(padding)
        )
    }

    fn parse(
        op: &str,
        limits: ExprParseLimits,
    ) -> Result<RawSkillDatabase, crate::game::data::live_ron::LiveRonError> {
        let input = skill_ron(op);
        RawSkillDatabase::from_ron_with_limits("bounded/skills.ron", &input, &limits, 128)
    }

    fn nested(depth: usize) -> String {
        let mut value = "Const((value: 1, unit: Magnitude))".to_owned();
        for _ in 1..depth {
            value = format!("Add([{value}])");
        }
        value
    }

    fn wide(nodes: usize) -> String {
        assert!(nodes >= 2);
        format!(
            "Add([{}])",
            std::iter::repeat_n("Const((value: 1, unit: Magnitude))", nodes - 1)
                .collect::<Vec<_>>()
                .join(",")
        )
    }

    #[test]
    fn actual_skill_ron_enforces_depth_and_node_edges_before_unbounded_ast_creation() {
        let limits = ExprParseLimits {
            max_depth: 8,
            ..ExprParseLimits::default()
        };
        for depth in [limits.max_depth - 1, limits.max_depth] {
            parse(&damage_expr(&nested(depth), 0), limits).unwrap();
        }
        let error = parse(&damage_expr(&nested(limits.max_depth + 1), 0), limits).unwrap_err();
        assert_eq!(error.code, "value_expr_depth_limit");
        assert!(error.cause.contains("9 > 8"));

        for nodes in [limits.max_nodes - 1, limits.max_nodes] {
            parse(&damage_expr(&wide(nodes), 0), limits).unwrap();
        }
        let error = parse(&damage_expr(&wide(limits.max_nodes + 1), 0), limits).unwrap_err();
        assert_eq!(error.code, "value_expr_node_limit");
        assert!(error.cause.contains("before sequence element allocation"));
    }

    fn aggregate(members: usize) -> String {
        let members = (0..members)
            .map(|index| {
                format!(
                    "(identity: \"m{index}\", value: (value: {index}, unit: Magnitude), owner: None, projected_tile: None)"
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "Aggregate(source: AuthoredMembers([{members}]), grouping: WholeSet, reduction: Sum, outer_reduction: None, member: Read(AggregateMember(unit: Magnitude)))"
        )
    }

    fn resolved_predicate_aggregate(predicates: &[&str]) -> String {
        format!(
            "Aggregate(source: ResolvedTargetsWhere([{}]), grouping: ByResolvedOwner(ActualEmployee), reduction: Max, outer_reduction: Some(Sum), member: Read(Stat(subject: AggregateMember, stat: Attack)))",
            predicates.join(",")
        )
    }

    #[test]
    fn resolved_predicates_use_the_shared_node_budget_before_element_allocation() {
        let limits = ExprParseLimits {
            max_nodes: 4,
            ..ExprParseLimits::default()
        };
        parse(
            &damage_expr(
                &resolved_predicate_aggregate(&["IsActualEmployee", "HasEquippedSlot(Weapon)"]),
                0,
            ),
            limits,
        )
        .unwrap();

        let error = parse(
            &damage_expr(
                &resolved_predicate_aggregate(&[
                    "IsActualEmployee",
                    "HasEquippedSlot(Weapon)",
                    "HasEquippedSlot(Armor)",
                    "HasEquippedSlot(Accessory)",
                ]),
                0,
            ),
            limits,
        )
        .unwrap_err();
        assert_eq!(error.code, "value_expr_node_limit");
        assert!(error.cause.contains("before predicate allocation"));
    }

    #[test]
    fn actual_skill_ron_enforces_authored_member_edges_before_member_allocation() {
        let limits = ExprParseLimits {
            max_authored_members: 3,
            ..ExprParseLimits::default()
        };
        for members in [2, 3] {
            parse(&damage_expr(&aggregate(members), 0), limits).unwrap();
        }
        let error = parse(&damage_expr(&aggregate(4), 0), limits).unwrap_err();
        assert_eq!(error.code, "value_expr_member_limit");
        assert!(error.cause.contains("before member allocation"));
    }

    #[test]
    fn expression_input_span_accepts_limit_minus_one_and_limit_then_rejects_overlong_payload() {
        let base = damage_expr("Const((value: 1, unit: Magnitude))", 0);
        let limit = base.len() + 32;
        let limits = ExprParseLimits {
            max_input_bytes: limit,
            ..ExprParseLimits::default()
        };
        for length in [limit - 1, limit] {
            let op = damage_expr("Const((value: 1, unit: Magnitude))", length - base.len());
            assert_eq!(op.len(), length);
            parse(&op, limits).unwrap();
        }
        let over = damage_expr("Const((value: 1, unit: Magnitude))", limit + 1 - base.len());
        let error = parse(&over, limits).unwrap_err();
        assert_eq!(error.code, "value_expr_input_limit");
        assert!(error.cause.contains("input byte limit exceeded"));

        let long_unclosed = format!(
            "DamageExpr(amount: (moment: Snapshot, root: Const((value: 1, unit: Magnitude))), damage_type: True, /*{}",
            "x".repeat(ExprParseLimits::HARD_MAX.max_input_bytes)
        );
        let error = parse(&long_unclosed, ExprParseLimits::default()).unwrap_err();
        assert_eq!(error.code, "value_expr_input_limit");
        assert!(error.cause.contains("input byte limit exceeded"));
    }

    #[test]
    fn scanner_treats_strings_and_comments_as_opaque_and_bounds_its_own_nesting() {
        validate_expression_input(
            "(\"DamageExpr((((\", r###\"HealExpr([[[\"###, /* DamageExpr({{{ */ 1)",
            64,
            4,
        )
        .unwrap();
        let error = validate_expression_input("(((((0)))))", 64, 4).unwrap_err();
        assert_eq!(error.cause, "parser nesting limit exceeded");
    }

    #[test]
    fn expression_struct_variants_reject_unknown_fields() {
        let invalid_roots = [
            "Mul(value: Const((value: 1, unit: Magnitude)), scalar: Const((value: 2, unit: Scalar)), typo_parameter: 123)",
            "Ratio(numerator: Const((value: 4, unit: Magnitude)), denominator: Const((value: 2, unit: Scalar)), rounding: TowardZero, typo_parameter: 123)",
            "Clamp(value: Const((value: 2, unit: Magnitude)), min: Const((value: 1, unit: Magnitude)), max: Const((value: 3, unit: Magnitude)), typo_parameter: 123)",
            "Aggregate(source: AuthoredMembers([]), grouping: WholeSet, reduction: Sum, outer_reduction: None, member: Const((value: 1, unit: Magnitude)), typo_parameter: 123)",
        ];
        for root in invalid_roots {
            let error = parse(&damage_expr(root, 0), ExprParseLimits::default()).unwrap_err();
            assert_eq!(error.code, "ron_parse", "root={root}");
            assert_eq!(error.object_id.as_deref(), Some("bounded_expr"));
            assert_eq!(error.action_id.as_deref(), Some("strike"));
            assert_eq!(error.effect_id.as_deref(), Some("amount"));
            assert!(
                error.cause.contains("unknown field"),
                "root={root}: {error:?}"
            );
        }
    }

    #[test]
    fn read_reference_payloads_reject_unknown_fields() {
        let invalid_roots = [
            "Read(Stat(subject: Source, stat: Attack, typo_parameter: 123))",
            "Read(HitPoints(subject: Source, field: Current, typo_parameter: 123))",
            "Read(Resource(resource: \"energy\", owner: SourceEmployee, field: Balance, typo_parameter: 123))",
            "Read(Result(selector: CurrentActionPrefix, field: Count, typo_parameter: 123))",
            "Aggregate(source: AuthoredMembers([(identity: \"a\", value: (value: 1, unit: Magnitude), owner: None, projected_tile: None)]), grouping: WholeSet, reduction: Sum, outer_reduction: None, member: Read(AggregateMember(unit: Magnitude, typo_parameter: 123)))",
        ];
        for root in invalid_roots {
            let error = parse(&damage_expr(root, 0), ExprParseLimits::default()).unwrap_err();
            assert_eq!(error.code, "ron_parse", "root={root}");
            assert_eq!(error.object_id.as_deref(), Some("bounded_expr"));
            assert_eq!(error.action_id.as_deref(), Some("strike"));
            assert_eq!(error.effect_id.as_deref(), Some("amount"));
            assert!(
                error.cause.contains("unknown field"),
                "root={root}: {error:?}"
            );
        }
    }

    #[test]
    fn strict_expression_fields_preserve_valid_forms_and_nested_member_contracts() {
        let valid_roots = [
            "Mul(value: Const((value: 2, unit: Magnitude)), scalar: Const((value: 3, unit: Scalar)))",
            "Ratio(numerator: Const((value: 4, unit: Magnitude)), denominator: Const((value: 2, unit: Scalar)), rounding: TowardZero)",
            "Clamp(value: Const((value: 2, unit: Magnitude)), min: Const((value: 1, unit: Magnitude)), max: Const((value: 3, unit: Magnitude)))",
            "Aggregate(source: AuthoredMembers([(identity: \"a\", value: (value: 1, unit: Magnitude), owner: None, projected_tile: None)]), grouping: WholeSet, reduction: Sum, outer_reduction: None, member: Read(AggregateMember(unit: Magnitude)))",
            "Read(Stat(subject: Source, stat: Attack))",
            "Read(HitPoints(subject: Source, field: Current))",
            "Read(Resource(resource: \"energy\", owner: SourceEmployee, field: Balance))",
            "Read(Result(selector: CurrentActionPrefix, field: Count))",
        ];
        for root in valid_roots {
            parse(&damage_expr(root, 0), ExprParseLimits::default())
                .unwrap_or_else(|error| panic!("valid root rejected: {root}: {error:?}"));
        }

        let member_typo = "Aggregate(source: AuthoredMembers([(identity: \"a\", value: (value: 1, unit: Magnitude), owner: None, projected_tile: None, typo_parameter: 123)]), grouping: WholeSet, reduction: Sum, outer_reduction: None, member: Read(AggregateMember(unit: Magnitude)))";
        let error = parse(&damage_expr(member_typo, 0), ExprParseLimits::default()).unwrap_err();
        assert_eq!(error.code, "ron_parse");
        assert!(error.cause.contains("unknown field"));
    }
}

#[cfg(test)]
mod integer_transport_tests {
    use super::super::{TypedInteger, ValueUnit};
    #[test]
    fn typed_integer_ron_transport_preserves_its_exact_range_and_rejects_wider_values() {
        for value in [0, 3, -i128::from(i64::MAX), i128::from(i64::MAX)] {
            let original = TypedInteger::new(value, ValueUnit::Scalar);
            let text = ron::ser::to_string(&original).unwrap();
            let restored: TypedInteger = ron::de::from_str(&text).unwrap();
            assert_eq!(original, restored);
        }
        for value in [i128::from(i64::MIN), i128::from(u64::MAX), i128::MAX] {
            assert!(ron::ser::to_string(&TypedInteger::new(value, ValueUnit::Scalar)).is_err());
        }
    }
}
