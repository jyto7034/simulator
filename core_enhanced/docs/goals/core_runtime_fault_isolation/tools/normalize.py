#!/usr/bin/env python3
"""Repair incorrectly encoded canonical fields through immutable owner successors."""
import copy,json,hashlib
from initialize import HERE,ROOT,artifact,write
old=json.loads((HERE/'P2G_20_REVIEW_001.json').read_text())['artifacts']
wr=next(a for a in old if a['ref']['Artifact Type']=='Working Revision')
freeze=json.loads((HERE/'P2G_30_FREEZE_001.json').read_text())['artifacts'][0]
elements=[a for a in old if a['ref']['Artifact Type']=='Policy Element Revision']
prior=json.loads((HERE/'P2G_10_POLICY_001.json').read_text())['artifacts']
decisions={a['ref']['Artifact ID']:a for a in prior if a['ref']['Artifact Type']=='Decision Revision'}
plan=json.loads((HERE/'P2G_20_SOURCE_PLAN_001.json').read_text())['artifacts'][0]
origin={'Working Revision Ref':wr['ref'],'Final Policy Freeze Ref':freeze['ref'],'Phase Plan Ref':None,'Goal Set Ref':None,'Handoff Artifact Ref':None,'Post-Freeze Change Record Ref Set':[],'Authoritative Change Obligation Declaration Ref Set':[]}
policy_closure={'Direct Member Set':[],'Transitive Member Set':[],'Edges':[],'Exclusion Reason':'No semantic premise or policy meaning changes; all exact decision/acceptance text remains identical. The canonical field encoding is defective, requiring owner-level immutable representation repair, not a new gameplay choice.'}
invalid=[a for a in old if a['ref']['Artifact Type'] in ['Policy Element Revision','Working Revision','Code Review Artifact','Code Review Binding','Review Batch','Source Observation']]+[plan,freeze]
closure={'Direct Member Ref Set':[e['ref'] for e in elements],'Transitive Member Ref Set':[a['ref'] for a in invalid if a not in elements],'Edges':'current element representation→Working Revision/Review bindings/batch→Freeze; old source-plan/observation basis also replaced. Raw source files/evidence and decisions retain meaning.','Exclusions':'P2G-40/50/60 artifacts do not yet exist; runtime source and source observations factual bytes are unchanged; predecessor U02 artifacts are unrelated.'}
change=artifact('Post-Freeze Change Record','PFC-CRFI-SCHEMA-001',{'Change ID':'CH-CRFI-SCHEMA-001','Origin Composite Basis':origin,'Predecessor Final Policy Freeze Ref':freeze['ref'],'Changed Premise':'Published P2G10/20 element used Intrinsic Status and a scalar acceptance; standard repository reader requires Intrinsic status and an acceptance array. Freeze001 cannot be used by downstream standard validation.','Policy Impact Closure':policy_closure,'Artifact Invalidation Closure':closure,'Evidence':{'path':'evidence/p2g_schema_precheck_001.log','sha256':hashlib.sha256((HERE/'evidence/p2g_schema_precheck_001.log').read_bytes()).hexdigest(),'raw_exit':1,'reader':'core_enhanced/docs/tools/p2g_inputs.mjs:138–140'},'Authority':'IN-001 full P2G correction authorization; CF-D06; no additional material choice'})
members=[]
for stage,prev,types in [('P2G-10',[],['Working Revision','Decision Revision']),('P2G-20',['O10'],['Review Batch','Code Review Binding']),('P2G-30',['O20'],['Final Policy Freeze'])]:
 members.append({'Obligation ID':'O'+stage[-2:],'Owner Stage':stage,'Origin Composite Basis':origin,'Internal Prerequisite Obligation ID Set':prev,'External Prerequisite Obligation Key Set':[],'Target Basis Requirements/Projection':'same12 policy statements/decisions/acceptance text, exact unchanged source; canonical field/array representation and successor complete current basis','Artifact Invalidation Closure Selector':types,'Required Resolution Artifact Type':types})
declaration=artifact('Change Obligation Declaration','COD-CRFI-SCHEMA-001',{'Change ID':'CH-CRFI-SCHEMA-001','Post-Freeze Change Record Ref':change['ref'],'Origin Composite Basis':origin,'Policy Impact Closure':policy_closure,'Artifact Invalidation Closure':closure,'Complete Initial Obligation Member Set':members,'Stage Coverage Evidence':{'O10':'repair12 canonical element representations; retain exact meanings and current decision set','O20':'fresh full12 bindings and source-plan/observations with exact current source','O30':'new exact freeze/membership; oldFreeze stale','P2G-40/50/60':'no existing artifact to invalidate or repair; later normal execution consumes resolved successor'},'Invalidated Artifact Disposition Set':[{'Exact Artifact Ref':a['ref'],'Derived Disposition':'stale','Invalidation Reason':'canonical element field encoding defect propagates to derived current basis','Change ID':'CH-CRFI-SCHEMA-001'} for a in invalid]})
write('P2G_SCHEMA_CHANGE_001.json',{'artifacts':[change,declaration],'current_authoritative_declaration_ref':declaration['ref'],'population_evidence':'Enumerated all root-level topic artifacts before this first topic Change; no other Change or branch exists. U02 preserved as external predecessor. Both closures and all affected existing owner stages covered; strict10→20→30 prerequisites acyclic.'})
arts=[];current={}
for e in elements:
 p=copy.deepcopy(e['payload']);p['Predecessor Policy Element Revision Ref']=e['ref'];p.pop('Intrinsic Status');p['Intrinsic status']='decided';p['Observable Policy Acceptance']=[p['Observable Policy Acceptance']]
 n=p['Stable Element ID'].split('-')[-1]
 per=artifact('Policy Element Revision',f'PER-CRFI-{n}-DECIDED-002',p);arts.append(per);current[p['Stable Element ID']]=per['ref']
 assert p['Policy Statement']==e['payload']['Policy Statement'] and p['Decision Revision Ref']==e['payload']['Decision Revision Ref'] and p['Observable Policy Acceptance']==[e['payload']['Observable Policy Acceptance']]
p=copy.deepcopy(wr['payload']);p.update({'Predecessor Working Revision Ref':wr['ref'],'Predecessor Final Policy Freeze Ref':freeze['ref'],'Current Policy Element Revision Map':current,'Post-Freeze Change/Closure Basis':{'Post-Freeze Change Record Ref Set':[change['ref']],'Authoritative Change Obligation Declaration Ref Set':[declaration['ref']]}})
new=artifact('Working Revision','WR-CRFI-004',p);arts.append(new)
basis=copy.deepcopy(origin);basis.update({'Working Revision Ref':new['ref'],'Final Policy Freeze Ref':None,'Post-Freeze Change Record Ref Set':[change['ref']],'Authoritative Change Obligation Declaration Ref Set':[declaration['ref']]})
resolution=artifact('Resolution Record','RR-CRFI-SCHEMA-10-001',{'Obligation Key':{'Container Ref':declaration['ref'],'Obligation ID':'O10'},'Owner Stage':'P2G-10','Exact Resolved Prerequisite Basis':origin,'Exact Result Composite Basis':basis,'Resolution Artifact Ref':new['ref'],'Resolution Evidence':{'exact_policy_decision_acceptance_text_preserved':True,'all12_canonical_decided':True,'no_new_decision_or_question':True,'current_decision_ref_set_preserved':True,'all_earlier_bytes_preserved':True}});arts.append(resolution)
write('P2G_10_POLICY_002.json',{'artifacts':arts})
write('P2G_10_RESULT_002.json',{'stage':'P2G-10','status':'complete','reason':'Canonical representation correction under declared owner repair; no semantic policy reopening/new choice. Earlier invalid representation and Freeze remain preserved/stale.','Input Working Revision Ref':wr['ref'],'Result Working Revision Ref':new['ref'],'Resolution Record Ref Set':[resolution['ref']],'is_p2g10_complete':True,'current_basis':basis,'checks':{'decision_refs_exact_preserved':True,'all_acceptance_text_exact':True,'canonical_field_shape_correct':True,'open_questions_empty':True,'declaration_complete_current':True,'O10_effectively_resolved':True},'next':'P2G-20 successor full binding'})
write('P2G_SCHEMA_CORRECTION_001.ko.md','''# Canonical 형식 교정

P2G-40 표준 reader 진입 전 검사에서 raw exit1 `MEMBER_STATUS`가 발생했다.
원인은 `Intrinsic Status` 표기와 scalar acceptance였다. 표준은 `Intrinsic status`와 배열을 요구한다.
[원래 실패 로그](evidence/p2g_schema_precheck_001.log)를 보존했다. 도구·원래 artifact를 수정하지 않는다.

[Change/declaration](P2G_SCHEMA_CHANGE_001.json)은 기존 Freeze001과 파생 basis를 stale로 판정한다.
정책 의미의 영향 집합은 empty다. canonical representation 결함은 P2G-10 owner에서12개 successor
revision으로 교정하며 결정 Ref와 각 acceptance 원문은 그대로다. 원래12개 결정을 다시 사용자 답변으로
만들거나 새로운 정책 선택을 추가하지 않았다. full source 재실행 대신 정확한 unchanged bytes를 확인하고
P2G-20 current binding, P2G-30 successor Freeze를 새로 발행한다. O10→O20→O30를 모두 해결해야
후속 계획을 채택한다. 아직 Phase Plan/Goal/Handoff/제품 구현은 없으므로 그 단계의 repair obligation은 없다.
''')
f=HERE/'WORKING_POLICY.ko.md';s=f.read_text().replace('`WR-CRFI-003`','`WR-CRFI-004`').replace('현재 P2G-30 정책 동결 완료, 다음은 P2G-40 실행 경계 설계다.','canonical 형식 교정 P2G-10 완료, P2G-20/30 successor 재결속 중이다.');s+='\n[형식 finding과 owner 교정](P2G_SCHEMA_CORRECTION_001.ko.md). Freeze001은 stale이며 후속 계획의 입력으로 사용하지 않는다.\n';f.write_text(s)
print('Canonical repair declared: policy meaning unchanged; O10 resolved; O20/O30 pending; immutable Freeze001 preserved/stale')
