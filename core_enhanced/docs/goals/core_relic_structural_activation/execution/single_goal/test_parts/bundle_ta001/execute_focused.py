#!/usr/bin/env python3
"""U09 independent focused test receipt collection; no oracle computation."""
import datetime, hashlib, json, os, pathlib, subprocess, sys, time
ROOT = pathlib.Path('/mnt/f/work/simulator')
OUT = pathlib.Path(__file__).resolve().parent

def ref(path):
    b=path.read_bytes()
    return {'path':str(path.relative_to(ROOT)),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
def identity():
    roots=('core_enhanced/src','core_enhanced/tests','game_server/src','game_server/tests','game_server/config','game_server/examples','config','game_resources/data','game_resources/live_data')
    paths=set()
    for name in roots:
        p=ROOT/name
        if p.exists(): paths.update(f for f in p.rglob('*') if f.is_file())
    for name in ('Cargo.toml','Cargo.lock','core_enhanced/Cargo.toml','game_server/Cargo.toml','core_enhanced/docs/audit/live_skill_catalog_manifest.ron','core_enhanced/docs/skills/live_content_index.md','core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json'):
        paths.add(ROOT/name)
    rows=[ref(p) for p in sorted(paths)]
    return {'files':rows,'sha256':hashlib.sha256(json.dumps(rows,sort_keys=True).encode()).hexdigest()}
label=sys.argv[1]
assert label.isidentifier() and label.startswith('u09_')
stdout,stderr,receipt=[OUT/(label+x) for x in ('.stdout','.stderr','.json')]
assert not any(p.exists() for p in (stdout,stderr,receipt))
env=os.environ.copy()
env.update(PATH='/home/blast/.cargo/bin:'+env['PATH'], CARGO_HOME='/tmp/wp02-cargo-home', CARGO_TARGET_DIR=str(ROOT/'.cache/d07-runtime-amount-product-target'), CARGO_BUILD_JOBS='1', CARGO_PROFILE_DEV_DEBUG='0', CARGO_PROFILE_TEST_DEBUG='0', CARGO_INCREMENTAL='0', PYTHONDONTWRITEBYTECODE='1')
command=['cargo','test','-p','game_core','--lib','game::relic::structural::tests','--offline','--locked','-j','1','--','--nocapture']
before=identity()
toolchain={name:subprocess.check_output([name,'--version'],env=env,text=True).strip() for name in ('rustc','cargo')}
started=datetime.datetime.now(datetime.timezone.utc).isoformat();tick=time.monotonic()
with stdout.open('xb') as out,stderr.open('xb') as err:
    process=subprocess.run(command,cwd=ROOT,env=env,stdout=out,stderr=err)
after=identity()
result={'label':label,'command':command,'cwd':str(ROOT),'environment':{k:env[k] for k in ('PATH','CARGO_HOME','CARGO_TARGET_DIR','CARGO_BUILD_JOBS','CARGO_PROFILE_DEV_DEBUG','CARGO_PROFILE_TEST_DEBUG','CARGO_INCREMENTAL')},'toolchain':toolchain,'started_at_utc':started,'closed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'duration_seconds':time.monotonic()-tick,'exit_code':process.returncode,'source_before':before,'source_after':after,'source_unchanged':before==after,'stdout':ref(stdout),'stderr':ref(stderr),'collector':ref(pathlib.Path(__file__).resolve()),'source_writers_during_check':0,'other_functional_runners':0,'kind':'independent_focused_tests','sharedhost_visibility':'visible PID namespace; no whole shared-host quiescence claim'}
import re
result['selected_test_count'] = sum(int(x) for x in re.findall(r'^running (\d+) tests?$', stdout.read_text(), re.M))
result['result_summaries'] = re.findall(r'^test result:.*$', stdout.read_text(), re.M)
receipt.write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({k:v for k,v in result.items() if k not in ('source_before','source_after','environment')},indent=2))
print(stderr.read_text(errors='replace')[-16000:])
sys.exit(process.returncode if before==after else 99)
