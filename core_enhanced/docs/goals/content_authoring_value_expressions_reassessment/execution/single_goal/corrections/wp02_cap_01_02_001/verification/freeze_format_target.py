import datetime
import gzip
import hashlib
import io
import json
import tarfile
from pathlib import Path

root = Path('/mnt/f/work/simulator')
segment = root / 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_cap_01_02_001'
predecessor = segment / 'target/cap_01_02_correction_001'
target = segment / 'target/cap_01_02_correction_002'


def identity(payload):
    return {'bytes': len(payload), 'sha256': hashlib.sha256(payload).hexdigest()}


def descriptor(path):
    return {'path': path.relative_to(root).as_posix(), **identity(path.read_bytes())}


def write_json(path, value):
    payload = (json.dumps(value, indent=2, ensure_ascii=False) + '\n').encode()
    path.write_bytes(payload)
    return identity(payload)


def write_archive(path, entries):
    with path.open('xb') as file:
        with gzip.GzipFile(fileobj=file, mode='wb', mtime=0, filename='') as packed:
            with tarfile.open(fileobj=packed, mode='w') as archive:
                for name, payload in sorted(entries.items()):
                    member = tarfile.TarInfo(name)
                    member.size = len(payload)
                    member.mode = 0o644
                    member.mtime = 0
                    archive.addfile(member, io.BytesIO(payload))
    return identity(path.read_bytes())


assert not target.exists(), target
predecessor_manifest_path = predecessor / 'MANIFEST.json'
assert identity(predecessor_manifest_path.read_bytes())['sha256'] == \
    '179bff1593608283c82ed51a2923f183d47312b8df51ad3dcb7556fdca8a4a5f'
predecessor_manifest = json.loads(predecessor_manifest_path.read_text())
with tarfile.open(predecessor / 'SOURCE_AFTER_CORRECTION.tar.gz', 'r:gz') as archive:
    before = {member.name: archive.extractfile(member).read()
              for member in archive if member.isfile()}
source_paths = [item['path'] for item in predecessor_manifest['source']['files']]
assert set(before) == set(source_paths) and len(source_paths) == 333
after = {path: (root / path).read_bytes() for path in source_paths}
modified = {path: after[path] for path in source_paths if before[path] != after[path]}
assert set(modified) == {
    'core_enhanced/src/authoring/control.rs',
    'core_enhanced/src/game/battle/runtime_control.rs',
    'core_enhanced/src/game/battle/core/runtime_facts.rs',
    'core_enhanced/src/game/battle/core/queue.rs',
    'core_enhanced/src/game/battle/core/value_expr_runtime.rs',
}

verification = segment / 'verification'
evidence_paths = [
    segment / 'FORMAT_REVIEW_HANDOFF.ko.md',
    verification / 'focused_rustfmt_baseline_exact.log',
    verification / 'focused_rustfmt.log',
    verification / 'focused_rustfmt_format_only.log',
    verification / 'verify_format_scope.py',
    verification / 'format_002_scope.json',
    verification / 'format_002_scope.log',
    verification / 'format_002_all_target_check.log',
    verification / 'format_002_all_target_test.log',
    verification / 'format_002_clippy_with_existing_allows.log',
    verification / 'freeze_format_target.py',
    verification / 'verify_format_target.py',
]
assert all(path.is_file() for path in evidence_paths)
evidence = {path.relative_to(root).as_posix(): path.read_bytes() for path in evidence_paths}
assert len(evidence) == len(evidence_paths)

target.mkdir(parents=True)
source_archive = write_archive(target / 'SOURCE_AFTER_FORMAT_CORRECTION.tar.gz', after)
evidence_archive = write_archive(target / 'EVIDENCE_AND_FORMAT_HANDOFF.tar.gz', evidence)
delta_archive = write_archive(target / 'SOURCE_DELTA_FROM_CORRECTION_001.tar.gz', modified)
delta = {
    'record': 'R06_WP02_CAP_FORMAT_SOURCE_DELTA_002',
    'predecessor': descriptor(predecessor_manifest_path),
    'modified': [
        {'path': path, 'before': identity(before[path]), 'after': identity(payload)}
        for path, payload in sorted(modified.items())
    ],
    'archive': delta_archive,
}
delta_identity = write_json(target / 'SOURCE_DELTA.json', delta)
manifest = {
    'record': 'R06_WP02_CAP_01_02_CORRECTION_TARGET_002',
    'state': 'format_correction_self_validation_complete_stage_review_pending',
    'captured_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'repository': str(root),
    'predecessor': descriptor(predecessor_manifest_path),
    'scope': {
        'implemented': ['format CAP-01/CAP-02 additions',
                        'format action-cast product regression additions'],
        'semantic_changes': [],
        'preserved': ['CAP-01/CAP-02 behavior and budgets', 'F01/F02-R01/F03 behavior',
                      'PC and R05 measurements', 'predecessor formatting outside changed regions'],
        'deferred': predecessor_manifest['scope']['deferred'],
    },
    'claims': {
        'self_validation_complete': True,
        'stage_review_target_frozen': True,
        'review_pass_declared': False,
        'wp_phase_goal_complete': False,
        'changed_region_rustfmt_complete': True,
        'whole_file_rustfmt_complete': False,
    },
    'source': {
        'file_count': len(after),
        'files': [{'path': path, **identity(payload)} for path, payload in sorted(after.items())],
        'archive': source_archive,
    },
    'evidence': {
        'file_count': len(evidence),
        'files': [{'path': path, **identity(payload)} for path, payload in sorted(evidence.items())],
        'archive': evidence_archive,
    },
    'delta': delta_identity,
}
manifest_identity = write_json(target / 'MANIFEST.json', manifest)
(target / 'MANIFEST.sha256').write_text(manifest_identity['sha256'] + '  MANIFEST.json\n')
print(json.dumps({
    'target': str(target),
    'manifest_sha256': manifest_identity['sha256'],
    'source_files': len(after),
    'modified': sorted(modified),
    'evidence_files': len(evidence),
}, indent=2))
