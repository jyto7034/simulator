#!/usr/bin/env python3
import json
from initialize import HERE, artifact, write
from review import file_row

arts = json.loads((HERE/'P2G_20_REVIEW_002.json').read_text())['artifacts']
def one(kind):
    return next(a for a in arts if a['ref']['Artifact Type'] == kind)
wr, batch, review, basis = [one(k) for k in ['Working Revision','Review Batch','Code Review Artifact','Reviewed Source Basis']]
elements = sorted([a for a in arts if a['ref']['Artifact Type'] == 'Policy Element Revision'], key=lambda a:a['payload']['Stable Element ID'])
bindings = [a for a in arts if a['ref']['Artifact Type'] == 'Code Review Binding']
assert not wr['payload']['Open Policy Question Ref Set']
assert [e['ref'] for e in elements] == batch['payload']['Result Current code_reviewed Policy Element Revision Ref Set'] == list(wr['payload']['Current Policy Element Revision Map'].values())
for e,b in zip(elements,bindings):
    assert e['payload']['Intrinsic status'] == 'code_reviewed'
    assert e['ref']['Artifact ID'].startswith('PER-') and e['payload']['Decision Revision Ref']['Artifact ID'].startswith('DR-')
    assert b['payload']['Bound Working Revision Ref'] == wr['ref']
    assert b['payload']['Policy Element Revision Ref'] == e['ref']
    assert b['payload']['Reviewed Source Basis Ref'] == basis['ref']
    assert b['payload']['Covered Required Claim/Boundary Set']
    assert e['payload']['Decision Revision Ref'] in wr['payload']['Decision Revision Ref Set']
for row in json.loads((HERE/'SOURCE_INDEX_002.json').read_text())['files']:
    assert file_row(row['path']) == row
freeze = artifact('Final Policy Freeze','FPF-CRRP-001',{
    'Working Revision Ref':wr['ref'], 'Review Batch Ref':batch['ref'], 'Predecessor Freeze':None,
    'Ordered code_reviewed Policy Element Revision Ref Set':[e['ref'] for e in elements],
    'Decision Revision Ref Set':[e['payload']['Decision Revision Ref'] for e in elements],
    'Observable Policy Acceptance Set':[{'Policy Element Revision Ref':e['ref'],'Observable Policy Acceptance':e['payload']['Observable Policy Acceptance']} for e in elements],
    'Current Code Review Binding Ref Set':[b['ref'] for b in bindings], 'Reviewed Source Basis Ref':basis['ref'],
    'Binding Completeness Evidence':review['ref'], 'Applicable Change/Obligation Basis':[], 'Resolved Prerequisite Evidence':[],
    'Known Unknowns':'Implementation execution evidence remains pending V01–15. Exact helper names/fixture parameters within TD01–08 are GEX-owned. No unknown required policy/source boundary; no deployed/Windows-crash claim.',
    'Compatibility/Migration Disposition':'U04 immediate settlement/ack-only, choice and node rollback, stable Continue and separate SavePoint, active/Core successor codecs and conditional journal publication change together. Unsupported V4/V9/unsupported profile originals preserved/refused; no mandatory legacy conversion. Compatible permanent profile V3 retained. U03 fault gates/private candidate and native resource contracts preserved. No future schema preaddition.',
    'Lifecycle Disposition':'current'})
current = {'Working Revision Ref':wr['ref'],'Final Policy Freeze Ref':freeze['ref'],'Phase Plan Ref':None,'Goal Set Ref':None,'Handoff Artifact Ref':None,'Post-Freeze Change Record Ref Set':[],'Authoritative Change Obligation Declaration Ref Set':[]}
write('P2G_30_FREEZE_001.json', {'artifacts':[freeze]})
write('P2G_30_RESULT_001.json', {'stage':'P2G-30','status':'complete','Final Policy Freeze Ref':freeze['ref'],
    'Freeze Membership Set':[{'Final Policy Freeze Ref':freeze['ref'],'Policy Element Revision Ref':e['ref']} for e in elements],
    'P2G-30 Resolution Record Ref Set':[], 'current_basis':current,'All Applicable Change Obligations Resolved':True,
    'checks':{'member_sets_exact':True,'acceptance_projection_exact':True,'binding_coverage_complete':True,'70_source_files_current':True,'no_open_question':True,'standard_PER_DR_identifiers':True},
    'Current Planning Freeze Selection':freeze['ref'],'next':'P2G-40'})
p = HERE/'WORKING_POLICY.ko.md'
p.write_text(p.read_text()+'\n현재 planning Freeze는 [FPF-CRRP-001](P2G_30_FREEZE_001.json)이다. 15 exact member/decision/acceptance와 result-basis binding을 검증했다. [P2G-30 결과](P2G_30_RESULT_001.json). 다음은 P2G-40이다.\n')
print('U04 P2G-30 complete: exact15 members/decisions/acceptances/bindings;70 source files current')
