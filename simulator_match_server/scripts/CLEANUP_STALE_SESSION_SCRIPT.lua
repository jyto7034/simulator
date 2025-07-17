local loading_key = KEYS[1]
local current_time = tonumber(ARGV[1])
local timeout_seconds = tonumber(ARGV[2])

-- 'created_at' 필드를 가져옴
local created_at = redis.call('HGET', loading_key, 'created_at')

-- 키가 존재하지 않거나 'created_at' 필드가 없으면 아무것도 하지 않음
if not created_at then
    return {}
end

-- 타임아웃이 지났는지 확인
if current_time > tonumber(created_at) + timeout_seconds then
    -- 타임아웃된 세션의 모든 정보를 가져옴
    local all_players_status = redis.call('HGETALL', loading_key)
    -- 세션 키를 삭제
    redis.call('DEL', loading_key)

    local game_mode = ''
    local players_to_requeue = {}

    -- 플레이어 목록을 순회하며 재입장시킬 플레이어를 찾음
    for i=1, #all_players_status, 2 do
        local key = all_players_status[i]
        local value = all_players_status[i+1]

        if key == 'game_mode' then
            game_mode = value
        elseif key ~= 'created_at' and key ~= 'status' then
            table.insert(players_to_requeue, key)
        end
    end
    
    -- game_mode를 맨 앞에 추가하여 반환
    table.insert(players_to_requeue, 1, game_mode)
    return players_to_requeue
end

-- 타임아웃되지 않았으면 빈 테이블 반환
return {}