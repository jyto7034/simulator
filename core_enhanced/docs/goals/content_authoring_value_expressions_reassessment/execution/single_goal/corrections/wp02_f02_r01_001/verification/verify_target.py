import hashlib
import json
import tarfile
from pathlib import Path

root = Path('/mnt/f/work/simulator')
segment = root / 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_f02_r01_001'
target = segment / 'target/f02_r01_correction_002'


def identity(payload):
    return {'bytes': len(payload), 'sha256': hashlib.sha256(payload).hexdigest()}


def check_file(path, expected):
    assert identity(path.read_bytes()) == {k: expected[k] for k in ('bytes', 'sha256')}, path


def unpack(path, allow_identical_duplicates=False):
    with tarfile.open(path, 'r:gz') as archive:
        result = {}
        for member in archive:
            assert member.isfile(), member.name
            payload = archive.extractfile(member).read()
            if member.name in result:
                assert allow_identical_duplicates and result[member.name] == payload, member.name
            else:
                result[member.name] = payload
        return result


manifest_bytes = (target / 'MANIFEST.json').read_bytes()
manifest_hash = hashlib.sha256(manifest_bytes).hexdigest()
assert (target / 'MANIFEST.sha256').read_text().split()[0] == manifest_hash
manifest = json.loads(manifest_bytes)
baseline_path = root / manifest['baseline']['path']
check_file(baseline_path, manifest['baseline'])
baseline = json.loads(baseline_path.read_bytes())
check_file(root / manifest['predecessor']['path'], manifest['predecessor'])

before_path = baseline_path.parent / baseline['archive']['path']
check_file(before_path, baseline['archive'])
before = unpack(before_path, allow_identical_duplicates=True)
assert set(before) == {item['path'] for item in baseline['files']}
for item in baseline['files']:
    assert identity(before[item['path']]) == {k: item[k] for k in ('bytes', 'sha256')}

for section, name in (('source', 'SOURCE_AFTER_CORRECTION.tar.gz'),
                      ('evidence', 'EVIDENCE_AND_HANDOFF.tar.gz')):
    entry = manifest[section]
    check_file(target / name, entry['archive'])
    payloads = unpack(target / name)
    assert len(payloads) == entry['file_count']
    assert set(payloads) == {item['path'] for item in entry['files']}
    for item in entry['files']:
        expected = {k: item[k] for k in ('bytes', 'sha256')}
        assert identity(payloads[item['path']]) == expected, item['path']
        check_file(root / item['path'], item)

delta_path = target / 'SOURCE_DELTA.json'
check_file(delta_path, manifest['delta'])
delta = json.loads(delta_path.read_bytes())
check_file(target / 'SOURCE_DELTA_FROM_ENTRY.tar.gz', delta['archive'])
overlays = unpack(target / 'SOURCE_DELTA_FROM_ENTRY.tar.gz')
assert set(overlays) == {item['path'] for item in delta['modified']}
restored = {item['path']: before[item['path']] for item in baseline['files']
            if item['path'] in {source['path'] for source in manifest['source']['files']}}
restored.update(overlays)
after = {item['path']: item for item in manifest['source']['files']}
assert set(restored) == set(after)
for path, payload in restored.items():
    assert identity(payload) == {k: after[path][k] for k in ('bytes', 'sha256')}, path

print(json.dumps({'result': 'FROZEN_F02_R01_TARGET_AND_RESTORATION_VERIFIED',
                  'manifest_sha256': manifest_hash,
                  'source_files': manifest['source']['file_count'],
                  'evidence_files': manifest['evidence']['file_count'],
                  'modified': [item['path'] for item in delta['modified']]}, indent=2))
