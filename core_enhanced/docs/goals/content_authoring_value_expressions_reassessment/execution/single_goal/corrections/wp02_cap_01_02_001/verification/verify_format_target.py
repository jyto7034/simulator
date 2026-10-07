import hashlib
import json
import tarfile
from pathlib import Path

root = Path('/mnt/f/work/simulator')
segment = root / 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_cap_01_02_001'
target = segment / 'target/cap_01_02_correction_002'


def identity(payload):
    return {'bytes': len(payload), 'sha256': hashlib.sha256(payload).hexdigest()}


def check(path, expected):
    assert identity(path.read_bytes()) == {key: expected[key] for key in ('bytes', 'sha256')}, path


def unpack(path):
    with tarfile.open(path, 'r:gz') as archive:
        result = {}
        for member in archive:
            assert member.isfile() and member.name not in result, member.name
            result[member.name] = archive.extractfile(member).read()
        return result


manifest_bytes = (target / 'MANIFEST.json').read_bytes()
manifest_hash = hashlib.sha256(manifest_bytes).hexdigest()
assert (target / 'MANIFEST.sha256').read_text().split()[0] == manifest_hash
manifest = json.loads(manifest_bytes)
predecessor_path = root / manifest['predecessor']['path']
check(predecessor_path, manifest['predecessor'])
predecessor = json.loads(predecessor_path.read_text())
predecessor_dir = predecessor_path.parent
check(predecessor_dir / 'SOURCE_AFTER_CORRECTION.tar.gz', predecessor['source']['archive'])
before = unpack(predecessor_dir / 'SOURCE_AFTER_CORRECTION.tar.gz')

for section, filename in [
    ('source', 'SOURCE_AFTER_FORMAT_CORRECTION.tar.gz'),
    ('evidence', 'EVIDENCE_AND_FORMAT_HANDOFF.tar.gz'),
]:
    descriptor = manifest[section]
    check(target / filename, descriptor['archive'])
    entries = unpack(target / filename)
    assert set(entries) == {item['path'] for item in descriptor['files']}
    assert len(entries) == descriptor['file_count']
    for item in descriptor['files']:
        assert identity(entries[item['path']]) == {
            key: item[key] for key in ('bytes', 'sha256')}, item['path']
        check(root / item['path'], item)

check(target / 'SOURCE_DELTA.json', manifest['delta'])
delta = json.loads((target / 'SOURCE_DELTA.json').read_text())
check(target / 'SOURCE_DELTA_FROM_CORRECTION_001.tar.gz', delta['archive'])
overlays = unpack(target / 'SOURCE_DELTA_FROM_CORRECTION_001.tar.gz')
assert set(overlays) == {item['path'] for item in delta['modified']}
after = {item['path']: item for item in manifest['source']['files']}
restored = {path: before[path] for path in after}
restored.update(overlays)
assert set(restored) == set(after)
for path, payload in restored.items():
    assert identity(payload) == {key: after[path][key] for key in ('bytes', 'sha256')}, path

print(json.dumps({
    'result': 'FROZEN_CAP_FORMAT_TARGET_AND_RESTORATION_VERIFIED',
    'manifest_sha256': manifest_hash,
    'source_files': manifest['source']['file_count'],
    'evidence_files': manifest['evidence']['file_count'],
    'modified': [item['path'] for item in delta['modified']],
}, indent=2))
