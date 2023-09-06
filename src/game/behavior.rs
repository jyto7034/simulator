use crate::game::Game;
use crate::enums::SpellType;

pub enum Behavior {
    EndGame,
    
    CastingSpell(SpellType),
    InterruptSpell,
    BeUnderSpell,

    GiveDamageTo,
    BeDamaged,

    DrawCard,
    InsertCard,
    DestroyCard,

}