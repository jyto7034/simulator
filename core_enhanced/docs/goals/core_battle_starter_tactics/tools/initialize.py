#!/usr/bin/env python3
"""P2G-00: preserve raw inputs and the already approved U06 scope."""
import datetime
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parents[1]
DOC_ID = 'WPD-CBST-20261004'


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
    previous = goals / 'core_resonance_lifecycle'
    audit = previous / 'post_audit/u05_20261004_002'
    closed = json.loads((audit / 'AUDIT_CLOSURE.json').read_text())
    assert closed['verdict'] == 'Complete' and closed['confidence'] == 'High'
    assert not closed['open_findings']
    assert sha(audit / 'AUDIT_CLOSURE.json') == '59db92177b241c8737406cdc1e6b937d3413df68decfffdb0b11343ead516412'
    closure = ROOT / 'core_enhanced/docs/roadmaps/core_first/execution/u05_closure_20261004_001'
    publication = json.loads((closure / 'PUBLICATION_RECORD.json').read_text())
    assert publication['audit_closure_sha256'] == sha(audit / 'AUDIT_CLOSURE.json')
    assert publication['source_before'] == publication['source_after']
    workflow = ROOT / 'core_enhanced/docs/roadmaps/core_first/execution/workflow_efficiency_update_20261004_001'
    update = json.loads((workflow / 'EVIDENCE.json').read_text())
    with tarfile.open(workflow / 'BEFORE.tar.gz') as archive:
        assert len(archive.getmembers()) == 7
        for path, row in update['changed_documents'].items():
            assert sha(ROOT / path) == row['after_sha256'], path
            assert hashlib.sha256(archive.extractfile(path).read()).hexdigest() == row['before_sha256'], path
    for path, digest in publication['documents_after'].items():
        assert update['changed_documents'][path]['before_sha256'] == digest, path
    for path, digest in update['protected_artifact_hashes'].items():
        assert sha(ROOT / path) == digest, path
    for row in publication['source_after']['files']:
        assert sha(ROOT / row['path']) == row['sha256'], row['path']
    membership = json.loads((previous / 'execution/single_goal/target/u05_002/MEMBERSHIP.json').read_text())
    changed = []
    for row in membership:
        actual = sha(ROOT / row['path'])
        if actual != row['sha256']:
            change = update['changed_documents'][row['path']]
            assert actual == change['after_sha256']
            expected_before = publication['documents_after'].get(row['path'], row['sha256'])
            assert change['before_sha256'] == expected_before
            changed.append(row['path'])
    write('WORKFLOW_INTAKE_001.json', {
        'kind': 'U06 current predecessor and operational revision intake',
        'record': str((workflow / 'RECORD.ko.md').relative_to(ROOT)),
        'evidence_sha256': sha(workflow / 'EVIDENCE.json'),
        'old_revisions': ['P2G r7', 'GEX gex-r7', 'usage v2.1.2'],
        'applied_revisions': ['P2G r8', 'GEX gex-r8', 'usage v2.1.3'],
        'safe_boundary': 'U06 P2G-00 before initial artifact publication; first unpublished initializer failed exact status hash assertion, no artifacts/product written',
        'checks': {'seven_before_after_exact': True, 'protected_artifacts_exact': True,
                   'product368_exact': True, 'predecessor_members': len(membership), 'changed_members': changed},
        'semantic_review': 'Reviewed all seven actual archived-before/current diffs. Only artifact draft checking, fixture/tool reuse, risk-first test order, concurrent document/currentness handoff and notices changed. U05 policy/acceptance/Goal/oracles/product/audit remain exact. U05 status-only publication is separately attributed and matched to update-before. No other membership delta exists.',
        'predecessor_currentness': 'current / proven non-impacting delta',
        'reuse': 'U05 Complete/High applies to its original frozen target; product and condition evidence inherited with unchanged meaning. No new audit or test execution claimed.',
        'next': 'P2G-00 raw capture with current r8 docs, then P2G-10; use existing validated artifact/fixture/tools and preserve full acceptance/final audit'
    })
    prior = json.loads((previous / 'P2G_00_INITIAL_001.json').read_text())['artifacts'][0]
    request = prior['payload']['Intake Rows'][0]['Exact Content or Content Reference']
    paths = [
        'core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md',
        'core_enhanced/docs/roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md',
        'core_enhanced/docs/roadmaps/core_first/execution/DECISIONS_20261004.ko.md',
        'core_enhanced/docs/roadmaps/core_first/execution/workflow_efficiency_update_20261004_001/RECORD.ko.md',
        'core_enhanced/docs/game_rulebook.md',
        'core_enhanced/docs/game_rules/combat.md',
        'core_enhanced/docs/game_rules/battle-controls.md',
        'core_enhanced/docs/game_rules/open-decisions.md',
        'core_enhanced/docs/skill_target_contract.md',
        'core_enhanced/docs/game_rules/equipment-and-fragments.md',
        'core_enhanced/docs/game_rules/blocking-and-targeting.md',
        'core_enhanced/docs/game_rules/results-and-save.md',
        'core_enhanced/docs/roadmaps/core_first/execution/rulebook_path_migration_20261004_001/RECORD.ko.md',
        'core_enhanced/docs/policy_to_goal_pipeline_revised_v2.md',
        'core_enhanced/docs/goal_execution_pipeline(1).md',
        'core_enhanced/docs/roadmaps/content_authoring/guides/policy_to_goal_pipeline_roadmap_usage_v2_1.ko.md',
        'core_enhanced/docs/roadmaps/core_first/execution/u01_closure_20261004_001/RESULT.ko.md',
        'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/post_audit/20261004_f01_001/COMPLETION_AUDIT.md',
        'core_enhanced/docs/goals/core_resonance_lifecycle/post_audit/u05_20261004_002/COMPLETION_AUDIT.md',
        'core_enhanced/docs/goals/core_resonance_lifecycle/post_audit/u05_20261004_002/CURRENTNESS_EXIT.json',
        'core_enhanced/docs/goals/core_resonance_lifecycle/post_audit/u05_20261004_002/AUDIT_CLOSURE.json',
        'core_enhanced/docs/goals/core_resonance_lifecycle/execution/single_goal/target/u05_002/MANIFEST.json',
        'core_enhanced/docs/roadmaps/core_first/execution/u05_closure_20261004_001/RESULT.ko.md',
        'core_enhanced/docs/roadmaps/core_first/execution/u05_closure_20261004_001/PUBLICATION_RECORD.json',
    ]
    rows = [{'Intake Item ID': 'IN-001', 'Intake Kind': 'user request',
             'Exact Content or Content Reference': request,
             'User-Provided Locator': 'Original continuous Core-first request, preserved verbatim in U05 intake',
             'User-Stated Constraint': '순차 P2G/GEX, 기존 합의·승인 범위 내 정책 판단과 기록, 구현 비참여 Astra High 독립 감사·교정, 선행 통과 전 종속 구현 금지, 원본/사용자 변경 보존, 중간 재승인 없이 계속',
             'Capture Limitation': 'None; copied original request, not a new user decision'}]
    with tarfile.open(HERE / 'INTAKE_SOURCE_001.tar.gz', 'x:gz') as archive:
        for index, name in enumerate(paths, 2):
            path = ROOT / name
            rows.append({'Intake Item ID': f'IN-{index:03}', 'Intake Kind': 'exact repository reference',
                         'Exact Content or Content Reference': {'repository': str(ROOT), 'path': name, 'sha256': sha(path)},
                         'User-Provided Locator': 'ROADMAP §5 U06/B01,B05,B06,B07,B08,B10,B11,UC01,UC02, P03-P06, CF-D01–09, U01/U04/U05 prerequisites',
                         'User-Stated Constraint': '기본 전투·순수 치유 여섯 starter·전술 명령·시간/조회·정상 RON과 복원까지; Unity/미합의 최종보스/새 무기계열은 제외',
                         'Capture Limitation': 'Exact local bytes captured; raw intake does not establish policy meaning or current runtime behavior'})
            archive.add(path, arcname=name, recursive=False)
    intake = artifact('Policy Intake Snapshot', 'PIS-CBST-001', {
        'Working Policy Document ID': DOC_ID, 'Snapshot Kind': 'initial',
        'Predecessor Policy Intake Snapshot Ref': None, 'Created By Stage': 'P2G-00', 'Intake Rows': rows})
    approval = {'user_request': 'Policy Intake Snapshot.IN-001',
                'adopted_scope': 'IN-002 ROADMAP §5 U06/B01,B05,B06,B07,B08,B10,B11,UC01,UC02 and named P03/P05 input',
                'delegation': 'IN-004 CF-D06/07 and original continuous request',
                'boundary': 'Current U05 independent Complete/High and attributed status-only closure consumed; bounded P2G00-60 then separately authorized GEX implementation/audit/correction'}
    scope = [('I01', 'in', '적 중심 기본 공격을 순수 아군 단일/범위 치유까지 확장하고 여섯 시작 세트를 정상 RON으로 실행한다', 'U06 responsibility/implementation; B01,UC01,UC02'), ('I02', 'in', '배치·철수·재배치·태세·이동·교대·작업·저지·대상 판정을 연결하며 기존 이동·교대·작업을 재사용한다', 'U06 implementation; B05,B06,B07,B08,B10,B11'), ('I03', 'in', 'Pause·Focus·배치 감속·설정 복귀의 Core 의미와 가능한 위치·비용·거절·예상/실제 결과 조회를 제공한다', 'U06 implementation/outputs/completion; P03-P06'), ('I04', 'in', '무기/schema·등록/검증·executor/회복·시간/공명·query·저장/entry 복원·정상 작성/회귀 시험을 함께 인수한다', 'U06 together-boundaries and prerequisites U04/U05'), ('I05', 'in', '교대 원자성·fullHP 작업1회·CC 정지·동시각 피해/회복·미리보기/실제 권위·시간 중첩 금지·Pause 입력·강제 시계·치유 damage proc 금지를 검증한다', 'U06 completion conditions'), ('I06', 'in', '선행 유효 증거를 재사용하고 영향 조건 검증·구현 비참여 Astra High 감사·finding 교정 및 재감사를 수행한다', 'Original continuous request; U06 prerequisites/owner'), ('O01', 'out', 'Unity 입력/HUD/고스트/카메라·최종보스 고유 UI·카드 위치 고정·미합의 minimap/rotation', 'U06 explicit exclusions; U26-A owner'), ('O02', 'out', '새 무기계열·순수 치유의 흡혈 대체·이후 콘텐츠 전체 선구현', 'U06 explicit exclusions; U07+ later owners'), ('O03', 'out', '로드맵 전체 미래 상세 설계·무관한 정리·사용자 변경과 과거 감사 원본 덮어쓰기', 'Original continuous request')]
    working = artifact('Working Revision', 'WR-CBST-001', {
        'Predecessor Working Revision Ref': None, 'Predecessor Final Policy Freeze Ref': None,
        'Approved Metadata and Scope': {'Working Policy Document ID': DOC_ID,
            'Policy Topic': 'U06 기본 전투·치유 starter·전술 명령과 조회',
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
                   'no_later_stage_artifacts': True, 'U05_independent_audit_closed': True,
                   'U05_status_only_successor_product_unchanged': True},
        'archive_sha256': sha(HERE / 'INTAKE_SOURCE_001.tar.gz'),
        'predecessor_audit_close_sha256': sha(audit / 'AUDIT_CLOSURE.json'),
        'next': 'P2G-10 under original continuous authorization', 'new_user_request_required': False,
        'production_mutation': False})
    write('WORKING_POLICY.ko.md', '# U06 기본 전투·치유 starter·전술 명령과 조회\n\nDocument Identity: `WPD-CBST-20261004` · Target Documentation Root: `core_enhanced/docs`.\nPolicy Directory: `core_battle_starter_tactics` · Current Working Revision: `WR-CBST-001`.\n승인 범위와 exact refs는 [초기 인수](P2G_00_INITIAL_001.json)가 소유한다.\n\n현재 P2G-00 완료, 다음은 P2G-10이다. 초기 정책 요소·결정·미결 질문은 비어 있다.\nP2G-10 Completion Predicate: N/A. 이 초기화는 정책 의미·코드 검토·동결·제품 구현 완료가 아니다.\n\nU05 독립 Complete / High와 제품이 불변인 상태 공개 기록을 확인하고 U06을 선택했다.\n기존 연속 요청에 따라 P2G00–60 이후 GEX 구현·검증·구현 비참여 Astra High 감사와 교정을 수행한다.\nCF-D01–09와 최신 game_rulebook.md/game_rules를 인수한다. 이름 변경 전 동결 기록은 보존한다.\n\n범위는 순수 단일/범위 기본 치유와 여섯 시작 세트, 배치·철수·태세·이동·교대·작업·저지,\nPause·포커스·배치 감속·설정 복귀의 Core 명령/조회, 정상 RON·복원·headless 검증이다.\nUnity 입력/HUD/고스트·카메라·최종보스 UI·새 무기계열과 후속 owner의 미래 콘텐츠는 제외한다.\n\n[원문 보관](INTAKE_SOURCE_001.tar.gz)과 [초기 검증](P2G_00_RESULT_001.json)을 보존한다.\n정책 의미와 코드 사실은 P2G-10/20의 각 owner에서 판정한다.\n')
    print(f'U06 P2G-00 complete: 2 immutable artifacts, {len(rows)} raw rows, {len(scope)} approved scope items; no product mutation')


if __name__ == '__main__':
    main()
