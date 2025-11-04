-- KEYS[1] = queue:{mode} (Sorted Set)
-- ARGV[1] = player_id

local queue_key = KEYS[1]
local player_id = ARGV[1]

-- metadata를 먼저 가져옴 (test event 발행용)
local metadata_key = 'metadata:' .. player_id
local metadata = redis.call('GET', metadata_key)

-- queue에서 제거
local removed = redis.call('ZREM', queue_key, player_id)

-- metadata 삭제
if removed == 1 then
    redis.call('DEL', metadata_key)
end

-- 현재 큐 크기 반환
local size = redis.call('ZCARD', queue_key)

-- metadata가 없으면 빈 문자열 반환
if not metadata then
    metadata = ''
end

return {removed, size, metadata}
