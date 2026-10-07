#!/usr/bin/env python3
"""Abort candidate review 001 and clarify discovered owner boundaries in P2G-10."""
import copy,json,tarfile
from initialize import HERE,ROOT,artifact,write,sha

def main():
    old=json.loads((HERE/'P2G_10_POLICY_001.json').read_text())['artifacts']
    working=next(a for a in old if a['ref']['Artifact Type']=='Working Revision')
    intake=json.loads((HERE/'P2G_00_INITIAL_001.json').read_text())['artifacts'][0]
    plan=json.loads((HERE/'P2G_20_SOURCE_PLAN_001.json').read_text())['artifacts'][0]
    paths=['core_enhanced/docs/game_rulebook.md','core_enhanced/src/game/battle/stat_pipeline.rs','core_enhanced/src/game/data/consumable_data.rs','core_enhanced/src/game/ability/flow.rs','core_enhanced/src/game/ability/sustained.rs','core_enhanced/src/game/data/abnormality_data.rs','core_enhanced/src/game/data/equipment_data.rs']
    with tarfile.open(HERE/'evidence/CLARIFICATION_SOURCE_001.tar.gz','x:gz') as archive:
        for p in paths:archive.add(ROOT/p,arcname=p,recursive=False)
    facts=artifact('Review Evidence','REV-CRSL-ABORT-001',{'Repository/environment identity':str(ROOT),'Evaluated Working Revision Ref':working['ref'],'Review Scope/Source Requirement Plan Ref':plan['ref'],
        'Observed sources':[{'path':p,'sha256':sha(ROOT/p)} for p in paths],
        'Observation command/method':'Read-only sed/rg from public employee stat pipeline, actual live InitialSkillCharge RON and adopted P03 redeploy rule; trace AutoCastStart and resonance writers',
        'Observed result':'Current employee pipeline adds InitialSkillCharge after fragment assembly; game_rulebook.md:426-429 explicitly reapplies it on every deployment and next combat.md:71-73 preserves valid deployment effects. AutoCastStart and hit-charge also serve opponent/other authored actors. New employee resource rule must not silently rewrite unrelated enemy cadence.',
        'Supported claim':'Clarify base amount versus valid explicit deployment effects and exact employee versus autonomous actor scope before code_reviewed publication',
        'Limitations':'Static local review only; no runtime execution or approved target implementation claim'})
    findings=[{'id':'MF-CRSL-001','classification':'material_policy','affected':['PE-CRSL-01'],
        'finding':'Absolute floor-only wording would discard explicitly preserved InitialSkillCharge/on-deploy effect; clarify base versus modifier order in policy owner.'},
        {'id':'MF-CRSL-002','classification':'material_policy','affected':['PE-CRSL-02','PE-CRSL-12'],
        'finding':'Shared old resonance structs serve employee and opponent autonomous cadence. Clarify domain applicability and removed-field projection without selecting a new opponent skill cadence.'}]
    write('P2G_20_ABORT_001.json',{'stage':'P2G-20','status':'aborted/non-current','Input Working Revision Ref':working['ref'],'Candidate Review Scope/Source Requirement Plan Ref':plan['ref'],'artifacts':[facts],
        'Finding Triage':findings,'Produced code_reviewed Policy Element Revision Ref Set':[],'Result Working Revision Ref':None,'Review Batch Ref':None,'next':'P2G-10 under existing CF-D06; no user question required after ownership evidence'})
    ip=copy.deepcopy(intake['payload']);ip['Snapshot Kind']='successor';ip['Predecessor Policy Intake Snapshot Ref']=intake['ref'];ip['Created By Stage']='P2G-10'
    ip['Intake Rows'].append({'Intake Item ID':'IN-018','Intake Kind':'existing canonical reference discovered during bounded review',
        'Exact Content or Content Reference':{'repository':str(ROOT),'path':paths[0],'sha256':sha(ROOT/paths[0])},
        'User-Provided Locator':'Original request preserve existing valid contracts; next combat.md:71-73 deployment effects',
        'User-Stated Constraint':'Keep valid deployment effects; apply newer P03 employee base charge',
        'Capture Limitation':'Existing canonical input, not a new user answer or permission; exact bytes in evidence/CLARIFICATION_SOURCE_001.tar.gz'})
    new_intake=artifact('Policy Intake Snapshot','PIS-CRSL-002',ip)
    artifacts=[new_intake]; current=copy.deepcopy(working['payload']['Current Policy Element Revision Map']);decisions=copy.deepcopy(working['payload']['Decision Revision Ref Set'])
    changes={
    1:('직원 배치 기본량과 명시 효과',
       '아군 직원의 최초 배치와 재배치 기본 공명은 장착한 실제 파편별 최대 공명의 20%를 내림한 값이다. 그 뒤 현재 유효한 InitialSkillCharge와 authored 배치 시 추가 획득을 기존 수치/상한/순서로 적용한다. 이전 개체의 공명·사용 잠금은 새 개체로 이전하지 않는다. 같은 개체의 전술 이동은 재배치 초기화가 아니다.',
       ['추가 효과 없는 최대1/4/5/99/100 직원의 최초/재배치 기본값은0/0/1/19/20이다.', '현재 InitialSkillCharge 40%의 최대100 직원은 기본20에 기존 bonus40을 더한60으로 배치/재배치되고 상한을 넘지 않는다.', '전술 이동은 공명을 초기화하지 않으며 재배치는 이전 개체의 충전량/사용 자격을 상속하지 않는다.'],
       'IN-006:71-73,140; IN-018:426-429; REV-CRSL-ABORT-001',
       'P03의 배치 기본량과 유효 배치 효과 재적용을 함께 보존한다. 기존 소모품 효과를 새로 만들거나 폐기하지 않으며 floor를 명시 bonus까지 덮는 최종값으로 해석하지 않는다.'),
    2:('직원 전투 시간 자동 충전',
       '배치되어 살아 있는 아군 직원은 전투 시간1초마다 공명1을 자동 획득한다. 초기 조절값이며 공격·방어·작업·침묵은 이를 막지 않고 정지·철수·전투불능에는 충전하지 않는다. 초과량과 차단 중 놓친 충전은 비축/보충하지 않는다. 직원의 기본 공격 방출·가한 피해·받은 피해 기본 획득과 Rest 공명5 보너스를 제거한다. Rest HP 회복과 명시 콘텐츠 자원은 유지한다. 적·소환물의 기존 autonomous skill charge/시전 주기는 이 직원 규칙만으로 초당1/20%로 변경하지 않는다.',
       ['공격 없는 치유 직원도 같은 전투 시간에 충전하며 배속은 wall time만 바꾼다.', 'Pause/철수/전투불능·상한/차단 시간을 지난 뒤 누락분을 보충하지 않는다.', '직원 기본 공격·피해·Rest는 공명 추가분을 만들지 않고 Rest HP 회복은 유지한다.', '기존 적 자동 스킬이 직원 기본 획득 제거 때문에 영구 정지하거나 새 초당1 주기로 바뀌지 않는다.'],
       'IN-006:140-147; IN-002 U05/current-state boundary; REV-CRSL-ABORT-001',
       'P03은 배치되어 살아 있는 직원의 자원을 규정한다. 별도 적/소환물 owner의 현재 authored 자동 주기를 유지하며 향후 owner의 새로운 전투 주기를 U05에서 정하지 않는다.'),
    12:('공개 작성과 전종 적용',
       '현재 모든 skill 및 Flow·시간제 버프·채널·독립 생성의 실제 사용 종료를 공개 작성 계약으로 표현하고 전종 적용성을 확인한다. 직원 공명에서 제거한 기본 획득/임의 postcast lock과 Rest 보너스 필드는 해당 live 작성 경계에서 조용히 무시하지 않는다. 적/소환물 autonomous cadence의 유효 필드는 소유 범위를 명시하고 보존하며 직원에게 잘못 적용하지 않는다. query는 사용 잠금과 획득 불가 이유를 실제 상태와 일치하게 관측한다. action control 종료·effect settlement·resource lifetime을 사용 잠금 종료와 합치지 않는다.',
       ['공개 loader→catalog/query→normal command→실제 실행에서 전종 종료 규칙을 추적하고 미분류/암묵 fallback이 없다.', '직원/Rest 경계의 제거 필드는 정확한 field 경로 오류로 거절되며 적 autonomous용 유효 데이터는 실제 해당 consumer에서 사용된다.', '발사 뒤 effects unsettled와 사용 잠금 해제가 동시에 사실일 수 있고 query가 모순 없이 표시한다.'],
       'IN-002 U05 together-boundaries/residual; IN-001; REV-CRSL-ABORT-001',
       '현재 공통 profile을 직원 규칙의 재정의로 적 AI까지 조용히 바꾸지 않는다. public schema에 각 의미의 실제 owner를 유지하면서 직원 구형 계약만 거절한다. 구체 분리 타입과 검증 방식은 기술 owner가 정한다.')}
    for n,(name,statement,acceptance,span,reason) in changes.items():
        eid=f'PE-CRSL-{n:02}';prior=next(a for a in old if a['ref']==current[eid]);prior_dec=next(a for a in old if a['ref']==prior['payload']['Decision Revision Ref'])
        dp=copy.deepcopy(prior_dec['payload']);dp.update({'Predecessor Decision Revision Ref':prior_dec['ref'],'Decision Name':name,'Exact Decision':statement,'Reason':reason,'Impact':acceptance,'Source Reference':{'intake_ref':new_intake['ref'],'span':span}})
        dp['Delegation Criteria']['derived_from_ownership']=reason
        dp['Exact User Approval Reference']['clarification']='Original IN-001/CF-D06 scoped policy discretion; ownership evidence REV-CRSL-ABORT-001; no new user answer'
        decision=artifact('Decision Revision',f'DR-CRSL-{n:02}-002',dp)
        ep=copy.deepcopy(prior['payload']);ep.update({'Predecessor Policy Element Revision Ref':prior['ref'],'Policy Statement':statement,'Decision Revision Ref':decision['ref'],'Observable Policy Acceptance':acceptance})
        element=artifact('Policy Element Revision',f'PER-CRSL-{n:02}-DECIDED-002',ep)
        decisions.remove(prior_dec['ref']);decisions.append(decision['ref']);current[eid]=element['ref'];artifacts.extend([decision,element])
    # Every candidate is reevaluated on the new exact intake/working basis; the old records stay immutable.
    basis={'Input Working Revision Ref':working['ref'],'Exact User-Approved Target Scope/Change Request':{'delegation':'IN-001/CF-D06','scope_unchanged':True,'findings':['MF-CRSL-001','MF-CRSL-002']},'Policy Intake Snapshot Ref':new_intake['ref']}
    dispositions=[]
    for previous in [a for a in old if a['ref']['Artifact Type']=='Candidate Policy Element Disposition Record']:
        dp=copy.deepcopy(previous['payload']);dp['P2G-10 Evaluation Basis']=basis
        eid=dp.get('Stable Element ID')
        if eid in current:
            element=next(a for a in artifacts+old if a['ref']==current[eid]);dp['Candidate Statement']=element['payload']['Policy Statement'];dp['Decision Revision Ref']=element['payload']['Decision Revision Ref']
        dp['Reevaluation Evidence']='Existing 17 candidates remain promoted/out_of_scope; 3 current policy revisions clarified; no scope expansion'
        d=artifact('Candidate Policy Element Disposition Record',previous['ref']['Artifact ID'].removesuffix('001')+'002',dp);artifacts.append(d);dispositions.append(d['ref'])
    wp=copy.deepcopy(working['payload']);wp.update({'Predecessor Working Revision Ref':working['ref'],'Policy Intake Snapshot Ref':new_intake['ref'],'Current Policy Element Revision Map':current,'Decision Revision Ref Set':decisions})
    result=artifact('Working Revision','WR-CRSL-003',wp);artifacts.append(result)
    for a in artifacts:assert artifact(a['ref']['Artifact Type'],a['ref']['Artifact ID'],a['payload'])==a
    assert len(current)==len(decisions)==14 and len(dispositions)==17
    write('P2G_10_POLICY_002.json',{'artifacts':artifacts})
    write('P2G_10_RESULT_002.json',{'stage':'P2G-10','status':'complete','Input Working Revision Ref':working['ref'],'Result Working Revision Ref':result['ref'],
        'Produced Candidate Disposition Ref Set':dispositions,'Result Current Candidate Policy Element Disposition Effective View':dispositions,'is_p2g10_complete':True,
        'Finding Triage':[{'classification':'material_policy','disposition':'CF-D06 routine delegated clarification in owning P2G-10; base+valid deployment effects and employee/autonomous owner boundaries','evidence':facts['ref'],'affected_elements':list(changes),'user_answer_invented':False}],
        'checks':{'all14_decided':True,'one_leaf_per_decision':True,'unchanged11_exact':True,'candidate17_reevaluated':True,'all_five_delegation_criteria_recorded':True,'policy_scope_unchanged':True},
        'Open Policy Question Log':[],'Evidence Gap Log':[],'next':'P2G-20 fresh batch with successor source plan','production_mutation':False})
    p=HERE/'WORKING_POLICY.ko.md';s=p.read_text().replace('`WR-CRSL-002`','`WR-CRSL-003`').replace('(P2G_10_POLICY_001.json)','(P2G_10_POLICY_002.json)').replace('(P2G_10_RESULT_001.json)','(P2G_10_RESULT_002.json)')
    s+='\nP2G-20 최초 candidate는 [초기량/적 owner 경계 발견](P2G_20_ABORT_001.json)으로 성공 publish 없이 반환했다. P2G-10 successor가 직원 기본20%+기존 배치 효과, 직원 충전과 적 autonomous 주기, 제거 작성 필드의 소유 범위를 명시했다. 14요소 중3개 successor/11개 exact 보존이며 다음은 새 코드 검토 batch다.\n';p.write_text(s)
    print('Aborted noncurrent review001; U05 P2G10 successor complete: 3 corrected policy revisions/11 exact preserved, no open questions')
if __name__=='__main__':main()
