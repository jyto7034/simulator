-- Players table for Steam Player Account Mapping
CREATE TABLE IF NOT EXISTS players (
    id BIGINT PRIMARY KEY, -- SteamID64 as primary key
    last_known_username VARCHAR(64), -- Steam nickname snapshot
    status player_status NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMPTZ
);
COMMENT ON TABLE players IS 'Core table linking Steam player accounts with game data';
COMMENT ON COLUMN players.id IS 'Unique player ID (SteamID64)';
COMMENT ON COLUMN players.last_known_username IS 'Last known player Steam nickname';
COMMENT ON COLUMN players.status IS 'Account status (ENUM: active, suspended, banned)';
CREATE TRIGGER trigger_players_updated_at
BEFORE UPDATE ON players
FOR EACH ROW
EXECUTE FUNCTION update_modified_column();