import hashlib
import json
import tarfile
from collections import Counter
from pathlib import Path

root = Path('/mnt/f/work/simulator')
segment = root / 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_cap_01_02_001'
predecessor = segment / 'target/cap_01_02_correction_001'
manifest = json.loads((predecessor / 'MANIFEST.json').read_text())

with tarfile.open(predecessor / 'SOURCE_AFTER_CORRECTION.tar.gz', 'r:gz') as archive:
    before = {member.name: archive.extractfile(member).read()
              for member in archive if member.isfile()}


regions = {
    'core_enhanced/src/authoring/control.rs': [
        ('pub(crate) fn growth_work_allowance', 'pub(crate) fn container_capacity_limit'),
    ],
    'core_enhanced/src/game/battle/runtime_control.rs': [
        ('pub(crate) fn admitted_buffer_capacity', '#[cfg(test)]'),
        ('    fn bounded_buffer_growth_uses_minimum_when_spare_reservation_exceeds_allowance',
         '    fn native_clone_limit_is_cumulative_inclusive_and_recovers_next_operation'),
    ],
    'core_enhanced/src/game/battle/core/runtime_facts.rs': [
        ('    fn actor_rows_grow_in_bounded_steps_and_cow_copies_only_live_rows',
         '    fn runtime_strike_admission_precedes_formula_limits_and_rolls_back'),
    ],
    'core_enhanced/src/game/battle/core/queue.rs': [
        ('pub(super) fn reserve_timestamp_bucket_slot', '#[derive(Debug, Clone, Default)]'),
        ('    fn queued_slots_grow_by_steps_fall_back_to_minimum_and_preserve_retry_order',
         '    fn native_packed_queue_preserves_mixed_priority_and_admits_shared_unpack'),
        ('    fn push_queued(', '    pub(super) fn pop('),
    ],
    'core_enhanced/src/game/battle/core/value_expr_runtime.rs': [
        ('    fn actual_cast_binding_sources(', None),
    ],
}


def mask_changed_regions(payload, selected):
    text = payload.decode()
    spans = []
    for ordinal, (start_marker, end_marker) in enumerate(selected):
        start = text.index(start_marker)
        end = len(text) if end_marker is None else text.index(end_marker, start)
        spans.append((start, end, f'<FORMAT_REGION_{ordinal}>'))
    for start, end, marker in sorted(spans, reverse=True):
        text = text[:start] + marker + text[end:]
    return text


changed = []
for item in manifest['source']['files']:
    path = item['path']
    current = (root / path).read_bytes()
    if current != before[path]:
        assert path in regions, path
        assert mask_changed_regions(current, regions[path]) == \
            mask_changed_regions(before[path], regions[path]), path
        changed.append({
            'path': path,
            'before_sha256': hashlib.sha256(before[path]).hexdigest(),
            'after_sha256': hashlib.sha256(current).hexdigest(),
        })

expected = {
    'core_enhanced/src/authoring/control.rs',
    'core_enhanced/src/game/battle/runtime_control.rs',
    'core_enhanced/src/game/battle/core/runtime_facts.rs',
    'core_enhanced/src/game/battle/core/queue.rs',
    'core_enhanced/src/game/battle/core/value_expr_runtime.rs',
}
assert {item['path'] for item in changed} == expected


def rustfmt_hunks(name):
    text = (segment / 'verification' / name).read_text()
    paths = [line.split(':', 1)[0].rsplit('/', 1)[-1]
             for line in text.splitlines() if line.startswith('Diff in ')]
    return dict(sorted(Counter(paths).items()))


baseline_hunks = rustfmt_hunks('focused_rustfmt_baseline_exact.log')
pre_format_hunks = rustfmt_hunks('focused_rustfmt.log')
post_format_hunks = rustfmt_hunks('focused_rustfmt_format_only.log')
assert baseline_hunks == {'control.rs': 4, 'queue.rs': 27, 'runtime_control.rs': 6}
assert pre_format_hunks == {
    'control.rs': 5,
    'queue.rs': 30,
    'runtime_control.rs': 11,
    'runtime_facts.rs': 4,
    'value_expr_runtime.rs': 7,
}
assert post_format_hunks == {'control.rs': 4, 'queue.rs': 25, 'runtime_control.rs': 6}

result = {
    'record': 'R06_WP02_CAP_FORMAT_SCOPE_001',
    'predecessor_manifest_sha256': hashlib.sha256(
        (predecessor / 'MANIFEST.json').read_bytes()).hexdigest(),
    'source_files_checked': len(manifest['source']['files']),
    'changed_files': changed,
    'changes_confined_to_cap_and_regression_regions': True,
    'format_mechanism': 'rustfmt output applied only inside the listed successor regions',
    'rustfmt_hunks': {
        'entry_baseline': baseline_hunks,
        'cap_target_before_format': pre_format_hunks,
        'format_target_after': post_format_hunks,
    },
    'scope_conclusion': (
        'All five successor deltas are confined to the CAP/regression regions and '
        'match rustfmt layout. New CAP/regression rustfmt hunks are gone; only '
        'predecessor hunks remain, minus two predecessor queue hunks whose bodies '
        'were replaced by the CAP change.'),
}
(segment / 'verification/format_002_scope.json').write_text(
    json.dumps(result, indent=2) + '\n')
print(json.dumps(result, indent=2))
