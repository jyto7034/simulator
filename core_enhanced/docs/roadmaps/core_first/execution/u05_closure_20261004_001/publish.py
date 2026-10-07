"""Publish an independently completed U05 without changing audited originals."""
import datetime
import hashlib
import json
import re
import subprocess
import sys
import tarfile
from pathlib import Path

R = Path('/mnt/f/work/simulator')
H = R / 'core_enhanced/docs/goals/core_resonance_lifecycle'
C = Path(__file__).resolve().parent
A = H / 'post_audit/u05_20261004_002'
T = H / 'execution/single_goal/target/u05_002'
sys.path.insert(0, str(H / 'tools'))
from run import identity, sha


def write(path, value):
    with path.open('x') as f:
        json.dump(value, f, ensure_ascii=False, indent=2)
        f.write('\n')


audit = json.loads((A / 'AUDIT_CLOSURE.json').read_text())
assert audit['verdict'] == 'Complete' and audit['confidence'] == 'High' and audit['whole_single_phase']
assert not audit['open_findings'] and audit['exit']['passed']
assert sha(A / 'COMPLETION_AUDIT.md') == 'e82c49dd37905a8fbe2d02be842c90c8bbe23a1d71fa4161afb80176c9fe7a75'
assert sha(T / 'MANIFEST.json') == audit['target_manifest_sha256']
for row in audit['files']:
    assert sha(A / row['path']) == row['sha256'], row['path']
membership = json.loads((T / 'MEMBERSHIP.json').read_text())
assert all(sha(R / row['path']) == row['sha256'] for row in membership)
before = identity()
assert before == json.loads((T / 'MANIFEST.json').read_text())['product_source']
paths = [R / 'core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md',
         R / 'core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md']
original = {p: p.read_text() for p in paths}
before_hashes = {str(p.relative_to(R)): sha(p) for p in paths}
with tarfile.open(C / 'DOCUMENTS_BEFORE.tar.gz', 'x:gz') as archive:
    for p in paths:
        archive.add(p, arcname=str(p.relative_to(R)), recursive=False)

summary = '''# U05 완료 인수와 U06 시작 위치

2026-10-04 · GOAL-CRSL-01 / GS-CRSL-001 **Complete / High**.
[최종 독립 GEX-20](../../../../goals/core_resonance_lifecycle/post_audit/u05_20261004_002/COMPLETION_AUDIT.md)와
[감사 종료 기록](../../../../goals/core_resonance_lifecycle/post_audit/u05_20261004_002/AUDIT_CLOSURE.json)이 판정 owner다.
단일 Phase 전체 Goal을 감사했으며 별도 GEX-30은 해당하지 않는다.

직원의 유효한 스킬 입력을 접수할 때 공명을 한 번 전량 소비하고, private action 자격으로 지연 시작과
기존 침식 시작을 실행한다. 잘못된 입력·미리보기는 소비하지 않고 취소·중단은 환불하지 않는다.
사용 수명은 남은 동작·연속 공격·실제 유한 버프/채널과 연결하며, 이미 방출된 발사체·독립 장판·
지속 피해·영구 효과의 잔여 수명과 구분한다. Hard CC와 사용 잠금은 모든 양수 획득을 막는다.
직원은 배치된 생존 상태에서 전투 시간의 매 온전한 1초마다 공명1을 받고 누락분을 보충하지 않는다.
기본 타격/피격과 Rest 공명 보너스 및 종료 후 임의 잠금을 제거했다. 상대 자율 충전과 Rest HP는 유지한다.

선택한 파편 형태의 필수 최대치와 배치 시작20%를 적용하고, 원래 유효 배치 효과를 합성한다.
접수한 직접 대상과 범위 기준 타일을 구분하며 필수 대상 소실은 미래 동작을 취소한다.
수동 재지정 금지, 유효한 빈 타일 선택, 명시적 사용 수명, 조회의 사용 단계·획득 금지 사유를 연결했다.
현재 직원 작성과 Rest 입력을 엄격히 검증하고 실제 데이터·공개 소비자를 이행했다.
Continue V11과 변환 전용 V12, 같은 actor의 접수 상태 재접속과 새 process의 확정/진입 전 복원 경계를 구분한다.

14 수락 조건·14 검증·10 차원,127 선행 조건과 별도31 R06 조건을 독립 판정했다.
Core1758/1758/1767, host91/99 및 별도 heap1, fmt/check/event-log/clippy/release가 통과했다.
공개 소비자·API21개 명령, 실제 stock WebSocket Continue/rewind·접수 후 정지 재접속,
다섯 fault 관측 모드·queue/observer 정리, strict schema와 독립 장판 검증을 현재 source에 결속했다.

PC576행의 원래 전투 로그·HP·facts·queue·거절 기준은36조건(31성공/5거절) 모두 통과했다.
비직원 검증 fixture의 원래0/100/1000 입력을 명시해 employee 기본값 변경에 따른 입력 변동을 교정했다.
실제 비용은 상태64bytes/unit과 source 순회 등에 따라 증가했다. 비용 불변 비교의 실패를 보존하고,
명시한 증가분을 모든 행에서 대조한 뒤 시간 외 나머지 필드를 완전 비교했다. 비용이 같다고 주장하지 않는다.
원래 public 성능/할당 상한은 통과했다. R05 raw101은 기존 승인 concurrent peak 단독 경고
(+466,821bytes)이며 다른 hard guard 실패는 없다. 계측 latency는 관찰값이고20분 시험 제외도 유지한다.
새 Unity UI·배포된 원격 서비스·직원 소환 API·구형 저장 자동 호환을 완료했다고 주장하지 않는다.

첫 감사는 감사 중 외부 CF-D09 문서 이동으로11개 현재 참조가 달라진 F01 하나로 Not Complete / High였다.
제품 결함은 별도로 확인되지 않았다.46개 문서의 전후 hash와 정책 본문 보존을 확인하고 새 경로와
대상을 결속했다. 재감사는 F01을 닫고43개 실행의 정확한 재사용,12개 보관본과 새 정역 복원을 확인했다.
종료 시3,511개 구성원과 제품368개가 일치했다. 과거 target/감사·실패 원본과 외부 사용자 변경은 보존한다.
현재 정책은 game_rulebook.md·game_rules/이며 과거 룰북은 archive의 비교 자료다.

이번 완료 게시는 현황 두 문서만 갱신한다. [게시 전 원문](DOCUMENTS_BEFORE.tar.gz)과
[실제 전후 내용·비영향 검증](PUBLICATION_RECORD.json)에 기록하며 독립 감사 원문은 수정하지 않는다.

다음 위치는 U06/B01·B05–B08·B10·B11·UC01·UC02의 별도 P2G-00→60이다.
순수 단일/범위 치유 기본 행동, 여섯 시작 세트, 전술 이동·교대·작업·저지와 배치/조건,
공통 포커스·정지·설정 복귀의 Core 명령·조회 및 정상 RON 실행을 현재 코드와 대조한다.
선행141조건(기존127+U05의14)과 별도31 R06을 인수한다. Unity UI와 후속 생성물/유물/최종보스는
이번 U06 범위에 미리 넣지 않는다. 원 연속 요청에 따라 다음 정책·구현·검증·독립 감사까지 계속한다.
'''
(C / 'RESULT.ko.md').write_text(summary)
status, roadmap = paths
status_text = original[status]
old = '2026-10-04 · v1.140 · 현재 개발 주작업: **U04 독립 Complete / High 인수 → U05 P2G**.'
assert status_text.count(old) == 1
status_text = status_text.replace(old, '2026-10-04 · v1.141 · 현재 개발 주작업: **U05 독립 Complete / High 인수 → U06 P2G**.')
block = '''U05 공명 접수·사용 수명은 [최종 독립 재감사](../../goals/core_resonance_lifecycle/post_audit/u05_20261004_002/COMPLETION_AUDIT.md)에서
**Complete / High**를 받았다.14 수락 조건·10차원·127 선행 조건·별도31 R06을 판정했다.
접수 시 전량 소비와 private 자격, 실제 사용 종료 잠금, 공통 양수 획득 gate·직원 초당1 충전,
엄격한 형태 최대치·사용 수명·대상·현재 저장/재접속을 정상 Core/host/data 경로로 연결했다.
Core1758/1758/1767·host91/99와 별도 heap1, 공개 소비자·PC576 전투 기준이 통과했다.
실제 비용 증가분을 명시해 비교했고 기존 수치 상한을 유지했다. R05는 승인된 peak 단독 경고(+466,821bytes)다.
첫 감사 중 외부 문서 이동으로 생긴 F01을 새 경로·대상 인수로 닫았으며 제품368개와43개 실행은 그대로였다.
[완료 인수·검증·남은 경고](../core_first/execution/u05_closure_20261004_001/RESULT.ko.md)를 따른다.
**현재 다음 작업은 U06의 기본 전투·순수 치유와 여섯 시작 세트·전술/시간 명령·조회 P2G-00→60이다.**
구현은 U05의 독립 완료를 선행 조건으로 인수하며 최신 룰북 경로와 기존 유효 증거를 재사용한다.

### 선행 U03·U04 완료 이력

'''
assert status_text.count('## 현재 작업\n\n') == 1
status_text = status_text.replace('## 현재 작업\n\n', '## 현재 작업\n\n' + block)
status_text = status_text.replace('**현재 다음 작업은 U05/B02·B03·B04·B09 공명 접수 자격과 실제 사용 수명의 P2G-00→60이다.**',
    'U04 완료 직후에는 U05/B02·B03·B04·B09의 공명 접수·사용 수명 P2G로 인계했다.')
roadmap_text = original[roadmap]
old = '**상태: 2026-10-04 U00·U01·U02·U03·U04 독립 감사 Complete / High / U05 공명 접수·사용 수명 P2G 인수**'
assert roadmap_text.count(old) == 1
roadmap_text = roadmap_text.replace(old, '**상태: 2026-10-04 U00·U01·U02·U03·U04·U05 독립 감사 Complete / High / U06 기본 전투·치유·전술/시간 P2G 인수**')
old = '현재 다음 위치는 §5 U05/B02·B03·B04·B09 공명 접수 자격과 실제 사용 수명의 별도 P2G-00→60이다.'
assert roadmap_text.count(old) == 1
roadmap_text = roadmap_text.replace(old, 'U05는 [완료 인수](execution/u05_closure_20261004_001/RESULT.ko.md)와 [독립 재감사](../../goals/core_resonance_lifecycle/post_audit/u05_20261004_002/COMPLETION_AUDIT.md)로 전체 Complete / High다.\n외부 CF-D09 문서 이동의 현재성 finding을 새 대상 인수로 닫았으며 실제 비용 증가와 기존 R05 경고는 명시적으로 보존했다.\n현재 다음 위치는 §5 U06/B01·B05–B08·B10·B11·UC01·UC02 기본 전투·순수 치유·전술 및 시간 명령/조회 P2G-00→60이다.')
status.write_text(status_text)
roadmap.write_text(roadmap_text)
assert status_text.split('2026-10-04 · v1.', 1)[0] == original[status].split('2026-10-04 · v1.', 1)[0]
after = identity()
assert after == before
changed = [row['path'] for row in membership if sha(R / row['path']) != row['sha256']]
assert set(changed) == set(before_hashes), changed
links = []
for p, text in [(C / 'RESULT.ko.md', summary), (status, block)]:
    for target in re.findall(r'\]\(([^)]+)\)', text):
        if target == 'PUBLICATION_RECORD.json':
            continue
        assert (p.parent / target.split('#')[0]).resolve().is_file(), (p, target)
        links.append(target)
subprocess.run(['git', 'diff', '--check', '--', *map(str, paths)], cwd=R, check=True)
write(C / 'PUBLICATION_RECORD.json', {
    'kind': 'post-audit status-only successor', 'recorded_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'audit': str(A.relative_to(R)), 'audit_report_sha256': sha(A / 'COMPLETION_AUDIT.md'),
    'audit_closure_sha256': sha(A / 'AUDIT_CLOSURE.json'), 'end_currentness_sha256': sha(A / 'CURRENTNESS_EXIT.json'),
    'target_manifest_sha256': sha(T / 'MANIFEST.json'), 'documents_before': before_hashes,
    'documents_after': {str(p.relative_to(R)): sha(p) for p in paths},
    'documents_before_archive_sha256': sha(C / 'DOCUMENTS_BEFORE.tar.gz'),
    'source_before': before, 'source_after': after,
    'changed_bound_members': changed, 'checked_new_links': links,
    'non_impact': 'Only current progress/publication text in two status documents changes after audit closure. CF-D08/09 user authority sections, policy detail, product, test inputs, frozen targets and audit originals are preserved.',
    'next': 'U06 P2G-00 intake under existing continuous authorization',
})
print(json.dumps({'U05': 'Complete / High', 'status_documents': 2, 'product_unchanged': True, 'next': 'U06 P2G'}))
