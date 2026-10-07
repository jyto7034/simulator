# R06 WP02 D04-OWNER-F01 교정 구현 지시서

2026-09-27 · 범위: 장비 관계 조회의 전역 탐색 제거와 기존 작업량 계량 보장.
계획·독립 검토는 원 대화, 제품 구현은 별도 대화에서 수행한다.
이 문서는 기존 사용자 정책을 바꾸지 않는 교정 지시다. **D04-OWNER-F01**은 예전에 교정한
무대상 projectile의 **D04-F01**과 다른 finding이다.

## 1. 구현 담당자에게 전달할 요청

> 이 문서에 따라 D04-OWNER-F01을 교정해줘.
> runtime 슬롯에 정확한 runtime item ID를 함께 보관하고, 장비 조건·관계 조회에서
> 전투 전체 items 순회를 제거해줘. 두 spawn 경로의 기존 장비 ID와 생성 순서는 보존하고,
> 소유 유닛·definition·실제 사본 일치 검사와 typed 오류도 유지해줘.
>
> 먼저 동결본 currentness를 확인하고 새 correction 구간에 실제 before baseline과 PLAN을 남겨줘.
> 아래 구현 순서와 C01–C09 회귀 기준을 따라 구현·자체 검증한 뒤 successor target을 동결해줘.
> 기존 target·실패 로그·독립 검토서는 수정하지 마.
>
> 마지막에는 검토 인계서와 manifest SHA를 보고하고 중단해줘.
> 독립 재검토는 원 대화에서 할 거야. D07·WP03·WP04 구현이나 WP02/R06 완료 판정으로 진행하지 마.

## 2. 시작점과 읽을 자료

1. [독립 검토서](../reviews/wp02_d04_owner_relation_001_review_001/REVIEW.md), 특히 finding §2.
2. [재현 시험](../reviews/wp02_d04_owner_relation_001_review_001/REPRO_TEST.rs),
   [재현 결과](../reviews/wp02_d04_owner_relation_001_review_001/REPRO_TEST.log).
3. [owner 관계 구현 인계](../segments/wp02_d04_owner_relation_001/REVIEW_HANDOFF.ko.md),
   [시작 manifest](../segments/wp02_d04_owner_relation_001/target/d04_owner_relation_001/MANIFEST.json).
4. [기본 구현 방향](WP02_D04_OWNER_IMPLEMENTATION_GUIDE_20260927.ko.md),
   [사용자 정책·O01–O15](../../../successors/r06_scope_20260927_001/IMPLEMENTATION_REQUEST.ko.md).
5. [비용 정책 변경](R06_COST_POLICY_UPDATE_20260927.ko.md).
   과거 지시서의 약9MB peak 차단 요구보다 이 정책이 우선한다.

| 대상 | 정확한 값 |
| --- | --- |
| predecessor manifest SHA-256 | `c2e703c2a08e1fc38141d3d070e052d8207cd7aaf9790f650c685429728a2a9f` |
| 확장 source 337개 after digest | `eb9bd8fb3927f8a9f3c5b35908012300e4fad9c4e8d7a03a9e612e44b0dc804a` |
| 제품 source 333개 after digest | `8bde59151f95ad7301c7451f91af7b69c0a5e61c2a0a46239c1da8dd13f2b7b2` |

작성 시 337개 after 레코드를 현재 파일의 크기·SHA와 대조하여 모두 일치함을 확인했다.
[코드 대조 증거](WP02_D04_OWNER_F01_CODE_EVIDENCE_20260927.json)에 읽은 파일의 SHA를 남겼다.
이 문서를 작성하면서 제품 코드를 변경하거나 제품 시험을 재실행하지 않았다.

구현 시작 시 `segments/wp02_d04_owner_relation_001/freeze.py verify`로 currentness와 격리 복원을 재확인한다.
차이가 있으면 실제 차이부터 조사한다. 기존 dirty/untracked를 reset/clean/stash하거나 옛 source로 덮지 않는다.
이번 before는 위 **owner 구현 after**다. predecessor manifest의 더 오래된 before로 되돌려 시작하면 안 된다.
새 작업 디렉터리는 `corrections/wp02_d04_owner_f01_001/`, 새 target 이름은
`d04_owner_f01_correction_001`을 사용하되, 이미 존재하면 먼저 내용을 확인하고 덮어쓰지 않는다.

## 3. 현재 원인과 이번 구현 선택

`value_expr_runtime.rs::resolved_slot_binding()`은 현재 사본을 확인하기 위해
`self.items.values().filter(...).count()`를 수행한다. 같은 슬롯을 조건과 그룹 기준으로 사용하면
수량 산정/실제 view 생성의 두 pass에서 한 대상당 네 번 전체 items를 훑는다.
준비 비용 공식에는 이 전역 탐색량이 없다. 다른 유닛의 장비를 늘린 재현에서
실제 방문12/412/4,012회에 대해 기록 work는 모두149였다.

이번 지시서는 **슬롯 → 정확한 runtime item ID → items.get(id)**를 구현 경로로 선택한다.
장비 사본 UUID를 runtime item ID로 대신 사용하지 않는다. 두 ID는 서로 다르다.
전역 순회를 다른 helper나 첫 일치 항목 검색으로 옮기는 수정은 수용하지 않는다.
관계 결과나 최대 작업량 설정을 바꾸는 작업도 아니다.

## 4. 수정 위치와 순서

경로는 `core_enhanced/` 기준이다. 행 번호는 변할 수 있으므로 아래 symbol을 기준으로 찾는다.

| 순서 | 파일·symbol | 수행할 변경 |
| --- | --- | --- |
| 1 | `src/game/battle/core/types.rs`: `BattleEquipmentSlots` | 슬롯 entry에 binding과 runtime item ID를 함께 저장 |
| 2 | `src/game/battle/core/ids.rs`: `make_item_instance_id` | 아래 설명대로 동일한37바이트를 stack buffer로 구성하여 ID 생성의 임시 heap 할당 방지 |
| 3 | `src/game/battle/core/build.rs`: 두 spawn 함수·`prepare_battle_equipment_slots` | 정확한 equipped_items 순번으로 ID를 만들고 projection과 실제 item 삽입에서 공유 |
| 4 | `src/game/battle/core/value_expr_runtime.rs`: `resolved_slot_binding` | 전체 순회를 직접 조회로 교체하고 identity 검사 보존 |
| 5 | 같은 파일: `prepare_resolved_target_members_with_base` | 두 pass와 모든 node의 직접 조회가 기존 비용 상한 안에 포함됨을 명시·검증 |
| 6 | `core/runtime_facts.rs`, `core/types.rs`, 관련 시험 | projection 크기·Arc clone·scratch·생성 경로 회귀 검증 |

### 4.1 런타임 전용 슬롯 entry

`types.rs`에 다음 모양의 내부 타입을 둔다. 이름은 아래처럼 사용하면 된다.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeEquipmentSlot {
    pub(crate) binding: BattleEquipmentBinding,
    pub(crate) runtime_item_instance_id: Uuid,
}
// BattleEquipmentSlots.slots:
// [Option<RuntimeEquipmentSlot>; 3]
```

binding과 ID를 별도 평행 배열에 저장하지 않는다. 한 슬롯 entry에 함께 보관하면 서로 다른 슬롯의
ID를 붙이는 실수를 줄일 수 있다. `RuntimeUnit.equipment_slots: Option<Arc<BattleEquipmentSlots>>`는 유지한다.
새 ID는 전투 런타임 연결 정보이므로 inventory/RON/`BattleUnitDraft`/`BattleEquipmentBinding`에 추가하지 않는다.
layout 없음(None), 알려진 빈 슬롯(None entry), copy 미제공(binding 내부 None)을 계속 구별한다.

### 4.2 장비 ID 바이트와 생성 순서 보존

현재 `make_item_instance_id`는 `Vec`에 다음을 연결한다.
`equipment UUID(16) + owner unit UUID(16) + side tag(1) + salt big-endian(4)`.
같은 **37바이트**를 `[u8; 37]`에 구성하고 기존 `stable_seed`와
`determinism::uuid_v4_from_seed(seed, Self::ITEM_INSTANCE_NS, 0)` 호출은 유지한다.
이 작은 변경은 projection 준비에서 새 임시 heap allocation이 admission보다 앞서 생기는 것을 피하기 위한 것이다.
UUID namespace·side tag·seed 알고리즘·byte 순서를 바꾸거나 `make_instance_id`까지 재설계하지 않는다.

`build.rs`의 두 호출자는 현재 projection을 준비한 뒤 unit `instance_id`를 계산한다.
순서를 바꿔 순수한 unit ID 계산을 먼저 하고, helper에 `draft, side, instance_id`를 전달한다.
다음 두 경로 모두 변경한다.

- `spawn_scenario_group`: 기존 batch 경로.
- `spawn_scenario_group_member`: Wave/공유 Spawn Kernel의 단일 member 경로.

helper의 known-layout loop를 `draft.equipped_items.iter().enumerate()`로 만든다.
각 binding의 **원 목록 순번**으로 `make_item_instance_id`를 계산하고,
주소로 고른 고정 슬롯 칸에 `RuntimeEquipmentSlot`을 넣는다.
known layout의 유효 항목은 최대3개지만 순번 변환은 checked 방식으로 명시한다.
잘못된 layout/slot·중복 slot·nil/중복 copy 검사는 계속 unit 삽입 전에 수행한다.

**함정 예시:** Weapon이 비고 Armor만 있으면 Armor의 슬롯 칸은1이지만 equipped_items 순번은0이다.
ID salt는0이어야 한다. Any3 주소도 원 목록 순번의 대용으로 사용하지 않는다.

실제 `RuntimeItem`을 넣는 기존 loop의 순서와 provider 등록 순서는 유지한다.
known layout에서는 방금 만든 entry의 ID를 읽어 provider와 RuntimeItem 양쪽에 사용한다.
helper에서 생성한 ID를 실제 삽입 loop에서 다시 생성하지 않는다.
legacy layout=None 경로는 기존 enumerate 순번의 ID 생성 경로를 유지한다.
필요하면 두 spawn에서 공통으로 사용하는 작은 ID 선택 helper를 둔다.
entry 누락/불일치가 있으면 typed 오류이며 임의 ID 생성 fallback은 하지 않는다.

RuntimeUnit 삽입 뒤에도 지역 projection을 사용할 수 있도록 unit에 `equipment_slots.clone()`을 전달해도 된다.
이것은 Arc handle 복사이며 payload 재할당이 아니다. 장비 loop가 끝나면 지역 handle을 자연스럽게 drop한다.
새 전역 map/index·mutable 장비 저장소·런타임 재장착 기능은 만들지 않는다.

### 4.3 직접 조회와 오류 경계

`resolved_slot_binding`은 기존 반환형 `Result<Option<&BattleEquipmentBinding>, ValueExprFault>`를 유지할 수 있다.
slot entry를 고른 뒤 `let binding = &entry.binding`으로 기존 검사를 이어간다.

실제 copy가 있는 기존 검사 분기에서 `.values().filter().count()`를 제거하고
`self.items.get(&entry.runtime_item_instance_id)`를 한 번 수행한다. 다음을 모두 확인한다.

| 검사 | 불일치 때 |
| --- | --- |
| projection의 runtime ID가 유효하며 해당 map key의 item이 존재 | InvalidOwner |
| item 내부 `instance_id`가 projection/runtime map key와 일치 | InvalidOwner |
| `item.owner_unit_instance == unit.instance_id` | InvalidOwner |
| `item.base_uuid == binding.base_uuid` | InvalidOwner |
| `item.persistent_copy_uuid == Some(copy_uuid)` | InvalidOwner |

주소/layout mismatch, nil copy, stale item의 node/site provenance도 기존 fault 경로로 유지한다.
단순히 `.get(id).is_some()`만 확인하면 다른 장비를 가리키는 손상을 놓친다.
정상 빈 슬롯만 predicate false이며, unknown layout은 MissingContext다.
copy 미제공 legacy binding은 기존 의미대로 구별하고 `EquippedSlot` 그룹은 InvalidOwner로 거부한다.
`HasEquippedSlot`만 쓴 경우까지 새 정책으로 확대하여 실제 copy 보유를 강제하지 않는다.

동일 유닛의 중복 copy는 draft→runtime 생성 경계에서 계속 거부한다.
직접 조회로 바꾼 뒤 전투 전체에서 중복을 다시 세는 fallback을 넣지 않는다.
다른 정상 RuntimeItem이 있어도 잘못된 슬롯 ID를 그것으로 복구하거나 교체하지 않는다.

### 4.4 작업량·retained·clone

현재 준비식의 `2 * predicates + 2 * relation_views`는 두 pass의 각 호출 상한이다.
각 호출이 고정 크기 metadata 검사와 **최대 한 번의 직접 조회**를 수행하도록 만든다.
그러면 이 기존 항이 직접 조회를 포함하는 보수적 상한이 될 수 있으므로 숫자를 무조건 늘릴 필요는 없다.
두 pass의 실제 호출 경로와 항의 대응을 주석·시험·인계서에서 설명한다.
비용 단위는 기존 논리 작업 단위이며 CPU 명령 한 개나 나노초를 뜻하지 않는다.

canonical 대상 N명, 전체 spec의 슬롯 predicate 수 P, 장비 관계 수 R이면 item 직접 조회 횟수는
`2 * N * (P + R)` 이하가 되어야 한다. short-circuit·빈 슬롯 등은 더 적게 조회할 수 있다.
기존 일반 predicate/relation 항은 Side/Employee까지 보수적으로 포함하므로 유지해도 된다.
여러 node에 따로 무제한 예산을 주지 않는다. checked 계산·`limits.max_work` 확인·준비 전 charge를 유지한다.
`authoring::control::charge`는 native runtime charge에도 연결되어 있으므로 별도 native charge를 중복 추가하지 않는다.
evaluator의 별도 meter와 취소/오류 전파를 제거하지 않는다.

`BattleEquipmentSlots::retained_extent()`는 `size_of::<Self>() + Arc header` 방식이므로
entry가 커진 크기를 반영한다. 생성 helper의 allocation 전 admission과
`retained_runtime_fact_extent()`의 projection 합산이 이 helper를 계속 사용하는지 확인한다.
RuntimeUnit의 Arc handle 크기는 그대로다. payload 증가분을 handle 비용으로 잘못 대체하지 않는다.
`reuse_timestamp_value`와 `CloneShape`는 기존 Arc 공유 계약을 유지한다.
현재 Arc의 CloneShape는 payload deep copy를 세지 않는다. 무조건 payload를 매 clone마다 다시 할당·계량하지 않는다.

## 5. 회귀 시험 C01–C09

기존 `value_expr_runtime.rs::tests`, `world/tests/combat.rs`와 필요한 인접 test module에 작성한다.
독립 검토 probe를 제품의 `#[path = "/tmp/..."]`로 연결하지 않는다.
표의 여러 case는 parameterized 시험으로 묶어도 되지만 실제 수행한 case를 인계서에서 연결한다.

| ID | fixture와 관측 | 수용 기준 |
| --- | --- | --- |
| C01 | 기존 재현처럼 집계 밖 정상 직원0/100/1,000명 추가, A 한 명의 Weapon 조건+그룹 | 슬롯→item 조회4회, 다른 장비가 늘어도 해당 조회 횟수는 일정. 전역 item 방문은0. 시간 측정만으로 대체하지 않음 |
| C02 | predicate만, 관계만, 둘 다, 여러 대상·여러 aggregate·서로 다른 슬롯 | 실제 조회 횟수가 §4.4 상한 안이며 node별 view/key/결과 격리 유지 |
| C03 | 실제 준비 경로에서 낮은 ExprLimits 및 authoring/native work, 취소 token | 기존 typed 거부·취소 전파. 전부 필터링돼도 원 대상/조건 비용을 면제하지 않음. 허용 결과만 확인하지 않음 |
| C04 | 정상 entry의 ID를 다른 유닛 장비로 바꾸기, map item 제거, 내부 instance_id/owner/base/copy 각각 불일치 | 모두 InvalidOwner, 다른 장비 검색 fallback 없음. unknown/empty/nil/중복 입력의 기존 구별 유지 |
| C05 | ByType 앞 빈 슬롯의 Armor-only, Any3, 같은 definition의 다른 실제 copy | ID의 salt는 equipped_items 순번. 슬롯 ID=RuntimeItem ID=provider ID, copy별 key도 정확히 유지 |
| C06 | 실제 inventory→draft→runtime의 AtBattleStart와 Wave 경로 | 기존 `native_runtime_facts_preserve_actual_same_definition_inventory_copies_through_player_drafts`를 확장하여 두 경로에서 C05의 연결 검증 |
| C07 | 신규 projection의 native retained 경계와 clone/scratch 재사용 | 실제 증가한 extent로 충분한 한도 허용/부족한 한도 거부. entry의 binding·ID가 함께 보존되며 복제 후 조회 가능 |
| C08 | 기존 Snapshot/Live·ordered effect rollback·D04-F01 무대상 projectile 회귀 | Snapshot 고정, Live 재관측, 선평가 후 mutation, 실제 capture identity 및 후속 완료 유지 |
| C09 | item ID 생성의 예전 byte 연결과 새 stack buffer 대조 | Player/Opponent, 다른 owner·definition·salt에서37바이트와 최종 UUID 동일. 변경 전 UUID 체계 보존 |

C01/C02는 `cfg(test)`의 thread-local 관측 hook처럼 병렬 시험에 간섭하지 않는 방법으로
**실제 조회 지점**의 호출을 센다. 예상치만 반환하는 시험 전용 가짜 조회 함수를 만들지 않는다.
현재 계측 대상 helper에서 items 전체 순회가 없어졌다는 코드 대조도 함께 남긴다.
prepare 전체가 전투 크기와 완전히 무관하다고 주장하지 않는다. 기존 retained 계산 등 다른 경로와
이번에 교정하는 **장비 identity 조회**의 관측을 구분한다.

기존 시험에서 `slots.slots[..]`의 binding을 직접 조작하는 위치도 새 wrapper에 맞춘다.
특히 `value_expr_runtime.rs`의 Any3/no-copy fixture와 `world/tests/combat.rs`의 슬롯 순회가 해당한다.
assertion을 삭제하거나 stale fixture를 정상 fixture로 바꿔 통과시키지 않는다.
기존 O01–O15는 이 교정으로 대체되지 않으며 함께 유지한다.

## 6. 자체 검증과 비용 정책

최종 source에서 변경 영역 format과 다음을 실행한다.

```bash
cargo check -p game_core --all-targets --offline --locked
cargo test -p game_core --lib value_expr --offline --locked
cargo test -p game_core --all-targets --offline --locked
cargo clippy -p game_core --all-targets --offline --locked -- -A clippy::never_loop -A clippy::eq_op
```

지정된 allow 이외의 경고 억제를 추가하지 않는다. C01–C09 중 집중 filter에 안 걸리는 시험도
실제 selector나 전체 suite에서 실행됐는지 확인한다. 기준은 선행1,520개라는 숫자 자체가 아니라
최종 discovery의 전체 제품 시험이다. 이번 문서의 source 변경 전 기준은 value_expr83개/전체1,520개다.

구현/원본 시험과 격리 probe는 **서로 다른 CARGO_TARGET_DIR**를 사용한다.
이전 독립 검토에서 공유 target이 probe artifact를 원본 시험에 재사용한 사례가 있었다.
이 환경은 cargo 경로가 `/home/blast/.cargo/bin`, cache가 `/tmp/wp02-cargo-home`였으며,
`/tmp` tmpfs 공간 부족도 있었으므로 빌드 target은 여유 공간이 있는 별도 경로를 사용한다.
예: 제품 `/home/blast/.cache/d04-owner-f01-product-target`, probe는 별도 고유 경로.
환경이 다르면 존재하는 경로를 확인하여 조정하고 실제 env를 기록한다.

projection 크기·실제 ID 생성/조회 경로가 바뀌므로 최종 revision에서 기존 PC36조건과
R05 exact workload의 비용 guard도 확인한다. 기존 segment의 실행 도구는 참고하되,
출력·freeze 경로가 원 segment에 고정되어 있으면 새 correction 경로로 바꾼 복사본을 사용한다.
기존 frozen 증거를 새 실행 결과로 덮어쓰지 않는다.

- PC: 선행 성공31/typed refusal5, gameplay·완료 상태·오류 코드 parity를 유지한다.
  계량 변경 때문에 requested 상세가 달라지는 경우에는 원인과 실제 수치를 구분해 기록한다.
- R05: 원 cap9,030,188 bytes 대비 **64-session concurrent peak 초과만 경고**다.
  선행 peak9,049,432/초과19,244 bytes를 새 cap으로 삼지 않는다.
- raw exit101이면 정확히 해당 peak assertion만 실패했는지 확인한다.
  다른 operations/retained/parity/환경 실패까지 경고로 낮추지 않는다.
- R05 측정은 기존 기능의 비용 회귀 확인이며 R05 기능 개발 재개가 아니다.
  20분 시험, 자동 스킬 취소, 안전 설정 RON 분리, 실험실·편집기 작업을 추가하지 않는다.

각 실행의 command/cwd/env/toolchain·시험 수·stdout/stderr·exit와 실제 source 전후 SHA를 남긴다.
실패 시도와 환경 오류도 보존한다. 격리 계측은 built source 및 추가 hook을 별도로 결속한다.
기존 재현의 `1 passed`는 결함 관측 성공이므로 교정 성공으로 재사용하지 않는다.

## 7. 동결·인계·종료

새 correction PLAN과 before baseline을 편집 전에 작성한다. 기존337개를 출발 집합으로 삼고,
새 실제 소비자/파일이 추가되면 최초 편집 전에 보존하여 manifest 범위를 확장한다.
최종 target에는 다음을 포함한다.

- predecessor manifest·독립 검토 finding·본 지시서와 C01–C09/O01–O15 시험 매핑.
- 실제 before/after bytes·archive·SHA·추가/변경/삭제 목록과 양방향 복원 가능한 delta.
- 최종 source의 자체 검증·PC/R05 원자료와 정책 판정, 모든 실패 이력.
- 읽기 전용 currentness와 **격리 디렉터리에서의** before→after/after→before 검증.
- `REVIEW_HANDOFF.ko.md`: 전역 스캔 제거 위치, ID 일치 보장, 작업량 공식 근거,
  실제 retained 변화, 성공/거부 시험, 잔여 D07·WP03/04 및 publication 차단 상태.

원 target·review·baseline receipt를 수정하거나 runtime item identity 검사를 완화하지 않는다.
전체 expression의 production publication 차단은 유지한다.
자체 검증을 독립 PASS, D04/WP02/R06 전체 완료로 표시하지 않는다.

최종 응답은 **교정 구현·자체 검증 종료 / successor target 동결 / 독립 재검토 요청 대기**로 마친다.
검토 인계서 경로, 새 manifest SHA, 실제 시험 결과와 비용 경고를 함께 전달하고 중단한다.
