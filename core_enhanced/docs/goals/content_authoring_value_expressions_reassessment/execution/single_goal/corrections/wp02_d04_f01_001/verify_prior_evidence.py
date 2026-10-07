"""Bind copied D04-F01 failure evidence to the frozen predecessor."""
import hashlib
import json
from pathlib import Path

ROOT = Path('/mnt/f/work/simulator')
HERE = Path(__file__).resolve().parent
REVIEW = ROOT / ('core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/'
                 'execution/single_goal/reviews/wp02_d04_resolved_target_001_review_001')
PRE = ROOT / ('core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/'
              'execution/single_goal/segments/wp02_d04_resolved_target_001/'
              'target/d04_resolved_target_001/MANIFEST.json')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


names = [
    'REVIEW.md', 'projectile_probes.rs', 'projectile_probes_001.log',
    'projectile_probes_001.json',
    'predecessor_control/scalar_projectiles_001.log',
    'predecessor_control/scalar_projectiles_001.json',
]
copies = []
for name in names:
    original = REVIEW / name
    copied = HERE / 'evidence/predecessor_review' / Path(name).name
    assert original.read_bytes() == copied.read_bytes()
    copies.append({'original': str(original.relative_to(ROOT)),
                   'copy': str(copied.relative_to(ROOT)),
                   'bytes': original.stat().st_size, 'sha256': sha(original)})
predecessor = json.loads(PRE.read_text())
failure = json.loads((REVIEW / 'projectile_probes_001.json').read_text())
frozen = {row['path']: row['after']['sha256']
          for row in predecessor['source']['files']}
assert set(failure['source_before']) == set(frozen)
assert failure['source_before'] == failure['source_after']
hooked = [name for name in frozen if failure['source_before'][name] != frozen[name]]
assert hooked == ['core_enhanced/src/game/battle/core/mod.rs']
assert failure['exit_code'] != 0
log = (REVIEW / 'projectile_probes_001.log').read_text()
assert all(name in log for name in [
    'review_scalar_snapshot_projectile_target_disappears',
    'review_scalar_live_projectile_target_disappears',
    'review_aggregate_snapshot_projectile_target_disappears',
    'review_aggregate_live_projectile_target_disappears',
])
assert '4 failed' in log
assert '2 passed' in (REVIEW / 'predecessor_control/scalar_projectiles_001.log').read_text()
result = {
    'predecessor_manifest_sha256': sha(PRE),
    'predecessor_source_digest': predecessor['source']['after_digest'],
    'review_hooked_source_paths': hooked,
    'review_failure_exit_code': failure['exit_code'],
    'review_failure_cases': 4,
    'predecessor_scalar_control_passed': 2,
    'copied_evidence': copies,
}
with (HERE / 'evidence/PRIOR_EVIDENCE.json').open('x') as stream:
    json.dump(result, stream, indent=2)
    stream.write('\n')
print(json.dumps({'failure_cases': 4, 'control_passed': 2,
                  'copied_records': len(copies), 'hooked_paths': hooked}))
