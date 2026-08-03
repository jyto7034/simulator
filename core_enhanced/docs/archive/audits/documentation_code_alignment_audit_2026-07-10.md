# 문서–코드 정합성 감사 보고서 — 2026-07-10

> 상태: Historical audit input / 후속 master에서 재검증·처리됨
>
> 감사일: 2026-07-10 (KST)
>
> 후속 처리일: 2026-07-14 (KST)
>
> 주의: 이 파일은 2026-07-10 당시 발견 사항과 근거를 보존하는 정식 감사 이력이며 현재
> gameplay, runtime, data, wire 정책의 source of truth가 아니다. 현재 정책은
> [문서 인덱스](../../README.md)가 지정한 durable canonical과 외부 Unity canonical을 따른다.
> 아래 `1. 결론`부터 `10. 문서 승격 전 체크리스트`까지는 당시 판정과 검증 결과를 보존한
> historical snapshot이므로 현재 상태처럼 재해석하지 않는다.

## 후속 처리 상태 — 2026-07-14

이 보고서의 52개 DCA finding은
[Cross-Layer Contract Hardening Master](../../goals/cross_layer_contract_hardening_master/PLAN.md)에서
source-prefixed ID로 재검증되었다. 현재 disposition과 최종 검증 결과는
[Master Status](../../goals/cross_layer_contract_hardening_master/STATUS.md)가 요약한다.

| Disposition | Finding 수 | 의미 |
| --- | ---: | --- |
| Complete / active scope | 48 | 구현, durable 문서, 외부 계약과 검증 evidence에 반영됨 |
| Deferred by user | 3 | 콘텐츠 설계·밸런스 확정 전에는 구현하거나 값을 임의로 만들지 않음 |
| Excluded / not actionable | 1 | 재검증 결과 실제 계약 모순이 아니어서 변경 대상에서 제외됨 |

현재 정책의 canonical ownership은 다음과 같다.

| 영역 | 현재 canonical |
| --- | --- |
| 플레이 규칙, 노드, 보상, 성장, 실패 | [game_rulebook.md](../../game_rulebook.md) |
| 스킬·평타 타겟팅과 범위 | [skill_target_contract.md](../../skill_target_contract.md) |
| 결정론, runtime state, event/checkpoint/result 경계 | [core_runtime_contract.md](../../core_runtime_contract.md) |
| live RON loader, strictness, validation | [data_loading_contract.md](../../data_loading_contract.md) |
| 문서·goal·review·probe 작업 절차 | [engineering_workflow.md](../../engineering_workflow.md), [probe_registry.md](../../probe_registry.md) |
| Unity root command/snapshot/DTO | `/mnt/f/unity projects/ark/docs/unity_core_contract.md` |
| Unity battle setup/update/revision/resync | `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md` |
| Finding별 disposition과 후속 trigger | [Cross-Layer Master PLAN](../../goals/cross_layer_contract_hardening_master/PLAN.md) |

### 사용자 보류 finding

- `DCA-M-08`: 무기 block modifier 정책과 Equipment schema는 Equipment/Maintenance 콘텐츠
  범위와 밸런스가 확정될 때까지 보류한다.
- `DCA-M-21`: 현재 placeholder 환상체 기본 공격 범위는 환상체별 combat design과 승인된
  range 값이 준비될 때까지 임의 일괄 authoring하지 않는다.
- `DCA-M-28`: generic Maintenance 설명과 실제 live recipe coverage의 정리는 같은
  Equipment/Maintenance 콘텐츠 결정과 함께 재개한다.

재개 조건은 Master의 `G05`와 `G22` disposition이 소유한다. 이 세 항목은 현재 정책이
확정됐다는 뜻이 아니며, 이 보고서의 과거 권고만으로 구현하거나 밸런스 값을 정하지 않는다.

## 1. 결론

주요 문서와 코드는 **완전히 동일한 계약을 말하고 있지 않다**.

Core의 일반적인 전투·데이터 로딩 경로와 현재 live RON은 대체로 정상이며, 이번 감사에서 실행한 Core 테스트도 모두 통과했다. 그러나 후속 감사에서 progression과 Core-server-Unity 통합 표면까지 넓혀 보니 다음 범주에서 명확한 불일치가 확인되었다.

- 전투 중 Visual Novel 상태 갱신을 Unity가 버리는 지원 계약 경로 결함 1건(현재 live data에는 battle VN trigger 없음)
- 문서가 허용하지만 런타임이 효과 없이 종료하는 스킬 조합 1건
- 타겟 선택, 적군 range preview, HUD 렌더링 방식처럼 canonical 문서끼리 또는 문서와 코드가 서로 다른 계약
- Event node의 오래된 DTO 문서와 미구현 Unity 흐름
- 데이터 strictness/중복 검증 보장을 실제 코드보다 강하게 서술한 문서
- Visual Novel trigger 추가 이후 갱신되지 않아 실패하는 game_server 테스트 fixture
- Unity가 잘못된 소비 아이템 command field를 보내고, Core가 보내는 보상/채용 identity를 다른 field로 읽는 live integration 결함
- live battle의 `allowed_actions`를 Unity가 정적 whitelist로 대체하는 권한 source-of-truth 분리와 blocking VN reconnect 실패 경로
- canonical에 있는 checkpoint load, roster 이동, setup-loss recovery command의 Unity 구현 누락
- forced BossOmen이 Gate 진행 규칙을 우회하고, rollback 선택을 제공하지 못하는 progression 충돌
- 소비 아이템 duration의 “node resolved” 의미가 Core 룰북, Unity canonical 내부 문구, 현재 코드 사이에서 갈리는 정책 충돌
- live content index, 예시, 보조 서버 문서의 추가 drift

따라서 현재 상태는 “핵심 기반은 대부분 정렬되어 있으나, 배포 계약으로 사용하기 전에 P0/P1 불일치를 해소해야 함”으로 판단한다.

## 2. 감사 범위와 판정 기준

### 2.1 주요 문서

- `docs/README.md`
- `docs/game_rulebook.md`
- `docs/skill_target_contract.md`
- `docs/core_runtime_contract.md`
- `docs/data_loading_contract.md`
- `docs/engineering_workflow.md`
- `docs/goal_completion_review_guide.md`
- `docs/audit/live_skill_catalog_manifest.ron`
- `docs/skills/live_content_index.md`
- `docs/skills/lobotomy_content_catalog.md`

### 2.2 구현과 live data

- `src/game/**`
- `tests/**`
- `../game_resources/data/**`
- `../game_server/src/**`
- `../game_server/docs/**`

### 2.3 외부 Unity canonical 및 구현

- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
- `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`
- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/**`
- `/mnt/f/unity projects/ark/Assets/Tests/**`

### 2.4 판정 용어

- **확정 불일치**: 문서와 현재 코드가 서로 다른 동작이나 shape를 명시한다.
- **확정 구현 결함**: 현재 live/integration 흐름에서 잘못된 상태 또는 진행 불능이 코드상 성립한다.
- **잠재 리스크**: 일반 계약상 문제가 성립할 수 있으나 현재 live data에서는 직접 재현되지 않았거나 별도 재현 테스트가 필요하다.
- **문서 drift**: 런타임 동작은 분명하지만 예시, 목록, 보조 문서가 현재 상태를 반영하지 못한다.

## 3. 우선순위 요약

| ID | 심각도 | 상태 | 요약 |
|---|---|---|---|
| C-01 | Critical | 확정 계약 경로 결함 | 같은 battle seq/time으로 전달되는 VN 전용 update를 Unity가 버린다. 현재 live data에는 battle VN trigger가 없다. |
| H-01 | High | 확정 불일치 | 문서는 `WholeFieldValidTiles + TileArea`를 허용하지만 런타임은 tile pattern이 없으면 효과 없이 step을 종료한다. |
| H-02 | High | 확정 불일치 | 문서는 skill targeting이 `targeting_profile` 및 무기 profile 상속을 따른다고 하지만 런타임은 `UnitTargetRule`을 사용한다. |
| H-03 | High | 확정 불일치 | Core 문서는 적군 range preview를 기본 DTO에서 제외한다고 하지만 코드는 모든 active unit에 preview를 넣는다. |
| H-04 | High | 확정 불일치 | 데이터 문서의 “기본 deny unknown / 과거 alias 거부” 보장이 실제 serde schema보다 강하다. |
| H-05 | High | 확정 구현 누락 | canonical이 요구하는 Event node route와 command가 ArkClient에 없다. |
| H-06 | High | 확정 불일치 | Unity Event DTO 문서가 단일 `script_id`/speaker/portrait를 요구하지만 Core는 `script_ids` 배열을 보낸다. |
| H-07 | High | 확정 문서 충돌 | Core HUD 문서는 world-space billboard를, Unity 문서와 코드는 screen-space overlay를 요구한다. |
| H-08 | High | 검증 gate 실패 | game_server 전체 테스트 중 6개가 새 VN trigger 필수 조건을 반영하지 못한 fixture 때문에 실패한다. |
| H-09 | High | 확정 구현 결함 | Unity `use_consumable_item`이 Core 필수 field와 다른 key를 보내 역직렬화에 실패한다. |
| H-10 | High | 확정 구현 결함 | Core의 reward/HQ candidate identity와 Unity reader가 서로 다른 field를 사용한다. |
| H-11 | High | 확정 source-of-truth 충돌 | Unity가 active battle에서 snapshot `allowed_actions`를 정적 whitelist로 교체하며 transport에는 이를 대체할 동적 capability가 없다. |
| H-12 | High | 잠재 통합 결함 | blocking battle VN 중 reconnect의 자동 setup-loss recovery가 Core gate에서 거절된다. |
| H-13 | High | 확정 구현 누락 | canonical의 setup-loss recovery/checkpoint load/roster move command가 Unity runtime에 완성되어 있지 않다. |
| H-14 | High | 확정 불일치 | forced BossOmen 승리가 독립 Gate를 거치지 않고 즉시 Floor를 전환한다. |
| H-15 | High | 확정 불일치 | forced BossOmen 패배 뒤 문서상 rollback 선택을 실제 action으로 실행할 수 없다. |
| M-01~M-31 | Medium | drift 또는 잠재 리스크 | battle record key, action capability, workflow, DTO 예시, live index 등 추가 정리 대상이 있다. |
| L-01~L-05 | Low | 표시 drift 또는 미명시 정책 | 로컬 표시 resource, 예시 누락, 용어와 rewind 보존 정책 등을 정리해야 한다. |

## 4. 상세 발견 사항

### C-01. 전투 중 VN 전용 update가 Unity에서 버려질 수 있음

**판정: Critical / 확정 계약 경로 결함, 현재 live data에는 battle VN trigger 없음**

Canonical transport 계약은 전투 중 `advance_visual_novel` 또는 `select_visual_novel_choice`가 성공하면 실제 VN 변화가 뒤따르는 `battle_update.visual_novel`에 들어오며, `visual_novel == null`이면 overlay를 닫으라고 명시한다.

- Unity transport 계약: `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md:106-118`

Core는 실제로 이 정책대로 동작한다.

1. 전투 중 `VisualNovelState` 결과를 현재 battle update로 바꾼다: `src/game/world.rs:102-107`
2. 현재 latest seq 뒤의 빈 delta를 만든다: `src/game/world/visual_novel_flow.rs:159-179`
3. event가 추가되지 않았으므로 `after_seq == to_seq == latest_seq`이고 battle time도 기존 마지막 event time과 같다: `src/game/world/state.rs:524-563`
4. 그 update에 현재 VN snapshot 또는 `null`을 붙인다: `src/game/world/visual_novel_flow.rs:182-206`

그러나 Unity `BattleRuntimeState.ApplyUpdate`는 다음 조건이면 update를 성공 처리한 것처럼 반환하면서 `LatestUpdate`를 교체하지 않는다.

```text
to_seq == LastAppliedSeq
&& after_seq == LastAppliedSeq
&& server_battle_time_ms <= LastServerBattleTimeMs
```

- Unity 구현: `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/BattleRuntimeState.cs:91-133`
- VN reader는 바로 이 `LatestUpdate["visual_novel"]`을 읽음: `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/Reading/VisualNovelSnapshotModel.cs:83-98`

VN 전용 update는 바로 이 조건을 만족한다. 따라서 VN의 다음 scene이나 완료 후 `null` projection이 버려져 overlay가 stale 상태로 남을 수 있다. blocking VN이면 사용자가 전투로 복귀하지 못하는 형태로 나타날 수 있다.

현재 live trigger catalog는 pre-run `RunStart`만 사용하므로 production content에서 battle VN이 지금 발화한다는 뜻은 아니다. 다만 Core DTO, transport canonical, Unity reader가 모두 battle VN을 지원 표면으로 선언하고 있고, 그 지원 경로는 코드상 결정적으로 충돌한다.

현재 Unity 테스트는 seq/time이 증가하는 update만 다루므로 이 경로를 검출하지 못한다.

- 기존 테스트 범위: `/mnt/f/unity projects/ark/Assets/Tests/EditMode/ArkClientContractTests.cs:684-736`

**정렬에 필요한 결정**

- Unity가 같은 seq/time의 side-band VN 변화도 적용하게 할지,
- 또는 Core transport에 별도의 단조 증가 revision/seq를 둘지 결정해야 한다.

어느 정책을 택하든 `Some(scene A) -> Some(scene B) -> null`이 모두 같은 battle seq/time에서 적용되는 회귀 테스트가 필요하다.

### H-01. `WholeFieldValidTiles + TileArea`가 validation은 통과하지만 runtime에서는 no-op

**판정: High / 확정 불일치**

문서는 이 조합을 전장 광역 스킬 표현으로 명시적으로 허용한다.

- `docs/skill_target_contract.md:152-159`

데이터 validation도 `WholeFieldValidTiles`일 때 별도 `defense_tile_range` 없이 `TileArea`를 허용한다.

- `src/game/data/skill_data.rs:474-482`
- `src/game/data/skill_data.rs:510-528`

하지만 실행기는 `DeliveryDef::TileArea`에서 `defense_tile_range`가 없으면 빈 `SkillStepResult`로 step을 finalize하고 즉시 반환한다.

- `src/game/battle/core/sim.rs:2087-2096`

즉, 문서상 올바르고 validation도 통과한 전장 광역 스킬이 조용히 아무 효과도 내지 않는다. 현재 live skill에서 이 조합을 사용하지 않아 live catalog 테스트는 통과하지만, generic authoring contract는 깨져 있다.

**정렬 후보**

- runtime이 battlefield의 모든 유효 tile을 순회하도록 구현하거나,
- 해당 조합을 validation에서 거부하고 문서의 허용 문구를 제거한다.

### H-02. 스킬 대상 정렬의 source of truth가 문서와 코드에서 다름

**판정: High / 확정 불일치**

문서는 후보를 무기 또는 스킬의 `targeting_profile`로 정렬하고, 스킬이 별도 정책을 명시하지 않으면 장착 무기의 `targeting_profile`을 상속한다고 설명한다.

- `docs/skill_target_contract.md:64-76`
- `docs/skill_target_contract.md:174-178`
- `docs/game_rulebook.md:379`

기본 공격은 실제로 `TargetingProfile`을 사용한다.

- `src/game/battle/core/targeting.rs:122-159`

반면 스킬 대상 선택은 per-skill `UnitTargetRule`을 사용하며 무기 `targeting_profile` 상속 경로가 없다.

- `src/game/battle/core/skill_runtime/cast.rs:277-325`
- `src/game/battle/core/mod.rs:3182` 부근의 현재-target 보존 테스트

예를 들어 `CurrentTarget`은 더 가까운 적이 있어도 유효한 기존 공격 대상을 보존할 수 있다. 이는 단순한 profile 정렬과 동일하지 않다.

**필요 결정**: 문서를 `UnitTargetRule` 중심으로 고칠지, 런타임에 문서의 profile 상속 정책을 구현할지 확정해야 한다.

### H-03. 적군 range preview 포함 정책이 서로 반대임

**판정: High / 확정 불일치**

Core 문서는 적군 공격 범위를 기본 Unity-facing DTO에 매번 포함하지 않고 실제 telegraph/debug cell만 별도 전달한다고 명시한다.

- `docs/skill_target_contract.md:160-162`
- `docs/core_runtime_contract.md:113-114`

하지만 checkpoint 생성 코드는 owner 필터 없이 모든 active unit에 `range_previews_for_runtime_unit`을 적용한다.

- `src/game/world/state.rs:571-588`
- `src/game/range_preview.rs:25-53`

외부 Unity transport canonical도 `checkpoint.units[*].range_previews`를 모든 live unit 범위 overlay의 권위 상태로 설명한다.

- `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md:449-460`

현재는 **코드와 외부 Unity 문서는 서로 맞고, Core의 두 canonical 문서가 반대 정책을 서술**한다. 적군 preview를 일반 UI에 노출할지, DTO에서 제거할지 먼저 정책을 고정해야 한다.

### H-04. 데이터 strictness 보장이 실제 schema보다 강함

**판정: High / 확정 불일치**

데이터 계약은 gameplay schema가 기본적으로 unknown field를 거부하고, 제거된 과거 alias도 거부한다고 설명한다.

- `docs/data_loading_contract.md:20-29`

그러나 `DamageType`은 다음 alias를 명시적으로 허용한다.

- `physical`, `ad`, `Ad`
- `magic`, `ap`, `Ap`
- `true`

근거: `src/game/battle/damage.rs:241-247`

또한 다음 주요 bundle schema의 전부 또는 일부에는 `deny_unknown_fields`가 없어 unknown field가 무시될 수 있다.

- artifact: `src/game/data/artifact_data.rs:15-38`
- buff: `src/game/battle/buffs.rs:58-72`
- consumable: `src/game/data/consumable_data.rs:100-130`
- equipment: `src/game/data/equipment_data.rs:58-75`, `126-145`, `189-212`
- reward: `src/game/data/reward_data.rs:29-91`
- skill fragment: `src/game/data/skill_fragment_data.rs:53-119`, `369-442`

중복 검증도 모든 database에서 동일하지 않다. primary indexed entity는 보통 중복을 잡지만 shop/reward pool 내부 ID는 first-wins 경로가 있다.

- primary index: `src/game/data/mod.rs:94-113`
- shop pool: `src/game/data/shop_data.rs:181-210`
- reward pool: `src/game/data/reward_data.rs:210-250`
- equipment reward pool의 별도 검증: `src/game/data/validation.rs:436-466`

현재 live RON은 정상 로드되고 reference/generated preview validation도 통과한다. 문제는 **현재 데이터가 깨졌다는 뜻이 아니라 문서가 향후 stale field를 반드시 막는다고 과도하게 보장한다는 점**이다.

### H-05. canonical Event node flow가 ArkClient에 없음

**판정: High / 확정 구현 누락**

Unity canonical은 `selected_event.type == "event"` 화면과 다음 명령을 요구한다.

- `advance_event_scene`
- `select_event_choice`

근거:

- `/mnt/f/unity projects/ark/docs/unity_core_contract.md:635-710`
- `/mnt/f/unity projects/ark/docs/unity_core_contract.md:2265-2267`

그러나 ArkClient에는 다음이 없다.

- `GameScreenRoute.Event`: `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameScreenRouter.cs:3-23`
- `selected_event.type == "event"` 처리: 같은 파일 `:91-133`에서 결국 `Unknown`
- `AdvanceEventScene` / `SelectEventChoice` command: repo-wide 검색 결과 없음
- nested Event scene을 읽는 model: `SelectedEventModel`은 현재 필요한 scene shape를 parse하지 않음

Core/live data에는 Event node가 존재하므로 해당 node에 진입하면 현재 Unity route로는 진행할 수 없다. 일부 Unity goal 문서가 Event UI를 out-of-scope로 두었지만 canonical checklist는 완료된 필수 흐름처럼 기술하고 있어 상태 표기도 정렬해야 한다.

### H-06. Event DTO canonical이 현재 Core shape보다 오래됨

**판정: High / 확정 불일치**

Unity canonical은 scene presentation을 다음처럼 설명한다.

- 단일 `script_id`
- `speaker_id`
- `portrait_id`

근거: `/mnt/f/unity projects/ark/docs/unity_core_contract.md:635-710`

현재 Core DTO는 다음 shape다.

```text
background_id
script_ids: Vec<String>
committed_choice_id
choices
```

- DTO: `src/game/behavior.rs:328-345`
- projection: `src/game/world/event_node.rs:262-288`
- live RON: `../game_resources/data/events/story/base.ron:7-57`

Unity의 Event 구현을 추가하더라도 오래된 canonical대로 parser를 만들면 wire contract와 맞지 않는다. 구현 전에 canonical JSON 예시와 field 설명을 먼저 갱신해야 한다.

### H-07. HUD 렌더링 계약이 world-space billboard와 screen-space overlay로 갈림

**판정: High / 확정 문서 충돌**

Core runtime 문서는 유닛 아래 world-space anchor와 camera billboard/up-vector 방식이라고 설명한다.

- `docs/core_runtime_contract.md:110-112`

외부 Unity canonical과 구현 목표는 world anchor를 화면 좌표로 투영하는 dedicated screen-space overlay를 요구하며, world-space canvas/mesh 및 camera billboard를 사용하지 말라고 명시한다.

- `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md:449-456`
- `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md:507-509`

현재 Unity 구현은 외부 문서 쪽과 일치한다.

- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Battlefield/BattlefieldActorHudView.cs:75-99`
- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Battlefield/BattlefieldFieldAssembler.cs:155-160`
- `/mnt/f/unity projects/ark/Assets/Tests/PlayMode/ArkClientMockRenderTests.cs:2603-2628`

따라서 Core runtime 문서의 billboard 문구가 현재 구현에 맞지 않는다.

### H-08. game_server 전체 테스트 6개가 VN trigger fixture drift로 실패

**판정: High / 검증 gate 실패, production live data 실패는 아님**

`cargo test -p game_server -- --test-threads=1` 결과는 25개 중 19개 통과, 6개 실패였다.

실패 테스트:

1. `attaching_to_active_battle_recovers_to_node_confirm_snapshot`
2. `execute_behavior_returns_updated_state_snapshot`
3. `finished_live_battle_tick_pushes_combat_result_snapshot_after_final_update`
4. `live_battle_resync_rejects_future_since_seq`
5. `live_battle_tick_pushes_update_without_state_snapshot_after_confirm_enter`
6. `paused_live_battle_tick_does_not_push_delta_until_resumed`

공통 원인은 `starter_policy_game_data_builder()`가 VN definition은 넣지만 새 필수 catalog인 VN trigger를 넣지 않는 것이다.

- fixture builder: `../game_server/src/game/player_game_actor/handlers.rs:435-442`
- start helper의 성공 전제: 같은 파일 `:531-558`
- 관찰 오류: `InvalidStaticData("missing RunStart visual novel trigger for Standard")`

`cargo check -p game_server`와 production live loader는 통과하므로 이 결과만으로 production startup failure라고 볼 수는 없다. 하지만 전체 server 테스트 gate가 붉은 상태이므로 새 계약을 검증하는 안전망이 깨져 있다.

### H-09. Unity 소비 아이템 command가 Core와 다른 필드명을 전송함

**판정: High / 확정 구현 결함**

Canonical과 Core `PlayerBehavior`는 모두 대상 직원 필드를 `target_employee_uuid`로 정의한다.

- 외부 canonical: `/mnt/f/unity projects/ark/docs/unity_core_contract.md:1830-1847`
- Core enum: `src/game/behavior.rs:925-929`
- server deserialize contract test: `../game_server/src/game/player_game_actor/messages.rs:457-468`

그러나 Unity command bus는 같은 값에 `employee_uuid`라는 key를 사용한다.

- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameCommandBus.cs:118-125`
- gateway는 payload를 고치지 않고 그대로 위임: `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Views/GameCommandBusItemUseGateway.cs:21-23`
- 실제 Item Use 흐름의 호출: `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Views/ItemUseActionHandler.cs:51-59`

`target_employee_uuid`는 optional/default field가 아니므로 이 payload는 server의 `PlayerBehavior` 역직렬화 단계에서 거절된다. Unity 테스트는 fake gateway 호출만 확인하고 실제 `GameCommandBus` JSON key를 검증하지 않아 이 결함을 잡지 못한다.

### H-10. reward 및 HQ candidate identity를 Core와 Unity가 서로 다른 key로 읽음

**판정: High / 확정 구현 결함**

Core의 reward option identity는 `uuid`이며 보조 definition id는 `id`다.

- DTO: `src/game/behavior.rs:694-706`
- reward/combat-result projection: `src/game/world/snapshot.rs:1043-1052`, `:1077-1089`, `:1122-1134`

HQ recruitment candidate identity는 `StarterEmployeeCandidate.id`다.

- DTO: `src/game/employee.rs:39-47`
- projection: `src/game/world/snapshot.rs:1069-1075`

반면 Unity reader는 reward에서 `reward_id`, candidate에서 `candidate_id`만 읽고 identity가 없으면 entry를 목록에서 제거한다.

- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/Reading/SelectedEventModel.cs:70-76`, `:92-133`

더 나아가 Unity 테스트는 Core의 실제 `{ uuid, id }`/`{ id, ... }` shape를 legacy alias로 취급해 버리는 동작을 명시적으로 고정하고, mock은 `reward_id`/`candidate_id`를 사용한다.

- `/mnt/f/unity projects/ark/Assets/Tests/EditMode/ArkClientContractTests.cs:4022-4061`
- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Mock/ArkMockSnapshots.cs:443-455`, `:528-540`

현재 live combat reward는 `ClaimAll`이므로 reward 선택 command가 항상 필요한 것은 아니지만, 결과 reward 목록은 비어 보일 수 있다. HQ node는 live map catalog/generator에 존재하므로 채용 후보가 비어 보여 `RecruitEmployee` command를 구성하지 못하는 실제 진행 결함이다. 외부 canonical은 이 배열 원소의 identity shape도 명시적으로 보강해야 한다.

### H-11. Unity가 active battle의 `allowed_actions`를 정적 whitelist로 대체함

**판정: High / 확정 source-of-truth 충돌**

외부 canonical은 UI 버튼 활성화의 source of truth가 snapshot의 `allowed_actions`라고 명시한다.

- `/mnt/f/unity projects/ark/docs/unity_core_contract.md:236-242`, `:2236-2245`

Unity는 active battle setup/checkpoint가 있으면 snapshot에서 받은 목록을 무시하고 `battleRuntimeActions` 정적 집합의 포함 여부만 반환한다.

- 정적 목록: `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameSnapshotStore.cs:10-22`
- 대체 분기: 같은 파일 `:44-56`

이 구현에는 배경이 있다. battle command는 매번 full `state_snapshot`을 보내지 않으므로 마지막 root snapshot의 action 목록은 node-confirm 값으로 stale할 수 있다.

- server battle side-message mapping: `../game_server/src/game/player_game_actor/state.rs:223-258`

문제는 battle transport에도 이를 대체할 동적 capability field가 없다는 점이다. 즉 정적 whitelist를 제거하면 stale root 목록만 남고, 유지하면 canonical의 server-authoritative 목록을 무시한다.

Core는 blocking battle VN 동안 `AdvanceVisualNovel`과 `SelectVisualNovelChoice`만 허용한다.

- `src/game/managers/action_scheduler.rs:116-121`
- 고정 테스트: `src/game/world/tests/combat.rs:2315-2338`

그럼에도 Unity는 Deploy/Withdraw/Activate/Pause/Resume/Retreat 등을 계속 허용한다. deployment range preview는 command bus에서 action gate 자체를 우회한다.

- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameCommandBus.cs:382-394`
- Unity 테스트는 이 정적 동작을 기대값으로 고정: `/mnt/f/unity projects/ark/Assets/Tests/EditMode/ArkClientContractTests.cs:607-653`, `:1045-1067`

server/Core가 최종 authority라 금지 command가 실제 상태를 변경하지는 않는다. blocking overlay는 full-screen raycast target을 켜므로 일반 pointer click이 아래 battle UI까지 전달되는 것도 막을 가능성이 높다.

- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Views/UguiVisualNovelOverlayView.cs:125-135`

따라서 현재 증거만으로 일반 click이 곧바로 `InvalidAction`을 만든다고 단정하지 않는다. 다만 command bus/API와 programmatic 또는 별도 입력 경로는 금지 command를 허용하며, server capability 변화가 client에 표현되지 않는 source-of-truth 분리는 확정이다.

### H-12. blocking battle VN 중 reconnect의 자동 recovery가 Core gate에서 거절됨

**판정: High / 잠재 통합 결함, 현재 live data에는 battle VN trigger 없음**

game_server `AttachSession`은 새 session이 기존 session을 대체하면 기존 socket을 먼저 종료한다. Core가 `InBattle`이면 tick을 멈추고 일반 player command 경로로 `RecoverBattleSetupLoss`를 실행한 뒤에야 새 session/socket을 저장한다.

- `../game_server/src/game/player_game_actor/handlers.rs:19-53`

일반 `GameCore::execute`는 command가 현재 `allowed_actions`에 있어야 한다.

- `src/game/world.rs:90-124`

하지만 blocking VN은 허용 목록을 VN 진행 두 command로 교체하므로 recovery가 `InvalidAction`이 된다.

- `src/game/managers/action_scheduler.rs:116-121`

따라서 지원 계약상 battle VN이 blocking 상태일 때 reconnect하면 기존 socket은 이미 교체 종료되고 새 attach도 완료되지 않을 수 있다. 이는 reconnect 시 node-confirm으로 recovery한다는 canonical과 충돌한다.

- `/mnt/f/unity projects/ark/docs/unity_core_contract.md:47-60`, `:1064-1073`
- `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md:659-711`

현재 live trigger catalog에는 battle trigger가 없어 production content에서 당장 발화하는 경로는 아니다. 정상 battle reconnect test와 blocking VN Core test는 각각 존재하지만 두 조건을 결합한 회귀 테스트는 없다.

### H-13. canonical command 일부가 Unity runtime에 구현되어 있지 않음

**판정: High / 확정 구현 누락**

Core와 외부 canonical에 있는 다음 명령은 Unity에 전용 typed method/dispatcher/UI flow가 없어 정상 UI 경로에서 보낼 수 없다.

- `RecoverBattleSetupLoss`: canonical `/mnt/f/unity projects/ark/docs/unity_core_contract.md:1064-1073`, Core `src/game/behavior.rs:1048-1049`
- `LoadRunCheckpoint`: canonical `/mnt/f/unity projects/ark/docs/unity_core_contract.md:1394-1418`, Core `src/game/behavior.rs:982-983`
- `MoveRosterUnit`: canonical `/mnt/f/unity projects/ark/docs/unity_core_contract.md:1764-1795`, Core `src/game/behavior.rs:960-965`

Unity에는 `MoveRosterUnit` 상수만 있을 뿐 세 명령 모두 전용 command method/dispatcher/UI 흐름이 없다. 공개 `SendBehavior`/`SendBehaviorRequest` generic escape hatch로 임의 JSON을 만들 수는 있지만 이 기능 흐름에 연결되어 있지 않다.

- 상수 목록: `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameActionNames.cs:3-46`
- 실제 method 목록: `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameCommandBus.cs:66-445`
- generic escape hatch: 같은 파일 `:445-462`
- roster UI 테스트도 move command 부재를 고정: `/mnt/f/unity projects/ark/Assets/Tests/EditMode/ArkClientContractTests.cs:5285-5296`

영향은 서로 다르다. setup/scene actor 손실을 연결 유지 상태에서 복구할 수 없고, SavePoint load UX가 없으며, roster order를 바꿀 수 없다. 추가로 canonical은 roster move를 Map과 NodeConfirm 양쪽에서 허용한다고 하지만 Core scheduler는 `ViewingMap`에만 노출한다.

- Core scheduler: `src/game/managers/action_scheduler.rs:37-64`

Event command 누락은 H-05에 별도로 기록했다. 반대로 Unity에 남은 `SelectSupportTarget`은 현재 Core `PlayerBehavior`에 없는 legacy command다.

### H-14. forced BossOmen 승리가 독립 Gate를 우회해 Floor를 전환함

**판정: High / 확정 불일치**

룰북은 다음 Floor 이동 source가 반드시 별도 `Gate`이며, 엘리트나 보스가 Gate 역할을 겸하지 않고 동적 Boss node도 기존 Gate를 덮어쓰지 않는다고 명시한다.

- `docs/game_rulebook.md:53`, `:102-115`, `:751`

구현은 forced boss node를 추가하면서 `map.terminal_node_id`를 그 boss id로 교체한다.

- `src/game/boss_omen.rs:229-257`

terminal node 완료는 `complete_current_node()`에서 곧바로 floor 완료로 판정되고, node completion은 `advance_floor()`와 다음 map 생성으로 이어진다.

- `src/game/map/progression.rs:232-255`
- `src/game/world/node_flow.rs:396-436`

현재 회귀 테스트도 forced boss 승리 직후 `FloorAdvanced`를 기대한다.

- `src/game/world/tests/combat.rs:4118-4189`

즉 기존 Gate node가 자료구조에 남아 있어도 진행 source 역할을 잃으며, Gate 확인 UX와 기본 10% trauma transition reward도 건너뛴다.

### H-15. forced BossOmen 패배 후 문서상 SavePoint rollback을 선택할 수 없음

**판정: High / 확정 불일치**

룰북은 forced boss 패배가 run failure가 된 뒤 Unity가 남은 횟수에 따라 SavePoint rollback 사용자 선택을 표시한다고 한다.

- `docs/game_rulebook.md:754`

구현은 forced boss를 final boss와 같은 즉시 실패 대상으로 처리하고 `RunFailed`로 전환한다.

- `src/game/world/combat.rs:368-376`, `:1653-1669`
- 회귀 테스트: `src/game/world/tests/combat.rs:4062-4114`

그러나 `RunFailed`의 허용 action은 빈 목록이며 모든 일반 command는 이 gate를 통과해야 한다.

- `src/game/managers/action_scheduler.rs:111-121`
- `src/game/world.rs:112-124`

따라서 checkpoint와 잔여 load 횟수가 있어도 `LoadRunCheckpoint`를 실행할 수 없다. “실패 뒤 선택”을 유지하려면 별도 post-failure capability/state가 필요하고, 그렇지 않으면 룰북에서 rollback 문구를 제거해야 한다.

## 5. 추가 drift 및 잠재 리스크

### M-01. battle record의 “same abnormality” dedupe 설명이 구현보다 넓음

- 문서: `docs/core_runtime_contract.md:48-53`
- battle UUID 생성: `src/game/world/combat.rs:259-320`
- `abnormality_uuid` projection: `src/game/world/combat.rs:831-865`
- dedupe: `src/game/world/state.rs:748-760`

구현은 `battle_uuid = node_id`를 사용하므로 같은 node retry는 dedupe하지만, 서로 다른 node에서 같은 `primary_abnormality_id`를 만난 경우까지 “abnormality당 하나”로 묶지는 않는다. 문서의 “same abnormality” 표현을 “same node/battle identity”로 좁히거나 식별 정책을 바꿔야 한다.

### M-02. checkpoint 밖에 유지되는 VN과 rewind되는 UUID manager의 잠재 충돌

- VN은 checkpoint load 뒤에도 유지: `src/game/world/node_flow.rs:736-760`
- 관련 테스트: `src/game/world/tests/snapshots_and_start.rs:840-848`
- UUID manager는 checkpoint 저장/복원 대상: `src/game/world/state.rs:158`, `src/game/world/node_flow.rs:710`, `:753`
- VN session ID 생성: `src/game/world/visual_novel_triggers.rs:255-260`
- queue에는 duplicate guard가 없음: 같은 파일 `:270-275`

checkpoint 이후 생성된 nonblocking VN이 load 뒤에도 남고, rewind된 trigger가 다시 발화하면 같은 deterministic session ID가 재사용될 가능성이 있다. 현재 live trigger는 blocking RunStart 중심이라 즉시 live 재현된 문제는 아니다. 별도 재현 테스트가 필요한 잠재 리스크다.

### M-03. `ReuseCastTarget` 문서가 실제 유지 조건을 일부 생략

- 문서: `docs/skill_target_contract.md:264-268`
- runtime range 조건: `src/game/battle/core/sim.rs:795-816`
- usefulness 조건: `src/game/battle/core/skill_runtime/cast.rs:422-438`
- 범위를 벗어난 target을 버리는 테스트: `src/game/battle/core/mod.rs:4167` 부근

문서는 alive/hostile/air compatibility를 주로 설명하지만 runtime은 range와 usefulness도 다시 검사한다.

### M-04. TileArea RON 예시에 제거된 `range_units`가 남아 있음

- 오래된 예시: `docs/skill_target_contract.md:324-342`
- 현재 strict raw schema: `src/game/data/skill_data.rs:135-155`
- 거부 테스트: `src/game/data/skill_data.rs:886` 부근

문서 예시를 그대로 복사하면 현재 parser에서 실패한다.

### M-05. `RetargetOnStep` live 예시가 현재 catalog와 다름

- 문서 예시: `docs/skill_target_contract.md:276`
- 실제 live 사용 위치: `../game_resources/data/skills/base.ron:110`, `:290`, `:440`, `:1046`

문서에 적힌 Big Bird `peck_flurry`, Judgement Bird `final_verdict` 예시는 현재 live catalog와 맞지 않는다. 현재 사용자는 Little Red, Nothing There, Funeral 관련 정의다.

### M-06. live skill fragment index 수가 실제 RON보다 적음

- index의 “22개” 주장: `docs/skills/live_content_index.md:3`, `:25`
- 실제 live RON에는 30개 definition이 존재
- 누락된 후반 정의 시작점: `../game_resources/data/skill_fragments/base.ron:453`, `:472`, `:491`, `:510`, `:529`, `:548`, `:567`, `:586`

의도적으로 일부만 싣는 index라면 범위를 명시하고, 전체 live index라면 30개로 갱신해야 한다.

### M-07. “next Safe Node” 연구 전달과 Maintenance 처리 불일치

- Maintenance를 safe node로 분류: `docs/game_rulebook.md:177`
- next Safe Node에서 연구 전달: `docs/game_rulebook.md:432`
- 실제 delivery 대상: `src/game/skill_fragment.rs:469` 부근
- node entry 호출: `src/game/world/node_flow.rs:252`

현재 정책 대상은 Support, Headquarters, Shop, Reward이며 Maintenance는 빠져 있다.

### M-08. 문서의 weapon block modifier를 현재 profile이 지원하지 않음

- 문서: `docs/game_rulebook.md:518`
- `WeaponCombatProfile`: `src/game/data/equipment_data.rs:58` 부근
- weapon 적용 경로: `src/game/battle/types.rs:365` 부근

현재 weapon profile에는 basic block modifier field가 없고 weapon 적용도 `block_capacity`를 수정하지 않는다.

### M-09. live catalog coverage manifest가 behavior coverage를 직접 증명하지 않음

- manifest: `docs/audit/live_skill_catalog_manifest.ron:34` 이후
- test: `tests/live_skill_catalog_audit.rs:81` 이후

테스트는 live ID 집합과 manifest ID 집합을 비교하지만 각 ID가 실제 behavior assertion scenario에 연결되는지 검증하지 않는다. 따라서 green 결과를 “모든 live skill behavior가 테스트됨”으로 해석하면 과도하다.

### M-10. `FacilityEntity`를 production DTO variant로 부르는 외부 문서가 과도함

- 외부 문서: `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md:437-443`
- wire enum: `src/game/battle/types.rs:117-142`
- runtime draft에는 variant 없음: 같은 파일 `:98-115`
- setup에서 미지원 처리: `src/game/combat_setup/enemy_spawns.rs:165-167`
- validation 미지원: `src/game/data/validation.rs:572-576`
- Core rulebook은 미구현으로 설명: `docs/game_rulebook.md:338`, `:368`

현재 상태는 “wire-reserved/future”에 가깝다.

### M-11. VN definition의 `default_blocking`이 runtime source로 사용되지 않음

- definition field: `src/game/data/visual_novel_data.rs:54-59`
- runtime session은 trigger의 `blocking`을 사용: `src/game/world/visual_novel_triggers.rs:255-267`

definition과 trigger에 blocking 정책이 중복 authoring되어 있으나 현재 runtime source는 trigger다. `default_blocking`의 역할을 문서화하거나 제거/연결할 필요가 있다.

### M-12. game_server 보조 bridge 문서가 현재 battle side-message 정책과 충돌

- 보조 문서의 항상 2-message 주장: `../game_server/docs/unity_bridge_protocol.md:42-51`
- 실제 battle mapping: `../game_server/src/game/player_game_actor/state.rs:223-258`
- canonical의 battle snapshot 금지 설명: `/mnt/f/unity projects/ark/docs/unity_core_contract.md:130`

현재 battle command는 `command_result`와 `battle_update`/`battle_resync` side message를 보내고 `send_state_snapshot: false`다. 보조 문서의 “항상 command_result -> state_snapshot”은 현재 코드와 맞지 않는다.

### M-13. `allowed_actions`가 “실행 가능한 command만”이라는 보장이 Core에서도 성립하지 않음

- 외부 문서의 local-invariant 보장: `/mnt/f/unity projects/ark/docs/unity_core_contract.md:236-242`
- broad state 목록: `src/game/managers/action_scheduler.rs:66-95`
- Shop은 `RerollShop`을 항상 노출하지만 handler는 `can_reroll == false` 또는 hidden item 부재 시 거절: `src/game/world/shop.rs:14-32`
- Reward는 `SelectReward`와 `ClaimReward`를 함께 노출하지만 `SelectReward`는 `ChooseOne`에서만 허용: `src/game/world/reward.rs:267-287`

H-11은 Unity가 server 목록 자체를 무시하는 문제이고, 이 항목은 그 server 목록도 payload/local invariant를 모두 반영하지 않는 별도 문제다. 현 계약을 유지하려면 capability 계산을 세분화하고, broad affordance 목록을 의도한 것이라면 canonical 보장을 낮춰야 한다.

### M-14. Unity skill catalog JSON 예시가 실제 typed DTO와 다름

- 외부 예시는 cast/step에 `range_units`를 넣고 `range_policy`를 생략: `/mnt/f/unity projects/ark/docs/unity_core_contract.md:1934-1977`
- 실제 DTO는 `range_policy`를 가지며 `range_units`가 없음: `src/game/behavior.rs:2133-2171`
- 같은 문서는 typed DTO와 예시가 일치해야 한다고 명시: `/mnt/f/unity projects/ark/docs/unity_core_contract.md:1980-1984`

문서 예시로 Unity model/test를 작성하면 현재 Core snapshot과 다른 shape를 고정하게 된다.

### M-15. 외부 문서가 존재하지 않는 `ActComplete` result type을 안내함

- 오래된 문구: `/mnt/f/unity projects/ark/docs/unity_core_contract.md:1292`, `:1681`
- 현재 enum/mapping은 `FloorAdvanced`: `src/game/behavior.rs:2265-2270`, `:3266-3274`

현재 runtime에는 `ActComplete` result가 없다. 문서대로 result switch를 작성하면 Floor 전환 결과를 놓칠 수 있다.

### M-16. combat result 대표 환상체 field 이름이 canonical/mock과 다름

- 외부 canonical/mock: `abnormality_id`: `/mnt/f/unity projects/ark/docs/unity_core_contract.md:1184-1195`, `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Mock/ArkMockSnapshots.cs:443-449`
- Core DTO/projection: `primary_abnormality_id`: `src/game/behavior.rs:748-760`, `src/game/world/snapshot.rs:1077-1089`

현재 Unity `CombatResultModel`은 어느 field도 읽지 않으므로 현 UI의 실제 metadata 차이라고 단정할 수는 없다. 다만 canonical/mock 기준으로 새 consumer를 작성하면 Core live snapshot의 대표 환상체 field를 놓치게 되는 명확한 shape drift다.

### M-17. `NoDeployableEmployees` 판정이 선택 가능한 회복 경로를 검사하지 않음

- 룰북은 출전 가능한 직원과 회복 가능 node가 모두 없을 때만 run failure라고 설명: `docs/game_rulebook.md:84`, `:708-714`
- roster 기반 failure 판정: `src/game/world/combat.rs:675-689`
- 구현이 인정하는 복구 예외는 checkpoint뿐: `src/game/world/combat.rs:731-765`, `:1711-1726`, `:1813-1819`
- HQ는 실제 새 직원을 모집할 수 있음: `src/game/world/headquarters.rs:32-52`
- live map generator도 HQ node를 만들 수 있음: `../game_resources/data/map/generation_policy.ron:2-7`, `../game_resources/data/map/node_definitions.ron:76-84`

문서는 어떤 category를 “회복 가능 node”로 인정하는지 명시하지 않으며 SavePoint/Rest/Maintenance는 오히려 HP 회복 경로가 아니라고 설명한다. HQ 모집을 이 조건의 recovery로 의도했는지도 확정할 수 없다. 현재 정상 플레이에서 `Alive`이면서 deploy 불가인 roster 상태를 만드는 경로도 제한적이므로, 이 항목은 확정 구현 결함이 아니라 **map recovery predicate가 코드와 문서 양쪽에 정의되지 않은 잠재 정책 gap**으로 남긴다.

### M-18. `run_checkpoint.can_load`가 현재 실행 가능 여부까지 나타내지 않음

- 룰북 의미: `docs/game_rulebook.md:581-586`
- DTO 계산은 payload 존재와 잔여 횟수만 확인: `src/game/world/state.rs:161-174`
- handler는 `ViewingMap`에서만 허용: `src/game/world/node_flow.rs:716-722`

따라서 battle/shop/node-confirm에서도 `can_load: true`일 수 있지만 command는 불가능하다. 다만 Core 룰북과 외부 canonical은 실제 버튼에 `allowed_actions`도 함께 요구하므로 기능 계약 자체는 안전하다. 이 항목은 `can_load`가 capacity/존재 가능성인지 현재 context까지 포함한 command 가능성인지 이름과 설명을 명확히 할 필요가 있는 의미상 ambiguity다.

### M-19. 룰북에 명시된 `consumable_use_preview` DTO가 없음

- 요구: `docs/game_rulebook.md:217-228`
- root/inventory DTO: `src/game/behavior.rs:488-497`, `:826-844`

현재 snapshot에는 사용 가능 여부, 대상 후보, 효과 요약, disabled reason을 제공하는 preview가 없다. Unity는 command 실패에 의존하거나 Core 규칙을 로컬 추론해야 한다.

### M-20. 세 번째 후퇴는 combat result 확인 없이 node를 즉시 완료함

- 일반 결과 흐름: `docs/game_rulebook.md:635-662`
- 시도 소진 후 retreat 구현: `src/game/world/combat.rs:1532-1594`

마지막 후퇴는 battle record/outcome과 consumable 감소를 만든 뒤 `CombatResult` state를 거치지 않고 `handle_complete_node()`를 호출한다. 후퇴를 결과 화면 예외로 둘 의도라면 룰북에 명시해야 하고, 모든 전투 node outcome을 확인하게 할 정책이면 구현을 맞춰야 한다.

### M-21. 특수 유닛 기본 공격 range의 “fallback 금지” 계약과 generic runtime이 충돌

- 계약: `docs/skill_target_contract.md:141`
- authored abnormality schema는 `defense_tile_range: None`을 명시적으로 허용하고 Pattern일 때도 누락을 거부하지 않음: `src/game/data/abnormality_data.rs:119-164`, `:194-218`
- runtime은 role별 기본 pattern으로 fallback: `src/game/battle/types.rs:305-327`
- targeting/preview도 fallback 사용: `src/game/battle/core/targeting.rs:325-345`, `src/game/range_preview.rs:89-110`

현재 live abnormality data는 필요한 range를 명시해 직접 실패하지 않는다. 그러나 future boss/elite/special authoring 누락을 data error로 처리한다는 generic contract는 지켜지지 않는다.

### M-22. event log seq가 delivery identity와 동일 timestamp 처리 순서에 스며드는 구조적 리스크

- RNG/관측 분리 계약: `docs/core_runtime_contract.md:7-17`
- 기본 공격 projectile ID seed가 recording parent seq를 사용: `src/game/battle/core/basic_attack_projectile.rs:48-69`
- skill projectile도 동일: `src/game/battle/core/skill_runtime/projectile.rs:535-550`
- skill area identity는 cast seq를 직접 사용: `src/game/battle/core/skill_runtime/area.rs:45-61`, `:372-388`
- 동일 timestamp event 정렬이 skill delivery/area UUID를 tie-breaker로 사용: `src/game/battle/enums.rs:202-269`
- basic projectile 정렬도 projectile UUID를 사용: 같은 파일 `:348-354`

따라서 관측 log를 추가해 parent/cast seq가 바뀌면 delivery UUID가 바뀌고, 동일 timestamp의 gameplay event 처리 순서까지 바뀔 가능성이 있다. crit/proc RNG 독립 테스트는 있지만 이 ordering chain을 재현한 테스트는 없다. 실제 결과 변화는 아직 재현하지 않았으므로 구조적 잠재 리스크로 분류한다.

### M-23. goal cleanup과 completion review 사이의 lifecycle 순서가 문서화되지 않음

- goal은 진행 중 기억장치이고 완료 뒤 durable policy만 옮긴다고 설명: `docs/README.md:61-73`
- 완료 goal 문서를 제거하라는 workflow: `docs/engineering_workflow.md:194-199`
- completion review는 완료 item text와 연결된 subgoal directory를 입력으로 요구: `docs/goal_completion_review_guide.md:3-5`, `:29-42`

review를 cleanup 전에 수행하면 양립할 수 있지만 그 순서와 보존 책임이 명시되어 있지 않다. 실제 scan에서는 top-level goal directory 113개 중 22개가 비어 있고, 완료 표시가 남은 PLAN도 다수라 현재 filesystem 역시 어느 lifecycle 단계인지 불명확하다.

### M-24. live skill catalog manifest의 ownership/update lifecycle이 durable 문서에 없음

- 실제 test source: `docs/audit/live_skill_catalog_manifest.ron`
- include 및 집합 검증: `tests/live_skill_catalog_audit.rs:29-35`, `:81-104`

이 파일은 테스트 결과를 바꾸는 관리 대상이지만 누가 언제 어떤 behavior evidence를 보고 갱신하는지 durable workflow/contract가 설명하지 않는다. M-09의 “manifest가 behavior test 연결을 증명하지 않는다”와 결합되어 self-attested 목록이 고착될 위험이 있다.

### M-25. workflow가 요구하는 selected-event/WebSocket probe surface가 현재 workspace에 없음

- probe 동기화 요구와 변경 매트릭스: `docs/engineering_workflow.md:63`, `:71-84`
- 검증 순서/예시: 같은 파일 `:212-240`

`/mnt/f/work/simulator` 전체에서 `.py` probe 파일을 찾지 못했다. 예시의 `<available-probe-path>`는 명시적으로 placeholder이므로 잘못된 구체 경로라고 볼 수는 없지만, selected-event 변경 시 “반드시 확인”할 probe surface가 현재 제공되지 않는다.

### M-26. 장비 제작 비활성 문구와 `EquipItem` 자동 조합의 경계가 모호함

- 룰북은 제작이 현재 live Maintenance action이 아니라고 설명: `docs/game_rulebook.md:520-527`
- `EquipItem`은 recipe가 맞으면 장착 장비와 새 장비를 소비해 결과 장비를 생성: `src/game/world/maintenance.rs:16-175`
- live combination recipe 존재: `../game_resources/data/equipments/base.ron:1703-1717`

현재 문구는 “별도 Maintenance 제작 action/button이 없다”는 의미로 읽으면 자동 `EquipItem` 조합과 완전히 양립한다. 제작 자체가 없다는 뜻으로 오해되지 않게 자동 조합과 별도 `CraftEquipment` action의 경계를 명시할 필요가 있는 용어 ambiguity다.

### M-27. live 전투 보상 설명이 현재 catalog보다 앞서 있음

- 룰북은 E.G.O 잔재, 설계도, 낮은 확률 파편/완제품 후보를 현재 보상 조합처럼 기술: `docs/game_rulebook.md:420-424`
- reward definition catalog: `../game_resources/data/events/rewards/base.ron:8-145`
- live encounter가 실제 참조하는 고정 reward UUID: `../game_resources/data/pve/encounters.ron:9-15`, `:420-425`, `:904-909`
- 고정 지급 경로: `src/game/combat_setup/rewards.rs:15-39`

live encounter의 실제 UUID 집합은 재화, 장비 가루, 연구, 일부 확정 파편 정의를 사용하고 equipment-pool reward는 참조하지 않는다. E.G.O 잔재/설계도/확률형 후보 지급 계약도 없다. 현재 정책과 미래 콘텐츠 방향을 구분해야 한다.

### M-28. Maintenance 일반 기능 설명보다 live recipe 범위가 좁음

- 일반 무기·방어구·악세서리 분쇄/강화 설명: `docs/game_rulebook.md:600-613`
- live recipe: `../game_resources/data/equipments/base.ron:1719-1753`

runtime은 recipe 기반으로 일반화되어 있지만 현재 live 분쇄·강화 recipe는 armor인 `standard_armor` 한 개뿐이며 weapon/accessory recipe는 없다. 이것이 86개 definition마다 recipe가 필요하다는 뜻은 아니다. 다만 룰북이 무기·방어구·악세서리 모두를 현재 일반 기능처럼 서술하는 범위는 live content보다 넓다.

### M-29. 직원 XP/level의 실제 정책이 주요 canonical에 거의 없음

- 실제 XP/tier 정책: `../game_resources/data/run/policy.ron:7-14`
- level-up 계산: `src/game/employee.rs:331-336`
- `AliveRoster` reward amount는 대상별 지급량이 아니라 전체 대상에 분할: `src/game/reward.rs:473-540`
- 룰북 관련 언급: `docs/game_rulebook.md:666`

레벨당 XP, 생존자 XP, battle tier 전환, reward 분배 의미를 주요 문서에서 확인하기 어렵다. 코드와 RON은 동작하지만 사용자-visible 성장 계약이 문서화되지 않았다.

### M-30. inventory capacity와 full-reward 처리 계약이 없음

- 장비/소비 아이템/artifact capacity: `src/game/resources/inventory.rs:442-480`, `:546-580`, `:637-683`
- item reward의 full 오류: `src/game/reward.rs:451-465`

snapshot에는 최대치가 없고 주요 문서도 full inventory에서 reward를 어떻게 처리하는지 설명하지 않는다. 현재 live combat reward는 이 경로를 사용하지 않지만 향후 item reward가 추가되면 `CombatResult`에서 inventory를 비울 action 없이 진행 불능이 될 수 있다.

### M-31. 소비 아이템 duration의 “node resolved” 정책이 문서끼리도 충돌함

- Core 룰북은 전투/보스 node가 해결될 때만 감소한다고 설명: `docs/game_rulebook.md:217-228`
- 외부 canonical은 같은 이상현상에서 남은 시도가 있는 재진입에는 modifier가 유지된다고 하면서, 같은 문단 뒤에서는 “퇴각 없이 실패”하면 감소한다고 설명: `/mnt/f/unity projects/ark/docs/unity_core_contract.md:1830-1836`
- 현재 일반 패배 코드는 감소 후 남은 시도가 있으면 같은 node의 `NodeConfirm`으로 복귀: `src/game/world/combat.rs:1672-1694`
- retreat는 시도 소진 시에만 감소: `src/game/world/combat.rs:1532-1587`

현재 코드는 외부 canonical 문단의 마지막 문장과 맞지만 Core 룰북의 “node resolved” 문구 및 같은 문단의 재진입 유지 문구와는 맞지 않는다. 따라서 구현 결함으로 단정하지 않고, `NextCombatNode`/`CombatNodes(n)`이 재도전 가능한 일반 패배에서 소모되는지 정책을 먼저 확정해야 하는 Medium 문서/정책 충돌로 분류한다.

### L-01. Endless pre-run VN의 Unity 로컬 presentation resource 누락

- 계약상 Standard/Endless 허용: `/mnt/f/unity projects/ark/docs/unity_core_contract.md:432`
- live Endless VN: `../game_resources/data/visual_novels/base.ron:50-64`
- live trigger: `../game_resources/data/visual_novel_triggers/base.ron:13-19`
- Unity catalog에는 Standard만 존재: `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Presentation/VisualNovelLocalCatalog.cs:171-180`
- missing script fallback: 같은 파일 `:222-230`

진행 자체는 가능하지만 Endless briefing 내용이 `"..."` fallback으로 보일 수 있다. live flow test도 Standard만 다룬다.

### L-02. Unity snapshot root canonical 예시에 `visual_novel`과 `skill_catalog` 누락

- 오래된 root JSON 예시: `/mnt/f/unity projects/ark/docs/unity_core_contract.md:182-234`
- 실제 DTO: `src/game/behavior.rs:826-844`
- 같은 문서의 후반 요구: `/mnt/f/unity projects/ark/docs/unity_core_contract.md:434`, `:1928-1984`

문서 내부에서도 후반 설명과 root 예시가 서로 다르다.

### L-03. Unity가 active consumable의 실제 `name`을 읽지 않음

- Core/canonical field: `name`: `src/game/behavior.rs:587-596`, `src/game/world/snapshot.rs:898-907`, `/mnt/f/unity projects/ark/docs/unity_core_contract.md:1857-1864`
- Unity reader는 `display_name`을 보고 없으면 effect JSON으로 fallback: `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/Reading/RosterModel.cs:180-191`

H-09의 command key를 고쳐도 직원의 active consumable label이 실제 아이템 이름 대신 raw effect로 표시될 수 있다.

### L-04. SavePoint/Gate의 “전투불능 제외” 용어가 life state와 availability를 혼용함

- 문서: `docs/game_rulebook.md:113-114`, `:587-590`
- 구현은 `life_state == Alive`만 확인: `src/game/world/support.rs:556-568`, `:590-602`

정상 `apply_incapacitation` 경로는 전투불능 직원을 `Dead + Unavailable`로 만들므로 현재 live 흐름에서 `Alive + Unavailable` 회복 불일치는 확인되지 않았다.

- `src/game/employee.rs:357-369`

따라서 이 항목은 오직 “전투불능”이 life state인지 availability인지 문서 용어를 명확히 해야 하는 ambiguity다.

### L-05. checkpoint 뒤 battle record 보존 결정이 goal note에만 남아 있음

- checkpoint payload에는 battle record가 포함되지 않음: `src/game/world/node_flow.rs:688-711`
- load 시 현재 battle record를 별도로 보존: 같은 파일 `:729-746`

rollback으로 취소된 timeline의 완료 전투 기록도 남는다. 과거 goal note에는 이를 “실제로 발생한 기록”으로 보존한다는 결정이 남아 있다.

- `docs/goals/save_point_checkpoint_support/PLAN.md:88`, `:234`
- `docs/goals/save_point_checkpoint_support/EXPERIMENT_NOTES.md:38`, `:44`

즉 정책 자체가 전혀 결정되지 않은 것이 아니라, durable major canonical로 승격되지 않아 goal cleanup 뒤 근거를 잃을 수 있는 문서 lifecycle drift다.

## 6. 확인된 주요 일치 항목

다음 영역은 이번 감사 범위에서 문서·코드·live data가 대체로 일치했다.

- `docs/README.md`가 가리키는 canonical 위치와 외부 Unity 문서 위치
- 문서 내부 상대 링크 및 로컬에 남은 중복 Unity canonical 부재
- embedded live bundle의 main 18개 영역과 domain-owned 4개 영역의 경로/loader ownership
- game_server startup이 Core의 `load_live_embedded` 경로에 위임되는 구조
- 현재 live RON의 parsing, cross-reference validation, generated combat preview validation
- pre-run `RunStartFlowStarted`, `pre_run_briefing`, top-level `visual_novel` snapshot shape
- Core -> server -> Unity의 `battle_update.visual_novel` wire field shape 자체
- event log/checkpoint, movement presentation, combat result/stat projection의 주요 구조
- VN trigger ordering, blocking queue, threshold crossing의 주요 Core 동작
- Standard 3-floor 구조, deployment/redeployment, gate/savepoint, ClaimAll의 주요 정책
- HUD bar mode/threat class의 DTO source-of-truth 원칙
- Event node의 Core-side retry/commit 흐름
- `PlayerBehavior.type` snake_case, `ActionKind` PascalCase, `FacingDirection` snake_case, battle speed enum의 기본 wire casing
- battle setup -> update, nested resync shape, future cursor rejection, final update -> combat-result snapshot의 server 순서
- 현재 live combat encounter의 `ClaimAll` reward mode와 Core validation
- 일반 Gate 전환의 10% trauma 회복, SavePoint 단일 checkpoint/최대 3회 load의 Core-side 기본 정책

주의: `battle_update.visual_novel`의 **shape**는 맞지만 C-01의 Unity acceptance policy 때문에 **적용 동작**은 맞지 않는다.

## 7. 실행한 검증

### 7.1 통과

```text
cargo fmt --all -- --check
cargo test -p game_core --lib -- --test-threads=1
  640 passed
cargo test -p game_core --test ron_loading -- --test-threads=1
  18 passed
cargo test -p game_core --test skill_runtime_contract -- --test-threads=1
  13 passed
cargo test -p game_core --test live_skill_catalog_audit -- --test-threads=1
  3 passed
cargo test -p game_core --test live_item_skill_activation -- --test-threads=1
  3 passed
cargo test -p game_core --test skill_test_suite -- --test-threads=1
  4 passed
cargo check -p game_server
```

위 lib/integration 묶음은 `cargo test -p game_core` 전체 681개 test invocation과 동일 범위를 이루며 모두 통과했다. 추가로 다음 현재 동작 고정 test를 focused 실행했고 통과했다.

- forced BossOmen victory/failure 2개
- consumable retreat/re-entry 1개
- no-deployable failure 1개
- SavePoint recovery/checkpoint 1개
- blocking battle VN 1개
- setup-loss recovery 1개
- checkpoint load 2개
- server `PlayerBehavior` deserialize 1개

이 통과 결과는 문서 정합성을 증명하는 것이 아니라 H-14/H-15 등에서 지적한 **현재 구현 동작이 테스트에도 고정되어 있음**을 확인한 것이다. M-31은 retreat 쪽 유지/소진 branch만 test가 있고 일반 패배 조합은 아직 없다. data/builtin 및 event/range/result/VN 관련 일부 focused test는 681개 전체 범위의 부분집합이므로 별도 총계로 합산하지 않는다.

### 7.2 실패

```text
cargo test -p game_server -- --test-threads=1
  19 passed, 6 failed
```

실패 원인은 H-08에 기록했다.

### 7.3 실행하지 않은 검증

- Unity EditMode/PlayMode test runner 실제 실행
- Core-server-Unity를 모두 연결한 end-to-end live session
- C-01의 same-seq/same-time VN progression 전용 재현 테스트
- H-09의 Unity 실제 payload를 server에 전달하는 WebSocket 재현
- H-10의 live Core snapshot으로 reward/HQ 화면을 구성하는 Unity test
- H-12의 blocking battle VN + reconnect 조합 test
- M-31의 일반 패배 + 남은 재도전 + consumable duration 조합 test
- M-02의 checkpoint/UUID collision 전용 재현 테스트
- M-22의 관측 log 추가 전후 same-time delivery ordering 비교 test

Unity 영역의 판정은 canonical과 C# 구현 및 기존 test source의 정적 교차 검토 결과다.

## 8. 권장 정리 순서

1. **P0:** H-09의 command key와 H-10의 reward/HQ identity를 typed DTO 기준으로 맞추고 양방향 wire-shape test를 추가한다.
2. **P0:** live Event node를 지원할지 미지원으로 선언할지 결정하고 H-05/H-06의 route, command, DTO canonical을 함께 정리한다.
3. **P0:** C-01, H-11, H-12를 묶어 battle side-band revision, `allowed_actions`, reconnect recovery의 authority를 하나로 정하고 조합 회귀 테스트를 만든다.
4. **P1:** H-13의 setup-loss/checkpoint/roster command surface를 Unity에 구현하고 NodeConfirm 허용 범위를 맞춘다.
5. **P1:** H-14/H-15의 forced BossOmen Gate 및 rollback 정책을 결정하고, M-31의 consumable 감소 시점에서 “node resolved” 의미를 통일한다.
6. **P1:** H-01의 whole-field TileArea를 구현하거나 validation에서 금지한다.
7. **P1:** H-02/H-03/H-07에서 실제 source of truth를 하나씩 선택해 양쪽 canonical을 맞춘다.
8. **P1:** H-08 fixture에 VN trigger catalog를 넣어 server test gate를 복구한다.
9. **P2:** 데이터 strictness를 실제로 강화할지 H-04의 문서 보장을 낮출지 결정한다.
10. **P2:** M/L 항목의 capability 의미, workflow, index, 예시, 보조 문서, latent invariant를 정리한다.

## 9. 정책 결정이 필요한 질문

이 항목들은 단순 문구 수정 전에 팀 차원의 source-of-truth 결정이 필요하다.

1. VN 전용 battle update는 event seq와 별개의 revision을 가져야 하는가, 아니면 Unity가 동일 seq/time update를 항상 받아야 하는가?
2. `WholeFieldValidTiles + TileArea`는 실제 지원 기능인가, 금지할 미래 기능인가?
3. 스킬 타겟 선택의 권위는 `UnitTargetRule`인가, `targeting_profile` 및 weapon inheritance인가?
4. 적군 `range_previews`는 일반 checkpoint에 포함되는가, telegraph 전용으로 제한되는가?
5. Battle HUD canonical은 현재 구현과 같은 screen-space overlay인가?
6. Event node는 현재 지원 대상인가, 아직 미구현인 future contract인가?
7. 데이터 계약은 전 영역 strict rejection을 보장해야 하는가, 영역별 예외를 문서화해야 하는가?
8. `allowed_actions`는 payload/local invariant까지 반영한 “지금 실행 가능” 목록인가, state-level coarse affordance인가?
9. forced BossOmen 승리도 독립 Gate를 거쳐야 하는가? 패배 후 rollback을 허용하려면 어떤 post-failure state가 필요한가?
10. 소비 아이템 duration의 “node resolved”는 승리/소진뿐인가, 재도전 가능한 패배도 포함하는가?
11. completed goal review는 goal cleanup 전 반드시 수행하는가, 아니면 별도 보존 index가 필요한가?

## 10. 문서 승격 전 체크리스트

- [ ] 각 C/H 항목의 intended policy를 결정
- [ ] 회귀 테스트 또는 validation test 추가
- [ ] Core serialize fixture와 Unity reader/command payload를 같은 JSON으로 검증하는 contract test 추가
- [ ] Core canonical 문서 갱신
- [ ] 외부 Unity canonical 문서 갱신
- [ ] game_server 보조 문서 갱신
- [ ] 사용 가능한 WebSocket/non-combat probe 경로 복구 또는 workflow 요구 조정
- [ ] live content index/예시 재생성 또는 수동 동기화
- [ ] Core, game_server, Unity 전체 test gate green 확인
- [ ] 이 임시 문서를 정식 audit report로 승격하거나 삭제
