-- KEYS[1] = queue:{mode} (Sorted Set)
-- ARGV[1] = batch_size (integer)

local queue_key = KEYS[1]
local batch_size = tonumber(ARGV[1])

-- 유효성 검사
if batch_size == nil or batch_size <= 0 then
    return {}
end

-- ZPOPMIN으로 원자적으로 pop (FIFO 보장)
local popped = redis.call('ZPOPMIN', queue_key, batch_size)

if #popped == 0 then
    return {}
end

local result = {}

-- popped format: [player_id, score, player_id, score, ...]
for idx = 1, #popped, 2 do
    local player_id = popped[idx]
    local score = popped[idx + 1]

    -- metadata 가져오기 (JSON 문자열 그대로)
    local metadata_key = 'metadata:' .. player_id
    local metadata_json = redis.call('GET', metadata_key)

    -- metadata가 없으면 빈 객체
    if not metadata_json then
        metadata_json = "{}"
    end

    -- 결과에 추가: [player_id, score, metadata_json, ...]
    table.insert(result, player_id)
    table.insert(result, score)
    table.insert(result, metadata_json)

    -- metadata 삭제 (이미 pop했으므로)
    redis.call('DEL', metadata_key)
end

return result
