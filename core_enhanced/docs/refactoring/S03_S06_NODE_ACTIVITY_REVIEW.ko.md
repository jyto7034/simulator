# S03/S06 상점·이벤트 전이 경계 조사와 유지 판단

작성·갱신일: 2026-10-17

현재 단계: 대표 흐름 조사·후보 판정·선택 테스트 25개 검증 완료. 후속 [S05 조사](S05_OWNERSHIP_ECONOMY_REVIEW.ko.md)도 마쳤다.
최신 진행 위치는 [출발점](README.md)을 따른다.

## 1 범위와 판단 기준

선택·입장 → 상점 거래 → 상점 종료 → 노드 완료를 먼저 추적하고 이벤트 판정·마지막 장면과 비교했다.
이번 단계는 조사·후보 판정·후속 계획까지다. 제품 코드·게임 규칙·데이터·테스트를 수정하지 않았다.
S03 원정 전체와 S06의 본부·정비·보상·VN·유물 활동 전체를 감사한 것은 아니다.

게임 의미는 [상점](../game_rules/run-and-map.md#상점), [Event](../game_rules/run-and-map.md#event),
[저장과 복구](../game_rules/results-and-save.md#저장과-복구), 기술 의미는
[runtime 계약](../core_runtime_contract.md)의 Shop Session / Event / Durable 절을 기준으로 했다.
테스트는 이 의미와 구현을 대조하는 증거이며 테스트용 설정을 제품 규칙으로 사용하지 않았다.
[S01](S01_APPLICATION_BOUNDARY_REVIEW.ko.md)의 outer draft·commit,
[S04](S04_PERSISTENCE_BOUNDARY_REVIEW.ko.md)의 미완료 Continue·SavePoint 구분,
[S07](S07_COMBAT_BOUNDARY_REVIEW.ko.md)의 전투 결과·노드 완료 경계는 재사용했다.
새로 확인한 것은 이 경계에 들어오는 상점·이벤트의 상태와 호출 순서다.

## 2 실제 실행과 소유권

| 단계 | 입력·원본과 준비 | 변경·응답·다음 소유자 |
| --- | --- | --- |
| 입장 | `SelectMapNode` 이후 `ConfirmEnterNode`; map 선택·phase·진입 가능성 검사. 미완료 복원용 진입 전 상태와 원정 입장 비용을 처리 | `handle_confirm_enter_node_inner`가 map 진입·`node_session`·VN trigger를 조립하고 해당 활동에 위임. 상점은 `InShop`, 이벤트는 `InNode`와 active event session |
| 상점 생성 | 기존 node-keyed session이 있으면 재사용. 없으면 metadata/pool에서 구성 | `build_shop_session`이 identity 정렬 뒤 node/Shop/page seed로 중복 없는 페이지 생성. `commit_shop_session`이 run의 세션과 active 사본을 같이 갱신 |
| 구매 | 현재 `offer_id`·총액·자원 자격 확인. `EconomyGrantState`에서 차감·지급 준비 | 지급 성공 후 offer 제거·구매 이력·자원 및 두 session 표현을 게시. 같은 offer 재요청은 더 이상 구매 불가 |
| 판매 | 소유 instance/stack·수량·귀속/장착·종류·tag·checked quote·통화 overflow 검사 | 허용된 수량만 제거하고 계산한 통화·inventory diff 반환. 유물 매각은 거절. 판매는 페이지 구매 이력 변경과 별개 |
| 새로고침 | policy·잔여 횟수·재화·full-page 후보 수를 공통 evaluator로 확인 | 새 페이지 생성에 성공해야 통화·page index·사용량·session을 함께 바꿈. 구매한 반복 불가 상품은 후보에서 제외 |
| 상점 종료 | `ExitShop` capability는 active shop과 node session 요구 | `execute_shop_action(Exit)` → `handle_complete_node` → 공통 완료 계획·commit |
| 이벤트 판정 | node/event/scene/choice/직원 identity, 생존·장비 조건 재검사. 선택·직원과 독립적인 node seed d20 | grant 또는 battle 시작을 준비한 후 판정 receipt·다음 scene·두 session 표현을 갱신. 판정 실패도 정상 다음 장면 |
| 이벤트 마지막 장면 | `AdvanceEventScene`은 현재 scene identity와 `End`를 확인 | 공통 `plan_complete_current_node` 결과를 준비하고 event session과 완료를 commit. 일반 `CompleteNode`로 선택 장면을 건너뛸 수 없음 |
| 노드 완료 | map/progression과 recovery를 계획하고 `StagedNodeCompletion`으로 분기 | node 완료, 다음 층, run 완료가 각각 필요한 정리를 수행. 정상 node 완료는 active content·node session을 비우고 `ViewingMap`; run 완료는 terminal 정산과 `RunComplete` |

근거: [명령 dispatch](../../src/game/world.rs), [owner 분류](../../src/game/world/application.rs),
[입장·완료](../../src/game/world/node_flow.rs), [상점 진입](../../src/game/world/map_content.rs),
[상점 거래](../../src/game/world/shop.rs), [이벤트](../../src/game/world/event_node.rs),
[공통 자격 판정](../../src/game/world/action_capability.rs), [grant 상태](../../src/game/world/grant_state.rs).
저장소가 설정된 서버의 일반 변경 명령은 [player actor](../../../game_server/src/game/player_game_actor/handlers.rs)의
`prepare_command_with_generation` → `publish_prepared_application_bound`를 사용한다.
개별 거래 규칙을 host에서 재계산하지 않으며 publication 의미는 S01/S04 조사 결과를 따른다.

### 실패·중복·복원의 구분

- **공개 명령 실패:** outer application draft가 commit되지 않으면 호출 전 권위 상태를 유지한다.
  `node_flow::with_state_transaction`의 내부 rollback은 profile을 따로 유지하는 런 단위 처리이므로
  이를 공개 명령 전체의 실패 보존 계약과 동일시하지 않는다. 기존 S01 D1 설명을 유지한다.
- **지급 중 오류:** 상점·이벤트는 공통 `EconomyGrantState`에서 자원 변경을 준비한다.
  현재 이 객체는 inventory·skill fragments·relics·crafting resources·roster·UUID·통화의 일곱 root를 소유한다.
  과거 테스트 이름의 `five_roots`를 현재 전체 자원 수나 완전한 검증 범위로 해석하지 않는다.
- **동일 이벤트 판정 재요청:** active event의 확정 source scene/choice/employee와 동일할 때
  preflight가 저장된 결과 반환을 허용한다. 다시 굴리거나 지급하지 않는다. 다른 identity는 거절한다.
  이 보장은 active event 문맥에 대한 것이며 노드 종료 후 모든 과거 명령의 재실행을 보장하지 않는다.
- **판정 장면 수:** [event validator](../../src/game/data/event_data.rs)는 도달 가능한 직원 판정 장면을
  최대 하나로 제한한다. 단일 `committed_check`가 다중 직원 판정을 덮어쓰는 정상 경로를 확인하지 못했다.
  유물 선택 이력은 별도 순서·수명이며 직원 판정 receipt와 통합하지 않는다.
- **완료 정리:** [RunState](../../src/game/world/state.rs)의 `retire_completed_node_setup`은 완료된 전투 setup과
  일반 event session을 제거하되 유물 선택 이력이 있는 event receipt는 남긴다. 상점 세션은 층 내에서 유지한다.
  `for_next_floor_from`/`for_run_complete_from`은 새 층·terminal용 run을 구성한다. 모든 세션을 같은 시점에
  지우는 규칙을 도입하면 필요한 이력의 수명이 바뀐다.
- **Continue와 checkpoint payload:** 미완료 상점·이벤트의 Continue는 입장 전 상태이며 완료 후에는
  안정 상태를 보존한다. `RunCheckpointPayload`가 담은 상점 page/history를 정확히 복원한다는 계약은
  미완료 거래 도중의 Continue가 그 페이지에서 재개된다는 뜻이 아니다. S04의 구분으로 해석 가능하다.

## 3 구조 평가

| 기준 | 이번 흐름의 근거·판정 |
| --- | --- |
| 책임·도메인 경계 | map 진입/완료, 활동 거래/선택, 공통 지급, 외부 게시가 기존 경계를 사용한다. 대표 흐름 유지 |
| 상태 소유권 | run 세션과 active 사본은 같은 commit helper로 갱신한다. 독립된 두 권위 owner로 볼 근거 없음 |
| 전이·수명 | 입장부터 완료까지 추적. 일반 event·유물 이력·상점의 종료 수명 차이는 의도와 연결됨 |
| 실패·commit | outer draft와 내부 grant 준비가 별도 범위를 보호. 국소 rollback 제거를 정당화할 증거 없음 |
| 규칙 중복 | 상점 자격·quote와 이벤트 payload 검사 재사용. 활동별 정책 차이를 억지로 공통화할 이유 없음 |
| 의존·공개 범위 | Core 활동 handler는 파일 I/O나 전송 타입에 의존하지 않음. 공개 `CompleteNode`는 활동 전용 완료 절차를 우회하지 못함 |
| 변경 용이성 | 상품 종류 지급은 공통 grant로 연결, event 종료는 공통 node completion 사용. 새 registry/trait 도입 필요 미확인 |
| 데이터·외부 계약 | typed offer/event DTO와 receipt를 반환. event validator가 한 직원 판정 제약 보장. UI 전체는 조사하지 않음 |
| 결정론·복원 | 상점 후보 정렬·page seed, event node seed·확정 receipt, 미완료/완료 Continue 구분 확인 |
| 오류·관측 | 활동 거절과 정상 판정 실패를 구분. query가 새 상품/사본을 생성하지 않는 기존 검증 선택. 별도 telemetry 변경 없음 |
| 검증 가능성 | 직접 handler·내부 setup 테스트와 공개 application 테스트 모두 존재. fixture·admin 진입 한계를 아래에 구분 |
| 성능·자원 | outer draft, 국소 transaction, grant capture에 복사 비용은 있으나 이번 측정 없음. 세션 보존도 수명상 이유가 있어 비용 결함이나 최적화 효과로 단정하지 않음 |

## 4 후보 판정과 후속 계획

**선택한 대표 흐름은 유지한다.** 확정한 실행 오류나 이번 범위에서 착수할 구조 변경 후보는 없다.
새 코드 변경을 만들기 위해 정상적인 상태 사본이나 활동별 차이를 결함으로 분류하지 않았다.

| 검토한 후보 | 판정과 이유 | 재검토 조건 |
| --- | --- | --- |
| active/run session 단일화 | 보류. 현재 갱신 helper와 검증이 있고 수명·저장 표현이 다름 | 정상 경로의 불일치 재현 또는 구체적인 변경에서 누락이 확인될 때 |
| 상점·이벤트 완료 추상화 추가 | 유지. 이미 공통 node completion 사용 | 새로운 활동이 실제로 같은 완료 정책을 독립 재구현할 때 |
| 내부 transaction·grant 복사 제거 | 보류. 실패 범위가 다르고 성능 증거 없음 | 대표/지원 최대 규모 측정에서 비용 문제가 확인될 때, 실패 보존 검증과 함께 |
| 상점 복원 설명 교정 | 현행 유지. payload의 정확한 복원과 미완료 Continue의 입장 전 복원은 별개 | 실제 소비자가 두 경로를 혼동하는 계약/구현이 확인될 때 |

따라서 별도의 구현·완료 감사 작업을 만들지 않고 이번 조사와 검증 기록을 확정한 뒤 S05 대표 흐름으로 이동한다.
향후 후보가 생기면 재현·보존 의미·작은 변경 범위·필수 검증을 먼저 정하고 구현 후 별도 완료 검토를 한다.

## 5 검증과 한계

선택한 코드·계약·build·테스트 입력 28개는 [기준선](evidence/s03_s06_001/baseline.json)에 기록했다.
전체 transitive dependency manifest나 저장소 전체 freeze라는 뜻은 아니다.
실행 명령·환경은 [commands.json](evidence/s03_s06_001/commands.json), 최종 입력·문서 확인은
[verification.json](evidence/s03_s06_001/verification.json)에 기록한다.

| 실제 실행 | 결과 | 시간·근거 |
| --- | --- | --- |
| Core lib: 상점 내부 7, 진입/종료·세션 3, U16 거래·복원 4, event 전이/오류/DTO 6, event validator 1 | 21 통과, 실패·무시 0 | build 39.66초, test 29.42초. [core001.log](evidence/s03_s06_001/core001.log) |
| 공개 application: event 3, 미완료 event Continue 1 | 4 통과, 실패·무시 0 | build 1분 7초, test 13.40초. [public001.log](evidence/s03_s06_001/public001.log) |

합계 **25개 통과, 실패·무시 0**이다. U16의 24개 seed 반복과 완료/미완료 두 입력은 각각 해당 테스트 안의
사례이며 별도 테스트 수로 부풀리지 않았다. 기존 Core 경고 33개(lib test)·36개(lib)는 남아 있다.

- 상점 내부 테스트는 deterministic identity/quote, 정확한 구매·판매 수량, 실패 시 보존, full-page 갱신을 검증한다.
  누락 장비를 넣은 지급 실패는 내부 fault fixture다. 유효 데이터의 자연 플레이에서 해당 오류를 재현했다는 뜻은 아니다.
- node/event 테스트 일부는 node graph·자원을 내부 helper로 구성하거나 private checkpoint/handler를 직접 호출한다.
  일반 경로 동등성과 내부 실패 범위의 근거로 사용하며 순수 공개 API 종단 실행으로 부르지 않는다.
- U16 상점 Continue 테스트는 공식 데이터 기반 custom shop과 입장 전 자원 지급, 내부 node 설정을 사용한다.
  손대지 않은 export를 새 Core 인스턴스에 넣어 미완료/완료의 자원 차이를 비교한다. OS 재시작 검증은 아니다.
- 공개 U08 event 테스트는 외부 pack loader·`CoreApplication` command/read-model을 통과한다.
  `event.rs`의 세 사례는 admin으로 event에 진입하고 이후 선택·전이는 일반 명령을 사용한다.
  `lifecycle.rs`의 미완료 복원 사례는 유효한 별도 작성 map을 load하여 실제 선택·입장 명령을 사용하지만,
  일부 사전 자원 지급은 admin을 사용한다. 두 종류를 자연 발생 공식 map 전체 플레이로 합쳐 주장하지 않는다.
- 기존 S01/S04/S07 통과 수는 이번 실행 수에 더하지 않는다. 전체 suite, release, 서버 crash 재실행,
  Unity, 모든 비전투 활동, 장기 세션/최대 규모 성능은 이번 검증에 포함하지 않는다.

테스트 종료 후 코드 근거·fixture 범위·유지 판단·다음 작업을 다시 대조했다. 이는 같은 작성자의
조사 기록 검토이며 외부 독립 감사나 미실행 시스템 전체의 완료 선언이 아니다.

## 6 다음 재개 위치

다음은 **S05 소유·경제·성장 대표 흐름 상세 조사와 개선 계획**이다.
이번에 확인한 상점 취득을 출발점으로, 장비/유물 사본의 지급·소유 → 판매 또는 분쇄 → 원정 terminal 시
영구 profile/창고로 넘기는 경계를 선정한다. 처음부터 모든 경제·성장 시스템을 전수 조사하지 않는다.

시작 위치는 `world/grant_state.rs::EconomyGrantState`, `game/reward.rs::GrantExecutor`,
`world/expedition_lifecycle.rs::settle_expedition_terminal`이다. 실제 지급·소비·반환 소유자를 따라가며
S04의 warehouse debit 복원과 S07의 보상 수량 근거를 재사용한다. 필요한 게임 규칙만 추가로 확인하고,
정확한 사본 identity·중복 지급/회수·실패 보존·Continue 의미를 기준으로 유지/개선 후보와 검증 범위를 정한다.
종료점은 선택 흐름의 조사·후보 판정·개선 계획·재개 위치 갱신이다.
