#!/usr/bin/env python3
import copy,json
from initialize import HERE,artifact,write
old=json.loads((HERE/'P2G_20_REVIEW_002.json').read_text())['artifacts'];wr=next(a for a in old if a['ref']['Artifact Type']=='Working Revision');elements=[a for a in old if a['ref']['Artifact Type']=='Policy Element Revision']
freeze=json.loads((HERE/'P2G_30_FREEZE_002.json').read_text())['artifacts'][0];plan=json.loads((HERE/'P2G_40_PHASE_PLAN_001.json').read_text())['artifacts'][0]
prior=json.loads((HERE/'P2G_10_POLICY_001.json').read_text())['artifacts'];decisions={a['ref']['Artifact ID']:a for a in prior if a['ref']['Artifact Type']=='Decision Revision'}
origin=copy.deepcopy(json.loads((HERE/'P2G_30_RESULT_002.json').read_text())['current_basis']);origin['Phase Plan Ref']=plan['ref']
policy={'Direct Member Set':[],'Transitive Member Set':[],'Edges':[],'Exclusion Reason':'All exact policy/decision/acceptance meanings unchanged; canonical ID representation only. P2G50 standard parser recognizes DR- decision identities, not DEC-.'}
invalid=[a for a in old if a['ref']['Artifact Type'] in ['Policy Element Revision','Working Revision','Code Review Artifact','Code Review Binding','Review Batch','Source Observation']]+[freeze,plan]
closure={'Direct Member Ref Set':[e['ref'] for e in elements],'Transitive Member Ref Set':[a['ref'] for a in invalid if a not in elements],'Edges':'decision ID representation→element ref→Working/Review→Freeze→Phase Plan; exact statement/acceptance/source unchanged','Exclusions':'No Goal Set/Handoff exists; source/test data and U02 audit unaffected. Prior resolved schema Change remains applicable.'}
change=artifact('Post-Freeze Change Record','PFC-CRFI-SCHEMA-002',{'Change ID':'CH-CRFI-SCHEMA-002','Origin Composite Basis':origin,'Predecessor Final Policy Freeze Ref':freeze['ref'],'Changed Premise':'P2G50 standard member field reader uses DR- prefix for Covered Decision Revision Ref Set; DEC- IDs from initial policy cannot satisfy exact Goal mapping. Correct immutable identity lineage before Goal authoring.','Policy Impact Closure':policy,'Artifact Invalidation Closure':closure,'Evidence':'core_enhanced/docs/tools/validate_p2g50_goal_set.mjs:378–381; read-only source inspection, no validator modification','Authority':'Original full P2G correction scope; no new user policy choice'})
members=[]
for stage,prev,types in [('P2G-10',[],['Working Revision','Decision Revision']),('P2G-20',['O10'],['Review Batch','Code Review Binding']),('P2G-30',['O20'],['Final Policy Freeze']),('P2G-40',['O30'],['Phase Plan'])]:
 members.append({'Obligation ID':'O'+stage[-2:],'Owner Stage':stage,'Origin Composite Basis':origin,'Internal Prerequisite Obligation ID Set':prev,'External Prerequisite Obligation Key Set':[],'Target Basis Requirements/Projection':'canonical DR identity successors with unchanged exact meanings; fresh full current binding/freeze/identical phase plan under exact new basis','Artifact Invalidation Closure Selector':types,'Required Resolution Artifact Type':types})
dec=artifact('Change Obligation Declaration','COD-CRFI-SCHEMA-002',{'Change ID':'CH-CRFI-SCHEMA-002','Post-Freeze Change Record Ref':change['ref'],'Origin Composite Basis':origin,'Policy Impact Closure':policy,'Artifact Invalidation Closure':closure,'Complete Initial Obligation Member Set':members,'Stage Coverage Evidence':{'O10':'12 same-meaning Decision successors and element references','O20':'all12 fresh bindings at same59 source files','O30':'current successor freeze with exact12 members','O40':'same4 WPs/1 atomic group/1 Phase/12 seams, new exact freeze receipt','P2G-50/60':'no published artifact yet; no repair obligation'},'Invalidated Artifact Disposition Set':[{'Exact Artifact Ref':a['ref'],'Derived Disposition':'stale','Invalidation Reason':'decision canonical ID representation propagates to downstream exact refs','Change ID':'CH-CRFI-SCHEMA-002'} for a in invalid]})
write('P2G_SCHEMA_CHANGE_002.json',{'artifacts':[change,dec],'current_authoritative_declaration_ref':dec['ref'],'population_evidence':'Enumerated root-level topic artifacts: Change001 resolved remains applicable; new Change002 only other change. No disjoint branch; strict10→20→30→40 acyclic.'})
arts=[];current={};drs=[]
for e in elements:
 d=decisions[e['payload']['Decision Revision Ref']['Artifact ID']];n=e['payload']['Stable Element ID'].split('-')[-1]
 p=copy.deepcopy(d['payload']);p['Predecessor Decision Revision Ref']=d['ref'];dr=artifact('Decision Revision',f'DR-CRFI-{n}-002',p)
 p=copy.deepcopy(e['payload']);p['Predecessor Policy Element Revision Ref']=e['ref'];p['Intrinsic status']='decided';p['Decision Revision Ref']=dr['ref'];per=artifact('Policy Element Revision',f'PER-CRFI-{n}-DECIDED-003',p)
 assert dr['payload']['Exact Decision']==d['payload']['Exact Decision'] and p['Observable Policy Acceptance']==e['payload']['Observable Policy Acceptance']
 arts.extend([dr,per]);current[p['Stable Element ID']]=per['ref'];drs.append(dr['ref'])
p=copy.deepcopy(wr['payload']);p.update({'Predecessor Working Revision Ref':wr['ref'],'Predecessor Final Policy Freeze Ref':freeze['ref'],'Current Policy Element Revision Map':current,'Decision Revision Ref Set':drs});p['Post-Freeze Change/Closure Basis']['Post-Freeze Change Record Ref Set'].append(change['ref']);p['Post-Freeze Change/Closure Basis']['Authoritative Change Obligation Declaration Ref Set'].append(dec['ref'])
new=artifact('Working Revision','WR-CRFI-006',p);arts.append(new)
basis=copy.deepcopy(origin);basis.update({'Working Revision Ref':new['ref'],'Final Policy Freeze Ref':None,'Phase Plan Ref':None});basis['Post-Freeze Change Record Ref Set'].append(change['ref']);basis['Authoritative Change Obligation Declaration Ref Set'].append(dec['ref'])
rr=artifact('Resolution Record','RR-CRFI-SCHEMA2-10-001',{'Obligation Key':{'Container Ref':dec['ref'],'Obligation ID':'O10'},'Owner Stage':'P2G-10','Exact Resolved Prerequisite Basis':origin,'Exact Result Composite Basis':basis,'Resolution Artifact Ref':new['ref'],'Resolution Evidence':{'all12_single_successor_decision_lineages':True,'exact_decision_policy_acceptance_preserved':True,'canonical_DR_PER_names_and_shapes':True,'no_new_question_or_scope':True}});arts.append(rr)
write('P2G_10_POLICY_003.json',{'artifacts':arts});write('P2G_10_RESULT_003.json',{'stage':'P2G-10','status':'complete','Input Working Revision Ref':wr['ref'],'Result Working Revision Ref':new['ref'],'Resolution Record Ref Set':[rr['ref']],'current_basis':basis,'is_p2g10_complete':True,'checks':{'O10_effectively_resolved':True,'all12_decision_meaning_unchanged':True,'all12_acceptance_arrays_exact':True,'canonical_DR_IDs':True,'open_questions_empty':True},'next':'P2G-20'})
write('P2G_SCHEMA_CORRECTION_002.ko.md','''# Decision identity 교정

P2G-50 검증기 소스 확인에서 Covered Decision Ref parser가 `DR-` prefix를 사용함을 확인했다.
초기 `DEC-` identity로는 정확한 mapping을 표현할 수 없어 P2G-10에서 동일 의미의 Decision successor12개를
발행한다. 표준 도구와 과거 artifacts는 바꾸지 않는다. 기존 Choice/acceptance/Phase 구조의 의미는 같다.

[Change002/declaration](P2G_SCHEMA_CHANGE_002.json)이 Freeze002와 Plan001을 stale로 판정하며
O10→O20→O30→O40 교정을 모두 요구한다. Change001은 해소된 상태로 계속 적용한다.
Goal/Handoff/제품 구현은 아직 없어 해당 단계 교정 의무는 없다. 새 질문·사용자 답변은 생성하지 않았다.
''')
f=HERE/'WORKING_POLICY.ko.md';s=f.read_text().replace('`WR-CRFI-005`','`WR-CRFI-006`').replace('현재 P2G-40 완료, 다음은 P2G-50 Single Goal 작성이다.','Decision identity 교정 P2G-10 완료; P2G-20/30/40 successor 준비 중이다.');s+='\n[Decision identity 표준 형식 교정](P2G_SCHEMA_CORRECTION_002.ko.md). 이전 Freeze002/Plan001은 stale이며 원본을 보존했다.\n';f.write_text(s)
print('Decision ID correction:12 exact-meaning successors; O10 resolved; all prior artifacts preserved; O20/30/40 pending')
