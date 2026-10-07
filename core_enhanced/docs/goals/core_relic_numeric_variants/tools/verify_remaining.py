"""Fresh release before public consumers; fresh debug before finite playback."""
import os
import subprocess
import sys
from run import HERE
from release_gate import bind

start = int(sys.argv[1]) if len(sys.argv) > 1 else 0
label = sys.argv[2] if len(sys.argv) > 2 else '001'
assert label.isdigit()
os.environ['U08_RELEASE_LABEL'] = label
os.environ['U08_DEBUG_LABEL'] = label
commands = [
    ('host_measurement_heap' + label, ['env','CARGO_PROFILE_TEST_DEBUG=0','CARGO_PROFILE_DEV_DEBUG=0','CARGO_INCREMENTAL=0',
     'cargo','test','--jobs=1','-p','game_server','--lib','--features','runtime-measurement','--offline','--locked',
     'exporter_heap_is_bounded_without_scrapes_and_retires_on_idle','--','--ignored','--nocapture','--test-threads=1']),
    ('measurement_support' + label, ['cargo','run','--jobs=1','-p','game_server','--features','runtime-measurement',
     '--example','measurement_support','--offline','--locked']),
    ('release_matrix' + label, ['python3',HERE/'tools/release_matrix.py','0',label,label]),
    ('public_matrix' + label, ['python3',HERE/'tools/public_matrix.py','0',label,label,label]),
    ('debug_build_' + label, ['cargo','build','--jobs=1','-p','game_core','--bins','-p','game_server',
     '--features','game_server/runtime-measurement','--offline','--locked']),
    ('playback_prepare' + label, ['python3',HERE/'tools/prepare_playback_inputs.py',label]),
    ('playback_actual' + label, ['python3',HERE/'tools/playback_matrix.py',label,label]),
    ('lifecycle_actual' + label, ['python3',HERE/'tools/sample_lifecycle.py',label,label]),
    ('host_actual' + label, ['python3',HERE/'tools/sample_host.py',label,label]),
]
for name, command in commands[start:]:
    print('Starting ' + name, flush=True)
    result = subprocess.run([sys.executable,str(HERE/'tools/run.py'),name,'--',*map(str,command)])
    if result.returncode:
        sys.exit(result.returncode)
    if name == 'debug_build_' + label:
        bind('debug', label)
