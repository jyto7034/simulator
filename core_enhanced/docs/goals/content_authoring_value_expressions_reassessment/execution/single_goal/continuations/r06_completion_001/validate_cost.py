import subprocess,sys,json,time
from pathlib import Path
H=Path(__file__).resolve().parent
E=H/'evidence'
while not (E/'final_format_001.json').exists():time.sleep(10)
def run(name,args,cwd=None):
 return subprocess.run([sys.executable,str(H/'run.py'),name,*(['--cwd',str(cwd)] if cwd else []),'--',*args]).returncode
isolate=json.loads((E/'pc_isolate_boundary_001.json').read_text())['isolate']
assert run('pc_boundary_001',['env','CARGO_TARGET_DIR=/tmp/wp02-d07-b-pc-target','R06_BASELINE_OUTPUT='+str(E/'pc_raw_boundary_001.json'),'R06_BASELINE_SAMPLES=11','cargo','test','-p','game_core','--lib','--release','--offline','--locked','pc_load_baseline','--','--ignored','--nocapture','--test-threads=1'],isolate)==0
subprocess.run([sys.executable,str(H/'compare_pc.py'),str(E/'pc_raw_boundary_001.json'),str(E/'PC_SUMMARY.json')],check=True)
code=run('r05_boundary_001',['env','CARGO_TARGET_DIR=/tmp/wp02-d07-b-r05-target','cargo','run','--manifest-path','tools/runtime_architecture_final_validation/Cargo.toml','--bin','final_performance','--release','--features','allocation-observer','--offline','--locked','--','evidence/P01_PERFORMANCE_BASELINE.json',str(E/'r05_raw_boundary_001.json')],'/mnt/f/work/simulator/core_enhanced')
assert code in (0,101)
subprocess.run([sys.executable,str(H/'summarize_r05.py')],check=True)
