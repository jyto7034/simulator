local loading_key = KEYS[1]
local current_time = tonumber(ARGV[1])
local timeout_seconds = tonumber(ARGV[2])

-- 'created_at' 필드를 가져옴
local created_at = redis.call('HGET', loading_key, 'created_at')
local status_val = redis.call('HGET', loading_key, 'status')

-- 키가 존재하지 않거나 'created_at' 필드가 없으면 아무것도 하지 않음
if not created_at then
    return {}
end

-- 타임아웃이 지났는지 확인
if current_time > tonumber(created_at) + timeout_seconds then
    -- 타임아웃된 세션의 모든 정보를 가져옴
    local all_players_status = redis.call('HGETALL', loading_key)
    
    -- 세션이 이미 삭제되었을 수 있으므로 확인
    if #all_players_status == 0 then
        -- 세션이 이미 정리됨
        return {}
    end
    
    -- 세션 키를 삭제
    redis.call('DEL', loading_key)

    local game_mode = ''
    local players_to_requeue = {}
    local timed_out_count = 0

    -- 플레이어 목록을 순회하며 재입장시킬 플레이어를 찾음
    for i=1, #all_players_status, 2 do
        local key = all_players_status[i]
        local value = all_players_status[i+1]

        if key == 'game_mode' then
            game_mode = value
        elseif key ~= 'created_at' and key ~= 'status' then
            -- 모든 플레이어는 재큐 대상
            table.insert(players_to_requeue, key)
            -- 실제 타임아웃(준비가 되지 못한) 플레이어만 집계
            if value == 'loading' then
                timed_out_count = timed_out_count + 1
            end
        end
    end

    -- 상태가 'ready'라면 재큐하지 않고 키만 정리
    if status_val == 'ready' then
        -- 완료된 세션임을 명시적으로 표시: timed_out_count = 0, no players
        return { game_mode, '0' }
    else
        -- 반환 형식: { game_mode, timed_out_count, unpack(players_to_requeue) }
        local result = { game_mode, tostring(timed_out_count) }
        for i=1, #players_to_requeue do
            table.insert(result, players_to_requeue[i])
        end
        return result
    end
end

-- 타임아웃되지 않았으면 빈 테이블 반환
return {}
