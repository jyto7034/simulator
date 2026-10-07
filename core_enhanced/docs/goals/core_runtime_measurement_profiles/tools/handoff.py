"""Close U07-M authored Goal and exact P2G handoff; not product activation."""
import copy,json
from initialize import HERE,ROOT,artifact,write,sha
v=json.loads((HERE/'P2G_50_VALIDATION_002.json').read_text());assert v['result']=='passed' and not v['failures']
plan=json.loads((HERE/'P2G_40_PHASE_PLAN_001.json').read_text())['artifacts'][0]
f=json.loads((HERE/'P2G_30_FREEZE_001.json').read_text())['artifacts'][0]
assert len(f['payload']['Ordered code_reviewed Policy Element Revision Ref Set'])==10
assert plan['payload']['Bound Final Policy Freeze Ref']==f['ref']
assert len(plan['payload']['phases'])==1 and len(plan['payload']['integration'])==10
text=(HERE/'P2G_50_GOAL_SET.ko.md').read_text()
for expected in ['- Authoring Status: Authored','- Activation Status: Not Activated','- Implementation Authorized: No']:assert expected in text
for e in f['payload']['Ordered code_reviewed Policy Element Revision Ref Set']:assert e['Artifact ID'] in text
for r in json.loads((HERE/'evidence/REVIEW_SOURCE_MANIFEST_002.json').read_text())['files']:assert sha(ROOT/r['path'])==r['sha256']
gs={'Artifact Type':'Goal Set','Artifact ID':'GS-CMRP-001','Content Digest':'sha256:'+sha(HERE/'P2G_50_GOAL_SET.ko.md')}
basis=copy.deepcopy(json.loads((HERE/'P2G_40_RESULT_001.json').read_text())['current_basis']);basis['Goal Set Ref']=gs
write('P2G_50_RESULT_001.json',{'stage':'P2G-50','status':'complete','Goal Set Ref':gs,'member_ids':['GOAL-CMRP-01'],'validator':'P2G_50_VALIDATION_002.json','raw_exit':0,'current_basis':basis,
 'semantic_review':{'all10_frozen_acceptances':True,'five_nodes_one_integrated_phase':True,'all10_seams_owned':True,'all10_dimensions_required':True,'181_predecessors_and31_R06':True,'default_preserved_and_trusted_finite_dev_explicit':True,'separate_measurements_bounded_normal_Core_host_required':True,'current_host_restore_and_profile_replay_gate_distinct':True,'no_implementation_or_independent_completion_claim':True},'P2G-50 Resolution Record Ref Set':[],'next':'P2G-60'})
h=artifact('Handoff Artifact','HA-CMRP-001',{'Predecessor Handoff Artifact Ref':None,'Result Composite Basis':{k:v for k,v in basis.items() if k!='Handoff Artifact Ref'},'Final Policy Freeze Ref':f['ref'],'Phase Plan Ref':plan['ref'],'Goal Set Ref':gs,
 'Remaining Gap/Blocker/Limitation Summary':'No required policy/source blocker. TD01–06 exact profile/private entry/outside snapshot/metrics/collector/allocator/RSS/normal runner and V01–10 runtime evidence remain GEX work. U07 target002/audit002 exact baseline plus two publication deltas are inherited. 181 predecessor+31 R06 current impact required. Prior R05 raw101 peak-only warning and20minute exclusion preserved.32MiB is temporary bounded measurement ceiling; no final tuning/Unity/all-content adequacy claim.',
 'Scope/Authorization Boundary':'P2G authoring only; no activation, implementation authorization, production mutation or completion verdict. Original IN-001 continuous request separately authorizes GEX implementation/self-validation by GPT-6.1 Sol/xHigh, nonparticipant GPT-6 Astra/High audit/corrections and roadmap continuation without renewed handoff approval.',
 'Out-of-Scope Summary':'Default cap relaxation or old failure waiver; unbounded/automatic mode; untrusted content/save authority; new remote service/dashboard; all-content/Unity/final deployment tuning; new live-battle disk codec; individual finalbosses; unrelated cleanup/20minute stress.',
 'Lifecycle Disposition':'current'})
write('P2G_60_HANDOFF_001.json',{'artifacts':[h]})
write('P2G_60_RESULT_001.json',{'stage':'P2G-60','status':'complete','Handoff Artifact Ref':h['ref'],'P2G-60 Resolution Record Ref Set':[],
 'checks':{'upstream_exact':True,'single_phase_single_goal':True,'all10_elements_and_seams':True,'no_open_obligation':True,'no_production_mutation':True,'known_limitations_preserved':True},'next':'separately authorized GEX-00 GOAL-CMRP-01'})
p=HERE/'WORKING_POLICY.ko.md';p.write_text(p.read_text()+'\n[P2G-50 Goal](P2G_50_GOAL_SET.ko.md) quality2 검증002 PASS(초기 owner-label 실패001 보존), [P2G-60 exact 인계](P2G_60_HANDOFF_001.json) 완료. 다음은 원 연속 요청이 별도로 승인한 GEX-00/10 GOAL-CMRP-01이다. 제품 구현/독립 감사는 아직 미수행.\n')
print('U07-M P2G50/60 complete:10acceptances/10dimensions/5nodes/1goal; GEX00 next')
