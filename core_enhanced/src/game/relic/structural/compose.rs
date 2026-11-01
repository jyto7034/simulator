//! Checked, order-independent composition; no runtime state or hidden clamping.
use super::{authoring::*, capabilities::RelicCapabilityDescriptor, FaultCode, RelicPlanError};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
pub struct NumericProgram {
    pub(crate) replacement: Option<TypedRelicValue>,
    pub(crate) add: i128,
    pub(crate) scale_increment_percent: i128,
}
impl NumericProgram {
    pub(crate) fn include(
        &mut self,
        op: NumericEditDef,
        d: &RelicCapabilityDescriptor,
    ) -> Result<(), RelicPlanError> {
        if !d.numeric {
            return Err(RelicPlanError::new(
                FaultCode::UnsupportedOperation,
                "numeric edit of recipe-only property",
            ));
        }
        match op {
            NumericEditDef::Scale { increment_percent } => {
                self.scale_increment_percent = self
                    .scale_increment_percent
                    .checked_add(i128::from(increment_percent))
                    .ok_or_else(overflow)?;
            }
            NumericEditDef::Add(v) | NumericEditDef::Replace(v) => {
                if v.unit() != d.unit {
                    return Err(RelicPlanError::new(
                        FaultCode::UnitMismatch,
                        "property/edit units differ",
                    ));
                }
                if matches!(op, NumericEditDef::Add(_)) {
                    self.add = self
                        .add
                        .checked_add(i128::from(v.value()))
                        .ok_or_else(overflow)?;
                } else {
                    if self.replacement.is_some_and(|old| old != v) {
                        return Err(RelicPlanError::new(
                            FaultCode::ReplaceConflict,
                            "different typed Replace values",
                        ));
                    }
                    self.replacement = Some(v);
                }
            }
        }
        Ok(())
    }
    pub fn apply(
        &self,
        base: TypedRelicValue,
        d: &RelicCapabilityDescriptor,
    ) -> Result<TypedRelicValue, RelicPlanError> {
        self.apply_additional_percent(base, d, 0)
    }
    /// Native positive-strike progression joins the already normalized Scale
    /// class. Keep Replace/Add/Scale and its one final division unchanged.
    pub(crate) fn apply_additional_percent(
        &self,
        base: TypedRelicValue,
        d: &RelicCapabilityDescriptor,
        additional_percent: i128,
    ) -> Result<TypedRelicValue, RelicPlanError> {
        if base.unit() != d.unit {
            return Err(RelicPlanError::new(
                FaultCode::UnitMismatch,
                "base/property units differ",
            ));
        }
        let base = self.replacement.unwrap_or(base);
        let factor = 100i128
            .checked_add(self.scale_increment_percent)
            .and_then(|v| v.checked_add(additional_percent))
            .ok_or_else(overflow)?;
        if factor < 0 {
            return Err(RelicPlanError::new(
                FaultCode::OutOfRange,
                "negative Scale factor",
            ));
        }
        let result = i128::from(base.value())
            .checked_add(self.add)
            .and_then(|v| v.checked_mul(factor))
            .ok_or_else(overflow)?
            / 100;
        if result < i128::from(d.minimum) || result > i128::from(d.maximum) {
            return Err(RelicPlanError::new(
                FaultCode::OutOfRange,
                "effective property outside consumer range",
            ));
        }
        Ok(base.with_value(i64::try_from(result).map_err(|_| overflow())?))
    }
    pub(crate) fn validate_dynamic(
        &self,
        d: &RelicCapabilityDescriptor,
    ) -> Result<(), RelicPlanError> {
        if self.scale_increment_percent < -100 {
            return Err(RelicPlanError::new(
                FaultCode::OutOfRange,
                "negative Scale factor",
            ));
        }
        if let Some(v) = self.replacement {
            self.apply(v, d)?;
        }
        Ok(())
    }
}
fn overflow() -> RelicPlanError {
    RelicPlanError::new(
        FaultCode::ArithmeticOverflow,
        "checked composition overflow",
    )
}
