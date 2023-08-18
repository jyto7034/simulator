pub enum GameStep {
    GameStart,
    GameEnd,
    RoundStart,
    RoundEnd,
    AttackTurn,
    DefenseTurn,
    Execution,
    Mulligun,
}

pub enum GameState {
    GameStart,
    GameEnd,
    RoundStart,
    RoundEnd,
    AttackTurn,
    DefenseTurn,
    Execution,
    Mulligun,
}

pub enum EntityType {
    Player,
    Hero,
    Agent,
}

pub enum PlayerType {}

pub enum SpellType {
    SlowSpell,
    FastSpell,
}

pub enum CardType {
    Dummy,
    Agent,
    Spell(SpellType),
    Field,
}

pub enum HeroType {
    Name1,
    Name2,
}

pub const MAX_CARD_SIZE: u32 = 30;
