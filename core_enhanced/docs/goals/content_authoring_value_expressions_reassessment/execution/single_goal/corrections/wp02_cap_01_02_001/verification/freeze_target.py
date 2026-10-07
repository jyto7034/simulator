import datetime
import gzip
import hashlib
import io
import json
import tarfile
from pathlib import Path

root = Path('/mnt/f/work/simulator')
segment = root / 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_cap_01_02_001'
target = segment / 'target/cap_01_02_correction_001'
baseline_path = segment / 'baseline/entry_001/MANIFEST.json'
predecessor_path = root / 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_f02_r01_001/target/f02_r01_correction_002/MANIFEST.json'


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
assert identity(baseline_path.read_bytes())['sha256'] == '21f63a5f801cd40e51488876ed1acac7faca851a4d439240d0dfbbbf2ad87b5c'
assert identity(predecessor_path.read_bytes())['sha256'] == 'ae2c540157c9caa769febb56e2664da7be8b5807b917e4ecfcb685103aa5f0f7'
baseline = json.loads(baseline_path.read_text())
source_paths = [item['path'] for item in json.loads(predecessor_path.read_text())['source']['files']]
assert len(source_paths) == baseline['source_file_count'] == 333
with tarfile.open(baseline_path.parent / baseline['archive']['path'], 'r:gz') as archive:
    before = {member.name: archive.extractfile(member).read() for member in archive if member.isfile()}
assert len(before) == baseline['file_count']

after = {path: (root / path).read_bytes() for path in source_paths}
modified = {path: after[path] for path in source_paths if after[path] != before[path]}
assert set(modified) == {
    'core_enhanced/src/authoring/control.rs',
    'core_enhanced/src/game/battle/runtime_control.rs',
    'core_enhanced/src/game/battle/core/runtime_facts.rs',
    'core_enhanced/src/game/battle/core/queue.rs',
    'core_enhanced/src/game/battle/core/value_expr_runtime.rs',
}

evidence_paths = [segment / 'REVIEW_HANDOFF.ko.md',
                  *(path for path in (segment / 'verification').rglob('*') if path.is_file())]
evidence = {path.relative_to(root).as_posix(): path.read_bytes() for path in evidence_paths}
assert len(evidence) == len(evidence_paths)

target.mkdir(parents=True)
source_archive = write_archive(target / 'SOURCE_AFTER_CORRECTION.tar.gz', after)
evidence_archive = write_archive(target / 'EVIDENCE_AND_HANDOFF.tar.gz', evidence)
delta_archive = write_archive(target / 'SOURCE_DELTA_FROM_ENTRY.tar.gz', modified)
delta = {'record': 'R06_WP02_CAP_01_02_SOURCE_DELTA_001',
         'baseline': descriptor(baseline_path),
         'modified': [{'path': path, 'before': identity(before[path]), 'after': identity(payload)}
                      for path, payload in sorted(modified.items())],
         'archive': delta_archive}
delta_identity = write_json(target / 'SOURCE_DELTA.json', delta)
manifest = {
    'record': 'R06_WP02_CAP_01_02_CORRECTION_TARGET_001',
    'state': 'correction_self_validation_complete_stage_review_pending',
    'captured_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'repository': str(root),
    'baseline': descriptor(baseline_path),
    'predecessor': descriptor(predecessor_path),
    'scope': {
        'implemented': ['CAP-01 ActorSummaries growth', 'CAP-02 EventQueue growth',
                        'action-cast product regression tests'],
        'preserved': ['F01 admission/cancellation', 'F02-R01 binding', 'F03 lifecycle',
                      'gameplay calculations and budget constants'],
        'deferred': ['sequential effects/result-prefix', 'WP03', 'publication'],
    },
    'claims': {'self_validation_complete': True, 'stage_review_target_frozen': True,
               'review_pass_declared': False, 'wp_phase_goal_complete': False,
               'focused_rustfmt_pass': False},
    'source': {'file_count': len(after),
               'files': [{'path': path, **identity(payload)} for path, payload in sorted(after.items())],
               'archive': source_archive},
    'evidence': {'file_count': len(evidence),
                 'files': [{'path': path, **identity(payload)} for path, payload in sorted(evidence.items())],
                 'archive': evidence_archive},
    'delta': delta_identity,
}
manifest_identity = write_json(target / 'MANIFEST.json', manifest)
(target / 'MANIFEST.sha256').write_text(manifest_identity['sha256'] + '  MANIFEST.json\n')
print(json.dumps({'target': str(target), 'manifest_sha256': manifest_identity['sha256'],
                  'source_files': len(after), 'modified': sorted(modified),
                  'evidence_files': len(evidence)}, indent=2))
