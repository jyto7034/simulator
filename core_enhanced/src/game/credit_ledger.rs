//! Run-scoped credit belongs to the exact fragment UUID; equipment transfers keep it.
use crate::{
    authoring::{clone_shape::controlled_clone, control},
    game::{behavior::GameError, skill_fragment::SkillFragmentInventory},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreditCarry {
    pub definition: String,
    pub unpaid: u32,
    pub borrowed: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreditLedgers {
    #[serde(deserialize_with = "crate::game::skill_fragment::bounded::map::<_, _, _, 1024>")]
    pub entries: BTreeMap<Uuid, CreditCarry>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditSettlement {
    pub copy: Uuid,
    pub before: Option<CreditCarry>,
    pub after: CreditCarry,
}
crate::authoring::clone_shape::struct_shape!(CreditCarry {
    definition,
    unpaid,
    borrowed
});
crate::authoring::clone_shape::struct_shape!(CreditLedgers { entries });
crate::authoring::clone_shape::struct_shape!(CreditSettlement {
    copy,
    before,
    after
});
impl CreditLedgers {
    pub fn retained_extent(&self) -> Option<usize> {
        self.entries
            .values()
            .try_fold(0usize, |n, v| {
                n.checked_add(std::mem::size_of::<(Uuid, CreditCarry)>() + 128)?
                    .checked_add(v.definition.capacity())
            })
    }
    pub fn preflight(&self) -> Result<(), GameError> {
        // The inline map header belongs to RunState/checkpoint storage. An
        // empty ledger owns no allocation and must not open a growth budget
        // before the caller establishes its actual owner extent.
        if self.entries.is_empty() {
            return Ok(());
        }
        control::charge(self.entries.len() as u64, "credit_ledger_scan")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.retained_extent(),
            "credit_ledger_storage",
        )
    }
    pub fn validate(&self, inventory: &SkillFragmentInventory) -> Result<(), GameError> {
        self.preflight()?;
        if self.entries.len() > 1024
            || self.entries.iter().any(|(k, v)| {
                inventory.get(k).is_none()
                    || v.definition.trim().is_empty()
                    || v.unpaid > i32::MAX as u32
            })
        {
            return Err(GameError::InvalidAction);
        }
        Ok(())
    }
    pub fn reconcile(
        &mut self,
        inventory: &SkillFragmentInventory,
        terminal: bool,
    ) -> Result<(), GameError> {
        self.preflight()?;
        if terminal {
            self.entries.clear();
        } else {
            self.entries.retain(|copy, _| inventory.get(copy).is_some());
        }
        Ok(())
    }
    pub fn apply(&mut self, results: &[CreditSettlement]) -> Result<(), GameError> {
        self.preflight()?;
        control::charge(results.len() as u64, "credit_settlement")?;
        let mut seen = std::collections::BTreeSet::new();
        for r in results {
            if !seen.insert(r.copy) || self.entries.get(&r.copy) != r.before.as_ref() {
                return Err(GameError::InvalidAction);
            }
        }
        let mut next = controlled_clone(self)?;
        for r in results {
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(std::mem::size_of::<CreditCarry>() + 128 + r.after.definition.len()),
                "credit_settlement_storage",
            )?;
            next.entries.insert(r.copy, controlled_clone(&r.after)?);
        }
        if next.entries.len() > 1024 {
            return Err(GameError::InvalidAction);
        }
        control::poll("credit_settlement_ready")?;
        *self = next;
        Ok(())
    }
}
