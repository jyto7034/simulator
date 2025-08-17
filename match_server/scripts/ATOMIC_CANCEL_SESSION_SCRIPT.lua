local loading_key = KEYS[1]
local disconnected_player_id = ARGV[1]

-- 세션이 존재하는지 확인
if redis.call('EXISTS', loading_key) == 0 then
    return {} -- 이미 처리되었거나 존재하지 않음
end

-- 세션 정보를 가져옴
local all_players_status = redis.call('HGETALL', loading_key)

-- 세션 키를 삭제하여 다른 프로세스의 개입을 막음
redis.call('DEL', loading_key)

local game_mode = ''
local players_to_requeue = {}

-- 플레이어 목록을 순회하며 재입장시킬 플레이어를 찾음
for i=1, #all_players_status, 2 do
    local key = all_players_status[i]
    local value = all_players_status[i+1]

    if key == 'game_mode' then
        game_mode = value
    elseif key ~= 'created_at' and key ~= 'status' and key ~= disconnected_player_id then
        table.insert(players_to_requeue, key)
    end
end

-- game_mode를 맨 앞에 추가하여 반환
table.insert(players_to_requeue, 1, game_mode)
return players_to_requeue