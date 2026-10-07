"""Fresh U08 P2G-00 publisher: planning records only; immutable create-new outputs."""
import datetime
import hashlib
import json
import subprocess
import tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parents[1]
DOC_ID = 'WPD-CRNV-20261006'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def artifact(kind, identity, payload):
    raw = json.dumps(payload, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
    return {'ref': {'Artifact Type': kind, 'Artifact ID': identity,
                    'Content Digest': 'sha256:' + hashlib.sha256(raw).hexdigest()}, 'payload': payload}


def write(name, value):
    with (HERE / name).open('x') as stream:
        stream.write(value if isinstance(value, str) else json.dumps(value, ensure_ascii=False, indent=2) + '\n')


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
    predecessor = ROOT / 'core_enhanced/docs/goals/core_runtime_measurement_profiles'
    audit = predecessor / 'post_audit/u07m_20261006_001/AUDIT_CLOSURE.json'
    verdict = json.loads(audit.read_text())
    assert verdict['verdict'] == 'Complete' and verdict['confidence'] == 'High' and not verdict['open_findings']
    assert sha(audit) == 'a9fac11bb702a2e484daeb8c34ee04e39d3c30d553cb1c996611c1b7aca4dbb8'
    pubdir = ROOT / 'core_enhanced/docs/roadmaps/core_first/execution/u07m_closure_20261006_001'
    pub = json.loads((pubdir / 'PUBLICATION_RECORD.json').read_text())
    assert pub['audit_closure_sha256'] == sha(audit)
    changed = {row['path']: row for row in pub['documents']}
    target = predecessor / 'execution/single_goal/target/u07m_001'
    members = json.loads((target / 'MEMBERSHIP.json').read_text())
    for row in members:
        expected = changed[row['path']]['after_sha256'] if row['path'] in changed else row['sha256']
        assert sha(ROOT / row['path']) == expected, row['path']
    for row in verdict['audit_artifacts']:
        assert sha(audit.parent / row['path']) == row['sha256'], row['path']
    write('PREDECESSOR_INTAKE_001.json', {
        'audit': str(audit.relative_to(ROOT)), 'audit_closure_sha256': sha(audit),
        'target_manifest_sha256': sha(target / 'MANIFEST.json'),
        'publication': str((pubdir / 'PUBLICATION_RECORD.json').relative_to(ROOT)),
        'publication_sha256': sha(pubdir / 'PUBLICATION_RECORD.json'),
        'currentness': 'current / proven non-impacting delta', 'changed_members': list(changed),
        'member_count': len(members), 'product_identity': verdict['product_sha256'],
        'non_impact': 'All 4463 target members and 27 audit artifacts rechecked; only two exact published status-document deltas. Product, policy requirements, inputs, evidence and frozen records unchanged. This is prerequisite intake, not another independent verdict.',
        'inherited': '191 conditions:181 predecessor conditions plus10 U07-M;31 separate R06 acceptances. Default8MiB/PC31success5refusal preserved. Only the inherited R05 concurrent peak warning remains eligible;20-minute stress excluded.',
        'roles': 'root planner/coordinator; existing verified GPT-6.1 Sol/xHigh implementation/testing; nonparticipant verified GPT-6 Astra/High audit. Sol U08 assistance is read-only pending formal handoff.',
        'applied_revisions': ['P2G2026-10-04-r8', 'GEX2026-10-04-gex-r8', 'Goal quality2', 'usagev2.1.3']})
    prefix = 'core_enhanced/docs/'
    paths = [prefix + x for x in [
        'roadmaps/core_first/ROADMAP.ko.md',
        'roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md',
        'game_rulebook.md', 'game_rules/relics.md', 'game_rules/results-and-save.md',
        'game_rules/equipment-and-fragments.md', 'game_rules/open-decisions.md',
        'roadmaps/content_authoring/handoffs/R13_R16_CONTINUOUS_IMPLEMENTATION_REQUEST_20261002.ko.md',
        'roadmaps/core_first/execution/u07m_closure_20261006_001/RESULT.ko.md',
        'roadmaps/core_first/execution/u07m_closure_20261006_001/PUBLICATION_RECORD.json',
        'roadmaps/core_first/execution/workflow_efficiency_update_20261004_001/RECORD.ko.md',
        'core_runtime_contract.md', 'policy_to_goal_pipeline_revised_v2.md', 'goal_execution_pipeline(1).md',
        'roadmaps/content_authoring/guides/policy_to_goal_pipeline_roadmap_usage_v2_1.ko.md',
    ]] + [str(p.relative_to(ROOT)) for p in [audit, audit.parent / 'COMPLETION_AUDIT.md', target / 'MANIFEST.json']]
    excerpt = '''기존 연속 요청의 계획·구현·자체 검증·독립 감사·finding 교정·재감사·후속 작업 권한을 유지한다.
단계 전환이나 감사 인계마다 사용자 재승인을 요구하지 않는다.
Core 우선, Unity 후속, 개별 최종보스 고유 구현 제외를 유지한다.
위임된 정책 판단은 선택·이유·영향·위임 근거를 남긴다.'''
    rows = [{'Intake Item ID': 'IN-001', 'Intake Kind': 'current user request excerpt',
             'Exact Content or Content Reference': excerpt,
             'User-Provided Locator': '2026-10-05 current continuous resume request, autonomy paragraphs',
             'User-Stated Constraint': 'Separate root planning, designated Sol/xHigh implementation and nonparticipant Astra/High audit; preserve existing work and continuous authorization.',
             'Capture Limitation': 'Exact operative excerpt, not full conversational request. Whole request remains controlling.'}]
    with tarfile.open(HERE / 'INTAKE_SOURCE_001.tar.gz', 'x:gz') as archive:
        for index, name in enumerate(paths, 2):
            path = ROOT / name
            rows.append({'Intake Item ID': f'IN-{index:03}', 'Intake Kind': 'exact repository reference',
                         'Exact Content or Content Reference': {'repository': str(ROOT), 'path': name, 'sha256': sha(path)},
                         'User-Provided Locator': 'ROADMAP §5 U08 / P08 / H04 / applicable instructions and immediate prerequisite',
                         'User-Stated Constraint': 'C02/C03/C04 normal path; immutable definitions, copies,3slots,numeric plan,corruption; preserve prior completed contracts.',
                         'Capture Limitation': 'Raw capture only. Policy meaning and source necessity belong to subsequent owners.'})
            archive.add(path, arcname=name, recursive=False)
    intake = artifact('Policy Intake Snapshot', 'PIS-CRNV-001', {
        'Working Policy Document ID': DOC_ID, 'Snapshot Kind': 'initial',
        'Predecessor Policy Intake Snapshot Ref': None, 'Created By Stage': 'P2G-00', 'Intake Rows': rows})
    approval = {'user_request': 'IN-001 current continuous request and IN-003 continuous record',
                'scope': 'IN-002 ROADMAP §5 U08/C02–C04;IN-005 relic rules;IN-009 H04 adopted R14 scope',
                'boundary': 'U07 and U07-M independent Complete/High consumed. P2G00–60 then separately authorized GEX implementation, independent whole-goal audit/correction, subsequent eligible roadmap work.'}
    scope = [
        ('I01', 'in', 'Relic schema/validation/registration, immutable definition and copy/effect identity;normal grant,three slots,equip/transfer/release and query', 'ROADMAP U08/C02;relic rules ownership;H04§4.1/A14-01'),
        ('I02', 'in', 'Same-effect damage flat then same-kind percent sums;individual immutable numeric plan used by actual cast/preview/queued/projectile/area and preserved raw source', 'ROADMAP U08/C04;relic rules combination;H04§4.2/A14-03,04'),
        ('I03', 'in', 'Authored normal/corrupt variants;one-time per-copy catalyst/free Event transformation,disclosure,revalidation,atomic cost/result and save/entry/release persistence', 'ROADMAP U08/C03;relic rules corruption/save;H04 C-01–03/X-02'),
        ('I04', 'in', 'Representative external RON and normal expedition/trial path,applicable predecessor/fault/resource/performance regressions and independent completion', 'ROADMAP U08 acceptance;H04§5–7/X-03,04;current request'),
        ('O01', 'out', 'Full shop/supply activities and UI;new slot/duplicate policy decisions;unagreed individual content penalty/frequency commitments', 'ROADMAP U08 explicit exclusions and remaining uncertainty;U16/U26-B/U21'),
        ('O02', 'out', 'U09 finite activation/structural recipes and U10 causal chains implementation;unsupported closure must never become unconditional-active fallback', 'ROADMAP U08 exclusion and U09/U10 cards'),
        ('O03', 'out', 'Unity first,individual finalboss implementations,whole-roadmap advance detailed design,destructive cleanup or retroactive alteration of prior evidence', 'current request;ROADMAP scope'),
    ]
    working = artifact('Working Revision', 'WR-CRNV-001', {
        'Predecessor Working Revision Ref': None, 'Predecessor Final Policy Freeze Ref': None,
        'Approved Metadata and Scope': {'Working Policy Document ID': DOC_ID,
            'Policy Topic': 'U08 유물 사본·수치 계획·침식 변형의 정상 경로',
            'Requested Result': 'bounded_pipeline P2G00–60 then separately authorized GEX implementation/audit/correction and roadmap continuation',
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
                   'U07M_current_independent_Complete_High_open0': True, 'digest_refs_match': True},
        'archive_sha256': sha(HERE / 'INTAKE_SOURCE_001.tar.gz'), 'production_mutation': False,
        'next': 'P2G-10', 'new_user_request_required': False})
    write('WORKING_POLICY.ko.md', '''# U08 유물 사본·수치 계획·침식 변형의 정상 경로

Document Identity: `WPD-CRNV-20261006` · Target Documentation Root: `core_enhanced/docs`.
Policy Directory: `core_relic_numeric_variants` · Current Working Revision: `WR-CRNV-001`.

[초기 raw 인수](P2G_00_INITIAL_001.json)와 [초기화 검증](P2G_00_RESULT_001.json)이 exact scope/refs를 소유한다.
P2G-00 완료;초기 정책 요소·결정·질문은 비어 있으며 다음 P2G-10이다. 초기화는 정책 검토·구현 완료가 아니다.
P2G-10 Completion Predicate: N/A.
[선행 인수](PREDECESSOR_INTAKE_001.json)는 U07-M 독립 Complete/High/open0와 상태문서2개 비영향 delta를 인수한다.
root가 계획·설계·조정, 기존 GPT-6.1 Sol/xHigh가 제품·포맷·빌드·검증·교정, 비참여 GPT-6 Astra/High가 독립 감사를 맡는다.
기존 연속 권한에 따라 GEX와 후속 작업까지 이어가며 현재 제품 구현은 아직 시작하지 않았다.
''')
    print('U08 P2G00 complete:2 initial artifacts;4463 members and audit artifacts verified;no product mutation')


if __name__ == '__main__':
    main()
