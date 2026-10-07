"""Bind all 181 inherited acceptances and the separate 31 R06 requirements."""
import copy
from evidence_common import ROOT, HERE, S, read, write, ref, selected, passed_tests, bind

selection = selected()
suites = passed_tests(selection)
prior = ROOT / 'core_enhanced/docs/goals/core_independent_objects/execution/single_goal'
old = read(prior / 'PREDECESSOR_IMPACT_002.json')
coverage = read(prior / 'COVERAGE_002.json')
assert len(old['conditions']) == 161 and len(old['r06_acceptances']) == 31 and len(coverage['acceptances']) == 20
rows = []
for original in old['conditions']:
    selectors = original['current_selectors']
    rows.append({'id':original['id'], 'requirement':original['requirement'],
                 'predecessor_row':dict(ref(prior/'PREDECESSOR_IMPACT_002.json'), id=original['id']),
                 'original_meaning_basis':original.get('original_meaning_basis'),
                 'previous_semantic_adjustment':original['current_impact'],
                 'current_selectors':selectors, 'selector_receipts':bind(selectors,suites),
                 'direct_receipts':selection['public_receipts'] if not selectors else [],
                 'disposition':'Affected shared settings/native/observation/host consumers re-executed in current integrated suites. Original gameplay, save, input, numeric and default-budget meanings retained; separately selected development emergency profile is an explicit new trusted mode, not a rewritten historical oracle.',
                 'default_cost_receipts':['compat_pc_gameplay_'+selection['labels']['pc'],'compat_pc_cost_'+selection['labels']['pc']],
                 'development_difference':'Only immutable host settings can select the finite 16/32MiB retained ceiling. Public raw overrides remain strict8MiB; explicit small owner limits stay effective. Work/clone/expression/cumulative sticky refusal, ordering and persistence authority remain unchanged.'})
for index, original in enumerate(coverage['acceptances'],1):
    validations = [v for v in coverage['validations'] if v['id'] in original['validation_refs']]
    selectors = sorted({name for v in validations for kind in ['core_selectors','host_selectors'] for name in v[kind]})
    rows.append({'id':f'PE-CIOB-{index:02d}', 'requirement':original['Observable Policy Acceptance'],
                 'policy_ref':original['Policy Element Revision Ref'],
                 'predecessor_row':dict(ref(prior/'COVERAGE_002.json'), validation_refs=original['validation_refs']),
                 'current_selectors':selectors, 'selector_receipts':bind(selectors,suites),
                 'direct_receipts':[name for name in selection['public_receipts'] if any(k in name for k in ['objects','combat','area','schema','continuation','rewind'])],
                 'disposition':'U07 object ownership and the audited F01 blocking transition correction remain unchanged. Their exact inherited selectors and normal public fixtures are re-executed through the current profile/native/observer integration. Development samples preserve the same normal object results; no new object/save authority.'})
assert len(rows) == 181 and len({row['id'] for row in rows}) == 181
r06 = copy.deepcopy(old['r06_acceptances'])
for row in r06:
    assert row['iva'] and set(row['iva']) <= {r['id'] for r in rows}
    row['status'] = 'Original requirement and IVA meaning retained; exact current inherited selectors and immediate U07 PC non-timing equality are bound. Independent review pending.'
write(S/'PREDECESSOR_IMPACT.json',{'goal':'GOAL-CMRP-01','source_sha256':selection['source_sha256'],
      'conditions':rows,'r06_acceptances':r06,'original_requirement_sources':old['original_requirement_sources'],
      'predecessor_coverage':ref(prior/'COVERAGE_002.json'),'predecessor_impact':ref(prior/'PREDECESSOR_IMPACT_002.json'),
      'predecessor_complete_audit':ref(ROOT/'core_enhanced/docs/goals/core_independent_objects/post_audit/u07_20261005_002/AUDIT_CLOSURE.json'),
      'selection':ref(S/'EVIDENCE_SELECTION.json'),'independent_audit':False})
print('PASS181 inherited conditions and31 separate R06 acceptances')
