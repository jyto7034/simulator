-- Cards master data table
CREATE TABLE IF NOT EXISTS cards (
    id SERIAL PRIMARY KEY,
    internal_name VARCHAR(100) UNIQUE NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    rarity card_rarity,
    mana_cost INT NOT NULL DEFAULT 0,
    attack INT,
    health INT,
    card_type VARCHAR(50),
    image_url TEXT,
    attributes JSONB,
    is_collectible BOOLEAN NOT NULL DEFAULT TRUE
);
COMMENT ON TABLE cards IS 'Master table for all card information in the game';
-- Insert sample card data
INSERT INTO cards (id, internal_name, display_name, rarity, mana_cost, attack, health, card_type, is_collectible) VALUES
(1, 'fireball', 'Fireball', 'common', 4, 6, 0, 'spell', TRUE),
(2, 'frost_golem', 'Frost Golem', 'rare', 5, 4, 5, 'minion', TRUE)
ON CONFLICT (id) DO NOTHING;