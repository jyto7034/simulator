"""Explicit immutable successors for finalization artifacts and their consumers."""
from run import HERE


def paths(revision='001'):
    assert revision.isdigit() and len(revision) == 3
    suffix = '' if revision == '001' else '_' + revision
    names = {key: base + suffix + '.json' for key, base in {
        'selection': 'EVIDENCE_SELECTION', 'impact': 'PREDECESSOR_IMPACT',
        'coverage': 'COVERAGE', 'service': 'SERVICE_REUSE', 'gex': 'GEX_10_RESULT',
    }.items()}
    return HERE / 'execution/single_goal', names
