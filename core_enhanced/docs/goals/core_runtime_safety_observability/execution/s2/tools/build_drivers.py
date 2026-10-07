from pathlib import Path
import subprocess,sys,os,hashlib,json,shutil
R=Path.cwd();D=R/'core_enhanced/docs/goals/core_runtime_safety_observability';S=D/'execution/s2';out=R/'.cache/crso-s2-drivers-002';out.mkdir(exist_ok=False)
rows=[]
for mode,features in [('on',['--features','runtime-observation']),('off',[])]:
 cmd=['cargo','build','--release','--offline','--locked','-p','game_server','--example','observe_runtime',*features]
 code=subprocess.run([sys.executable,str(D/'tools/run.py'),'s2_driver_release_'+mode+'002','--',*cmd]).returncode
 assert code==0
 source=R/'.cache/d07-runtime-amount-product-target/release/examples/observe_runtime';dest=out/mode
 shutil.copy2(source,dest)
 rows.append({'mode':mode,'path':str(dest),'sha256':hashlib.sha256(dest.read_bytes()).hexdigest(),'bytes':dest.stat().st_size,'command':cmd})
(S/'evidence/driver_binaries002.json').write_text(json.dumps(rows,indent=2))
