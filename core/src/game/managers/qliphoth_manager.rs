use crate::{config::balance, ecs::resources::Qliphoth};
use tracing::info;

/// 클리포트 관리 헬퍼
pub struct QliphothManager;

impl QliphothManager {
    /// 전투 후 클리포트 감소
    ///
    /// 투영체를 사용하면 원본 환상체가 불안정해짐
    pub fn apply_battle_cost(qliphoth: &mut Qliphoth) {
        let changes = balance::qliphoth_changes();
        let old_amount = qliphoth.amount();
        qliphoth.decrease(changes.battle_cost);

        info!(
            "Battle cost applied: {} → {} (cost: {})",
            old_amount,
            qliphoth.amount(),
            changes.battle_cost
        );
    }

    /// 진압 성공 시 클리포트 증가
    pub fn apply_suppress_success(qliphoth: &mut Qliphoth) {
        let changes = balance::qliphoth_changes();
        let old_amount = qliphoth.amount();
        qliphoth.increase(changes.suppress_success);

        info!(
            "Suppress success: {} → {} (recovery: {})",
            old_amount,
            qliphoth.amount(),
            changes.suppress_success
        );
    }

    /// 진압 실패 시 클리포트 감소
    pub fn apply_suppress_failure(qliphoth: &mut Qliphoth) {
        let changes = balance::qliphoth_changes();
        let old_amount = qliphoth.amount();
        qliphoth.decrease(changes.suppress_failure);

        info!(
            "Suppress failure: {} → {} (penalty: {})",
            old_amount,
            qliphoth.amount(),
            changes.suppress_failure
        );
    }

    /// Breach 성공 시 클리포트 증가 (하이리스크 하이리턴)
    pub fn apply_breach_success(qliphoth: &mut Qliphoth) {
        let changes = balance::qliphoth_changes();
        let old_amount = qliphoth.amount();
        qliphoth.increase(changes.breach_success);

        info!(
            "Breach success: {} → {} (recovery: {})",
            old_amount,
            qliphoth.amount(),
            changes.breach_success
        );
    }

    /// Breach 실패 시 클리포트 감소 (큰 페널티)
    pub fn apply_breach_failure(qliphoth: &mut Qliphoth) {
        let changes = balance::qliphoth_changes();
        let old_amount = qliphoth.amount();
        qliphoth.decrease(changes.breach_failure);

        info!(
            "Breach failure: {} → {} (penalty: {})",
            old_amount,
            qliphoth.amount(),
            changes.breach_failure
        );
    }

    /// 페이즈 종료 시 자동 회복
    pub fn apply_phase_recovery(qliphoth: &mut Qliphoth) {
        let changes = balance::qliphoth_changes();
        let old_amount = qliphoth.amount();
        qliphoth.increase(changes.phase_recovery);

        info!(
            "Phase recovery: {} → {} (recovery: {})",
            old_amount,
            qliphoth.amount(),
            changes.phase_recovery
        );
    }

    /// 특수 아이템 사용 시 회복
    pub fn apply_item_recovery(qliphoth: &mut Qliphoth) {
        let changes = balance::qliphoth_changes();
        let old_amount = qliphoth.amount();
        qliphoth.increase(changes.item_recovery);

        info!(
            "Item recovery: {} → {} (recovery: {})",
            old_amount,
            qliphoth.amount(),
            changes.item_recovery
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battle_cost() {
        let mut qliphoth = Qliphoth::new();
        let initial = qliphoth.amount();

        QliphothManager::apply_battle_cost(&mut qliphoth);

        // Then: 기본값 battle_cost=1
        assert_eq!(qliphoth.amount(), initial - 1);
    }

    #[test]
    fn test_suppress_success() {
        let mut qliphoth = Qliphoth::new();
        // Given: 중간값으로 설정
        qliphoth.set_amount(5);

        let initial = qliphoth.amount();
        QliphothManager::apply_suppress_success(&mut qliphoth);

        // Then: 기본값 suppress_success=2
        assert_eq!(qliphoth.amount(), initial + 2);
    }

    #[test]
    fn test_suppress_failure() {
        let mut qliphoth = Qliphoth::new();
        let initial = qliphoth.amount();

        QliphothManager::apply_suppress_failure(&mut qliphoth);

        // Then: 기본값 suppress_failure=1
        assert_eq!(qliphoth.amount(), initial - 1);
    }

    #[test]
    fn test_breach_success() {
        let mut qliphoth = Qliphoth::new();
        qliphoth.set_amount(5);

        let initial = qliphoth.amount();
        QliphothManager::apply_breach_success(&mut qliphoth);

        // Then: 기본값 breach_success=3
        assert_eq!(qliphoth.amount(), initial + 3);
    }

    #[test]
    fn test_breach_failure() {
        let mut qliphoth = Qliphoth::new();
        let initial = qliphoth.amount();

        QliphothManager::apply_breach_failure(&mut qliphoth);

        // Then: 기본값 breach_failure=2
        assert_eq!(qliphoth.amount(), initial - 2);
    }

    #[test]
    fn test_phase_recovery() {
        let mut qliphoth = Qliphoth::new();
        qliphoth.set_amount(5);

        let initial = qliphoth.amount();
        QliphothManager::apply_phase_recovery(&mut qliphoth);

        // Then: 기본값 phase_recovery=1
        assert_eq!(qliphoth.amount(), initial + 1);
    }
}
