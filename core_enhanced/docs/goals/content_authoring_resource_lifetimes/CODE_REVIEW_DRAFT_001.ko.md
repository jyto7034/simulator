# R05 코드 검토 초안 — current publication 전

2026-09-12 · P2G-20 candidate 작업 메모. 정식 Code Review Artifact/Review Batch/Binding이 아니며 `code_reviewed` 판정을 발행하지 않았다. 입력 `WR-CARL-P2G10-001`, source plan `RSP-CARL-P2G20-001` 고정. 승인 의미와 acceptance는 [P2G-10 payload](P2G_10_POLICY_001.json)가 소유한다.

## 실제 현재 경계

### SR01 / E01 — 영구 identity 전달은 재사용 가능

`world`의 roster/loadout → `combat_setup/player_spawns.rs::battle_unit_draft_for_employee_from_item_slot`은 실제 직원 UUID를 `BattleUnitDraft.owned_uuid`에, 각 `EquippedRef.instance_uuid`를 `BattleEquipmentBinding.persistent_copy_uuid`에 넣는다. `core/build.rs`가 transient unit/item ID를 만들 때 이 필드를 `RuntimeItem`과 `EquipmentSourceSnapshot`으로 보존한다. 시나리오에 영구 inventory 사본이 없으면 값은 `None`이며 추정하지 않는다.

`RuntimeFactSource`는 actor/equipment/cast/authored/action/delivery를 분리한다. `EquipmentSourceSnapshot`에는 definition UUID, runtime item ID와 optional persistent copy UUID가 모두 있다. 반면 실제 `ResourceOwner`는 Unit/Side이고 `FactResource`는 Resonance/TacticalPoints만 있다. 현재 출처 기록의 사본 identity를 공통 자원 authority로 사용하려면 별도 typed owner와 lifetime 결속이 필요하다. 기존 provider fact summary는 자원 잔량 저장소가 아니다.

### SR02 / E02 — action/delivery 종료 경계는 있으나 여섯 자원 수명은 없다

`RuntimeFactState`는 private `AttemptToken`과 `ActionToken`을 가진다. `live()`는 token pointer identity 및 active map의 동일 handle을 대조한다. `accept/release/dispatched/terminal`은 control 상태를 관리하고, `delivery/close_delivery/settle_if_ready`는 finite delivery를 따로 센다. control 취소는 pending step을 지우지만 release된 delivery가 남으면 effects-settled를 늦춘다. 감사 당시와 동일한 `runtime_actions_cancelled_control_waits_for_actual_finite_delivery_once`는 이 의미를 직접 assert한다. 이는 R05 새 자원 수명의 시험 성공이 아니다.

투사체는 source snapshot의 delivery ticket을 보존하고 마지막 제거 때 닫는다. area는 실제 만료에서 ticket을 닫는다. `queue::push_completion`은 dispatch 시 종료하는 finite source를 구별한다. `reserve_finite_source`는 `allows_finite_children`과 action handle을 사용한다. 이것을 Run 자원이나 Chain authority로 그대로 부르면 안 된다. `RuntimeFactSource::observation()`은 관찰용 투영이므로 live resource capability를 그 출력에서 복구하면 안 된다.

`withdraw_unit`은 runtime lifecycle·buff·pending cast·active skill runtime을 정리한다. actual deployment epoch의 소유와 재배치 시 새 instance 생성은 추가로 전체 호출 흐름을 확인해야 한다. **Chain root/자식/종료와 Activation 수명에 대한 정확한 공통 결속은 아직 검토 중이다.** 새 Chain lifetime을 단순 action diagnostic ordinal이나 bounded event log로 구현하는 것은 승인 의미를 충족하지 않는다. R16의 재방문/분기 합류 전체를 이 단계에 선구현하는 것도 승인 범위를 넘는다.

### SR03/SR08 / E03/E08 — 작성·컴파일·실행을 함께 전환해야 함

현재 `SkillEffectDef`에는 Damage/Heal/ModifyResonance/ModifyTacticalResource 등 고정 effect가 있고 일반 resource declaration/ref/transaction operation은 없다. raw skill parser는 `RawSkillDef`/`RawSkillStepDef`를 strict serde로 읽어 `SkillDef`/`SkillStepDef`로 변환한다. `AuthoredSkillEffect`의 source는 컴파일마다 재결속하는 metadata이며 영구 index가 아니다.

`sim.rs`는 effect별 BattleCommand를 생성한다. ModifyResonance는 target별 명령을 보내고 ModifyTacticalResource는 별도의 `record_skill_step_live_signals`에서 처리하므로 빈 match arm만 보고 미실행으로 판정하면 안 된다. `commands.rs`의 ModifyResonance는 실제 mutation을 거친 뒤 관찰 witness를 작성한다. 새 공통 resource op는 typed 작성 입력과 실제 owner resolution, 명령/receipt, 실패 원자성을 함께 가져야 한다.

`ability/authoring.rs::EFFECT_SUPPORT`는 현재 Instant Damage/Heal 편집 지원표다. `authoring/query.rs`는 이 표와 `PackManifest::SUPPORTED_CAPABILITIES`를 describe한다. 기존 literal 편집 지원과 새 자원 실행 지원을 구별해서 표시해야 한다. `authoring/view.rs`는 live/terminal runtime facts를 borrowed view로 제공한다. 새 자원 관찰을 추가할 때 제한 없이 복제/encode하거나 mutable state/owner capability를 공개하면 안 된다.

### SR04 / E04 — 기존 원자적 mutation 경계 재사용

`world/application.rs::execute_application`은 route owner 확인 → 현재 lifecycle 검사 → controlled clone → draft operation → lifecycle/projection 및 retained extent 검사 → 최종 poll → 단일 `commit_application`으로 진행한다. operation 실패에서 원래 state/run_seed/game_data/관찰 owner를 교체하지 않는다.

`BattleCore::runtime_transaction`과 `runtime_timestamp_reusing`도 controlled candidate를 거쳐 commit하며 authoring control과 native runtime control을 구별한다. 새 reservation state가 그 candidate 밖의 공유 mutable cell에 있으면 rollback이 깨진다. 예약·revision·capability·queue와 receipt extent를 같은 authoritative clone/candidate 경계 안에 넣어야 한다. 현재 이후 timestamp 실패가 이전 성공 prefix까지 되감는다는 계약은 없다.

### SR05 / E05 — 사본 보존/소멸의 실제 owner

`EquipmentLoadoutTransition::unequip`은 동일 owned item의 `equipped_to`만 비우며 인벤토리에서 제거하지 않는다. 장착/직원 이전은 같은 UUID의 binding 변경이다. `remove_for_dismantle`은 loadout과 인벤토리를 함께 stage해 해당 UUID를 제거한다. `world/maintenance.rs::handle_dismantle_equipment`는 기존 recipe grant를 확인하고 그 전이를 사용한다. `handle_enhance_equipment`는 material 비용을 stage한 뒤 같은 owned equipment를 강화한다. `world/shop.rs::sell_item_quantity`는 검증한 장비 UUID를 staged inventory에서 제거하고 기존 판매 quote를 지급한다.

새 Run 자원을 inventory/roster와 별도 저장한다면 이 모든 정상 소멸 경로에서 같은 transaction으로 정리해야 한다. 단순 equip hook 초기화는 같은 사본을 재장착할 때 잔량을 되돌리는 결함을 만든다. 판매·분해 자원 폐기는 추가 보상/환불과 분리한다. 변경 실패에서는 기존 application candidate가 소유하는 사본과 자원을 모두 보존한다.

### SR06/SR07 / E06/E07 — AT05의 필수 동시 전환

`BattleEntrySnapshot`은 run/inventory/roster/UUID/checkpoint 등을 capture/restore한다. `commit_planned_live_battle_start`는 NodeConfirm 진입 상태를 capture한 뒤 attempt를 시작하고 battle UUID를 발급한다. `handle_retreat_battle`은 active battle의 entry snapshot을 복원한다. 새 resource state가 이 root 집합에서 누락되면 후퇴 의미가 깨진다.

`CombatBattleState`로 변환할 때 runtime facts는 snapshot으로 보존되지만 새 resource value/delta는 없다. `transition_finished_live_battle_to_result`에서 active battle이 제거되므로 이 지점 전에 Run 자원 결과를 소실 없이 전달해야 한다. 실제 `CompleteCombatResult`/staged economy commit과 event battle 재진입이 어디서 한 번만 처리되는지 추가 확인하여 정확한 결과 반영 owner를 닫아야 한다. 관찰 summary를 gameplay delta로 재사용하지 않는다.

`RunCheckpointPayload`는 macro에서 run fields/inventory/roster를 capture/restore하고 `CloneShape`도 열거한다. `active_run.rs`는 V6 envelope와 checksum을 strict deserialize하며 최신 버전만 읽는다. `continue_from_active_run_checkpoint`는 application candidate 안에서 parse/checksum/semantic graph validate → 새 run state 복원을 수행한다. `node_flow.rs`는 실제 content identity와 seed, roster identity, inventory ownership graph를 검증한다. 새 자원에도 owner 존재·definition/contract 일치·값/상한·중복 key 검증이 함께 필요하다. 기존 파일 자동 변환·삭제는 없다.

`ContentContract`는 pack/catalog/authoring/execution/fingerprint 버전을 저장하고 canonical serializer는 실제 typed 입력을 정렬해 의미 fingerprint를 만든다. 새 실행/저장 계약 전환은 공식 pack 생성·소비 및 계약 비교까지 함께 반영해야 한다. 정확한 새 버전 번호/내부 자료형은 구현 설계의 기술 선택이다. 기존 최신-only 정책의 의미를 바꾸지는 않는다.

### SR09 / E09 — 관찰·clone·예산·실행 경로

`CloneShape`는 native clone 전에 소유 그래프를 방문하고 비용을 검사한다. scalar/Vec/Arc/map 형태별 정책이 다르며, 새 store·reservation·lease·snapshot·receipt 필드를 추가할 때 단순 `derive(Clone)`만으로 예산 coverage를 얻지 못한다. 기존 native runtime fact storage accounting은 R04 소유 상태만 가격화한다고 명시하므로 새 상태의 예산을 자동 충족한 것으로 세지 않는다.

`Cargo.toml`의 author_content/package_official_content/consume_content_pack은 default production bins이고 `test-fixtures`는 비기본 기능이다. 실제 외부 resource probe는 현재 ModifyResonance A/B만 시험한다. 이 receipt는 R05 사본 자원·여섯 수명·저장의 완료 근거가 될 수 없다. 새 외부 RON probe는 public API/CLI와 실제 lifecycle·저장 포트를 소비해야 한다.

기존 `retreat_battle_restores_exact_pre_entry_snapshot_and_rng_continuation`, `public_application_checkpoint_port_round_trips_into_a_fresh_application`, 사본 source 인수 및 clone 경계 시험은 확장할 직접 seam이다. 실제 본문을 읽은 범위와 검색만 한 후보를 evidence에서 구별한다. 이번 P2G-20에서 게임 build/test/probe를 새로 실행하지 않았다.

## 검토 중인 구조안 — 아직 채택 완료 아님

1. **공통 resource 정의/owner/lifetime 모듈**과 **실제 Run state/전투 overlay**를 분리한다. immutable catalog에 런 잔량을 쓰지 않고 definition key와 runtime capability를 혼합하지 않는다. 모듈 경로/정수 표현은 기술 선택이나 여섯 lifetime과 owner 의미는 E01/E02 그대로다.
2. **자원 값·예약·revision·lease는 authoritative candidate 소유**로 둔다. reservation은 credit을 잠그며 commit/abort는 한 번만 상태를 변경한다. 수명 종료와 stale 처리도 동일 owner에서 수행하고, observed receipt가 다시 spending authority를 만들지 못하게 한다.
3. **Run state를 entry/result/checkpoint에 함께 결속**한다. persistent equipment UUID와 employee UUID로 binding하고, transient deployment ID는 추가 epoch다. 정상 inventory grant/구매/보상/시작 장비 등 초기화 경로 전체를 확인하여 중복 초기화 없이 동일 정의의 두 사본을 유지한다.
4. **현재 action/delivery/queue 경계에 lifetime hook을 연결**한다. Activation과 Delivery의 종료가 다르며 Chain의 실제 finite causal root와 자식 수명은 따로 닫아야 한다. full Flow/새 join 게임 규칙 없이 R05 일반 state scope를 성립시키는 최소 계약을 검토한다.
5. **작성 schema→validation→compile→consumer→public describe/view→외부 RON 시험을 한 전환에 포함**한다. 다음 R이 없어도 작동하는 증거를 만든다. store-only 공개나 unsupported dummy executor로 대체하지 않는다.

대안 중 모든 상태를 RuntimeUnit 필드에 넣으면 Attempt/Run 보존과 사본 공유가 깨진다. global catalog에 값을 넣으면 런/세대 격리가 깨진다. 공통 store만 먼저 완료하면 AT05와 실제 작성 인수가 빠진다. 따라서 연결된 생산자/소비자/저장/public 검증을 같은 유효 중간 상태로 묶어야 한다. 정확한 Work Package/Atomic Group/Phase 발행은 P2G-40 owner다.

## 아직 닫지 않은 필수 검토

- 여섯 lifetime 중 특히 Chain/Activation의 live authority·실제 생성/종료·취소·배치 철수 후 released source 처리. 이를 R16의 새 정책·기능으로 무단 확대하지 않는 구조.
- 일반 grant/start/shop/reward/admin 등 실제 영구 owner 생성과 source binding의 전체 소비 경계 및 lazy/eager resource 초기화의 관찰 동등성.
- CompleteCombatResult·event battle retry·실패/후퇴·run 종료에서 Run delta를 한 번만 처리하는 정확한 owner와 새 저장 schema 필드/validator/비기본 consumer 영향.
- 새 authored resource의 compiler 검증·target usefulness·기존 효과 소비자 exhaustive match·CloneShape/serialization guards·public/native budget 적용 목록과 직접 시험 대응.
- 요구11개에 대한 exact Source Observation/Review Evidence, 의미 검토/구조 검증 및 전체 current binding coverage. 이 작업 전에 Freeze/Goal/Handoff/GEX를 만들지 않는다.
