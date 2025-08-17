-- Player profiles table for game-specific profile and stats
CREATE TABLE IF NOT EXISTS player_profiles (
    player_id BIGINT PRIMARY KEY REFERENCES players(id) ON DELETE CASCADE,
    mmr DOUBLE PRECISION NOT NULL DEFAULT 1500.0,
    rd DOUBLE PRECISION NOT NULL DEFAULT 350.0,
    volatility DOUBLE PRECISION NOT NULL DEFAULT 0.06,
    last_rating_update_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tier_id INT NOT NULL REFERENCES tiers(id),
    rank_points INT NOT NULL DEFAULT 0,
    experience_points BIGINT NOT NULL DEFAULT 0,
    level INT NOT NULL DEFAULT 1,
    -- Game-specific profile customization items
    -- custom_profile_icon_id VARCHAR(100),
    -- custom_profile_banner_id VARCHAR(100),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
COMMENT ON TABLE player_profiles IS 'In-game player profiles, rank, and stats information';
COMMENT ON COLUMN player_profiles.player_id IS 'Player SteamID64';
CREATE INDEX IF NOT EXISTS idx_player_profiles_mmr ON player_profiles(mmr);
CREATE TRIGGER trigger_player_profiles_updated_at
BEFORE UPDATE ON player_profiles
FOR EACH ROW
EXECUTE FUNCTION update_modified_column();