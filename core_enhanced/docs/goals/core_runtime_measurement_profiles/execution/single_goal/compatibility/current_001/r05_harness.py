"""Exact R05 fixture migration and historical include provenance, without cap changes."""
import hashlib
import json
from pathlib import Path

ROOT = Path('/mnt/f/work/simulator')
C = Path(__file__).resolve().parent
OLD_PRESERVATION = '64debfd6546afb617636caf520fcb9e8ee49acc1ee2377306cf8a19afbd8c631'
CURRENT_PRESERVATION = '2ae536ccfdbe6cf28692a6fe4dfa51e0225614d58445985524882e49a107ed14'
OLD_FINAL = '6dbb86885db3f3fb1a55226b3de016c50d2ad75a252190c07be1df0466eec640'
CURRENT_FINAL = 'bbbae7691cdc824bf2828f59e18b9cd2205baacca6fdca48cf98bfdd64b1e0f4'
ANCHOR = b'        battlefield: BattleFieldSpec {\n'
MIGRATION = b'            object_placement: None,\n'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify(label, source_sha256):
    assert label.isdigit()
    path = C / f'evidence/r05_harness_{label}/PROOF.json'
    proof = json.loads(path.read_text())
    assert proof['source_sha256'] == source_sha256
    assert proof['archive']['sha256'] == sha(ROOT / proof['archive']['path'])
    for name, row in proof['files'].items():
        assert row['sha256'] == sha(path.parent / name), name
    for family, old_hash, new_hash in [('preservation', OLD_PRESERVATION, CURRENT_PRESERVATION), ('final', OLD_FINAL, CURRENT_FINAL)]:
        before = (path.parent / f'{family}_original.rs').read_bytes()
        current = (path.parent / f'{family}_current.rs').read_bytes()
        assert hashlib.sha256(before).hexdigest() == old_hash
        assert hashlib.sha256(current).hexdigest() == new_hash
        assert before.count(ANCHOR) == current.count(ANCHOR + MIGRATION) == 1
        assert before.replace(ANCHOR, ANCHOR + MIGRATION) == current
        assert current.replace(ANCHOR + MIGRATION, ANCHOR) == before
        assert current == (ROOT / proof[family + '_workspace']).read_bytes()
    assert proof['baseline']['sha256'] == sha(ROOT / proof['baseline']['path'])
    baseline = json.loads((ROOT / proof['baseline']['path']).read_text())
    assert baseline['tool_source_sha256'] == OLD_PRESERVATION
    original_manifest = (path.parent / 'Cargo_original.toml').read_bytes()
    isolate_manifest = (path.parent / 'Cargo_isolate.toml').read_bytes()
    before = b'game_core = { path = "../..", features = ["test-fixtures"] }'
    after = ('game_core = { path = "' + str(ROOT / 'core_enhanced') + '", features = ["test-fixtures"] }').encode()
    assert original_manifest.count(before) == 1
    assert original_manifest.replace(before, after) == isolate_manifest
    assert original_manifest == (ROOT / proof['manifest_workspace']).read_bytes()
    assert (path.parent / 'Cargo.lock').read_bytes() == (ROOT / proof['lock_workspace']).read_bytes()
    isolate = Path(proof['isolate'])
    for name, row in proof['isolate_files'].items():
        assert sha(isolate / name) == row['sha256'], name
    assert (isolate / 'runtime_architecture_final_validation/src/final_performance.rs').read_bytes() == (path.parent / 'final_current.rs').read_bytes()
    assert (isolate / 'runtime_architecture_preservation/src/main.rs').read_bytes() == (path.parent / 'preservation_original.rs').read_bytes()
    assert (isolate / 'runtime_architecture_final_validation/Cargo.toml').read_bytes() == isolate_manifest
    assert (isolate / 'runtime_architecture_final_validation/Cargo.lock').read_bytes() == (path.parent / 'Cargo.lock').read_bytes()
    return path, proof
