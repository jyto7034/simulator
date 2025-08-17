-- 데이터베이스에 uuid-ossp 확장 기능 활성화 (여전히 다른 테이블에서 UUID를 사용할 수 있으므로 유지)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- updated_at 컬럼 자동 갱신을 위한 트리거 함수
CREATE OR REPLACE FUNCTION update_modified_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- =================================================================
-- 0. 커스텀 ENUM 타입 정의 (Custom ENUM Types)
-- =================================================================
DO $$ BEGIN
    CREATE TYPE player_status AS ENUM ('active', 'suspended', 'banned'); -- [수정됨] pending_verification 제거
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

DO $$ BEGIN
    CREATE TYPE card_rarity AS ENUM ('common', 'rare', 'epic', 'legendary');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;


-- =================================================================
-- 1. 플레이어 계정 (Steam Player Account Mapping)
-- [수정됨] 스팀 연동에 맞게 테이블 구조 대폭 변경
-- =================================================================
CREATE TABLE IF NOT EXISTS players (
    id BIGINT PRIMARY KEY, -- SteamID64를 기본 키로 사용
    last_known_username VARCHAR(64), -- 스팀 닉네임 스냅샷 (선택 사항)
    status player_status NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMPTZ
);
COMMENT ON TABLE players IS '스팀 플레이어 계정과 게임 데이터를 연결하는 핵심 테이블';
COMMENT ON COLUMN players.id IS '고유 플레이어 ID (SteamID64)';
COMMENT ON COLUMN players.last_known_username IS '마지막으로 알려진 플레이어의 스팀 닉네임';
COMMENT ON COLUMN players.status IS '계정 상태 (ENUM: active, suspended, banned)';
CREATE TRIGGER trigger_players_updated_at
BEFORE UPDATE ON players
FOR EACH ROW
EXECUTE FUNCTION update_modified_column();


-- =================================================================
-- 2. 티어 정보 (Master Data - 변경 없음)
-- =================================================================
CREATE TABLE IF NOT EXISTS tiers (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    order_key INT UNIQUE NOT NULL,
    icon_url TEXT
);
COMMENT ON TABLE tiers IS '게임의 티어(등급) 정의 마스터 테이블';
-- 예시 티어 데이터 삽입
INSERT INTO tiers (id, name, order_key, icon_url) VALUES
(1, 'Bronze', 1, '/icons/tiers/bronze.png'),
(2, 'Silver', 2, '/icons/tiers/silver.png'),
(3, 'Gold', 3, '/icons/tiers/gold.png'),
(4, 'Platinum', 4, '/icons/tiers/platinum.png'),
(5, 'Diamond', 5, '/icons/tiers/diamond.png')
ON CONFLICT (id) DO NOTHING;


-- =================================================================
-- 3. 플레이어 프로필 (Game-specific Profile & Stats)
-- [수정됨] 스팀 연동에 맞게 컬럼 축소 및 FK 변경
-- =================================================================
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
    -- 게임 고유의 프로필 꾸미기 아이템이 있다면 아래 컬럼들을 사용
    -- custom_profile_icon_id VARCHAR(100),
    -- custom_profile_banner_id VARCHAR(100),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
COMMENT ON TABLE player_profiles IS '게임 내 플레이어 프로필, 랭크, 스탯 정보';
COMMENT ON COLUMN player_profiles.player_id IS '플레이어의 SteamID64';
CREATE INDEX IF NOT EXISTS idx_player_profiles_mmr ON player_profiles(mmr);
CREATE TRIGGER trigger_player_profiles_updated_at
BEFORE UPDATE ON player_profiles
FOR EACH ROW
EXECUTE FUNCTION update_modified_column();


-- =================================================================
-- 4. 게임 모드 (Master Data - 변경 없음)
-- =================================================================
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
COMMENT ON TABLE game_modes IS '게임 모드 정의 마스터 테이블';
INSERT INTO game_modes (id, internal_name, display_name, is_ranked, player_count_per_team, team_count) VALUES
(1, 'ranked_1v1', '1v1 랭크 게임', TRUE, 1, 2),
(2, 'unranked_1v1', '1v1 일반 게임', FALSE, 1, 2)
ON CONFLICT (id) DO NOTHING;


-- =================================================================
-- 5. 게임 기록 (Match History - 변경 없음)
-- =================================================================
CREATE TABLE IF NOT EXISTS match_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    game_mode_id INT NOT NULL REFERENCES game_modes(id),
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ NOT NULL,
    duration_seconds INT GENERATED ALWAYS AS (EXTRACT(EPOCH FROM (ended_at - started_at))::INT) STORED,
    winning_team_id INT,
    additional_data JSONB
);
COMMENT ON TABLE match_history IS '완료된 모든 게임의 기록';
CREATE INDEX IF NOT EXISTS idx_match_history_started_at ON match_history(started_at DESC);


-- =================================================================
-- 6. 매치 참여자 정보 (Match Participants)
-- [수정됨] player_id의 FK 변경
-- =================================================================
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
COMMENT ON TABLE match_participants IS '각 매치의 참여자 및 성과 정보';
CREATE INDEX IF NOT EXISTS idx_match_participants_player_id ON match_participants(player_id);


-- =================================================================
-- 7. 친구 관계 (Friendships)
-- [제거됨] 스팀 친구 시스템을 사용하므로 테이블 전체를 제거합니다.
-- =================================================================


-- =================================================================
-- 8. 카드 마스터 테이블 (Card Master Data - 변경 없음)
-- =================================================================
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
COMMENT ON TABLE cards IS '게임 내 모든 카드 정보 마스터 테이블';
-- 예시 카드 데이터 삽입
INSERT INTO cards (id, internal_name, display_name, rarity, mana_cost, attack, health, card_type, is_collectible) VALUES
(1, 'fireball', 'Fireball', 'common', 4, 6, 0, 'spell', TRUE),
(2, 'frost_golem', 'Frost Golem', 'rare', 5, 4, 5, 'minion', TRUE)
ON CONFLICT (id) DO NOTHING;


-- =================================================================
-- 9. 플레이어 소유 카드 (Player Card Collection)
-- [수정됨] player_id의 FK 변경
-- =================================================================
CREATE TABLE IF NOT EXISTS player_card_collection (
    player_id BIGINT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    card_id INT NOT NULL REFERENCES cards(id) ON DELETE RESTRICT,
    quantity INT NOT NULL DEFAULT 1 CHECK (quantity > 0),
    is_new BOOLEAN NOT NULL DEFAULT TRUE,
    PRIMARY KEY (player_id, card_id)
);
COMMENT ON TABLE player_card_collection IS '플레이어가 소유한 카드 목록';


-- =================================================================
-- 10. 플레이어 덱 (Player Decks)
-- [수정됨] player_id의 FK 변경
-- =================================================================
CREATE TABLE IF NOT EXISTS player_decks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    player_id BIGINT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    deck_name VARCHAR(100) NOT NULL,
    cover_card_id INT REFERENCES cards(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (player_id, deck_name)
);
COMMENT ON TABLE player_decks IS '플레이어가 생성한 덱 목록';
CREATE TRIGGER trigger_player_decks_updated_at
BEFORE UPDATE ON player_decks
FOR EACH ROW
EXECUTE FUNCTION update_modified_column();


-- =================================================================
-- 11. 덱 구성 카드 (Deck Card Entries - 변경 없음)
-- =================================================================
CREATE TABLE IF NOT EXISTS deck_cards (
    deck_id UUID NOT NULL REFERENCES player_decks(id) ON DELETE CASCADE,
    card_id INT NOT NULL REFERENCES cards(id) ON DELETE RESTRICT,
    quantity INT NOT NULL CHECK (quantity > 0 AND quantity <= 2),
    PRIMARY KEY (deck_id, card_id)
);
COMMENT ON TABLE deck_cards IS '각 덱에 포함된 카드 정보';


-- =================================================================
-- 스크립트 실행 완료
-- =================================================================
SELECT 'Database schema setup for Steam integration finished successfully.';