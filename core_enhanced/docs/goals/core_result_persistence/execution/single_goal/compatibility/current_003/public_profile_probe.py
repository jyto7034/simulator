"""Actual release CLI/profile/host probes; no writes to predecessor evidence."""
import base64
import hashlib
import json
import os
from pathlib import Path
import signal
import socket
import struct
import subprocess
import sys
import time
import urllib.request

import sys
sys.path.insert(0, '/mnt/f/work/simulator/core_enhanced/docs/goals/core_result_persistence/tools')
from initialize import ROOT
HERE = Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_result_persistence/execution/single_goal/compatibility/current_003')

BINS = ROOT / ".cache/d07-runtime-amount-product-target/release"
label = sys.argv[1] if len(sys.argv) > 1 else "001"
assert label.isdigit()
OUT = HERE / ("evidence/public_profile_" + label)
OUT.mkdir(exist_ok=False)
COMMANDS = []


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(name, args, expected=0):
    result = subprocess.run(list(map(str, args)), cwd=ROOT, capture_output=True)
    (OUT / (name + ".stdout")).write_bytes(result.stdout)
    (OUT / (name + ".stderr")).write_bytes(result.stderr)
    COMMANDS.append(dict(name=name, command=list(map(str, args)), exit_code=result.returncode,
                         stdout_sha256=sha(OUT / (name + ".stdout")), stderr_sha256=sha(OUT / (name + ".stderr"))))
    assert result.returncode == expected, (name, result.returncode, result.stderr[-2000:])
    return result


def normalize(value):
    if isinstance(value, dict):
        out = {key: normalize(item) for key, item in value.items()}
        if {"employees", "physical_damage_unchanged", "replaces_run_checkpoint"} <= out.keys():
            out["employees"].sort(key=lambda row: row["employee_uuid"])
        return out
    if isinstance(value, list):
        return [normalize(item) for item in value]
    return value


def documents(result):
    return [normalize(json.loads(line)) for line in result.stdout.splitlines() if line.strip()]


def hashes(value):
    if isinstance(value, dict):
        return ([value["runtime_safety_sha256"]] if "runtime_safety_sha256" in value else []) + [h for item in value.values() for h in hashes(item)]
    if isinstance(value, list):
        return [h for item in value for h in hashes(item)]
    return []


raw = (ROOT / "config/runtime_safety.ron").read_text()
default = OUT / "default.ron"
default.write_text(raw)
comments = OUT / "comments.ron"
comments.write_text("// same effective profile\n" + raw + "\n")
small = OUT / "small.ron"
small.write_text(raw.replace("max_events: 8192", "max_events: 64"))
packs = OUT / "content"
packs.mkdir()
run("package", [BINS / "package_official_content", ROOT / "game_resources", packs / "official"])
selection = OUT / "selection.json"
selection.write_text('{"packs":["official"],"winners":[]}')
binary = BINS / "author_content"
original = documents(run("official_default", [binary, "validate", packs, selection]))
default_hash, = hashes(original)
selected = documents(run("official_small", [binary, "validate", packs, selection, "--runtime-safety", small]))
small_hash, = hashes(selected)
assert default_hash != small_hash
assert original[0]["identity"] == selected[0]["identity"]
assert original[0]["source_receipt"] == selected[0]["source_receipt"]

# The predecessor scenario is unchanged and already exercises real public APIs.
fixture = ROOT / "core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/evidence/cli_predecessor_current_001/r06"
explain = OUT / "explain.json"
explain.write_text(json.dumps({"query": "definition", "key": {"kind": "Skill", "id": "fragment_freischutz_black_round"}}))
for operation in ("validate", "describe", "explain", "simulate"):
    args = [binary, operation, fixture / "content", fixture / "selection.json"]
    if operation == "explain":
        args.append(explain)
    if operation == "simulate":
        args.extend([fixture / "scenario.json", "--amount-observation"])
    implicit = documents(run(operation + "_implicit", args))
    explicit = documents(run(operation + "_explicit", args + ["--runtime-safety", default]))
    commented = documents(run(operation + "_comments", args + ["--runtime-safety", comments]))
    assert implicit == explicit == commented, operation
    if operation != "explain":
        assert hashes(implicit) and set(hashes(implicit)) == {default_hash}, operation
    if operation == "simulate":
        traced = documents(run("simulate_profile_trace_amount", args + ["--runtime-safety", default, "--trace-output", OUT / "trace.jsonl"]))
        # Trace bookkeeping has its own metadata peak. Keep every gameplay,
        # amount and receipt payload field exact while excluding usage totals.
        assert [{k: v for k, v in row.items() if k != "usage"} for row in traced] == [
            {k: v for k, v in row.items() if k != "usage"} for row in implicit]
        assert (OUT / "trace.jsonl").stat().st_size > 0

for name, contents, field in (
    ("unknown", raw.replace("max_events: 8192", "surprise: 8192"), "battle_history.surprise"),
    ("duplicate", raw.replace("schema_version: 1,", "schema_version: 1, schema_version: 1,"), "schema_version"),
    ("zero", raw.replace("max_events: 8192", "max_events: 0"), "battle_history.max_events"),
    ("oversize", " " * 65537, None),
):
    path = OUT / (name + ".ron")
    path.write_text(contents)
    result = run("invalid_" + name, [binary, "validate", OUT / "absent-root", OUT / "absent-selection", "--runtime-safety", path], 2)
    assert str(path).encode() in result.stderr
    if field:
        assert field.encode() in result.stderr
    assert not result.stdout
run("missing_profile", [binary, "validate", packs, selection, "--runtime-safety", OUT / "missing.ron"], 2)
run("duplicate_option", [binary, "validate", packs, selection, "--runtime-safety", default, "--runtime-safety", default], 2)
tiny_trial = OUT / "tiny_trial.ron"
tiny_trial.write_text(raw.replace("max_total_work: 1073741824", "max_total_work: 1"))
result = run("trial_work_limit", [binary, "simulate", fixture / "content", fixture / "selection.json", fixture / "scenario.json", "--runtime-safety", tiny_trial], 1)
assert b"work" in result.stderr
tiny_receipt = OUT / "tiny_receipt.ron"
tiny_receipt.write_text(raw.replace("max_receipt_bytes: 67108864", "max_receipt_bytes: 1"))
result = run("trial_receipt_limit", [binary, "simulate", fixture / "content", fixture / "selection.json", fixture / "scenario.json", "--runtime-safety", tiny_receipt], 1)
assert b"receipt" in result.stderr


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


host_rows = []
for name, profile, expected_hash in (("implicit", None, default_hash), ("explicit", default, default_hash), ("selected", small, small_hash), ("environment", None, small_hash), ("invalid", OUT / "zero.ron", None), ("invalid_observation", None, default_hash)):
    host = OUT / ("host_" + name)
    (host / "config").mkdir(parents=True)
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        port = reservation.getsockname()[1]
    config = (ROOT / "game_server/config/development.toml").read_text().replace('"0.0.0.0"', '"127.0.0.1"').replace('port = 8081', f'port = {port}')
    config = config.replace('"../content_packs"', json.dumps(str(packs))).replace('"../content_selection.json"', json.dumps(str(selection)))
    config = config.replace('active_run_directory = "persistent_data"', 'active_run_directory = ' + json.dumps(str(host / "active")))
    config = config.replace('player_profile_directory = "persistent_data"', 'player_profile_directory = ' + json.dumps(str(host / "profile")))
    if profile:
        config = 'runtime_safety_path = ' + json.dumps(str(profile)) + '\n' + config
    (host / "config/development.toml").write_text(config)
    env = {key: value for key, value in os.environ.items() if not key.startswith("APP_")}
    env["RUN_MODE"] = "development"
    if name == "environment":
        env["APP__RUNTIME_SAFETY_PATH"] = str(small)
    if name == "invalid_observation":
        observation_path = host / "invalid_observation.ron"
        observation_path.write_text('(enabled: true, snapshot_ms: 0)')
        env["GAME_OBSERVABILITY_PATH"] = str(observation_path)
    with (host / "process.log").open("xb") as log:
        process = subprocess.Popen([BINS / "game_server"], cwd=host, env=env, stdout=log, stderr=subprocess.STDOUT)
        messages = []
        try:
            if expected_hash is None:
                assert process.wait(timeout=30) != 0
                assert not (host / "active").exists() and not (host / "profile").exists()
                assert b"battle_history.max_events" in (host / "process.log").read_bytes()
            else:
                deadline = time.monotonic() + 30
                while True:
                    assert process.poll() is None, (name, (host / "process.log").read_text())
                    try:
                        with urllib.request.urlopen(f"http://127.0.0.1:{port}/ready", timeout=1) as response:
                            assert response.read() == b"READY"
                        break
                    except OSError:
                        assert time.monotonic() < deadline
                        time.sleep(0.1)
                ws = WebSocket(port)
                player = "00000000-0000-4000-8000-000000000025"
                ws.send({"type": "auth", "player_id": player})
                authed, snapshot = ws.receive(), ws.receive()
                assert authed["type"] == "authed" and snapshot["type"] == "state_snapshot"
                messages.extend([authed, snapshot])
                ws.send({"type": "command", "request_id": "profile-bootstrap-command", "behavior": {"type": "start_new_game", "game_mode": "Standard", "replace_active_run": True}})
                while True:
                    reply = ws.receive()
                    messages.append(reply)
                    assert reply["type"] != "error", reply
                    if reply["type"] == "command_result":
                        assert reply["request_id"] == "profile-bootstrap-command" and reply["ok"]
                        break
                ws.close()
                ws = WebSocket(port)
                ws.send({"type": "auth", "player_id": player})
                reauthed, restored = ws.receive(), ws.receive()
                assert reauthed["type"] == "authed" and restored["type"] == "state_snapshot"
                messages.extend([reauthed, restored])
                ws.close()
        finally:
            if process.poll() is None:
                process.send_signal(signal.SIGINT)
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
        (host / "messages.json").write_text(json.dumps(messages, indent=2))
    if expected_hash:
        assert expected_hash in (host / "process.log").read_text()
    if name == "invalid_observation":
        assert "snapshot_ms must be" in (host / "process.log").read_text()
    host_rows.append(dict(name=name, config_sha256=sha(host / "config/development.toml"), process_log_sha256=sha(host / "process.log"), message_count=len(messages), expected_profile=expected_hash, exit_code=process.returncode))

summary = dict(commands=COMMANDS, default_hash=default_hash, small_hash=small_hash,
               binary_sha256={name: sha(BINS / name) for name in ("author_content", "package_official_content", "game_server")},
               input_files={str(path.relative_to(ROOT)): sha(path) for path in fixture.rglob("*") if path.is_file()},
               hosts=host_rows, passed=True)
(OUT / "SUMMARY.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps(dict(commands=len(COMMANDS), hosts=len(host_rows), passed=True)))
