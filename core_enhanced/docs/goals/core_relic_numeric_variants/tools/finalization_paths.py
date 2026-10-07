"""Explicit, immutable U08 successors. No inferred labels or historical defaults."""
import re
from run import HERE

S = HERE / 'execution/single_goal'
C = S / 'compatibility/current_001'


def paths(config):
    revision = config['revision']
    target = config['target']
    assert re.fullmatch(r'[0-9]{3}', revision), revision
    assert re.fullmatch(r'u08_[a-z0-9_]+', target), target
    result = {key: S / (stem + '_' + revision + '.json') for key, stem in {
        'selection': 'EVIDENCE_SELECTION', 'impact': 'PREDECESSOR_IMPACT',
        'coverage': 'COVERAGE', 'gex': 'GEX10_RESULT'}.items()}
    result['adaptation'] = C / ('ADAPTATION_FINAL_' + revision + '.json')
    result['target'] = S / 'target' / target
    return result
