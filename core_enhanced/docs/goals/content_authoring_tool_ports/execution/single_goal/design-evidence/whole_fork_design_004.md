# R03 whole opaque fork design 004 — 한정 읽기 조사

권장 최소 구현은 **기존 derive(Clone)을 유지하고, 실제 clone 직전에 동일한 private structural preflight를 호출하는 것**이다. CoreApplication의 새 opaque fork, 기존 GameCore application draft, ActiveBattleSession tactical draft, BattleCore boss draft에 각각 연결한다. preflight는 실제 복제되는 owned collections/문자열과 공유되는 Arc를 구별해야 한다. 현재 runtime retained count와 serialized-shape만으로 whole clone을 포괄했다고 할 수 없다.

이 문서는 설계 보조이며 완료 감사나 실행 증명은 아니다. root가 scenario binding을 수정하는 동안 application/world/state/core/control 중심으로 읽고, 직접 owned 필드의 타입 정의만 주변 파일에서 따라갔다. production/tests/docs/Git/build/test를 변경하거나 실행하지 않았다. 지정한 두 새 `/tmp` 파일만 작성한다. 숫자별 allocator/physics 내부 storage의 정확한 비용은 이번 제한 source에서 입증되지 않은 부분을 명시한다.

## 1. 소유권과 실제 복제 지점

`CoreApplication { core: GameCore }`는 public Clone이 없다(application.rs781–791). `GameCore`는 derive(Clone)이며 `{ state: GameCoreState, game_data: ValidatedGameData, run_seed:u64 }`이다(world.rs50–55). game_data는 immutable generation Arc 핸들이므로 이 fork 때문에 데이터베이스 전체를 복제하지 않는다. state는 아래 graph대로 복제한다.

| 실제 지점 | 현재 동작 | 최소 추가 seam |
| --- | --- | --- |
| 새 `CoreApplication::fork_for_authoring` | 현재 없음 | crate-private `Result<Self, ApplicationError>`; 내부 `core.try_clone_for_authoring` 호출, opaque app 반환 |
| world/application.rs183 | 모든 mutating public application route가 `self.clone()`으로 whole GameCore draft 생성 | 바로 앞 공통 GameCore preflight. 외부 fork에서 이미 검사했어도 **새 실제 복제 비용**은 다시 부과 |
| world/combat.rs1823–1838 `with_staged_tactical_world_mutation` | ActiveBattleSession 전체 clone | 해당 owner의 preflight 후 기존 clone; entry_snapshot/BattleCore가 다시 복제되는 비용 포함 |
| core/boss_runtime.rs1761–1770 `run_boss_runtime_boundary` | BattleCore 전체 clone | BattleCore preflight 후 기존 clone |
| core/boss_runtime.rs1773–1778 및 `plan_next`298 | boss runtime `.cloned()`와 `self.clone()` 반복 | BossEncounterRuntime의 같은 typed cost helper 재사용 |
| state.rs548–565 BattleEntrySnapshot::capture | run/inventory/roster/UUID/checkpoint 등 다수 owned fields를 따로 clone | entry snapshot projection 비용을 capture **전** charge. whole GameCore clone과 별도 실제 copy |
| state.rs394–420 RunCheckpointPayload::capture | run fields와 inventory/roster/UUID 등을 별도 clone | 해당 payload typed visitor를 capture 전에 재사용 |

`clone_non_battle_transaction_state`(state.rs163–189)는 active battle가 있으면 None이고 profile을 default로 만든다. 이것은 완전한 authoring fork가 아니다. V6/RunCheckpointPayload는 active battle와 profile을 포함하지 않고 restore가 namespace monotonic 규칙도 적용한다(state.rs423–457). public snapshot 역시 projection이므로 substitute가 될 수 없다.

기존 derive Clone 구현 자체에 fallible `charge`를 끼워 넣지 않는다. Clone에서 silent partial copy/default 또는 panic으로 control failure를 전달할 수 없다. scoped call sites에 fallible helper를 붙이고, control 미설치 시 바로 기존 clone으로 가는 경로를 유지한다. constructor/startup 전 top-level scope 설치는 기존 계획대로 필요하다.

## 2. GameCoreState에서 실제로 방문할 필드

state.rs137–160의 21개 필드를 `let GameCoreState { player, lifecycle, ... } = self;`처럼 **`..` 없는 exhaustive destructuring**으로 visitor에 결속하면 새 owned field 추가를 compile 단계에서 드러낼 수 있다. serializer가 빠뜨린 필드를 Debug 문자열로 대체하거나 public DTO에서 재추정하지 않는다.

| owner | 실제 owned 내용과 preflight 대상 | 공유/작은 고정 내용 |
| --- | --- | --- |
| GameCoreState 상위 | player name, lifecycle variant strings, active_node_content, pending_mastered_elite_sweep, node_session, RunState, VN state/trigger sets, inventory, roster, starter_candidates의 EmployeeDefinition, fragments/policy, profile, active_battle, checkpoint | enkephalin/game mode/power scalars; seed 값 자체 |
| RunState (state.rs1413–1427) | RunMap nodes/start IDs/terminals; map progression vectors; facility/encounter progression; event/shop/combat-preview/abnormality-attempt maps; recruitment; record progress; recovery restriction allowed_node_ids | RunProgression은 seed+mode+floor 상태(map/progression.rs25–36) |
| Inventory | EquipmentInventory HashMap<Uuid,OwnedEquipment>, material HashMap<String,u32>, consumable HashMap<String,ConsumableStack>; 키와 entries/slots/capacity | OwnedEquipment.meta와 ConsumableStack.meta는 immutable Arc이며 target catalog 전체는 복제되지 않음(inventory.rs565–570,703–706) |
| EmployeeRoster | HashMap<Uuid,Employee>, Employee name/definition ID/profile/loadout/skill-fragments/active modifier/combat profile 등 값 필드 | roster는 Arc가 아니며 루트·entry·checkpoint·employee_attempt마다 별도 copy |
| VN/profile | VN active/queue의 session/vn/scene/choice strings; persistent/checkpoint consumed HashSets; profile registered ID BTreeSet | profile booleans/u32 scalar |
| UuidManager | BTreeMap<u64,u64> counters를 복제 | run_seed와 next-state를 그대로 복사; visitor는 `next`를 절대 호출하지 않음(uuid_manager.rs12–16,36–40) |
| RunCheckpointState | payload:Option<RunCheckpointPayload>는 deep clone. 현재 run과 유사한 내용이 있어도 **별도 소유라 두 번 계산** | encoded_core_payload:Option<Arc<[u8]>>는 shared handle. serde(skip)이라고 누락된 대용량 deep bytes가 아님 |
| pending sweep | MasteredEliteSweepCommitPlan의 result, EconomyGrantState, encounter_progression_after, node_completion을 방문 | 단순 'Option 1개'로 계산하지 않음. grant_state는 inventory/fragments/roster/UUID/enkephalin의 별도 copy |
| ActiveNodeContent | exhaustive enum: Shop/Reward/Event/SavePoint/Maintenance/HQ/CombatBattle. Shop offers/tags/purchased-set, Reward reward vector, terminal result/rewards/participants/employee_conditions/compendium candidates를 소유 | CombatBattle.event_log는 cfg(test/debug/test-fixtures)에서만 존재; profile authority는 여기에 없음 |

위 table의 private child type에는 owner-local `visit_authoring_clone_shape(&self, &mut CloneMeter)`가 필요하다. 공개 iterator가 있어도 HashMap capacity를 읽을 수 없다면 owner에서 scalar capacity accessor/visitor를 제공한다. mutable getter를 추가하지 않는다. RunState의 facade snapshot을 serialize하여 event_sessions/shop_sessions/combat_previews/restriction을 간접 추정하지 않는다.

`serde`가 실제 owned fields를 모두 포함하고 shared metadata recursion이 없음을 확인한 leaf에만 counting Serialize를 **보수적 logical work**로 재사용할 수 있다. Inventory를 통째로 serialize하면 Arc metadata까지 반복 순회해 실제 clone과 다른 비용이 된다. 그리고 HashMap capacity/private state가 누락되므로 이것만으로 전체 allocation bound를 주장할 수 없다.

## 3. ActiveBattleSession과 중복 소유

state.rs507–548은 중요한 duplication 경계다. ActiveBattleSession은 outcome_policies, combat_preview, BattleCore, BattleExecutionState, entry_snapshot, employee_attempt, live_deployment, tactical_focus를 각각 소유한다.

- entry_snapshot은 run·active_node_content·node_session·VN·UUID·inventory·roster·starter candidates·fragments·checkpoint를 다시 소유한다. 같은 내용이라는 이유로 root와 deduplicate하면 실제 copy를 과소계산한다. `BattleEntrySnapshot`에 active_battle 자체는 없어 재귀 battle stack은 생기지 않는다.
- `BattleEmployeeAttemptState`는 EmployeeRoster와 processed_incapacitations HashSet을 소유한다(state.rs603–607).
- `LiveBattleDeploymentState`는 deployed_units/redeploy_locks HashMaps를 소유한다(618–625). entries 외 spare capacity도 native-clone proxy 대상이다.
- `BattleExecutionState`는 last time/hooks/finalized 및 optional terminal signal이다(sim.rs61–99). terminal outcome ID String을 제외하면 고정 크기다. time/finished getter만으로 clone하여 숨겨진 startup/finalized 상태를 새로 만들면 안 된다.
- playback/remainder/tactical resource/revision/UUID는 고정 상태를 그대로 복사한다. clock/UUID/RNG를 진행해 복제 대상을 재구성하지 않는다.

## 4. BattleCore의 기존 count가 포괄하지 않는 저장소

core/mod.rs79–127의 derive Clone을 그대로 유지한다. 현재 `admit_authoring_runtime_replacement`(221–289)은 unit/tactical/item/graveyard/buff/cast/proc/projectile/area/movement/field-work/live-signal의 root **항목 수**와 cast progress/deferred/context/hit 및 delivery hit vectors를 합산한다. 이미 연결된 entity admission으로서 유용하지만, 함수 주석도 byte/backend storage는 별도라고 명시한다. 이것을 whole-fork meter로 이름만 바꾸면 충분하지 않다.

| 추가로 방문할 concrete type/path | actual owned clone 내용 |
| --- | --- |
| queue::EventQueue(BinaryHeap<BattleEvent>) | heap length/slots 및 각 BattleEvent 문자열·SkillId·DamageSourceSnapshot 보너스 vector. enqueue cap은 heap의 기존 payload clone 비용까지 부과하지 않음(queue.rs9–32, enums.rs20–96) |
| BattleScenario + BattlefieldLayout | scenario groups/spawns/drafts/events/terminal outcomes/tactical points/field-work defs/valid tiles/obstacles; layout의 별도 valid_tiles/static_obstacles HashSets(battlefield/mod.rs11–17). ResolvedBossEncounterSetup의 Arc definition은 shared지만 actors/zones/cells/routes는 owned |
| ScenarioRuntimeState + WaveRuntimeState | spawned_groups HashSet, unit_refs HashMap, wave triggers의 String/group/activation/request/status, death_epochs; source scenario와 runtime 요청 복제본을 각각 계산 |
| SpawnKernelState | requests의 request placements/status spawned unit vectors, triggers, compendium_candidates, candidate-ID set(spawn_kernel.rs201–223). settled history도 clone 대상 |
| SummonRuntimeState | pending request views, tuple keys/counters, batch metadata/member vectors/removal signal ID(summon_runtime.rs42–80). live units만 세면 과소계산 |
| BossEncounterRuntime | actors, summon zones/routes, signal epoch/set, pending phase bool vector; actor cursor/cooldown maps; active pattern loop/batch/cleared maps; active tasks와 owned IDs. compiled definition/behavior/pattern/argument-frame Arcs는 handle copy(boss_runtime.rs73–155) |
| RuntimeUnit / keys | source_identity/skill/pending-skill IDs, basic attack nested range/delivery, enemy route cells, cfg target_traits; AbilityProcKey ability_id. 스칼라 unit 1개와 동일하지 않음(types.rs400–441,369–380) |
| casts/projectiles/areas | deferred skill/step strings; effect source Arc; impact contexts/hits; source_snapshot bonus vector; tile_range rows, optional VFX strings. 현재 retained count가 vector 길이 일부를 세지만 문자열/payload bytes는 세지 않음(types.rs60–147,326–348) |
| block/field work/movement | blocker_to_enemies child vectors, reverse map; FieldWorkObjectRuntime의 **owned definition와 assignment map**; active movement segment의 실제 fields. field-work root 수만으로 work cells를 덮지 않음 |
| result_stats_accumulator | unit_sources/employees/active_deployments HashMaps. BattleResultStatsDto serialization과 private accumulator는 같지 않음(result_stats.rs115–123) |
| recording/history | cause stack, command String stack, optional proc origin; effect source Arc는 share. event history는 아래 COW 별도 경계 |
| RapierMovementWorld | arenas/BVH/narrow-phase/joints/handle maps 등 아래 backend 별도 경계 |

`BattleCore.game_data:Arc<GameDataBase>`, compiled boss graph, source metadata Arcs를 재귀 방문하지 않는다. Immutable reference의 수와 핸들 복사만 meter에 기록한다. `seed`, cast/projectile/area/damage-source/event-log sequence, spawn/summon/boss activation counters는 현재 값을 복제한다. 이번에 읽은 최상위 state에는 별도 PRNG object 대신 seed/counters가 있지만, **모든 nested dependency에 RNG가 없다는 전역 증명은 하지 않았다**. opaque clone이 해당 실제 상태를 자동 보존하게 한다.

## 5. 최소 private meter와 call flow

새 공개 framework 없이 `authoring::control` 옆 private `clone_shape.rs` 또는 control 내부에 작은 `CloneMeter`를 둔다. owner-local visitor들이 한 meter에 합산한다. `CloneMeter`의 단위는 명확히 분리한다.

- `owned_records`: 실제 복제되는 collection elements/enum payload records.
- `owned_payload_bytes`: String/byte buffer/vector의 실제 owned 내용 byte upper-cost. native fixed element는 size_of::<T>()와 len을 checked 곱셈하되 child heap은 별도로 방문한다.
- `native_slots`: HashMap/HashSet capacity, heap/arena 등 native storage의 별도 logical slots. `len`만으로 sparse map clone/scan 비용을 대체하지 않는다.
- `shared_handles`: Arc clone 수. pointee 전체 bytes로 부과하지 않는다.
- `backend_shape`: 아래 Rapier contract가 제공하는 별도 scalar 구성. 알려지지 않은 항목을 0으로 채우지 않는다.

이 값은 allocator metadata/fragmentation 포함한 정확 RSS가 아니다. `size_of_val`도 heap pointee를 포함하지 않는다. native HashMap buckets와 public capacity의 비율, BTree nodes/rapier arena layout은 pinned dependency/standard-library contract에 의존하므로 measured-exact bytes로 이름 붙이지 않는다. Root가 하나의 `max_clone_units` 또는 별도 `max_clone_payload_bytes`/native slots의 trusted cap을 정할 수 있지만, 이것은 result/metadata/retained_entities와 다른 자원이어야 한다. 기존 max_work는 scan과 admitted-copy work의 누적 fuel에 계속 사용한다. 복제본이 여러 개일 때 한 번만 할인하지 않는다.

호출 순서:

```text
existing top-level authoring control scope
  -> poll(before_shape_scan)
  -> field-shape scan with bounded per-record polling/charge
  -> final checked clone-shape admission + deterministic work charge
  -> poll(before_clone)
  -> existing full derive clone
  -> poll(after_clone)
  -> public operation on opaque candidate
  -> mandatory result/receipt preparation and existing final acceptance
```

scan 자체도 bounded해야 한다. HashMap iteration은 spare table을 스캔할 수 있으므로 해당 capacity의 work를 **loop 전에** 부과한다. String length를 얻기 위해 문자열을 복제/format/serialize하지 않는다. 각 child 방문 전 고정 scan charge/poll을 하고 payload/native totals는 순서 독립적으로 누적한 뒤 최종 admission에서 확인한다. 서로 다른 길이의 HashMap key를 방문하는 순서에 따라 first failure receipt가 달라지는 문제를 피하려면 순서를 그대로 두되 payload 합은 마지막에 비교하고, 순회 자체에는 일정한 record charge를 사용한다. ledger 정렬을 위해 cloned Vec를 먼저 만들지 않는다.

whole-fork visitor에서 GameCoreState→ActiveBattleSession→BattleCore preflight가 **한 meter**에 기여한다. 하위 helper가 별도 top-level run을 열면 fuel 초기화/nested attempt 오류가 생기므로 금지한다. 기존 `admit_authoring_runtime_growth(0)`도 부수 검증으로 재사용할 수 있지만 이 함수의 scan을 별도 과금하는 사실을 숨기지 않는다. visitor는 source state를 mutate하지 않는다.

표준 derive Clone 한 호출 도중에는 cancellation을 확인할 수 없다. preflight로 허용한 단일 복제량을 제한하고 복제 직후 poll해서 cancellation이면 candidate를 drop한다. 이를 hard real-time cancellation이라고 주장하지 않는다. 더 촘촘한 중단이 반드시 필요하면 fieldwise fallible clone가 필요하지만 HashMap을 새 insertion 순서로 재구성하면서 기존 iteration/order/seed를 바꾸지 않는 검증이 따라야 한다. 현재 최소안은 기존 clone semantics를 보존하는 pre/post poll이다.

## 6. 이벤트 이력의 Arc와 후속 COW

BattleEventEntries는 `Arc<Vec<BattleEventLogEntry>>`이고 derive Clone은 핸들만 복제한다(event_log.rs76–101). 따라서 whole-fork 시 전체 history를 deep-copy했다고 count하는 것은 실제와 다르다. 하지만 첫 mutable access는 `Arc::make_mut`를 호출해 retained entries를 복제할 수 있다. append/reserve/remove/clear가 그 지점이다(검색 spans514–676).

가장 작은 정확한 분리는 whole-fork visitor에서 history handle만 가격을 매기고, **history의 첫 COW write 전에** 별도 deep-copy preflight를 하는 것이다. entries Vec 본체·각 event의 owned payload와 source Arc handles를 계산한다. `skill_effect_source`는 serde(skip)이므로 serialized event bytes에 나타나지 않지만 pointee의 불변 source 전체를 deep-copy하는 것도 아니다. scalar accounting fields 역시 skip되어도 derive Clone은 그대로 복사한다.

현재 BattleEventEntries의 DerefMut는 infallible이므로 그 안에서 실패를 무시하고 copy를 진행하거나 빈 vector로 돌려주면 안 된다. 기존 mutable log writer가 COW를 일으키기 전에 fallible private guard를 거치도록 owner seam을 정해야 한다. 이 guard의 실제 적용/모든 mutable log 경로 확인은 이후 구현 의무이며 이번 조사에서 연결되었다고 판정하지 않는다. entry-window의 count/encoded/span cap는 별도 bounded history 정책이고 clone/COW work charge를 자동으로 대체하지 않는다.

미래 optional authoring trace storage는 기존 gameplay history와 구별한다. optional trace의 실제 rows/copy bytes를 execution fuel/mandatory clone allowance에 더하면 trace flag가 acceptance를 바꿀 수 있다. optional observer storage는 자체 bounded budget으로 복제하고, mandatory semantic counter는 trace off/on에 동일한 fixed allowance를 둔다.

## 7. Rapier backend — 현재 확인 가능한 최소 연결과 남은 근거

rapier_backend.rs36–47의 derive Clone에는 IslandManager, BroadPhaseBvh, NarrowPhase, RigidBodySet, ColliderSet, ImpulseJointSet, MultibodyJointSet, controller, unit_handles HashMap, static_obstacles HashMap가 포함된다. unit body/public battle DTO만 복제하여 backend를 재구성하는 변경은 현재 clone과 동등하다고 입증되지 않았다. 기존 backend clone를 유지한다.

wrapper source의 좁은 mutation set은 unit/static insert(212–246), remove(115–129,254–268), sync/update, clear(167–178), broad_phase.update(291–305)다. 명시 joint 생성은 이 wrapper에서 발견하지 못했다. 그러나 rigid-body/collider removal 과정이 하위 storage를 변경할 수 있고, arena/BVH의 남은 capacity/cache는 live unit 수와 같지 않다. 모든 내부 capacity가 노출되어 있다는 근거도 이번 source에는 없다.

따라서 owner-local `RapierMovementWorld::visit_authoring_clone_shape`를 **별도 seam**으로 둔다. 즉시 wrapper가 직접 알 수 있는 handles/static-obstacle len/capacity를 수집한다. 추가로 pinned Rapier 타입이 실제 제공하는 arena/storage 계측 API가 있는지 확인하여 그 결과를 연결해야 한다. API가 없다면 wrapper의 실제 insert/remove/update 지점에서 저장소의 retained high-water/growth credit을 추적하고, pinned 내부 clone/container의 상한과 결속해야 한다. 단순 `units*constant`를 임의로 넣으면 whole backend bound가 아니다. repeated create/remove 이후 live count가 작아져도 arena/BVH capacity가 작아졌다고 가정하지 않는다.

이 backend 부분은 **현재 실제 API 호출명/내부 storage upper bound가 미확정인 구체 gap**이다. 이번 조사 범위를 넘어 Rapier dependency 내부를 읽거나 테스트하지 않았으므로 invented `.capacity()` accessor나 고정 계수를 승인된 설계처럼 적지 않았다. world/core typed visitor는 먼저 구체화할 수 있으나 backend 결과를 0/unknown 상태로 성공시키면서 whole native clone이 bounded라고 마감할 수 없다. 필요하면 다음 좁은 helper를 pinned dependency clone-layout 조사에 배정할 수 있다. 사용자에게 새 정책 결정을 요구할 사안은 아니다.

## 8. 구현 순서와 확인할 oracle

1. control의 private clone meter와 explicit limits/usage 단위를 정한다. no scope이면 즉시 return. 실제 source field 목록으로 GameCoreState/RunState/entry/checkpoint/active owner visitors를 exhaustive하게 연결한다.
2. BattleCore visitor를 root field list와 각 private child owner에 연결한다. 기존 retained count helper와 역할을 구분하고 backend/COW unresolved를 명시한다.
3. 새 opaque CoreApplication fork와 위 표의 실제 whole/active/boss clone 지점들에 preflight→기존 clone→poll을 붙인다. std Clone trait을 fallible하게 흉내내지 않는다.
4. backend storage와 history first-COW의 실제 copy work를 닫고, 일반 caller와 authoring candidate의 parity를 검증한다.

검증은 root가 구현 뒤 수행할 일이며 여기서는 실행하지 않았다: pre-cancel 및 exact/one-over clone budget가 clone 진입 전에 실패하는지; 작은 live unit 수/큰 dormant run checkpoint·entry snapshot·settled spawn history·backend high-water가 과소계산되지 않는지; cancel-before/after clone A의 identity/UUID/counters/backend/receipt가 보존되는지; outer+inner+tactical+boss의 실제 반복 clone가 각각 과금되는지; history fork는 share하고 first-write는 정확히 분리되는지; same input ordinary continuation 및 trace off/on mandatory 결과가 같은지; default release와 debug cfg payload 차이가 정직하게 계산되는지.

## 9. 상태와 제한

현재 입력에서 확인한 것은 source 구조와 미연결 seam이다. 전체 clone safety를 달성했다고 판단하지 않는다. 실제 source hash/spans는 JSON에 남겼다. 최초와 종료 hash를 모두 가진 중심 파일만 drift/no-drift를 판정하며, 나머지 supporting type은 종료 hash 한 개여서 원자적 initial snapshot을 주장하지 않는다. 검색만 한 symbols는 상세 leaf field 검증과 구별했다. 모든 repository input을 검토하거나 completion audit를 대체하지 않는다.

| Source | Close SHA-256 | Capture |
| --- | --- | --- |
| `core_enhanced/src/application.rs` | `636b3c9cc15ae2adfa39a15392e0dee22ee7c6683885087f9cc77cefda66081a` | unchanged_between_hashes |
| `core_enhanced/src/game/world.rs` | `fc72d1029c33c560f918dc2487071c5720ad231d6983b9aa66dc15e4d5ee75ed` | unchanged_between_hashes |
| `core_enhanced/src/game/world/state.rs` | `129a4063f7e448b5795fd370a22a997f66eb09cd3029fb87d41fee892cf34bb0` | unchanged_between_hashes |
| `core_enhanced/src/game/world/application.rs` | `eda3e77e4509368aec58228b8070b5d7edfdab6fc6f98ae036cb45e97ea7eafd` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/world/combat.rs` | `23d440b58516258e9e174d586c6d06a675f9ce8c9b21534cde102574b2feb35f` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/world/active_run.rs` | `c344f0fff0aa28e3f8afb9b5a48c2c706855dbca13c5c53342aa14d2b7d52b20` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/world/grant_state.rs` | `248d55a4feca4e4fac376327254c6ff6ce31721c31b7df750f13c9c3420167de` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/world/mastered_elite_sweep.rs` | `4edc42a44b2fb8da17ec0717b7a01687bd328c41129f0673cf57491f26ef1ed7` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/battle/core/mod.rs` | `026b85b1b7d47cfc3f08d3e88467dfe7de9558926aa1783e76b3cb56bf5bf67e` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/battle/core/types.rs` | `385402e5e49ead95a505f2345211875542ed370aa5558e73472c33de03e42a2d` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/battle/core/sim.rs` | `e624854ac13e8172b104c54e4626a761c4d017d67b8ffdb4adf3f15e50a31207` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/battle/core/queue.rs` | `a2b3edc11f43590fc368cf3c9ae7be8919055bdd3d5c2aa448d533a9372714d5` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/battle/core/movement/rapier_backend.rs` | `764124edb49ab1181352f1e9da83b0ef48d2fe517dc64dc32982402fcf153af9` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/battle/core/boss_runtime.rs` | `600fcbad2734539d90dfd55c780a5370d1b8e3109774bc6cc39846baafd29c00` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/battle/core/spawn_kernel.rs` | `3a970eb9bcac440ae61b2a9074fecc4d3654b347b7d9fd43511e6b1386fb884e` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/battle/core/summon_runtime.rs` | `9fc236ce1032422b89ac3b965be6ef25b465dad114c73498d189fa5301ecc095` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/battle/core/wave_runtime.rs` | `23eadbd1192cfe28e2e94f3d555b1ebf283ec6d319e9cec5e2120a654a1bde93` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/battle/core/field_work.rs` | `507b0ced1cf6923238bf48d737545aa7bb2beff317bdf6a0b8660462304a9d66` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/battle/event_log.rs` | `e20c7ff9ab45a4797c615b744dbfc15848267af40b040f04a192ba6362ad2c5a` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/battle/result_stats.rs` | `1b11b716a276ab3bfd0590836f44ec71ffe0f01f800a309f8020d15ff7e8556b` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/battle/scenario.rs` | `1caed730779f276b0e83369d2f3bbd818272c557cf56c7775b37a883561aebc1` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/battle/battlefield/mod.rs` | `54d0466d93ac176ed131c57ccdd4954966bf2224d1a41ee5e0943f9a9ae77144` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/battle/enums.rs` | `cac118b9704c9c461d3e8c6146943d097ffc0e531a7fc6e54bf0b81473857b00` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/battle/damage.rs` | `5bfa34c21ad87ad672a4c249b38641f620358a7f34d57f4aa59df47246b5e46a` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/data/boss_runtime_data.rs` | `05b2fa00aa7aed65becf8dbc20da866b35d2dfe4dca4318f4b8cc8fc1013d4d3` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/managers/uuid_manager.rs` | `b78fca9caefe3ebda1203686e3efc6f6dcb1f02e7b9cb2276280aea9532809a9` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/resources/inventory.rs` | `2b32a864fdae8958a0d57610d5193b688fe1699c4ae4f46a5cc43bbb50371de3` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/resources/selection.rs` | `8c01853213e60a5773234fa557a6e97174220f755c77df2b98dd15e03caee7a3` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/employee.rs` | `8d3940dce94a7e8d8858b39c2c48c22f367d76d3b31d23319e76859cc53eaa54` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/visual_novel.rs` | `bd9c83551d5547a5ca425859fd40db197d3b64fbfcab27683dc5ce5a89ed29c3` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/encounter_compendium.rs` | `4b8d897d8c683b4f7ce87ba0a168d1545b495c784928deb0dfaaa2fd041b1083` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/skill_fragment.rs` | `b4bb2acf2bda08b3ebcb1d7f740fb7b51a12d4cb3fb588b3ae1d1ad50a13a38d` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/map/types.rs` | `fd5d4a2900ccf5ed228d45ced89b9def0f0d60c6443e24e58ba9029fb686fc9f` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/game/map/progression.rs` | `798c04a1fe8dce510e67832884ae5c32b45a1ea49556fe324caf52e055bfe014` | single_close_hash; no atomic initial snapshot |
| `core_enhanced/src/authoring/control.rs` | `39d5aa1869a16c2d62d4e74b64c6cac5fbd39553ecceca36b72f9cba1832d6c9` | unchanged_between_hashes |
