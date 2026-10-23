//! Permanent exact ownership and identity, separate from monotonic knowledge.
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::game::{
    data::{equipment_data::EquipmentType, skill_fragment_data::SkillFragmentId},
    resources::inventory::{Inventory, OwnedEquipment},
    skill_fragment::{OwnedFragment, SkillFragmentInventory},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpeditionInstance {
    pub player_id: Uuid,
    pub sequence: u64,
    pub uuid: Uuid,
}

impl ExpeditionInstance {
    pub fn new(player_id: Uuid, sequence: u64) -> Result<Self, String> {
        if player_id.is_nil() || sequence == 0 {
            return Err("expedition instance requires player and positive sequence".into());
        }
        let mut hash = Sha256::new();
        hash.update(b"core.expedition.instance.v1\0");
        hash.update(player_id.as_bytes());
        hash.update(sequence.to_le_bytes());
        let mut bytes = [0; 16];
        bytes.copy_from_slice(&hash.finalize()[..16]);
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        Ok(Self { player_id, sequence, uuid: Uuid::from_bytes(bytes) })
    }

    pub fn validate(&self) -> Result<(), String> {
        if *self != Self::new(self.player_id, self.sequence)? {
            return Err("expedition instance UUID is inconsistent".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpeditionContinuationChoice { Return, Continue, Abandon }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpeditionContinuationCause { Gate, Exhaustion }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingExpeditionContinuation {
    pub node_id: crate::game::map::MapNodeId,
    pub depth: u32,
    pub cause: ExpeditionContinuationCause,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FirstReturnStory {
    #[default]
    Never,
    Pending,
    Consumed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VaultEquipment {
    #[serde(default)]
    pub weapon_stage: crate::game::ability::weapon_marks::WeaponStage,
    pub copy_uuid: Uuid,
    pub definition_id: String,
    pub equipment_type: EquipmentType,
    pub enhancement_level: u8,
}

impl VaultEquipment {
    pub fn from_returned(copy: &OwnedEquipment) -> Self {
        Self {
            weapon_stage: copy.weapon_stage,
            copy_uuid: copy.instance_uuid,
            definition_id: copy.meta.id.clone(),
            equipment_type: copy.meta.equipment_type,
            enhancement_level: if copy.meta.equipment_type == EquipmentType::Weapon {
                0
            } else {
                copy.enhancement_level
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VaultFragment {
    pub investment: crate::game::material_economy::MaterialInvestment,
    pub copy_uuid: Uuid,
    pub definition_id: SkillFragmentId,
    #[serde(deserialize_with = "crate::game::skill_fragment::bounded::forms")]
    pub unlocked_forms: BTreeSet<String>,
    pub selected_form: String,
}

impl From<&OwnedFragment> for VaultFragment {
    fn from(copy: &OwnedFragment) -> Self {
        Self {
            investment: copy.investment.clone(),
            copy_uuid: copy.copy_uuid,
            definition_id: copy.definition_id.clone(),
            unlocked_forms: copy.unlocked_forms.clone(),
            selected_form: copy.selected_form.clone(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpeditionVault {
    pub materials: BTreeMap<String, u32>,
    #[serde(deserialize_with = "unique_uuid_map")]
    pub equipment: BTreeMap<Uuid, VaultEquipment>,
    #[serde(deserialize_with = "unique_uuid_map")]
    pub fragments: BTreeMap<Uuid, VaultFragment>,
}

impl ExpeditionVault {
    pub fn validate(&self) -> Result<(), String> {
        let mut ids = BTreeSet::new();
        for (key, copy) in &self.equipment {
            validate_copy(*key, copy.copy_uuid, &copy.definition_id, &mut ids)?;
            if copy.equipment_type == EquipmentType::Weapon && copy.enhancement_level != 0 {
                return Err("vault weapon enhancement must be zero".into());
            }
        }
        for (key, copy) in &self.fragments {
            validate_copy(*key, copy.copy_uuid, copy.definition_id.as_str(), &mut ids)?;
            if copy.unlocked_forms.is_empty()
                || copy.unlocked_forms.len() > crate::game::data::skill_fragment_data::MAX_FRAGMENT_FORMS
                || !copy.unlocked_forms.contains(crate::game::data::skill_fragment_data::BASE_FRAGMENT_FORM)
                || !copy.unlocked_forms.contains(&copy.selected_form)
                || copy.unlocked_forms.iter().any(|form| form.len() > 64 || form.is_empty() || !form.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')))
            {
                return Err("vault fragment forms are inconsistent".into());
            }
        }
        Ok(())
    }

    pub fn validate_active_ownership(
        &self,
        inventory: &Inventory,
        fragments: &SkillFragmentInventory,
    ) -> Result<(), String> {
        self.validate()?;
        let mut ids = self.equipment.keys().chain(self.fragments.keys()).copied()
            .collect::<BTreeSet<_>>();
        for copy in inventory.equipments.iter() {
            if copy.instance_uuid.is_nil() || !ids.insert(copy.instance_uuid) {
                return Err("equipment copy has multiple owners".into());
            }
        }
        for copy in fragments.copies() {
            if copy.copy_uuid.is_nil() || !ids.insert(copy.copy_uuid) {
                return Err("fragment copy has multiple owners".into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpeditionLifecycle {
    pub warehouse_debits: BTreeMap<String, u32>,
    pub expedition_sequence: u64,
    pub active_expedition: Option<ExpeditionInstance>,
    pub vault: ExpeditionVault,
    pub first_return_story: FirstReturnStory,
}

impl ExpeditionLifecycle {
    pub fn validate(&self) -> Result<(), String> {
        self.vault.validate()?;
        if let Some(instance) = self.active_expedition {
            instance.validate()?;
            if instance.sequence != self.expedition_sequence {
                return Err("active expedition sequence is inconsistent".into());
            }
        }
        if self.expedition_sequence == 0 && self.first_return_story != FirstReturnStory::Never {
            return Err("return story cannot precede the first expedition".into());
        }
        Ok(())
    }

    pub fn next_instance(&self, player_id: Uuid) -> Result<ExpeditionInstance, String> {
        self.validate()?;
        if self.active_expedition.is_some() {
            return Err("an expedition must be terminated before departure".into());
        }
        ExpeditionInstance::new(player_id, self.expedition_sequence.checked_add(1)
            .ok_or("expedition sequence overflow")?)
    }
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_alphanumeric()
        || matches!(byte, b'_' | b'-' | b'.'))
}

fn unique_uuid_map<'de, D, T>(deserializer: D) -> Result<BTreeMap<Uuid, T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct UniqueMap<T>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for UniqueMap<T> {
        type Value = BTreeMap<Uuid, T>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a vault map with unique UUID keys")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut access: A,
        ) -> Result<Self::Value, A::Error> {
            let mut result = BTreeMap::new();
            while let Some(key) = access.next_key::<Uuid>()? {
                if result.contains_key(&key) {
                    return Err(serde::de::Error::custom("duplicate vault copy UUID key"));
                }
                let value = access.next_value()?;
                result.insert(key, value);
            }
            Ok(result)
        }
    }
    deserializer.deserialize_map(UniqueMap(std::marker::PhantomData))
}

fn validate_copy(
    key: Uuid,
    embedded: Uuid,
    definition: &str,
    ids: &mut BTreeSet<Uuid>,
) -> Result<(), String> {
    if key.is_nil() || key != embedded || !ids.insert(key) || !valid_id(definition) {
        return Err("vault copy identity, ownership or canonical definition is invalid".into());
    }
    Ok(())
}

crate::authoring::clone_shape::copy_shape!(ExpeditionInstance, FirstReturnStory, PendingExpeditionContinuation);
crate::authoring::clone_shape::struct_shape!(VaultEquipment {
    copy_uuid, definition_id, equipment_type, enhancement_level, weapon_stage
});
crate::authoring::clone_shape::struct_shape!(VaultFragment {
    copy_uuid, definition_id, unlocked_forms, selected_form, investment
});
crate::authoring::clone_shape::struct_shape!(ExpeditionVault { equipment, fragments, materials });
crate::authoring::clone_shape::struct_shape!(ExpeditionLifecycle {
    expedition_sequence, active_expedition, vault, first_return_story, warehouse_debits
});
