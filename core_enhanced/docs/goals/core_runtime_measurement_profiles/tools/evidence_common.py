"""Exact evidence/currentness checks shared by the U07-M finalizers."""
import json
import re
from run import ROOT, HERE, identity, sha

S = HERE / 'execution/single_goal'
C = S / 'compatibility/current_001'


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write('\n')


def ref(path):
    assert path.is_file(), path
    return {'path': str(path.relative_to(ROOT)), 'sha256': sha(path), 'bytes': path.stat().st_size}


def currentness(record_source, current, *, cli=False):
    if record_source == current:
        return {'kind': 'exact current source'}
    # The only permitted ordinary-suite rebase is the separately captured
    # canonical README publication. Full row equality proves every other input.
    proof_path = HERE / 'evidence/docs_delta_001/PROOF.json'
    proof = read(proof_path)
    assert proof['source_after'] == current and proof['product_changed'] == ['game_server/config/observability/README.md']
    before = proof['source_before']
    if record_source == before:
        return {'kind': 'document-only non-impacting delta', 'proof': ref(proof_path)}
    assert cli, 'Unapproved source rebase'
    host_proof_path = HERE / 'evidence/host_exit_correction_002/PROOF.json'
    host_proof = read(host_proof_path)
    assert host_proof['source_after'] == before and host_proof['source_before'] == record_source
    assert host_proof['changed_product_paths'] == ['game_server/src/main.rs']
    # This exception is scoped to the measurement CLI, whose binary, library,
    # Core, profiles and inputs exclude the server executable's shutdown main.
    return {'kind': 'measurement CLI reused across server-main and documentation delta',
            'proofs': [ref(host_proof_path), ref(proof_path)]}


def receipt(name, current, *, r05=False, cli=False):
    path = HERE / 'evidence' / (name + '.json')
    value = read(path)
    assert value['exit_code'] in ((0, 101) if r05 else (0,)), name
    assert value['source_unchanged'] and value['source_before'] == value['source_after'], name
    assert value['log_sha256'] == sha(path.with_suffix('.log')), name
    binding = dict(ref(path), name=name, exit_code=value['exit_code'],
                   actual_source_sha256=value['source_before']['sha256'],
                   currentness=currentness(value['source_before'], current, cli=cli))
    for item in value['command_inputs']:
        assert sha(HERE / item['archive']) == item['sha256'], item
    runner = HERE / 'evidence/runner_versions' / (value['runner_sha256'] + '.py')
    assert sha(runner) == value['runner_sha256']
    return binding


def selected():
    value = read(S / 'EVIDENCE_SELECTION.json')
    assert value['source_sha256'] == identity()['sha256']
    for binding in value['receipt_bindings']:
        assert sha(ROOT / binding['path']) == binding['sha256']
    return value


def passed_tests(selection):
    suites = {}
    for name in selection['suites']:
        log = HERE / 'evidence' / (name + '.log')
        suites[name] = set(re.findall(r'^test (\S+) \.\.\. ok$', log.read_text(), re.M))
        assert suites[name], name
    return suites


def bind(selectors, suites):
    result = {name: [suite for suite, names in suites.items() if name in names] for name in selectors}
    assert all(result.values()), [name for name, rows in result.items() if not rows]
    return result
