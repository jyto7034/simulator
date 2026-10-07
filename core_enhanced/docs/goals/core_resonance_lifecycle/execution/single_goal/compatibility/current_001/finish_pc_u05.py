"""Recovery after the preserved PC002 implicit-fixture mismatch.

PC003 is measured separately, after explicit original nonemployee fixture
values are restored. This recipe only compares those immutable raw rows.
"""
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path('/mnt/f/work/simulator')
H = ROOT / 'core_enhanced/docs/goals/core_resonance_lifecycle'
C = Path(__file__).resolve().parent
sys.path.insert(0, str(H / 'tools'))
from run import identity

for name in ('compat_pc_prepare_003', 'compat_pc_003'):
    receipt = json.loads((H / 'evidence' / (name + '.json')).read_text())
    assert receipt['exit_code'] == 0 and receipt['source_before'] == identity() == receipt['source_after']


def run(name, command, expected):
    result = subprocess.run([sys.executable, str(H / 'tools/run.py'), name, '--', *map(str, command)])
    assert result.returncode == expected, (name, result.returncode)


run('compat_pc_compare_003', ['python3', ROOT / 'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/compare_pc_v32.py', C / 'evidence/pc_raw_current_003.json', C / 'evidence/PC_SUMMARY_CURRENT_003.json'], 0)
# Preserve a strict unchanged-cost comparison as a failed result. The next
# comparator must account for only explicit, source-derived cost changes.
run('compat_pc_immediate_002', ['python3', C / 'compare_pc_exact_u05.py'], 1)
run('compat_pc_immediate_003', ['python3', C / 'compare_pc_cost_u05.py'], 0)
