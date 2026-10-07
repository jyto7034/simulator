import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[4] / 'tools'))
from release_gate import require
require('release')
import re
import sys
from pathlib import Path
sys.path.insert(0, '/mnt/f/work/simulator/core_enhanced/docs/goals/core_relic_numeric_variants/tools')
import run
run.HERE = Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_relic_numeric_variants/execution/single_goal/compatibility/current_001')
"""Check each corrected public cancellation input with diagnostic tracing enabled."""
from pathlib import Path
import json
import subprocess
import run

E = run.HERE / 'evidence'
label = sys.argv[1] if len(sys.argv) > 1 else 'after'
assert re.fullmatch(r'(?:after|after_[0-9]{3})', label)
input_root = E / ('public_' + label)
S = json.loads((input_root / 'SUMMARY.json').read_text())
O = E / ('public_' + label + '_trace')
O.mkdir(exist_ok=False)
rows = []

def events(path):
    found = {}
    def walk(value):
        if isinstance(value, dict):
            if {'seq', 'time_ms', 'event'} <= value.keys():
                found[value['seq']] = value
            for child in value.values():
                walk(child)
        elif isinstance(value, list):
            for child in value:
                walk(child)
    for line in path.read_text().splitlines():
        walk(json.loads(line))
    return found

for row in S['rows']:
    assert row['passed']
    name = row['case']
    trace = O / (name + '.trace.jsonl')
    command = row['command'] + ['--trace-output', str(trace)]
    assert run.sha(Path(command[0])) == S['binary_sha256']
    log = O / (name + '.jsonl')
    with log.open('xb') as stream:
        result = subprocess.run(command, cwd=run.ROOT, stdout=stream, stderr=subprocess.PIPE)
    (O / (name + '.stderr')).write_bytes(result.stderr)
    assert result.returncode == 0, (name, result.stderr)
    expected = events(input_root / (name + '.jsonl'))
    actual = events(log)
    assert expected and expected == actual, name
    assert trace.stat().st_size > 0
    rows.append(dict(case=name, command=command, exit_code=0, event_count=len(actual), gameplay_events_equal=True, output_sha256=run.sha(log), trace_sha256=run.sha(trace)))
with (O / 'SUMMARY.json').open('x') as stream:
    json.dump(dict(binary_sha256=S['binary_sha256'], input_summary_sha256=run.sha(input_root / 'SUMMARY.json'), rows=rows), stream, indent=2)
print(json.dumps({'passed':len(rows),'failed':0}))
