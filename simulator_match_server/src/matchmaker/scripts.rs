pub(super) const ATOMIC_MATCH_SCRIPT: &str = r#"
    local queue_key = KEYS[1]
    local required_players = tonumber(ARGV[1])
    if redis.call('SCARD', queue_key) >= required_players then
        return redis.call('SPOP', queue_key, required_players)
    else
        return {}
    end
"#;

pub(super) const ATOMIC_LOADING_COMPLETE_SCRIPT: &str = r#"
    local loading_key = KEYS[1]
    local player_id = ARGV[1]

    if redis.call('EXISTS', loading_key) == 0 then
        return {} -- Session already handled
    end

    redis.call('HSET', loading_key, player_id, 'ready')

    local players = redis.call('HGETALL', loading_key)
    local all_ready = true
    local player_ids = {}
    local game_mode = ''
    for i=1, #players, 2 do
        if players[i] == 'game_mode' then
            game_mode = players[i+1]
        elseif players[i] ~= 'created_at' then
            if players[i+1] ~= 'ready' then
                all_ready = false
                break
            end
            table.insert(player_ids, players[i])
        end
    end

    if all_ready and #player_ids > 0 then
        redis.call('DEL', loading_key)
        -- Return game_mode as the first element
        table.insert(player_ids, 1, game_mode)
        return player_ids
    else
        return {}
    end
"#;
