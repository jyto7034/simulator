import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

C = Path(__file__).resolve().parent
D = Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_battle_starter_tactics')
R = D.parents[3]
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
label = sys.argv[1] if len(sys.argv) > 1 else '001'
out = C / ('public_probe_' + label)
out.mkdir(exist_ok=False)
old = json.loads((Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_runtime_safety_observability/post_audit/s1_20261004_001/ROLLBACK_PROBE.json')).read_text())
build_command = ['cargo', 'build', '-p', 'game_core', '-p', 'game_server', '--release', '--bins', '--offline', '--locked', '--features', 'game_server/runtime-observation', '--message-format=json']
build = subprocess.run(build_command, cwd=R, capture_output=True)
(out / 'build.jsonl').write_bytes(build.stdout)
(out / 'build.log').write_bytes(build.stderr)
assert build.returncode == 0, build.stderr[-2000:]
libraries = {}
for line in build.stdout.splitlines():
    item = json.loads(line)
    if item.get('reason') == 'compiler-artifact' and item['target']['name'] in ('game_core', 'game_server', 'actix', 'serde_json', 'uuid'):
        for filename in item['filenames']:
            if filename.endswith('.rlib'):
                libraries[item['target']['name']] = Path(filename)
assert set(libraries) == {'game_core', 'game_server', 'actix', 'serde_json', 'uuid'}
inputs = [libraries[name] for name in ('game_core', 'game_server', 'actix', 'serde_json', 'uuid')]
source = C / 'host_peak_failure_probe.rs'
with tempfile.TemporaryDirectory(prefix='s1f02-public-') as temp:
    binary = Path(temp) / 'probe'
    command = ['rustc', '--edition=2021', str(source)]
    for path in inputs:
        command += ['--extern', path.name[3:].split('-')[0] + '=' + str(path)]
    command += ['-L', 'dependency=' + str(inputs[0].parent), '-o', str(binary)]
    result = subprocess.run(command, capture_output=True)
    (out / 'compile.log').write_bytes(result.stdout + result.stderr)
    assert result.returncode == 0, result.stderr[-2000:]
    result = subprocess.run([str(binary)], capture_output=True)
    (out / 'run.log').write_bytes(result.stdout + result.stderr)
    fixture = Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_battle_starter_tactics/execution/single_goal/compatibility/current_001/inputs') / old['fixture']
    inputs += [source] + [p for p in fixture.rglob('*') if p.is_file()]
    receipt = dict(command=command, exit_code=result.returncode, source_sha256=sha(source), binary_sha256=sha(binary), input_files=[dict(path=str(p.relative_to(R)), sha256=sha(p)) for p in sorted(inputs)], output_files=[dict(path=p.name, sha256=sha(p)) for p in sorted(out.iterdir())], expectation='failed host publication preserves run and durable checkpoint without battle; retry enters battle with higher persisted peak')
    (out / 'RESULT.json').write_text(json.dumps(receipt, indent=2) + '\n')
    assert result.returncode == 0, result.stdout + result.stderr
    print(result.stdout.decode())
