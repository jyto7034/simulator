-- Match participants table for player performance in matches
CREATE TABLE IF NOT EXISTS match_participants (
    match_id UUID NOT NULL REFERENCES match_history(id) ON DELETE CASCADE,
    player_id BIGINT NOT NULL REFERENCES players(id) ON DELETE SET NULL,
    team_id INT NOT NULL,
    is_winner BOOLEAN NOT NULL,
    initial_mmr DOUBLE PRECISION NOT NULL,
    final_mmr DOUBLE PRECISION NOT NULL,
    mmr_change DOUBLE PRECISION GENERATED ALWAYS AS (final_mmr - initial_mmr) STORED,
    score INT,
    stats JSONB,
    disconnected BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (match_id, player_id)
);
COMMENT ON TABLE match_participants IS 'Match participant and performance information';
CREATE INDEX IF NOT EXISTS idx_match_participants_player_id ON match_participants(player_id);