"""Original PC36/R05 workloads on the final source, with fresh output paths."""
import subprocess,sys,json,shutil
from pathlib import Path
H=Path(__file__).resolve().parent;E=H/'evidence'
def run(name,args,cwd=None):
    return subprocess.run([sys.executable,str(H/'run.py'),name,*(['--cwd',str(cwd)] if cwd else []),'--',*map(str,args)]).returncode
subprocess.run([sys.executable,str(H/'prepare_pc_isolate_v32.py'),'final_004'],check=True)
isolate=json.loads((E/'pc_isolate_final_004.json').read_text())['isolate']
assert run('pc_final_004',['env','CARGO_TARGET_DIR=/tmp/wp02-d07-b-pc-target','R06_BASELINE_OUTPUT=/tmp/r08-r12-pc-raw-final-003.json','R06_BASELINE_SAMPLES=11','cargo','test','-p','game_core','--lib','--release','--offline','--locked','pc_load_baseline','--','--ignored','--nocapture','--test-threads=1'],isolate)==0
shutil.copyfile('/tmp/r08-r12-pc-raw-final-003.json',E/'pc_raw_final_004.json')
pc_compare = subprocess.run([sys.executable,str(H/'compare_pc_v32.py'),str(E/'pc_raw_final_004.json'),str(E/'PC_SUMMARY_004.json')]).returncode
code=run('r05_final_004',['env','CARGO_TARGET_DIR=/tmp/wp02-d07-b-r05-target','cargo','run','--manifest-path','tools/runtime_architecture_final_validation/Cargo.toml','--bin','final_performance','--release','--features','allocation-observer','--offline','--locked','--',str(E/'P01_LOG_V32_BASELINE_001.json'),str(E/'r05_raw_final_004.json')],'/mnt/f/work/simulator/core_enhanced')
assert code in (0,101)
subprocess.run([sys.executable,str(H/'summarize_r05_004.py')],check=True)

assert pc_compare == 0
