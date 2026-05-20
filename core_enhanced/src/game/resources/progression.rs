#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QliphothLevel {
    Stable,
    Caution,
    Critical,
    Meltdown,
}

/// 클리포드
#[derive(Debug, Clone)]
pub struct Qliphoth {
    pub level: QliphothLevel,
    pub amount: u32,
}

impl Qliphoth {
    pub fn new() -> Qliphoth {
        use crate::config::balance;
        let thresholds = balance::qliphoth_thresholds();

        Self {
            level: QliphothLevel::Stable,
            amount: thresholds.stable_min,
        }
    }

    pub fn level(&self) -> QliphothLevel {
        self.level
    }

    pub fn amount(&self) -> u32 {
        self.amount
    }

    pub fn increase(&mut self, amount: u32) {
        use crate::config::balance;
        let thresholds = balance::qliphoth_thresholds();

        self.amount = (self.amount + amount).min(thresholds.stable_min);
        self.update_level();
    }

    pub fn decrease(&mut self, amount: u32) {
        self.amount = self.amount.saturating_sub(amount);
        self.update_level();
    }

    pub fn set_amount(&mut self, amount: u32) {
        use crate::config::balance;
        let thresholds = balance::qliphoth_thresholds();

        self.amount = amount.min(thresholds.stable_min);
        self.update_level();
    }

    fn update_level(&mut self) {
        use crate::config::balance;
        let thresholds = balance::qliphoth_thresholds();

        self.level = match self.amount {
            x if x >= thresholds.stable_max => QliphothLevel::Stable,
            x if x >= thresholds.caution_max => QliphothLevel::Caution,
            x if x >= thresholds.critical_max => QliphothLevel::Critical,
            _ => QliphothLevel::Meltdown,
        };
    }
}

impl Default for Qliphoth {
    fn default() -> Self {
        Self::new()
    }
}
