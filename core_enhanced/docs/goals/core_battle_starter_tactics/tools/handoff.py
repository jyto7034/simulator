#!/usr/bin/env python3
import copy,json
from initialize import HERE,ROOT,artifact,write,sha
v=json.loads((HERE/'P2G_50_VALIDATION_001.json').read_text());assert v['result']=='passed' and not v['failures']
plan=json.loads((HERE/'P2G_40_PHASE_PLAN_001.json').read_text())['artifacts'][0]
f=json.loads((HERE/'P2G_30_FREEZE_001.json').read_text())['artifacts'][0]
assert len(f['payload']['Ordered code_reviewed Policy Element Revision Ref Set'])==20
assert plan['payload']['Bound Final Policy Freeze Ref']==f['ref']
assert len(plan['payload']['phases'])==1 and len(plan['payload']['integration'])==20
text=(HERE/'P2G_50_GOAL_SET.ko.md').read_text()
for expected in ['- Authoring Status: Authored','- Activation Status: Not Activated','- Implementation Authorized: No']:assert expected in text
for e in f['payload']['Ordered code_reviewed Policy Element Revision Ref Set']:assert e['Artifact ID'] in text
for r in json.loads((HERE/'evidence/REVIEW_SOURCE_MANIFEST_003.json').read_text())['files']:assert sha(ROOT/r['path'])==r['sha256']
gs={'Artifact Type':'Goal Set','Artifact ID':'GS-CBST-001','Content Digest':'sha256:'+sha(HERE/'P2G_50_GOAL_SET.ko.md')}
basis=copy.deepcopy(json.loads((HERE/'P2G_40_RESULT_001.json').read_text())['current_basis']);basis['Goal Set Ref']=gs
write('P2G_50_RESULT_001.json',{'stage':'P2G-50','status':'complete','Goal Set Ref':gs,'member_ids':['GOAL-CBST-01'],'validator':'P2G_50_VALIDATION_001.json','raw_exit':0,'current_basis':basis,
 'semantic_review':{'all20_frozen_acceptances':True,'six_minimal_standalone_nodes_single_integrated_phase':True,'all20_seams_owned':True,'all10_dimensions_required':True,'141_inherited_and31_R06':True,'basic_heal_and_hostile_exception_scope_preserved':True,'current_corrosion_timing_preserved':True,'supported_restore_and_original_protection':True,'no_implementation_or_independent_completion_claim':True},'P2G-50 Resolution Record Ref Set':[],'next':'P2G-60'})
# P2G-60 uses the validated current upstream exact basis, not duplicated member payloads.
result_basis={k:v for k,v in basis.items() if k!='Handoff Artifact Ref'}
h=artifact('Handoff Artifact','HA-CBST-001',{'Predecessor Handoff Artifact Ref':None,'Result Composite Basis':result_basis,'Final Policy Freeze Ref':f['ref'],'Phase Plan Ref':plan['ref'],'Goal Set Ref':gs,
 'Remaining Gap/Blocker/Limitation Summary':'No required policy/source blocker. TD01–08 exact heal/schema/grant/tactical/contact/restore/DTO realization and V01–20 actual execution remain GEX work. U06 product368 and operating-doc delta baseline reused before mutation;141 inherited+31 R06 impact still required. Prior R05 raw101 approved concurrent-peak exception/20-minute exclusion preserved. No remote/Unity/deployed proof.',
 'Scope/Authorization Boundary':'P2G authoring only; no activation, implementation authorization, production mutation or completion verdict. Original IN-001 continuous request separately authorizes GEX implementation/self-validation, nonparticipant GPT-6 Astra High audit and corrections without renewed handoff approval.',
 'Out-of-Scope Summary':'Unity rendering/camera/ghost, unagreed UI/finalboss/new weapon family, future complete recruitment/expedition or corrosion damage, live-battle disk format, unrelated cleanup/remote deployment, mandatory old-save conversion/automatic original deletion or overwrite.',
 'Lifecycle Disposition':'current'})
write('P2G_60_HANDOFF_001.json',{'artifacts':[h]})
write('P2G_60_RESULT_001.json',{'stage':'P2G-60','status':'complete','Handoff Artifact Ref':h['ref'],'P2G-60 Resolution Record Ref Set':[],
 'checks':{'upstream_exact':True,'single_phase_single_goal':True,'all20_elements_and_seams':True,'no_open_obligation':True,'no_production_mutation':True,'known_limitations_preserved':True},'next':'separately authorized GEX-00 GOAL-CBST-01'})
p=HERE/'WORKING_POLICY.ko.md';p.write_text(p.read_text()+'\n[P2G-50 Goal](P2G_50_GOAL_SET.ko.md) quality2 표준 검증 PASS, [P2G-60 exact 인계](P2G_60_HANDOFF_001.json) 완료. 다음은 원 연속 요청이 별도로 승인한 GEX-00/10 GOAL-CBST-01이다. 제품 구현/독립 감사는 아직 미수행.\n')
print('U06 P2G50/60 complete:GS-CBST-001/GOAL-CBST-01,20acceptances/seams,10dimensions;GEX00 next')
