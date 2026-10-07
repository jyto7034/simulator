"""Remaining integrated suites on one immutable source basis; stop on first refusal."""
from pathlib import Path
import subprocess,sys
H=Path(__file__).resolve().parents[1]
label=sys.argv[1] if len(sys.argv)>1 else '001'
commands=[
('fmt',['cargo','fmt','--all','--','--check']),
('core_default',['cargo','test','-p','game_core','--all-targets','--offline','--locked','--','--test-threads=8']),
('core_fixtures',['cargo','test','-p','game_core','--all-targets','--features','test-fixtures','--offline','--locked','--','--test-threads=8']),
('core_observation',['cargo','test','-p','game_core','--all-targets','--features','runtime-observation','--offline','--locked','--','--test-threads=8']),
('host_default',['cargo','test','-p','game_server','--all-targets','--offline','--locked','--','--test-threads=8']),
('host_observation',['cargo','test','-p','game_server','--all-targets','--features','runtime-observation','--offline','--locked','--','--test-threads=8']),
('check',['cargo','check','-p','game_core','-p','game_server','--all-targets','--features','game_server/runtime-observation','--offline','--locked']),
('event_log',['cargo','check','-p','game_core','--all-targets','--features','event_log_checks,runtime-observation','--offline','--locked']),
('clippy_core',['cargo','clippy','-p','game_core','--all-targets','--features','runtime-observation','--offline','--locked','--','-A','clippy::never_loop','-A','clippy::eq_op']),
('clippy_host',['cargo','clippy','-p','game_server','--all-targets','--features','runtime-observation','--offline','--locked','--','-A','clippy::never_loop','-A','clippy::eq_op']),
('heap',['cargo','test','-p','game_server','--lib','--features','runtime-observation','--offline','--locked','exporter_heap_is_bounded_without_scrapes_and_retires_on_idle','--','--ignored','--nocapture','--test-threads=1']),
]
start=int(sys.argv[2]) if len(sys.argv)>2 else 0
for name,command in commands[start:]:
 if command[1] != 'fmt': command.insert(2, '--jobs=1')
 if command[1] != 'fmt': command=['env','CARGO_PROFILE_TEST_DEBUG=0','CARGO_PROFILE_DEV_DEBUG=0','CARGO_INCREMENTAL=0',*command]
 print('Starting integrated_'+name,flush=True)
 result=subprocess.run([sys.executable,str(H/'tools/run.py'),'integrated_'+name+label,'--',*command])
 if result.returncode:sys.exit(result.returncode)
