"""Close self-verification only; independent Complete/High is a later artifact."""
from evidence_common import (read, write, ref, load_config, result_base, selected,
    load_stage, identity, nonempty)


def main():
    cfg, current, names = load_config()
    selected(cfg, current, names)
    coverage = load_stage('coverage', cfg, current, names)
    for key, field in [('selection','selection'),('impact','predecessor_impact'),('adaptation','compatibility_adaptation')]:
        assert coverage[field] == ref(names[key])
    assert coverage['counts'] == {'predecessor_conditions':191,'u08_groups':16,'main_conditions':207,
        'u08_acceptance_sentences':32,'r06_separate':31,'dimensions':10,'nodes':3,'integration_seams':16}
    assert coverage['pending_completion_sentences'] == ['PE-CRNV-16.2']
    from evidence_common import ROOT, resolve
    review_path = resolve(cfg['self_verification_review_path'])
    review = ref(review_path)
    review_value = read(review_path)
    assert review_value['source_sha256'] == current['sha256']
    assert review_value['config'] == cfg['_ref']
    for key in ('selection','impact','adaptation','coverage'):
        assert review_value[key] == ref(names[key]), 'Review must bind exact finalized predecessors'
    assert review_value['open_product_or_validation_findings'] == []
    assert review_value['independent_auditor'] is False
    assert review_value['coverage_reviewed'] is True
    nonempty(review_value['reviewer'])
    assert review_value['actual_validation_results']
    for row in review_value['actual_validation_results']:
        nonempty(row['check'])
        nonempty(row['observed'])
        assert row['outcome'] == 'satisfied'
        assert row['evidence']
        from evidence_common import check_ref
        for item in row['evidence']:
            check_ref(item)
    assert identity() == current
    write(names['gex'], dict(result_base(cfg, current), goal_set='GS-CRNV-001', phase='PHASE-CRNV-01',
        state='Implementation and current self-verification complete; freeze and independent GEX20 pending',
        goal_status='Not independently Complete', selection=ref(names['selection']),
        coverage=ref(names['coverage']), predecessor_impact=ref(names['impact']),
        compatibility_adaptation=ref(names['adaptation']), self_verification_review=review,
        execution_nodes=coverage['execution_nodes'], counts=coverage['counts'],
        pending_completion_sentences=['PE-CRNV-16.2'],
        next={'target':cfg['target'], 'required_steps':['Update PLAN and EXPERIMENT_NOTES to these exact results',
            'Freeze source, records and exact external references; isolated forward and reverse restoration',
            'Implementation-nonparticipant GPT-6 Astra / High whole-U08 audit'],
            'completion_requires':'Exact target-linked independent Complete / High / open0 closure; no U09 before it'},
        self_awarded_independent_verdict=False))
    print('GEX10 self-verification recorded; no independent completion verdict issued')


if __name__ == '__main__':
    main()
