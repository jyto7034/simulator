use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enkephalin {
    pub amount: u32,
}

impl Enkephalin {
    pub fn new(initial_amount: u32) -> Self {
        Self {
            amount: initial_amount,
        }
    }

    pub fn checked_add(&mut self, amount: u32) -> Result<u32, crate::game::behavior::GameError> {
        self.amount = self
            .amount
            .checked_add(amount)
            .ok_or(crate::game::behavior::GameError::InvalidAction)?;
        Ok(self.amount)
    }
}
