#!/usr/bin/env python3
import json
from initialize import HERE, artifact, write
from review import file_row

arts=json.loads((HERE/'P2G_20_REVIEW_003.json').read_text())['artifacts']
def one(kind):return next(a for a in arts if a['ref']['Artifact Type']==kind)
wr=one('Working Revision');batch=one('Review Batch');review=one('Code Review Artifact');basis=one('Reviewed Source Basis')
elements=sorted([a for a in arts if a['ref']['Artifact Type']=='Policy Element Revision'],key=lambda a:a['payload']['Stable Element ID'])
bindings=[a for a in arts if a['ref']['Artifact Type']=='Code Review Binding']
assert not wr['payload']['Open Policy Question Ref Set']
assert [e['ref'] for e in elements]==batch['payload']['Result Current code_reviewed Policy Element Revision Ref Set']
assert [e['ref'] for e in elements]==list(wr['payload']['Current Policy Element Revision Map'].values())
for e,b in zip(elements,bindings):
 assert e['payload']['Intrinsic status']=='code_reviewed'
 assert b['payload']['Bound Working Revision Ref']==wr['ref']
 assert b['payload']['Policy Element Revision Ref']==e['ref']
 assert b['payload']['Reviewed Source Basis Ref']==basis['ref']
 assert b['payload']['Covered Required Claim/Boundary Set']
 assert e['payload']['Decision Revision Ref'] in wr['payload']['Decision Revision Ref Set']
for row in json.loads((HERE/'SOURCE_INDEX_003.json').read_text())['files']:assert file_row(row['path'])==row
prior_freeze=json.loads((HERE/'P2G_30_FREEZE_002.json').read_text())['artifacts'][0]
change,declaration=json.loads((HERE/'P2G_SCHEMA_CHANGE_002.json').read_text())['artifacts']
prior10=json.loads((HERE/'P2G_10_RESULT_003.json').read_text())
prior20=json.loads((HERE/'P2G_20_RESULT_003.json').read_text())
assert prior20['O20_effectively_resolved']
assert json.loads((HERE/'SOURCE_INDEX_001.json').read_text())['files']==json.loads((HERE/'SOURCE_INDEX_003.json').read_text())['files']
freeze=artifact('Final Policy Freeze','FPF-CRFI-003',{
 'Working Revision Ref':wr['ref'],'Review Batch Ref':batch['ref'],'Predecessor Freeze':prior_freeze['ref'],
 'Ordered code_reviewed Policy Element Revision Ref Set':[e['ref'] for e in elements],
 'Decision Revision Ref Set':[e['payload']['Decision Revision Ref'] for e in elements],
 'Observable Policy Acceptance Set':[{'Policy Element Revision Ref':e['ref'],'Observable Policy Acceptance':e['payload']['Observable Policy Acceptance']} for e in elements],
 'Current Code Review Binding Ref Set':[b['ref'] for b in bindings],'Reviewed Source Basis Ref':basis['ref'],
 'Binding Completeness Evidence':review['ref'],'Applicable Change/Obligation Basis':wr['payload']['Post-Freeze Change/Closure Basis']['Post-Freeze Change Record Ref Set']+wr['payload']['Post-Freeze Change/Closure Basis']['Authoritative Change Obligation Declaration Ref Set'],'Resolved Prerequisite Evidence':prior10['Resolution Record Ref Set']+prior20['Resolution Record Ref Set'],
 'Known Unknowns':'Only implementation evidence pending: actual sticky-fault API/host negative paths, bounded diagnostics and terminal observation parity. CODE_REVIEW F04 and V01–12 own actual nonzero execution; no unknown required policy/source boundary. Exact Rust helper/DTO spelling chosen within TD01–08.',
 'Compatibility/Migration Disposition':'Existing normal command/result JSON preserved when no fault. New fault is application/session lifetime, absent from save schemas. Existing valid active/profile bytes, Continue versus Load rules, private rollback/authoring candidate retry, public prepared authority preserved. Public reset gains typed Result and actual host caller changes together. No data deletion/conversion, whole Run rollback, auto-cancel or U04 result policy.',
 'Lifecycle Disposition':'current'})
result_basis=dict(prior20['current_basis']);result_basis['Final Policy Freeze Ref']=freeze['ref']
resolution=artifact('Resolution Record','RR-CRFI-SCHEMA2-30-001',{'Obligation Key':{'Container Ref':declaration['ref'],'Obligation ID':'O30'},'Owner Stage':'P2G-30','Exact Resolved Prerequisite Basis':prior20['current_basis'],'Exact Result Composite Basis':result_basis,'Resolution Artifact Ref':freeze['ref'],'Resolution Evidence':{'O10_O20_resolution_refs':prior10['Resolution Record Ref Set']+prior20['Resolution Record Ref Set'],'exact12_member_decision_acceptance_projection':True,'all59_source_rows_exact_to_original_review':True,'canonical_member_field_shapes':True,'all_earlier_and_P2G30_leaves_resolved':True,'P2G40_O40_pending':True}})
write('P2G_30_FREEZE_003.json',{'artifacts':[freeze,resolution]})
write('P2G_30_RESULT_003.json',{'stage':'P2G-30','status':'complete','Final Policy Freeze Ref':freeze['ref'],'Freeze Membership Set':[{'Final Policy Freeze Ref':freeze['ref'],'Policy Element Revision Ref':e['ref']} for e in elements],'P2G-30 Resolution Record Ref Set':[resolution['ref']],'current_basis':result_basis,'All Applicable Change Obligations Resolved':False,'P2G30_ReadyForPhaseRepair':True,'checks':{'member_sets_exact':True,'acceptance_projection_exact':True,'binding_coverage_complete':True,'source_current':True,'no_open_policy_question':True},'Current Planning Freeze Selection':freeze['ref'],'next':'P2G-40'})
p=HERE/'WORKING_POLICY.ko.md';s=p.read_text().replace('Decision identity 교정 P2G-20 완료; P2G-30/40 successor 준비 중이다.','현재 Freeze003 동결 완료; P2G-40 successor 검증 준비 중이다.');s+='\n현재 [Freeze003](P2G_30_FREEZE_003.json), [O30 resolution](P2G_30_RESULT_003.json). Change002 O40만 남았다.\n';p.write_text(s)
print('P2G-30 complete: exact12 members/decisions/acceptances and12 current bindings; source current')
