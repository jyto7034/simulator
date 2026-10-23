"""Audit-local public replay; preserve frozen inputs and exercise a valid return.

Preparation: replay supported public supply, first return, mixed bloom, unfinished
Continue, SavePoint restore and completed maintenance in fresh paired stores.
Phase 4 then reaches depth-three return; the valid 2-part + 1-catalyst request
must reach journal IO, leave all roots unchanged on failure, and succeed on retry.
The frozen host checks exact inventories/lifecycle and disk bytes at each boundary.
This script additionally distinguishes filesystem failure from domain rejection.
"""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import tempfile
import time

AUDIT = Path(__file__).resolve().parent
EXECUTION = AUDIT.parent.parent
ROOT = next(p for p in EXECUTION.parents if (p / 'game_server/Cargo.toml').is_file())
OUT = Path(sys.argv[1]).resolve()
OUT.mkdir(parents=True, exist_ok=False)
STORES = Path(tempfile.mkdtemp(prefix='u15-audit-return-io-'))
HOST = EXECUTION / 'artifacts/normal001/u15_economy_host'
CONFIG = EXECUTION / 'test_parts/normal_inputs003/admission.json'
load = lambda path: json.loads(path.read_bytes())
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()

receipt = load(EXECUTION / 'artifacts/normal001/u15_economy_host.RECEIPT.json')
assert sha(HOST) == receipt['sha256']
request = load(EXECUTION / 'test_parts/host_replays001/phase4.json')
assert request['fail_index'] == 29 and 29 in request['reject_indices']
assert request['commands'][30] == {
    'type': 'return_from_expedition',
    'materials': [{'material_id': 'u15_part_b', 'amount': 2},
                  {'material_id': 'revival_catalyst', 'amount': 1}],
}
request['fail_index'] = 30
request_path = OUT / 'phase4.valid_return.json'
request_path.write_text(json.dumps(request, ensure_ascii=False, indent=2) + '\n')
results = []
for phase in range(1, 5):
    source = request_path if phase == 4 else EXECUTION / f'test_parts/host_replays001/phase{phase}.json'
    argv = [str(p) for p in [HOST, CONFIG, STORES / 'active', STORES / 'profile',
                            OUT / f'phase{phase}', source]]
    started = time.monotonic()
    with (OUT / f'phase{phase}.stdout.log').open('xb') as stdout, (OUT / f'phase{phase}.stderr.log').open('xb') as stderr:
        process = subprocess.run(argv, cwd=ROOT, stdout=stdout, stderr=stderr, timeout=180)
    result = {'phase': phase, 'argv': argv, 'input_sha256': sha(source),
              'exit_code': process.returncode, 'seconds': time.monotonic() - started}
    results.append(result)
    (OUT / f'phase{phase}.process.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result), flush=True)
    assert process.returncode == 0, result
    observed = load(OUT / f'phase{phase}/RESULT.json')
    assert observed['passed'] and observed['fresh_process']
    if phase > 1:
        failure = load(OUT / f'phase{phase}/IO_FAILURE.json')
        assert failure['passed'] and failure['memory_and_disk_unchanged']
        assert 'filesystem error: Is a directory (os error 21)' in failure['error'], failure
    if phase == 4:
        assert failure['index'] == 30 and failure['command'] == request['commands'][30]
        trace = load(OUT / 'phase4/commands.json')
        assert trace[29]['rejected'] is True
        assert trace[30]['result']['disposition'] == 'accepted'
        assert trace[30]['result']['kind'] == 'run_complete'
summary = {'passed': True, 'host_sha256': receipt['sha256'], 'stores': str(STORES),
           'fresh_processes': 4, 'actual_io_failures_and_retries': 3,
           'commands': sum(load(OUT / f'phase{i}/RESULT.json')['commands'] for i in range(1, 5)),
           'frozen_phase4_failure_was_domain_rejection': True,
           'valid_return_failure_index': 30, 'product_and_frozen_input_mutations': 0,
           'processes': results}
(OUT / 'RESULT.json').write_text(json.dumps(summary, indent=2) + '\n')
print(json.dumps({k: v for k, v in summary.items() if k != 'processes'}), flush=True)
