from release_gate import require
require('release')
"""Exercise current stock WebSocket transport, separate active/profile files and process restart.
Uses an isolated officially packaged catalog with short authored victory timers and ChooseOne.
Original production catalogs and predecessor evidence remain unchanged.
"""
import base64, hashlib, json, os, re, shutil, signal, socket, struct, subprocess, sys, time, urllib.request
from pathlib import Path
from initialize import ROOT, HERE
BINS = ROOT / ".cache/d07-runtime-amount-product-target/release"
OUT = HERE / ("evidence/public_continuation_" + sys.argv[1])
OUT.mkdir(exist_ok=False)
class WebSocket:
    """Small RFC6455 client for local, unauthenticated test service traffic."""
    def __init__(self, port):
        self.sock = socket.create_connection(("127.0.0.1", port), timeout=10)
        self.buffer = b""
        key = base64.b64encode(os.urandom(16)).decode()
        self.sock.sendall(f"GET /game HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: {key}\r\n\r\n".encode())
        while b"\r\n\r\n" not in self.buffer:
            self.buffer += self.sock.recv(4096)
        header, self.buffer = self.buffer.split(b"\r\n\r\n", 1)
        assert b"101 Switching Protocols" in header
        accept = base64.b64encode(hashlib.sha1((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest())
        assert accept in header

    def read(self, count):
        while len(self.buffer) < count:
            chunk = self.sock.recv(max(4096, count - len(self.buffer)))
            assert chunk, "unexpected websocket EOF"
            self.buffer += chunk
        result, self.buffer = self.buffer[:count], self.buffer[count:]
        return result

    def send_frame(self, opcode, body):
        mask = os.urandom(4)
        size = len(body)
        header = bytes([0x80 | opcode, 0x80 | (size if size < 126 else 126)])
        if size >= 126:
            header += struct.pack("!H", size)
        self.sock.sendall(header + mask + bytes(byte ^ mask[index % 4] for index, byte in enumerate(body)))

    def send(self, message):
        self.send_frame(1, json.dumps(message).encode())

    def receive(self):
        body = b""
        while True:
            first, second = self.read(2)
            assert not second & 0x80
            size = second & 0x7f
            if size == 126:
                size, = struct.unpack("!H", self.read(2))
            elif size == 127:
                size, = struct.unpack("!Q", self.read(8))
            assert size <= 8 * 1024 * 1024
            payload = self.read(size)
            if first & 15 == 9:
                self.send_frame(10, payload)
                continue
            assert first & 15 in (0, 1), first
            body += payload
            if first & 0x80:
                return json.loads(body)

    def close(self):
        self.sock.close()


messages = []
latest_update = None
sequence = 0
process = None
logs = []
starts = 0
player = "00000000-0000-4000-8000-00000000c004"

def record(ws):
    global latest_update
    row = ws.receive()
    messages.append(row)
    if row["type"] in ("battle_update", "snapshot_resync_required"):
        latest_update = row
    elif row["type"] == "battle_resync":
        latest_update = row["update"]
    return row

def auth(identity=player):
    global latest_update
    latest_update = None
    ws = WebSocket(port)
    ws.send({"type":"auth", "player_id":identity})
    assert record(ws)["type"] == "authed"
    row = record(ws)
    assert row["type"] == "state_snapshot", row
    return ws, row["state"]

def command(ws, kind, expect="state", authority=False, **fields):
    global sequence
    sequence += 1
    rid = str(sequence)
    request = {"type":"command", "request_id":rid, "behavior":{"type":kind, **fields}}
    if authority:
        assert latest_update
        request["battle_authority"] = {k:latest_update[k] for k in ("battle_uuid", "battle_state_revision")}
    ws.send(request)
    result = None
    while True:
        row = record(ws)
        if row.get("request_id") == rid:
            if row["type"] == "error":
                return None, row
            assert row["type"] == "command_result" and row["ok"], row
            result = row
        if result and expect == "state" and row["type"] == "state_snapshot":
            return row["state"], result
        if result and expect == "update" and row["type"] in ("battle_update", "battle_resync"):
            return latest_update, result

def require(ws, kind, **fields):
    state, result = command(ws, kind, **fields)
    assert state is not None, result
    return state

def durable():
    paths = list((OUT / "active").rglob("active_run.json"))
    assert len(paths) == 1, paths
    envelope = json.loads(paths[0].read_bytes())
    assert envelope["version"] == 5
    assert paths[0].read_bytes() == paths[0].with_name("active_run.backup.json").read_bytes()
    encoded = envelope["core_payload_base64"]
    core = json.loads(base64.b64decode(encoded)) if encoded else None
    if core:
        assert core["version"] == 11
    return envelope, core

def stop():
    global process
    if process and process.poll() is None:
        process.kill()  # Actual abrupt process exit, including before result acknowledgement.
        process.wait(timeout=10)
    process = None

def start():
    global process, starts
    starts += 1
    log = (OUT / f"server_{starts:02}.log").open("xb")
    logs.append(log)
    process = subprocess.Popen([BINS / "game_server"], cwd=OUT, env=env, stdout=log, stderr=subprocess.STDOUT)
    for _ in range(300):
        assert process.poll() is None, "server failed to start"
        try:
            with urllib.request.urlopen(f"http://127.0.0.1:{port}/ready", timeout=1) as response:
                assert response.read() == b"READY"
            return
        except OSError:
            time.sleep(.1)
    raise AssertionError("host startup timeout")

def restart_continue(ws):
    ws.close()
    stop()
    start()
    ws, state = auth()
    assert state["game_state"] == "not_started"
    assert state["active_run_continue"]["available"]
    return ws, require(ws, "continue_run")

# Package an isolated fixture through the real official pack writer and reader.
resources = OUT / "resources"
shutil.copytree(ROOT / "game_resources/data", resources / "data")
pve_path = resources / "data/pve/encounters.ron"
pve = pve_path.read_text()
terminal_count = 0
# TimeReached is authored only for the two normal Defense encounters used here.
# Preserve every other mission's terminal and reward constraints.
parts = re.split(r"(?=        PveEncounter\()", pve)
for index, part in enumerate(parts):
    if not any(f'id: "{name}"' in part for name in ("normal_corroded_patrol", "normal_breach_response")):
        continue
    part, count = re.subn(r'(verdict: victory,\s*condition:)\s*.*?,\s*(priority:)',
        r'\1 TimeReached(time_ms: 1500),\n                    \2', part, flags=re.S)
    terminal_count += count
    part = part.replace("required_for_victory: true", "required_for_victory: false")
    part = part.replace("reward_mode: ClaimAll", "reward_mode: ChooseOne")
    parts[index] = part
assert terminal_count == 2
pve = "".join(parts)
pve_path.write_text(pve)
packs = OUT / "content"
packs.mkdir()
package = subprocess.run([BINS / "package_official_content", resources, packs / "official"], cwd=ROOT, capture_output=True)
(OUT / "package.stdout").write_bytes(package.stdout)
(OUT / "package.stderr").write_bytes(package.stderr)
assert package.returncode == 0, package.stderr[-3000:]
selection = OUT / "selection.json"
selection.write_text('{"packs":["official"],"winners":[]}')
validation = subprocess.run([BINS / "author_content", "validate", packs, selection], cwd=ROOT, capture_output=True)
(OUT / "validate.stdout").write_bytes(validation.stdout)
(OUT / "validate.stderr").write_bytes(validation.stderr)
assert validation.returncode == 0, validation.stderr[-3000:]
with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    port = sock.getsockname()[1]
config = (ROOT / "game_server/config/development.toml").read_text().replace('"0.0.0.0"', '"127.0.0.1"').replace('port = 8081', f'port = {port}')
config = config.replace('"../content_packs"', json.dumps(str(packs))).replace('"../content_selection.json"', json.dumps(str(selection)))
config = config.replace('active_run_directory = "persistent_data"', 'active_run_directory = ' + json.dumps(str(OUT / "active")))
config = config.replace('player_profile_directory = "persistent_data"', 'player_profile_directory = ' + json.dumps(str(OUT / "profile")))
(OUT / "config").mkdir()
(OUT / "config/development.toml").write_text(config)
env = {k:v for k,v in os.environ.items() if not k.startswith("APP_")}
env.update(RUN_MODE="development", ENABLE_ADMIN_COMMANDS="false")
try:
    start()
    ws, state = auth()
    state = require(ws, "start_new_game", game_mode="Standard", replace_active_run=True)
    for _ in range(160):
        vn = state.get("visual_novel")
        if vn:
            fields = {k:vn[k] for k in ("session_id", "current_scene_id")}
            choices = [x for x in vn.get("choices", []) if x["selectable"]]
            if choices:
                state = require(ws, "select_visual_novel_choice", choice_id=choices[0]["choice_id"], **fields)
            else:
                state = require(ws, "advance_visual_novel", **fields)
            continue
        context = state["game_state_context"]
        if context["type"] == "selecting_starter_employees":
            state = require(ws, "select_starter_employees", employee_definition_ids=[x["employee_definition_id"] for x in context["candidates"][:context["required_count"]]])
            continue
        assert state["game_state"] == "viewing_map", state["game_state"]
        break
    else:
        raise AssertionError("frontmatter bound")
    initial_envelope, initial = durable()
    assert initial["continuation"]["savepoint"] is None
    ws, state = restart_continue(ws)
    assert durable()[1] == initial
    assert state["run_checkpoint"]["loads_used"] == 0
    for _ in range(4):
        available = [n for n in state["map"]["nodes"] if n["state"] == "Available"]
        combats = [n for n in available if n["category"] == "Combat"]
        if combats:
            node = combats[0]["id"]
            break
        starts_nodes = [n for n in available if n["category"] == "Start"]
        assert starts_nodes, available
        state = require(ws, "select_map_node", node_id=starts_nodes[0]["id"])
        state = require(ws, "confirm_enter_node")
        state = require(ws, "complete_node")
    else:
        raise AssertionError("no opening combat")
    state = require(ws, "select_map_node", node_id=node)
    preentry_envelope, preentry = durable()
    require(ws, "confirm_enter_node", expect="update")
    # Same-process reconnect retains the live battle and pauses it.
    ws.close()
    time.sleep(.1)
    ws, state = auth()
    assert state["game_state"] == "in_battle"
    while latest_update is None:
        record(ws)
    assert latest_update["checkpoint"]["playback"]["paused"]
    assert durable()[1] == preentry
    # Abrupt restart instead restores the confirmed pre-entry root.
    ws, state = restart_continue(ws)
    assert state["game_state"] == "viewing_map"
    assert state["run_checkpoint"]["loads_used"] == 0
    assert durable()[1] == preentry
    state = require(ws, "select_map_node", node_id=node)
    require(ws, "confirm_enter_node", expect="update")
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        row = record(ws)
        if row["type"] == "state_snapshot" and row["state"]["game_state"] == "combat_result":
            state = row["state"]
            break
    else:
        raise AssertionError("actual stock host tick did not settle")
    confirmed_envelope, confirmed = durable()
    assert confirmed["continuation"]["result"] is not None
    candidates = confirmed["continuation"]["reward"]["rewards"]
    assert len(candidates) >= 2
    ws, state = restart_continue(ws)
    assert state["game_state"] == "combat_result"
    assert durable()[1] == confirmed
    state = require(ws, "complete_combat_result")
    assert state["game_state"] == "in_reward"
    candidate = candidates[0]["uuid"]
    state = require(ws, "select_reward", reward_id=candidate)
    selected = durable()[1]
    assert selected["continuation"]["reward"]["selected_reward_uuid"] == candidate
    ws, state = restart_continue(ws)
    assert durable()[1] == selected
    competitor_id = "00000000-0000-4000-8000-00000000c005"
    competitor, _ = auth(competitor_id)
    competitor_state = require(competitor, "continue_run")
    assert competitor_state["game_state"] == "in_reward"
    assert durable()[1] == selected
    state = require(ws, "claim_reward")
    claimed = durable()[1]
    assert state["game_state"] == "in_reward_claimed"
    claim_files = {str(p.relative_to(OUT)):hashlib.sha256(p.read_bytes()).hexdigest()
        for folder in ("active", "profile") for p in (OUT/folder).rglob("*") if p.is_file()}
    _, stale = command(competitor, "claim_reward")
    assert stale["code"] == "active_run_revision_conflict", stale
    assert durable()[1] == claimed
    assert claim_files == {str(p.relative_to(OUT)):hashlib.sha256(p.read_bytes()).hexdigest()
        for folder in ("active", "profile") for p in (OUT/folder).rglob("*") if p.is_file()}
    competitor.close()
    competitor, competitor_after = auth(competitor_id)
    assert competitor_after["game_state"] == "in_reward"
    assert competitor_after["game_state_context"] == competitor_state["game_state_context"]
    competitor.close()
    _, refused = command(ws, "claim_reward")
    assert refused["type"] == "error"
    assert durable()[1] == claimed
    ws, state = restart_continue(ws)
    assert state["game_state"] == "in_reward_claimed"
    assert durable()[1] == claimed
    state = require(ws, "exit_reward")
    assert state["game_state"] == "viewing_map"
    assert next(n for n in state["map"]["nodes"] if n["id"] == node)["state"] == "Completed"
    state = require(ws, "abandon_run")
    final_envelope, final = durable()
    assert final is None
    assert final_envelope["revision"] != confirmed_envelope["revision"]
    ws.close(); stop(); start()
    ws, state = auth()
    assert not state["active_run_continue"]["available"]
    ws.close()
    summary = {"passed": True, "process_starts":starts, "terminal_fixture_count":terminal_count,
        "messages":len(messages), "initial_revision":initial_envelope["revision"], "terminal_revision":final_envelope["revision"],
        "before_first_savepoint":True, "reconnect_pauses":True, "unfinished_restart_restores_preentry":True,
        "actual_host_tick_result_before_ack":True, "choice_selection_and_claim_restart":True,
        "duplicate_claim_refused":True, "stale_actor_claim_refused_without_mutation":True, "abandon_tombstone":True,
        "binary_sha256":hashlib.sha256((BINS / "game_server").read_bytes()).hexdigest()}
    (OUT / "SUMMARY.json").write_text(json.dumps(summary, indent=2))
    print(json.dumps(summary))
finally:
    stop()
    for log in logs:
        log.close()
    (OUT / "messages.json").write_text(json.dumps(messages, indent=2))
