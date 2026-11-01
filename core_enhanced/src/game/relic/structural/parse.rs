//! Reject recursive condition depth/nodes before decoding another owned child.
use super::authoring::{EffectPredicateDef, RelicConditionDef, StructuralSelectorDef};
use serde::{de::Error, Deserialize, Deserializer};
use std::cell::RefCell;

#[derive(Default)]
struct Meter {
    depth: usize,
    nodes: usize,
}
thread_local! { static METER: RefCell<Option<Meter>> = const { RefCell::new(None) }; }
struct Guard {
    root: bool,
}
impl Drop for Guard {
    fn drop(&mut self) {
        METER.with(|slot| {
            let mut slot = slot.borrow_mut();
            if self.root {
                *slot = None;
            } else if let Some(m) = slot.as_mut() {
                m.depth -= 1;
            }
        });
    }
}
impl<'de> Deserialize<'de> for RelicConditionDef {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let guard = METER.with(|slot| {
            let mut slot = slot.borrow_mut();
            let root = slot.is_none();
            let m = slot.get_or_insert_with(Meter::default);
            let limits = crate::game::value_expr::current_parse_limits();
            if m.depth >= limits.max_depth || m.nodes >= limits.max_nodes {
                if root {
                    *slot = None;
                }
                return Err(D::Error::custom("relic condition parse depth/node limit"));
            }
            m.depth += 1;
            m.nodes += 1;
            Ok(Guard { root })
        })?;
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Authored {
            Always,
            SameEffect(EffectPredicateDef),
            Exists(StructuralSelectorDef),
            All(Vec<RelicConditionDef>),
            Any(Vec<RelicConditionDef>),
            Not(Box<RelicConditionDef>),
        }
        let value = match Authored::deserialize(d)? {
            Authored::Always => Self::Always,
            Authored::SameEffect(p) => Self::SameEffect(p),
            Authored::Exists(p) => Self::Exists(p),
            Authored::All(v) => Self::All(v),
            Authored::Any(v) => Self::Any(v),
            Authored::Not(v) => Self::Not(v),
        };
        drop(guard);
        Ok(value)
    }
}
