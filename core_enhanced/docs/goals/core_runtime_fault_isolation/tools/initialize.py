#!/usr/bin/env python3
"""P2G-00 only: capture the authorized U03 input, without policy decisions."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tarfile

ROOT=Path(__file__).resolve().parents[5]
HERE=Path(__file__).resolve().parents[1]
DOC_ID='WPD-CRFI-20261004'

def artifact(kind,identity,payload):
    raw=json.dumps(payload,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
    return {'ref':{'Artifact Type':kind,'Artifact ID':identity,'Content Digest':'sha256:'+hashlib.sha256(raw).hexdigest()},'payload':payload}

def write(name,value):
    with (HERE/name).open('x') as f:
        if isinstance(value,str): f.write(value)
        else: json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')

def main():
    assert ROOT.name=='simulator'
    assert not (HERE/'WORKING_POLICY.ko.md').exists()
    for p in (ROOT/'core_enhanced/docs/goals').glob('*/WORKING_POLICY*.md'):
        assert DOC_ID not in p.read_text(),p
    previous=ROOT/'core_enhanced/docs/goals/core_runtime_safety_observability'
    audit=previous/'post_audit/master_20261004_002'
    assert 'Complete / High' in (audit/'COMPLETION_AUDIT.md').read_text()
    assert (audit/'END_CURRENTNESS.json').is_file()
    spec=importlib.util.spec_from_file_location('crso_intake',previous/'tools/initialize.py')
    old=importlib.util.module_from_spec(spec);spec.loader.exec_module(old)
    request=old.REQUEST
    paths=[
        'core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md',
        'core_enhanced/docs/roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md',
        'core_enhanced/docs/roadmaps/core_first/execution/DECISIONS_20261004.ko.md',
        'core_enhanced/docs/roadmaps/mod_validation_safety/ROADMAP.ko.md',
        'core_enhanced/docs/plans/runtime_safety_observability/LAB_STRESS_PROPOSAL.ko.md',
        'core_enhanced/docs/goals/core_runtime_safety_observability/post_audit/master_20261004_002/COMPLETION_AUDIT.md',
        'core_enhanced/docs/goals/core_runtime_safety_observability/post_audit/master_20261004_002/END_CURRENTNESS.json',
        'core_enhanced/docs/goals/core_runtime_safety_observability/execution/master/followups/n02_001/target/master_002/MANIFEST.json',
        'core_enhanced/docs/roadmaps/core_first/execution/u02_closure_20261004_001/RESULT.ko.md',
        'core_enhanced/docs/roadmaps/core_first/execution/u02_closure_20261004_001/PUBLICATION_RECORD.json',
    ]
    rows=[{'Intake Item ID':'IN-001','Intake Kind':'user request','Exact Content or Content Reference':request,'User-Provided Locator':'2026-10-04 conversation continuous Core-first implementation request, verbatim preserved by prior intake','User-Stated Constraint':'기존 합의·승인 범위 안 재량, 독립 감사 이전 종속 구현 금지, 원본 보존, 범위 완료/중단/실제 장애까지 계속','Capture Limitation':'None; full text reused unchanged from preceding intake'}]
    with tarfile.open(HERE/'INTAKE_SOURCE_001.tar.gz','x:gz') as tar:
        for index,name in enumerate(paths,2):
            p=ROOT/name;raw=p.read_bytes()
            rows.append({'Intake Item ID':f'IN-{index:03}','Intake Kind':'exact repository reference','Exact Content or Content Reference':{'repository':str(ROOT),'path':name,'sha256':hashlib.sha256(raw).hexdigest()},'User-Provided Locator':'활성 로드맵 U03/A06, MS04 원래 확정 정책과 독립 선행 완료 근거','User-Stated Constraint':'해당 전투 오류 격리·진단·원본 저장 보호; 정상 결과/보상으로 우회하지 않음','Capture Limitation':'Exact local bytes captured; intake alone does not certify source or new behavior'})
            tar.add(p,arcname=name,recursive=False)
    intake=artifact('Policy Intake Snapshot','PIS-CRFI-001',{'Working Policy Document ID':DOC_ID,'Snapshot Kind':'initial','Predecessor Policy Intake Snapshot Ref':None,'Created By Stage':'P2G-00','Intake Rows':rows})
    approval={'user_request':'Policy Intake Snapshot.Intake Rows[IN-001]','adopted_scope':'IN-002 ROADMAP §5 U03/A06; IN-005 MS04 and IN-006 confirmed runtime fault policy','execution_record':paths[1],'delegation_record':paths[2]+' CF-D06','boundary':'P2G through Handoff and separately authorized GEX; audited U02 prerequisite consumed, U04 result/save policy remains later owner'}
    scope=[
        ('I01','in','U03/A06: 실제 Core의 typed runtime fault를 정상 승패와 구분하여 해당 전투 중단 상태·진단·후속 명령 거절을 유지'),
        ('I02','in','서버 tick·수동 명령·늦은 callback·재접속과 저장 쓰기 gate를 함께 연결하여 보상·잘못된 저장을 차단하고 다른 player를 격리'),
        ('I03','in','기존 valid save bytes·rollback/first fault·bounded 진단을 보존하고 정상 headless host로 안전 종료와 기존 범위 복구 선택을 실제 검증'),
        ('I04','in','필요한 정책/코드 검토·정식 Goal·구현/자체검증·비참여 Astra High 독립 감사/교정과 선행 증거 적용성 인수'),
        ('O01','out','오류를 패배로 정산, 전 Run 전체 rollback 보장, 개별 skill 자동 cancel로 계속, 전 서버 강제 종료'),
        ('O02','out','U04 정상 결과 확정·보상·후퇴·SavePoint 정책 전환, 새 복구 비용/환불/상한, 원본 저장 삭제·자동 변환'),
        ('O03','out','20분 시험, 임의 성능 합격값/새 gameplay cap, Unity 구현, 개별 최종보스 고유 콘텐츠, 후속 모드 승인 정책의 일괄 설계'),
    ]
    wr=artifact('Working Revision','WR-CRFI-001',{'Predecessor Working Revision Ref':None,'Predecessor Final Policy Freeze Ref':None,'Approved Metadata and Scope':{'Working Policy Document ID':DOC_ID,'Policy Topic':'U03 런타임 오류의 명시적 종료와 세션·저장 보호','Requested Result':'bounded_pipeline P2G-00→60, then authorized GEX and independent audit','Target Project/Repository':{'project':'simulator','workspace':str(ROOT),'head_at_intake':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()},'Approval Reference':approval,'Scope Items':[{'Scope Item ID':i,'Approved Disposition':d,'Statement':s,'Exact User Approval Reference':approval} for i,d,s in scope]},'Policy Intake Snapshot Ref':intake['ref'],'Current Policy Element Revision Map':{},'Decision Revision Ref Set':[],'Open Policy Question Ref Set':[],'Post-Freeze Change/Closure Basis':{'Post-Freeze Change Record Ref Set':[],'Authoritative Change Obligation Declaration Ref Set':[]}})
    write('P2G_00_INITIAL_001.json',{'artifacts':[intake,wr]})
    write('P2G_00_RESULT_001.json',{'stage':'P2G-00','status':'complete','created_refs':[intake['ref'],wr['ref']],'checks':{'path_unoccupied':True,'identity_unique':True,'raw_intake_archived':True,'scope_approval_per_item':True,'empty_initial_policy_sets':True,'no_later_stage_artifacts':True,'U02_independent_Master_closed':True},'archive_sha256':hashlib.sha256((HERE/'INTAKE_SOURCE_001.tar.gz').read_bytes()).hexdigest(),'next':'P2G-10','production_mutation':False})
    write('WORKING_POLICY.ko.md','''# U03 런타임 오류의 명시적 종료와 세션·저장 보호

Document Identity: `WPD-CRFI-20261004` · Target Documentation Root: `core_enhanced/docs`.
Policy Directory: `core_runtime_fault_isolation` · Current Working Revision: `WR-CRFI-001`.
Exact Ref·승인 in/out 범위는 [initial revision](P2G_00_INITIAL_001.json)이 소유한다.

현재 P2G-00 완료, 다음은 P2G-10이다. initial 정책 요소·결정·미결 질문은 비어 있다.
P2G-10 Completion Predicate: N/A. 입력 인수는 새 정책 검토·제품 구현 완료를 뜻하지 않는다.

U02 S1/S2와 Master successor의 독립 Complete / High 종료 뒤 활성 U03/A06를 선택했다.
사용자의 연속 요청과 CF-D06 재량 위임을 인수하며 필요한 P2G/GEX·감사·교정을 계속한다.
원래 MS04 확정 정책은 실제 오류 전투만 중단하고 정상 결과/보상으로 처리하지 않는 것이다.
Core fault 생산→실제 application 실패/commit→host tick/재접속→진단/저장 gate의 경계를 조사한다.
정상 입력 거절과 전투 실행 실패의 분류, 복구·조회 수명은 P2G-10/20에서 근거를 남겨 결정한다.
U04의 정상 결과/SavePoint 전환이나 전체 Run rollback 보장을 이 입력으로 채택하지 않는다.

[입력 archive](INTAKE_SOURCE_001.tar.gz)와 [초기 검증](P2G_00_RESULT_001.json)을 보존한다.
기존 유효 시험·선행 100조건(U02 12+U01/R06/R07 88)의 적용성을 조건별로 확인하며
새 source 시험으로 과거 결과를 표시하지 않는다. 중단은 사용자 요청·승인 범위 완료·실제 해결 불가 장애뿐이다.
''')
    print('U03 P2G-00 complete: 2 immutable artifacts, 11 raw intake rows, 7 approved scope items; no product mutation')

if __name__=='__main__': main()
