-- Deck cards table for deck composition
CREATE TABLE IF NOT EXISTS deck_cards (
    deck_id UUID NOT NULL REFERENCES player_decks(id) ON DELETE CASCADE,
    card_id INT NOT NULL REFERENCES cards(id) ON DELETE RESTRICT,
    quantity INT NOT NULL CHECK (quantity > 0 AND quantity <= 2),
    PRIMARY KEY (deck_id, card_id)
);
COMMENT ON TABLE deck_cards IS 'Information about cards included in each deck';