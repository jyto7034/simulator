-- Player decks table
CREATE TABLE IF NOT EXISTS player_decks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    player_id BIGINT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    deck_name VARCHAR(100) NOT NULL,
    cover_card_id INT REFERENCES cards(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (player_id, deck_name)
);
COMMENT ON TABLE player_decks IS 'List of decks created by players';
CREATE TRIGGER trigger_player_decks_updated_at
BEFORE UPDATE ON player_decks
FOR EACH ROW
EXECUTE FUNCTION update_modified_column();