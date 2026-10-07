"""Execute remaining functional/public/numeric checks before isolated performance."""
import subprocess
import sys
from run import HERE

commands = [
    ('host_measurement_heap001', ['env','CARGO_PROFILE_TEST_DEBUG=0','CARGO_PROFILE_DEV_DEBUG=0','CARGO_INCREMENTAL=0',
      'cargo','test','--jobs=1','-p','game_server','--lib','--features','runtime-measurement','--offline','--locked',
      'exporter_heap_is_bounded_without_scrapes_and_retires_on_idle','--','--ignored','--nocapture','--test-threads=1']),
    ('measurement_support006', ['cargo','run','--jobs=1','-p','game_server','--features','runtime-measurement',
      '--example','measurement_support','--offline','--locked']),
    ('public_matrix001', ['python3',HERE/'tools/public_matrix.py','0','001','001','001']),
    ('release_matrix001', ['python3',HERE/'tools/release_matrix.py','0','001','001']),
]
start = int(sys.argv[1]) if len(sys.argv)>1 else 0
for name, command in commands[start:]:
    print('Starting '+name,flush=True)
    result = subprocess.run([sys.executable,str(HERE/'tools/run.py'),name,'--',*map(str,command)])
    if result.returncode:
        sys.exit(result.returncode)
