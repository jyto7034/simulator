# R03 Monolith native-storage provenance 설계012

범위: AUTORUN §8의 bounded read-only 구현 설계 보조. 완료 감사·구현 완료 판정이 아니다. Root가 제안한 Session 외부 scalar ledger의 가능성과 실제 Monolith 유입·변경·복원만 조사했다. Repository/source/docs/Git 수정, cargo/build/test 실행은 하지 않았다.

Capture: 2026-09-10 09:47:21.847553 UTC; 추가 직접 유입/dependency source capture 09:48:34.290769 UTC; close 09:50:54.750135 UTC. 실제 파일 20개 재해시에서 drift 0. 처음 탐색한 world/run.rs는 존재하지 않아 source 근거로 쓰지 않았다. 전체 SHA-256, 읽은 span, 로컬 dependency 원문 URL은 동반 JSON에 있다. Root의 concurrent BattleCore 변경/398 focused 실행은 이 보고서의 검증 근거가 아니다.

## 판단과 가장 작은 연결안

**제안은 닫힌 opaque application 계보에 한해 source-backed 상한을 만들 수 있다.** 한 scalar credit을 Session에 보존하고 attempted candidate에 값으로 복사하며, 모든 Monolith 독립 table에 그 credit의 bucket envelope를 각각 적용하는 것은 가능하다. 현재 source에는 이 Session ledger나 Monolith hook이 없다. 아래 조건을 충족하기 전에는 arbitrary public Monolith의 native Clone 비용까지 계수했다고 주장할 수 없다.

공개 struct literal/raw HashMap/derived Clone/Serialize/Deserialize를 모두 그대로 유지하는 가장 작은 현재 seam은 다음 두 곳이다.

1. monolith.rs:164–191의 유효한 begin_remanifestation에서 실제 native insert:190 직전에 scalar insertion credit을 1 기록한다. 기존 contains_key/insert 순서, HashMap hasher, node.clone, UUID/RNG를 그대로 둔다. 추가 lookup·재삽입·map 재구축을 하지 않는다.
2. active_run.rs:123–143의 import_v6_checkpoint가 fresh typed parse와 기존 version/hash 검증을 성공한 뒤, 반환 직전에 document.checkpoint.encounter_progression.monolith.remanifestation_nodes.capacity()를 fresh seed로 기록한다. 실제 첫 checkpoint.clone은 :265이다. 이 경계에서 기록하면 공개 Monolith serde annotation도 필요 없다.

두 번째 hook은 현 TrialInitial(DefaultMemory만 있음; scenario.rs:97–109)에 새 checkpoint 입력 기능을 만들라는 제안이 아니다. 기존 public continuation을 authoring 경로에서 사용한다면 현 byte ingress를 그대로 덮는 연결점이다. Session에 arbitrary CoreApplication/GameCore/RunState/Monolith 인수나 mutable borrow를 추가하지 않는다는 PLAN:52–56의 경계가 전제다.

더 넓은 미래 typed deserialize 유입까지 자동 대응하려면 Monolith map field의 deserialize_with helper에서 기존 HashMap::deserialize를 한 번 호출하고 fresh 결과 capacity를 기록하는 대안이 있다. raw field/type/struct literal/Serialize wire shape를 유지할 수 있으나, derive의 in-place 경로·진단 위치와 모든 serde consumer에 영향 범위를 넓힌다. 현재 닫힌 ingress에서는 boundary hook 쪽이 더 작다. 이 보고서는 field helper 구현의 serde 동등성을 검증하지 않았다.

## 실제 소유·변경·유입 근거

- MonolithRunState는 monolith.rs:91–102의 public Clone/Default/Serialize/Deserialize struct다. remanifestation_nodes는 public std HashMap<MapNodeId, MonolithRemanifestationNode>. key는 MapNodeId이고 value는 String 두 개와 u8(:66–72). 다른 fields는 BTreeMap, Vec, scalar/enum이다. map의 native buckets와 live key/value children을 분리 방문해야 한다.
- 실제 생산 map 증가는 begin_remanifestation의 insert:190 한 곳. 정상 victory :213과 failure :266은 remove, floor advance는 drain :281–289이다. 전체 src의 필드/타입 참조 검색에서 추가 production reserve/retain/clear/extend/direct insert를 찾지 못했다. node_flow.rs:1569의 direct insert는 cfg(test) 구간이다. public raw field 자체와 RunState의 public DerefMut(state.rs:1585–1588)는 외부 임의 변경 가능성이다. 일반 공개 struct 전체에는 닫힌 계보 증명이 성립하지 않는다.
- fresh application은 application.rs:782–790 → world.rs:93–103 → GameCoreState::new(state.rs:1807–1829; run None/checkpoint default)로 만들어진다. 새 run은 world.rs:808,828–832의 EncounterProgressionState::default 계보와 state.rs:1460–1478의 fresh caches다. EncounterProgressionState:127–135가 Monolith를 값으로 소유한다.
- node_flow.rs:548–551은 native Monolith clone에 insert한 뒤 :577–582에서 run에 move한다. combat.rs:3063–3069는 progression clone에 victory removal을 적용한다. failure :3017–3020은 current run에서 removal한다. state.rs:1508–1526은 progression clone/move, :1491–1494는 next-floor progression을 옮긴 뒤 drain한다.
- RunCheckpointPayload는 state.rs:384–398/464–476의 derived Clone/Deserialize owner이며 encounter_progression을 포함한다. capture :401–422는 run field를 각각 clone하고 restore :425–459는 move한다. save_run_checkpoint(node_flow.rs:1237–1247), load :1264–1285에서 checkpoint 복제·복원이 생긴다.
- BattleEntrySnapshot은 run과 run_checkpoint를 각각 소유(state.rs:530–546)하며 capture :549–565에서 각각 clone한다. restore :568–583는 move한다. 따라서 live run, saved checkpoint, battle-entry run, battle-entry checkpoint에 별도의 Monolith tables가 동시에 존재할 수 있다.
- pending MasteredEliteSweepCommitPlan은 자체 encounter_progression_after를 소유(mastered_elite_sweep.rs:21–27). :169–172의 pristine state copy와 :276–281의 completion plan도 보존된다. :324–326은 pristine state를 복원하면서 새 plan을 pending으로 남긴다. ledger를 pristine state와 같이 되감는 것은 이 별도 retained plan 계보를 놓칠 수 있다.
- typed byte ingress는 active_run.rs:115–151의 serde_json::from_slice → V6 document이다. :259–276은 import, 검증, encoding, checkpoint clone, restore 순서다. public application.rs:987–1001은 byte ApplicationCheckpoint를 common domain continuation에 넘긴다. 이 검색 범위에서 다른 production typed Monolith/RunCheckpointPayload deserialize ingress를 찾지 못했다. 임의 public serde 호출 또는 외부 struct literal까지 덮었다는 뜻은 아니다.

## 상한의 귀납적 근거와 한계

P를 현재 opaque 계보에서 발생한 insertion과 fresh reservation의 보수적 credit이라고 하자. 기존 MapCloneHistory(owned_collections.rs:14–46)의 pinned 식을 재사용한다.

- E(0) = 0.
- P > 0이면 E(P) = max(16, checked_next_power_of_two(checked(2 × P × 8) / 7)).
- 연산 overflow는 unknown/실패다. 0 또는 현재 len/capacity로 대체하지 않는다.

Rust1.91.1 std HashMap Clone은 base.clone에 위임한다(std hash/map.rs:1301–1314). hashbrown0.15.5 RawTable Clone은 source의 실제 buckets로 새 table을 만든다(raw/mod.rs:3155–3184). 현재 capacity는 items + growth_left(:1277–1283)이므로 deletion 뒤의 raw bucket 상한이 아니다. P 식은 reserve_rehash의 resize request <= 2 × new_items(:2642–2677), capacity_to_buckets의 load-factor/power-of-two(:103–159)에 기초한다. 설치 allocator bytes/RSS, 모든 Rust 버전의 보장은 아니다.

귀납:

- fresh empty map은 P=0에서 시작한다.
- fresh Deserialize는 serde_core1.0.228 de/impls.rs:1536–1546,1565–1571에서 capacity를 요청하고 insert만 한다. 따라서 **그 fresh 반환값**의 capacity C를 seed에 포함하면 size hint/중복 key를 포함한 실제 retained allocation을 덮는다. 단순 final len을 seed로 쓰면 안 된다.
- 실제 삽입 때 global P를 증가시키면 해당 lineage의 insertion credit이 global P 이하로 남는다. 다른 map의 상한도 줄지 않는다.
- remove/drain/drop/move/semantic checkpoint restore는 P를 감소시키지 않는다.
- native clone은 source layout을 복제하므로 새 lineage의 storage도 E(P) 이하이다. clone 자체는 P를 증가시키지 않는다.
- 새 fresh map의 seed를 P에 checked-add하는 root 후보는 충분히 보수적이다. 더 작은 대안으로 P=max(P,C)도 각 lineage에 대한 상한 invariant를 유지한다(그 뒤 모든 insertion은 P+1). 이는 전체 lifetime 합계의 의미 대신 최대 lineage envelope를 사용하는 기술적 차이다. 기존 sum 방식도 안전하며 이를 반드시 바꾸라는 판단은 아니다.

하나의 E(P)를 whole aggregate에 한 번 더하면 **부족하다**. 각 독립 Monolith map마다 E(P)+owner record를 CloneMeter::scan_storage에 먼저 넣고, 해당 live key/value를 방문해야 한다. 같은 P를 N개 map에 적용하여 N배 과계수하는 것은 의도된 상한이다. String 두 개의 실제 복사 work도 별도로 들어간다. native Clone 자체는 계속 그대로다.

한 scalar는 constant retained storage이며 source generation Arc를 붙잡지 않는다. 반대로 이미 사라진 map의 history도 남으므로 실제 작은 table에 보수적으로 큰 clone cost를 부과하거나 더 빨리 한계에 도달할 수 있다. 이 usage는 실제 bucket 수/byte/heap 측정값으로 표시하면 안 된다. 기존 progress/use counters/residuals는 reservation 또는 삭제 이력을 증명하지 못하며 대체 credit으로 삼지 않는다.

## Attempt·rollback·A/B·trace 조건

**MP012-C01 — seed 소유와 설치 시점.** private Copy scalar/provenance 값을 Session aggregate가 소유하고, 후보 시작 때 값으로 복사하여 기존 Attempt에 Option 형태로 넣는 것이 작다. A의 mutable counter나 Arc<Mutex<...>>를 공유하지 않는다. 최초 opaque A fork의 preflight 전에 설치한다. generic validate/query scope의 “provenance 미제공”을 “증명된 zero”와 구별한다. 알려지지 않은 provenance로 active whole Monolith visitor를 호출하면 fail closed해야 한다. ordinary no-scope 경로에서는 기록/visitor가 기존 gameplay 결과를 바꾸지 않는다. ordinary에서 이미 변경된 raw root를 뒤늦게 session에 넣고 P=0으로 시작할 수는 없다.

**MP012-C02 — rollback 수준.** control.rs:155–208의 Attempt는 이미 gameplay clone 밖에 있고 ScopeGuard가 TLS를 제거한다. outer 실패/취소는 candidate와 candidate P만 폐기하며 A의 P·quota·receipt를 보존한다. 내부 execute_application의 draft(:183–199), with_state_transaction의 state restore(node_flow.rs:783–802), checkpoint/BattleEntry 복원에서는 살아 있는 outer P를 되감지 않는다. 예외적으로 내부 rollback에서 credit을 낮추려면 모든 surviving owned tables를 재증명해야 하므로 최소안에 넣지 않는다.

**MP012-C03 — acceptance 순서.** control::run은 T를 그대로 반환하므로 T=Result의 Err도 outer Ok가 될 수 있다(control.rs:205–208; mod.rs:141–148도 별도 domain unwrap). ledger를 꺼냈다는 이유로 commit하면 안 된다. public/domain operation success, mandatory result/receipt admission, sticky failure/cancel final poll까지 통과한 prepared aggregate와 P를 한 번에 infallible swap한다. TLS 반환은 (T, usage, candidate_provenance) 같은 내부 결과로 확장할 수 있으나 Session caller가 T의 실패를 반드시 분리한다. swap 이후 observer/transport 실패를 이유로 app만 유지하고 ledger만 rollback하지 않는다.

**MP012-C04 — 제어의 분리와 재진입.** P는 누적 allocation provenance이고 ExecutionUsage/remaining fuel은 매 attempt의 새 값이다. gameplay clone으로 fuel을 복구하지 않으며 P를 usage에서 역산하지 않는다. scope helper는 잠깐 scalar를 복사/갱신하고 RefCell borrow를 해제한 뒤 native insert/deserialize/visitor를 호출한다. borrow를 유지한 채 serde/callback을 실행하면 기존 ACTIVE 재진입에서 panic할 수 있다. 기록은 기존 first failure를 지우지 않는다. PhaseGuard만 phase를 바꾸며 provenance를 reset하지 않는다. 기존 nested top-level rejection도 유지한다.

**MP012-C05 — 세대/lifetime.** fresh B는 새 app/새 provenance로 준비할 수 있다. B 전체 준비·time-zero·필수 출력 성공 뒤 A aggregate와 ledger를 같이 교체한다. B가 A의 owned run/checkpoint를 일부 이어받는 설계라면 P도 그 계보를 포함해야 하며 무조건 zero reset은 안 된다. failed B는 A P에 영향 0. reload/drop/process end에서 scalar와 candidate owners가 같이 끝나며 전역 cache/allocator registry/address key가 필요 없다.

**MP012-C06 — continuation·trace·저장.** 같은 accepted Session에서 실패 후 재시도는 그대로인 A P를 사용한다. 같은 초기 provenance와 같은 실행 순서의 replay에서는 insertion/seed 횟수도 동일해야 한다. optional trace 및 readonly explain은 이 hook을 호출하거나 P를 commit하지 않게 한다. HashMap 열거·hasher를 바꾸지 않으며 ledger가 native iteration의 다른 특성을 교정한다고 주장하지 않는다. V6에 ledger를 넣지 않는다. V6는 semantic checkpoint를 fresh deserialize하며 이때 새로운 allocation seed를 증명한다; native live-state/history 복원의 보장이 아니다. 기존 canonical checkpoint hash/Serialize 출력(active_run.rs:86–113)도 그대로 유지한다.

## 닫히기 위해 확인할 구체 oracle

후속 구현자가 검증해야 할 항목이며 여기서는 실행하지 않았다.

- insert/remove 반복, drain, native Clone 후에도 P가 줄지 않고 각 별도 table copy에 storage가 합산되는지.
- V6 fresh deserialize의 capacity seed가 checkpoint.clone 전에 들어오는지; malformed/hash-invalid import가 A ledger를 변경하지 않는지.
- 명령에서 insertion 후 늦은 오류/취소/mandatory-output 초과를 강제하고 A의 다음 continuation/ledger/receipt가 보존되는지.
- checkpoint rewind 및 pristine+pending plan 공존 뒤 더 큰 계보의 상한을 유지하는지.
- trace on/off와 readonly/rejected retry가 이후 같은 step의 mandatory admission과 P에 영향 없는지.
- nested scope/early return/unwind 뒤 TLS 누출이 없는지; overflow/미제공 provenance를 zero로 처리하지 않는지; ordinary public struct/serde/clone 의미가 유지되는지.

이 자료는 위 ledger 설계의 조건부 근거다. whole visitor 연결, clone coexistence, parse transient allocation 및 실제 비용 측정을 대신하지 않는다.
