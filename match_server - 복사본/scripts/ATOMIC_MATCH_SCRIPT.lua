local queue_key = KEYS[1]
local required_players = tonumber(ARGV[1])
local loading_session_id = ARGV[2]
local current_timestamp = ARGV[3]
local loading_session_timeout_seconds = tonumber(ARGV[4])

-- Extract game_mode from queue_key (e.g., "queue:game_mode_id" -> "game_mode_id")
local game_mode_start_index = string.find(queue_key, ":")
local game_mode = string.sub(queue_key, game_mode_start_index + 1)

if redis.call('SCARD', queue_key) >= required_players then
    local player_ids = redis.call('SPOP', queue_key, required_players)
    if #player_ids == required_players then
        local loading_key = "loading:" .. loading_session_id
        local hset_args = {loading_key, "game_mode", game_mode, "created_at", current_timestamp, "status", "loading"}
        for i=1, #player_ids do
            table.insert(hset_args, player_ids[i])
            table.insert(hset_args, "loading")
        end
        redis.call('HMSET', unpack(hset_args))
        -- TTL을 정리 주기보다 길게 설정하여 정리 루프가 세션을 관측할 수 있도록 여유를 둠
        redis.call('EXPIRE', loading_key, loading_session_timeout_seconds * 2)

        -- Return game_mode, loading_session_id, and player_ids
        local result = {game_mode, loading_session_id}
        for i=1, #player_ids do
            table.insert(result, player_ids[i])
        end
        return result
    else
        -- Not enough players popped, re-add them to the queue (this should ideally not happen if SCARD check is accurate)
        if #player_ids > 0 then
            redis.call('SADD', queue_key, unpack(player_ids))
        end
        return {}
    end
else
    return {}
end
