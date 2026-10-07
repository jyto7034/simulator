from pathlib import Path
import datetime, hashlib, json, os, subprocess, sys

HERE = Path(__file__).resolve().parent
CFG = json.loads((HERE / 'config.json').read_text())
ROOT, ISO = Path(CFG['root']), Path(CFG['isolate'])
tag, *cmd = sys.argv[1:]
def identities(root):
    return {f['path']: hashlib.sha256((root / f['path']).read_bytes()).hexdigest() for f in CFG['source']}
env = os.environ.copy()
env.update(PATH='/home/blast/.cargo/bin:' + env['PATH'], CARGO_HOME='/tmp/wp02-cargo-home',
           CARGO_TARGET_DIR=str(ISO / 'build'), TMPDIR=str(ISO / 'tmp'), CARGO_BUILD_JOBS='4')
receipt = dict(command=cmd, cwd=str(ISO), start=datetime.datetime.now(datetime.timezone.utc).isoformat(),
               source_before=identities(ISO), workspace_before=identities(ROOT),
               env={k:env[k] for k in ['CARGO_HOME','CARGO_TARGET_DIR','TMPDIR','CARGO_BUILD_JOBS']})
with (HERE / (tag + '.log')).open('xb') as log:
    result = subprocess.run(cmd, cwd=ISO, env=env, stdout=log, stderr=subprocess.STDOUT)
receipt.update(exit_code=result.returncode, end=datetime.datetime.now(datetime.timezone.utc).isoformat(),
               source_after=identities(ISO), workspace_after=identities(ROOT))
with (HERE / (tag + '.json')).open('x') as f:
    json.dump(receipt, f, indent=2); f.write('\n')
print(json.dumps(dict(tag=tag, exit_code=result.returncode, source_unchanged=receipt['source_before']==receipt['source_after'],
                      workspace_unchanged=receipt['workspace_before']==receipt['workspace_after'],
                      tail=(HERE / (tag + '.log')).read_text()[-2200:])))
sys.exit(result.returncode)
