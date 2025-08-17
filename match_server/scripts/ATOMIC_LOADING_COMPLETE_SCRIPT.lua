    local loading_key = KEYS[1]
    local player_id = ARGV[1]

    -- Stop if session does not exist or is already handled
    if redis.call('EXISTS', loading_key) == 0 then
        return {}
    end
    -- 추가: 세션 상태가 'loading'이 아니면 (예: 'cancelled') 중단
    local status = redis.call('HGET', loading_key, 'status')
    if status and status ~= 'loading' then
        return {}
    end

    redis.call('HSET', loading_key, player_id, 'ready')

    local players = redis.call('HGETALL', loading_key)
    local all_ready = true
    local player_ids = {}
    local game_mode = ''
    for i=1, #players, 2 do
        if players[i] == 'game_mode' then
            game_mode = players[i+1]
        elseif players[i] ~= 'created_at' and players[i] ~= 'status' then
            if players[i+1] ~= 'ready' then
                all_ready = false
                break
            end
            table.insert(player_ids, players[i])
        end
    end

    if all_ready and #player_ids > 0 then
        -- 모든 플레이어가 준비 완료: 키를 삭제하지 않고 상태만 'ready'로 전환하여 재시도/감시가 가능하도록 유지
        redis.call('HSET', loading_key, 'status', 'ready')
        -- game_mode를 맨 앞에 추가하여 반환 (호환성 유지)
        table.insert(player_ids, 1, game_mode)
        return player_ids
    else
        return {}
    end