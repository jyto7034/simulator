"""Write local distribution guidance. Never reads or writes the connected repository."""
from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[1]
M = json.loads((ROOT / 'roadmap_v2.2-review/GOALS.json').read_text(encoding='utf-8'))
VERSION = M['version']
DEST = 'core_enhanced/docs/roadmaps/content_authoring/reviews/2026-09-09-dependency-review-v2.2'


def write(rel: str, text: str) -> None:
    p = ROOT / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text.strip() + '\n', encoding='utf-8')


write('README.ko.md', f'''
# Simulator 콘텐츠 제작 기반 — 의존성 검토·개정 배포본

**{VERSION} · 2026-09-09 · 한국어 연구 보고서 / 저장소 미적용**

**권고: 기존 M0 성과와 GOAL-CAMEC-01을 유지하고, 후속 구현 순서를 최소 계약과 유효한 중간 상태 중심으로 재분할한다.** 최소 Goal 영향 판정은 **A**다. 이는 정식 current/stale 판정, GEX 자격·실행 승인 또는 게임 테스트 통과를 뜻하지 않는다.

## 읽는 순서

| 문서 | 확인할 내용 |
| --- | --- |
| [핵심 보고서](report/REVIEW.ko.md) | 권장 순서, 핵심 동시 전환, 21개 finding, 두 최종 질문의 답 |
| [최소 Goal 영향 보고](report/GOAL_CAMEC_IMPACT.ko.md) | A 권고, exact artifact, 기존 성과 재사용, 수정이 필요할 때의 owner |
| [M/CA 전체 대응표](report/M_CA_MAPPING.ko.md) | M0–M8·CA00–CA23의 유지·분할·재배치·미검토 구별 |
| [개정 로드맵](roadmap_v2.2-review/ROADMAP.ko.md) | R00 인수와 R01–R24의 하나의 기본 순서 |
| [의존성 판정](roadmap_v2.2-review/DEPENDENCIES.ko.md) | 기존 49개 관계의 개별 disposition, 새 76개 관계, 16개 동시 전환 |
| [개정 파일 색인](roadmap_v2.2-review/00_README.md) | GOALS.json, 25개 카드, 8개 아키텍처 문서, 계약·정책·검증·인수 안내 |
| [근거 색인](evidence/SOURCES.ko.md) | 현재 본문을 읽은 97개 파일의 경로·범위·내용 식별 한계 |
| [조사 한계](report/LIMITATIONS.ko.md) | 전체 COV 재검증·실행 시험·정밀 기하·현재성의 미완료 범위 |
| [문서 검사 결과](evidence/DOCUMENT_VALIDATION.ko.md) | 실제 로컬 정합성 검사와 게임·정식 검증 미실행의 구별 |
| [적용 안내](APPLY.ko.md) | 별도 경로에 새 파일로만 배치하는 방법 |
| [현황 추가 기록안](ROADMAP_STATUS_ADDENDUM.ko.md) | 기존 기록 0010·NEXT-0009를 보존하는 미적용 제안 |
| [변경 요약](CHANGELOG.ko.md) | 기존 v2.1과 비교한 변경 이유·보존 범위 |

## 확인한 것과 하지 않은 것

97개는 **기재된 본문 구간을 읽은 파일 수**다. 모든 파일의 완독, 전체 dirty snapshot, 전체 354 COV/127 BOSS-COV의 새 행별 표현력 검증을 뜻하지 않는다. 읽은 원본에서 관찰한 사실과 개정 설계 판단은 보고서에서 구별했다.

이번에는 **게임 구현·RON 이식·게임 빌드/시험·독립 감사·저장소 쓰기·NEXT-0009 실행을 하지 않았다.** 실제 수행한 자동 검사는 이 로컬 배포본의 문서·JSON·참조·후보 그래프 검사다. 검사기의 PASS를 P2G/GEX 또는 게임 완료로 사용하지 않는다.

## 파일 형식과 재현

`roadmap_v2.2-review/GOALS.json`은 연구용 버전 스키마 `content-authoring-roadmap-review/1`을 쓰는 **별도 개정 로드맵**이다. 정식 `GS-CAMEC-P2G50-001`이나 기존 배열형 로드맵 파일에 덮어쓰는 입력이 아니다. 원래 파일을 요구하는 도구와의 호환성을 인증하지 않았다.

`tools/validate_package.py`는 Python 표준 라이브러리만 사용하며, 이 배포 폴더 안의 파일을 읽고 `evidence/DOCUMENT_VALIDATION.*`만 쓴다. 저장소·네트워크·게임 명령을 호출하지 않는다.

```bash
python tools/validate_package.py
```

배포본의 `SHA256SUMS.json`은 **배포 파일**의 해시 목록이다. 연결된 simulator의 dirty source 해시가 아니다. 파일을 편집하거나 검사 결과를 다시 생성하면 그 배포 해시도 달라질 수 있다.

`input_package/`는 제공된 인수인계 ZIP의 문서 사본이다. 프로젝트 소스가 들어 있지 않으며, 그 안의 과거 sandbox 경로·검사 결과는 이번 결과로 승격하지 않는다. `tools/`의 다른 파일은 이 조사 문서를 작성한 로컬 생성 스크립트이며 필수 실행 절차가 아니다.
''')

write('APPLY.ko.md', f'''
# 별도 배포 경로 적용 안내

**상태: 미적용. 이 안내 자체는 저장소 변경·구현·배포 승인이 아니다.**

## 1. 배치 대상

원본 로드맵·정식 P2G/GEX 산출물을 보존하기 위해 다음과 같은 **새 연구 폴더**를 권고한다.

```text
{DEST}/
  README.ko.md
  report/
  evidence/
  roadmap_v2.2-review/
  ROADMAP_STATUS_ADDENDUM.ko.md
  ...
```

같은 경로가 이미 존재하면 내용을 합치거나 덮어쓰지 말고, 기존 목적·내용을 확인한 뒤 충돌하지 않는 새 이름을 정한다. 본 검토는 이 경로가 현재 비어 있다고 확인하지 않았고 실제로 만들지도 않았다. 압축본의 최상위 폴더 안 내용을 새 폴더에 배치하면 문서의 상대 링크가 유지된다.

[create-only diff](CREATE_ONLY.patch)는 이 배포 파일들을 위 새 경로에 추가하는 검토용 패치다. **기존 v2.1 파일을 수정하는 diff가 아니다.** 패치 자신과 해시 manifest는 패치에서 제외했다. 실제 적용 전에 작업 트리와 대상 경로 충돌을 확인해야 하며, 적용·검사 명령을 이번 연결된 저장소에서 실행하지 않았다.

## 2. 보존할 원본

`ROADMAP_STATUS.md` v1.9의 기록 0010과 이전 이력, NEXT-0009, CA00 요구/COV, `WORKING_POLICY.ko.md`, 기존 Review·Decision·Freeze·Phase Plan·Receipt·Goal Set·Handoff·감사, AGENTS/P2G/GEX/검증기, production 코드·RON·저장은 그대로 둔다.

이 배포본은 새 정식 Topic, successor, Change Declaration, Goal activation 또는 실행 기록을 만들지 않는다. 원래 미커밋 변경을 reset/stash/삭제/재포맷하는 정리 단계도 없다.

## 3. 현황에 연결하는 방법

[현황 추가 기록안](ROADMAP_STATUS_ADDENDUM.ko.md)은 **문서 채택 뒤 덧붙일 내용의 제안**이다. 기존 최근 기록의 번호를 미리 소비하지 않는다. 실제 현황 owner가 변경된 최신 기록 번호를 확인하고, 새 연구 위치·적용 여부를 연결한다. NEXT-0009를 지우거나 자동으로 다른 실행 카드로 교체하지 않는다.

정식 최소 Goal의 순서는 기존 `PP-CAMEC-P2G40-002`가 소유한다. 본 연구의 R01은 그것을 유지하는 A 권고다. 후속 R 순서는 별도 로드맵 권고이므로, 아직 존재하지 않는 정식 Phase/Goal의 확정 실행 순서로 표시하지 않는다.

## 4. 후속 정식 절차와 책임

최소 Goal의 실행 여부는 새로운 명시적 실행 요청과 GEX 인수에서 결정한다. 이 패키지를 배치하거나 읽는 것만으로 실행이 승인되지 않는다.

이후 실제 기능을 위한 정식 작업을 선택할 때는 필요한 최소 계약을 각 owner에서 인수한다. 새 gameplay·수명·미정 정책은 P2G-10, 실제 source/consumer 근거·구현 delta는 P2G-20, Phase·동시 전환·검증 owner는 P2G-40, Goal의 표현/누락은 P2G-50의 책임이다. 기존 불변 artifact를 직접 덮어쓰지 않는다. 본 연구는 그 절차를 수행하지 않았다.

개정 `GOALS.json`은 형식이 다른 연구용 객체이므로 기존 배열을 기대하는 프로그램에 그대로 전달하지 않는다. 별도 위치에서 읽거나 후속 문서 owner가 명시적으로 소비자를 조정한다. 기존 정식 검증기를 우회·변조할 근거로 쓰지 않는다.

## 5. 적용 전 내용 식별

읽기 기준은 simulator의 branch `cleanup/simulator-dirty-worktree-20260803`, short HEAD `f318414`, dirty=true였다. 시작과 마지막의 현황 선택은 같았지만, dirty 전체의 원자적 해시/사본은 없다. 각 판단의 영향 source는 [SOURCES](evidence/SOURCES.ko.md)의 범위·심볼로 재대조한다. 관련 부분이 바뀌었으면 그 판단만 갱신하고 과거 값으로 되돌리지 않는다.

로컬 검사:

```bash
python tools/validate_package.py
```

이 명령의 통과는 배포 파일 내부 정합성만 확인한다. 게임 동작, 실제 팩 로딩, 저장 복원, 독립 제작, GEX 자격 또는 서버/Unity 호환성을 보장하지 않는다.
''')

write('ROADMAP_STATUS_ADDENDUM.ko.md', f'''
# ROADMAP_STATUS 추가 기록 제안 — 저장소 미적용

**이 파일은 원본 ROADMAP_STATUS.md의 교체본이 아니다. 기록 번호 미배정 / 적용하지 않음.**

## 기준 관찰

| 항목 | 마지막으로 읽은 값 |
| --- | --- |
| 현황 문서 | v1.9 · 2026-09-09 · 기록 0010 |
| 현재 완료 | NEXT-0008에서 P2G-60 Handoff 작성 완료 |
| 다음 카드 | NEXT-0009 — 기존 원문 그대로 보존 |
| 정식 Goal | GS-CAMEC-P2G50-001 / GOAL-CAMEC-01 |
| 상태 | Authored / Not Activated / Implementation Authorized: No |
| 이번 저장소 적용 | **아니오** |
| 이번 NEXT-0009 실행 | **아니오** |

근거: [종료 관찰](evidence/workspace_closing_observation.json) · [S001 원문 범위](evidence/SOURCES.ko.md#s001).

## 실제 문서 채택 시 덧붙일 기록의 내용

아래 내용은 현황 owner가 **당시 최신 기록 번호를 확인한 뒤** 새 기록에 연결할 제안이다. 현재 기록 0010 및 이전 실패·완료 이력을 삭제하거나 재서술하지 않는다. 현황의 최근 기록 보관 규칙을 바꾸지 않는다.

> **콘텐츠 제작 기반 전체 의존성 연구·로드맵 개정안 수령 ({VERSION}).** 기존 M0 조사·최소 계약 Handoff를 인수하고, 현재 코드의 생산자·소비자·저장·종료 경계를 바탕으로 기존 49개 관계를 재분류했다. R00 인수 + R01–R24의 기본 순서, 76개 typed 관계, 16개 최소 동시 전환과 M0–M8/CA00–CA23 대응을 별도 연구 배포본에 작성했다.
>
> **최소 Goal 영향: A 권고.** GOAL-CAMEC-01과 PP-CAMEC-P2G40-002를 첫 구현 단위로 유지한다. 새 정식 current/stale·GEX 자격·구현 승인은 발행하지 않았다.
>
> **검증:** 로컬 문서·JSON·후보 그래프 정합성 검사만 수행했다. 게임 시험·전체 354 COV/127 BOSS-COV의 fresh 행별 표현력 검증·정밀 기하 검토·독립 감사는 완료하지 않았다. 해당 한계와 재개 위치를 연구 보고서에 유지한다.
>
> **권한·보존:** production/RON/저장/기존 정식 artifact/규칙·검증기 변경 없음. NEXT-0009는 실행하지 않았으며 기존 다음 카드와 이력을 보존한다. 연구 파일의 실제 배치 위치·채택 여부는 이 기록을 적용하는 owner가 사실대로 기입한다.

## 현황 표에 추가할 연결 제안

| 추가/보완할 조회 항목 | 제안 내용 |
| --- | --- |
| 로드맵 검토 자료 | 기존 v2.1은 보존. 별도 {VERSION}은 연구 권고이며 미실행 |
| 순서의 코드 근거 | 연구 DEPENDENCIES·카드와 source ledger 연결; formal 최소 Plan은 기존 원본 유지 |
| 남은 문제 | 기존 GEX 인수·구현/시험/감사 + 연구의 조건부/미검토 범위 유지 |
| 실제 적용 여부 | 현재 **미적용**. 실제 배치 후에만 경로와 채택 상태를 기록 |

[보고서](report/REVIEW.ko.md) · [Goal 영향](report/GOAL_CAMEC_IMPACT.ko.md) · [적용 안내](APPLY.ko.md).
''')

write('CHANGELOG.ko.md', '''
# v2.1 → 2.2-review.1 변경 요약

**별도 연구 배포본. 기존 v2.1과 정식 P2G/GEX 원본은 수정하지 않았다.**

| 변경 | 이전 기준과 이번 개정 |
| --- | --- |
| 순서의 근거 | v2.1의 후보 선행 49개를 각각 유지·세분화·동시 전환·권장·조건부로 disposition했다. 화살표 삭제만으로 독립성을 선언하지 않는다. |
| 단위 재분할 | CA 번호를 원요구 추적용으로 유지하고, R00 인수 + R01–R24의 기본 순서와 실제 유효 상태를 별도 카드로 작성했다. |
| 최소 Goal | 현 formal metadata slice는 유지(A). 외부 팩·전체 Flow·사본·보스 전체를 새 필수 선행으로 붙이지 않는다. |
| 도구의 선행 | 공개 CLI/headless는 먼저 권장하되, 기존 kernel fixture로 수행 가능한 개발·검증을 전체 도구 완료까지 차단하지 않는다. |
| 보스 | 명시적 요청/완료/실패/취소/방향의 최소 task adapter를 R08로 분리했다. 기존 graph·SpawnKernel은 재사용하고 고급 전장 기믹은 R18로 남겼다. |
| 침식 | 전체 유물·보스 완성 뒤라는 큰 선행을 제거하고 R09로 분리했다. 실제 시전·정상 wave bookkeeping·부가 효과 억제·직원 대가·강제 시계·Breakdown을 함께 배정했다. |
| 파편 사본 | 종류별 count/progress를 바꾸는 순간 지급·장착·개화·선택·분쇄·회수·profile·snapshot/save도 전환하도록 R07에 명시했다. |
| 유물 합성 | 최소 numeric 합성부터 지연 projectile/area의 전역 SkillId 재조회 경계를 같이 전환한다. RelicCopy 관리와 composer는 별도 책임을 유지하되 기본안의 인수 단위는 R14다. |
| 저장·예산 | v2.1 A07/CA19/CA22에도 조기 동반 원칙이 있었다. 이를 새 주장으로 포장하지 않고, 각 R의 owner·필요 상태·실패·예산 시험으로 구체화했다. |
| 등록·획득 | 현재 1 fragment reward source와 plan/commit/validator를 확인하고, 복수 파편 선택·등장/획득 연결의 공동 전환을 R19에 배정했다. |
| 문서 형식 | GOALS.json은 연구용 명시 스키마 객체다. 기존 배열형 loader나 정식 Goal Set과의 호환성을 주장하지 않으며 별도 경로에만 배포한다. |
| 증거/한계 | 본문을 읽은 97개 파일 구간, 21개 finding, 9개 제한을 기록했다. 미실행 계획·과거 증거·이번 문서 검사·독립 감사는 구별한다. |

## 보존

제3자의 RON 조합 제작, 공통 엔진 확장 허용, 조합기 UI 제외, 외부 코드 실행 제외, 명시적 다중 팩/교체/충돌 검증, 새 런 세대 pin, 개별 파편 개화, 승인 유물/성장/침식 의미, 15개 분해 축, K01–K10, PG00–PG09, T00–T29 및 C01–C07을 보존한다. 대표 공식 RON 이식과 전체 합의 구조의 표현력 대조는 서로를 대체하지 않는다.

기존 CA00의 354 COV/127 BOSS-COV는 원본을 그대로 참조하며 새 실행 결과로 바꾸지 않는다. 이번 작업에서 모든 개별 행을 다시 검증하지 않은 한계는 [LIMITATIONS](report/LIMITATIONS.ko.md)에 명시했다.

## diff의 의미

[CREATE_ONLY.patch](CREATE_ONLY.patch)는 새 연구 폴더의 파일 추가를 보여주는 실제 unified diff다. 원본 v2.1의 모든 바이트를 로컬에 확보한 것이 아니므로 가상의 원본/수정본 비교를 만들지 않았다. 의미 변화는 위 표와 [기존 관계별 판정](roadmap_v2.2-review/DEPENDENCIES.ko.md)을 사용한다.

새 게임 정책·밸런스를 결정하지 않았다. 미정 family는 해당 PG와 owner가 결정하기 전 그 기능의 완료를 주장할 수 없다. 공식 범위 확대나 기존 정식 Plan 변경은 후속 정식 owner의 절차를 요구하며 이번 배포에서는 실행하지 않는다.
''')

card_rows = '\n'.join(f"| {u['id']} | [{u['name']}]({u['card']}) |" for u in M['units'])
arch = [p.name for p in sorted((ROOT/'roadmap_v2.2-review/architecture').glob('*.md'))]
arch_rows = '\n'.join(f'- [{x}](architecture/{x})' for x in arch)
write('roadmap_v2.2-review/00_README.md', f'''
# 개정 로드맵 파일 색인

**{VERSION} · 연구 권고 / 정식 Goal Set 아님 / 저장소 미적용**

[핵심 보고서](../report/REVIEW.ko.md)에서 결론과 한계를 먼저 읽는다. 원래 CA/M는 요구 추적을 위해 남기고 R 번호는 권장 구현 단위를 식별한다.

| 문서 | 역할 |
| --- | --- |
| [ROADMAP](ROADMAP.ko.md) | 하나의 기본 순서와 중간 상태 |
| [DEPENDENCIES](DEPENDENCIES.ko.md) | 49개 기존 관계 disposition·76개 새 관계·16개 동시 전환 |
| [GOALS.json](GOALS.json) | 카드·표의 연구용 구조 데이터 |
| [CONTRACTS](CONTRACTS.ko.md) | K01–K10와 처음/확장 owner |
| [POLICY_GATES](POLICY_GATES.ko.md) | PG00–PG09·승인 의미·미정 조건 |
| [ACCEPTANCE](ACCEPTANCE.ko.md) | T00–T29·주/보조 owner·시험 입력·기대 결과 |
| [COVERAGE](COVERAGE.ko.md) | 대표 C01–C07와 전체 표현력 대조의 분리 |
| [HANDOFF](HANDOFF.ko.md) | 기존 M0를 보존한 후속 인수 안내 |
| [적용 안내](../APPLY.ko.md) | 원본을 덮어쓰지 않는 별도 배치 |

## 작업 카드

| ID | 실제 개정 카드 |
| --- | --- |
{card_rows}

## 아키텍처 문서

{arch_rows}

## 검사 경계

[문서 검사 결과](../evidence/DOCUMENT_VALIDATION.ko.md)는 이 배포본 내부의 참조·ID·순서·소유 관계만 확인한다. 실제 parser/compiler/runtime의 구현·시험·독립 감사는 수행하지 않았다. [현재 source](../evidence/SOURCES.ko.md)와 [한계](../report/LIMITATIONS.ko.md)를 함께 인수한다.
''')

# Canonical JSON remains authoritative for a readable rendered card.
for p in (ROOT/'roadmap_v2.2-review/goals').glob('*.md'):
    t = p.read_text(encoding='utf-8')
    t = t.replace('보존한다..', '보존한다.')
    p.write_text(t, encoding='utf-8')
print('Wrote README, APPLY, CHANGELOG, unapplied STATUS addendum and roadmap index.')
