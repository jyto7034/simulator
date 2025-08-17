-- Match history table for completed games
CREATE TABLE IF NOT EXISTS match_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    game_mode_id INT NOT NULL REFERENCES game_modes(id),
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ NOT NULL,
    duration_seconds INT GENERATED ALWAYS AS (EXTRACT(EPOCH FROM (ended_at - started_at))::INT) STORED,
    winning_team_id INT,
    additional_data JSONB
);
COMMENT ON TABLE match_history IS 'Records of all completed games';
CREATE INDEX IF NOT EXISTS idx_match_history_started_at ON match_history(started_at DESC);