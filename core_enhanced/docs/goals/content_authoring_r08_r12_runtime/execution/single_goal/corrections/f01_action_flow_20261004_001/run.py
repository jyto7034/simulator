"""Immutable U01 command evidence; reuse the audited source membership rule."""
from pathlib import Path
import hashlib
import importlib.util
import json
import os
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[8]
HERE = Path(__file__).resolve().parent
PRIOR = ROOT / 'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/reviews/r08_r12_20261002_001'
spec = importlib.util.spec_from_file_location('u01_prior_identity', PRIOR / 'run.py')
prior = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prior)
identity = prior.identity
sha = prior.sha

def main():
    name, separator, *command = sys.argv[1:]
    assert separator == '--' and command and re.fullmatch(r'[a-z0-9_]+', name)
    evidence = HERE / 'evidence'
    evidence.mkdir(exist_ok=True)
    log = evidence / (name + '.log')
    receipt = evidence / (name + '.json')
    assert not log.exists() and not receipt.exists()
    env = os.environ.copy()
    env['PATH'] = '/home/blast/.cargo/bin:' + env['PATH']
    env.setdefault('CARGO_HOME', '/tmp/wp02-cargo-home')
    env.setdefault('CARGO_TARGET_DIR', str(ROOT / '.cache/d07-runtime-amount-product-target'))
    before = identity()
    started = time.monotonic()
    with log.open('xb') as stream:
        process = subprocess.run(command, cwd=ROOT, env=env, stdout=stream, stderr=subprocess.STDOUT)
    after = identity()
    output = log.read_text(errors='replace')
    results = re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out', output)
    result = dict(command=command, cwd=str(ROOT), runner_sha256=sha(Path(__file__)),
                  environment={key: env.get(key) for key in ('CARGO_HOME', 'CARGO_TARGET_DIR', 'RUST_TEST_THREADS')},
                  duration_seconds=time.monotonic()-started, exit_code=process.returncode,
                  source_before=before, source_after=after, source_unchanged=before == after,
                  log_sha256=sha(log), test_results=results,
                  passed=sum(int(row[1]) for row in results), failed=sum(int(row[2]) for row in results),
                  rustc=subprocess.check_output(['/home/blast/.cargo/bin/rustc', '--version'], text=True).strip())
    with receipt.open('x') as stream:
        json.dump(result, stream, indent=2)
        stream.write('\n')
    print(json.dumps({key:value for key,value in result.items() if key not in ('source_before','source_after')}, ensure_ascii=False))
    print(output[-2500:])
    return process.returncode if before == after else 99

if __name__ == '__main__':
    sys.exit(main())
