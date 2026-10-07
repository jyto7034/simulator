"""Preserve each of 191 predecessor requirements and the separate 31 R06 rows."""
from evidence_common import (ROOT, read, write, ref, check_ref, digest, load_config,
    result_base, selected, evidence_claim, nonempty, delta, identity)

PREVIOUS = ROOT / 'core_enhanced/docs/goals/core_runtime_measurement_profiles'
CLOSURE_SHA = 'a9fac11bb702a2e484daeb8c34ee04e39d3c30d553cb1c996611c1b7aca4dbb8'
MANIFEST_SHA = '50c3c228d2c8f0915ab6133014bcf7783bb44d3b8880a8ba7c784ffe130a5733'


def main():
    cfg, current, names = load_config()
    selection = selected(cfg, current, names)
    closure = PREVIOUS / 'post_audit/u07m_20261006_001/AUDIT_CLOSURE.json'
    manifest = PREVIOUS / 'execution/single_goal/target/u07m_001/MANIFEST.json'
    assert ref(closure)['sha256'] == CLOSURE_SHA
    assert ref(manifest)['sha256'] == MANIFEST_SHA
    prior_impact = PREVIOUS / 'execution/single_goal/PREDECESSOR_IMPACT.json'
    prior_coverage = PREVIOUS / 'execution/single_goal/COVERAGE.json'
    old, coverage = read(prior_impact), read(prior_coverage)
    assert len(old['conditions']) == 181 and len(old['r06_acceptances']) == 31
    assert len(coverage['acceptances']) == 10
    originals = [(r['id'], r, ref(prior_impact), r['current_selectors']) for r in old['conditions']]
    for index, row in enumerate(coverage['acceptances'], 1):
        validations = [v for v in coverage['validations'] if v['id'] in row['validation_refs']]
        selectors = sorted({s for v in validations for s in v['selectors']})
        originals.append((f'PE-CMRP-{index:02d}', row, ref(prior_coverage), selectors))
    configs = cfg['predecessor_conditions']
    assert set(configs) == {key for key, _, _, _ in originals} and len(configs) == 191
    predecessor_source = read(manifest)['product_source']
    actual_delta = delta(predecessor_source, current)
    assert cfg['predecessor_source_delta'] == actual_delta
    conditions = []
    selected_pool = {s for row in selection['receipt_bindings'] for s in row['selectors']}
    for key, original, owner, selectors in originals:
        spec = configs[key]
        assert spec['original_row_sha256'] == digest(original), key
        nonempty(spec['current_impact'])
        assert set(spec['affected_paths']) <= {r['path'] for r in actual_delta}
        if spec['source_relation'] == 'affected':
            assert spec['affected_paths']
            review = None
        else:
            assert spec['source_relation'] == 'reviewed_nonimpact' and spec['affected_paths'] == []
            review = read(check_ref(spec['nonimpact_review']))
            assert review['condition_id'] == key and review['examined_delta'] == actual_delta
            nonempty(review['reason'])
            assert review['evidence']
            for item in review['evidence']:
                check_ref(item)
        assert set(selectors) <= selected_pool, (key, sorted(set(selectors) - selected_pool))
        claims = [evidence_claim(c, selection) for c in spec['claims']]
        assert claims, key
        claimed_selectors = {s for c in claims for s in c['selectors']}
        assert set(selectors) <= claimed_selectors, (key, 'Inherited exact selectors omitted')
        conditions.append({'id':key, 'original_row':original, 'original_row_sha256':digest(original),
                           'original_owner':owner, 'current_impact':spec['current_impact'],
                           'affected_paths':spec['affected_paths'], 'source_relation':spec['source_relation'],
                           'nonimpact_review':spec.get('nonimpact_review'), 'claims':claims,
                           'current_selectors':selectors,
                           'status':'current self-verification evidence; independent review pending'})
    # R06 remains a separate ledger. Preserve the complete original rows without
    # rewriting their oracle or treating them as 31 extra U08 policy groups.
    r06 = []
    assert len(cfg['r06_claims']) == 31
    for original, spec in zip(old['r06_acceptances'], cfg['r06_claims']):
        assert spec['original_row_sha256'] == digest(original)
        assert original['iva'] and set(original['iva']) <= set(configs)
        claims = [evidence_claim(c, selection) for c in spec['claims']]
        assert claims
        r06.append({'original_row':original, 'original_row_sha256':digest(original),
                    'original_owner':ref(prior_impact), 'iva':original['iva'], 'claims':claims})
    for row in old['original_requirement_sources']:
        check_ref(row) if 'bytes' in row else None
        from run import sha
        assert sha(ROOT / row['path']) == row['sha256']
    result = dict(result_base(cfg, current), conditions=conditions, r06_acceptances=r06,
                  original_requirement_sources=old['original_requirement_sources'],
                  predecessor_impact=ref(prior_impact), predecessor_coverage=ref(prior_coverage),
                  predecessor_closure=ref(closure), predecessor_manifest=ref(manifest),
                  actual_source_delta=actual_delta, selection=ref(names['selection']))
    assert identity() == current
    write(names['impact'], result)
    print('Bound all 191 predecessor rows and 31 separate R06 rows without rewriting them')


if __name__ == '__main__':
    main()
