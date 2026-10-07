"""Publish U06 completion after independent audit; preserve all audited originals."""
import datetime
import difflib
import json
import re
import subprocess
import sys
import tarfile
from pathlib import Path

R = Path('/mnt/f/work/simulator')
H = R / 'core_enhanced/docs/goals/core_battle_starter_tactics'
C = Path(__file__).resolve().parent
A = H / 'post_audit/u06_20261005_002'
T = H / 'execution/single_goal/target/u06_002'
sys.path.insert(0, str(H / 'tools'))
from run import identity, sha


def write(path, value):
    with path.open('x') as f:
        json.dump(value, f, ensure_ascii=False, indent=2)
        f.write('\n')


audit = json.loads((A / 'AUDIT_CLOSURE.json').read_text())
assert sha(A / 'AUDIT_CLOSURE.json') == '25127be9463989b2a85c1c9b98386f7607b360bf275bef409c756c8d83fc58b5'
assert audit['verdict'] == 'Complete / High' and audit['complete_high_issued']
assert not audit['open_findings'] and not audit['ending']['errors']
assert audit['dependent_u07_eligible'] and audit['scope'].startswith('single_phase whole')
assert sha(T / 'MANIFEST.json') == audit['ending']['manifest_sha256']
for row in audit['audit_artifacts']:
    assert sha(A / row['path']) == row['sha256'], row['path']
membership = json.loads((T / 'MEMBERSHIP.json').read_text())
assert all(sha(R / row['path']) == row['sha256'] for row in membership)
before = identity()
assert before == json.loads((T / 'MANIFEST.json').read_text())['product_source']
paths = [R / 'core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md',
         R / 'core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md']
original = {p: p.read_text() for p in paths}
before_hashes = {str(p.relative_to(R)): sha(p) for p in paths}

summary = '''# U06 완료 인수와 U07 시작 위치

2026-10-05 · GS-CBST-001 / GOAL-CBST-01 / PHASE-CBST-01 **Complete / High**.
[최종 독립 GEX-20](../../../../goals/core_battle_starter_tactics/post_audit/u06_20261005_002/COMPLETION_AUDIT.md)와
[감사 종료](../../../../goals/core_battle_starter_tactics/post_audit/u06_20261005_002/AUDIT_CLOSURE.json)가 판정 owner다.
단일 Phase 전체 Goal Set이 종료됐고 별도 GEX-30은 없다. 첫 감사의 Not Complete와 두 target 원본은 보존한다.

정상 RON의 여섯 시작 세트와 active 파편을 실제 grant/equip/채용에 연결했다. 기본 단일·범위 치유는
현재 회복 가능한 아군을 선택하고 실제 HP만 회복한다. 로스터10·동시배치8, 출전자격 우선,
철수·새 개체 재배치, 태세·이동·양쪽 원자 교대·즉시 작업·완료 뒤 Attack을 정상 명령으로 실행한다.
이동 뒤1500ms 강제 Defense, 연속 위치의 실제 저지 접촉, 읽기 전용 전술 미리보기와 거절 사유를 제공한다.
일반·배치·스킬 선택과 설정의 시간 복원은 공통 owner가 처리하며 Pause·disconnect·종료 우선순위를 유지한다.
기존 V11 저장은 기본 Damage의 canonical serialization 생략으로 checksum 호환성을 유지했다.

첫 감사의 U06-F01은 회복량0인 기본 Heal도 자동 행동을 시작하고 query에 표시하던 결함이었다.
공통 현재 회복량 계산을 자격과 방출에 적용했다. 0·SP0·정수 계산으로0인 단일/범위 입력은 idle이고,
양수 대조군은 정상 치유한다. 준비 뒤 능력치 변화·고정 대상·clone·실패 action 정리까지 검증했다.
zero 계수의 작성 자체는 유효하며 현재 상태에서 쓸모없는 행동을 시작하지 않는다는 정책을 유지한다.
비참여 GPT-6 Astra High의 fresh 재감사가 F01을 닫았으며 미해결 finding은0개다.

20 수락 조건·10 차원·141 선행 조건·별도31 R06 조건을 독립 판정했다.
현재 최종 Core1794/1794/1803, host92/100와 별도 heap1, fmt/check/event-log/clippy/release가 통과했다.
공개 CoreApplication8사례·원래 감사 probe 재현·strict 작성11사례를 포함한15개 새 실행과,
영향을 직접 검토한30개 기존 실행을 구분했다. 시작/종료에3,123 구성원과 제품 identity가 일치했다.
원본/현재/참조 보관본의 구성원과 정역 patch fuzz0 복원도 확인했다.

PC576의36조건(31성공/5typed거절), 실제 Continue/rewind·WebSocket·5모드 fault와 공개 비용 검증의
원래 source와 raw는 그대로다. RuntimeUnit bucket40bytes와 초기 spawn element80bytes 증가를 명시한
이전 비용 비교를 인수했고 이번 F01 교정에는 layout 변경이 없다. R05 raw exit101은 기존 승인된
concurrent peak 단독 경고(9,635,797 대9,030,188; +605,609bytes)이며 다른 hard guard 실패는 없다.
20분 stress 제외·계측 latency 제한을 유지했고 새 수치 상한/예외나 이번 성능 재측정을 주장하지 않는다.

재사용 설명의 정정: public_guards의 실제 측정은 정상 starter/SavePoint의 snapshot·checkpoint export·
event-log query·거절 staging이다. Little Red/dummy 전투는 final_performance의 경로다.
[독립 재사용 판정](../../../../goals/core_battle_starter_tactics/post_audit/u06_20261005_002/REUSE_JUDGMENTS.json)이
변경된 basic-Heal 자격/방출을 실행하지 않는 정확한 경로와 불변 입력/layout을 확인했다.
과거의 넓은 설명 대신 이 판단을 사용하며 동결 기록은 고치지 않는다.

이번 게시에서는 현황 두 문서의 진행 설명만 갱신한다. [변경 전 원문](DOCUMENTS_BEFORE.tar.gz)과
[전후 내용·비영향 검증](PUBLICATION_RECORD.json)이 이를 인수한다. 동결 PLAN·검증·정책·감사 원본은 보존한다.
Unity UI·배포 서비스·개별 최종보스 구현은 U06 완료 범위가 아니다.

다음 위치는 U07/B13·C01의 P2G-00→60이다. 독립 소환·설치·함정·장판·벽·진화·분절HP와
래티씨아 작은/큰 친구·얼음벽의 실제 소비, HP비용+생성 원자성을 현재 코드/정책에서 좁혀 인수한다.
선행161조건(기존141+U06의20)과 별도31 R06을 넘긴다. 전종 relation/infection은 U18,
보상/wave/도감 권한 승격과 개별 최종보스는 이 작업으로 추가하지 않는다.
원 연속 요청에 따라 정책·계획·구현·자체검증·비참여 독립 감사와 필요한 교정을 계속한다.
'''
status, roadmap = paths
banner = '**2026-10-05 U06 완료:** 독립 재감사 Complete / High, F01 해소·미해결 finding0. 현재 Core1794/1794/1803·host92/100+heap1과15개 새 실행·30개 명시적 재사용을 독립 판정했다. U07/B13·C01의 P2G 인수로 이어간다.'
updated = {}
for p in paths:
    updated[p], count = re.subn(r'^\*\*2026-10-05 재개 진행:\*\*[^\n]*', banner, original[p], count=1, flags=re.M)
    assert count == 1
old = '2026-10-05 · v1.145 · 현재 개발 주작업: **U06-F01 교정 자체검증 완료 / 새 동결·독립 재감사 대기**.'
assert old in updated[status]
updated[status] = updated[status].replace(old, '2026-10-05 · v1.146 · 현재 개발 주작업: **U06 독립 Complete / High 인수 → U07 P2G**.')
block = '''U06은 [최종 독립 재감사](../../goals/core_battle_starter_tactics/post_audit/u06_20261005_002/COMPLETION_AUDIT.md)에서 **Complete / High**로 종료했다.
기본 순수 치유·정상6세트·전술/작업/저지·시간 선택/설정·공개 query와 복원을 구현하고 회복량0 자격 F01을 닫았다.
20 수락 조건·10차원·141 선행 조건·별도31 R06이 통과했다. 현재 최종 Core1794/1794/1803·host92/100+heap1,
공개8사례·strict작성11·원래감사재현을 포함한15개 새 실행과30개 명시적 재사용을 구분했다.
[완료 인수·검증·남은 경고](../core_first/execution/u06_closure_20261005_001/RESULT.ko.md)를 따른다.
R05는 기존 peak 단독 경고(+605,609bytes)를 유지한다. public_guards의 정확한 재사용 근거는 최종 감사 정정을 따른다.
**현재 다음 작업은 U07/B13·C01 독립 생성물·진화·분절 상태와 공식 소비 사례의 P2G-00→60이다.**

### U06 재개·교정 이력

'''
updated[status] = updated[status].replace('## 현재 작업\n\n', '## 현재 작업\n\n' + block, 1)
updated[status] = updated[status].replace('**U06 최신 위치:**', '**첫 감사 직후 위치(이력):**', 1)
updated[status] = updated[status].replace('U05 완료 당시 다음 작업은 U06 P2G-00→60이었다. 현재는 위 재개 검증의 GEX-10 위치까지 진행했다.', 'U05 완료 당시 다음 작업은 U06 P2G-00→60이었다. U06의 현재 완료는 위 최종 재감사와 완료 인수가 소유한다.')
old = '**현재 상태: U00–U05 완료 유지 / U06 P2G-00→60 완료·GEX-10 구현/검증 중. 이전 중지 인계는 당시 증거로 보존한다.**'
assert old in updated[roadmap]
updated[roadmap] = updated[roadmap].replace(old, '**현재 상태: U00–U06 독립 Complete / High / U07 독립 생성물·진화·분절 상태 P2G 인수.**')
updated[roadmap] = updated[roadmap].replace('현재 구현·검증 위치는 [U06 PLAN](../../goals/core_battle_starter_tactics/execution/single_goal/PLAN.md)이 소유한다. U06 전체 완료·독립 감사 통과는 아직 아니다.', 'U06의 동결 [PLAN](../../goals/core_battle_starter_tactics/execution/single_goal/PLAN.md)은 감사 전 이력이며, 현재 완료는 [최종 독립 감사](../../goals/core_battle_starter_tactics/post_audit/u06_20261005_002/COMPLETION_AUDIT.md)와 [완료 인수](execution/u06_closure_20261005_001/RESULT.ko.md)가 소유한다.')
start = updated[roadmap].index('현재 재개 위치는 §5 U06/')
end = updated[roadmap].index('\n\n', start)
updated[roadmap] = updated[roadmap][:start] + 'U06은 순수 기본 치유·6세트·전술/작업/저지·시간/조회와 V11 implicit Damage 저장 호환을 완료했다.\n회복량0 자격/조회 F01은 교정·fresh 독립 재감사로 닫았고 첫 감사와 실패 원본은 보존한다.\n현재 다음 위치는 §5 U07/B13·C01 독립 생성물·진화·분절 상태와 공식 소비 사례의 **P2G-00→60**이다.' + updated[roadmap][end:]
assert updated[roadmap].split('## 0. 입력·권위·증거 수준', 1)[1] == original[roadmap].split('## 0. 입력·권위·증거 수준', 1)[1]

with tarfile.open(C / 'DOCUMENTS_BEFORE.tar.gz', 'x:gz') as archive:
    for p in paths:
        archive.add(p, arcname=str(p.relative_to(R)), recursive=False)
(C / 'RESULT.ko.md').write_text(summary)
for p in paths:
    assert sha(p) == before_hashes[str(p.relative_to(R))]
    p.write_text(updated[p])
(C / 'SUMMARY_DELTA.patch').write_text(''.join(''.join(difflib.unified_diff(original[p].splitlines(True), updated[p].splitlines(True), fromfile='before/' + str(p.relative_to(R)), tofile='after/' + str(p.relative_to(R)))) for p in paths))
after = identity()
assert after == before
changed = [row['path'] for row in membership if sha(R / row['path']) != row['sha256']]
assert set(changed) == set(before_hashes), changed
for row in audit['audit_artifacts']:
    assert sha(A / row['path']) == row['sha256']
links = []
for p, content in [(C / 'RESULT.ko.md', summary), (status, block)]:
    for target in re.findall(r'\]\(([^)]+)\)', content):
        if target != 'PUBLICATION_RECORD.json':
            assert (p.parent / target.split('#')[0]).resolve().is_file(), (p, target)
        links.append(target)
subprocess.run(['git', 'diff', '--check', '--', *map(str, paths)], cwd=R, check=True)
write(C / 'PUBLICATION_RECORD.json', {
    'kind': 'post-audit status-only successor', 'recorded_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'audit': str(A.relative_to(R)), 'audit_report_sha256': sha(A / 'COMPLETION_AUDIT.md'),
    'audit_closure_sha256': sha(A / 'AUDIT_CLOSURE.json'), 'end_currentness_sha256': sha(A / 'CURRENTNESS_END.json'),
    'target_manifest_sha256': sha(T / 'MANIFEST.json'), 'documents_before': before_hashes,
    'documents_after': {str(p.relative_to(R)): sha(p) for p in paths},
    'documents_before_archive_sha256': sha(C / 'DOCUMENTS_BEFORE.tar.gz'), 'summary_delta_sha256': sha(C / 'SUMMARY_DELTA.patch'),
    'source_before': before, 'source_after': after, 'source_unchanged': before == after,
    'changed_bound_members': changed, 'checked_new_links': links,
    'non_impact': 'Only progress/publication prose in two summaries changes after independent closure. ROADMAP section0 onward, policy/Goal, all execution/frozen/audit originals, product, fixtures, oracles and authority remain exact. public_guards wording clarification is attributed to the final independent audit.',
    'next': 'U07 P2G-00 intake under existing continuous authorization;161 inherited conditions plus31 separate R06 acceptances',
})
print(json.dumps({'U06': 'Complete / High', 'F01': 'closed', 'status_documents': 2, 'product_unchanged': True, 'next': 'U07 P2G'}))
