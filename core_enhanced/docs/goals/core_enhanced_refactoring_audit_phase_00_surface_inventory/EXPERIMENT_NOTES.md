# Core Enhanced Refactoring Audit Phase 00 — Experiment Notes

Status: complete

## 초기 구조 판단

- 이 Phase는 후보 판정이나 리팩토링 구현이 아니라 Master의 coverage seam을 만드는 inventory다.
- 디렉터리 또는 파일 크기를 boundary로 먼저 확정하지 않고 실제 authority, producer, consumer,
  live-data와 test 수명주기를 읽은 뒤 경계를 제안한다.
- 후보 신호는 후속 Phase 배정 근거로만 사용할 수 있으며 Phase 0에서 Candidate ID, score 또는 최종
  disposition을 만들지 않는다.

## Parent와 dependency

- Parent Master: `core_enhanced_refactoring_audit_master`
- Direct predecessor: 없음.
- Master initialization은 완료됐고 registry의 Phase 0 dependency는 충족됐다.
- 2026-08-07 사용자의 Master PLAN 수행 요청을 Master가 제안한 정확한 Phase 0 이름의 activation
  승인으로 해석했다. 승인 범위는 investigation-only와 production no-mutation에서 확장하지 않았다.

## Initial repository와 dirty state

- Repository root: `/mnt/f/work/simulator`
- Primary scope: `/mnt/f/work/simulator/core_enhanced`
- Start commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- Baseline: 2026-08-07 current dirty working tree
- Phase path는 초기화 전에 없었고 이름 충돌도 없었다.

초기 status에는 다음 사용자 소유 변경이 존재한다.

- `core_enhanced/docs`: README/canonical/workflow/review guide와 기존 Goal/decision/tmp 산출물
- `core_enhanced/src`: battle/world/data/employee/behavior 등 다수 수정과 untracked tactical module
- `core_enhanced/tests`: RON loading과 skill runtime contract 수정
- `game_resources`: Core가 소비할 수 있는 production live RON 다수 수정
- `game_server`: protocol, fixture generator와 actor handler/message/state 수정

Phase 0은 위 변경의 작성자를 추정하지 않고, current tree를 actual runtime inventory baseline으로 읽는다.
Goal artifact 외에는 수정하지 않는다.

## Source-of-truth와 evidence boundary

- 현재 runtime 사실은 `core_enhanced` code와 authoritative mutation이 우선한다.
- Live data는 runtime loader와 lookup 연결이 확인된 path만 production supporting evidence로 분류한다.
- Server는 실제 Core producer/DTO/command seam 확인에 필요한 범위만 supporting evidence로 읽는다.
- Unity는 명시적 비범위이며 접근 가능한 것처럼 취급하지 않는다.
- Canonical은 `docs/README.md`가 주제별로 지정한 durable owner만 현재 계약 근거로 사용한다.
- Goal/archive/tmp/debug output은 current source of truth가 아니다.

## 적용하지 않은 대안

### `src`, `tests`, `docs`를 각각 후속 Phase로 고정

적용하지 않는다. 하나의 authority가 code/live data/test/canonical/server seam을 가로지를 수 있어
동일 계약 중복 감사와 consumer 누락 위험이 있다.

### Phase 0에서 후보를 확정하고 점수화

적용하지 않는다. 전체 runtime 경로와 contract safety net을 깊게 확인하기 전 점수는 검색 hit나
정적 모양에 과도하게 의존한다. Candidate ID와 verdict는 승인된 boundary Phase가 소유한다.

### Baseline test 실행

적용하지 않는다. 이 Phase는 behavior correctness나 회귀 여부를 주장하지 않는다. Test target과
safety-net surface만 inventory하고 필요한 runner는 후속 Phase가 후보 계약에 맞게 정한다.

## 외부 시스템과 소비자

- `game_resources`: Core runtime이 실제 읽는 production data family 확인에 필요한 범위만 read-only.
- `game_server`: Core DTO/command/result direct seam 확인에 필요한 범위만 read-only.
- Unity: 사용자 승인 out-of-scope, unverified consumer로 완료 조건에 포함하지 않음.

## 현재 정책 질문

Gameplay/runtime/data/wire 정책 질문은 없다. Phase 0 종료 뒤 후속 boundary Phase 수, 이름, scope와
순서에는 사용자 승인이 필요하다.

## 남은 위험

- 대규모 dirty tree 때문에 Git history 기반 change frequency는 현재 누적 작업 빈도를 대표하지 않을
  수 있어 후속 후보에서 `unknown`이 필요할 수 있다.
- Conditional compilation, generated include, embedded asset path 또는 macro registration은 단순 file
  tree inventory로 누락될 수 있어 entrypoint/export/loader 연결을 별도 확인해야 한다.
- Server/Unity 비범위 때문에 일부 consumer seam은 confirmed 대신 unverified로 남을 수 있다.
- Candidate evidence를 수집하지 않는 Phase 특성상 boundary를 지나치게 크게 잡지 않도록 authority와
  lifecycle 독립성을 각각 기록해야 한다.

## 다음 안전한 작업

1. current path/module/test/doc/manifest inventory를 재현 가능한 count와 함께 만든다.
2. crate entrypoint부터 state owner, projection, loader/DTO seam을 read-only로 연결한다.
3. 모든 surface에 boundary/out-of-scope/unverified disposition을 부여한다.
4. source-of-truth 경계별 후속 Phase 분할·순서를 제안한다.
5. 별도 completion audit에서 coverage, evidence 품질과 no-production-mutation을 재확인한다.

위 항목은 완료됐다. 현재 다음 안전한 작업은 `BOUNDARY_PROPOSAL.md`의 Phase 1~10/Final 이름과 범위에
대한 사용자 승인 또는 Master 범위 안의 명명 위임이다.

## Surface inventory 판단

- Current crate의 actual public runtime은 library `game_core`이며 server가 path dependency로 사용한다.
  `src/main.rs`는 빈 autobin이고 별도의 Core fixture autobin 5개가 있다.
- `GameCore::execute_with_transport_context()`가 player command의 단일 Core entry이고 private
  `GameCoreState`를 domain handler로 mutation한다.
- `GameCoreState`는 run/non-battle state와 `ActiveBattleSession`을 함께 host하지만,
  `ActiveBattleSession` 안의 `BattleCore`/`BattleExecutionState`가 simulation runtime authority다.
- `behavior.rs`는 command, result, snapshot과 battle transport DTO를 집중 소유한다. 이 집중은 후보
  자체가 아니라 Phase 10에서 producer/consumer와 함께 판단할 boundary evidence다.
- `GameDataBase::load_live_embedded()`는 production loader 하나이며 현재 `game_resources/data/**` 23개
  전부가 exact include set에 있다.
- Source tree는 battle 63, world 33, data 19가 크지만 크기 때문에 Phase를 나누지 않았다. Battle은
  lifecycle/spatial/effect/targeting, world는 run/economy/node/combat/projection authority로 분리했다.

## Canonical/source count drift

`data_loading_contract.md`는 official loader가 manifest 외 21개를 읽는다고 쓰지만 current
`GameplayRonSourceSet`, `LIVE_GAMEPLAY_RON_SOURCE_COUNT`, focused test와 실제 production file set은
22개다. Runtime behavior보다 canonical 숫자가 stale한 것으로 보이지만, Phase 1이 full loader contract를
읽고 work type과 candidate 여부를 판정하기 전에는 correctness/cleanup candidate로 확정하지 않는다.

## Unverified inventory signals

- `config/game_balance.toml`: workspace path/symbol consumer를 찾지 못했으나 external tool 여부 미확인.
- `tests/test_actor/mod.rs`: current integration root에서 module edge를 찾지 못했으나 build target 전체
  reachability는 Phase 10이 다시 확인해야 함.
- Core fixture autobin 5개: history를 제외한 workspace direct consumer를 찾지 못했으나 외부 fixture
  generation workflow 여부 미확인.
- `src/main.rs`/public logging setup: current server consumer를 찾지 못했으나 public library 외부 소비자
  존재는 unknown.
- VS Code `card_game` launch config: current package `game_core`와 다르지만 IDE tooling은 Master runtime
  candidate 범위 밖.
- `stacked_code_decomposition_audit`: historical PLAN-only directory이며 current dependency가 아님.

검색 부재만으로 위 항목을 obsolete/rejected/candidate로 판정하지 않는다.

## Proposed boundary 결정

후속 Phase는 다음 순서를 권장한다.

1. live data loading/validation
2. run/map/persistence
3. roster/inventory/economy
4. interactive node/VN
5. combat preview/setup
6. battle lifecycle/result
7. battle spatial/tactical
8. battle effects/stats
9. battle targeting/delivery
10. command/projection/server seam
11. final catalog consolidation

각 경계와 source coverage는 `BOUNDARY_PROPOSAL.md`가 소유한다. 이름/수/scope는 사용자 승인 전
proposal이며 child directory를 만들지 않는다.

## Validation 판단

- Phase 0은 behavior correctness를 주장하지 않으므로 test runner를 실행하지 않았다.
- 910개 test attribute는 safety-net 위치 inventory일 뿐 pass/coverage evidence가 아니다.
- 후속 후보 Phase는 해당 contract를 직접 assert하는 named test를 확인하고 필요하면 실행해야 한다.
- Completion audit은 file count reconciliation, boundary coverage, supporting consumer classification과
  initial/final production status 비교를 사용한다.
