use std::fmt;
#[derive(Debug, PartialEq)]
pub enum Exception {
    PlayerInitializeFailed,
    PlayerDataNotIntegrity,
    PathNotExist,
    GameInitializeFailed,
    DifferentCardTypes,
    GenerateUUIDFaild,
    ExceededCardLimit,
    FailedToDrawCard,
    NothingToRemove,
    JsonParseFailed,
    DecodeError,
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
            Self::JsonParseFailed => write!(f, "Json Parse Failed"),
            Self::DeckParseError => write!(f, "Deck Parse Error"),
            Self::PathNotExist => write!(f, "Path Not Exist"),
            Self::CardError => write!(f, "Card Error"),
            Self::Ok => write!(f, "Ok"),
            _ => write!(f, ""),
        }
    }
}
