use super::{SelectorLogic, TargetCondition};

pub struct ComplexSelector {
    conditions: Vec<TargetCondition>,
    logic: SelectorLogic,
}
