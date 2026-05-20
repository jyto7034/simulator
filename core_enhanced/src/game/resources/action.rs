use std::collections::HashSet;

use crate::game::behavior::{ActionKind, PlayerBehavior};

/// 현재 상태에서 허용되는 액션 capability를 저장한다.
#[derive(Default)]
pub struct ActionValidator {
    allowed_actions: HashSet<ActionKind>,
}

impl ActionValidator {
    pub fn new() -> Self {
        Self {
            allowed_actions: HashSet::new(),
        }
    }

    pub fn set_allowed_actions(&mut self, actions: Vec<ActionKind>) {
        self.allowed_actions = actions.into_iter().collect();
    }

    pub fn is_kind_allowed(&self, action: ActionKind) -> bool {
        self.allowed_actions.contains(&action)
    }

    pub fn is_action_allowed(&self, action: &PlayerBehavior) -> bool {
        self.is_kind_allowed(action.kind())
    }

    pub fn allowed_actions(&self) -> Vec<ActionKind> {
        let mut actions = self.allowed_actions.iter().copied().collect::<Vec<_>>();
        actions.sort();
        actions
    }

    pub fn clear(&mut self) {
        self.allowed_actions.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_actions_are_sorted_and_clearable() {
        let mut validator = ActionValidator::new();
        validator.set_allowed_actions(vec![ActionKind::SellItem, ActionKind::StartNewGame]);

        assert!(validator.is_kind_allowed(ActionKind::StartNewGame));
        assert_eq!(
            validator.allowed_actions(),
            vec![ActionKind::StartNewGame, ActionKind::SellItem]
        );

        validator.clear();
        assert!(validator.allowed_actions().is_empty());
    }
}
