import copy
import hashlib
import json
from pathlib import Path
C = Path(__file__).resolve().parent
D = Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_runtime_fault_isolation')
a = Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_runtime_safety_observability/execution/s2/compatibility/current_001/evidence/pc_raw_current_002.json')
b = C / 'evidence/pc_raw_current_002.json'
def normalize(p):
    value = json.loads(p.read_text())
    for row in value['rows']:
        for key in ('measurement', 'setup'):
            if key in row:
                row[key].pop('elapsed_ns')
    return value
assert normalize(a) == normalize(b), 'Immediate U02 non-timing gameplay, native usage or allocation difference'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
(C / 'evidence/PC_IMMEDIATE_PARITY.json').write_text(json.dumps(dict(passed=True, before_sha256=sha(a), after_sha256=sha(b), exclusion='Only elapsed_ns inside setup and measurement; every other field compared', rows=576), indent=2) + '\n')
print('PASS 576 rows exact non-timing immediate U02 parity')
