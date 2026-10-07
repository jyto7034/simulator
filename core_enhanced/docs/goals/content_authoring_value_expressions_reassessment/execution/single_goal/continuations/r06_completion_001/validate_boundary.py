from pathlib import Path
import subprocess,sys,time
H=Path(__file__).resolve().parent;E=H/'evidence'
while not (E/'r05_boundary_001.json').exists():time.sleep(10)
for name,args in [('boundary_all_002',['test','--all-targets']),('boundary_feature_002',['test','--all-targets','--features','test-fixtures']),('boundary_clippy_002',['clippy','--all-targets']),('boundary_list_002',['test','--lib','r06_'])]:
 cmd=['cargo',args[0],'-p','game_core',*args[1:],'--offline','--locked']
 if 'clippy' in args:cmd+=['--','-A','clippy::never_loop','-A','clippy::eq_op']
 if name=='boundary_list_002':cmd+=['--','--list']
 subprocess.run([sys.executable,str(H/'run.py'),name,'--',*cmd],check=True)
subprocess.run([sys.executable,str(H/'run.py'),'boundary_format_002','--','cargo','fmt','--check'],check=True)
