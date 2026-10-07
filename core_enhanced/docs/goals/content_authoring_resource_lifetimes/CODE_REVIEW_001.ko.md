# R05 코드 기반 검토

2026-09-12 · 입력 `WR-CARL-P2G10-001` · plan `RSP-CARL-P2G20-001`. 정책 statement/acceptance는 [P2G-10](P2G_10_POLICY_001.json)의 exact 값이다. 이 문서는 구현 전 검토이며 새 게임 시험을 실행하지 않았다. 정식 current 여부는 complete [P2G-20 result](P2G_20_RESULT_001.json)가 소유한다. 조사 과정의 [초안](CODE_REVIEW_DRAFT_001.ko.md)은 역사적 non-current 메모로 보존한다.

## C01 — 실제 owner

**현재:** `combat_setup/player_spawns.rs::battle_unit_draft_for_employee_from_item_slot`은 직원 UUID와 각 장비의 `instance_uuid`를 draft에 전달한다. `core/build.rs`는 `RuntimeUnit.source_owned_uuid`와 `RuntimeItem.persistent_copy_uuid`에 이를 보존한다. `EquipmentSourceSnapshot`은 정의 UUID, runtime item ID, optional persistent copy UUID를 분리한다. 공명/전술 포인트의 `ResourceOwner::Unit/Side`는 이 영구 사본 저장소가 아니다. `commands.rs::TriggerAbility`는 실제 `equipment_source`를 시전 출처에 전달한다.

**차이·구현:** immutable resource definition key, typed actual owner, lifetime handle을 별도 자료형으로 둔다. 장비 owner는 실제 사본 UUID, 직원 owner는 실제 직원 UUID로 해석한다. provider/executor/beneficiary를 대입해 owner를 추정하지 않는다. 선언한 owner가 없는 standalone scenario나 잘못된 binding은 명시적 거부다. transient owner를 요구하는 선언은 실제 runtime owner에만 결속한다. 다른 정의의 두 사본뿐 아니라 같은 정의의 두 사본, 같은 직원의 재배치가 직접 인수 seam이다.

## C02 — 수명과 실제 causal 작업

**현재:** `RuntimeFactState`는 attempt/action의 private token identity를 검사한다. control terminal과 finite delivery settle은 분리돼 있다. `sim::process_event`는 queue source를 설치하고 성공한 completion의 delivery를 닫는다. `skill_runtime/projectile.rs`의 실제 제거, `area.rs`의 만료도 ticket을 닫는다. `build::withdraw_unit`은 철수·buff·pending/active cast를 정리한다. 재배치는 새로운 group/instance salt로 runtime unit을 만든다. 그러나 공통 여섯 자원 수명은 없고, `TriggerAbility` 및 새 cast source의 `..Default`는 현재 causal resource context를 전달하지 않는다.

**구조 제안 TD02:** 자원 store가 별도의 private lifetime registry를 소유한다. 수명마다 generation/token, live status, bound entries/reservations 및 유한한 child ownership을 관리한다. 복제 candidate 밖의 mutable authority는 두지 않는다. 로그 seq나 pointer의 직렬화 값에서 authority를 복구하지 않는다.

| 수명 | 실제 생성·소비·종료 연결 |
| --- | --- |
| Run | 새 Run의 state root; 전투 overlay와 결과 commit; checkpoint/entry restore는 승인된 동일 timeline을 복원; run 종료에서 종료 |
| Attempt | 실제 battle session 생성부터 결과/후퇴로 session을 버리는 경계; 철수·재배치만으로 종료하지 않음 |
| Deployment | 실제 unit deployment 생성/epoch부터 철수·사망·해당 배치 종료; 재배치는 새 epoch |
| Activation | 실제 accepted action/cast의 control instance; 정상 마지막 control step 또는 취소 때 종료; released delivery의 종료와 별개 |
| Delivery | 실제 finite payload/queued completion/투사체/area가 생성될 때 결속; dispatch 완료·소멸·만료 때 자기 handle 종료 |
| Chain | 실제 root occurrence의 causal context; trigger/시전/queue/지연 child에 전달; root control과 이미 접수된 finite descendants가 모두 끝나면 종료 |

Chain은 기존 승인된 root/causal 경계의 자원 lifetime realization이다. 새로운 유물 재방문 규칙, fork/join 결과, full Flow를 추가하지 않는다. independent root는 별도 context, child는 기존 root를 유지한다. 진짜 root와 child의 결속을 source replacement와 queue admission에서 보존하고, 취소는 미생성 child를 생성하지 않으며 이미 방출한 child는 자기 수명까지 유지한다. refcount는 runtime registry의 값이며 `Arc::strong_count`나 retained log 수를 authority로 쓰지 않는다. 예약·등록 실패는 candidate 전체에서 취소되므로 ticket만 남지 않는다. 재귀 queue가 유한 budget을 초과하면 기존 실패 경계로 돌아간다. 단계별/순환 causal grammar의 추가 gameplay는 R16 범위다.

**인수:** 각 여섯 수명별 실제 RON 소비·생성·정상 종료·취소·stale timer, deployment 철수 후 attempt 값 유지, control 취소 후 released delivery의 유효 자원과 종료된 activation 자원 분리, 같은 ordinal의 새 attempt를 old token이 변경하지 못함, independent roots의 격리. 기존 action-cancel/delivery150→300 시험은 회귀 seam이며 새 상태 시험을 대신하지 않는다.

## C03 — 선언·수치·작성 입력

**현재:** strict `RawSkillDef/RawSkillStepDef`에서 authored effect로 변환하고 source metadata를 다시 결속한다. `SkillEffectDef`는 고정 효과들만 가진다. `skill_data.rs`의 validation과 `target_usefulness.rs`의 effect match, `sim.rs`의 command 생성이 실제 소비자다. 전술 포인트는 target loop의 빈 arm과 별도로 `record_skill_step_live_signals`에서 한 번 실행된다.

**구조 제안 TD03:** resource 선언을 일반 catalog의 immutable 정의로 두고 owner kind/lifetime/explicit initial/finite cap을 검사한다. 효과는 typed resource reference와 grant/decay/consume/reservation 연산을 전달한다. field·enum 이름, 정수 폭과 내부 index는 GEX-10 상세 선택이나 선언→해석→실행→descriptor는 같은 전환이다. 비존재 정의, owner-kind 불일치, initial>cap, 잘못된 값, 부적합한 lifetime context를 source 위치가 있는 오류로 거부한다. 수치 의미는 E03 그대로다. grant는 cap까지, decay는 잔량까지, consume 부족은 전체 거부다. 기존 resonance/tactical 수치 정책을 이 과정에서 바꾸지 않는다.

owner-only 자원 효과를 target 수만큼 중복 실행하면 안 된다. target-dependent selector는 실제 각 target을 해석하고 하나의 atomic operation 범위 안에서 처리한다. 자원 연산 때문에 기존 hostile-damage 판정을 임의로 참으로 만들지 않는다. source-only 연산이 유효한 self/nonhostile 작성 경로와 거부 경로를 함께 시험한다. 내용 이름별 executor나 literal Damage/Heal 편집 지원표를 자원 실행 지원으로 위장하지 않는다.

## C04 — 예약·실패 원자성

**현재:** `world/application.rs::execute_application`은 validation, controlled candidate, operation, retained extent/poll, sole commit을 거친다. `BattleCore::runtime_transaction`과 timestamp candidate도 해당 commit 전 실패를 원본에 게시하지 않는다. `queue::push_queued`는 void라도 budget 오류를 deferred control에 남기므로 push 호출만으로 성공을 판단할 수 없다.

**구조 제안 TD04:** 값, reserved amount, revision, reservation record, owner/lifetime handle, pending timer와 receipt를 같은 authoritative candidate에 넣는다. reserve는 available을 잠그고 commit은 해당 예약 한 번만 지출하며 abort는 해당 잠금을 한 번만 해제한다. stale/종료/중복 요청은 새 값·환불·ID 발급을 만들지 않는다. 값 변경과 후속 사건의 admission 및 관찰 비용 검사가 끝나야 commit한다. 거부를 성공 receipt로 기록하지 않는다. 이전 timestamp의 성공 prefix는 보존한다. 기존 단일 candidate를 재사용하되 공유 mutable `Arc<Mutex<...>>`를 추가하지 않는다.

**인수:** 부족·잘못된 owner·stale lease·중복 commit/abort·queue/clone/output 비용 초과마다 값/예약/RNG/UUID/revision/queue/관찰의 전후 일치, 성공 시 정확한 before/after와 source owner. arithmetic 경계, 두 경쟁 예약과 released child도 포함한다.

## C05 — 영구 사본의 생성과 소멸

**현재:** `GrantExecutor::grant_inventory_item`은 실제 owned-equipment UUID를 발급해 `Inventory::add_item_owned`로 넣는다. `EconomyGrantState`는 reward/shop/event/admin/combat grant의 inventory/roster/UUID 등 공유 roots를 stage/commit한다. starter helper는 이 grant 결과 UUID로 실제 loadout을 구성한다. headquarters 모집은 새 직원 UUID로 roster를 stage한다. unequip/transfer는 같은 사본을 보존하고 enhance는 같은 item UUID를 강화한다. sale/dismantle은 실제 inventory 사본을 제거한다.

**구조 제안 TD05:** Run store는 현재 inventory/roster의 actual owner graph와 함께 검증한다. 모든 정상 생성·판매·분해·시작·모집·보상·관리자 경로가 통과하는 application commit 경계에 owner reconciliation을 둔다. binding 또는 조회는 초기값의 논리적 존재를 보존하며 값이 한 번 생긴 뒤 재장착·이전에 초기화하지 않는다. sparse materialization을 쓰더라도 미기록 값은 선언 initial과 정확히 같아야 하며 authored 연산 외 시간 경과에 의한 임의 decay는 없다. 초기값 읽기·save/load·첫 사용이 eager 표현과 동일해야 한다. 중간 grant가 최종 roster보다 먼저 실행되는 starter 경로에서는 일시 owner 부재로 자원을 삭제하지 않는다. 최종 graph에 대해 한 번 reconcile한다.

실제 삭제는 같은 transaction에서 그 사본의 상태/예약을 폐기하고 추가 환불·상속을 만들지 않는다. 직원이 roster에 남는 한 같은 owner다. 초기화의 자료구조 최적화는 GEX-10 재량이며 이 관찰 동등성을 깨뜨리는 최적화는 허용하지 않는다.

## C06 — Run timeline과 결과 한 번 반영

**현재:** `commit_planned_live_battle_start`가 attempt 시작 전 entry snapshot을 capture한다. event의 `StartCombat`/started-combat 재진입도 이 경계를 사용한다. `transition_finished_live_battle_to_result`는 `CombatBattleState`로 옮긴 뒤 active battle을 없앤다. `CompleteCombatResult`는 CombatResult phase에서만 enabled다. defeat/retry/승리 경로는 staged economy/employee 상태를 commit하고 node/phase를 이동한다. retry는 active content를 비운다. `with_state_transaction`은 실패 시 run-local 상태/관찰을 복원하되 profile의 승인된 단조 지식은 보존한다.

**구조 제안 TD06:** Run root의 자원 store에서 battle overlay를 만들고 terminal result에 resource result payload를 capture한 후 active core를 버린다. payload는 actual owner와 battle/entry basis revision을 결속한다. staged combat result가 resource 상태도 소유하고, normal/event/defeat/retry 모두 같은 완료 transaction에서 한 번 반영한다. 반복 CompleteCombatResult는 기존 phase/소비된 active content와 result correlation으로 거부하며, 새 무제한 processed-ID 집합에 의존하지 않는다. application 실패면 result pending과 원래 자원이 보존된다. 후퇴는 pre-entry 자원/RNG를 복원하고 재시도는 승인된 committed Run 상태에서 새 attempt를 만든다. run 종료는 final result를 정상 반영한 timeline을 닫는다.

**인수:** 두 battle 결과에서 연속 Run 값, 동일 결과 재접수 무변경, 실패 후 재접수 한 번, event 재진입, 철수·재배치·후퇴·checkpoint의 값 차이, profile/load peak 예외 유지. `world/tests/combat.rs` retreat exact snapshot/RNG 시험과 public checkpoint fresh-application 시험에 실제 새 값 assert를 추가한다.

`RunState::for_next_floor_from/for_run_complete_from`은 새 RunState를 만들고 `with_persistent_run_state_from`으로 일부 필드만 전달한다. 새 store를 여기에 누락하면 층 이동에서 잔량이 초기화된다. 같은 Run의 층 이동은 store를 보존하고 terminal 전환은 최종 관찰값을 보존하되 spending capability를 닫아야 한다. 새 Run은 이전 capability/잔량을 상속하지 않는다.

## C07 — AT05 저장·최신 계약

**현재:** `RunCheckpointPayload` macro와 `BattleEntrySnapshot`이 root capture/restore 및 CloneShape를 명시한다. V6 envelope는 strict schema/checksum/latest-only다. `node_flow`는 content identity/seed와 inventory/roster ownership graph를 검증한다. `ContentContract`와 canonical semantic bytes가 loader의 실제 generation identity가 된다.

**구조 제안 TD07:** 새 Run root, entry snapshot, result payload, checkpoint writer/reader/graph validator, clone/cost accounting을 한 AT05 cutover로 갱신한다. durable key는 resource definition ID와 actual owner UUID이며 runtime pointer/compiled index가 아니다. duplicate/unknown owner/key, cap 위반, stale contract, 잘못된 checksum은 candidate 복원 전에 거부한다. 새 최신 버전과 capability를 공식 pack 생성·소비자까지 일치시킨다. 기존 저장파일의 자동 변환·삭제·default-zero 복구는 없다. owner-only 정적 관찰에는 mutable lease가 노출되지 않는다. 서버/Unity wire 변경과 배포 호환성 인증은 O01에 의해 비적용이다.

## C08 — 실제 public authoring 경로

**현재:** `content_pack::load`는 selected 외부 pack snapshot→catalog parse→resolver→semantic hash→raw assemble/into_validated→GameDataBase→generated combat contract 검사 후 generation을 게시한다. `CatalogKind::Skills`는 strict raw parser를 사용한다. `GameDataBase` 조립과 fixture path도 명시된 catalog를 열거한다. `authoring/query`는 지원을 describe하고 view는 borrowed 관찰을 제공한다. 현재 외부 resource probe는 resonance A/B만 검증한다.

**구조 제안 TD08:** 새 catalog/opcode를 raw definition/resolver/assemble/live validation/database/semantic identity/packaging/describe/view와 실제 runtime에 연결한다. canonical typed serializer를 재사용하되 새 정의를 semantic input에서 빼지 않는다. default production CLI/API에서 외부 RON을 받아 같은 code path로 실행한다. fixture builder는 편의 수단이며 public 인수 대체물이 아니다. resource value/receipt의 bounded borrowed projection과 descriptor에 owner/lifetime/operation/error 지원을 사실대로 보여준다.

**인수:** 독립 external pack의 authored initial/cap/연산을 바꾸면 같은 public 실행의 실제 잔량·receipt가 바뀌어야 한다. 여섯 수명과 영구 사본을 포함한다. malformed/unknown ref/unsupported context/owner mismatch/cap/contract negative pack은 명확한 진단과 원본 무변경. official pack consume/build와 literal editing 회귀도 직접 소비 범위다.

## C09 — 비용·회귀·감사

**현재:** `controlled_clone`은 clone 전에 native/authoring owner graph 비용을 계산한다. 기존 R04 retained extent는 R04 상태만 가격화한다. queue admission과 fact source storage도 유한 예산을 갖는다. Cargo production bins와 test-fixtures가 구별된다.

**구조 제안 TD09:** 새 entries/reservations/lifetime registry/causal tickets/delayed queue/results/receipts의 저장, clone, traversal, 출력 예산을 도입과 동시에 추가한다. 종료 때 실제 owned graph를 해제한다. BTreeMap 또는 canonical iteration으로 동일 input/seed의 ordering을 유지한다. stale work도 유한 처리비용이며 무한 tombstone/log replay를 만들지 않는다. native/public candidate, checkpoint, observation 및 retained source를 모두 계측한다.

**검증 인계:** GEX에서 baseline source를 다시 확인하고 focused store/lifecycle/application/authoring 시험, 외부 RON production probe, affected regression, 필수 통합/build를 실행한다. 각 count>0 및 실제 command/output/source hash를 보존한다. budget failure 직전/직후와 deterministic 재실행, 새 저장 owner graph의 clone 실패, 로그 off/eviction에서도 gameplay 동일성을 본다. 안정된 exact target을 구현에 참여하지 않은 reviewer가 fresh 감사해야 한다. 기존 역사적 pass나 이번 정적 읽기는 새 시험 pass가 아니다.

## 구조 결론·대안·상세 설계 경계

공통 resource catalog/store/transaction/lifetime registry와 world Run projection, battle execution adapters, public authoring adapters를 나눈다. authority는 world/battle의 기존 candidate가 소유한다. 이 공통 계약과 모든 실제 소비·저장·관찰을 하나의 유효 전환으로 묶는다. 커널만 먼저 공개하거나 저장을 후속 R20에 미루는 중간 상태는 유효하지 않다. P2G-40은 이 결속을 보존한 atomic group 또는 증명된 stable seam으로만 분해한다.

RuntimeUnit만의 저장은 Run/Attempt/장비 사본 보존을 깨뜨리고, catalog mutable 전역 저장은 Run 격리를 깨뜨린다. 로그 기반 Chain은 retention에 gameplay가 의존한다. 공통 registry 방식은 clone/retention 계측 비용이 추가되지만 owner와 종료 authority를 명시할 수 있다.

TD01–TD09의 primary classification은 모두 `technical_decision`, disposition은 승인된 의미를 보존하는 위 구조로 GEX-10 상세 설계 인계다. 추가 material policy 없음. 파일명·내부 enum·map layout·budget 항목 수치·테스트 fixture 값·버전 번호는 상세 선택이며, 최종 owner/lifetime/원자성/저장 경계와 실패 의미를 바꿀 재량은 아니다. 추가 근거가 정책·Phase 경계를 바꾸면 해당 P2G owner로 반환한다. R07 fragment-copy·R16 full Chain/Flow gameplay는 `out_of_scope`이며 R05의 실제 Chain 자원 수명은 제외하지 않는다.

required source는 workspace에서 확인했다. 새 구현의 runtime 성공은 아직 unknown이나 이는 GEX 검증 의무이며 current code delta 검토의 evidence gap이 아니다. 배포본·외부 앱은 claim하지 않는다. 기존 dirty는 사용자 작업으로 보존하고 이번 P2G는 production 변경을 하지 않았다.
