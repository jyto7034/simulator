use super::TargetCondition;

pub enum AutoSelectType {
    Weakest,
    Strongest,
    Random,
    All,
}

// 자동 선택기 (가장 약한 카드, 가장 강한 카드 등)
pub struct AutomaticSelector {
    condition: TargetCondition,
    selection_type: AutoSelectType,
}
