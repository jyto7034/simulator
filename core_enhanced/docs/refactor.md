core_enhanced 설계 리뷰 (종합)

---

## Codex 검토 코멘트 (2026-07-03)

이 문서는 리팩터 후보를 찾기 위한 감사 로그로는 유용하지만, 현재 코드 기준의 canonical 판단 문서로 그대로 사용하면 안 된다. 2026-07-03에 실제 코드와 대조한 결과, 일부 지적은 여전히 타당하고 일부는 최근 구현으로 이미 해소되었거나 표현이 과장되어 있다. `cargo check -p game_core`는 현재 통과한다.

현재도 타당한 핵심 지적:

- `src/game/world/combat.rs`의 `handle_complete_combat_result()`는 보상/로스터/인벤토리 staged state를 먼저 커밋한 뒤 `commit_staged_node_completion()`이 실패할 수 있어, CombatResult 재호출 시 보상 중복 위험이 남아 있다.
- `src/game/world/snapshot.rs`는 상점 `hidden_items`, `hidden_item_uuids`를 Unity-facing snapshot에 노출한다. 내부 상태로는 필요해도 클라이언트 계약에는 숨김 상품을 내려보내지 않는 편이 맞다.
- `src/game/boss_omen.rs`의 `map_distance_to_terminal()`은 DFS `pop()` 기반이라 최단 거리 계산이 아니다. 오멘 배치 우선순위가 거리 의미를 가진다면 BFS가 더 적합하다.
- `src/game/world/event_node.rs`의 `ApplyBossOmenStepResult`는 현재 no-op이다. 정책상 오멘 스텝 결과가 실제 효과를 가져야 한다면 구현 누락이고, 의도적 no-op이라면 canonical 문서에 명시해야 한다.
- live RON 로더 조립이 `game_server`, `tests/common`, `world/tests`에 여러 벌 존재한다. 데이터 적재 범위 drift를 막으려면 장기적으로 단일 loader 진입점이 필요하다.
- `include_str!` 기반 데이터 적재는 runtime hot reload가 아니라 compile-time embedding이다. hot reload를 목표로 한다면 별도 설계가 필요하다.

현재 코드 기준으로 낡았거나 부분 수정된 지적:

- `tests/common/mod.rs`가 `omen_chain_id` 때문에 컴파일되지 않는다는 지적은 현재는 낡았다. 관련 fixture와 loader는 갱신되어 있고 `cargo check -p game_core`가 통과한다.
- Event choice 효과 적용이 원자적이지 않다는 지적은 현재 코드와 다르다. `apply_event_choice_effects()`는 inventory, skill fragments, roster, uuid manager, enkephalin을 staged clone에 적용한 뒤 마지막에 state로 반영한다. 단, `ApplyBossOmenStepResult` no-op 문제는 별개로 남아 있다.
- 기본 공격 이동/정지 판단이 `range_units`에 종속된다는 표현은 현재 코드 기준으로는 과하다. `movement/planner.rs`는 tile range 기반 target selection을 사용하며, `range_units`는 일부 데이터/테스트/helper에 남아 있지만 기본 공격 판정 source로 보기는 어렵다.
- `ActionValidator`가 쓰기만 되고 읽히지 않는다는 표현은 부정확하다. allowed actions snapshot/검증 경로에서 사용된다. 다만 `ActionScheduler`와 역할이 겹치는 구조 부채라는 평가는 검토할 만하다.

추천 처리:

1. 이 문서는 "리팩터 후보 감사 로그"로 취급한다.
2. 실제 goal을 만들 때는 각 지적을 다시 코드와 live RON으로 재검증한다.
3. 우선순위는 CombatResult 원자성, shop hidden item DTO 노출 제거, boss omen 거리 계산, `ApplyBossOmenStepResult` 의미 확정, loader 단일화 순서가 적절하다.

---

리뷰 방법: 6개 병렬 심층 분석(world/state/session·DTO, battle core, skill fragment·환상체, PVE 스케일링, 맵·보스 오멘, RON 로딩·테스트)으로 대상 파일을 전부 실제로 읽고 라인 단위 근거를 수집했습니다. 파일은 일절 수정하지 않았습니다.

---

1. 전체 설계 총평

전반적으로 "서버 권위 + 결정론 + 데이터 계약 fail-fast"라는 세 축이 의도적으로, 그리고 상당히 일관되게 지켜지는 코드베이스입니다. 커맨드 → 검증 → 상태 변이 → DTO의 단방향 파이프라인, 계층적 시드 파생(run → floor → node → wave → unit), plan/commit 2단계 트랜잭션, 로드 타임 cross-reference 검증은 이 규모의 프로젝트치고 이례적으로 규율이 잡혀 있습니다. grep 기준 legacy/TODO 주석 부채도 거의 없습니다 — "레거시가 새 정책 위에 억지로 남아있는" 문제는 우려보다 훨씬 적었습니다.

구조적 리스크는 세 갈래로 수렴합니다:

1. 경계 침식: DTO 계층이 존재하지만 예외가 많습니다. 내부 타입(BattleEventLogEntry, NodeSession, CombatPreview)이 직렬화 경계를 그대로 관통하고, format!("{:?}") Debug 문자열이 Unity wire format이 된 곳이 다수이며, 상점 hidden_items처럼 숨김 정보가 새는 곳도 있습니다.
2. source of truth 분열: "현재 노드" 4중 저장, 로더 조립 코드 3벌 복제, 공격 간격의 이중 기록, 노드 가용성의 이중 저장 등 — 각각은 수동 동기화 규약으로 버티고 있지만 기능 추가마다 감사 대상이 늘어나는 구조입니다.
3. 관측과 게임플레이의 커플링: RNG가 이벤트 로그 seq에서 파생되어 "로깅 개선 = 밸런스 변경"이 되는 문제가 결정론 설계의 가장 깊은 균열입니다.

그리고 당장 조치가 필요한 사실 하나: tests/common/mod.rs가 AbnormalityMetadata의 omen_chain_id 필드 추가 이후 갱신되지 않아 컴파일되지 않으며, live RON 회귀 테스트 전체(ron_loading.rs 포함)가 현재 죽어 있습니다.

---

2. 컴포넌트별 설계 평가

2.1 Game world / state / session flow — ★★★☆ (구조 좋음, 원자성 균열)

- 좋음: GameState가 데이터를 담은 enum variant로 명시적 상태 머신을 이루고, 모든 명령이 GameCore::execute(world.rs:65~214)의 단일 게이트를 통과. StagedNodeCompletion/StagedCombatResultState의 plan/commit 패턴.
- 문제: handle_complete_combat_result(world/combat.rs:14이며, 보상 커밋(combat.rs:1657) 이후
  commit_staged_node_completion이 ?로 실패할 수 있어 부분 니다. staged 패턴의 취지가 가장 중요한 경로에서 깨져있습니다.
- "현재 노드" 사실이 GameState::InNode / node_session / active_node_content / run.event_sessions의 최소 3~4곳에 저장되고, 클리어 지점이 8곳 이상이며 각 지점이 조금씩 다른 조합을 비웁니다.
- ActionValidator(world/state.rs:108)는 쓰기만 되고 읽히
- 코어 경로에 panic! 4곳(world/state.rs:434, map_content.rs:246 등)과, 상태 계층이 CARGO_MANIFEST_DIR 기반 파일 I/O(battle_records)를 직접 수행하는 계층 위반이 있습니다.

  2.2 Battle core (movement / targeting / attack delivery) — ★★★★ (강한 코어, 정책성 균열 2개)

- 좋음: 이벤트 로그가 유일한 관측 채널이고 result_stats/validator가 로그에서만 파생. calculate_damage는 순수 함수이고 DamageSourceSnapshot이 "발사 시점 공격자 / 착탄 시점 방어자" 규칙을 타입으로 강제. HashMap 순회 전 UUID 정렬의 canonical ordering 규율이 전방위적. mod.rs 4,770줄은 착시로, 실로직은 ~540줄(나머지는 테스트)입니다.
- 문제 1: 크리롤/투사체 ID 시드가 event_log_seq에서 파생(commands.rs:247, 777-784) — 로그 기록을 하나 추가/삭제하면 이후 모든 크리티컬 판정이 바뀝니다.
- 문제 2: 기본 이동 백엔드가 결정론 feature 없는 f32 Rapier(engine.rs:149-153)인데, 실제로는 유닛 충돌을 전부 ignore하고 정적 장애물 슬라이딩만 수행(rapier_backend.rs:97-140) — Direct 백엔드와 기능이 같으면서 결정론 리스크와 유지비만 부담하는 조합입니다.
- 기본공격 데미지 적용 경로가 3벌(즉발/투사체/스킬발)이고 "피해량/10 공명" 규칙이 4곳에 복제. 스킬발 데미지만 on_hit_effects:
  &[](commands.rs:1573)인 것이 의도인지 누락인지 판별 불가
- windup 스냅샷 규칙 비대칭: 즉발은 windup 시작 스냅샷, 투사체는 resolve 시점 재스냅샷(commands.rs:1352 vs 1429) — 문서화되지 않은 분기.
- sim.rs(2,998줄)만이 진짜 거대 파일: 실행 루프 + 17-variant 디스패치 + 스킬 스텝 실행기 + 승패 판정의 4책임 혼재.

  2.3 Skill fragments / abnormality — ★★★☆ (모델 적합, 실

- 좋음: 인벤토리/로드아웃/정책 3분할이 런 구조에 자연스럽고, 획득→연구→장착→전투 프로필의 단방향 파이프라인(stat_pipeline.rs가 순서를 주석으로 계약화). 호환성 실패가 failure_codes로 구조화되어 클라이언트까지 흐름.
- 문제: 무기 장착이 파편의 attack_interval_ms_reduction을 대입으로 덮어씀(battle/types.rs:376-380 vs skill_fragment.rs:813-819). 현재 live trace 파편 31개 전부가 이 필드를 갖고 있어, 무기 장착 직원에게서 파편 효과의 절반이 조용히 죽어 있습니다.
- behavior.rs(2,974줄)는 DTO 60종 + 입력 계약 + 결과 enum 10종 + JSON 글루 + GameError의 집합소이며, 결과 하나 추가에 5곳 수정이 필요한 3중 표현(BehaviorResult → contract 변환 400줄 → 수동 json! 매핑) 구조. GameError가 여기 살아서 data/ 계층이 프로토콜에 역방향 의존합니다.
- 검증 경로의 와일드카드 함정: validation.rs:202와 skill*data.rs:408의 * => 때문에 새 effect variant는 검증 없이 통과. ModifyStabilization은 이미 no-op으로 노출된 전례(sim.rs:1787).

  2.4 PVE encounter / wave / floor scaling — ★★★★ (가장 잘

- 좋음: "프리뷰가 곧 전투"라는 단일 진실 + validate_preven wave 금지" 테스트 봉인. 스케일링 수치는 전부run/policy.ron에 있고 코드는 조회/적용만. authored↔default 오버라이드의 모순을 로드 타임에 거부(ObjectiveWinContract).
- 문제: 스케일링이 프리뷰 캐시 시점의 floor_index에 박제되고 프리뷰는 재생성되지 않음(world/combat.rs:83-84) — 프리뷰 수명 정책이 바뀌는 순간 stale 스케일 버그. run policy의 extra_waves는 로드 타임 cross-reference 검증을 우회해 런타임 panic으로 터지며, Normal 인카운터의 "corroded만" 계약도 뚫습니다.
- 스폰 존 미매치 시 전 존으로 무음 폴백(enemy_spawns.rs:215-219) — 유일하게 fail-fast 규율에서 벗어난 곳.
- 수비 오브젝트 HP 350이 코드 상수 고정이라 층이 오를수 보이는 방식으로 어려워집니다.

  2.5 Node map / floor / gate + boss omen — ★★★☆ (코어 견고, 오멘은 볼트온)

- 좋음: 계층적 시드 파생, generation_policy의 데이터화 + row repair, 오멘 체인 저작 계약 검증(Event 스텝은 event_id 필수 등 상호배타 검증)의
  밀도.
- 문제: boss omen은 이벤트 시스템의 시민이 아니라 "생성 후 오버레이 + 완료 훅 + 그래프 외과수술"의 3중 특수 처리. 특히 ApplyBossOmenStepResult 이벤트 효과가 no-op(event_node.rs:283)이어서 "외면한다"를 골라도 오멘 스텝이 진행됩니다 — 데이터 계약과 런타임 의미의 정면 불일치.
- 실버그: map_distance_to_terminal(boss_omen.rs:479-495)이 Vec::pop 기반 DFS라 최단 거리가 아닌 임의 경로 길이를 반환 — 오멘 배치 우선순위가
  의도와 다르게 동작.
- 이벤트 노드의 event_id: None 폴백이 항상 DB의 첫 이벤트를 실행(map_content.rs:240-249) — 이벤트가 늘어나는 즉시 모든 일반 이벤트 노드가 white_night 이벤트를 재생하는 placeholder.
- reachable*available_node_ids의 Concealed guard가 fallthrough arm 때문에 죽어 있음(progression.rs:301-313). 새 노드 카테고리 추가 비용은 코드 6~8곳(그중 * => Ok(None) 폴스루는 누락 시 조용히 빈 노드로 동작).

  2.6 Unity-facing DTO / live RON loading — ★★☆ (검증은 최상급, 경계와 로더가 약점)

- 좋음: GameDataBase::new()의 로드 타임 검증 그래프(skill→buff, fragment→abnormality, defense route→tile range 등)와, 절차 생성 결과물까지 5개 시드로 계약 검사하는 드문 수준의 방어선.
- 문제: 로더 조립 코드가 3벌(game_server/main.rs, tests/common, world/tests)이고 적재 범위가 이미 서로 다름 → 그 드리프트의 결과가 현재의 테스트 컴파일 실패. 장비/아티팩트의 ability_activations → skill 참조는 프로덕션 로드 타임에 검증되지 않고 (죽어 있는) 테스트에만 존재.
- CombatPreview가 도메인/시뮬레이션 입력/wire의 3역 겸직 + #[serde(skip)] enemy_stat_scale — serde 라운드트립 시 스케일이 Default로 리셋되는 잠복 버그(런 저장을 직렬화로 전환하는 순간 발화).
- RON enum 케이싱이 3종 혼재(ALEPH / boss / Common), deny_unknown_fields는 run policy와 skill에만 적용. 상위 레거시 `artifacts.ron`, `events/shops.ron` 파일은 제거됐지만, RON authoring 일관성 정리는 별도 과제로 남아 있다.
- 모든 로딩이 include_str! — "live RON loading"은 사실상터 수정 = 재컴파일입니다.

  2.7 테스트 — ★★★☆ (계약 보호 장치 우수, 사각지대 명확)

- 좋음: ASCII 보드 DSL(테스트가 곧 문서), live 스킬 카탈로그 감사 매니페스트(스킬 추가 시 커버리지 갱신을 CI로 강제), 원자성 계약 테스트(부분
  적용 방지, catch_unwind 검증), 정책 상수 파생 단언.
- 문제: combat_setup 계층(enemy_spawns, defense_object, balance 등 7개 파일) 단위 테스트 0개. 동일 (scenario, seed) → 동일 event_log의 전투 리플레이 결정론 계약 테스트 부재 — battle_records 재생이 제품 기능인데 가장 비싼 회귀를 못 잡습니다. world/tests/combat.rs의 내부 상태 직접 주입 패턴(상태 머신 우회)과 재배치 코스트 자연 회복 미검증. 테스트가 리포지토리 파일(battle_records/, debug_event_log_exports/)을 오염시켜 git diff 노이즈 발생. skill_refactor_validation.rs는 이름과 달리 현역 계약 테스트이므로 삭제 금지, 개명 대상.

---

3. 가장 위험한 구조적 문제 Top 5

1. 전투 결과 커밋의 원자성 파괴 — 보상 이중 수령 가능 (world/combat.rs:1657-1661)
   보상/인벤토리 커밋 후 commit_staged_node_completion이 실패하면 상태가 CombatResult에 남아 CompleteCombatResult 재호출로 보상을 다시 받을 수 있습니다. 서버 권위 게임에서 경제 익스플로잇 직결이며, staged 패턴이 정확히 막으려던 사고입니다.

1. RNG ↔ 이벤트 로그 seq 커플링 — 로깅이 밸런스를 바꿈 (commands.rs:247, 777-784)
   관측 채널이 게임플레이 상태가 되어, 로그 한 줄 추가가 이 다. 리플레이/버전 간 검증이 원천 불가능해지고, 시간이갈수록 풀기 어려워지는 유형의 부채입니다. f32 Rapier 기본값(결정론 feature 미사용)과 함께 "결정론 보장 범위"를 지금 정해야 합니다.

1. Unity wire 경계의 침식 (behavior.rs:1196, snapshot.rs:494/725, types.rs:268)
   BattleEventLogEntry가 그대로 클라이언트 계약(이미 버전 2t, 상점 hidden_items 노출(치팅 벡터), CombatPreview의#[serde(skip)] 라운드트립 손실 — 개별 사안이 아니라 "무엇이 클라이언트 계약인가"를 코드에서 판별할 수 없다는 하나의 문제입니다. 라이브 서비스 전이 분리 비용이 가장 싼 마지막 시점입니다.

1. 로더 3벌 복제와 그로 인한 현재의 테스트 사멸 (tests/common/mod.rs:150 컴파일 에러)
   "테스트가 통과한 데이터 ≠ 서버가 로드하는 데이터"인 구조가 이미 실제 사고(live RON 회귀 테스트 전체 비활성)로 발현됐습니다. 장비 ability 참조 검증이 프로덕션 경로에 없는 것도 같은 뿌리입니다.

1. "현재 노드" 상태의 3~4중 저장 + 수동 동기화 (resources/state.rs:27, node_flow.rs:157, event_node.rs:53-165)
   클리어 지점 8곳이 각기 다른 조합을 비우고, boss_omen과 admin fixtures는 동기화 규약을 우회해 직접 조작합니다. 새 세션 필드/노드 타입이 추가될 때마다 전 지점 감사가 필요한, 회귀가 가장 쉽게 스며드는 지형입니다.

---

4. 좋은 설계로 보이는 부분

- 결정론 규율의 일관성: run→floor→node→wave→unit의 네임스페이스 시드 파생, HashMap 순회 전 UUID 정렬, RNG 전역 상태 없음. 코드베이스 전체에서 예외 없이 지켜집니다.
- 로드 타임 데이터 계약: cross-reference 검증 그래프 + 절차 생성물까지 검사 + #[should_panic] 계약 테스트 동반. 컨텐츠 오류가 전투 중이 아니라 부팅 시 잡힙니다.
- "프리뷰가 곧 전투" 단일 진실: 전투가 프리뷰를 소비하는 버그가 구조적으로 차단됨.
- 이벤트 로그 중심 아키텍처: 시뮬레이션과 관측의 분리, 로그 기반 사후 검증기, 로그에서 파생되는 결과 통계.
- plan/commit 트랜잭션 패턴과 이를 보호하는 원자성 테스트들.
- 조기 추상화 거부의 자기 규율: mission_policy.rs의 "Do ework" 주석, SplitRoom fallback을 의도적으로 에러로
  차단하고 테스트로 고정한 것.

---

5. 리팩터링 후보

소규모 (반나절 이하, 저위험)

┌───────────────────────────────────────────────────────────────┬──────────────────────────────────────┐
│ 항목 │ 위치 │
├───────────────────────────────────────────────────────────────┼──────────────────────────────────────┤
│ tests/common omen_chain_id 컴파일 수정 (최우선) │ tests/common/mod.rs:150 │
├───────────────────────────────────────────────────────────────┼──────────────────────────────────────┤
│ 스냅샷에서 hidden_items/hidden_item_uuids 제거 │ world/snapshot.rs:725-734 │
├───────────────────────────────────────────────────────────────┼──────────────────────────────────────┤
│ map_distance_to_terminal DFS→BFS │ boss_omen.rs:479 │
├───────────────────────────────────────────────────────────────┼──────────────────────────────────────┤
│ record_battle dedupe 키 교체 (재도전 기록 유실) │ world/state.rs:697 │
├───────────────────────────────────────────────────────────────┼──────────────────────────────────────┤
│ 검증 경로 와일드카드 2곳 exhaustive match화 │ validation.rs:202, skill_data.rs:408 │
├─────────────────────────────────────────────────────────────────────────┤
│ equipment/artifact ability 참조 검증을 프로덕션 로드 타임으로 │ validation.rs │
├───────────────────────────────────────────────────────────────┼──────────────────────────────────────┤
│ 스폰 존 폴백에 warn, route_id 미존재 warn/에러화 │ enemy_spawns.rs:215 │
├─────────────────────────────────────────────────────────────────────────┤
│ 코어 경로 panic! 4곳 GameError화 │ world/state.rs:434 외 │
├───────────────────────────────────────────────────────────────┼──────────────────────────────────────┤
│ ActionValidator 제거 또는 단일화 │ world/state.rs:108 │
├───────────────────────────────────────────────────────────────┼──────────────────────────────────────┤
│ "피해량/10 공명" 4벌 → 헬퍼 1개 │ commands.rs:849/925/1239/1366 │
├───────────────────────────────────────────────────────────────┼──────────────────────────────────────┤
│ 완료: 죽은 RON 파일 삭제 (artifacts.ron, events/shops.ron) │ game_resources/data/ │
├───────────────────────────────────────────────────────────────┼──────────────────────────────────────┤
│ try_generate_for_node → \_at_floor(…, 0) 위임 │ combat_preview/mod.rs:463-547 │
├───────────────────────────────────────────────────────────────┼──────────────────────────────────────┤
│ Concealed guard 의도 확정 후 죽은 arm 정리 │ progression.rs:301-313 │
├─────────────────────────────────────────────────────────────────────────┤
│ skill_refactor_validation.rs → skill_step_pipeline.rs 개명 │ tests/ │
└───────────────────────────────────────────────────────────────┴──────────────────────────────────────┘

중간 규모 (수일)

- 로더 단일화: GameDataBase::load_live() -> Result<…> 하나로 서버/테스트 3벌 수렴 + 검증의 Result화(panic은 최상위 한 곳).
- handle_complete_combat_result 분해 + 커밋 순서 재배치: "순수 계산 → 단일 커밋"으로 부분 커밋 제거.
- Wire enum/DTO 경계 복원: Debug 문자열 → serde rename enum, 내부 타입 직통 지점(NodeSession, BattleEventLogEntry, unit.stats)에 명시 DTO.
  push(BehaviorResult)와 pull(RunSnapshotDto)이 같은 프로
- behavior.rs 분할: GameError 분리(데이터 계층 역방향 의존 해소)가 선결.
- 층 전환 단일화: gate 경유와 boss 완료 경유를 advance_to_next_floor(reason) 하나로.
- extra_waves 검증을 로드 타임으로 + 프리뷰에 scaling st
- sim.rs 분할: 캐스트 상태머신 / 스킬 스텝 실행기 / 승패 판정 분리(basic_attack.rs 선례 확장).
- 무기 vs 파편 공격 간격 우선순위 명시(정책 확정 후): 대 검증.
- "elite" contains 매칭 3곳 → 노드 정의 데이터 필드로 승격.
- 이벤트 노드에 이벤트 풀 도입("첫 이벤트 폴백" 제거).

장기 재설계

- RNG-로그 분리: 크리롤/투사체 시드를 게임플레이 고유 카운터로 파생. 이벤트 로그 버전 bump와 함께 1회 수행.
- 이동 백엔드 결정: Rapier에 enhanced-determinism + 유닛 충돌까지 위임해 값을 뽑거나, Direct를 기본값으로 승격.
- BattleId newtype: battle_uuid == node_id.0 암묵 별칭 제거.
- 노드 콘텐츠 라우팅의 등록 기반화(NodeKindSpec 테이블): 카테고리 추가 산탄총 6~8곳 → 1곳.
- boss omen을 범용 맵 오버레이(MapMutator) 파이프라인으로 일반화.
- include_str! → 파일시스템 로딩 + 버전드 데이터 팩 (검증 Result화 선행).

---

6. 테스트 보강 후보 (우선순위순)

1. 전투 리플레이 결정론 계약: 동일 (BattleScenario, GameDataBase, seed) 2회 실행 → event_log 완전 일치. battle_records 재생이 제품 기능이므로 최우선.
1. combat_setup 단위 테스트: 스폰 위치의 존 준수, 수비 오브젝트의 경로 종점 배치, 층별 스케일 단조성.
1. 보상 이중 수령 회귀 테스트: commit_staged_node_completion 실패 주입 후 CompleteCombatResult 재호출 시 보상 불변 확인 (Top 5 #1의 봉인).
1. 배치 코스트 자연 회복 E2E: 내부 필드 조작 없이 서버 틱만으로 재배치 성공 검증.
1. 생존 타이머 만료 → Player 승리 시뮬레이션 계약.
1. 트라우마 회복 매직넘버(68/20/81) → 정책 파생 수식화.
1. 맵 불변식 property 테스트: 임의 시드 N개에 대해 도달 가능성/터미널 규칙.
1. 테스트 산출물의 tempdir 이전 (battle_records/debug_event_log_exports git 오염 제거).

---

7. 정책 질문 (결정 필요 — 임의 결론 내리지 않음)

결정론·프로토콜

1. 결정론 보장 범위는 "동일 바이너리 재현"까지인가, "크로스 플랫폼/버전 간 리플레이 검증"까지인가? (후자면 RNG-로그 분리와 f32 Rapier 교체가 즉시 로드맵행)
2. BattleEventLogEntry와 Debug 문자열을 Unity가 이미 파싱 중인가? (그렇다면 wire 분리는 브레이킹 체인지 — 버저닝 전략 필요)
3. 데이터 hot-reload를 지원할 것인가? (K8s pod 배포 정책과 맞물림 — 안 하면 include_str! 유지 확정, 하면 검증 Result화 선행)

전투 규칙 4. 스킬발 데미지에 OnHit 트리거 미적용(on_hit_effects: &[])은 의도인가 누락인가? 5. windup 중 스탯 변화: 즉발=시작 스냅샷 / 투사체=착탄 재스냅샷 비대칭을 유지할 것인가? 6. 무기 장착 시 파편 attack_interval_ms_reduction 소실이 의도인가? (의도라면 live trace 파편 31개의 밸런스가 사실상 반토막) 7. 수비 오브젝트 HP(350 고정)도 층 스케일링을 받아야 하는가? Manual 웨이브 count 미스케일은 영구 정책인가?

진행·오멘 8. 오멘 이벤트에서 "외면한다"를 골라도 스텝이 진행되는 현재 동작이 의도인가? (ApplyBossOmenStepResult가 no-op) 9. Concealed 노드는 인접 완료 시 자동 공개가 맞는가, 별도 해금 수단까지 은폐 유지인가? 10. 런 종료(RunComplete/RunFailed) 후 재시작은 GameCore 재생성인가 StartNewGame 재허용인가? (후자면 리셋 범위 정의 필요) 11. 프리뷰 캐시 수명: 층 이동/재정찰 시 무효화 규칙은?

데이터·기타 12. extra_waves의 소유권: run policy(전 인카운터 공통)가 맞는가, 인카운터별 정의가 맞는가? 13. RON enum 케이싱 표준(ALEPH/boss/Common 중 하나로 통일)과 deny_unknown_fields 전면 적용 여부 — 데이터 저작 워크플로우와 합의 필요. 14. 파편 강화 진행도의 공유 범위: 종류 단위(현재) vs 사본 단위? 15. battle_records의 목적: 디버그 아티팩트인가 정산 근거인가? (재도전 기록 유실 심각도와 저장 위치 정책이 여기에 달림)

---

8. 추천 작업 순서

1. [즉시] 테스트 부활: tests/common 컴파일 수정 → live R 는 이후 모든 리팩터링의 안전망이 없습니다.
1. [1주 내] 보안·경제 직결 수정: 보상 이중 수령 경로 봉인(+회귀 테스트), 스냅샷 hidden_items 제거, 코어 panic 4곳 에러화. 전부 소규모이고 파급이 큽니다.
1. [결정 선행] 정책 질문 1~3 확정: 결정론 범위·wire 계약·hot-reload는 이후 모든 구조 작업의 방향을 정합니다. Unity 클라이언트가 본격 연동되기 전인 지금이 wire 분리 비용이 최소인 시점입니다.
1. [2~3주] source of truth 정리: 로더 단일화 → 검증 Result화 → equipment ability 검증 편입. 이어서 "현재 노드" 중복 축소(clear_node_context 헬퍼부터).
1. [결정 후] 전투 코어 정책 수정: RNG-로그 분리(이벤트 로그 버전 bump와 함께 1회), 이동 백엔드 결정, 무기/파편 간격 규칙 확정.
1. [콘텐츠 확장 전] 산탄총 제거: 검증 와일드카드 2곳, "elite" contains 3곳, 노드 라우팅 \_ => Ok(None) — 새 컨텐츠가 늘기 전에 막아야 비용이 최소입니다.
1. [여유 시] 파일 분할: sim.rs, behavior.rs, handle_complete_combat_result. 기능 변화 없는 기계적 작업이므로 안전망(1번) 복구 후 언제든.

전체적으로 이 코드베이스는 "잘 설계된 코어에 경계 관리가 따라가지 못한" 상태이지, 재설계가 필요한 상태가 아닙니다. 위 순서대로면 큰 구조 변경 없이 대부분의 리스크를 해소할 수 있습니다.
