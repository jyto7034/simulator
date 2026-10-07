#!/usr/bin/env python3
"""P2G-00: preserve raw inputs and the already approved U05 scope."""
import datetime
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parents[1]
DOC_ID = 'WPD-CRSL-20261004'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def artifact(kind, identity, payload):
    raw = json.dumps(payload, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
    return {'ref': {'Artifact Type': kind, 'Artifact ID': identity,
                    'Content Digest': 'sha256:' + hashlib.sha256(raw).hexdigest()}, 'payload': payload}


def write(name, value):
    with (HERE / name).open('x') as stream:
        if isinstance(value, str):
            stream.write(value)
        else:
            json.dump(value, stream, ensure_ascii=False, indent=2)
            stream.write('\n')


def main():
    assert ROOT.name == 'simulator'
    assert not (HERE / 'WORKING_POLICY.ko.md').exists()
    goals = ROOT / 'core_enhanced/docs/goals'
    checked = []
    for directory in goals.iterdir():
        if not directory.is_dir() or directory == HERE:
            continue
        for path in directory.iterdir():
            if path.is_file() and (path.name.startswith('WORKING_POLICY') or
                                   path.name.startswith('P2G_') and path.suffix == '.json'):
                assert DOC_ID not in path.read_text(), path
                checked.append(str(path.relative_to(ROOT)))
    previous = goals / 'core_result_persistence'
    audit = previous / 'post_audit/u04_20261004_002'
    closed = json.loads((audit / 'AUDIT_ARTIFACTS.json').read_text())
    assert closed['verdict'] == 'Complete' and closed['confidence'] == 'High'
    assert not closed['open_findings']
    assert sha(audit / 'AUDIT_ARTIFACTS.json') == 'fd0ab4eb04073c6fbe16193ec43ce3dabb387de3862afeaf8d2972ebe26be426'
    closure = ROOT / 'core_enhanced/docs/roadmaps/core_first/execution/u04_closure_20261004_001'
    publication = json.loads((closure / 'PUBLICATION_RECORD.json').read_text())
    assert publication['audit_closure_sha256'] == sha(audit / 'AUDIT_ARTIFACTS.json')
    assert publication['source_unchanged']
    for path, digest in publication['documents_after'].items():
        assert sha(ROOT / path) == digest, path
    for row in publication['source_after']['files']:
        assert sha(ROOT / row['path']) == row['sha256'], row['path']
    prior = json.loads((previous / 'P2G_00_INITIAL_001.json').read_text())['artifacts'][0]
    request = prior['payload']['Intake Rows'][0]['Exact Content or Content Reference']
    paths = [
        'core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md',
        'core_enhanced/docs/roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md',
        'core_enhanced/docs/roadmaps/core_first/execution/DECISIONS_20261004.ko.md',
        'core_enhanced/docs/game_rulebook_next.md',
        'core_enhanced/docs/game_rules_next/combat.md',
        'core_enhanced/docs/game_rules_next/battle-controls.md',
        'core_enhanced/docs/game_rules_next/open-decisions.md',
        'core_enhanced/docs/skill_target_contract.md',
        'core_enhanced/docs/roadmaps/core_first/execution/u01_closure_20261004_001/RESULT.ko.md',
        'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/post_audit/20261004_f01_001/COMPLETION_AUDIT.md',
        'core_enhanced/docs/goals/core_result_persistence/post_audit/u04_20261004_002/COMPLETION_AUDIT.md',
        'core_enhanced/docs/goals/core_result_persistence/post_audit/u04_20261004_002/EXIT_CURRENTNESS.json',
        'core_enhanced/docs/goals/core_result_persistence/post_audit/u04_20261004_002/AUDIT_ARTIFACTS.json',
        'core_enhanced/docs/goals/core_result_persistence/execution/single_goal/target/u04_003/MANIFEST.json',
        'core_enhanced/docs/roadmaps/core_first/execution/u04_closure_20261004_001/RESULT.ko.md',
        'core_enhanced/docs/roadmaps/core_first/execution/u04_closure_20261004_001/PUBLICATION_RECORD.json',
    ]
    rows = [{'Intake Item ID': 'IN-001', 'Intake Kind': 'user request',
             'Exact Content or Content Reference': request,
             'User-Provided Locator': 'Original continuous Core-first request, preserved verbatim in U04 intake',
             'User-Stated Constraint': '순차 P2G/GEX, 기존 합의·승인 범위 내 정책 판단과 기록, 구현 비참여 Astra High 독립 감사·교정, 선행 통과 전 종속 구현 금지, 원본/사용자 변경 보존, 중간 재승인 없이 계속',
             'Capture Limitation': 'None; copied original request, not a new user decision'}]
    with tarfile.open(HERE / 'INTAKE_SOURCE_001.tar.gz', 'x:gz') as archive:
        for index, name in enumerate(paths, 2):
            path = ROOT / name
            rows.append({'Intake Item ID': f'IN-{index:03}', 'Intake Kind': 'exact repository reference',
                         'Exact Content or Content Reference': {'repository': str(ROOT), 'path': name, 'sha256': sha(path)},
                         'User-Provided Locator': 'ROADMAP §5 U05/B02,B03,B04,B09, P03/P05, CF-D06/07, U01/U04 prerequisites',
                         'User-Stated Constraint': '공명 접수·소비·사용 종료·양수 획득·대상 계약과 현재 침식 caller 호환을 함께 전환; 새 침식 피해는 U19, 미합의 최종보스·미래 owner 선구현 제외',
                         'Capture Limitation': 'Exact local bytes captured; raw intake does not establish policy meaning or current runtime behavior'})
            archive.add(path, arcname=name, recursive=False)
    intake = artifact('Policy Intake Snapshot', 'PIS-CRSL-001', {
        'Working Policy Document ID': DOC_ID, 'Snapshot Kind': 'initial',
        'Predecessor Policy Intake Snapshot Ref': None, 'Created By Stage': 'P2G-00', 'Intake Rows': rows})
    approval = {'user_request': 'Policy Intake Snapshot.IN-001',
                'adopted_scope': 'IN-002 ROADMAP §5 U05/B02,B03,B04,B09 and named P03/P05 input',
                'delegation': 'IN-004 CF-D06/07 and original continuous request',
                'boundary': 'Current U04 independent Complete/High and attributed status-only closure consumed; bounded P2G00-60 then separately authorized GEX implementation/audit/correction'}
    scope = [
        ('I01', 'in', '최초/재배치20% 내림·전투시간 초당1 공명과 기본 타격/피격/Rest 추가 공명 제거를 현재 입력/작성 데이터에서 전환한다', 'U05 B02; P03 공명과 스킬 사용'),
        ('I02', 'in', '유효 최종 입력에서 공명 전량을 즉시 한 번 소비하고 pending 실행에 접수 자격을 유지하며 거부 무소비·취소 무환불·중복 소비 방지를 인수한다', 'U05 B03 and together-boundaries'),
        ('I03', 'in', '실제 스킬 유형별 사용 종료와 방출/독립 생성물 수명을 구분하고 잠금·HardCC 중 모든 양수 추가 획득 gate 및 종료 재충전을 통일한다', 'U05 B04; P03 사용 종료 표/CC/침묵'),
        ('I04', 'in', '대상 종류·수명·유효 효과·오래된 preview를 최종 접수에서 검증하고 접수한 tile anchor와 직접 효과의 임의 retarget 금지를 유지한다', 'U05 B09; P03 수동 스킬; P05'),
        ('I05', 'in', '현재 침식 caller의 소비/접수 자격 호환을 함께 전환하고 별도 합의 없이 실제 연출/실행 타이밍을 입력 순간으로 옮기지 않는다', 'U05 scope/prerequisites/residual uncertainty; ROADMAP D07'),
        ('I06', 'in', '입력 validate→접수/소비→start gate→cast/Flow/buff/channel/생성종료→획득 gate→query·RON 제거필드·복원 fixture를 함께 전환하고 현재 전종 적용성을 점검한다', 'U05 together-boundaries and completion'),
        ('I07', 'in', 'U01 취소와 U04 결과/복원 및 유효한 선행 안전성·공개 제작/실행 증거를 인수하고 영향 조건 검증·비참여 독립 감사·교정을 수행한다', 'U05 prerequisites; original continuous request'),
        ('O01', 'out', '적 분류별 최신 침식 피해와 전체 보호/보스 기능의 선구현', 'U05 explicit exclusions; U19 owner'),
        ('O02', 'out', '독립 child/방출물 전체 생존 수명을 사용 잠금으로 묶거나 U07 이후의 새 actor/장비/유물/모드 기능을 선구현', 'U05 explicit exclusions and downstream consumers'),
        ('O03', 'out', '미합의 최종보스 고유 구현과 Unity 후속 화면 구현', 'ROADMAP USER-01/02 and U05 Core/authoring scope'),
        ('O04', 'out', '로드맵 전체 미래 상세 설계, 무관한 제품 정리, 과거 감사/사용자 변경의 덮어쓰기', 'Original continuous request and ROADMAP §5 owner sequence'),
    ]
    working = artifact('Working Revision', 'WR-CRSL-001', {
        'Predecessor Working Revision Ref': None, 'Predecessor Final Policy Freeze Ref': None,
        'Approved Metadata and Scope': {'Working Policy Document ID': DOC_ID,
            'Policy Topic': 'U05 공명 접수 자격과 실제 스킬 사용 수명',
            'Requested Result': 'bounded_pipeline P2G-00→60, then authorized GEX implementation/verification/independent audit/correction',
            'Target Project/Repository': {'project': 'simulator', 'workspace': str(ROOT),
                'head_at_intake': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()},
            'Approval Reference': approval,
            'Scope Items': [{'Scope Item ID': key, 'Approved Disposition': disposition, 'Statement': statement,
                             'Exact User Approval Reference': {**approval, 'specific_locator': locator}}
                            for key, disposition, statement, locator in scope]},
        'Policy Intake Snapshot Ref': intake['ref'], 'Current Policy Element Revision Map': {},
        'Decision Revision Ref Set': [], 'Open Policy Question Ref Set': [],
        'Post-Freeze Change/Closure Basis': {'Post-Freeze Change Record Ref Set': [], 'Authoritative Change Obligation Declaration Ref Set': []}})
    assert working['payload']['Policy Intake Snapshot Ref'] == intake['ref']
    write('P2G_00_INITIAL_001.json', {'artifacts': [intake, working]})
    write('P2G_00_RESULT_001.json', {
        'stage': 'P2G-00', 'status': 'complete', 'recorded_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'created_refs': [intake['ref'], working['ref']], 'identity_checked_paths': checked,
        'checks': {'path_unoccupied': True, 'identity_unique': True, 'conforming_path': True,
                   'scope_approval_per_item': True, 'raw_intake_archived': True, 'empty_initial_policy_sets': True,
                   'no_later_stage_artifacts': True, 'U04_independent_audit_closed': True,
                   'U04_status_only_successor_product_unchanged': True},
        'archive_sha256': sha(HERE / 'INTAKE_SOURCE_001.tar.gz'),
        'predecessor_audit_close_sha256': sha(audit / 'AUDIT_ARTIFACTS.json'),
        'next': 'P2G-10 under original continuous authorization', 'new_user_request_required': False,
        'production_mutation': False})
    write('WORKING_POLICY.ko.md', '''# U05 공명 접수 자격과 실제 스킬 사용 수명

Document Identity: `WPD-CRSL-20261004` · Target Documentation Root: `core_enhanced/docs`.
Policy Directory: `core_resonance_lifecycle` · Current Working Revision: `WR-CRSL-001`.
승인 범위와 exact refs는 [initial revision](P2G_00_INITIAL_001.json)이 소유한다.

현재 P2G-00 완료, 다음은 P2G-10이다. initial 정책 요소·결정·미결 질문은 비어 있다.
P2G-10 Completion Predicate: N/A. 입력 인수는 정책 동결·제품 구현·독립 감사 완료가 아니다.

U04 독립 Complete / High와 status-only 완료 인수를 확인한 뒤 다음 U05/B02·B03·B04·B09를 선택했다.
기존 연속 요청은 P2G00–60, 이후 GEX 구현·자체 검증·구현 비참여 Astra High 독립 감사와 교정을 포함한다.
CF-D06/07의 승인 범위 내 정책 판단·명령 발전 위임을 인수하며, 필요한 결정은 해당 단계에 근거를 남긴다.

공명 접수·소비·pending 자격·실제 사용 종료·모든 양수 획득·대상 검증과 현재 침식 caller 호환이 범위다.
새 침식 피해는 U19에 남기고 미합의 최종보스·Unity와 이후 owner의 미래 기능을 선구현하지 않는다.
현재 전종 적용성·정상 공개 RON/조회/실행·복원과 기존 안전성/관측 계약은 이후 검토 단계에서 확인한다.

[입력 원문 archive](INTAKE_SOURCE_001.tar.gz)와 [초기 검증](P2G_00_RESULT_001.json)을 보존한다.
정책 의미와 코드 사실은 owning P2G-10/20에서 판정한다. 이 초기화는 제품 변경을 포함하지 않는다.
''')
    print(f'U05 P2G-00 complete: 2 immutable artifacts, {len(rows)} raw rows, {len(scope)} approved scope items; no product mutation')


if __name__ == '__main__':
    main()
