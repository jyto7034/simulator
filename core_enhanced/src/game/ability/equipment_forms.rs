//! Copy-owned forms selected by qualified victories. Only the typed resource
//! store owns progression; presentation and effects are projections of it.
use crate::game::resource::{ResourceCatalog, ResourceId, ResourceLifetime, ResourceOwnerKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentVictoryForms {
    pub pending: ResourceId,
    pub confirmed: ResourceId,
    pub forms: Vec<EquipmentVictoryForm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentVictoryForm {
    pub name: String,
    pub description: String,
    pub appearance: String,
    /// Zero only for the final form. All other values are stage-local.
    pub wins_required: u32,
    pub effect: EquipmentFormEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EquipmentFormEffect {
    MaxHealth { amount: u32 },
    Attack { amount: u32 },
    IncomingDamage { percent: u32 },
}
crate::authoring::clone_shape::copy_shape!(EquipmentFormEffect);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentFormView {
    pub index: u32,
    pub progress: u32,
    pub wins_required: u32,
    pub name: String,
    pub description: String,
    pub appearance: String,
    pub effect: EquipmentFormEffect,
}
crate::authoring::clone_shape::struct_shape!(EquipmentFormView {
    index, progress, wins_required, name, description, appearance, effect
});

impl EquipmentVictoryForms {
    pub(crate) fn selected(&self, confirmed: u32) -> (usize, u32, &EquipmentVictoryForm) {
        let mut progress = confirmed;
        for (index, form) in self.forms.iter().enumerate() {
            if form.wins_required == 0 || progress < form.wins_required {
                return (index, if form.wins_required == 0 { 0 } else { progress }, form);
            }
            progress -= form.wins_required;
        }
        unreachable!("validated forms always have a final form")
    }

    pub(crate) fn view(&self, confirmed: u32) -> EquipmentFormView {
        let (index, progress, form) = self.selected(confirmed);
        EquipmentFormView {
            index: index as u32, progress, wins_required: form.wins_required,
            name: form.name.clone(), description: form.description.clone(),
            appearance: form.appearance.clone(), effect: form.effect,
        }
    }

    pub(crate) fn validate(&self, resources: &ResourceCatalog) -> Result<(), (String, String)> {
        let fail = |field: &str, message: &str| (format!("program.victory_forms.{field}"), message.into());
        if !(2..=16).contains(&self.forms.len()) {
            return Err(fail("forms", "requires 2..16 forms"));
        }
        let mut total = 0u32;
        for (i, form) in self.forms.iter().enumerate() {
            let path = format!("forms[{i}]");
            if (form.wins_required == 0) != (i + 1 == self.forms.len()) {
                return Err(fail(&path, "only the final form must require zero wins"));
            }
            total = total.checked_add(form.wins_required).ok_or_else(|| fail(&path, "win total overflow"))?;
            for (field, text, cap) in [("name", &form.name, 128), ("description", &form.description, 2048), ("appearance", &form.appearance, 128)] {
                if text.trim().is_empty() || text.len() > cap || text.chars().any(char::is_control) {
                    return Err(fail(&format!("{path}.{field}"), "requires bounded nonempty text without control characters"));
                }
            }
            let value = match form.effect {
                EquipmentFormEffect::MaxHealth { amount } | EquipmentFormEffect::Attack { amount } => amount,
                EquipmentFormEffect::IncomingDamage { percent } => percent,
            };
            if value == 0 || value > i32::MAX as u32 {
                return Err(fail(&format!("{path}.effect"), "effect must be positive and fit i32"));
            }
        }
        for (field, id, lifetime, cap) in [
            ("pending", &self.pending, ResourceLifetime::Attempt, 1),
            ("confirmed", &self.confirmed, ResourceLifetime::Run, total),
        ] {
            let declaration = resources.resources.iter().find(|r| &r.id == id)
                .ok_or_else(|| fail(field, "unknown form resource"))?;
            if declaration.owner != ResourceOwnerKind::EquipmentCopy || declaration.lifetime != lifetime
                || declaration.initial != 0 || declaration.cap != cap {
                return Err(fail(field, "form resources require EquipmentCopy, zero initial, matching lifetime and exact cap (pending 1, confirmed total wins)"));
            }
        }
        Ok(())
    }

    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = self.pending.retained_text_capacity().checked_add(self.confirmed.retained_text_capacity())?
            .checked_add(self.forms.capacity().checked_mul(std::mem::size_of::<EquipmentVictoryForm>())?)?;
        for form in &self.forms {
            bytes = bytes.checked_add(form.name.capacity())?.checked_add(form.description.capacity())?.checked_add(form.appearance.capacity())?;
        }
        Some(bytes)
    }
}
