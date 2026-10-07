"""Bind inherited condition selectors to current successful suite evidence."""
from pathlib import Path
import json
import re
import run

H = run.HERE
G = H.parents[3]
E = H / 'evidence'
SUITES = ['all_default_002', 'all_feature_001']
passed = {}
identities = []
for suite in SUITES:
    receipt = json.loads((E / (suite + '.json')).read_text())
    assert receipt['exit_code'] == 0 and receipt['source_unchanged']
    assert receipt['log_sha256'] == run.sha(E / (suite + '.log'))
    identities.append(receipt['source_after'])
    passed[suite] = set(re.findall(r'^test (\S+) \.\.\. ok$', (E / (suite + '.log')).read_text(), re.M))
assert identities[0] == identities[1] == run.identity()

conditions = json.loads((H.parents[1] / 'COVERAGE.json').read_text())['conditions']
impact = json.loads((G / 'reviews/r08_r12_20261002_001/R06_R07_IMPACT_REVIEW.json').read_text())
selected = []
for condition in conditions:
    selectors = condition['actual_test_selectors']
    missing = {suite:sorted(set(selectors) - found) for suite, found in passed.items()}
    assert not any(missing.values()), (condition['id'], missing)
    selected.append(dict(id=condition['id'], requirement=condition['required_input_and_oracle'], selectors=selectors, receipts=SUITES, disposition='current selectors verified; independent semantic audit pending', directly_affected=condition['id'] in ['A08-03','A08-04','A10-04','A10-06','X10']))
for group in ['R06_integration', 'R07_F01_F12']:
    for condition in impact[group]:
        selectors = condition['actual_selectors']
        assert all(set(selectors) <= found for found in passed.values()), condition['id']
        selected.append(dict(id=condition['id'], group=group, selectors=selectors, receipts=SUITES, disposition='current selectors verified; independent impact audit pending'))
new = sorted(name for name in passed[SUITES[0]] if 'boss_runtime::tests::f01_' in name)
assert len(new) == 7 and set(new) <= passed[SUITES[1]]
result = dict(goal='GOAL-CARX-01', source_digest=identities[0]['digest'], independent_audit=False, inherited_conditions=55, r06_acceptances=impact['R06_acceptances'], conditions=selected, correction_selectors=new, public_oracle='evidence/public_after/SUMMARY.json', public_trace='evidence/public_after_trace/SUMMARY.json', note='Selector success does not itself establish all acceptance semantics. GEX-20 must inspect changed producer/consumer paths and apply unchanged prior condition reasoning.')
with (H / 'COVERAGE_CURRENT.json').open('x') as stream:
    json.dump(result, stream, ensure_ascii=False, indent=2)
print(json.dumps({'goal_conditions':55,'r06_groups':21,'r07_groups':12,'correction_selectors':len(new),'source_digest':identities[0]['digest']}))
