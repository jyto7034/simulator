# R08~R12 연속 구현 지시서

2026-09-30 · v1 · 상태: 코드 조사에 근거한 구현 요청 문서. 정식 P2G 검증·제품 구현·감사 완료를 뜻하지 않는다.

## 1. 요청과 종료점

현재 콘텐츠 제작 기반 로드맵의 **R08~R12를 구현하고, 최종 통합 소스에서 자체 검증한 뒤 감사 대상으로 동결**한다.
보스의 명시 스킬 호출, 파편 침식, 구조화 스킬 흐름, 지속 효과·집중·기본 공격 교체, 보호막·방어 중재를
실제 공개 데이터 입력과 전투 소비까지 연결한다. R별 내부 경계에서 근거를 남기되 매번 사용자 검토 대기로 끝내지 않는다.

이번 문서는 사용자가 구현 담당자에게 전달할 지시서다. 이 문서를 작성한 대화에서는 제품 코드를 변경하지 않았다.
구현 시작 요청은 [시작 요청문](R08_R12_START_PROMPT_20260930.ko.md)을 사용한다.

- 필수 상세 설계·수용표: [코드 기반 구현 방향](R08_R12_DESIGN_AND_ACCEPTANCE_20260930.ko.md).
- 조사 시점의 파일 identity: [코드·문서 조사 근거](R08_R12_CODE_EVIDENCE_20260930.json).
- 절차: [GEX `2026-09-13-gex-r7`](../../../goal_execution_pipeline%281%29.md),
  [P2G](../../../policy_to_goal_pipeline_revised_v2.md).
- 참고 템플릿: [부분 구현 지시서](../../../templates/IMPLEMENTATION_CHECKPOINT_REQUEST.template.ko.md),
  [시작 요청문](../../../templates/IMPLEMENTATION_START_PROMPT.template.ko.md)의 `2026-09-27-v1`.
  **이번에는 사용자가 R08~R12 전체 범위를 요청했으므로 템플릿의 ‘첫 부분 구현에서 중단’은 적용하지 않는다.**
  코드 기반 설계, 세 기록, 실패 보존, 전후 보존, 검토 분리는 그대로 적용한다.

최종 종료 문구는 **R08~R12 구현·자체 검증 종료 / 최종 통합 target 동결 / 독립 감사 요청 대기**다.
구현자가 독립 감사 PASS나 `Complete / High`를 발행하지 않는다. R13 이상과 자동 후속 감사는 시작하지 않는다.
필수 선행이나 필수 기능이 충족되지 않았다면 이 완료 문구 대신 실제 부분 상태·남은 조건을 보고한다.

## 2. 선행 상태 — ‘감사 종료’로 잘못 인수하지 않기

사용자는 처음에 R06~R07 감사 종료라고 설명했으나, 경로 확인에 대한 후속 답변은 아래 **구현·자체 검증 인계**이며
“독립 감사는 수행하지 않았습니다”라고 명시했다. 작성자가 읽은 현재 현황과 인계서도 감사 대기다.
이 요청 문서로 과거 구현 인계서를 독립 감사 결과로 바꾸지 않는다.

| 선행 자료 | 확인한 내용 |
| --- | --- |
| [R06·R07 최종 인계](../../../goals/content_authoring_fragment_copies/execution/single_goal/REVIEW_HANDOFF.md) | 기본/feature 각각 1,581개 통과라는 구현자 보고. 이번 작성자가 제품 시험을 재실행한 결과가 아님 |
| [최종 manifest](../../../goals/content_authoring_fragment_copies/execution/single_goal/target/r06_r07_integrated_001/MANIFEST.json) | 사용자 제공 SHA-256 `22b61c4eb0518fcc11209eaf00e415a4f8b6f283c73037d9a04e8e5c168df59a` |
| [R06 경계 manifest](../../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/continuations/r06_completion_001/target/r06_boundary_001/MANIFEST.json) | SHA-256 `0b6938d77cc6cded598b2b01e51d41bf7e651dd66e8acf2f695a1cf3b5fd44f1` |
| 최종 독립 감사 | 작성 시점에는 확인되지 않음. 실제 감사 경로·verdict·target·잔여 finding을 인수할 때 연결 |

이번 **연속 구현 전체를 시작하기 전** R06/R07의 필요한 독립 감사와 현재성부터 확인한다.
감사 대기이면 본 지시서의 조사·정식 계획 준비는 할 수 있으나, 감사가 끝났다고 가정해 종속 제품 구현을 시작하지 않는다.
이 지시서는 R06/R07 감사를 수행하라는 요청이 아니다. 별도 감사 요청으로 해당 절차를 닫고 재개한다.
R08 자체의 직접 기능 선행은 R01/R04이며 R06/R07 전체가 아니다. 다만 이번 전체 묶음의 R09/R10이 R07/R06을 소비하므로
착수 조건을 통합해 둔다. 불필요한 R08 재설계 선행으로 R06/R07을 추가하는 뜻은 아니다.

선행의 source delta는 범위별로 조사한다. 허용된 후속 변경 때문에 영향받는 과거 감사가 stale해지면 영향 조건을 기록하고
후속 감사에 인계한다. 과거 감사 원문 수정·자체 회귀를 독립 감사로 표시하는 방법으로 해결하지 않는다.

## 3. 구현 기준과 비범위

현재 [현황 진입점](../ROADMAP_STATUS.md), 아래 카드, 인수한 최신 승인 계약을 기준으로 한다.
오래된 카드의 `proposed_not_implemented`는 작성 당시 상태이며 현재 코드 조사와 정식 계획을 대신하지 않는다.

| 범위 | 책임 | 기능 선행 |
| --- | --- | --- |
| [R08](../reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R08.ko.md) | 보스 스킬 요청·완료·실패·취소·방향 adapter | R01/R04 |
| [R09](../reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R09.ko.md) | 침식·강제 정지·직원 대가·Breakdown | R04/R05/R07 |
| [R10](../reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R10.ko.md) | 대상·전달·구조화 Flow·유한 반복 | R01/R04/R05/R06 |
| [R11](../reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R11.ko.md) | 가역 modifier·membership·channel·기본 공격 교체 | R05/R06/R10 |
| [R12](../reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R12.ko.md) | 개인/공유 보호막·생존·제어·HP 비용 | R04/R05/R06/R11 |

R08→R09→R10→R11→R12를 기본 구현 순서로 사용한다. R10은 R08 요청 포트와 R09 침식 gate를 유지하고,
R11/R12는 R09의 강제 정지·침식 부가 효과 억제를 함께 인수한다. AT08~AT12의 동반 전환을 빠뜨리지 않는다.
의존 관계 원본은 [DEPENDENCIES](../reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/DEPENDENCIES.ko.md),
정책 family는 [POLICY_GATES](../reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/POLICY_GATES.ko.md)다.

사용자는 **현재 로드맵을 먼저 구현하고 차기 원정 전환은 나중에 별도 수행**하기로 했다.
[현행 룰북](../../../game_rulebook.md)과 이미 승인된 주제별 계약을 현재 구현 기준으로 삼는다.
[차기 룰북](../../../game_rulebook_next.md)의 최대 깊이8·시설 개방·억제력·황금가지·성공 귀환·반납/대여·강화 반출 초기화는
이번 범위에 추가하지 않는다. 미래 전환에서 영향을 받을 현재 런/저장 가정은 NOTES에 남긴다.
이미 R06/R07에서 승인·구현한 주문력·사본별 개화·형태 선택을 옛 룰로 되돌리지 않는다.

다음도 제외한다.

- 편집기 GUI, Unity·game_server 제품 기능 확장, 마켓·배포, 외부 코드 실행, 일반 함수 정의/인자/반환을 갖는 범용 언어.
- 실험실 스트레스 인증·캐시·모드 적용, metrics, 기존 안전값 RON 분리. 별도 안전 로드맵의 범위다.
- **20분 시험은 모든 단계에서 제외**. 짧은 fake-clock·유한 반복·경계 부하로 검증한다.
- 예산 초과 시 자동으로 스킬을 취소하는 새 gameplay 정책. 기존 typed fault/transaction rollback은 유지한다.
  R08 task 취소·R11 channel 중지 같은 기능상 취소는 필요한 구현 범위다.
- R13 소환/설치물/진화 전체, R14~R17 유물 합성·연쇄·연결망, R18 보스 전체 개편, R19~R24 전체 완료.
  이미 존재하는 소환·웨이브는 보존하고 이번 경계 회귀에 사용한다.
- 대표 기능을 입증하는 외부 RON fixture와 본편 콘텐츠 전체 이식을 혼동하지 않는다. 전체 콘텐츠·밸런스 완료는 요구하지 않는다.

## 4. 계획 준비와 GEX 연결

R08~R12 카드만으로 정식 Goal이 있다고 가정하지 않는다. 먼저 같은 범위의 기존 P2G 산출물 유무를 확인해 재사용한다.
없다면 권장 신규 위치는 `core_enhanced/docs/goals/content_authoring_r08_r12_runtime/`다.
현재 이 경로는 **구현 담당자가 정식 계획을 만들 위치**이며, 이 지시서 작성으로 P2G PASS가 발생한 것은 아니다.

이번 시작 요청에는 필요한 P2G 정책 정리·실제 코드 대조·동결·계획/Goal/Handoff 작성·검증과 이어지는 GEX-00/10을 포함한다.
기존 validator를 찾아 실제 실행하고 명령·결과를 기록한다. 문서 링크 검사를 P2G 검증으로 바꾸지 않는다.

권장 구조는 **하나의 통합 Single Goal 안에 WP08~WP12와 최종 통합 검증을 두는 것**이다.
공통 action/flow/lifetime/damage 계약을 한 최종 source에서 인수하는 목적이며, R별 수용표와 변경 책임은 분리한다.
R 번호나 내부 WP를 독립 GEX Phase로 잘못 선언해 매 단계 선행 감사가 필요해지는 구조를 만들지 않는다.
이는 기존 동결 Goal들을 합치라는 뜻이 아니다. 신규 P2G에서 실제 원자적 전환 경계·Goal 크기·의존성을 검토해 확정한다.
실제 근거상 multi-phase만 유효하면 그 이유와 감사 선행을 계획에 명시한다. 연속 실행을 위해 필수 Phase 감사를 몰래 생략하지 않는다.
그 경우 이번 ‘최종 감사만 별도’ 요청과 충돌하는 경계를 정확히 보고하며, 구현자가 자동 감사 권한을 만들지 않는다.

필수 선행이 충족되고 정식 Handoff가 준비되면 GEX-00 인수와 GEX-10 구현을 진행한다.
중간 WP에서 focused 검증·비교 가능한 경계 source를 보존하고 다음 WP로 계속한다. 전체 최종 target에서 모든 수용 조건을 다시 매핑한다.
설명·계획 작성만으로 구현 요청을 끝내지 않는다. 구현 전 before와 최종 after는 필수 보존이며 내부 WP마다 거대한 전체 archive를 복제할 필요는 없다.

## 5. 정책 결정 권한과 미정 항목 처리

이전 사용자의 ‘부재 중 질문에 답할 수 없으므로 재량으로 정책을 결정하고 구현을 계속’한다는 위임을,
이번 R08~R12 범위의 기존 승인 방향을 보존하는 통상적 정책 판단에 적용한다. **차기 룰북의 미합의 원정 정책까지 위임받은 것은 아니다.**

- 이미 확정된 정책·기존 공개 계약을 먼저 찾는다. 그와 충돌하는 추천안을 그대로 채택하지 않는다.
- 내부 자료구조·함수 분리·동등 최적화는 구현자가 결정한다.
- 필요한 세부 gameplay 결정은 P2G-10에 판단 근거·대안·영향·되돌릴 방법·시험 oracle을 남겨 닫고 후속 artifact를 갱신한다.
  확정 사용자 답변과 위임으로 채택한 잠정 구현 정책을 구별한다.
- 동결 뒤 정책 변경은 successor 절차로 반영한다. GEX 구현 메모만으로 원문 계약을 바꾸지 않는다.
- 신규 영구 자산 손실, 차기 원정 정책 변경, 실제 사용자 파일 삭제/덮어쓰기, 외부 배포 등 범위 밖 행동은 위임에 포함되지 않는다.
- 필수 기능의 정책을 미지원으로 남기고 R08~R12 완료라고 선언하지 않는다. 실제 해결할 수 없으면 해당 조건을 미완료로 남긴다.

[상세 설계 §7](R08_R12_DESIGN_AND_ACCEPTANCE_20260930.ko.md#7-정책-결정표)의 권장값은 출발점이다.
특히 침식 cast-start, 공명 소비, Breakdown 회복/만료 순서, 개인/공유 보호막 순서, HP1과 HP 비용,
패링의 피해·부착 효과 범위, channel의 저지, 같은 시각 재입력/만료는 코드에 숨긴 default로 두지 않는다.
새 숫자는 시험 입력 또는 명시적 잠정 콘텐츠값으로 데이터에 작성하고 사용자 확정 밸런스라고 보고하지 않는다.

## 6. 실행 기록과 작업 보호

사용자 진입점은 기존 `docs/roadmaps/content_authoring/ROADMAP_STATUS.md` 하나다.
새 실행 위치는 정식 Handoff가 선택하며 신규 기본값은 `docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/`다.

| 문서 | 책임 |
| --- | --- |
| `PLAN.md` | exact Goal/Handoff·현재 WP·현재 baseline/target·진행·남은 조건·다음 행동의 단일 위치 |
| `EXPERIMENTS.md` | 의미 있는 시도·실패·교정·검증을 당시 source/명령/환경/exit/원자료와 함께 보존 |
| `EXPERIMENT_NOTES.md` | 채택·폐기한 설계, 위임 정책의 근거, 미검증 가정, 미래 원정 전환에 영향받는 부분 |
| `COVERAGE.json` | 상세 지시서의 A08~A12/X 조건, 정식 Goal 요구, actual test selector·기대값·원자료 대응 |
| `REVIEW_HANDOFF.md` | 최종 source, WP별 완료/제약, 정책 변경, 검증·실패 이력·동결 자료를 연결하는 요약 |

재개할 때 세 기록의 현재 부분과 직접 선행 실패/감사를 읽는다. 같은 실패를 가설 변화 없이 반복하지 않는다.
컨텍스트 소진이나 대화 자동 재개는 새 작업이 아니다. PLAN에 남은 조건을 인수해 계속한다.
실행 중 현재 WP와 발견한 위험·다음 검증을 사용자에게 알리고, 장시간 명령도 진행 로그를 노출한다.

현재 저장소에는 untracked 문서·제품 파일이 많다. git HEAD만 before로 쓰지 말고 실제 dirty/untracked bytes를 보존한다.
다른 대화 작업을 reset/clean/stash하거나 이번 성과로 귀속하지 않는다. 같은 파일의 병행 변경은 currentness로 확인하고
안전하게 구분되지 않으면 충돌 범위를 보고한다. 단일 writer로 제품 변경을 수행한다.

## 7. 검증 명령과 회귀

작업 루트: `/mnt/f/work/simulator` (Windows `F:\work\simulator`). 패키지는 `game_core`, Rust 2021이다.
현재 검사 명령은 아래와 같다. 필요한 환경을 확인한 뒤 사용하고 개인 환경 경로가 없다는 이유로 설정을 덮어쓰지 않는다.

```bash
cargo check -p game_core --all-targets --offline --locked
cargo test -p game_core --all-targets --offline --locked
cargo test -p game_core --all-targets --features test-fixtures --offline --locked
cargo check -p game_core --all-targets --features event_log_checks --offline --locked
cargo build -p game_core --release --bins --offline --locked
cargo clippy -p game_core --all-targets --offline --locked -- -A clippy::never_loop -A clippy::eq_op
cargo fmt --all -- --check
```

Clippy의 두 allow는 R06/R07 검증 명령에서 이어받은 것이다. 새 경고를 숨기기 위해 allow를 늘리지 말고 기존 경고와 새 결함을 구분한다.
기본 병렬 시험을 사용한다. 모든 시험에 `--test-threads=1`을 붙이지 않는다. 실제 충돌이 재현된 selector만 이유와 함께 격리한다.
루트 fmt가 무관한 기존 파일에서 실패하면 baseline 실패와 이번 파일 검사 결과를 구별한다. 무관한 포맷을 일괄 수정하지 않는다.

WP마다 관련 focused 시험과 변경 영향 회귀를 수행하고, 광범위 검증은 초기 기준선과 최종 통합 source를 중심으로 수행한다.
이미 같은 source·조건으로 유효한 baseline이 있으면 적용성을 입증해 재사용한다. 최종 broad 회귀는 공유 실행부가 바뀌므로 새로 실행한다.
0개 시험, 필터 오타, interrupted, receipt 부재는 통과가 아니다. 실제 명령·건수·실패·exit를 남기고 예상 시험 수를 복사하지 않는다.

반드시 포함할 실제 소비자 검증:

1. 공개 RON→loader/검증→compiled plan→보스 또는 직원 action→실제 HP/상태/완료 fact→조회/결과.
2. [실제 전투 통합 시험](../../../../tests/skill_runtime_contract.rs)과
   [World 전투 경로](../../../../src/game/world/tests/combat.rs)의 영향을 확인하고,
   새 기능을 정상 GameCore 명령·시간 진행으로 실행하는 종단 시험을 추가한다. private unit fixture만으로 완료하지 않는다.
3. release `author_content` CLI의 실제 지원 명령/입력 schema를 확인해 validate/preview/explain/simulate 경로를 검증한다.
   trace on/off가 gameplay·amount·오류 의미를 바꾸지 않는다. fixture 패치를 엔진 특례로 넣지 않는다.
4. PC36·R05의 기존 실행기/원자료를 인수해 최종 source에서 재측정한다. 의미 변화가 있는 fixture는 전후와 이유를 보존한다.
   R05 concurrent allocation peak의 **원 cap 9,030,188 bytes 단독 초과는 경고**다.
   R07의 초과 184,281 bytes는 과거 측정이며 새 목표 상한이 아니다. 다른 hard guard·결정론·parity 실패는 유지한다.
5. 새 상태의 allocation 전 admission, retained extent, native clone, rollback, 최신 저장 형식 복원과 R06/R07 영향 회귀.
   예산/취소/원자성 검증을 무력화하거나 측정 workload를 축소해서 통과시키지 않는다.

기존 R07 `run.py`, `freeze.py`, PC/R05 도구를 조사해 재사용한다. **동결된 실행 디렉터리에 새 로그를 쓰거나 기존 경로를 그대로 실행하지 않는다.**
필요한 wrapper를 새 실행 위치에 복사·조정하고 경로·membership·출력 대상·환경 변수를 확인한다.
모든 신규 source/test/RON/config/lock/필수 fixture와 실제 변경을 source 집계에 포함한다. `target`/캐시를 순회하지 않도록 제외한다.

## 8. 동결·인계와 완료 판정

before/after를 byte 단위로 복원할 archive 또는 실제 동등한 방식, 추가·삭제를 포함한 정/역 patch,
입력 membership와 sha256 manifest를 남긴다. manifest 자기 자신과 미래 감사 보고서는 자기 해시 대상에서 제외한다.
격리 디렉터리에서 양방향 복원과 최종 currentness를 검증하고, 원본 작업 트리에 역 patch를 적용하지 않는다.
동결 뒤 제품 source가 바뀌면 같은 target을 수정하지 말고 successor와 영향 재검증으로 인계한다.

인계서에는 다음을 포함한다.

- R08~R12 각각의 수용표와 T17/26, T19/20/29, T08/23/26/27, T09/27, T10/19/27의 실제 대응.
- final source 기준 R06/R07 영향, 이전 감사의 재사용 가능 범위·새 감사 필요 범위.
- 실제 정책 결정과 원래 추천안의 차이, 미지원 조합과 이유, 미래 원정 전환에 영향받는 현재 계약.
- 원자료와 세 기록, 초기 baseline·내부 WP 경계·최종 통합 target, 실패/중단/재시도 이력.
- 공개 데이터 경로·실제 전투·저장·오류·성능의 한계. 추가한 primitive가 공식 콘텐츠 전체 완료를 뜻하지 않음.
- **독립 감사 미수행**, 별도 감사가 확인할 조건. 정식 Single Goal이면 GEX-20 대상이며 존재하지 않는 Master/GEX-30을 만들지 않음.

필수 수용 조건이 하나라도 미완료이면 전체 구현 완료를 주장하지 않는다.
최종 동결이 끝나면 제품 mutation을 멈추고 별도 독립 감사 요청을 기다린다.
