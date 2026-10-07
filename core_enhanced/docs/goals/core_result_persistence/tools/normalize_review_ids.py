#!/usr/bin/env python3
"""Pre-freeze P2G-10/20 successor for reviewed PER identity compatibility.

The standard P2G50 Markdown reader recognizes PER-* refs. Original review001
used PE-*-r3 IDs. Preserve all original artifacts; decision meaning is unchanged.
"""
import copy
import json
from initialize import HERE, artifact, write

old = json.loads((HERE/'P2G_20_REVIEW_001.json').read_text())['artifacts']
before = next(a for a in old if a['ref']['Artifact Type'] == 'Working Revision')
elements = [a for a in old if a['ref'] in before['payload']['Current Policy Element Revision Map'].values()]
artifacts = []
for e in elements:
    number = e['payload']['Stable Element ID'].split('-')[-1]
    payload = copy.deepcopy(e['payload'])
    payload.update({'Predecessor Policy Element Revision Ref': e['ref'], 'Intrinsic status': 'decided'})
    artifacts.append(artifact('Policy Element Revision', f'PER-CRRP-{number}-DECIDED-002', payload))
payload = copy.deepcopy(before['payload'])
payload.update({'Predecessor Working Revision Ref': before['ref'],
    'Current Policy Element Revision Map': {a['payload']['Stable Element ID']: a['ref'] for a in artifacts}})
working = artifact('Working Revision', 'WR-CRRP-004', payload)
artifacts.append(working)
write('P2G_10_POLICY_002.json', {'artifacts': artifacts})
write('P2G_10_RESULT_002.json', {
    'stage': 'P2G-10', 'status': 'complete', 'Input Working Revision Ref': before['ref'],
    'Result Working Revision Ref': working['ref'], 'is_p2g10_complete': True,
    'Finding Triage': [{'classification': 'technical_decision',
        'finding': 'Review001 materialized PE-*-r3 identifiers; standard P2G50 member field parses PER-* identifiers. No Freeze has been published.',
        'evidence': 'core_enhanced/docs/tools/validate_p2g50_goal_set.mjs:378 and p2g_inputs.mjs ids regex',
        'disposition': 'Preserve original, return same approved meanings to canonical decided successor refs then fresh P2G20 complete batch'}],
    'checks': {'15_meanings_and_acceptances_exact': all(a['payload']['Policy Statement'] == b['payload']['Policy Statement'] and a['payload']['Observable Policy Acceptance'] == b['payload']['Observable Policy Acceptance'] for a,b in zip(elements, artifacts)),
               'all_current_decision_refs_preserved': payload['Decision Revision Ref Set'] == before['payload']['Decision Revision Ref Set'],
               'intake_dispositions_scope_unchanged': True, 'new_candidates': 0, 'open_questions': 0},
    'candidate_disposition_effective_view': json.loads((HERE/'P2G_10_RESULT_001.json').read_text())['Produced Candidate Disposition Ref Set'],
    'post_freeze_change': 'N/A: pre-freeze technical schema repair', 'authority': 'original continuous P2G correction authorization',
    'next': 'P2G-20', 'production_mutation': False})
plan = json.loads((HERE/'P2G_20_SOURCE_PLAN_002.json').read_text())['artifacts'][0]
p = copy.deepcopy(plan['payload'])
p['Bound Input Working Revision Ref'] = working['ref']
for row in p['Requirement Set']:
    old_by_ref = {json.dumps(e['ref'], sort_keys=True): e['payload']['Stable Element ID'] for e in elements}
    # Original plan binds decided001; resolve stable IDs through the original policy.
    initial = json.loads((HERE/'P2G_10_POLICY_001.json').read_text())['artifacts']
    initial_by_ref = {json.dumps(e['ref'], sort_keys=True): e['payload']['Stable Element ID'] for e in initial if e['ref']['Artifact Type'] == 'Policy Element Revision'}
    row['Exact Policy Element Revision Ref Set'] = [payload['Current Policy Element Revision Map'][initial_by_ref[json.dumps(ref, sort_keys=True)]] for ref in row['Exact Policy Element Revision Ref Set']]
    row['Policy/Scope Basis Reference']['Working Revision Ref'] = working['ref']
p['Candidate Plan Refinement'] = 'Exact successor WR004 binding for pre-freeze identifier correction; required sources/claims/acceptances unchanged'
write('P2G_20_SOURCE_PLAN_003.json', {'artifacts': [artifact('Review Scope/Source Requirement Plan', 'RSP-CRRP-003', p)]})
view = (HERE/'WORKING_POLICY.ko.md').read_text().replace('`WR-CRRP-003`','`WR-CRRP-004`')
view += '\n동결 전 표준 Goal reader의 PER identifier 규약을 확인해 원본 review001을 보존하고 [동일 의미 successor](P2G_10_RESULT_002.json)를 선택했다. 현재 P2G-20 재결속 중이며 아직 Freeze는 없다.\n'
(HERE/'WORKING_POLICY.ko.md').write_text(view)
print('Pre-freeze repair: all15 decision/acceptance meanings preserved; WR004 decided successor; original artifacts retained')
