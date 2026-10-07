# DefenseRoute 전술 지휘 Phase 4 — Experiments

Status: active

이 문서는 Phase 4에서 실제 수행한 조사, 변경과 검증만 시간순으로 기록한다.

## 2026-08-06 — Phase Goal 초기화 전 경로·dependency·dirty 확인

### 검증하려던 가설

Phase 3이 `Complete / High`, Master Phase 4가 `ready`, 다른 active Phase가 없고 proposed child path가 기존
Goal과 충돌하지 않는다.

### 수행한 확인

- Master registry와 Phase 3 completion audit 확인
- Phase 4 후보 디렉터리 존재 여부 확인
- Start commit과 current dirty state 확인

### 실제 결과

- `defense_route_tactical_command_phase_04_weapon_tactical_contract` 경로는 존재하지 않았다.
- Start commit은 `f318414fb0156a67e04c5479000c2a2683fc0c09`이다.
- Phase 3은 complete, Phase 4만 ready였고 다른 active Phase는 없었다.
- Phase 0~3 verified output과 user dirty는 `EXPERIMENT_NOTES.md`에 보존 경계로 기록했다.

### 결과에 따른 판단

Phase 4 child를 초기화하고 Master activation 뒤에만 production discovery를 시작한다.

## 2026-08-06 — Authority 조사와 변경 전 baseline

### 검증하려던 가설

Weapon schema, affinity/block consumers, starter grant, weaponless admission과 active-run persistence가 이미
존재하는 seam을 통해 strict하게 전환될 수 있고 Phase 5 runtime effect를 선행할 필요가 없다.

### 수행한 확인

- `EquipmentMetadata`/`WeaponCombatProfile`, live RON loader와 cross validation을 읽었다.
- `BattleUnitDraft`→effective profile→deploy/range/manual path와 fragment/certified-power block consumer를
  읽었다.
- Unified starter definition/loadout grant와 equipment reward pool을 읽었다.
- Employee condition/capability, current no-deployable rule, snapshot와 server error mapping을 읽었다.
- Active-run Core payload와 host envelope, checkpoint embedded inventory metadata를 읽었다.
- Missing-block, starter grant, archetype block과 active-run focused baseline 및 Phase 3 broad baseline을
  확보했다.

### 실제 결과

- Weapon profile은 basic attack과 base block을 소유했지만 affinity는 employee base profile에도 있어 중복
  authority였다. Ground/Platform tactical profile은 없었다.
- Employee draft가 weaponless일 때 base employee combat profile을 fallback으로 사용할 수 있었다.
- Safe-zone snapshot과 fragment compatibility가 battle profile builder에 결합되어 있어 fallback 제거 시
  별도 non-battle projection이 필요했다.
- Starter loadout에는 armor만 있었고 general equipment reward pool에 explicit exclusion 기능이 없었다.
- Active-run checkpoint inventory가 `EquipmentMetadata`를 embed하므로 required weapon schema cutover는 Core
  payload version과 결합됐다.
- 변경 전 focused baseline은 모두 pass, broad baseline은 Core 872/server 55/RON 23이었다.

### 결과에 따른 판단

Per-weapon required fields를 `WeaponCombatProfile`에 추가하고 employee battle draft를 armed-only로 만든다.
Safe-zone projection은 weaponless를 허용하는 별도 helper로 유지한다. Core payload는 V3-only로 cutover하고
Phase 5 stance/effect state는 추가하지 않는다.

## 2026-08-06 — Required weapon profile, live catalog와 starter contract 구현

### 변경한 내용

- `deployment_affinity`, required Option `ground_defense_focus`/`platform_rest`와 strict tile/profile validation을
  `WeaponCombatProfile`에 추가했다. Explicit zero는 허용하고 damage reduction은 100 이하로 제한했다.
- Existing 29 weapons를 migration하고 `standard_training_sword`를 추가해 exact 30개 catalog를 만들었다.
  Staff 16/Bow 1/Gun 1/Sword 10/Spear 1/Shield 1과 승인 tuning을 모두 RON에 명시했다.
- Starter-eligible 6명의 initial loadout에 training sword를 추가하고 starter가 실제 Weapon을 가지는지
  startup에서 검증했다.
- General equipment reward filter에 `exclude_equipment_ids`를 추가하고 training sword를 제외했다. Missing와
  duplicate exclusion ID도 publication 전에 reject한다.
- Weapon apply가 affinity와 base block을 함께 소유하도록 만들고 fragment/certified power는 base block만
  읽도록 명시했다.

### 직접 검증

- Exact live catalog/tuning과 starter/reward exclusion test를 `ron_loading`에 추가했다.
- Missing/unknown tactical key, tile/profile mismatch, explicit zero와 starter weapon missing rejection을
  loader/unit test로 추가했다.
- Current archetype base block과 training sword progression을 direct regression으로 검증했다.

## 2026-08-06 — Weaponless gate, persistence와 server cutover 구현

### 변경한 내용

- Employee battle draft에서 weaponless fallback을 제거하고 `GameError::WeaponRequired`를 반환한다.
- Condition-first `battle_deployment_disabled_reason`과 `is_battle_deployable`을 추가해 deploy/redeploy/range,
  certified membership과 no-deployable rule을 armed eligibility에 연결했다.
- Safe-zone snapshot/fragment compatibility는 weaponless `weapon_profile: None`을 유지하는 별도 helper로
  분리했다. Equipment target과 roster visibility는 그대로다.
- Root reason/code/message와 game_server error mapping을 `weapon_required`로 확장했다. Employee DTO의 중복
  base affinity는 제거하고 effective affinity만 optional로 투영했다.
- Active-run Core payload를 V3로 올리고 header version을 먼저 검사해 V2를 current body parse 전에
  reject했다. Current V3에서 embedded weapon tactical field가 없을 때도 strict reject한다.
- 무기를 지급하지 않는 HQ recruitment를 no-deployable recovery로 잘못 취급하던 기존 경로를 제거했다.

### 실패 원인과 수정

- Required `Option` field는 plain serde Option일 때 missing과 explicit `None`을 구분하지 못했다. 전용
  `deserialize_required_option`을 사용해 missing은 reject하고 authored `None`은 허용했다.
- 기존 core tests 일부는 직원이 항상 weaponless fixture였거나 starter slot이 비어 있다고 가정했다.
  Battle-only fixture는 explicit test source 또는 training sword를 사용하고 장비 tests는 starter weapon을
  먼저 안전하게 unequip하도록 갱신했다.
- Certified-power tests는 armed membership 정책에 맞게 actual owned/equipped weapon fixture를 추가했다.
- HQ recruit definition의 `initial_loadout`은 starter selection처럼 equipment instance를 grant하지 않는다.
  따라서 authored loadout을 보고 recovery 가능하다고 추정하지 않고 HQ recruitment recovery branch를
  제거했다.
- One-cell forward training sword range는 battlefield edge에서 회전 preview가 empty일 수 있다. Preview
  regression은 four-facing typed shape와 mutation-free 성질을 검증하고 off-grid empty를 허용하도록 바로잡았다.
- game_server local fixture starter는 armor-only여서 validation에 실패했다. Core fixture와 같은 training
  sword metadata/loadout을 추가했다.

## 2026-08-06 — Final focused, broad, static와 canonical 감사

### 실행한 정확한 명령

```bash
cargo test -p game_core --lib --quiet
cargo test -p game_core --test ron_loading --quiet
cargo test -p game_core --quiet
cargo test -p game_server --quiet
cargo check -p game_core -p game_server --all-targets
cargo fmt --all -- --check
git diff --check
rg -n "ActiveRunCorePayloadV2|ACTIVE_RUN_CORE_PAYLOAD_VERSION: u32 = 2|strict core payload version 2|weapon_profile.*unwrap_or|unwrap_or.*weapon_profile|infer.*weapon|fallback.*weapon" core_enhanced/src game_server/src game_resources/data core_enhanced/docs/{game_rulebook.md,core_runtime_contract.md,data_loading_contract.md} --glob '!**/goals/**'
rg -n "deployment_affinity" core_enhanced/src/game/behavior.rs core_enhanced/src/game/world/snapshot.rs core_enhanced/src/game/employee.rs
rg -c "deployment_affinity:" game_resources/data/equipments/base.ron
rg -c "ground_defense_focus:" game_resources/data/equipments/base.ron
rg -c "platform_rest:" game_resources/data/equipments/base.ron
```

### 실제 결과와 test 수

- Core library: 832 passed
- Core full: 880 passed total (library 832 + integration 3 + 4 + 24 + 13 + 4)
- Live RON: 24 passed
- game_server: 55 passed
- All-target compile, formatter와 diff whitespace: pass, warning 없음
- Failed/ignored: 0
- 세 tactical field RON count는 각각 exact 30이었다.
- Legacy Core V2/fallback 검색은 production/canonical hit가 없었다.
- Employee DTO에는 optional `effective_deployment_affinity` projection만 남고 base authority field는 없다.

### 최종 판단

모든 evidence row가 pass다. Phase 4는 `Complete / High`이며 Phase 5만 active로 전환할 수 있다.
