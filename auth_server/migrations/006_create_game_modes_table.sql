-- Game modes master data table
CREATE TABLE IF NOT EXISTS game_modes (
    id SERIAL PRIMARY KEY,
    internal_name VARCHAR(50) UNIQUE NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    description TEXT,
    is_ranked BOOLEAN NOT NULL DEFAULT FALSE,
    player_count_per_team INT NOT NULL DEFAULT 1,
    team_count INT NOT NULL DEFAULT 2,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);
COMMENT ON TABLE game_modes IS 'Game mode definition master table';
INSERT INTO game_modes (id, internal_name, display_name, is_ranked, player_count_per_team, team_count) VALUES
(1, 'ranked_1v1', '1v1 Ranked Game', TRUE, 1, 2),
(2, 'unranked_1v1', '1v1 Casual Game', FALSE, 1, 2)
ON CONFLICT (id) DO NOTHING;