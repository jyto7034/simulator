import hashlib
import json
import tarfile
from pathlib import Path

root = Path('/mnt/f/work/simulator')
segment = root / 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_cap_01_02_001'
predecessor = root / 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_f02_r01_001/target/f02_r01_correction_002/MANIFEST.json'
baseline_archive = segment / 'baseline/entry_001/SOURCE_AND_INPUTS_BEFORE_CORRECTION.tar.gz'
source_paths = [entry['path'] for entry in json.loads(predecessor.read_text())['source']['files']]
with tarfile.open(baseline_archive) as archive:
    before = {member.name: archive.extractfile(member).read() for member in archive if member.isfile()}

hook = b'    #[path = "/mnt/f/work/simulator/core_enhanced/docs/tmp/r06_pc_baseline_20260926/probe.rs"]\n    mod load_baseline;\n'
adapter = b'''#[cfg(test)]
pub(crate) fn baseline_usage() -> Option<(u64, usize, usize)> {
    ACTIVE.with(|slot| slot.borrow().as_ref().map(|budget| {
        (budget.work, budget.clone_slots, budget.retained_admission.unwrap_or(0))
    }))
}
'''
instrumented = {
    'core_enhanced/src/game/battle/core/value_expr_runtime.rs': hook,
    'core_enhanced/src/game/battle/runtime_control.rs': adapter,
}

result = {}
for name, sandbox in [('before', Path('/tmp/r06-cap-before-source')),
                      ('after', Path('/tmp/r06-cap-after-source'))]:
    changed = []
    for path in source_paths:
        copied = (sandbox / path).read_bytes()
        if path in instrumented:
            marker = instrumented[path]
            if path.endswith('/runtime_control.rs'):
                marker = b'\n' + marker if name == 'before' else marker + b'\n'
            assert copied.count(marker) == 1, (name, path)
            copied = copied.replace(marker, b'', 1)
        expected = before[path] if name == 'before' else (root / path).read_bytes()
        assert copied == expected, (name, path)
        if copied != before[path]:
            changed.append(path)
    result[name] = {'source_files_verified': len(source_paths), 'product_delta_paths': changed,
                    'instrumentation_only_paths': list(instrumented)}

probe = root / 'core_enhanced/docs/tmp/r06_pc_baseline_20260926/probe.rs'
result['fixture_sha256'] = hashlib.sha256(probe.read_bytes()).hexdigest()
result['workload_unchanged'] = True
(segment / 'verification/pc_source_identity.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result, indent=2))
