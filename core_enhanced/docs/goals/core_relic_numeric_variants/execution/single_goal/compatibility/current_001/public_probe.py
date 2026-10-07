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
"""Re-execute the five frozen public F01 inputs without rewriting old evidence."""
import json
import subprocess
import sys
from pathlib import Path
import run
run.PRIOR = Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_independent_objects/execution/single_goal/compatibility/current_001/inputs') / 'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/reviews/r08_r12_20261002_001'

label = sys.argv[1]
assert re.fullmatch(r'(?:before|after|after_[0-9]{3})', label)
out = run.HERE / 'evidence' / ('public_' + label)
out.mkdir()
binary = run.ROOT / '.cache/d07-runtime-amount-product-target/release/author_content'
rows = []
input_files = {}

def events(path):
    found = {}
    def walk(value):
        if isinstance(value, dict):
            if 'time_ms' in value and 'event' in value:
                found[value['seq']] = value
            for child in value.values():
                walk(child)
        elif isinstance(value, list):
            for child in value:
                walk(child)
    for line in path.read_text().splitlines():
        walk(json.loads(line))
    return found

for folder, cases in [('timeout_marker_probe_002', ('early_cancel', 'cancel', 'detach')),
                      ('timeout_marker_probe_003', ('inflight_cancel', 'state_exit'))]:
    for case in cases:
        pack = run.PRIOR / 'evidence' / folder / case
        for path in pack.rglob('*'):
            if path.is_file():
                input_files[str(path.relative_to(run.ROOT))] = run.sha(path)
        command = [str(binary), 'simulate', str(pack/'content'), str(pack/'selection.json'),
                   str(pack/'scenario.json'), '--amount-observation']
        log = out / (case + '.jsonl')
        with log.open('xb') as stream:
            result = subprocess.run(command, cwd=run.ROOT, stdout=stream, stderr=subprocess.PIPE)
        (out / (case + '.stderr')).write_bytes(result.stderr)
        if result.returncode:
            raise RuntimeError((case, result.returncode, result.stderr[-2000:]))
        observed = events(log)
        steps = {value['event']['step_id']:value['time_ms'] for value in observed.values()
                 if value['event'].get('skill_id') in ('r08_strike','r12_boss_marker')
                 and value['event']['type'] == 'AbilityStepTriggered'}
        hp = [value for value in observed.values()
              if value['event']['type'] == 'HpChanged' and value['event'].get('raw_damage') == 1]
        passed = 'marker' in steps and (('area' in steps) == (case == 'detach'))
        if case == 'early_cancel':
            passed = passed and 'projectile' not in steps and not hp
        else:
            passed = passed and 'projectile' in steps and any(
                value['time_ms'] == 5885 and value['event']['delta'] == -1 for value in hp)
        rows.append(dict(case=case, command=command, exit_code=result.returncode, steps=steps,
                         hp=hp, passed=passed, log_sha256=run.sha(log)))

summary = dict(label=label, binary_sha256=run.sha(binary), probe_sha256=run.sha(Path(__file__)),
               input_files=input_files, passed=sum(row['passed'] for row in rows),
               failed=sum(not row['passed'] for row in rows), rows=rows)
with (out/'SUMMARY.json').open('x') as stream:
    json.dump(summary, stream, indent=2)
print(json.dumps({key:value for key,value in summary.items() if key not in ('input_files','rows')}, indent=2))
print(json.dumps([dict(case=row['case'], passed=row['passed'], steps=row['steps']) for row in rows], indent=2))
sys.exit(0 if summary['failed'] == 0 else 1)
