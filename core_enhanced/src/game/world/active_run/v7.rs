//! Explicit V7 → V12 conversion. Validate original identity/checksum before any transformation.
use super::*;
use crate::game::{
    data::content_pack::ContentIdentity,
    skill_fragment::{legacy::LegacyFragmentProgress, SkillFragmentInventory},
};
use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use std::collections::BTreeMap;

pub(super) struct UniqueJson(pub(super) Value);
impl<'de> Deserialize<'de> for UniqueJson {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = UniqueJson;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("JSON without duplicate object keys")
            }
            fn visit_bool<E: Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(UniqueJson(Value::Bool(v)))
            }
            fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(UniqueJson(v.into()))
            }
            fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(UniqueJson(v.into()))
            }
            fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
                serde_json::Number::from_f64(v)
                    .map(|n| UniqueJson(Value::Number(n)))
                    .ok_or_else(|| E::custom("nonfinite JSON"))
            }
            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(UniqueJson(v.into()))
            }
            fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(UniqueJson(v.into()))
            }
            fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueJson(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut v = Vec::new();
                while let Some(UniqueJson(x)) = a.next_element()? {
                    v.push(x);
                }
                Ok(UniqueJson(Value::Array(v)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut v = serde_json::Map::new();
                while let Some(k) = a.next_key::<String>()? {
                    if v.contains_key(&k) {
                        return Err(A::Error::custom("duplicate checkpoint key"));
                    }
                    let UniqueJson(x) = a.next_value()?;
                    v.insert(k, x);
                }
                Ok(UniqueJson(Value::Object(v)))
            }
        }
        d.deserialize_any(V)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyStack {
    count: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyInventory {
    stacks: BTreeMap<crate::game::data::skill_fragment_data::SkillFragmentId, LegacyStack>,
    progress:
        BTreeMap<crate::game::data::skill_fragment_data::SkillFragmentId, LegacyFragmentProgress>,
    fragment_dust: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyLoadout {
    baseline_ids: Vec<crate::game::data::skill_fragment_data::SkillFragmentId>,
    active_fragment_id: Option<crate::game::data::skill_fragment_data::SkillFragmentId>,
}
impl GameCore {
    pub(crate) fn convert_active_run_v7(
        &self,
        input: &[u8],
        expected: &ContentIdentity,
    ) -> Result<Vec<u8>, GameError> {
        let _ = (input, expected);
        Err(active_run_checkpoint_codec_error("V7 conversion", "legacy lifecycle has no exact ownership/instance mapping; original preserved"))
    }
}
