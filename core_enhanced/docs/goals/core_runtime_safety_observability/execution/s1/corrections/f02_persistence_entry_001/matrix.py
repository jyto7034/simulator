import subprocess
import sys
from pathlib import Path

C = Path(__file__).resolve().parent
D = C.parents[3]
runner = D / 'tools/run.py'
commands = [
    ('fmt', ['cargo', 'fmt', '--all', '--', '--check']),
    ('all_default', ['cargo', 'test', '-p', 'game_core', '--all-targets', '--offline', '--locked']),
    ('all_feature', ['cargo', 'test', '-p', 'game_core', '--all-targets', '--features', 'test-fixtures', '--offline', '--locked']),
    ('server', ['cargo', 'test', '-p', 'game_server', '--all-targets', '--offline', '--locked']),
    ('check', ['cargo', 'check', '-p', 'game_core', '-p', 'game_server', '--all-targets', '--offline', '--locked']),
    ('event_log', ['cargo', 'check', '-p', 'game_core', '--all-targets', '--features', 'event_log_checks', '--offline', '--locked']),
    ('clippy_core', ['cargo', 'clippy', '-p', 'game_core', '--all-targets', '--offline', '--locked', '--', '-A', 'clippy::never_loop', '-A', 'clippy::eq_op']),
    ('clippy_server', ['cargo', 'clippy', '-p', 'game_server', '--all-targets', '--offline', '--locked', '--', '-A', 'clippy::never_loop', '-A', 'clippy::eq_op']),
    ('release', ['cargo', 'build', '-p', 'game_core', '-p', 'game_server', '--release', '--bins', '--offline', '--locked']),
    ('public_probe', ['python3', str(C / 'probe.py')]),
    ('boundary', ['python3', str(C / 'boundary.py')]),
]
commands = commands[-3:] + commands[:-3]
for name, command in commands:
    print('Starting s1f02_' + name, flush=True)
    label = '002' if name in ('check', 'server') else '001'
    result = subprocess.run([sys.executable, str(runner), 's1f02_' + name + '_' + label, '--', *command])
    if result.returncode:
        sys.exit(result.returncode)
