from pathlib import Path
import subprocess,sys
D=Path.cwd()/'core_enhanced/docs/goals/core_runtime_fault_isolation'
commands=[
('fmt',['cargo','fmt','--all','--','--check']),
('core_default',['cargo','test','-p','game_core','--all-targets','--offline','--locked']),
('core_fixtures',['cargo','test','-p','game_core','--all-targets','--features','test-fixtures','--offline','--locked']),
('core_observation',['cargo','test','-p','game_core','--all-targets','--features','runtime-observation','--offline','--locked']),
('host_default',['cargo','test','-p','game_server','--all-targets','--offline','--locked']),
('host_observation',['cargo','test','-p','game_server','--all-targets','--features','runtime-observation','--offline','--locked']),
('check',['cargo','check','-p','game_core','-p','game_server','--all-targets','--features','game_server/runtime-observation','--offline','--locked']),
('event_log',['cargo','check','-p','game_core','--all-targets','--features','event_log_checks,runtime-observation','--offline','--locked']),
('clippy_core',['cargo','clippy','-p','game_core','--all-targets','--features','runtime-observation','--offline','--locked','--','-A','clippy::never_loop','-A','clippy::eq_op']),
('clippy_host',['cargo','clippy','-p','game_server','--all-targets','--features','runtime-observation','--offline','--locked','--','-A','clippy::never_loop','-A','clippy::eq_op']),
('heap',['cargo','test','-p','game_server','--lib','--features','runtime-observation','--offline','--locked','exporter_heap_is_bounded_without_scrapes_and_retires_on_idle','--','--ignored','--nocapture','--test-threads=1']),
('release',['cargo','build','-p','game_core','-p','game_server','--release','--bins','--features','game_server/runtime-observation','--offline','--locked']),
]
for name,command in commands:
 print('Starting final_'+name,flush=True)
 p=subprocess.run([sys.executable,str(D/'tools/run.py'),'final_'+name+'001','--',*command])
 if p.returncode:sys.exit(p.returncode)
