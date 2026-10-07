"""Audit-local command receipt. Never writes predecessor execution records."""
import hashlib, json, os, selectors, subprocess, sys, time
from pathlib import Path
ROOT = Path('/mnt/f/work/simulator')
HERE = Path(__file__).resolve().parent
TARGET = json.loads((HERE / 'AUDIT_TARGET.json').read_text())
def identity():
    rows = []
    for entry in TARGET['files']:
        data = (ROOT / entry['path']).read_bytes()
        rows.append([entry['path'], len(data), hashlib.sha256(data).hexdigest()])
    return hashlib.sha256(json.dumps(rows, separators=(',', ':')).encode()).hexdigest()
name, *command = sys.argv[1:]
assert command.pop(0) == '--'
log = HERE / (name + '.log')
receipt = HERE / (name + '.json')
assert not log.exists() and not receipt.exists()
env = os.environ.copy()
env['CARGO_HOME'] = '/tmp/wp02-cargo-home'
env['CARGO_TARGET_DIR'] = str(ROOT / '.cache/d07-runtime-amount-product-target')
env.pop('RUST_TEST_THREADS', None)
before = identity()
assert before == TARGET['source_digest']
start = time.time()
p = subprocess.Popen(command, cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
sel = selectors.DefaultSelector()
sel.register(p.stdout, selectors.EVENT_READ)
last = start
with log.open('xb') as stream:
    while True:
        ready = sel.select(5)
        if ready:
            chunk = os.read(p.stdout.fileno(), 65536)
            if not chunk:
                break
            stream.write(chunk)
            stream.flush()
        if time.time() - last >= 20:
            print(f'{name}: running {time.time()-start:.0f}s; raw log {log}', flush=True)
            last = time.time()
exit_code = p.wait()
after = identity()
data = {'command': command, 'cwd': str(ROOT), 'exit_code': exit_code,
        'elapsed_seconds': time.time()-start, 'source_before': before, 'source_after': after,
        'env': {k:env.get(k) for k in ('CARGO_HOME','CARGO_TARGET_DIR','RUST_TEST_THREADS')},
        'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest()}
with receipt.open('x') as f:
    json.dump(data, f, indent=2)
    f.write('\n')
print(json.dumps(data, indent=2), flush=True)
assert before == after
sys.exit(exit_code)
