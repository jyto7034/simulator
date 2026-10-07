"""Bind immutable predecessor inputs and actual U08 harness bytes to CURRENT."""
from evidence_common import (ROOT, HERE, read, write, ref, check_ref, load_config,
    result_base, selected, load_stage, archive_exact, nonempty, identity)
from run import sha
from pathlib import Path


def main():
    cfg, current, names = load_config()
    selection = selected(cfg, current, names)
    load_stage('impact', cfg, current, names)
    original_manifest = HERE / 'evidence/gex00_002/HARNESS_INPUT_BEFORE.json'
    original_archive = HERE / 'evidence/gex00_002/HARNESS_INPUT_BEFORE.tar.gz'
    original_rows = read(original_manifest)
    assert len(original_rows) == 129
    archive_exact(original_archive, original_rows)
    for row in original_rows:
        check_ref(row)
    intake = ROOT / 'core_enhanced/docs/goals/core_independent_objects/execution/single_goal/compatibility/current_001/INTAKE_U07.json'
    original_inputs = read(intake)['unchanged_input_files']
    assert len(original_inputs) == 448
    for row in original_inputs:
        assert sha(ROOT / row['source']) == sha(ROOT / row['copy']) == row['sha256']
    input_validation = read(ROOT / selection['artifacts']['input_validation']['path'])
    assert input_validation['passed'] is True and input_validation['predecessor_inputs_unchanged'] is True
    # Prepared proofs alone cannot establish execution. Each used script must
    # match the runner's archived command input; imported dependencies must be
    # independently hash-bound by a reviewed input manifest.
    prepared = cfg['adaptation']['prepared_manifests']
    assert prepared
    for row in prepared:
        check_ref(row)
    adaptations = {r['path']: r for r in cfg['adaptation']['harnesses']}
    assert len(adaptations) == len(cfg['adaptation']['harnesses']) and adaptations
    executed = []
    historical_reviews = {(r['receipt_id'],r['path']):r for r in cfg['adaptation']['historical_input_reviews']}
    used_reviews = set()
    for receipt in selection['receipt_bindings']:
        for item in receipt['inputs']:
            actual_path = Path(item['path']).resolve()
            assert actual_path.is_relative_to(ROOT) or actual_path.is_relative_to('/tmp')
            path = str(actual_path.relative_to(ROOT)) if actual_path.is_relative_to(ROOT) else str(actual_path)
            current_sha = sha(actual_path)
            if path.endswith('.py'):
                assert path in adaptations, ('Unreviewed executed Python tool', path)
                adaptation = adaptations[path]
                assert adaptation['current']['sha256'] == current_sha
                check_ref(adaptation['current'])
                assert adaptation['current']['path'] == path
                nonempty(adaptation['reason'])
                assert adaptation['evidence']
                for row in adaptation['evidence']:
                    check_ref(row)
                if adaptation['predecessor'] is not None:
                    old_path = check_ref(adaptation['predecessor'])
                    patch = check_ref(adaptation['diff'])
                    import difflib
                    actual_diff = ''.join(difflib.unified_diff(old_path.read_text().splitlines(True),
                        (ROOT / path).read_text().splitlines(True), fromfile=adaptation['predecessor']['path'], tofile=path))
                    assert patch.read_text() == actual_diff, 'Adaptation diff must contain exact predecessor/current text'
                else:
                    assert adaptation['classification'] == 'new_u08_tool'
            if item['sha256'] != current_sha:
                key = (receipt['id'],path)
                assert key in historical_reviews, 'Changed executed input requires a specific historical review'
                review = historical_reviews[key]
                assert review['executed_archive'] == item['archived_ref']
                assert review['executed_sha256'] == item['sha256'] and review['current_sha256'] == current_sha
                nonempty(review['reason_for_no_effect_on_claimed_selectors'])
                assert review['evidence'] and review['selectors']
                assert set(review['selectors']) <= set(receipt['selectors'])
                for row in review['evidence']:
                    check_ref(row)
                used_reviews.add(key)
            executed.append({'receipt_id':receipt['id'], 'path':path,
                             'executed_sha256':item['sha256'], 'archive':item['archived_ref'],
                             'current_sha256':current_sha})
    # Tool dependencies such as release_gate.py are not all argv inputs.
    dependency_rows = cfg['adaptation']['execution_dependencies']
    global_binding = read(check_ref(selection['execution_harness_binding']))
    globally_bound = {r['id'] for r in global_binding['receipts']}
    python_consumers = {r['id'] for r in selection['receipt_bindings'] if any(i['path'].endswith('.py') for i in r['inputs'])}
    assert {r['receipt_id'] for r in dependency_rows} == python_consumers - globally_bound
    assert used_reviews == set(historical_reviews), 'Do not carry unused blanket historical exceptions'
    for row in dependency_rows:
        manifest = read(check_ref(row['manifest']))
        assert manifest['files'], row['receipt_id']
        nonempty(row['execution_binding_reason'])
        for dependency in manifest['files']:
            check_ref(dependency)
    r05_proof = read(ROOT / selection['artifacts']['r05_proof']['path'])
    r05_deps = read(ROOT / selection['artifacts']['r05_dependencies']['path'])
    assert r05_deps['passed'] is True and r05_deps['source_sha256'] == current['sha256']
    assert r05_proof['source_sha256'] == current['sha256']
    import re
    import sys
    from finalization_paths import C
    proof_path = ROOT / selection['artifacts']['r05_proof']['path']
    match = re.fullmatch(r'r05_harness_([0-9]+)', proof_path.parent.name)
    assert match, 'Explicit R05 proof path must identify its actual label'
    sys.path.insert(0, str(C))
    from r05_harness import verify
    verified_path, verified_proof = verify(match[1], current['sha256'])
    assert verified_path == proof_path and verified_proof == r05_proof
    assert identity() == current
    write(names['adaptation'], dict(result_base(cfg, current),
        predecessor_impact=ref(names['impact']), selection=ref(names['selection']),
        immediate_unchanged_inputs=original_inputs, immediate_input_count=448,
        historical_preimages=ref(original_manifest), historical_archive=ref(original_archive),
        historical_harness_input_count=129, prepared_manifests=prepared,
        harnesses=cfg['adaptation']['harnesses'], executed_harness_bindings=executed,
        execution_harness_binding=selection['execution_harness_binding'],
        historical_input_reviews=cfg['adaptation']['historical_input_reviews'],
        execution_dependencies=dependency_rows,
        r05_harness=selection['artifacts']['r05_proof'],
        r05_execution_dependencies=selection['artifacts']['r05_dependencies'],
        input_validation=selection['artifacts']['input_validation'],
        claim='Original inputs/oracles/caps retained; exact reviewed U08 tool deltas and actual executions bound. Preparation alone is not a result.'))
    print('Bound CURRENT adaptation after predecessor impact and actual input validation')


if __name__ == '__main__':
    main()
