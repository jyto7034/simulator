use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BonusType {
    Gold,
    Experience,
    Item,
    Abnormality,
}
