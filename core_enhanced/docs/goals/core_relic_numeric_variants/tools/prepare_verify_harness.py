"""Create U08 successor harnesses; preserve U07-M bytes and every adaptation."""
import difflib
import json
from pathlib import Path
import sys
from run import ROOT, HERE, identity, sha

label = sys.argv[1]
assert label.isdigit()
prior = ROOT / 'core_enhanced/docs/goals/core_runtime_measurement_profiles'
relative_c = Path('execution/single_goal/compatibility/current_001')
proof = HERE / 'evidence' / ('verify_harness_' + label)
proof.mkdir()
source = identity()
rows = []

def publish(relative, text=None):
    original, path = prior / relative, HERE / relative
    old = original.read_bytes()
    data = old if text is None else text.encode()
    assert not path.exists(), path
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    archived = proof / relative
    archived.parent.mkdir(parents=True, exist_ok=True)
    archived.write_bytes(old)
    archived.with_suffix(archived.suffix + '.diff').write_text(''.join(difflib.unified_diff(
        old.decode().splitlines(True), data.decode().splitlines(True), fromfile=str(original), tofile=str(path))))
    rows.append({'path':str(path.relative_to(ROOT)), 'sha256':sha(path),
                 'original':str(original.relative_to(ROOT)), 'original_sha256':sha(original)})

tools = ['integrated_matrix', 'public_matrix', 'release_matrix', 'validate_public_inputs',
         'public_starter_schema', 'public_schema_probe', 'public_independent_area_probe',
         'public_continuation_probe', 'public_rewind_probe', 'public_object_schema',
         'public_objects', 'public_object_combat', 'public_cost_guards', 'final_fault_probes',
         'public_fault_probe', 'prepare_public_inputs', 'bind_r05_execution',
         'prepare_playback_inputs', 'playback_matrix', 'sample_lifecycle', 'sample_host', 'local_websocket']
for name in tools:
    relative = Path('tools') / (name + '.py')
    text = (prior / relative).read_text()
    text = text.replace('core_runtime_measurement_profiles', 'core_relic_numeric_variants')
    text = text.replace('/tmp/u07m-r05-target', '/tmp/u08-r05-target')
    if name == 'public_matrix':
        text = text.replace('commands = [', "from release_gate import require\nrequire('release')\n\ncommands = [")
        text = text.replace("    (f'compat_public_cli_{label}',", "    (f'public_relics_{label}', ['python3', H / 'tools/public_relics.py', label]),\n    (f'compat_public_cli_{label}',")
    elif name == 'release_matrix':
        text = text.replace('import subprocess', 'import os\nimport subprocess', 1)
        text = text.replace('commands = [', "os.environ['U08_RELEASE_LABEL'] = label\nfrom release_gate import bind, require\ncommands = [")
        text = text.replace("    print('Starting ' + name, flush=True)", "    if index > 0:\n        require('release')\n    print('Starting ' + name, flush=True)")
        text += "\n    if index == 0:\n        bind('release', label)\n"
    elif name in ['prepare_playback_inputs', 'playback_matrix', 'sample_lifecycle', 'sample_host']:
        text = text.replace('from run import ROOT, HERE, identity, sha', "from run import ROOT, HERE, identity, sha\nfrom release_gate import require\nrequire('debug')")
    elif name.startswith('public_') or name in ['validate_public_inputs', 'final_fault_probes']:
        # All direct public consumers require the same freshly built release.
        text = "from release_gate import require\nrequire('release')\n" + text
    publish(relative, text)

for name in ['probe', 'boundary', 'default_parity', 'public_probe', 'public_trace_probe',
             'public_cli', 'public_profile_probe', 'public_boundary_probe',
             'prepare_r05', 'r05_harness', 'summarize_r05_current']:
    relative = relative_c / (name + '.py')
    text = (prior / relative).read_text()
    text = text.replace('core_runtime_measurement_profiles', 'core_relic_numeric_variants')
    text = text.replace('u07m-r05-harness-', 'u08-r05-harness-')
    if name == 'probe':
        text = text.replace("'game_server/runtime-observation'", "'game_server/runtime-measurement'")
        text = text.replace("['cargo', 'build',", "['cargo', 'build', '--jobs=1',")
        text = text.replace("assert build.returncode == 0, build.stderr[-2000:]", "assert build.returncode == 0, build.stderr[-2000:]\nrequire('release')")
    if name not in ['prepare_r05', 'r05_harness', 'summarize_r05_current']:
        text = ("import sys\nfrom pathlib import Path\nsys.path.insert(0, str(Path(__file__).resolve().parents[4] / 'tools'))\n"
                "from release_gate import require\nrequire('release')\n") + text
    publish(relative, text)

for relative in [relative_c / 'host_peak_failure_probe.rs', relative_c / 'PREDECESSOR_ADAPTATION_001.json',
                 Path('evidence/observation_fault_001.ron')]:
    publish(relative)
assert source == identity()
(proof / 'ADAPTATION.json').write_text(json.dumps({'passed':True, 'source_sha256':source['sha256'], 'rows':rows,
    'scope':'U07-M successor output namespace, exact historical inputs, fresh source/binary gates, U08 public comparator insertion. No workload, numeric, cap, refusal or normalization changes.'}, indent=2) + '\n')
print('PASS prepared harness only; no build, suite, public or performance execution')
