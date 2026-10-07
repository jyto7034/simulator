import hashlib
import json
import tarfile
from pathlib import Path

root = Path('/mnt/f/work/simulator')
segment = root / 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_f02_f03_001'
target = segment / 'target/f02_f03_correction_001'
manifest_path = target / 'MANIFEST.json'
manifest_bytes = manifest_path.read_bytes()
manifest_hash = hashlib.sha256(manifest_bytes).hexdigest()
assert (target / 'MANIFEST.sha256').read_text().split()[0] == manifest_hash
manifest = json.loads(manifest_bytes)

def identity(path):
    payload = path.read_bytes()
    return (len(payload), hashlib.sha256(payload).hexdigest())

def archive_payloads(path):
    payloads = {}
    with tarfile.open(path, 'r:gz') as archive:
        for member in archive:
            assert member.isfile() and member.name not in payloads, member.name
            payloads[member.name] = archive.extractfile(member).read()
    return payloads

def verify_artifact(path, item):
    assert identity(path) == (item['bytes'], item['sha256']), path

for key in ('global_status', 'scoped_status'):
    verify_artifact(target / manifest[key]['path'], manifest[key])
for key in ('source', 'evidence'):
    archive_item = manifest[key]['archive']
    archive_path = target / archive_item['path']
    verify_artifact(archive_path, archive_item)
    archived = archive_payloads(archive_path)
    entries = {item['path']: item for item in manifest[key]['files']}
    assert len(entries) == manifest[key]['file_count']
    assert set(archived) == set(entries)
    for relative, item in entries.items():
        assert (len(archived[relative]), hashlib.sha256(archived[relative]).hexdigest()) == (item['bytes'], item['sha256']), relative
        verify_artifact(root / relative, item)

baseline_item = manifest['pre_implementation_baseline']
baseline_path = root / baseline_item['path']
verify_artifact(baseline_path, baseline_item)
baseline = json.loads(baseline_path.read_bytes())
before_archive = segment / 'baseline/entry_001' / baseline['archive']['path']
verify_artifact(before_archive, baseline['archive'])
restored = archive_payloads(before_archive)
assert set(restored) == {item['path'] for item in baseline['files']}

delta_item = manifest['delta_from_entry']
delta_path = target / delta_item['path']
verify_artifact(delta_path, delta_item)
delta = json.loads(delta_path.read_bytes())
delta_archive = target / delta_item['archive']['path']
verify_artifact(delta_archive, delta_item['archive'])
overlays = archive_payloads(delta_archive)
assert set(overlays) == set(delta['added']) | {item['path'] for item in delta['modified']}
for item in delta['deleted']:
    del restored[item['path']]
restored.update(overlays)
after = {item['path']: item for item in manifest['source']['files']}
assert set(restored) == set(after)
for relative, payload in restored.items():
    assert (len(payload), hashlib.sha256(payload).hexdigest()) == (after[relative]['bytes'], after[relative]['sha256']), relative

predecessor_item = manifest['predecessor_target']
verify_artifact(root / predecessor_item['path'], predecessor_item)
actual_scope = {item['path'] for item in baseline['files']}
for subtree in ('core_enhanced/src', 'core_enhanced/tests', 'game_resources/data'):
    actual_scope.update(str(path.relative_to(root)) for path in (root / subtree).rglob('*') if path.is_file())
assert actual_scope == set(after)
print(json.dumps({
    'result': 'FROZEN_TARGET_ARCHIVES_DELTA_AND_CURRENTNESS_VERIFIED',
    'manifest_sha256': manifest_hash,
    'source_files': len(after),
    'evidence_files': manifest['evidence']['file_count'],
    'modified_from_entry': [item['path'] for item in delta['modified']],
    'added_from_entry': delta['added'],
    'deleted_from_entry': [item['path'] for item in delta['deleted']],
    'predecessor_frozen_source_drift': manifest['predecessor_frozen_source_drift'],
}, indent=2))
