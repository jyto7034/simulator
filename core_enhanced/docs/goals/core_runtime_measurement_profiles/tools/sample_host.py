"""Finite actual loopback host samples, persistence restart and lease retirement.

Core authority comes only from preserved public WebSocket responses. Each host
has isolated persistence; OS/process requested-byte scopes are kept separate.
"""
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import sys
import time
import urllib.request
from local_websocket import WebSocket
from run import ROOT, HERE, identity, sha

label, inputs_label = sys.argv[1:]
assert label.isdigit() and inputs_label.isdigit()
inputs = HERE / 'evidence' / ('playback_inputs_' + inputs_label)
intake = json.loads((inputs / 'INPUTS.json').read_text())
source = identity()
assert source['sha256'] == intake['source_sha256']
assert all(sha(ROOT / p) == h for p, h in intake['files'].items())
out = HERE / 'evidence' / ('host_samples_' + label)
out.mkdir()
binary = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / '.cache/d07-runtime-amount-product-target')) / 'debug/game_server'
binary_sha = sha(binary)
rows = []

def port():
    with socket.socket() as s:
        s.bind(('127.0.0.1', 0))
        return s.getsockname()[1]

def read_json(url):
    with urllib.request.urlopen(url, timeout=2) as response:
        body = response.read(4 * 1024 * 1024 + 1)
    assert len(body) <= 4 * 1024 * 1024
    return json.loads(body)

def eventually(predicate, seconds=8):
    deadline = time.monotonic() + seconds
    while True:
        value = predicate()
        if value is not None:
            return value
        assert time.monotonic() < deadline, 'bounded sample deadline'
        time.sleep(0.05)

class Client:
    def __init__(self, ws_port, player, path):
        self.ws = WebSocket(ws_port)
        self.player = player
        self.state = None
        self.battle = None
        self.messages = 0
        self.bytes = 0
        self.requests = 0
        self.stream = path.open('xb')
        self.send({'type': 'auth', 'player_id': player})
        assert self.receive()['type'] == 'authed'
        assert self.receive()['type'] == 'state_snapshot'
        self.barrier()

    def record(self, direction, message):
        encoded = json.dumps({'direction': direction, 'message': message}, separators=(',', ':')).encode() + b'\n'
        self.messages += 1
        self.bytes += len(encoded)
        assert self.messages <= 4096 and self.bytes <= 16 * 1024 * 1024
        self.stream.write(encoded)
        self.stream.flush()

    def send(self, value):
        self.record('send', value)
        self.ws.send(value)

    def receive(self):
        value = self.ws.receive()
        self.record('receive', value)
        if value['type'] == 'state_snapshot':
            self.state = value['state']
        if value['type'] == 'battle_update':
            self.battle = value
        elif value['type'] == 'battle_resync':
            self.battle = value['update']
        return value

    def barrier(self):
        self.send({'type': 'ping'})
        for _ in range(1024):
            if self.receive()['type'] == 'pong':
                return
        raise AssertionError('bounded command barrier')

    def command(self, behavior, authority=False, expected_error=None):
        self.requests += 1
        assert self.requests <= 512
        request = 'cmrp-' + str(self.requests)
        message = {'type': 'command', 'request_id': request, 'behavior': behavior}
        if authority:
            assert self.battle is not None
            message['battle_authority'] = {k: self.battle[k] for k in ['battle_uuid', 'battle_state_revision']}
        self.send(message)
        for _ in range(1024):
            value = self.receive()
            if value.get('request_id') == request and value['type'] in ['command_result', 'error']:
                break
        else:
            raise AssertionError('bounded command result')
        self.barrier()
        if expected_error is not None:
            assert value['type'] == 'error' and value['code'] == expected_error, value
        else:
            assert value['type'] == 'command_result' and value['ok'], value
        return value

    def close(self):
        self.ws.close()
        self.stream.close()

def to_map(client):
    for _ in range(128):
        state = client.state
        assert state is not None
        novel = state.get('visual_novel')
        context = state['game_state_context']
        if novel:
            choice = next((c for c in novel['choices'] if c['selectable']), None)
            action = {'type': 'select_visual_novel_choice' if choice else 'advance_visual_novel',
                      'session_id': novel['session_id'], 'current_scene_id': novel['current_scene_id']}
            if choice:
                action['choice_id'] = choice['choice_id']
            client.command(action)
        elif context['type'] == 'selecting_starter_employees':
            client.command({'type': 'select_starter_employees', 'employee_definition_ids':
                            [c['employee_definition_id'] for c in context['candidates'][:context['required_count']]]})
        elif state.get('map') and context['type'] == 'viewing_map':
            return
        else:
            raise AssertionError(('unsupported normal frontmatter', context))
    raise AssertionError('frontmatter bound')

def enter_combat(client):
    for _ in range(128):
        available = [n for n in client.state['map']['nodes'] if n['state'] == 'Available']
        node = next((n for n in available if n['category'] == 'Combat'), None)
        if node is None:
            node = next((n for n in available if n['category'] == 'Start'), None)
        assert node is not None, ('normal map has no adjacent Start/Combat', available)
        client.command({'type': 'select_map_node', 'node_id': node['id']})
        client.command({'type': 'confirm_enter_node'})
        if node['category'] == 'Combat':
            client.command({'type': 'request_battle_state'})
            assert client.battle is not None
            return
        client.command({'type': 'complete_node'})
    raise AssertionError('ordinary combat bound')

for case, profile, observed in [('default_none', 'default', False), ('default_normal', 'default', True),
                                ('dev16_normal', 'dev16', True), ('dev32_normal', 'dev32', True)]:
    folder = out / case
    (folder / 'config').mkdir(parents=True)
    ws_port, metric_port, detail_port = port(), port(), port()
    assert len({ws_port, metric_port, detail_port}) == 3
    config = (ROOT / 'game_server/config/development.toml').read_text()
    config = config.replace('"0.0.0.0"', '"127.0.0.1"').replace('port = 8081', f'port = {ws_port}')
    config = config.replace('"../content_packs"', json.dumps(str(inputs / 'official_content')))
    config = config.replace('"../content_selection.json"', json.dumps(str(inputs / 'official_selection.json')))
    for key, name in [('active_run_directory', 'active'), ('player_profile_directory', 'profile')]:
        config = config.replace(key + ' = "persistent_data"', key + ' = ' + json.dumps(str(folder / name)))
    if profile != 'default':
        config = 'runtime_safety_path = ' + json.dumps(str(inputs / (profile + '.ron'))) + '\n' + config
    with (folder / 'config/development.toml').open('x') as f:
        f.write(config)
    observation = folder / 'observation.ron'
    with observation.open('x') as f:
        f.write(f'(enabled: {str(observed).lower()}, metrics_listen: "127.0.0.1:{metric_port}", detail_listen: "127.0.0.1:{detail_port}", snapshot_ms: 100)\n')
    env = {k: v for k, v in os.environ.items() if not k.startswith('APP_') and k not in ['GAME_OBSERVABILITY_PATH', 'RUN_MODE']}
    env.update(RUN_MODE='development', GAME_OBSERVABILITY_PATH=str(observation))
    profile_hash = intake['profiles'][profile]['runtime_safety_sha256']
    hard_cap = {'default': 8, 'dev16': 16, 'dev32': 32}[profile] * 1024 * 1024
    player = '00000000-0000-4000-8000-000000000025'
    phases = []
    for phase in ['initial', 'restarted']:
        log_path = folder / (phase + '.log')
        messages_path = folder / (phase + '.messages.jsonl')
        process = None
        client = None
        details = []
        try:
            with log_path.open('xb') as log:
                process = subprocess.Popen([str(binary)], cwd=folder, env=env, stdout=log, stderr=subprocess.STDOUT)
                deadline = time.monotonic() + 40
                while True:
                    assert process.poll() is None, log_path.read_text()[-2000:]
                    try:
                        with urllib.request.urlopen(f'http://127.0.0.1:{ws_port}/ready', timeout=1) as response:
                            assert response.read() == b'READY'
                        break
                    except OSError:
                        assert time.monotonic() < deadline
                        time.sleep(0.1)
                client = Client(ws_port, player, messages_path)
                assert client.state['game_state_context']['type'] == 'not_started'
                if phase == 'initial':
                    client.command({'type': 'start_new_game', 'game_mode': 'Standard', 'replace_active_run': False})
                    to_map(client)
                    assert any((folder / 'active').rglob('*.json')), 'normal checkpoint file exists'
                else:
                    client.command({'type': 'continue_run'})
                    assert client.state['game_state_context']['type'] == 'viewing_map', client.state['game_state_context']
                    enter_combat(client)
                    client.command({'type': 'pause_battle'}, expected_error='missing_battle_state_authority')
                    client.command({'type': 'pause_battle'}, authority=True)
                    old_battle = client.battle['battle_uuid']
                    client.close()
                    client = Client(ws_port, player, folder / 'reconnected.messages.jsonl')
                    assert client.battle['battle_uuid'] == old_battle
                    if observed:
                        url = f'http://127.0.0.1:{detail_port}/state'
                        def active_detail():
                            value = read_json(url)
                            return value if value['sessions'] and value['raw_operations'] else None
                        detail = eventually(active_detail)
                        details.append(detail)
                        assert detail['process_heap']['instrumented'] and detail['process_heap']['live_requested_bytes'] > 0
                        assert detail['process_rss']['status'] == 'available'
                        window = detail['process_window']
                        assert window is not None and window['epoch'] % 2 == 1
                        if window['peak_requested_bytes'] is not None:
                            assert window['peak_requested_bytes'] >= window['baseline_live_requested_bytes']
                        expected_identity = detail['sessions'][0]['current']['identity']
                        assert bytes(expected_identity['effective_profile_sha256']).hex() == profile_hash
                        assert expected_identity['selected_profile_hard_bytes'] == hard_cap
                        assert expected_identity['profile_warning_bytes'] == 8 * 1024 * 1024
                        assert any(r['retained'] is not None and r['retained']['accepted_total_bytes'] > 0 for r in detail['raw_operations'])
                        assert any(r['operation'] == 'ContinueCheckpoint' and r['outcome'] == 'Accepted' and r['committed'] is not None for r in detail['raw_operations']), 'actual public host Continue commits'
                        for raw in detail['raw_operations']:
                            if raw['started_identity'] is not None:
                                assert bytes(raw['started_identity']['effective_profile_sha256']).hex() == profile_hash
                                assert raw['started_identity']['selected_profile_hard_bytes'] == hard_cap
                        client.send({'type': 'quit'})
                        client.close()
                        client = None
                        retired = eventually(lambda: (v if not v['sessions'] else None) if (v := read_json(url)) else None)
                        details.append(retired)
                        assert retired['raw_operations_received'] >= detail['raw_operations_received']
                        assert retired['raw_operations'], 'retired lease raw history retained'
                        assert len(retired['raw_operations']) <= 300 and len(retired['ring']) <= 300
                    else:
                        client.send({'type': 'quit'})
                if client is not None:
                    client.close()
                    client = None
                shutdown_signal = signal.SIGTERM if case == 'dev32_normal' and phase == 'restarted' else signal.SIGINT
                process.send_signal(shutdown_signal)
                process.wait(timeout=15)
                assert process.returncode == 0
        finally:
            with (folder / (phase + '.details.json')).open('x') as f:
                json.dump(details, f, indent=2)
            if client is not None:
                client.close()
            if process is not None and process.poll() is None:
                process.send_signal(signal.SIGINT)
                try:
                    process.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
        text = log_path.read_text(errors='replace')
        assert profile_hash in text
        if shutdown_signal == signal.SIGTERM:
            assert 'Server exited unexpectedly' in text, 'actual server-completion select branch'
        windows = [json.loads(line)['process_measurement_window'] for line in text.splitlines() if line.startswith('{"process_measurement_window":')]
        assert len(windows) == 1, ('actual graceful allocator footer', text[-2000:])
        window = windows[0]
        assert window['baseline']['instrumented'] and window['end']['instrumented']
        if window['peak_requested_bytes'] is not None:
            assert window['peak_requested_bytes'] >= max(window['baseline']['live_requested_bytes'], window['end']['live_requested_bytes'])
        phases.append({'phase': phase, 'command': [str(binary)], 'cwd': str(folder), 'exit_code': process.returncode, 'shutdown_signal': shutdown_signal.name,
                       'log_sha256': sha(log_path), 'messages_sha256': sha(messages_path), 'details': details,
                       'process_window': window})
        print(json.dumps({'case': case, 'phase': phase, 'exit_code': process.returncode, 'profile': profile_hash,
                          'heap_baseline': window['baseline']['live_requested_bytes'], 'heap_end': window['end']['live_requested_bytes'],
                          'heap_peak': window['peak_requested_bytes']}), flush=True)
    rows.append({'case': case, 'observation': observed, 'effective_profile_sha256': profile_hash,
                 'selected_hard_bytes': hard_cap, 'config_sha256': sha(folder / 'config/development.toml'),
                 'observation_sha256': sha(observation), 'phases': phases})
assert source == identity() and sha(binary) == binary_sha
assert all(sha(ROOT / p) == h for p, h in intake['files'].items())
with (out / 'SUMMARY.json').open('x') as f:
    json.dump({'passed': True, 'source_sha256': source['sha256'], 'binary_sha256': binary_sha,
               'input_manifest_sha256': sha(inputs / 'INPUTS.json'), 'client_source_sha256': sha(Path(__file__).with_name('local_websocket.py')),
               'platform': dict(zip(['sysname', 'nodename', 'release', 'version', 'machine'], os.uname())), 'rows': rows,
               'scope': 'Actual loopback server/actor/WebSocket/bootstrap and disk checkpoint Continue on fresh applications. Host-derived run seeds are read from session identity, not claimed identical across processes. Native, requested allocator, RSS and session retirement scopes remain distinct.'}, f, indent=2)
print('PASS actual host bootstrap/restart/Continue/public combat/profile identity/lease retirement')
