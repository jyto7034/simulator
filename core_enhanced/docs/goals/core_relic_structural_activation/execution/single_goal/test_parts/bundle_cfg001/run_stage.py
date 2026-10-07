#!/usr/bin/env python3
"""Serial metadata-only runner. Never edits or compiles gameplay source."""
import datetime, hashlib, json, os, pathlib, subprocess, sys, time

ROOT = pathlib.Path('/mnt/f/work/simulator')
OUT = pathlib.Path(__file__).resolve().parent
BUNDLE = OUT.parent.parent / 'product_parts/bundle_b001'

def ref(path):
    raw = path.read_bytes()
    return {'path': str(path.relative_to(ROOT)), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}

def identity():
    roots = ('core_enhanced/src','core_enhanced/tests','game_server/src','game_server/tests','game_server/config','game_server/examples','config','game_resources/data','game_resources/live_data')
    paths = set()
    for name in roots:
        p = ROOT / name
        if p.exists(): paths.update(f for f in p.rglob('*') if f.is_file())
    paths.update(ROOT / name for name in ('Cargo.toml','Cargo.lock','core_enhanced/Cargo.toml','game_server/Cargo.toml','core_enhanced/docs/audit/live_skill_catalog_manifest.ron','core_enhanced/docs/skills/live_content_index.md','core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json'))
    rows = [ref(p) for p in sorted(paths)]
    return {'files': rows, 'sha256': hashlib.sha256(json.dumps(rows, sort_keys=True).encode()).hexdigest()}

def tool_identity():
    return [ref(p) for p in sorted(OUT.rglob('*')) if p.is_file() and not {'target','runs'}.intersection(p.relative_to(OUT).parts)]

def main():
    label, stage = sys.argv[1:]
    assert label.startswith('u09_cfg001_') and label.isidentifier()
    dest = OUT / 'runs' / label; dest.mkdir(parents=True, exist_ok=False)
    freeze_path = BUNDLE / 'u09_product_bundle_b_check012.json'
    freeze = json.loads(freeze_path.read_text())
    assert freeze['exit_code'] == 0 and freeze['source_unchanged']
    before = identity(); assert before == freeze['source_after'], 'source differs from Root frozen check012'
    tool_before = tool_identity()
    env = os.environ.copy()
    env.update(PATH='/home/blast/.cargo/bin:' + env['PATH'], CARGO_HOME='/tmp/wp02-cargo-home', CARGO_TARGET_DIR=str(OUT/'parser/target'), CARGO_BUILD_JOBS='1', CARGO_PROFILE_DEV_DEBUG='0', CARGO_INCREMENTAL='0', PYTHONDONTWRITEBYTECODE='1')
    if stage == 'build':
        commands = [
            ['cargo','generate-lockfile','--manifest-path',str(OUT/'parser/Cargo.toml'),'--offline'],
            ['cargo','build','--manifest-path',str(OUT/'parser/Cargo.toml'),'--offline','--locked','-j','1'],
        ]
    elif stage == 'verify':
        commands = [[sys.executable,'-B',str(OUT/'verify_cfg.py'),'--binary',str(OUT/'parser/target/debug/u09_cfg_boundary_parser'),'--output',str(dest/'verification.json')]]
    elif stage == 'manual':
        commands = [[sys.executable,'-B',str(OUT/'collect_manual_evidence.py'),'--output',str(dest/'manual_evidence.json')]]
    elif stage == 'manual2':
        commands = [[sys.executable,'-B',str(OUT/'compare_reviewed_ast.py'),'--binary',str(OUT/'parser/target/debug/u09_cfg_boundary_parser'),'--output',str(dest/'manual_ast_comparison.json')]]
    else: raise ValueError(stage)
    started = datetime.datetime.now(datetime.timezone.utc).isoformat(); tick=time.monotonic(); runs=[]
    for index, command in enumerate(commands):
        stdout=dest/f'{index}.stdout'; stderr=dest/f'{index}.stderr'
        with stdout.open('xb') as out, stderr.open('xb') as err:
            result=subprocess.run(command,cwd=ROOT,env=env,stdout=out,stderr=err)
        runs.append({'command':command,'exit_code':result.returncode,'stdout':ref(stdout),'stderr':ref(stderr)})
        if result.returncode: break
    after=identity(); tool_after=tool_identity()
    code=runs[-1]['exit_code']
    if before != after: code=99
    if stage != 'build' and tool_before != tool_after: code=98
    binary=OUT/'parser/target/debug/u09_cfg_boundary_parser'
    report={'label':label,'stage':stage,'kind':'Rust syntax metadata validation only; no game_core compile or functional tests','freeze':ref(freeze_path),'started_at_utc':started,'closed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'duration_seconds':time.monotonic()-tick,'runs':runs,'exit_code':code,'source_before':before,'source_after':after,'source_unchanged':before==after,'tool_before':tool_before,'tool_after':tool_after,'tool_unchanged':tool_before==tool_after,'tool_binary':ref(binary) if binary.exists() else None,'environment':{k:env[k] for k in ('CARGO_HOME','CARGO_TARGET_DIR','CARGO_BUILD_JOBS','CARGO_PROFILE_DEV_DEBUG','CARGO_INCREMENTAL')},'gameplay_source_writes':0,'functional_cargo_runs':0,'other_runners':0}
    path=dest/'RECEIPT.json';path.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({'receipt':ref(path),'exit_code':code,'source_unchanged':report['source_unchanged'],'tool_unchanged':report['tool_unchanged']},indent=2))
    for index in range(len(runs)):
        print((dest/f'{index}.stderr').read_text(errors='replace')[-12000:])
    return code

if __name__ == '__main__': raise SystemExit(main())
