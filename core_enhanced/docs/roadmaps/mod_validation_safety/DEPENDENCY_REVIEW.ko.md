# 편입 의존성 검토

2026-09-27 · 범위: 기존 R 순서와 새 MS 작업의 연결을 정할 만큼의 코드 조사. 후반 모든 함수의 상세 설계/실행 검증은 아니다.
기준 workspace와 관련 bytes는 [ENTRY](evidence/ENTRY.json)에 기록했다. 기존 dirty는 보존하며 파일 존재를 구현 완료 증거로 쓰지 않는다.

## 1. 기존 순서에서 보존할 관계

[v2.1 의존성](../content_authoring/roadmap/DEPENDENCIES.ko.md)은 후보·코드 검토·실행 준비를 구분한다.
[2.2-review.1 의존성](../content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/DEPENDENCIES.ko.md)은
기능 필수·동시 전환·검증·권장·조건부 관계를 나눈다. 정식 current Phase Plan이 실제 범위의 원본이라는 규칙을 유지한다.

| 기존 owner | 원래 맡은 일 | 새 작업과의 disposition |
| --- | --- | --- |
| [R02](../content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R02.ko.md) | 팩 선택·정적 검증 세대·run pin·save fingerprint | MS03/MS05가 재사용. 과거 완료 R02를 새 스트레스 승인까지 완료했다고 확장하지 않음 |
| [R03](../content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R03.ko.md) | 공개 validate/describe/explain/simulate·trial | MS03 실행 포트 재사용. 별도 가짜 executor 금지 |
| [R19](../content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R19.ko.md) | 등장·획득·등록·평가 모델의 통합 | MS05 승인 구성의 후속 소비자. 이름이 '등록'이라는 이유로 캐시·승인 전체를 R19에 넣지 않음 |
| [R20](../content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R20.ko.md) | 이미 도입한 상태의 결과·복원·저장 교차 검증 | MS04/MS05의 새 상태/저장 의무는 도입 때 완료. R20에서 종합 확인 |
| [R21](../content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R21.ko.md) | 대표 공식 RON 통합 | 최초 측정 fixture를 독점하지 않음. MS03/MS05는 현재 기능 fixture로 먼저 측정, R21 데이터는 MS06-A 보강 입력 |
| [R22](../content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R22.ko.md) | 엔진 동결 후 독립 data-only 인수 | 동결 전에 승인·시험 경로와 기준이 있어야 독립 작성자가 실제 경로를 사용할 수 있음 |
| [R23](../content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R23.ko.md) | 통합 결정론·안전·성능 닫힘 | MS06-B 증거 owner로 재사용. 최초 안전 기능/첫 승인 기준을 이때까지 미루지 않음 |
| [R24](../content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R24.ko.md) | 독립 완료 감사 | 새 scope 대응을 기존 감사에 연결. 별도 중복 감사/완료 판정 없음 |

R01–R05 완료와 R06 부분 구현은 [현재 현황](../content_authoring/ROADMAP_STATUS.md)에 연결된 당시 감사 결과를 따른다.
9월9일 R 카드의 `proposed_not_implemented`는 당시 연구 상태이며 현재 실행 상태의 원본이 아니다.
기존 R 번호·기능/검증 선행, R09 정책 대기 분기, R23 변경 후 R22 재검증 관계를 삭제하지 않는다.

## 2. 이번에 확인한 현재 코드

| 관찰한 위치·심볼 | 관찰 사실 | 편입에 미치는 영향 |
| --- | --- | --- |
| [content_pack::load / publish_snapshot](../../../src/game/data/content_pack/mod.rs) | parse→resolve→semantic identity→검증·컴파일→ValidatedGameData. 스트레스 승인 조회는 이 경로에 없음 | 후보 세대와 본편 승인 구분. 후보를 시험하려고 로드할 때 PASS를 요구하면 순환 |
| [PackManifest / ContentSelection](../../../src/game/data/content_pack/schema.rs), [resolver](../../../src/game/data/content_pack/resolver.rs) | dependencies·Replace/Remove·writer 선택 및 참조 유효성 | MS05에서 실패 모드 제거 뒤 최종 구성을 다시 resolve. 모드=pack 1개라고 미리 단정하지 않음 |
| [TrialSession](../../../src/authoring/trial.rs), [TrialScenario](../../../src/authoring/scenario.rs) | 공개 명령/clock, accepted session·receipt·input/content identity. run_to_end는 시나리오 입력 종료 | MS03은 실제 발동/부하 달성 검사와 대규모 setup delta 필요. 현재 포트만으로 1,000 유닛 지원 완료라고 못 함 |
| [RuntimeFactLimits](../../../src/game/battle/runtime_control.rs), [기존 검토](../../plans/runtime_safety_observability/REVIEW.ko.md) | 기존 operation 예산·fault latch·owner extent | MS01 기본 이관 가능성. extent를 정확한 전투별 heap cap으로 바꾸어 부르지 않음 |
| [CoreApplication::execute_with_generation / from_domain](../../../src/application.rs) | 새 generation은 StartNewGame 경로로 전달. BattleRuntimeFault가 typed application error로 변환 | MS05 실제 적용 지점 후보, MS04 오류 분류/전파 재사용. typed 반환 자체가 전투의 영구 중단 상태는 아님 |
| [GameCore::execute_application](../../../src/game/world/application.rs) | controlled clone draft→operation→검증/poll→commit. 오류 시 draft를 정상 commit하지 않음 | 기존 rollback 경계 보존. 오류 표시를 실패 draft 안에만 기록하면 함께 사라질 수 있어 MS04에서 별도 전달/유지 경계 검토 |
| [advance_active_battle_staged / transition_finished_live_battle_to_result](../../../src/game/world/combat.rs) | step/finalize 성공 경로와 정상 결과 전환 존재 | MS04가 오류를 정상 승패·보상 경로로 보내지 않도록 소비자까지 검토해야 함. 현재 코드의 새 정책 충족을 주장하지 않음 |
| [validate_run_checkpoint_payload_with_seed](../../../src/game/world/node_flow.rs) | checkpoint content identity가 retained generation과 다르면 거부 | MS05 세이브 보호의 기반은 이미 있음. 필요한 모드 안내/호환성의 상세 메타데이터는 추가 검토 |
| [server tick_live_battle](../../../../game_server/src/game/player_game_actor/handlers.rs) | tick 오류 시 client error를 보내고 timer를 멈추는 경로 있음 | 재사용 후보. 재접속·수동 명령으로 오류 전투가 다시 진행되는지, 결과/저장이 차단되는지는 아직 미검증 |
| [서버 의존성](../../../../game_server/Cargo.toml) | game_core가 ../core_enhanced를 사용 | host 연결은 실제 검사할 범위. 서버 build/런타임 동작을 이번 열람만으로 PASS라고 하지 않음 |

세이브 full identity 불일치 거부를 완화하지 않는다. '모드가 빠져도 호환되는 세이브'의 판정은 실제 identity·지원 계약을 충족해야 한다.
서버 오류 메시지 변환과 로컬 embedding의 모든 호출 지점을 전수 조사한 것은 아니다. 호스트 adapter 전체 정상 작동이나
새 전투 오류 정책의 구현 완료는 확인하지 않았다. 룰북의 [목표/구현 구분](../../game_rulebook_next.md)도 유지한다.

## 3. 추가 관계의 종류와 근거

| 생산자 → 소비자 | 관계 | 필요한 출력 / 순서를 무시할 때의 문제 |
| --- | --- | --- |
| R02·R03 → MS03 | 기능 필수 | 검증된 후보 세대·공개 실행/receipt. 우회 fixture 성공을 본편 시험으로 오인할 수 있음 |
| R04·R05의 현재 안전/수명 + R06 완료 → MS01/03 | 재사용 기반 / R06 뒤 배치는 권장 | 기존 한도와 expression profile, 실제 집계 시험 입력. MS01 파서 자체에 R06 전체 기능이 필수인 것은 아님 |
| MS01 → MS02 → MS03 | 권장, 일부 측정 계약 재사용 | 설정 identity와 지표 의미를 먼저 맞춤. Grafana 서비스 실행 여부는 headless 판정의 필수 조건이 아님 |
| MS01·MS03 → MS05 | 기능·검증 필수 | effective safety identity와 해당 지원 범위의 실제 시험·측정 기준. 데이터 hash만 같은 낡은 PASS 재사용 금지 |
| 각 R 기능 → 해당 기능 profile → MS05 지원 범위 | 조건부 기능·검증 필수 | 그 기능의 실제 executor와 발동/종료 증거. R17 전파를 아직 없는 MS03 기본 시험에 요구하지 않음 |
| R02·현재 transaction/result/save 경계 → MS04 | 기능 필수 | 세대/상태와 정상 commit 경계를 알아야 전투 오류·진단을 분리할 수 있음. MS04 전체에 미래 R20 완료를 요구하지 않음 |
| MS04 → MS05 본편 적용 | 통합 검증 필수 | 실험실 통과 이후의 예외 상황도 해당 전투에 한정. 승인 기능 공개 시 함께 유효해야 함 |
| MS05 → R19의 승인 구성 소비 | 새 범위의 통합 선행 | 기존 등록/보상·평가가 승인된 실제 구성에 결속. 기존 R19 전체가 MS05 구현 선행인 순환은 아님 |
| MS04·MS05 → R20 | 검증 선행 | 먼저 완성한 오류/저장 동작의 교차 검증. R20을 최초 오류 복원 구현으로 사용하지 않음 |
| R21·누적 profile → MS06-A → R22 | 검증 선행 | 대표 데이터 기반 시험/기준 정리 뒤 엔진·도구 동결. MS05 최초 기준이 R21을 기다려야 한다는 뜻은 아님 |
| R22 → R23 → R24 | 기존 검증 선행 유지 | 독립 작성 부하→최종 비용/회귀→독립 감사. 새 MS06-B가 별도 선행 순환을 만들지 않음 |
| R23 변경 → 영향받는 R22·캐시 | 조건부 재검증 | 변경된 엔진/정책에 낡은 인수/PASS 재사용 금지. 최초 실행 순서의 역방향 edge로 취급하지 않음 |

MS04와 MS05 내부에서는 producer/consumer 동시 전환이 필요하다. 오류 신호만 있고 진행/결과 차단이 없거나,
승인 캐시만 있고 실제 generation 설치가 이를 우회한다면 완료가 아니다. 정확한 최소 atomic group은 상세 설계에서 정한다.

## 4. 정책 대기와 적용 한계

- R09가 기존 정책 사유로 유예되면 그 기능 profile도 미지원으로 남긴다. MS03/MS05의 제한된 지원 범위는 독립 유효하게 만들 수 있다.
  R20–R24 전체 완료 차단은 기존 로드맵대로 유지한다. 미지원 기능을 사용하는 모드는 검증 PASS를 받지 않는다.
- R19 전 본편 모드 제공이 필요하면 MS04/MS05를 앞당기고 당시 지원 기능만 승인한다. 미래 R 완료 전체를 요구하지 않는다.
- 수치 기준은 각 지원 범위의 대표 측정 후 정한다. 기존 hard safety guard는 유지하고, 수치·시험 미완료를 PASS로 바꾸지 않는다.
- 이번에는 구현 시점/출시 시점이나 모드 UI 정책을 새로 확정하지 않았다. 따라서 새 작업의 정확한 삽입 위치는 잠정이다.
- 이전 S1–S4는 개별 설계 재사용 자료다. 별도 로드맵에서 MS03→MS04→MS05 등을 구분하면서 단순 'S3 측정'보다 범위를 명확히 했다.

## 5. 검증 범위

[validate.py](validate.py)는 기존 R 연구 JSON의 forward 관계를 보존한 권장 순서, 새 관계와 링크,
원본 bytes 및 문서 patch 양방향 적용을 확인한다. 조건부 재검증 edge는 별도 보고한다.
기본 순서 검사는 R09 유예 분기의 실행 시뮬레이션이 아니며 그 분기를 삭제하지 않는다.
기능별 의미 판단은 위 코드/문서 검토에 근거한 제안이다. 제품 시험·성능 수치 확정·선행 재감사·formal Goal 활성화는 없다.
