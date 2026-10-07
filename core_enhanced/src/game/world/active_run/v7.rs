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
        let bad = |m: &str| active_run_checkpoint_codec_error("V7 conversion", m);
        if input.len() > 32 * 1024 * 1024 {
            return Err(bad("payload exceeds 32 MiB"));
        }
        let UniqueJson(mut raw) = serde_json::from_slice(input).map_err(|e| bad(&e.to_string()))?;
        if raw.as_object().is_none_or(|o| o.len() != 3) || raw["version"] != 7 {
            return Err(bad("expected exact V7 envelope"));
        }
        let checkpoint = raw
            .get("checkpoint")
            .ok_or_else(|| bad("missing checkpoint"))?;
        let mut bytes = Vec::new();
        write_canonical_json(checkpoint, &mut bytes).map_err(|e| bad(&e.to_string()))?;
        let digest = format!("{:x}", Sha256::digest(&bytes));
        if raw["checkpoint_sha256"].as_str() != Some(&digest) {
            return Err(bad("original checksum mismatch"));
        }
        let identity: ContentIdentity =
            serde_json::from_value(checkpoint["content_identity"].clone())
                .map_err(|e| bad(&e.to_string()))?;
        if &identity != expected {
            return Err(bad("original content identity mismatch"));
        }
        let checkpoint = raw
            .get_mut("checkpoint")
            .and_then(Value::as_object_mut)
            .ok_or_else(|| bad("invalid checkpoint"))?;
        let legacy: LegacyInventory = serde_json::from_value(
            checkpoint
                .get("skill_fragments")
                .cloned()
                .ok_or_else(|| bad("missing fragments"))?,
        )
        .map_err(|e| bad(&e.to_string()))?;
        let mut uuids: UuidManager = serde_json::from_value(
            checkpoint
                .get("uuid_manager")
                .cloned()
                .ok_or_else(|| bad("missing UUID state"))?,
        )
        .map_err(|e| bad(&e.to_string()))?;
        if uuids.fragment_counter() != 0 {
            return Err(bad("V7 contains new fragment UUID stream"));
        }
        let mut fragments = SkillFragmentInventory::new();
        fragments.import_legacy_history(legacy.progress.clone(), legacy.fragment_dust);
        let mut by_definition = BTreeMap::new();
        for (id, stack) in legacy.stacks {
            if stack.count == 0 {
                return Err(bad("zero-count legacy stack"));
            }
            if stack.count as usize > crate::game::skill_fragment::MAX_OWNED_FRAGMENTS {
                return Err(bad("legacy copy limit"));
            }
            let metadata = self
                .game_data
                .skill_fragment_data
                .get_by_id(&id)
                .ok_or_else(|| bad("unknown fragment definition"))?;
            let progress = legacy.progress.get(&id).copied().unwrap_or_default();
            let mut copies = Vec::new();
            for _ in 0..stack.count {
                let copy =
                    fragments.grant(metadata, &mut uuids, &self.state.skill_fragment_policy)?;
                if progress.awakened {
                    fragments.unlock(copy, "awakened", &self.game_data.skill_fragment_data)?;
                    fragments.select(copy, "awakened", &self.game_data.skill_fragment_data)?;
                }
                copies.push(copy);
            }
            by_definition.insert(id, copies.into_iter());
        }
        let employees = checkpoint
            .get_mut("roster")
            .and_then(|r| r.get_mut("employees"))
            .and_then(Value::as_object_mut)
            .ok_or_else(|| bad("missing employees"))?;
        let mut ids = employees.keys().cloned().collect::<Vec<_>>();
        ids.sort_unstable();
        for id in ids {
            let employee = employees
                .get_mut(&id)
                .ok_or_else(|| bad("employee missing"))?;
            let owner: Uuid = id.parse().map_err(|_| bad("invalid employee UUID"))?;
            let old: LegacyLoadout = serde_json::from_value(
                employee
                    .get("skill_fragments")
                    .cloned()
                    .ok_or_else(|| bad("missing loadout"))?,
            )
            .map_err(|e| bad(&e.to_string()))?;
            let mut loadout = crate::game::skill_fragment::SkillFragmentLoadout::with_baseline_ids(
                old.baseline_ids,
            );
            if let Some(definition) = old.active_fragment_id {
                let copy = by_definition
                    .get_mut(&definition)
                    .and_then(Iterator::next)
                    .ok_or_else(|| bad("legacy owner count exceeds copies"))?;
                fragments.import_legacy_binding(copy, owner)?;
                loadout.set_copy(Some(copy));
            }
            employee["skill_fragments"] =
                serde_json::to_value(loadout).map_err(|e| bad(&e.to_string()))?;
        }
        checkpoint.insert(
            "skill_fragments".into(),
            serde_json::to_value(fragments).map_err(|e| bad(&e.to_string()))?,
        );
        checkpoint.insert(
            "uuid_manager".into(),
            serde_json::to_value(uuids).map_err(|e| bad(&e.to_string()))?,
        );
        checkpoint.insert(
            "content_identity".into(),
            serde_json::to_value(self.game_data.identity()).map_err(|e| bad(&e.to_string()))?,
        );
        super::v8::insert_breakdown_counters(
            checkpoint,
            self.run_policy().corrosion.breakdown_collapse_node_limit,
        )?;
        super::convert_legacy_employee_resonance(checkpoint)?;
        let candidate: RunCheckpointPayload =
            serde_json::from_value(Value::Object(checkpoint.clone()))
                .map_err(|e| bad(&e.to_string()))?;
        self.validate_run_checkpoint_payload_for_continue(&candidate)?;
        encode_active_run_checkpoint(&candidate)
    }
}
