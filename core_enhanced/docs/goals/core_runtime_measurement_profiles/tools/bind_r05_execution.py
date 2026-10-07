"""Bind the actual fresh compiler include/dependency paths to an R05 receipt."""
import json
import sys
from pathlib import Path
from run import ROOT, HERE, identity, sha

label = sys.argv[1]
assert label.isdigit()
C = HERE / 'execution/single_goal/compatibility/current_001'
sys.path.insert(0, str(C))
from r05_harness import verify, CURRENT_FINAL, OLD_PRESERVATION

current = identity()
proof_path, proof = verify(label, current['sha256'])
receipt_path = HERE / f'evidence/compat_r05_{label}.json'
receipt = json.loads(receipt_path.read_text())
assert receipt['source_before'] == current == receipt['source_after'] and receipt['source_unchanged']
assert receipt['exit_code'] in (0, 101)
target = Path(f'/tmp/u07m-r05-target{label}')
assert f'CARGO_TARGET_DIR={target}' in receipt['command']
environment_path = C / f'evidence/PERFORMANCE_ENVIRONMENT_{label}_r05.json'
environment = json.loads(environment_path.read_text())
assert environment['source_sha256'] == current['sha256']
assert environment['fresh_r05_target'] == str(target) and environment['r05_target_existed_before_measurement'] is False
deps = list((target / 'release/deps').glob('final_performance-*.d'))
assert len(deps) == 1, deps
dependency = deps[0].read_bytes()
main = Path(proof['isolate']) / 'runtime_architecture_final_validation/src/final_performance.rs'
include = main.parent / '../../runtime_architecture_preservation/src/main.rs'
# Cargo passes crate source paths relative to the actual package directory.
# Preserve raw depinfo and bind its exact source token set to that directory,
# which is independently identified by manifest command and compile log.
package = main.parent.parent
source_tokens = set()
for line in dependency.decode().splitlines():
    if ': ' in line:
        source_tokens.update(line.split(': ', 1)[1].split())
expected_tokens = {'src/final_performance.rs', 'src/../../runtime_architecture_preservation/src/main.rs'}
assert source_tokens == expected_tokens, source_tokens
resolved_dependencies = {token: str((package / token).resolve()) for token in sorted(source_tokens)}
assert set(resolved_dependencies.values()) == {str(main.resolve()), str(include.resolve())}
assert sha(main) == CURRENT_FINAL and sha(include) == OLD_PRESERVATION
assert not main.is_symlink() and not include.is_symlink()
assert str(ROOT / 'core_enhanced/tools/runtime_architecture_preservation/src/main.rs').encode() not in dependency
raw = C / f'evidence/r05_raw_current_{label}.json'
summary = C / f'evidence/R05_SUMMARY_CURRENT_{label}.json'
assert raw.is_file() and summary.is_file()
log = HERE / f'evidence/compat_r05_{label}.log'
assert receipt['log_sha256'] == sha(log)
assert f"Compiling runtime_architecture_final_validation v0.1.0 ({main.parent.parent})" in log.read_text()
out = proof_path.parent / 'EXECUTION_DEPENDENCIES.json'
copy = proof_path.parent / 'FRESH_TARGET_DEPINFO.d'
with copy.open('xb') as stream:
    stream.write(dependency)
result = {'source_sha256': current['sha256'], 'receipt': str(receipt_path.relative_to(ROOT)),
          'receipt_sha256': sha(receipt_path), 'harness_proof_sha256': sha(proof_path),
          'environment': str(environment_path.relative_to(ROOT)), 'environment_sha256': sha(environment_path),
          'target': str(target), 'target_was_absent': True, 'depinfo_sha256': sha(copy),
          'depinfo_resolution_base':str(package), 'depinfo_sources':resolved_dependencies,
          'executed_main': str(main), 'executed_main_sha256': sha(main),
          'actual_include': str(include), 'actual_include_sha256': sha(include),
          'raw_sha256': sha(raw), 'summary_sha256': sha(summary),
          'binary_sha256': sha(target / 'release/final_performance'), 'passed': True}
with out.open('x') as stream:
    json.dump(result, stream, indent=2)
    stream.write('\n')
print('PASS fresh R05 target, actual migrated main and historical include depinfo')
