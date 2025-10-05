-- KEYS[1] = queue:{mode} (Sorted Set)
-- ARGV[1] = player_id
-- ARGV[2] = timestamp (score)
-- ARGV[3] = metadata JSON string

local queue_key = KEYS[1]
local player_id = ARGV[1]
local timestamp = tonumber(ARGV[2])
local metadata_json = ARGV[3]

-- 유효성 검사
if timestamp == nil or metadata_json == nil or metadata_json == "" then
    local size = redis.call('ZCARD', queue_key)
    return {0, size}
end

-- 이미 큐에 있는지 확인
local exists = redis.call('ZSCORE', queue_key, player_id)
if exists then
    local size = redis.call('ZCARD', queue_key)
    return {0, size}
end

-- queue에 추가 (Sorted Set)
redis.call('ZADD', queue_key, timestamp, player_id)

-- metadata 저장 (JSON 문자열 그대로 저장)
local metadata_key = 'metadata:' .. player_id
redis.call('SET', metadata_key, metadata_json)

-- 현재 큐 크기 반환
local size = redis.call('ZCARD', queue_key)
return {1, size}
