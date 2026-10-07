"""Audit-only command receipt; immutable output names and source before/after."""
from pathlib import Path
import hashlib
import json
import os
import re
import subprocess
import sys
import time

ROOT = Path('/mnt/f/work/simulator')
HERE = Path(__file__).resolve().parent
OLD = ROOT/'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal'

def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f,'sha256').hexdigest()

def identity():
    baseline = json.loads((OLD/'baseline/MANIFEST.json').read_text())
    names = {r['path'] for r in baseline['files'] if (ROOT/r['path']).is_file()}
    for folder in ('core_enhanced/src','core_enhanced/tests','game_resources/data',
                   'core_enhanced/tools/runtime_architecture_final_validation','core_enhanced/tools/runtime_architecture_preservation'):
        for parent,dirs,files in os.walk(ROOT/folder):
            dirs[:] = [d for d in dirs if d not in ('target','__pycache__')]
            names.update(str((Path(parent)/f).relative_to(ROOT)) for f in files if (Path(parent)/f).is_file())
    names.add('core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json')
    rows = [[n,(ROOT/n).stat().st_size,sha(ROOT/n)] for n in sorted(names)]
    return dict(file_count=len(rows),digest=hashlib.sha256(json.dumps(rows,separators=(',',':')).encode()).hexdigest(),files=rows)

def main():
    name, separator, *command = sys.argv[1:]
    assert separator == '--' and command and re.fullmatch(r'[a-z0-9_]+',name)
    evidence = HERE/'evidence';evidence.mkdir(exist_ok=True)
    log = evidence/f'{name}.log';receipt=evidence/f'{name}.json'
    assert not log.exists() and not receipt.exists()
    env = os.environ.copy()
    env['PATH'] = '/home/blast/.cargo/bin:'+env['PATH']
    env.setdefault('CARGO_HOME','/tmp/wp02-cargo-home')
    env.setdefault('CARGO_TARGET_DIR',str(ROOT/'.cache/d07-runtime-amount-product-target'))
    before=identity();start=time.monotonic()
    with log.open('xb') as stream:
        p=subprocess.Popen(command,cwd=ROOT,env=env,stdout=stream,stderr=subprocess.STDOUT)
        while p.poll() is None:
            time.sleep(1)
        code=p.returncode
    after=identity();output=log.read_text(errors='replace')
    results=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',output)
    result=dict(command=command,cwd=str(ROOT),env={k:env.get(k) for k in ('CARGO_HOME','CARGO_TARGET_DIR','RUST_TEST_THREADS')},
        duration_seconds=time.monotonic()-start,exit_code=code,source_before=before,source_after=after,
        source_unchanged=before==after,log_sha256=sha(log),test_results=results,
        passed=sum(int(r[1]) for r in results),failed=sum(int(r[2]) for r in results),
        rustc=subprocess.check_output(['/home/blast/.cargo/bin/rustc','--version'],text=True).strip())
    with receipt.open('x') as f:json.dump(result,f,indent=2);f.write('\n')
    print(json.dumps({k:v for k,v in result.items() if k not in ('source_before','source_after','env')},ensure_ascii=False))
    print(output[-3500:])
    return code if before==after else 99

if __name__=='__main__':
    sys.exit(main())
