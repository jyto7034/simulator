"""Successor outputs over immutable U07 public inputs; preserve every preimage."""
import difflib
import json
from pathlib import Path
import sys
from run import ROOT, HERE, identity, sha

label = sys.argv[1]
assert label.isdigit()
prior = ROOT / 'core_enhanced/docs/goals/core_independent_objects'
current = HERE / 'execution/single_goal/compatibility/current_001'
old_current = prior / 'execution/single_goal/compatibility/current_001'
proof = HERE / 'evidence' / ('verify_harness_' + label)
proof.mkdir()
source = identity()
rows = []

def publish(relative, original, transformed):
    path = HERE / relative
    assert not path.exists(), path
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('xb') as f:
        f.write(transformed)
    if original is not None:
        archived = proof / relative
        archived.parent.mkdir(parents=True, exist_ok=True)
        with archived.open('xb') as f:
            f.write(original.read_bytes())
        delta = archived.with_suffix(archived.suffix + '.diff')
        with delta.open('x') as f:
            f.write(''.join(difflib.unified_diff(original.read_text().splitlines(True), transformed.decode().splitlines(True), fromfile=str(original), tofile=str(path))))
    rows.append({'path': str(path.relative_to(ROOT)), 'sha256': sha(path), 'original': str(original.relative_to(ROOT)) if original else None,
                 'original_sha256': sha(original) if original else None})

initializer = HERE / 'tools/initialize.py'
assert "if __name__ == '__main__':" in initializer.read_text()
rows.append({'path': str(initializer.relative_to(ROOT)), 'sha256': sha(initializer),
             'role': 'existing P2G initializer declarations, main guard inactive; preserved without edits'})
tools = ['integrated_matrix', 'public_matrix', 'release_matrix', 'validate_public_inputs', 'public_starter_schema',
         'public_schema_probe', 'public_independent_area_probe', 'public_continuation_probe', 'public_rewind_probe',
         'public_object_schema', 'public_objects', 'public_object_combat', 'public_cost_guards', 'final_fault_probes', 'public_fault_probe']
for name in tools:
    original = prior / 'tools' / (name + '.py')
    text = original.read_text()
    if name in ['public_objects', 'public_object_combat']:
        text = text.replace("inputs=HERE/", "inputs=Path('" + str(prior) + "')/")
        if 'from pathlib import Path' not in text:
            text = 'from pathlib import Path\n' + text
    elif name == 'public_independent_area_probe':
        text = text.replace("fixture=HERE/", "fixture=Path('" + str(prior) + "')/")
        text = 'from pathlib import Path\n' + text
    if name == 'integrated_matrix':
        text = text.replace("('host_observation',", "('host_measurement',['cargo','test','-p','game_server','--all-targets','--features','runtime-measurement','--offline','--locked','--','--test-threads=8']),\n('host_observation',")
        text = text.replace("'game_server/runtime-observation','--offline'", "'game_server/runtime-measurement','--offline'")
        text = text.replace("('clippy_host',['cargo','clippy','-p','game_server','--all-targets','--features','runtime-observation'", "('clippy_host',['cargo','clippy','-p','game_server','--all-targets','--features','runtime-measurement'")
    if name == 'public_cost_guards':
        text = text.replace(b'/core_independent_objects/'.decode(), '/core_runtime_measurement_profiles/')
    if name == 'release_matrix':
        text = text.replace("'game_server/runtime-observation'", "'game_server/runtime-measurement'")
    publish(Path('tools') / (name + '.py'), original, text.encode())

# Import only the preserved pure declarations/functions. Its main guard keeps
# migration generation inactive; all fixture bytes remain in the U07 owner.
declarations = f'''"""Read-only preserved U07 input declarations; successor outputs only."""
import importlib.util
from pathlib import Path
from initialize import ROOT, HERE
_path = Path({str(prior / 'tools/prepare_public_inputs.py')!r})
_spec = importlib.util.spec_from_file_location('_preserved_u07_public_inputs', _path)
_module = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_module)
OUT = HERE / 'execution/single_goal/compatibility/current_001'
INPUTS = Path({str(old_current / 'inputs')!r})
PACKS, E, sha, spans = _module.PACKS, _module.E, _module.sha, _module.spans
'''
publish(Path('tools/prepare_public_inputs.py'), None, declarations.encode())

for name in ['probe', 'boundary', 'default_parity', 'public_probe', 'public_trace_probe', 'public_cli', 'public_profile_probe', 'public_boundary_probe']:
    original = old_current / (name + '.py')
    text = original.read_text()
    text = text.replace(str(prior / 'tools'), str(HERE / 'tools'))
    # Output assignments alone move. Literal historical /inputs references and
    # baseline archive owners are deliberately not globally replaced.
    text = text.replace("run.HERE = Path('" + str(old_current) + "')", "run.HERE = Path('" + str(current) + "')")
    text = text.replace("HERE = Path('" + str(old_current) + "')", "HERE = Path('" + str(current) + "')")
    text = text.replace("E=Path('" + str(old_current) + "')", "E=Path('" + str(current) + "')")
    text = text.replace("with (Path('" + str(old_current) + "') / f\"evidence/DEFAULT", "with (Path('" + str(current) + "') / f\"evidence/DEFAULT")
    if name == 'probe':
        text = text.replace("D = Path('" + str(prior) + "')", "D = Path('" + str(HERE) + "')")
    publish(Path('execution/single_goal/compatibility/current_001') / (name + '.py'), original, text.encode())
for name in ['host_peak_failure_probe.rs', 'PREDECESSOR_ADAPTATION_001.json']:
    original = old_current / name
    publish(Path('execution/single_goal/compatibility/current_001') / name, original, original.read_bytes())
original = prior / 'evidence/observation_fault_001.ron'
publish(Path('evidence/observation_fault_001.ron'), original, original.read_bytes())
assert source == identity()
with (proof / 'ADAPTATION.json').open('x') as f:
    json.dump({'passed': True, 'source_sha256': source['sha256'], 'rows': rows,
               'scope': 'Output namespace, bounded measurement feature suites and exact historical input references. No fixture/oracle/cap/numeric guard changes; original scripts/preimages/diffs retained.'}, f, indent=2)
print('PASS prepared current verification harness; not an execution of its suites')
