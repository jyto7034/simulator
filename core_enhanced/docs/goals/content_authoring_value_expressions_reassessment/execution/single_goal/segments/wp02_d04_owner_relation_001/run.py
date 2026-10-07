"""Record one direct validation command against the actual current source."""
import hashlib
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path('/mnt/f/work/simulator')
HERE = Path(__file__).resolve().parent
BASE = json.loads((HERE / 'baseline/entry_001/MANIFEST.json').read_text())
TOOL_BASE = json.loads((HERE / 'baseline/tool_consumers_001/MANIFEST.json').read_text())
TOOL_LOCK_BASE = json.loads((HERE / 'baseline/tool_locks_001/MANIFEST.json').read_text())
SOURCE = [entry['path'] for entry in BASE['source']['files']]
SOURCE.extend(entry['path'] for entry in TOOL_BASE['source']['files'])
SOURCE.extend(entry['path'] for entry in TOOL_LOCK_BASE['source']['files'])


def identity(source_root=ROOT):
    rows = []
    for name in SOURCE:
        data = (source_root / name).read_bytes()
        rows.append((name, len(data), hashlib.sha256(data).hexdigest()))
    digest = hashlib.sha256(json.dumps(rows, separators=(',', ':')).encode()).hexdigest()
    return {'file_count': len(rows), 'digest': digest, 'files': rows}


if __name__ == '__main__':
    args = sys.argv[1:]
    name = args.pop(0)
    cwd = ROOT
    source_root = ROOT
    if args[:1] == ['--cwd']:
        cwd = ROOT / args[1]
        args = args[2:]
    if args[:1] == ['--source-dir']:
        source_root = Path(args[1])
        args = args[2:]
    assert args.pop(0) == '--'
    command = args
    evidence = HERE / 'evidence'
    evidence.mkdir(exist_ok=True)
    assert not (evidence / f'{name}.json').exists(), name
    env = os.environ.copy()
    env['PATH'] = '/home/blast/.cargo/bin:' + env['PATH']
    env.setdefault('CARGO_HOME', '/tmp/wp02-cargo-home')
    env.setdefault('CARGO_TARGET_DIR', '/tmp/wp02-cargo-target')
    before = identity(source_root)
    start = time.time()
    result = subprocess.run(command, cwd=cwd, env=env,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    elapsed = time.time() - start
    output = result.stdout
    after = identity(source_root)
    (evidence / f'{name}.log').write_bytes(output)
    groups = [(int(passed), int(failed)) for passed, failed in
              re.findall(rb'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed;', output)]
    receipt = {
        'command': command, 'cwd': str(cwd),
        'env': {key: env[key] for key in ('CARGO_HOME', 'CARGO_TARGET_DIR',
                                      'R06_BASELINE_OUTPUT', 'R06_BASELINE_SAMPLES') if key in env},
        'toolchain': {
            'cargo': subprocess.check_output(['/home/blast/.cargo/bin/cargo', '--version'], text=True).strip(),
            'rustc': subprocess.check_output(['/home/blast/.cargo/bin/rustc', '--version'], text=True).strip(),
        },
        'source_dir': str(source_root), 'source_before': before, 'source_after': after,
        'exit_code': result.returncode, 'elapsed_seconds': elapsed,
        'test_groups': groups, 'test_passed': sum(passed for passed, _ in groups),
        'test_failed': sum(failed for _, failed in groups),
        'log_sha256': hashlib.sha256(output).hexdigest(),
    }
    with (evidence / f'{name}.json').open('x') as stream:
        json.dump(receipt, stream, indent=2)
        stream.write('\n')
    print(json.dumps({'name': name, 'exit_code': result.returncode,
                      'test_passed': receipt['test_passed'],
                      'test_failed': receipt['test_failed'],
                      'elapsed_seconds': round(elapsed, 1),
                      'log_tail': output[-1400:].decode(errors='replace')}, indent=2))
    sys.exit(result.returncode)
