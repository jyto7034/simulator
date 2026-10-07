"""Prepare an isolated review probe. Never writes product source or frozen evidence."""
import hashlib
import json
import tarfile
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
SINGLE = HERE.parent.parent
TARGET = SINGLE / 'segments/wp02_d04_owner_relation_001/target/d04_owner_relation_001'
EXPECTED_MANIFEST = 'c2e703c2a08e1fc38141d3d070e052d8207cd7aaf9790f650c685429728a2a9f'
assert hashlib.sha256((TARGET / 'MANIFEST.json').read_bytes()).hexdigest() == EXPECTED_MANIFEST
expected = json.loads((HERE / 'REPRO_SETUP.json').read_text())
archive = TARGET / 'SOURCE_AFTER.tar.gz'
assert hashlib.sha256(archive.read_bytes()).hexdigest() == expected['source_archive_sha256']
isolate = Path(tempfile.mkdtemp(prefix='d04-owner-independent-', dir='/tmp'))
with tarfile.open(archive) as tar:
    for member in tar.getmembers():
        path = Path(member.name)
        assert not path.is_absolute() and '..' not in path.parts
        assert member.isfile() or member.isdir()
    tar.extractall(isolate)
source_path = isolate / expected['modified_path']
source = source_path.read_text()
assert hashlib.sha256(source_path.read_bytes()).hexdigest() == expected['unmodified_source_sha256']
needle = '''                .filter(|item| {
                    item.owner_unit_instance == unit.instance_id'''
assert source.count(needle) == 1
source = source.replace(needle, '''                .filter(|item| {
                    #[cfg(test)]
                    tests::REVIEW_SLOT_VISITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    item.owner_unit_instance == unit.instance_id''')
source = source.rstrip()
assert source.endswith('}')
source_path.write_text(source[:-1] + (HERE / 'REPRO_TEST.rs').read_text() + '\n}\n')
assert hashlib.sha256(source_path.read_bytes()).hexdigest() == expected['instrumented_source_sha256']
print(isolate)
