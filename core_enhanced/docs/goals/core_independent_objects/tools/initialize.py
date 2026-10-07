#!/usr/bin/env python3
"""U07 P2G-00 raw intake and approved scope; no policy/source conclusions."""
import datetime
import hashlib
import json
import subprocess
import tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parents[1]
DOC_ID = 'WPD-CIOB-20261005'


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


REQUEST = '''core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md의 활성 범위를
현행 P2G와 GEX에 따라 순차적으로 계속 구현해줘.
이번 요청은 기존 연속 실행의 재개이며 처음부터 다시 시작하는 요청이 아니다.

먼저 다음 문서와 연결된 현재 작업의 실행 기록을 확인해:
- core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md
- core_enhanced/docs/roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md
- core_enhanced/docs/roadmaps/core_first/execution/workflow_efficiency_update_20261004_001/RECORD.ko.md

현재 Goal의 PLAN·EXPERIMENT_NOTES·최신 구현/교정 결과·독립 감사와
실제 작업 트리를 대조해 마지막 유효 완료 지점과 다음 필요한 단계를 판단해.
현황 요약이 뒤처졌다면 연결된 실제 실행 근거를 확인하고,
U00/U01부터 재시작하거나 이미 완료된 계획·구현·검증을 반복하지 마.
아직 실행 중인 명령이나 에이전트 작업이 있는지도 확인해 중복 실행을 피하고,
인수 판단을 간결하게 기록한 뒤 바로 필요한 작업을 이어가줘.

Core 우선·Unity 후속·개별 최종보스 고유 구현 제외라는 활성 범위를 유지해.
현재 게임 정책은 최신 사용자 합의와
core_enhanced/docs/game_rulebook.md 및 game_rules/를 기준으로 하고,
과거 문서와 동결 기록은 당시의 근거로 구분해.

이번 요청은 각 작업에 필요한 다음 과정을 모두 포함해:
- P2G 정책 정리·코드 검토·계획·Goal·인계
- GEX 구현·자체 검증·독립 감사
- 감사 finding 교정·재감사
- 선행 조건 충족 후 다음 작업 선택과 실행

단계 전환이나 감사 인계마다 재승인을 요청하지 마.
독립 감사에는 구현에 참여하지 않은 별도 에이전트
(gpt-6-astra, reasoning high)를 활용하고,
감사 통과 전에는 그 결과에 의존하는 후속 구현을 시작하지 마.
독립성 조건을 만족하면 기존 검토자를 재사용해도 된다.

구현 중 필요한 정책은 기존 명시적 합의와 승인 범위 안에서 재량으로 판단해.
결정·이유·영향·위임 근거를 해당 정책 문서에 남기고,
필요한 P2G 갱신·재검증 후 구현을 이어가줘.
사용자 직접 결정과 AI 위임 결정을 구분하고,
위임 범위를 벗어나는 새로운 선택만 질문해.

구현 품질을 우선하고 기존 계획과 유효한 증거를 재사용해.
형식·테스트 입력 오류는 일찍 확인하고, 변경 영향에 맞게 재검증하되
필수 통합 검증과 독립 감사를 생략하지 마.
로드맵 전체를 미리 상세 설계하거나,
새 세션이라는 이유만으로 전체 이력·전체 테스트를 반복하지 마.

공유 문서 변경은 진행 중 감사 대상과 겹치는지 확인하고,
동결 원본을 보존하면서 변경 영향과 인수 위치를 기록해.
기존 사용자 변경·실패 증거·과거 감사 원본은 보존해줘.

작업별 결과·검증·남은 문제·정확한 다음 위치를 계속 기록해.
의미 있는 진척은 간결하게 보고하되 보고만 하고 멈추지는 마.

내 중단 요청, 승인 범위 완료, 또는 실제로 사용자 입력이나
추가 권한·환경이 필요해 더 진행할 수 없는 장애에서만 멈춰줘.
해결 가능한 구현·테스트 실패는 승인 범위 안에서 교정하고 계속 진행해.
중단 시 완료·미완료 상태와 정확한 재개 위치를 남겨줘.'''


def main():
    assert ROOT.name == 'simulator'
    assert not (HERE / 'WORKING_POLICY.ko.md').exists()
    checked = []
    for directory in (ROOT / 'core_enhanced/docs/goals').iterdir():
        if not directory.is_dir() or directory == HERE:
            continue
        for path in directory.iterdir():
            if path.is_file() and (path.name.startswith('WORKING_POLICY') or path.name.startswith('P2G_') and path.suffix == '.json'):
                assert DOC_ID not in path.read_text(), path
                checked.append(str(path.relative_to(ROOT)))
    old = ROOT / 'core_enhanced/docs/goals/core_battle_starter_tactics'
    audit_dir = old / 'post_audit/u06_20261005_002'
    audit = json.loads((audit_dir / 'AUDIT_CLOSURE.json').read_text())
    assert audit['verdict'] == 'Complete / High' and not audit['open_findings'] and not audit['ending']['errors']
    assert audit['dependent_u07_eligible']
    for row in audit['audit_artifacts']:
        assert sha(audit_dir / row['path']) == row['sha256'], row['path']
    closure = ROOT / 'core_enhanced/docs/roadmaps/core_first/execution/u06_closure_20261005_001'
    publication = json.loads((closure / 'PUBLICATION_RECORD.json').read_text())
    assert publication['audit_closure_sha256'] == sha(audit_dir / 'AUDIT_CLOSURE.json')
    assert publication['source_before'] == publication['source_after'] and publication['source_unchanged']
    for row in publication['source_after']['files']:
        assert sha(ROOT / row['path']) == row['sha256'], row['path']
    membership = json.loads((old / 'execution/single_goal/target/u06_002/MEMBERSHIP.json').read_text())
    changed = []
    for row in membership:
        actual = sha(ROOT / row['path'])
        if actual != row['sha256']:
            assert publication['documents_before'][row['path']] == row['sha256']
            assert publication['documents_after'][row['path']] == actual
            changed.append(row['path'])
    assert set(changed) == set(publication['documents_after'])
    with tarfile.open(closure / 'DOCUMENTS_BEFORE.tar.gz') as archive:
        for path, digest in publication['documents_before'].items():
            assert hashlib.sha256(archive.extractfile(path).read()).hexdigest() == digest
    write('PREDECESSOR_INTAKE_001.json', {
        'kind': 'U07 current predecessor intake', 'audit': str(audit_dir.relative_to(ROOT)),
        'audit_closure_sha256': sha(audit_dir / 'AUDIT_CLOSURE.json'),
        'publication': str((closure / 'PUBLICATION_RECORD.json').relative_to(ROOT)),
        'publication_sha256': sha(closure / 'PUBLICATION_RECORD.json'),
        'currentness': 'current / proven non-impacting delta', 'changed_members': changed,
        'member_count': len(membership), 'product_identity': publication['source_after']['sha256'],
        'non_impact': 'Verified all target members. Only the two published progress summaries differ; exact before/current hashes and archived bytes match. Reviewed publication diff: progress/closure/history references only; ROADMAP section0 onward unchanged. U06 policy, acceptance, source, input, tests, execution records, frozen targets and independent audit remain unchanged.',
        'inherited': '161 conditions (141 predecessors plus20 U06), and31 separately tracked original R06 acceptances',
        'reuse_clarification': 'Final U06 audit REUSE_JUDGMENTS precisely distinguishes public_guards snapshot/checkpoint/query/rejection paths from final_performance Little Red/dummy workloads; use that applicability judgment.',
        'applied_revisions': ['P2G2026-10-04-r8', 'GEX2026-10-04-gex-r8', 'Goal quality2', 'usagev2.1.3'],
        'next': 'P2G-00 raw capture then P2G-10 under existing continuous authorization'})
    paths = [
        'core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md',
        'core_enhanced/docs/roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md',
        'core_enhanced/docs/roadmaps/core_first/execution/DECISIONS_20261004.ko.md',
        'core_enhanced/docs/roadmaps/core_first/execution/workflow_efficiency_update_20261004_001/RECORD.ko.md',
        'core_enhanced/docs/game_rulebook.md', 'core_enhanced/docs/game_rules/combat.md',
        'core_enhanced/docs/game_rules/equipment-and-fragments.md', 'core_enhanced/docs/game_rules/blocking-and-targeting.md',
        'core_enhanced/docs/game_rules/results-and-save.md', 'core_enhanced/docs/game_rules/open-decisions.md',
        'core_enhanced/docs/skills/abnormality_design_notes.ko.md',
        'core_enhanced/docs/skills/abnormality_skill_fragments_design_notes.md',
        'core_enhanced/docs/roadmaps/content_authoring/handoffs/R13_R16_CONTINUOUS_IMPLEMENTATION_REQUEST_20261002.ko.md',
        'core_enhanced/docs/roadmaps/content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R13.ko.md',
        'core_enhanced/docs/roadmaps/content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/CONTRACTS.ko.md',
        'core_enhanced/docs/roadmaps/content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/ACCEPTANCE.ko.md',
        'core_enhanced/docs/core_runtime_contract.md', 'core_enhanced/docs/skill_target_contract.md',
        'core_enhanced/docs/policy_to_goal_pipeline_revised_v2.md', 'core_enhanced/docs/goal_execution_pipeline(1).md',
        'core_enhanced/docs/roadmaps/content_authoring/guides/policy_to_goal_pipeline_roadmap_usage_v2_1.ko.md',
        str((audit_dir / 'COMPLETION_AUDIT.md').relative_to(ROOT)), str((audit_dir / 'AUDIT_CLOSURE.json').relative_to(ROOT)),
        str((old / 'execution/single_goal/target/u06_002/MANIFEST.json').relative_to(ROOT)),
        str((closure / 'RESULT.ko.md').relative_to(ROOT)), str((closure / 'PUBLICATION_RECORD.json').relative_to(ROOT)),
    ]
    rows = [{'Intake Item ID': 'IN-001', 'Intake Kind': 'user request',
             'Exact Content or Content Reference': REQUEST,
             'User-Provided Locator': 'Current continuous resume request; operative instructions transcribed verbatim; trailing prior-implementer quotation remains in U06 resume/handoff history',
             'User-Stated Constraint': 'Core first/Unity later/no individual final boss; sequential P2G+GEX, independent nonparticipant Astra High, corrections and next eligible owner without repeated approvals; preserve history and user changes',
             'Capture Limitation': 'Trailing historical handoff quote omitted from this operative-instruction excerpt; preserved linked U06 history is not a new U07 constraint'}]
    with tarfile.open(HERE / 'INTAKE_SOURCE_001.tar.gz', 'x:gz') as archive:
        for index, name in enumerate(paths, 2):
            path = ROOT / name
            rows.append({'Intake Item ID': f'IN-{index:03}', 'Intake Kind': 'exact repository reference',
                         'Exact Content or Content Reference': {'repository': str(ROOT), 'path': name, 'sha256': sha(path)},
                         'User-Provided Locator': 'Active ROADMAP U07/B13,C01 and linked current policy/accepted R13 references; U06 prerequisite',
                         'User-Stated Constraint': 'Capture current bytes; historical references/old paths are not promoted above current rulebook and user authority',
                         'Capture Limitation': 'Raw input capture only; no policy completeness or runtime behavior judgment'})
            archive.add(path, arcname=name, recursive=False)
    intake = artifact('Policy Intake Snapshot', 'PIS-CIOB-001', {
        'Working Policy Document ID': DOC_ID, 'Snapshot Kind': 'initial',
        'Predecessor Policy Intake Snapshot Ref': None, 'Created By Stage': 'P2G-00', 'Intake Rows': rows})
    approval = {'user_request': 'IN-001 current continuous resume', 'scope': 'IN-002 active ROADMAP §5 U07/B13,C01',
                'delegation': 'IN-001 and IN-004 CF-D06/07',
                'boundary': 'U06 independent whole-goal Complete/High consumed; bounded P2G00-60 then separately authorized GEX implementation/audit/correction'}
    scope = [
        ('I01', 'in', '공통 소환/장판/함정/벽/진화·분절HP의 정상 작성과 래티씨아 작은/큰 친구·얼음벽 등의 실제 소비', 'U07 implementation and outputs; C01'),
        ('I02', 'in', 'actor/object owner·source·TTL·정지·진화 invalidation·삭제/철수/회수와 정상 query·entry 복원', 'U07 together-boundaries and completion'),
        ('I03', 'in', '최대HP 변경·HP 비용·처형·source/released 수명과 실제 결과, HP비용+생성 실패 무변경', 'B13 and U07 implementation/completion'),
        ('I04', 'in', '공개 parser/validate/compile/executor와 실제 battle fixture·Lab capability; 비용/회수/공명 종료·오류/중복/복원 검증', 'U07 together-boundaries and completion'),
        ('I05', 'in', '유효 선행 증거 재사용·필수 통합 검증·비참여 Astra High 독립 감사·finding 교정/재감사와 다음 eligible 작업', 'IN-001 current request'),
        ('O01', 'out', '소환을 wave/보상/도감 권한으로 승격하거나 전종 relation/infection을 U07에 선구현', 'U07 explicit exclusions; U18'),
        ('O02', 'out', 'Unity 구현·개별 최종보스 고유 구현·전체 미래 로드맵 상세 설계', 'IN-001 and active roadmap scope'),
        ('O03', 'out', '미지원 입력의 가짜 성공·기존 사용자 변경/실패/과거 감사 원본 덮어쓰기·무관 정리', 'U07 explicit exclusions and IN-001'),
    ]
    working = artifact('Working Revision', 'WR-CIOB-001', {
        'Predecessor Working Revision Ref': None, 'Predecessor Final Policy Freeze Ref': None,
        'Approved Metadata and Scope': {'Working Policy Document ID': DOC_ID,
            'Policy Topic': 'U07 독립 생성물·진화·분절 상태와 공식 소비 사례',
            'Requested Result': 'bounded_pipeline P2G00-60 followed by separately authorized GEX implementation/self-verification/independent audit/correction',
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
    for item in [intake, working]:
        assert artifact(item['ref']['Artifact Type'], item['ref']['Artifact ID'], item['payload']) == item
    write('P2G_00_INITIAL_001.json', {'artifacts': [intake, working]})
    write('P2G_00_RESULT_001.json', {
        'stage': 'P2G-00', 'status': 'complete', 'recorded_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'created_refs': [intake['ref'], working['ref']], 'identity_checked_paths': checked,
        'checks': {'path_unoccupied': True, 'identity_unique': True, 'conforming_path': True,
                   'scope_approval_per_item': True, 'raw_intake_archived': True, 'empty_initial_policy_sets': True,
                   'no_later_stage_artifacts': True, 'U06_independent_audit_closed': True,
                   'U06_status_only_successor_product_unchanged': True, 'draft_refs_fields_digests_checked': True},
        'archive_sha256': sha(HERE / 'INTAKE_SOURCE_001.tar.gz'),
        'predecessor_audit_close_sha256': sha(audit_dir / 'AUDIT_CLOSURE.json'),
        'next': 'P2G-10 under current continuous authorization', 'new_user_request_required': False, 'production_mutation': False})
    write('WORKING_POLICY.ko.md', '# U07 독립 생성물·진화·분절 상태와 공식 소비 사례\n\n'
          'Document Identity: `WPD-CIOB-20261005` · Target Documentation Root: `core_enhanced/docs`.\n'
          'Policy Directory: `core_independent_objects` · Current Working Revision: `WR-CIOB-001`.\n'
          '승인 범위와 exact refs는 [초기 인수](P2G_00_INITIAL_001.json)가 소유한다.\n\n'
          '현재 P2G-00 완료, 다음은 P2G-10이다. 초기 정책 요소·결정·미결 질문은 비어 있다.\n'
          'P2G-10 Completion Predicate: N/A. 초기화는 정책 의미·코드 검토·동결·제품 구현 완료가 아니다.\n\n'
          '[선행 인수](PREDECESSOR_INTAKE_001.json)에서 U06 Complete/High와 두 현황 문서 delta의 비영향을 확인했다.\n'
          '기존 연속 요청에 따라 U07 P2G00–60 이후 GEX 구현·검증·비참여 Astra High 감사와 필요한 교정을 수행한다.\n'
          '선행161조건과 별도31 R06을 인수하며 최신 game_rulebook.md/game_rules·CF-D01–09가 정책 기준이다.\n'
          '범위는 U07/B13·C01의 독립 소환/장판/함정/벽·진화·분절HP와 공식 소비, source/TTL/회수/HP비용 원자성이다.\n'
          '전종 relation/infection·Unity·개별 최종보스·전체 미래 설계는 해당 후속/제외 경계를 따른다.\n\n'
          '[원문 보관](INTAKE_SOURCE_001.tar.gz)과 [초기 검증](P2G_00_RESULT_001.json)을 보존한다.\n')
    print(f'U07 P2G-00 complete:2 artifacts,{len(rows)} raw rows,{len(scope)} approved scope items; no product mutation')


if __name__ == '__main__':
    main()
