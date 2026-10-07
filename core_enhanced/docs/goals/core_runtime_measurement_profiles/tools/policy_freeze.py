#!/usr/bin/env python3
import json
from initialize import HERE,ROOT,artifact,write,sha
arts=json.loads((HERE/'P2G_20_REVIEW_001.json').read_text())['artifacts']
def one(kind):return next(a for a in arts if a['ref']['Artifact Type']==kind)
wr,batch,review,basis=[one(k) for k in ['Working Revision','Review Batch','Code Review Artifact','Reviewed Source Basis']]
es=sorted([a for a in arts if a['ref']['Artifact Type']=='Policy Element Revision'],key=lambda a:a['payload']['Stable Element ID'])
bs=[a for a in arts if a['ref']['Artifact Type']=='Code Review Binding']
assert not wr['payload']['Open Policy Question Ref Set']
assert [e['ref'] for e in es]==batch['payload']['Result Current code_reviewed Policy Element Revision Ref Set']==list(wr['payload']['Current Policy Element Revision Map'].values())
for e,b in zip(es,bs):
 assert e['payload']['Intrinsic status']=='code_reviewed'
 assert b['payload']['Bound Working Revision Ref']==wr['ref'] and b['payload']['Policy Element Revision Ref']==e['ref']
 assert b['payload']['Reviewed Source Basis Ref']==basis['ref'] and b['payload']['Covered Required Claim/Boundary Set']
 assert e['payload']['Decision Revision Ref'] in wr['payload']['Decision Revision Ref Set']
for r in json.loads((HERE/'evidence/REVIEW_SOURCE_MANIFEST_002.json').read_text())['files']:assert sha(ROOT/r['path'])==r['sha256']
f=artifact('Final Policy Freeze','FPF-CMRP-001',{'Working Revision Ref':wr['ref'],'Review Batch Ref':batch['ref'],'Predecessor Freeze':None,
 'Ordered code_reviewed Policy Element Revision Ref Set':[e['ref'] for e in es],'Decision Revision Ref Set':[e['payload']['Decision Revision Ref'] for e in es],
 'Observable Policy Acceptance Set':[{'Policy Element Revision Ref':e['ref'],'Observable Policy Acceptance':e['payload']['Observable Policy Acceptance']} for e in es],
 'Current Code Review Binding Ref Set':[b['ref'] for b in bs],'Reviewed Source Basis Ref':basis['ref'],'Binding Completeness Evidence':review['ref'],'Applicable Change/Obligation Basis':[],'Resolved Prerequisite Evidence':[],
 'Known Unknowns':'No required policy/source gap. Exact helper/container/runner-format and measured costs remain GEX TD01–06 work. Temporary32MiB is a bounded experiment ceiling, not final deployment tuning.',
 'Compatibility/Migration Disposition':'Embedded default RON/canonical hash, RuntimeFactLimits layout and strict8MiB public validation, default numeric oracles and gameplay/save semantics remain. Optional explicit trusted dev profile affects only native retained ceiling; private entry preserves owner overrides and nested budget. Observation metadata is not gameplay state. Outside-native snapshot local allocation uses its trusted owner cap; outside-native rollback capture retains its original scope. New bounded runner enforces expected profile before mutation; ordinary Continue restores under current trusted host. No new live-battle save codec or retroactive failure waiver.',
 'Lifecycle Disposition':'current'})
current={'Working Revision Ref':wr['ref'],'Final Policy Freeze Ref':f['ref'],'Phase Plan Ref':None,'Goal Set Ref':None,'Handoff Artifact Ref':None,'Post-Freeze Change Record Ref Set':[],'Authoritative Change Obligation Declaration Ref Set':[]}
write('P2G_30_FREEZE_001.json',{'artifacts':[f]})
write('P2G_30_RESULT_001.json',{'stage':'P2G-30','status':'complete','Final Policy Freeze Ref':f['ref'],'Freeze Membership Set':[{'Final Policy Freeze Ref':f['ref'],'Policy Element Revision Ref':e['ref']} for e in es],'P2G-30 Resolution Record Ref Set':[],'current_basis':current,'All Applicable Change Obligations Resolved':True,'checks':{'exact10_members':True,'acceptance_projection_exact':True,'binding_coverage_complete':True,'81_sources_current':True,'no_open_question':True},'Current Planning Freeze Selection':f['ref'],'next':'P2G-40'})
p=HERE/'WORKING_POLICY.ko.md';p.write_text(p.read_text()+'\n현재 planning Freeze는 [FPF-CMRP-001](P2G_30_FREEZE_001.json), exact10 member/decision/acceptance/binding. [P2G-30 결과](P2G_30_RESULT_001.json). 다음은 P2G-40이다.\n')
print('U07 P2G30 complete:10 exact members/decisions/acceptances/bindings;81 current source files')
