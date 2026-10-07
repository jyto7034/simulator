"""Read-only preserved U07 input declarations; successor outputs only."""
import importlib.util
from pathlib import Path
from initialize import ROOT, HERE
_path = Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_independent_objects/tools/prepare_public_inputs.py')
_spec = importlib.util.spec_from_file_location('_preserved_u07_public_inputs', _path)
_module = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_module)
OUT = HERE / 'execution/single_goal/compatibility/current_001'
INPUTS = Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_independent_objects/execution/single_goal/compatibility/current_001/inputs')
PACKS, E, sha, spans = _module.PACKS, _module.E, _module.sha, _module.spans
