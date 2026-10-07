"""Bind a current-product R05 tool isolate to exact historical baseline bytes."""
import difflib
import json
import shutil
import sys
import tarfile
import tempfile
from pathlib import Path
sys.path.insert(0, '/mnt/f/work/simulator/core_enhanced/docs/goals/core_relic_numeric_variants/tools')
from run import identity
from r05_harness import ROOT, C, sha, verify

label = sys.argv[1]
assert label.isdigit()
source = identity()
out = C / f'evidence/r05_harness_{label}'
out.mkdir()
archive = ROOT / 'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/baseline/SOURCE_BEFORE.tar.gz'
assert sha(archive) == '76dff4a8e8e2afd535f0c6a072d1233c6782e37b573adfee105a9618b349bbc1'
preservation = 'core_enhanced/tools/runtime_architecture_preservation/src/main.rs'
final = 'core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs'
with tarfile.open(archive) as tar:
    for family, name in [('preservation', preservation), ('final', final)]:
        before = tar.extractfile(name).read()
        current = (ROOT / name).read_bytes()
        (out / f'{family}_original.rs').write_bytes(before)
        (out / f'{family}_current.rs').write_bytes(current)
        (out / f'{family}_migration.diff').write_text(''.join(difflib.unified_diff(before.decode().splitlines(True), current.decode().splitlines(True), fromfile=family + '_original.rs', tofile=family + '_current.rs')))
tool = ROOT / 'core_enhanced/tools/runtime_architecture_final_validation'
isolate = Path(tempfile.mkdtemp(prefix='u08-r05-harness-', dir='/tmp'))
cloned = isolate / 'runtime_architecture_final_validation'
shutil.copytree(tool / 'src', cloned / 'src')
for name in ('Cargo.toml', 'Cargo.lock'):
    shutil.copyfile(tool / name, cloned / name)
original_manifest = (cloned / 'Cargo.toml').read_bytes()
old = b'game_core = { path = "../..", features = ["test-fixtures"] }'
new = ('game_core = { path = "' + str(ROOT / 'core_enhanced') + '", features = ["test-fixtures"] }').encode()
assert original_manifest.count(old) == 1
(cloned / 'Cargo.toml').write_bytes(original_manifest.replace(old, new))
(out / 'Cargo_original.toml').write_bytes(original_manifest)
shutil.copyfile(cloned / 'Cargo.toml', out / 'Cargo_isolate.toml')
shutil.copyfile(cloned / 'Cargo.lock', out / 'Cargo.lock')
historical = isolate / 'runtime_architecture_preservation/src/main.rs'
historical.parent.mkdir(parents=True)
shutil.copyfile(out / 'preservation_original.rs', historical)
with tarfile.open(out / 'ISOLATE_INPUTS.tar.gz', 'x:gz') as tar:
    for p in sorted(isolate.rglob('*')):
        if p.is_file(): tar.add(p, arcname=str(p.relative_to(isolate)), recursive=False)
proof = {'source_sha256': source['sha256'], 'isolate': str(isolate),
         'archive': {'path': str(archive.relative_to(ROOT)), 'sha256': sha(archive)},
         'preservation_workspace': preservation, 'final_workspace': final,
         'manifest_workspace': str((tool / 'Cargo.toml').relative_to(ROOT)),
         'lock_workspace': str((tool / 'Cargo.lock').relative_to(ROOT)),
         'baseline': {'path': 'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/evidence/P01_LOG_V32_BASELINE_001.json'},
         'files': {p.name: {'bytes': p.stat().st_size, 'sha256': sha(p)} for p in sorted(out.iterdir()) if p.is_file()},
         'isolate_files': {str(p.relative_to(isolate)): {'bytes': p.stat().st_size, 'sha256': sha(p)} for p in sorted(isolate.rglob('*')) if p.is_file()},
         'adaptation': 'Each workspace Rust fixture differs from its historical source by exactly object_placement: None. Execute current final_performance byte-for-byte, including unchanged guards/workload/oracles. Its include_bytes provenance sentinel reads the exact historical preservation source, which is not compiled or executed here. Only Cargo game_core path changes to bind the fixed current product. Baseline/lock/caps/workload/oracles are unchanged; old and current tools remain separately archived.'}
proof['baseline']['sha256'] = sha(ROOT / proof['baseline']['path'])
with (out / 'PROOF.json').open('x') as stream: json.dump(proof, stream, indent=2); stream.write('\n')
verify(label, source['sha256'])
assert identity() == source
print(json.dumps({'label': label, 'isolate': str(isolate), 'source_sha256': source['sha256'], 'proof_sha256': sha(out / 'PROOF.json'), 'historical_preservation_sha256': sha(historical), 'executed_final_sha256': sha(cloned / 'src/final_performance.rs')}))
