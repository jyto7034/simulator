import subprocess,sys
from pathlib import Path
H=Path(__file__).resolve().parent
for name,args in [('integrated_all_002',['test','--all-targets']),('integrated_feature_002',['test','--all-targets','--features','test-fixtures']),('integrated_check_002',['check','--all-targets']),('integrated_event_check_002',['check','--all-targets','--features','event_log_checks']),('integrated_release_002',['build','--release','--bins']),('integrated_clippy_002',['clippy','--all-targets'])]:
 cmd=['cargo',args[0],'-p','game_core',*args[1:],'--offline','--locked']
 if args[0]=='clippy':cmd+=['--','-A','clippy::never_loop','-A','clippy::eq_op']
 subprocess.run([sys.executable,str(H/'run.py'),name,'--',*cmd],check=True)
subprocess.run([sys.executable,str(H/'run.py'),'integrated_fmt_002','--','cargo','fmt','--check'],check=True)
