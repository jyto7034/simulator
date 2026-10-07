"""Measure unchanged PC/R05 workloads sequentially on one exact source.

Labels select fresh receipts/outputs; frozen workloads, probes and caps retain
those owners. PC gameplay is a separate gate before the measured cost review.
"""
import argparse
import datetime
import json
import subprocess
import sys
from pathlib import Path
from run import ROOT, HERE, identity

parser = argparse.ArgumentParser()
parser.add_argument('--label', default='001')
parser.add_argument('--prepare-label', default='001')
parser.add_argument('--phase', choices=('pc', 'r05'), default='pc')
parser.add_argument('--r05-prepare-label', default='004')
args = parser.parse_args()
assert args.label.isdigit() and args.prepare_label.isdigit() and args.r05_prepare_label.isdigit()
C = HERE / 'execution/single_goal/compatibility/current_001'
E = C / 'evidence'
source = identity()
preparation = json.loads((HERE / f'evidence/pc_prepare{args.prepare_label}.json').read_text())
assert preparation['exit_code'] == 0 and preparation['source_before'] == source == preparation['source_after']
isolate = json.loads((E / f'pc_isolate_{args.prepare_label}.json').read_text())['isolate']
own_compilers = []
other_training = False
for directory in Path('/proc').iterdir():
    if not directory.name.isdigit():
        continue
    try:
        name = (directory / 'comm').read_text().strip()
        command = (directory / 'cmdline').read_bytes()
        other_training = other_training or b'komap.train' in command
        if name in ('cargo', 'rustc') and command and (str(ROOT).encode() in command or (directory / 'cwd').resolve() == ROOT):
            own_compilers.append(int(directory.name))
    except (FileNotFoundError, PermissionError, ProcessLookupError):
        pass
assert not own_compilers, 'Finish our compilation and functional checks before recording performance'
r05_target = Path(f'/tmp/u07m-r05-target{args.label}') if args.phase == 'r05' else None
if r05_target is not None:
    assert not r05_target.exists(), 'R05 requires a never-used target; preserve failed/shared caches and select a new label'
environment = E / f'PERFORMANCE_ENVIRONMENT_{args.label}_{args.phase}.json'
with environment.open('x') as stream:
    json.dump({'recorded_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
               'source_sha256': source['sha256'], 'memory_info': Path('/proc/meminfo').read_text(),
               'other_training_present': other_training, 'own_cargo_rustc_active': False,
               'phase': args.phase, 'prepare_label': args.prepare_label,
               'fresh_r05_target': str(r05_target) if r05_target else None,
               'r05_target_existed_before_measurement': False if r05_target else None,
               'timing_claim': 'Shared-host raw observations only; allocation observer is process/thread scoped. PC gameplay, typed refusal and full non-timing parity are checked separately; inherited R05 allocation instrumentation disables its latency gate. No numeric ceiling changes.'}, stream, indent=2)
    stream.write('\n')


def run(name, command, allowed=(0,)):
    print('Starting ' + name, flush=True)
    result = subprocess.run([sys.executable, str(HERE / 'tools/run.py'), name, '--', *map(str, command)])
    assert result.returncode in allowed, (name, result.returncode)


if args.phase == 'pc':
    raw = E / f'pc_raw_current_{args.label}.json'
    run(f'compat_pc_{args.label}', ['env', 'CARGO_TARGET_DIR=/tmp/wp02-d07-b-pc-target',
        f'R06_BASELINE_OUTPUT={raw}', 'R06_BASELINE_SAMPLES=11', 'cargo', 'test', '--jobs=1',
        '--manifest-path', isolate + '/Cargo.toml', '-p', 'game_core', '--lib', '--release',
        '--offline', '--locked', 'pc_load_baseline', '--', '--ignored', '--nocapture', '--test-threads=1'])
    run(f'compat_pc_gameplay_{args.label}', ['python3', ROOT / 'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/compare_pc_v32.py', raw, E / f'PC_SUMMARY_CURRENT_{args.label}.json'])
else:
    sys.path.insert(0, str(C))
    from r05_harness import verify
    r05_preparation = json.loads((HERE / f'evidence/compat_r05_prepare_{args.r05_prepare_label}.json').read_text())
    assert r05_preparation['exit_code'] == 0 and r05_preparation['source_before'] == source == r05_preparation['source_after']
    proof_path, proof = verify(args.r05_prepare_label, source['sha256'])
    assert args.r05_prepare_label == args.label, 'Bind the measured run and exact harness proof label'
    run(f'compat_r05_{args.label}', ['env', f'CARGO_TARGET_DIR=/tmp/u07m-r05-target{args.label}',
        'cargo', 'run', '--jobs=1', '--manifest-path', Path(proof['isolate']) / 'runtime_architecture_final_validation/Cargo.toml',
        '--bin', 'final_performance', '--release', '--features', 'allocation-observer', '--offline', '--locked', '--',
        ROOT / 'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/evidence/P01_LOG_V32_BASELINE_001.json', E / f'r05_raw_current_{args.label}.json'], (0, 101))
    run(f'compat_r05_summary_{args.label}', ['python3', C / 'summarize_r05_current.py', args.label])
    run(f'compat_r05_dependencies_{args.label}', ['python3', HERE / 'tools/bind_r05_execution.py', args.label])
