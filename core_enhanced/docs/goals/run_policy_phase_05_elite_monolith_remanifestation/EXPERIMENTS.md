# Elite Monolith Remanifestation Experiments

Status: complete

Implementation complete: true

Created: 2026-07-27

## 2026-07-27 — Readiness audit

### Predecessors and available seams

- P01~P04 completion review: 모두 Complete / High.
- P01은 signed active-run current/backup/checksum과 checkpoint/non-rewind metadata seam을 제공한다.
- P02는 authored placeholder/discovered/defeated projection과 actual-result idempotent producer를 제공한다.
- P03 live facility에는 Endless `shop` tag slot과 generic capstone transition authority가 있다.
- P04 live run policy에는 `MonolithRemanifestation` role difficulty가 있다.
- Unity MCP 재연결 확인: `ark@b49d32f6f2f70bbe`, Unity 6000.3.4f1, editor idle/ready,
  compile pending 없음, Console error 0.

### Live content evidence

- `data/pve/encounters.ron`에는 `suppress_white_night`, `suppress_silent_orchestra`,
  `suppress_apocalypse_bird`가 FinalBoss로 있다.
- 같은 파일의 `suppress_queen_of_hatred`, `suppress_mountain`도 현재 FinalBoss라 target NormalBoss
  reclassification이 필요하다.
- `data/events/shops/base.ron`의 Unknown Shop은 offer 4, refresh 없음, equipment 중심 high-value preset이며
  typed monolith merchandise는 없다.
- `data/visual_novels/base.ron`과 triggers에는 monolith exchange/hidden capstone VN이 없다.
- Unity `VisualNovelLocalCatalog.cs`에도 두 VN의 script/cue가 없다.
- `data/map/facility_templates.ron`에는 generic BossOmen capstone만 있고 hidden-boss 전용 single-terminal
  template/pool은 없다.
- workspace-wide content search에 전용 hidden-boss abnormality/skill/encounter가 0건이다. `흉탄의 사수`
  문자열은 문서에만 있다.
- `docs/skills/abnormality_skill_design_notes.ko.md`는 실제 조우를 `추후 미정`으로 둔다.

### Readiness verdict

- Price/offer/refresh/stage multiplier는 승인된 decision 12절에 따라 보수적 초기 live balance로 저작할
  수 있어 독립 blocker가 아니다.
- Hidden boss는 identity만 확정됐고 실제 gameplay content가 결정/제공되지 않았다. Existing
  Der Freischutz reuse, empty placeholder와 temporary encounter는 master readiness gate가 금지한다.
- Result: `policy_decision_required`. Production files were not modified.

### Commands/evidence note

- Read all 404 lines of the approved monolith policy in ordered chunks.
- Searched Core/resources/server/Unity excluding generated Unity folders for hidden-boss, monolith, exchange and
  VN identifiers.
- Inspected live PVE final-boss entries, abnormality metadata, Shop definitions, facility templates/assignment,
  visual novels/triggers, fragments and Unity local VN catalog.
- Queried Unity MCP instances, pinned the only instance, read editor/project/custom-tool state and Console errors.
- This readiness audit ran no test suite, so it reports no new test count.

### Dirty overlap gate

- Core/resources/server/Unity are heavily dirty from the active P00~P04 campaign and pre-existing user work.
- No P05 production implementation was found and no production mutation was made during this audit.
- The new child documents and master status only record the blocking evidence; unrelated changes remain untouched.

## 2026-07-27 — Hidden-boss concept feasibility audit

- This was policy-independent read-only investigation while the child remained `policy_decision_required`.
- Current skill runtime supports directional-collision projectiles, piercing, bounded hits/pierces/range/lifetime,
  `Allies`/`Enemies`/`Any` collision filters, target/caster/impact-anchored tile areas and effect telegraph DTOs.
- Current skill effects and projectile runtime do not express a damage curve that accumulates power after crossing
  caster-side allied units, nor a side-sensitive collision history that a later impact explosion can consume.
- Therefore the concept seed can be implemented faithfully, but it requires a typed projectile collision-history
  and power-curve/runtime extension plus Core/server/Unity event/presentation tests. A stat-only RON clone of
  Der Freischutz would not implement the approved distinct identity.
- No production source or live data was modified and no tests were run.

## 2026-07-27 — User content-policy deferral

- 사용자는 보스 정책을 포함한 콘텐츠가 모두 확정된 뒤에만 구현한다고 결정했다.
- 현재 P05에서는 `흉탄의 사수` 세부 구현을 skip하며 작업자가 누락된 콘텐츠를 임의로 설계·저작하지
  않는다.
- P05-A/B만 선행 구현하는 partial migration도 시작하지 않는다.
- Child는 `policy_decision_required`, implementation false를 유지한다. Production 변경과 test 실행은
  없었다.

## 2026-07-27 — 사용자 범위 선택 `2번`과 P05 재활성화

- 사용자는 모노리스 시스템을 모두 구현하고 실제 보스 구현은 뒤로 미루는 범위를 명시적으로 선택했다.
- Master를 P05 모노리스 시스템, P06 hidden-boss content(`deferred_by_user`), P07 final review로 분리했다.
- P05의 현재 terminal은 잔재 네 개 교환과 변질된 모노리스 run-local key 획득이다.
- 변질된 모노리스의 소비처, FinalBoss 재분류/완료 interception, 관리자 VN, 전용 capstone과
  `흉탄의 사수` 전투는 P06 전까지 production에 추가하지 않는다.
- P05는 `active`, implementation false로 전환했다. 이 기록 시점까지 production runtime/data/Unity
  변경이나 test 실행은 없다.

## 2026-07-27 — P05 모노리스 수직 구현

### 구현 결과

- `MonolithRunState`가 자연 Elite 승리/완전 실패, 환상체별 구매·3 uses, stage 2/3, 변환 node,
  ordered residual, exchange 상태, 다음 Floor Shop 보장과 dormant corrupted key를 소유한다.
- 자연 Elite는 승리 또는 세 번째 패배 뒤 같은 run 후보에서 hard exclusion된다. 같은 Floor에도 같은
  identity를 중복 배정하지 않으며 후보가 모두 소진되면 Elite node를 authored Standard ordinary
  Combat으로 강등한다.
- Standard의 `unknown_exploration_shop`과 Endless Floor 4+의 `wandering_monolith_shop`을 분리했다.
  방랑 상인의 첫 page는 eligible 수에 따라 0/1/2 monolith와 item을 합쳐 정확히 3개이며, 단 한 번의
  유료 refresh는 item-only 3개로 전체 교체된다. 고가 pool은 반복 가능 consumable만 사용한다.
- `ConvertMapNodeWithMonolith`는 NodeConfirm이 투영한 카드 중 하나를 최종 확정할 때만 실행된다.
  순수 Available Normal Combat의 node/slot/category/topology를 유지하면서 encounter와 P04 difficulty
  role을 교체하고 use를 한 번 차감하는 transaction이다.
- 강화 재현은 최대 2회 시도한다. 첫 패배는 같은 node 재시도, 두 번째 패배는 무보상 완료이며 retreat는
  진입 snapshot으로 되돌아가 attempt를 소비하지 않는다. 승리만 stage를 올리고 stage 3은 고유 fragment를
  정확히 한 번 지급한다.
- stage 3 승리 때 use가 1 남으면 residual로 바뀐다. 네 개가 되면 다음 Floor의 Shop-tag slot을 정확히
  하나 보장하고, 상인 진입 시 Shop보다 먼저 `monolith_residual_exchange` VN을 시작한다. accept는 oldest
  four를 원자 소비해 dormant corrupted key를 주며 decline은 같은 run에서 비가역적이다.
- 위 상태는 run/checkpoint/active-run/battle-entry rollback과 root snapshot에 포함된다. 기존 Endless
  final-boss completion, 관리자 VN, hidden capstone과 hidden-boss content에는 변경을 만들지 않았다.
- 이전 Floor에서 해결하지 않고 떠난 재현 node mapping은 다음 Floor 생성 시 제거하되 이미 소비한 use는
  환불하지 않는다. residual threshold의 다음-Floor Shop 보장은 run당 한 번만 발행되며 checkpoint load는
  모노리스 stage/mapping/residual/exchange/key reference와 상태 불변식을 함께 검증한다.
- Unity는 typed Shop offer와 NodeConfirm projection을 읽는다. 진행 중 identity는 다른 사용 가능 카드가
  있는 panel에서 room이 표시된 disabled card로 남고, 변환된 node preview에는 identity, stage, 위험도,
  최대 시도와 reward 배율이 명시된다. 카드는 local selection만 수행하고 별도
  확정 버튼이 `{ node_id, abnormality_id }` command를 보낸다. 교환 VN script/choice/hint도 local catalog에
  추가했다.

### 정책/데이터 수치

- Floor 4+, first page 3, paid refresh 1회, initial uses 3, remanifestation attempts 2.
- residual threshold 4, monolith price 250, equal offer weight 100.
- stage 2 stat/reward 100%, stage 3 stat/reward 125%; P04 `MonolithRemanifestation` role multiplier와
  합성해 stage 3 target power도 stage 2보다 높다.
- Cross-catalog validation은 방랑 상인/VN/Elite unique fragment reference와 exact page/refresh 계약을
  live publication 전에 검사한다.

### 검증 기록

| 실행 | 결과 |
| --- | --- |
| 사전 `cargo check -p game_core` | 성공 |
| 사전 `cargo check -p game_server` | 성공 |
| `cargo test -p game_core embedded_source_set_parses_all_versioned_gameplay_ron --lib -- --nocapture` | 1 passed |
| `cargo test -p game_core run_policy_data::tests --lib -- --nocapture` | 10 passed |
| `cargo test -p game_core monolith --lib -- --nocapture` | 최종 9 passed |
| 자연 Elite 소진 ordinary fallback focused test | 1 passed |
| Standard/Endless Shop 분리와 first-page focused test | 1 passed |
| checkpoint Shop+monolith exact restore focused test | 1 passed |
| `cargo test -p game_core live_ron --lib -- --nocapture` | 52 passed, 0 failed |
| `cargo test -p game_core --lib` | 786 passed, 0 failed |
| `cargo test -p game_server` | 52 passed, 0 failed; doc tests 0 |
| Unity 최종 monolith EditMode focused job `44ed768f0d3b49e1bcc7a9e27c99217d` | 4/4 passed |
| Unity broad EditMode job `60aa28fc93f742b1b9c0bb4cf3420aab` | 258/258 passed |
| Unity script validation | 신규/변경 대상 errors 0 |
| Unity compile/Console | editor ready, `error CS` 0 |

첫 `live_ron` 실행은 방랑 상인 pool에 추가했던 `resonance_primer`의 기존 tag와 새 tag가 충돌해 13건이
실패했다. 중복 entry를 제거하고 consumable 3종 pool로 단일화한 뒤 52/52와 전체 783/783을 재실행했다.
신규 Unity Shop fixture 첫 실행은 root `selected_event` wrapper 누락으로 1건 실패했고 fixture를 canonical
shape로 고쳤다. 최종 projection 보강 뒤 focused 4/4과 broad 258/258을 재실행했다.
Server snake_case command fixture는 `convert_map_node_with_monolith`의 node/abnormality identity를
명시적으로 deserialize해 Core enum으로 전달하는지 검증한다.

### 범위 감사

- 관련 Core/RON/Unity 파일에 `흉탄`, `freischutz`, `administrator`, hidden boss/capstone production
  identifier가 추가되지 않았음을 scoped search로 확인했다.
- P06의 actual boss actor/skill/encounter/VN/capstone/completion은 구현하지 않았다.
- 완료 검토: `COMPLETION_REVIEW.md` — **Complete / High**.
