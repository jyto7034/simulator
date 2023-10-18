use std::fmt;
#[derive(Debug, PartialEq)]
pub enum Exception {
    PlayerInitializeFailed,
    PlayerDataNotIntegrity,
    GameInitializeFailed,
    DifferentCardTypes,
    GenerateUUIDFaild,
    ExceededCardLimit,
    FailedToDrawCard,
    NothingToRemove,
    JsonParseFailed,
    DeckParseError,
    ReadFileFailed,
    NoCardsLeft,
    NoCardLeft,
    CardError,
    Ok,
}

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlayerInitializeFailed => write!(f, "PlayerInitializeFailed"),
            Self::PlayerDataNotIntegrity => write!(f, "PlayerDataNotIntegrity"),
            Self::GenerateUUIDFaild => write!(f, "GenerateUUIDFaild"),
            Self::DeckParseError => write!(f, "Deck Parse Error"),
            Self::Ok => write!(f, "Ok"),
            _ => write!(f, ""),
        }
    }
}
