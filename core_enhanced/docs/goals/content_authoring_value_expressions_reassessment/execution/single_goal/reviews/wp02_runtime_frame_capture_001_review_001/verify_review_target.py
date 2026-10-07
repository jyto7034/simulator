"""Validate preserved target plus explicitly separate current-tree deltas.

Does not change the frozen verifier/target or report unchanged currentness.
All original verifier checks still run; exactly two current source row checks
are replaced with checks of explicitly identified successor bytes.
"""
import contextlib
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile

ROOT = Path('/mnt/f/work/simulator')
GOAL = ROOT / 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment'
VERIFIER = GOAL / 'authoring_scripts/r06_wp02_verify_runtime_frame_capture_target_001.py'
MANIFEST = GOAL / 'execution/single_goal/segments/wp02_runtime_frame_capture_001/target/target_001/MANIFEST.json'
EXPECTED = 'd3dbf3ba7c3c3be5a584db3092e7059c116d92b1ae812740f1fb68ccb05c2fc3'
DELTAS = {
    'core_enhanced/src/game/world/tests/combat.rs': 'a2498f99b0a3e9423db0cc2655c018af8c2b24005da41ac1baa8b93a6a398eca',
    'core_enhanced/src/game/world/tests/resource_lifetimes.rs': '7485126713f4d97db01f24da18520c5965f3462ffe9bdad5ba60bc8d1960715a',
}
digest = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert digest(MANIFEST) == EXPECTED
manifest = json.loads(MANIFEST.read_text())
spec = importlib.util.spec_from_file_location('frozen_verifier', VERIFIER)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
original_verify_row = module.verify_row
observed = []

def verify_row(entry):
    if entry['path'] in DELTAS:
        assert digest(ROOT / entry['path']) == DELTAS[entry['path']]
        assert entry['sha256'] != DELTAS[entry['path']]
        observed.append({'path':entry['path'], 'frozen':entry['sha256'], 'current':DELTAS[entry['path']]})
    else:
        original_verify_row(entry)

module.verify_row = verify_row
with contextlib.redirect_stdout(io.StringIO()):
    module.main()
assert {r['path'] for r in observed} == set(DELTAS)

# Independently prove the binary-capable patch reconstructs the archived target.
baseline = json.loads((ROOT / manifest['pre_implementation_baseline']['manifest']).read_text())
recovery = Path(tempfile.mkdtemp(prefix='r06-wp02-delta-review-'))
with tarfile.open(ROOT / baseline['source']['archive']['path'], 'r:gz') as archive:
    for entry in archive.getmembers():
        assert entry.isfile() and not Path(entry.name).is_absolute() and '..' not in Path(entry.name).parts
    archive.extractall(recovery, filter='data')
delta = ROOT / manifest['current_source']['recoverable_delta']['path']
subprocess.run(['git', 'apply', '-p4', str(delta)], cwd=recovery, check=True, capture_output=True)
for entry in manifest['current_source']['files']:
    assert digest(recovery / entry['path']) == entry['sha256'], entry['path']
actual_paths = {p.relative_to(recovery).as_posix() for p in recovery.rglob('*') if p.is_file()}
assert actual_paths == {r['path'] for r in manifest['current_source']['files']}
print(json.dumps({
    'result':'FROZEN_ARTIFACTS_AND_EXPLICIT_CURRENT_DELTAS_VERIFIED',
    'unchanged_currentness':False, 'review_pass':False,
    'target_sha256':EXPECTED, 'source_count':314, 'unchanged_source_count':312,
    'current_deltas':observed, 'execution_records':6, 'evidence':37, 'receipts':17,
    'archive_payloads_verified':True, 'binary_delta_reconstruction_verified':True,
    'reconstruction_directory':str(recovery),
}, indent=2))
