"""Measure unchanged PC/R05 workloads on the final U06 source, sequentially."""
import datetime,json,subprocess,sys
from pathlib import Path
from run import ROOT,HERE,identity
C=HERE/'execution/single_goal/compatibility/current_001'
E=C/'evidence'
preparation=json.loads((HERE/'evidence/compat_pc_prepare_003.json').read_text())
assert preparation['exit_code']==0 and preparation['source_before']==identity()==preparation['source_after']
isolate=json.loads((E/'pc_isolate_003.json').read_text())['isolate']
own_compilers=[];other_training=False
for directory in Path('/proc').iterdir():
 if not directory.name.isdigit():continue
 try:
  name=(directory/'comm').read_text().strip();command=(directory/'cmdline').read_bytes()
  other_training=other_training or b'komap.train' in command
  if name in ('cargo','rustc') and command and (str(ROOT).encode() in command or (directory/'cwd').resolve()==ROOT):own_compilers.append(int(directory.name))
 except (FileNotFoundError,PermissionError,ProcessLookupError):pass
assert not own_compilers,'Finish our compilation and functional checks before recording performance'
with (E/'PERFORMANCE_ENVIRONMENT_001.json').open('x') as f:
 json.dump({'recorded_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_sha256':identity()['sha256'],'memory_info':Path('/proc/meminfo').read_text(),'other_training_present':other_training,'own_cargo_rustc_active':False,'timing_claim':'Shared-host raw observations only; allocation observer is process/thread scoped. PC gameplay, typed refusal and full non-timing parity are checked separately; inherited R05 allocation instrumentation disables its latency gate. No numeric ceiling changes.'},f,indent=2);f.write('\n')
def run(name,command,allowed=(0,)):
 print('Starting '+name,flush=True)
 result=subprocess.run([sys.executable,str(HERE/'tools/run.py'),name,'--',*map(str,command)])
 assert result.returncode in allowed,(name,result.returncode)
run('compat_pc_001',['env','CARGO_TARGET_DIR=/tmp/wp02-d07-b-pc-target',f'R06_BASELINE_OUTPUT={E}/pc_raw_current_001.json','R06_BASELINE_SAMPLES=11','cargo','test','--jobs=1','--manifest-path',isolate+'/Cargo.toml','-p','game_core','--lib','--release','--offline','--locked','pc_load_baseline','--','--ignored','--nocapture','--test-threads=1'])
run('compat_pc_gameplay_001',['python3',ROOT/'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/compare_pc_v32.py',E/'pc_raw_current_001.json',E/'PC_SUMMARY_CURRENT_001.json'])
run('compat_r05_002',['env','CARGO_TARGET_DIR=/tmp/wp02-d07-b-r05-target','cargo','run','--jobs=1','--manifest-path','core_enhanced/tools/runtime_architecture_final_validation/Cargo.toml','--bin','final_performance','--release','--features','allocation-observer','--offline','--locked','--',ROOT/'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/evidence/P01_LOG_V32_BASELINE_001.json',E/'r05_raw_current_002.json'],(0,101))
run('compat_r05_summary_001',['python3',C/'summarize_r05_current.py'])
