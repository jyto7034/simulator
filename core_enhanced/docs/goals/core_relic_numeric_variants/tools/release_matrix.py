"""Release consumers with fresh receipt/output labels on a fixed source."""
import os
import subprocess
import sys
from initialize import ROOT, HERE
start = int(sys.argv[1]) if len(sys.argv) > 1 else 0
label = sys.argv[2] if len(sys.argv) > 2 else '003'
public_label = sys.argv[3] if len(sys.argv) > 3 else '002'
os.environ['U08_RELEASE_LABEL'] = label
from release_gate import bind, require
commands = [
    ('release_build_' + label, ['cargo', 'build', '--jobs=1', '--release', '-p', 'game_core', '--bins', '-p', 'game_server', '--features', 'game_server/runtime-measurement', '--offline', '--locked']),
    ('public_objects_' + label, ['python3', HERE / 'tools/public_objects.py', label]),
    ('public_object_combat_' + label, ['python3', HERE / 'tools/public_object_combat.py', label]),
    ('public_cost_guards_matrix_' + public_label, ['python3', HERE / 'tools/public_cost_guards.py', public_label]),
    ('final_fault_matrix_' + public_label, ['python3', HERE / 'tools/final_fault_probes.py', public_label]),
]
end = int(sys.argv[4]) if len(sys.argv) > 4 else len(commands)
for index, (name, command) in enumerate(commands[start:end], start):
    if index > 0:
        require('release')
    print('Starting ' + name, flush=True)
    if index >= 3:
        command = list(map(str, command))
    else:
        command = [sys.executable, str(HERE / 'tools/run.py'), name, '--', *map(str, command)]
    result = subprocess.run(command)
    if result.returncode:
        sys.exit(result.returncode)

    if index == 0:
        bind('release', label)
