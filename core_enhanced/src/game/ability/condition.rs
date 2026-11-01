//! Shared boolean structure. Leaves retain their Flow/State type and context.
use serde::{Deserialize, Serialize};

/// Composition-only syntax. Native compilers reject these calls; no call frame
/// or authored parameter can reach an executable condition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredicateCall {
    pub name: String,
    pub arguments: Vec<PredicateNamedArgument>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredicateNamedArgument {
    pub name: String,
    pub value: PredicateArgument,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PredicateArgument {
    Literal(PredicateValue),
    Parameter(String),
    Unresolved(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PredicateValue {
    Amount(crate::game::value_expr::AmountExpr),
    BoolField(String),
    FlowResult(FlowResultRef),
    FlowLiteral(super::flow::FlowLiteral),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowResultRef {
    pub node_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_id: Option<String>,
    pub metric: super::flow::FlowMetric,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ConditionLogic<T> {
    All(Vec<T>),
    Any(Vec<T>),
    Not(Box<T>),
}

impl<T> ConditionLogic<T> {
    pub fn children(&self) -> &[T] {
        match self {
            Self::All(items) | Self::Any(items) => items,
            Self::Not(item) => std::slice::from_ref(item),
        }
    }

    pub fn children_mut(&mut self) -> &mut [T] {
        match self {
            Self::All(items) | Self::Any(items) => items,
            Self::Not(item) => std::slice::from_mut(item),
        }
    }

    pub fn child_suffix(&self, index: usize) -> String {
        match self {
            Self::All(_) => format!(".All[{index}]"),
            Self::Any(_) => format!(".Any[{index}]"),
            Self::Not(_) => ".Not".into(),
        }
    }

    /// Compile every child, even when an earlier constant determines the result.
    pub fn try_map<U, E>(
        &self,
        mut map: impl FnMut(usize, &T) -> Result<U, E>,
    ) -> Result<ConditionLogic<U>, E> {
        Ok(match self {
            Self::All(items) => ConditionLogic::All(
                items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| map(i, item))
                    .collect::<Result<_, _>>()?,
            ),
            Self::Any(items) => ConditionLogic::Any(
                items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| map(i, item))
                    .collect::<Result<_, _>>()?,
            ),
            Self::Not(item) => ConditionLogic::Not(Box::new(map(0, item)?)),
        })
    }

    /// Declaration order, with short circuiting. Compilers reject empty groups.
    pub fn evaluate<E>(&self, mut eval: impl FnMut(&T) -> Result<bool, E>) -> Result<bool, E> {
        match self {
            Self::All(items) => {
                for item in items {
                    if !eval(item)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Self::Any(items) => {
                for item in items {
                    if eval(item)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Self::Not(item) => Ok(!eval(item)?),
        }
    }
}
