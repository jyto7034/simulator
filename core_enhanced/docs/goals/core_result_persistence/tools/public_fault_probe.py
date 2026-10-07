"""Real release websocket fault/reconnect/quit/Continue driver on isolated local files."""
import base64, hashlib, json, os, shutil, signal, socket, struct, subprocess, sys, time, urllib.request
from pathlib import Path
from initialize import ROOT, HERE
BINS = ROOT / ".cache/d07-runtime-amount-product-target/release"
label=sys.argv[1]
OUT=HERE/("evidence/public_fault_"+label)
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


messages=[]
def record(ws):
    row=ws.receive(); messages.append(row); return row
def auth(port,player):
    ws=WebSocket(port); ws.send({"type":"auth","player_id":player})
    assert record(ws)["type"]=="authed"
    row=record(ws); assert row["type"]=="state_snapshot"
    return ws,row["state"]
sequence=0
def command(ws,kind,admin=False,**fields):
    global sequence
    sequence+=1; rid=str(sequence)
    ws.send({"type":"admin_command" if admin else "command","request_id":rid,"admin" if admin else "behavior":{"type":kind,**fields}})
    result=None
    while True:
        row=record(ws)
        if row.get("request_id")==rid:
            if row["type"]=="error": return None,row
            assert row["type"] in ("command_result","admin_result"); result=row
        if result and row["type"]=="state_snapshot": return row["state"],result
        if result and row["type"]=="battle_setup_snapshot": return row,result

def files():
    return {str(p.relative_to(OUT)):hashlib.sha256(p.read_bytes()).hexdigest() for name in ("active","profile") for p in (OUT/name).rglob("*") if p.is_file()}
original_packs=ROOT/"core_enhanced/docs/goals/core_runtime_safety_observability/evidence/public_profile_002/content"
selection=original_packs.parent/"selection.json"
assert original_packs.exists()
# Admin SavePoint consumes the opening corridor. Its next authored room can use
# a lower-work defense encounter, which is not guaranteed to cross this test's
# native cap. Bind the isolated next-room input to the original high-work patrol
# so the scheduled tick fault is reproducible; production/old fixture bytes stay.
packs=OUT/"content"
shutil.copytree(original_packs,packs)
assignments=packs/"official/data/map/node_assignment_profiles.ron"
text=assignments.read_text()
assert "normal_black_box_pressure" in text
assignments.write_text(text.replace("normal_black_box_pressure","normal_corroded_patrol"))
profile=OUT/"tiny.ron"
profile.write_text((ROOT/"config/runtime_safety.ron").read_text().replace("max_work: 10000000", "max_work: "+(sys.argv[2] if len(sys.argv)>2 else "1000"),1))
with socket.socket() as sock:
    sock.bind(("127.0.0.1",0)); port=sock.getsockname()[1]
config=(ROOT/"game_server/config/development.toml").read_text().replace('"0.0.0.0"','"127.0.0.1"').replace('port = 8081',f'port = {port}')
config=config.replace('"../content_packs"',json.dumps(str(packs))).replace('"../content_selection.json"',json.dumps(str(selection)))
config=config.replace('active_run_directory = "persistent_data"','active_run_directory = '+json.dumps(str(OUT/"active"))).replace('player_profile_directory = "persistent_data"','player_profile_directory = '+json.dumps(str(OUT/"profile")))
config='runtime_safety_path = '+json.dumps(str(profile))+'\n'+config
(OUT/"config").mkdir(); (OUT/"config/development.toml").write_text(config)
env={k:v for k,v in os.environ.items() if not k.startswith("APP_")}
env.update(RUN_MODE="development",ENABLE_ADMIN_COMMANDS="true")
with (OUT/"server.log").open("xb") as log:
    process=subprocess.Popen([BINS/"game_server"],cwd=OUT,env=env,stdout=log,stderr=subprocess.STDOUT)
    try:
        for _ in range(300):
            assert process.poll() is None
            try:
                with urllib.request.urlopen(f"http://127.0.0.1:{port}/ready",timeout=1) as response: assert response.read()==b"READY"
                break
            except OSError: time.sleep(.1)
        else: raise AssertionError("host startup timeout")
        player="00000000-0000-4000-8000-00000000f003"
        ws,state=auth(port,player)
        state,result=command(ws,"start_new_game",game_mode="Standard",replace_active_run=True); assert state,result
        for _ in range(160):
            vn=state.get("visual_novel")
            if vn:
                fields={k:vn[k] for k in ("session_id","current_scene_id")}
                choices=[x for x in vn.get("choices",[]) if x["selectable"]]
                if choices: state,result=command(ws,"select_visual_novel_choice",choice_id=choices[0]["choice_id"],**fields)
                else: state,result=command(ws,"advance_visual_novel",**fields)
                assert state,result; continue
            context=state["game_state_context"]
            if context["type"]=="selecting_starter_employees":
                state,result=command(ws,"select_starter_employees",employee_definition_ids=[x["employee_definition_id"] for x in context["candidates"][:context["required_count"]]])
                assert state,result; continue
            nodes=state["map"]["nodes"]
            available=[n for n in nodes if n["state"]=="Available"]
            combats=[n for n in available if n["category"]=="Combat"]
            if combats:
                state,result=command(ws,"admin_enter_save_point",admin=True); assert state,result
                state,result=command(ws,"complete_node"); assert state,result
                saved_loads=state["run_checkpoint"]["loads_used"]
                combats=[n for n in state["map"]["nodes"] if n["state"]=="Available" and n["category"]=="Combat"]
                assert combats
                state,result=command(ws,"select_map_node",node_id=combats[0]["id"]); assert state,result
                before=files(); assert before
                state,result=command(ws,"confirm_enter_node"); assert state,result
                break
            starts=[n for n in available if n["category"]=="Start"]
            assert starts,(context,available)
            state,result=command(ws,"select_map_node",node_id=starts[0]["id"]); assert state,result
            state,result=command(ws,"confirm_enter_node"); assert state,result
            state,result=command(ws,"complete_node"); assert state,result
        else: raise AssertionError("frontmatter bound")
        deadline=time.monotonic()+100
        first=None
        while time.monotonic()<deadline:
            row=record(ws)
            if row["type"]=="state_snapshot" and row["state"].get("battle_fault"):
                first=row["state"]; break
        assert first,"actual scheduled tick did not fault"
        assert first["action_capabilities"]==[]
        for kind in ("pause_battle","resume_battle","request_battle_state","continue_run","reset_active_run","retry_player_profile","reset_player_profile"):
            _,error=command(ws,kind); assert error["code"]=="battle_faulted",error
        _,error=command(ws,"admin_set_enkephalin",admin=True,amount=1); assert error["code"]=="battle_faulted"
        assert files()==before
        ws.close(); time.sleep(.1)
        ws,state=auth(port,player); assert state==first
        replacement,replaced=auth(port,player); assert replaced==first
        ws.close(); ws=replacement
        assert files()==before
        ws.send({"type":"quit"}); time.sleep(.3); ws.close()
        assert files()==before
        ws,state=auth(port,player); assert not state.get("battle_fault")
        state,result=command(ws,"continue_run"); assert state,result
        assert not state.get("battle_fault") and state["run_checkpoint"]["loads_used"]==saved_loads
        ws.close()
        (OUT/"SUMMARY.json").write_text(json.dumps({"passed":True,"fault":first,"saved_file_hashes":before,"loads_used":saved_loads,"fixture_inputs":{str(p.relative_to(OUT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in packs.rglob("*") if p.is_file()},"binary_sha256":hashlib.sha256((BINS/"game_server").read_bytes()).hexdigest()},indent=2))
    finally:
        (OUT/"messages.json").write_text(json.dumps(messages,indent=2))
        if process.poll() is None:
            process.send_signal(signal.SIGINT)
            try: process.wait(timeout=10)
            except subprocess.TimeoutExpired: process.kill();process.wait()
print(json.dumps({"passed":True,"messages":len(messages)}))
