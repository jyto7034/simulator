-- Player card collection table
CREATE TABLE IF NOT EXISTS player_card_collection (
    player_id BIGINT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    card_id INT NOT NULL REFERENCES cards(id) ON DELETE RESTRICT,
    quantity INT NOT NULL DEFAULT 1 CHECK (quantity > 0),
    is_new BOOLEAN NOT NULL DEFAULT TRUE,
    PRIMARY KEY (player_id, card_id)
);
COMMENT ON TABLE player_card_collection IS 'List of cards owned by players';