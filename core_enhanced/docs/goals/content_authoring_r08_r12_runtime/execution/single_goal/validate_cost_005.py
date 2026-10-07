"""Original PC36/R05 workloads on the final source, with fresh output paths."""
import subprocess,sys,json,shutil
from pathlib import Path
H=Path(__file__).resolve().parent;E=H/'evidence'
def run(name,args,cwd=None):
    return subprocess.run([sys.executable,str(H/'run.py'),name,*(['--cwd',str(cwd)] if cwd else []),'--',*map(str,args)]).returncode
# Reuse the unchanged final004 instrumented isolate; only the output path collided.
assert (E/'pc_isolate_final_005.json').is_file()
isolate=json.loads((E/'pc_isolate_final_005.json').read_text())['isolate']
assert run('pc_final_005',['env','CARGO_TARGET_DIR=/tmp/wp02-d07-b-pc-target','R06_BASELINE_OUTPUT=/tmp/r08-r12-pc-raw-final-005.json','R06_BASELINE_SAMPLES=11','cargo','test','-p','game_core','--lib','--release','--offline','--locked','pc_load_baseline','--','--ignored','--nocapture','--test-threads=1'],isolate)==0
shutil.copyfile('/tmp/r08-r12-pc-raw-final-005.json',E/'pc_raw_final_005.json')
pc_compare = subprocess.run([sys.executable,str(H/'compare_pc_v32.py'),str(E/'pc_raw_final_005.json'),str(E/'PC_SUMMARY_005.json')]).returncode
code=run('r05_final_005',['env','CARGO_TARGET_DIR=/tmp/wp02-d07-b-r05-target','cargo','run','--manifest-path','tools/runtime_architecture_final_validation/Cargo.toml','--bin','final_performance','--release','--features','allocation-observer','--offline','--locked','--',str(E/'P01_LOG_V32_BASELINE_001.json'),str(E/'r05_raw_final_005.json')],'/mnt/f/work/simulator/core_enhanced')
assert code in (0,101)
subprocess.run([sys.executable,str(H/'summarize_r05_005.py')],check=True)

assert pc_compare == 0
