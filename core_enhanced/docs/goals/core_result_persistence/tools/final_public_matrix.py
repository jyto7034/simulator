"""Run all current public consumers sequentially with original assertions/caps."""
import os,subprocess,sys
from initialize import ROOT,HERE

env=os.environ.copy()
env['CARGO_BUILD_JOBS']='1'
env['PATH']='/home/blast/.cargo/bin:'+env['PATH']
env.setdefault('CARGO_HOME','/tmp/wp02-cargo-home')
env.setdefault('CARGO_TARGET_DIR',str(ROOT/'.cache/d07-runtime-amount-product-target'))
def execute(command):
    result=subprocess.run(list(map(str,command)),cwd=ROOT,env=env)
    if result.returncode:sys.exit(result.returncode)
def run(name,command):
    print('Starting '+name,flush=True)
    execute([sys.executable,HERE/'tools/run.py',name,'--',*command])
# Separate existing measurement target; stock host rebuild uses the shared target.
for name,feature in [('public_guards003',[]),('public_guards_allocator002',['--features','allocation-observer'])]:
    run(name,['env','CARGO_TARGET_DIR=/tmp/wp02-d07-b-r05-target','cargo','run','--jobs=1','--manifest-path','core_enhanced/tools/runtime_architecture_final_validation/Cargo.toml','--bin','public_guards','--release',*feature,'--offline','--locked','--',HERE/'evidence'/f'{name}_raw.json'])
run('integrated_release003',['cargo','build','--jobs=1','-p','game_core','-p','game_server','--release','--bins','--features','game_server/runtime-observation','--offline','--locked'])
for kind in ['continuation','rewind']:
    run('public_'+kind+'004',[sys.executable,HERE/'tools'/f'public_{kind}_probe.py','004'])
execute([sys.executable,HERE/'execution/single_goal/compatibility/current_003/regressions.py'])
execute([sys.executable,HERE/'tools/final_fault_probes.py','004'])
print('PASS all current public consumers',flush=True)
