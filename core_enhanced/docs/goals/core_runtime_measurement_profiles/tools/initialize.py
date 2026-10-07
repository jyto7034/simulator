"""P2G-00 raw intake for U07-M; does not implement or execute product code."""
import datetime
import hashlib
import json
import subprocess
import tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parents[1]
DOC_ID = 'WPD-CMRP-20261005'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def artifact(kind, identity, payload):
    raw = json.dumps(payload, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
    return {'ref': {'Artifact Type': kind, 'Artifact ID': identity,
                    'Content Digest': 'sha256:' + hashlib.sha256(raw).hexdigest()}, 'payload': payload}


def write(name, value):
    with (HERE / name).open('x') as stream:
        stream.write(value if isinstance(value, str) else json.dumps(value, ensure_ascii=False, indent=2) + '\n')


REQUEST_EXCERPT = '''기존 연속 요청의 계획·구현·자체 검증·독립 감사·finding 교정·재감사·후속 작업 권한을 유지한다.
단계 전환이나 감사 인계마다 사용자 재승인을 요구하지 않는다.
Core 우선, Unity 후속, 개별 최종보스 고유 구현 제외를 유지한다.
위임된 정책 판단은 선택·이유·영향·위임 근거를 남긴다.

기존 사용자 변경·실패 원본·과거 감사·동결 target을 보존한다.
전체 저장소 reset/clean/stash 등으로 작업 상태를 정리하지 않는다.
구현자와 감사자가 같은 대상 파일을 동시에 변경하지 않도록 조정한다.'''


def main():
    assert not (HERE / 'WORKING_POLICY.ko.md').exists()
    checked = []
    for directory in (ROOT / 'core_enhanced/docs/goals').iterdir():
        if not directory.is_dir() or directory == HERE:
            continue
        for path in directory.iterdir():
            if path.is_file() and (path.name.startswith('WORKING_POLICY') or path.name.startswith('P2G_') and path.suffix == '.json'):
                assert DOC_ID not in path.read_text(), path
                checked.append(str(path.relative_to(ROOT)))
    predecessor = ROOT / 'core_enhanced/docs/goals/core_independent_objects'
    audit = predecessor / 'post_audit/u07_20261005_002/AUDIT_CLOSURE.json'
    verdict = json.loads(audit.read_text())
    assert verdict['verdict'] == 'Complete' and verdict['confidence'] == 'High' and not verdict['open_findings']
    assert sha(audit) == 'd371f43c1122826347261ed68f0817e66432bbc97e85fdb6759deaf686a59cb9'
    pubdir = ROOT / 'core_enhanced/docs/roadmaps/core_first/execution/u07_closure_20261005_001'
    pub = json.loads((pubdir / 'PUBLICATION_RECORD.json').read_text())
    assert pub['audit_closure_sha256'] == sha(audit)
    changed = {row['path']: row for row in pub['documents']}
    members = json.loads((predecessor / 'execution/single_goal/target/u07_002/MEMBERSHIP.json').read_text())
    for row in members:
        expected = changed[row['path']]['after_sha256'] if row['path'] in changed else row['sha256']
        assert sha(ROOT / row['path']) == expected, row['path']
    write('PREDECESSOR_INTAKE_001.json', {
        'audit': str(audit.relative_to(ROOT)), 'audit_closure_sha256': sha(audit),
        'publication': str((pubdir / 'PUBLICATION_RECORD.json').relative_to(ROOT)),
        'publication_sha256': sha(pubdir / 'PUBLICATION_RECORD.json'),
        'currentness': 'current / proven non-impacting delta', 'changed_members': list(changed),
        'member_count': len(members), 'product_identity': verdict['product_sha256'],
        'non_impact': 'All7143 target members rechecked. Only two exact status/closure/next-step document deltas differ. Product, inputs, conditions, evidence, targets and both U07 audits remain unchanged. Publication is not a new independent verdict.',
        'inherited': '181 conditions (161 predecessors plus20 U07) and31 separately tracked R06 acceptances. Default profile preservation and explicitly authorized development-profile differences must be distinguished in impact review.',
        'roles': 'root planner/coordinator; existing GPT-6.1 Sol/xHigh implementation/testing; nonparticipant GPT-6 Astra/High audit. Current Sol readiness investigation is read-only, not product implementation.',
        'applied_revisions': ['P2G2026-10-04-r8', 'GEX2026-10-04-gex-r8', 'Goal quality2', 'usagev2.1.3']})
    prefix = 'core_enhanced/docs/'
    paths = [
        prefix + 'roadmaps/core_first/ROADMAP.ko.md',
        prefix + 'roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md',
        prefix + 'roadmaps/core_first/execution/u07_memory_budget_followup_20261005_001/FOLLOWUP.ko.md',
        prefix + 'roadmaps/core_first/execution/u07_memory_budget_followup_20261005_001/RECORD.ko.md',
        prefix + 'roadmaps/core_first/execution/u07_memory_budget_followup_20261005_001/PUBLICATION_RECORD.json',
        prefix + 'roadmaps/core_first/execution/u07_closure_20261005_001/RESULT.ko.md',
        prefix + 'roadmaps/core_first/execution/u07_closure_20261005_001/PUBLICATION_RECORD.json',
        prefix + 'roadmaps/core_first/execution/workflow_efficiency_update_20261004_001/RECORD.ko.md',
        prefix + 'core_runtime_contract.md', prefix + 'game_rulebook.md',
        prefix + 'game_rules/results-and-save.md',
        prefix + 'policy_to_goal_pipeline_revised_v2.md', prefix + 'goal_execution_pipeline(1).md',
        prefix + 'roadmaps/content_authoring/guides/policy_to_goal_pipeline_roadmap_usage_v2_1.ko.md',
        str(audit.relative_to(ROOT)), str((audit.parent / 'COMPLETION_AUDIT.md').relative_to(ROOT)),
        str((predecessor / 'execution/single_goal/target/u07_002/MANIFEST.json').relative_to(ROOT)),
    ]
    rows = [{'Intake Item ID': 'IN-001', 'Intake Kind': 'current user request excerpt',
             'Exact Content or Content Reference': REQUEST_EXCERPT,
             'User-Provided Locator': '2026-10-05 current continuous resume request, autonomy and preservation paragraphs',
             'User-Stated Constraint': 'Continue authorized roadmap scope with mandatory separate implementation and independent audit roles.',
             'Capture Limitation': 'Exact operative excerpt only, not the entire request. U07-specific resume instructions remain in historical PLAN/handoff; this successor scope is the registered FOLLOWUP.'}]
    with tarfile.open(HERE / 'INTAKE_SOURCE_001.tar.gz', 'x:gz') as archive:
        for index, name in enumerate(paths, 2):
            path = ROOT / name
            rows.append({'Intake Item ID': f'IN-{index:03}', 'Intake Kind': 'exact repository reference',
                         'Exact Content or Content Reference': {'repository': str(ROOT), 'path': name, 'sha256': sha(path)},
                         'User-Provided Locator': 'Active ROADMAP §5 U07-M, registered followup and applicable pipeline/prerequisite references',
                         'User-Stated Constraint': 'Preserve deployment defaults/current U07 verdict and history; finite trusted development measurement preparation before U08.',
                         'Capture Limitation': 'Raw capture; source necessity, semantic completeness and runtime implementation not judged by P2G-00.'})
            archive.add(path, arcname=name, recursive=False)
    intake = artifact('Policy Intake Snapshot', 'PIS-CMRP-001', {
        'Working Policy Document ID': DOC_ID, 'Snapshot Kind': 'initial',
        'Predecessor Policy Intake Snapshot Ref': None, 'Created By Stage': 'P2G-00', 'Intake Rows': rows})
    approval = {'user_request': 'IN-001 current continuous resume excerpt and IN-003 continuous request',
                'scope': 'IN-002 ROADMAP §5 U07-M and IN-004 FOLLOWUP §§1–6/A1–A8',
                'boundary': 'U07 independent Complete/High consumed; bounded P2G00-60 then separately authorized implementation/testing/audit/correction and next eligible work.'}
    scope = [
        ('I01', 'in', 'Trusted explicit development measurement profile, finite emergency memory cap, warning threshold and preserved deployment defaults', 'FOLLOWUP4.1/A1–A3'),
        ('I02', 'in', 'Separate accounted extent/cumulative admission/actual allocator/RSS measurements, bounded collection and normal Core/host samples with identity/reproduction', 'FOLLOWUP4.2/A4–A5'),
        ('I03', 'in', 'Normal repetition/lifecycle/rollback/fault/profile/clone/save/replay/approval boundaries and current applicable validation', 'FOLLOWUP4.3/A3,A6–A7'),
        ('I04', 'in', 'Independent completion and documented later U30-C/U30-P tuning ownership before U08', 'FOLLOWUP3/A7–A8 and current continuous request'),
        ('O01', 'out', 'Retroactive relaxation or PASS of U07/PC/R05 failures, unbounded mode, unrelated protection removal or runtime automatic cap changes', 'FOLLOWUP1/4.1/4.3'),
        ('O02', 'out', 'Remote collection service/dashboard, all-content/Unity play aggregation and final deployment budget adequacy in this preparation step', 'FOLLOWUP3/4.2/5'),
        ('O03', 'out', 'Unity implementation, individual finalbosses, whole-future-roadmap detailed design or unrelated cleanup/destructive history changes', 'IN-001 and active continuous request'),
    ]
    working = artifact('Working Revision', 'WR-CMRP-001', {
        'Predecessor Working Revision Ref': None, 'Predecessor Final Policy Freeze Ref': None,
        'Approved Metadata and Scope': {'Working Policy Document ID': DOC_ID,
            'Policy Topic': 'U07-M 실측 기반 실행 예산 조정 준비',
            'Requested Result': 'bounded_pipeline P2G00-60 then separately authorized GEX implementation, independent audit/correction, and roadmap continuation',
            'Target Project/Repository': {'project': 'simulator', 'workspace': str(ROOT), 'head_at_intake': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()},
            'Approval Reference': approval,
            'Scope Items': [{'Scope Item ID': key, 'Approved Disposition': disposition, 'Statement': statement,
                             'Exact User Approval Reference': {**approval, 'specific_locator': locator}}
                            for key, disposition, statement, locator in scope]},
        'Policy Intake Snapshot Ref': intake['ref'], 'Current Policy Element Revision Map': {},
        'Decision Revision Ref Set': [], 'Open Policy Question Ref Set': [],
        'Post-Freeze Change/Closure Basis': {'Post-Freeze Change Record Ref Set': [], 'Authoritative Change Obligation Declaration Ref Set': []}})
    for item in (intake, working):
        assert artifact(item['ref']['Artifact Type'], item['ref']['Artifact ID'], item['payload']) == item
    write('P2G_00_INITIAL_001.json', {'artifacts': [intake, working]})
    write('P2G_00_RESULT_001.json', {'stage': 'P2G-00', 'status': 'complete',
        'recorded_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'created_refs': [intake['ref'], working['ref']], 'identity_checked_paths': checked,
        'checks': {'path_unoccupied': True, 'identity_unique': True, 'scope_approval_per_item': True,
                   'raw_intake_archived': True, 'empty_initial_policy_sets': True, 'no_later_stage_artifacts': True,
                   'U07_current_independent_Complete_High_open0': True, 'digest_refs_match': True},
        'archive_sha256': sha(HERE / 'INTAKE_SOURCE_001.tar.gz'), 'production_mutation': False,
        'next': 'P2G-10', 'new_user_request_required': False})
    write('WORKING_POLICY.ko.md', '''# U07-M 실측 기반 실행 예산 조정 준비

Document Identity: `WPD-CMRP-20261005` · Target Documentation Root: `core_enhanced/docs`.
Policy Directory: `core_runtime_measurement_profiles` · Current Working Revision: `WR-CMRP-001`.

[초기 raw 인수](P2G_00_INITIAL_001.json)와 [초기화 검증](P2G_00_RESULT_001.json)이 승인 범위와 exact refs를 소유한다.
현재 P2G-00 완료. 초기 정책 요소·결정·질문 집합은 비어 있으며 다음은 P2G-10이다.
초기화는 정책 의미·코드 검토·제품 구현·실측 완료가 아니다. P2G-10 Completion Predicate: N/A.

[선행 인수](PREDECESSOR_INTAKE_001.json)는 U07 독립 Complete/High/open0와 현황 문서만의 비영향 delta를 인수한다.
원래 배포 기준과 실패 의미를 보존하면서 trusted 유한 개발 측정 경로·bounded 집계·재현·후속 조정 준비를 닫는다.
전체 콘텐츠/Unity 실제 플레이와 최종 배포값 적정성은 U30-C/U30-P로 남긴다. 다음 순서는 U07-M→U08이다.
root는 계획/설계·조정, GPT-6.1 Sol/xHigh는 제품/포맷/빌드/시험·교정, 비참여 GPT-6 Astra/High는 독립 감사다.
원 연속 요청에 따라 P2G 이후 GEX와 finding 교정·감사·후속 작업까지 계속하며 단계 재승인을 요구하지 않는다.
''')
    print('U07-M P2G00 complete: initial2 artifacts, raw scope captured, U07 prerequisite current, no product mutation')


if __name__ == '__main__':
    main()
