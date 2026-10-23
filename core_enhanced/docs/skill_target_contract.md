# Skill Target Contract

작성 목적: DefenseRoute 스킬의 기준점 선택, 표시 범위, 시전 가능 범위, 실제 피격 범위를 하나의 계약으로 유지하기 위한 문서다.

## 문서 책임

| 구분 | 내용 |
| --- | --- |
| 문서 등급 | Combat-domain canonical |
| 소유 범위 | 기본 공격과 스킬의 대상 획득, 타일 범위, 자동 적대 타겟 usefulness, cast/step/interrupt/projectile/TileArea 의미, 최종 range preview cell의 판정 의미 |
| 소유하지 않는 범위 | 전체 전투·런 진행 규칙, 일반 runtime commit/recovery, live loader의 공통 strictness, Unity event의 정확한 JSON shape와 message order |
| 함께 확인할 문서 | [game_rulebook.md](game_rulebook.md), [core_runtime_contract.md](core_runtime_contract.md), [data_loading_contract.md](data_loading_contract.md), 외부 `core_unity_battle_transport_contract.md` |

## Targeting Foundations

이 장은 기본 공격과 스킬이 사용할 후보 범위, 자동 대상 선택과 유효 적대 대상 판정의 공통 기반을
소유한다. 전체 전투 진행과 live loader의 공통 strictness는 인접 canonical을 따른다.

### 공식 전투 기준

DefenseRoute는 서로 다른 typed source로 범위를 결정한다.

- 기본 공격 후보: 무기 `basic_attack.defense_tile_range` 또는 침식 profile의 range preset
- Skill unit acquisition: `SkillCastTargetingDef.reach + target_range`
- Skill area delivery: `DeliveryDef::TileArea.footprint`

Core는 unit position/facing, loadout과 battlefield valid tile을 반영해
`range_previews.basic_attack/active_skill.*.cells`를 최종 계산한다. Unity는 authored metadata로
overlay를 재계산하지 않는다.

Preview는 valid tile과 obstacle/blocked tile을 포함하고 battlefield 밖, void와 invalid row
space를 제외한다. 장애물은 이동·배치·pathfinding에는 영향을 주지만 range에서 자동 제외되지
않으며 실제 대상 가능성은 air/target validation이 판단한다.

Core 내부에서는 하나의 typed tile-reach query가 위 세 source의 공통 투영 연산만 담당한다. Query는
policy/pattern, 기준 tile, facing과 현재 battlefield valid tile을 캡처하고, 대상 tile membership과
stable row-major valid-cell materialization을 별도 연산으로 제공한다. Target selector, PlayerTile
acceptance/capability, range preview, TileArea runtime과 automatic usefulness는 이 연산을 공유한다.
이 공통화는 source 의미를 합치지 않는다. Cast acquisition은 계속 `reach + target_range`, 실제 area
delivery는 계속 `TileArea.footprint`를 입력으로 사용하며 obstacle, occupancy, line-of-sight 또는 path
cost 필터는 query에 포함하지 않는다.

Default release에서 이 targeting/movement/skill 실행 상태를 소유하는 `battle::core` module은
crate-private이다. 외부 caller는 runtime unit field, kernel sequence/seed/log field 또는 raw `BattleEvent`
queue를 직접 변경할 수 없고, production command는 `GameCore` application authority가 소유한 active battle
port만 통과한다. 기존 integration oracle의 direct kernel fixture는 test/debug/`test-fixtures` capability에서만
컴파일된다.

Continuous world-space position은 finite `WorldVec2`로 운반한다. Raw float 생성과 deserialization은 x/y의
NaN/positive infinity/negative infinity를 mutation 전에 reject하며 radius, speed와 float time conversion도
동일한 checked finite boundary를 사용한다. Non-finite input을 clamp/default/normalize하지 않는다. Tile
acquisition과 public command position은 계속 integer `Position`을 사용하므로 이 cutover는 range cell,
facing, anchor, projectile/TileArea geometry 또는 deterministic tie-breaker 의미를 바꾸지 않는다.

공식 live skill은 연속좌표 `DeliveryDef::Area(Circle/Line/Box/Rectangle/Cone)`를 사용하지 않는다.

### Weapon Targeting Profile

직원의 고정 직업이 아니라 장착 무기의 `range_role`, `weapon_archetype`,
`targeting_profile`이 기본 공격 역할을 만든다. 현재 archetype은 Sword, Spear, Shield, Bow,
Gun, Staff다.

대상 선택 순서:

1. 공격/스킬의 eligibility와 tile range로 후보를 필터링한다.
2. 근거리 기본 공격은 저지 중인 적을 먼저 고른다.
3. 기본 공격은 weapon `targeting_profile`, skill `EnemySingle.rule`로 정렬한다.
4. 동률은 deterministic tie-breaker로 해소한다.

Skill은 `cast_targeting.target`을 명시하고 EnemySingle이면 rule도 필수다. Weapon profile을
상속하지 않으므로 무기 변경이 skill target rule을 바꾸지 않는다.

### Basic Attack Range Authoring

Basic attack의 stop/intent/target eligibility는 body radius나 continuous world distance가 아니라 runtime
tile과 최종 `TileRangePattern`으로만 판정한다. 저지 중인 적은 같은 logical
tile의 blocker-first 규칙을 적용한다.

- 직원은 weapon range를 사용하고 skill effect가 이를 대체/보정할 수 있다. Source가 없을 때만
  자기 tile과 facing 기준 전방 1칸을 fallback으로 사용한다.
- 침식 직원은 profile이 필수 range preset을 참조하고 loader가 이를 pattern으로 해석한다.
  `range_role` 또는 projectile 여부로 추론하지 않는다.
- 초기 preset은 `melee_front_1`, facing을 쓰지 않는 `ranged_center_3x3`와
  `ranged_center_5x5`다. Profile이 명시적으로 선택하며 직업 하드코딩이 아니다.
- Pattern은 valid tile과 교차하고 battlefield 밖/void를 제외하되 obstacle/blocked tile은
  제외하지 않는다.

### Enemy Ranged Route Attack Policy

원거리 route enemy는 전진하다 target을 포착하면 멈춰 공격하고 release 뒤
`ranged_reposition_ms` 동안 이동을 시도한 다음 기존 target을 재검증한다. 기본값은 1000ms이며
0이면 재이동 없이 즉시 재검증한다.

- 재이동 중 새 target을 찾지 않지만 block, death, route end, battle end와 hard movement lock이
  이를 중단한다. Blocker attack은 재이동과 기존 target보다 우선한다.
- `AttackStart`와 release에서 runtime tile eligibility를 검사하며 release 판정이 damage/projectile
  생성의 최종 gate다.
- Basic projectile은 release 때 확정한 target만 continuous sweep hit 후보로 삼고 bystander에
  충돌하거나 range/usefulness를 재평가하지 않는다.
- Target이 impact 전 죽거나 철수·아군화·air-ineligible이 되면 miss다. Attacker가 죽어도 launch
  owner snapshot으로 hostility를 유지한다.
- 정식 Boss/Elite/특수 unit은 일반 preset fallback 대신 자체 authored range/skill을 가져야 한다.
  현재 live abnormality 30개의 generic missing-pattern 범위는 dedicated range 설계 전 temporary
  placeholder이며 이 정식 authoring 정책의 완료 증거가 아니다. 현재 placeholder 수치는 이번
  정책 정리에서 변경하지 않는다.

### Basic-Attack Range And Skill Reach

Basic attack은 `TileRangePolicy/defense_tile_range`, skill cast는 별도 `reach`를 사용한다.

- Live skill은 `reach: pattern|global`과 `air_capable`을 명시한다.
- Pattern EnemySingle은 `target_range` 또는 preset이 필수고 global은 competing range를
  허용하지 않는다.
- Global은 unit acquisition만 전역으로 만들며 effect area는 `TileArea.footprint`가 소유한다.
  Step-level range/air field와 `WholeFieldValidTiles + TileArea` 혼합은 제거됐다.
- `WholeFieldValidTiles`는 route와 무관하지만 일반 `profile_role: Normal` 침식 직원은 사용할
  수 없다. Special/LegacyEcho와 명시된 특수 unit만 허용한다.
- 전장 전체 후보여도 alive/hostile/air/usefulness와 targeting rule을 적용한다. Skill은 weapon
  profile이나 area 최대화가 아니라 명시 `UnitTargetRule`을 사용한다.
- 일반 DTO는 player preview만 운반한다. Enemy telegraph가 필요하면 실제 피해/효과 tile을 별도
  event/DTO로 보낸다.

### Automatic Hostile Target Usefulness

자동 기본 공격, skill cast, step retarget과 TileArea 시전 가능성은 deterministic expected final
damage가 양수이거나 적에게 실제 hostile state change를 적용할 수 있을 때만 target을 고른다.
AoE는 범위 안에 이런 적이 하나 이상 있어야 하며 damage 가능 대상과 effect-only 대상 사이에
추가 우선순위를 두지 않는다.

Expected damage는 현재 defense/resistance, immunity, fixed damage와 deterministic modifier를
반영하되 crit/miss/random variance는 포함하지 않는다.

Hostile target-applied effect에는 debuff, control, forced movement, DoT, stat/resistance 감소,
vulnerability, mark, aggro 조작과 beneficial-effect 제거가 포함된다. Self/ally buff·heal·shield,
resource/cooldown 변화, presentation-only, projectile/summon 생성 자체와 target state를 바꾸지
않는 global trigger는 포함하지 않는다. `InterruptCast`만 가진 skill은 target이 실제 pending
cast를 가질 때만 유효하다.

Targeting 단계는 이미 적용된 effect의 refresh/stack/resist를 미리 계산하지 않지만 authored
target filter는 따른다.

Skill cast admission과 step execution은 같은 usefulness 분류를 사용하되 집계 범위가 다르다.

- `EnemySingle` 후보는 먼저 cast `reach + target_range`, lifecycle, hostility와 air 조건을 통과한다.
  후보 unit은 direct effect의 실제 대상일 수도 있고 `TileArea` geometry를 정하는 anchor일 수도 있다.
- Cast admission은 skill의 모든 step을 순서와 무관하게 검사하고, 실제 delivery target 집합에 유효한
  executable effect가 있는 step이 하나라도 있으면 허용한다. 첫 hostile step만으로 skill 전체를
  대표하지 않는다.
- `TileArea`의 실제 집합은 `footprint`를 battlefield valid tile과 교차해 off-board/void cell을
  제거한 뒤 `hit_targets`와 `include_caster`를 적용해 만든다. Unit 결과는 UUID 순으로 안정화한다.
  선택 anchor가 footprint에서 제외되면 anchor 자신의 usefulness는 admission 증거가 아니며, 반대로
  anchor 자신에게 효과가 없더라도 affected 집합 안에 유효한 대상이 있으면 admission할 수 있다.
- Hostile delivery의 materialized/filtered 집합이 비었거나 모든 대상에서 effect가 무효면 해당 step은
  unit admission 증거가 아니다. PlayerUnit validation과 automatic unit acquisition은 이 판정을
  공유한다. PlayerTile의 최종 입력은 유효 geometry/reach를 검사하며 빈 타일에도 접수할 수 있다.
- Cast가 admission된 뒤 각 step 실행/no-op은 현재 step의 실제 delivery target 집합만 평가한다.
  단순한 현재 step no-op은 뒤의 유효한 step을 취소·환불·retarget하지 않는다. 직원 사용의 필수 직접
  target lifecycle/관계/air 자격 상실은 별도로 남은 미방출 동작을 취소하며, accepted area anchor에는
  그 규칙을 적용하지 않는다. TargetLocked projectile도 release 전에 current-step 판정을 적용한다.
- Targetless `DirectionalCollision` projectile은 authored range 또는 lifetime bound와 caster facing이 있어
  실행 가능한 경우 기존 targetless admission 예외를 유지한다.

이 계약은 public/RON/save/wire shape를 추가하지 않으며, cast target rule과 deterministic tie-breaker를
바꾸지 않는다.

초기 profile:

- `DefaultForward`: route end까지 남은 진행 거리가 짧은 적 우선
- `AirFirst`: air 우선 후 DefaultForward
- `LowDefenseFirst`: 낮은 defense 후 DefaultForward
- `LowMagicResistFirst`: 낮은 magic resistance 후 DefaultForward

DefaultForward 동률은 먼저 spawn된 unit, 그다음 unit id 순서다.

## Skill Execution Lifecycle

이 장은 cast target 캡처부터 step retarget, cast interrupt와 projectile impact까지 대상 identity가
언제 선택·유지·무효화되는지를 소유한다.

### SkillTarget

`SkillTarget`은 범위가 아니라 기준 대상을 결정한다.

```rust
pub enum SkillTarget {
    SelfUnit,
    EnemySingle { rule: UnitTargetRule },
    CastTarget,
}
```

- `SelfUnit`: 시전자 자신을 기준으로 한다.
- `EnemySingle`: cast의 `reach`/`target_range`와 air eligibility를 통과한 유효 적 중 명시 rule로 하나를 선택한다.
- `CastTarget`: 이전 step 또는 cast-level 기준점을 재사용한다.
- `EnemySingle.rule`은 authoring 필수값이다. `Nearest`도 의도적으로 작성해야 하며 default,
  weapon-profile inheritance, unknown dual-authority field는 허용하지 않는다.

Step은 cast acquisition range를 다시 작성하지 않는다. Area step은 delivery 안에 concrete
`footprint`을 반드시 가진다.

### Skill Cast Selection

`SkillCastSelection`은 `SkillTarget`이 표현하는 effect 대상 의미와 별도로 누가 cast target을 선택하는지
결정한다.

```rust
pub enum SkillCastSelection {
    SelfUnit,      // RON/JSON: self
    AutomaticUnit,
    PlayerUnit,
    PlayerTile,
}
```

Live `SkillCastTargetingDef::Explicit`은 `selection`을 반드시 authoring한다. Runtime/live loader는 target,
첫 step, weapon archetype 또는 request payload에서 이를 추론하지 않는다.

| selection | cast target definition | activation payload | focus |
| --- | --- | --- | --- |
| `self` | `SelfUnit` | 없음; caster 고정 | 없음 |
| `automatic_unit` | `EnemySingle(rule)` | 없음; Core가 authored rule로 선택 | 없음 |
| `player_unit` | `EnemySingle(rule)` | required `Unit` | required PlayerUnit focus |
| `player_tile` | `CastTarget` | required `Tile` | required PlayerTile focus |

다른 조합은 static validation에서 거부한다. Self/AutomaticUnit에 explicit payload를 보내거나 player mode에
다른 target kind를 보내면 `skill_target_type_mismatch`, player payload가 없으면 `skill_target_required`다.

PlayerUnit/PlayerTile activation 순서는 다음과 같다.

1. Caster가 active player combatant, manual activation, Attack stance인지 확인한다.
2. Hard CC/Silence, action activity/lock, equipped skill identity와 full resonance를 확인한다.
3. PlayerUnit은 active hostile lifecycle, air eligibility와 final reach/range를, PlayerTile은 valid battlefield
   tile과 reach/range를 확인한다. PlayerUnit은 any-step actual-delivery usefulness를 추가 확인한다.
   PlayerTile은 빈 타일과 현재 hostile 대상이 없는 geometry도 허용한다.
4. 통과한 Unit identity/Tile과 Unit target의 현재 tile을 acceptance snapshot으로 캡처한다.
5. 직원 공명을 전량 소비하고 해당 action의 사용 자격을 기록한다. 잔량0이어도 그 자격으로 시작한다.

Player mode는 authoritative focus purpose/caster/skill/restore playback만 checkpoint에 저장한다. Read-only
preview는 matching focus에서 candidate를 검증하지만 state/event/revision을 바꾸지 않고 final activation이
같은 validator를 다시 실행한다. 성공은 manual cast start, focus end와 playback restore를 한 revision으로
commit하며 실패는 focus를 포함해 완전 무변경이다. Cancel은 비용 없이 restore playback으로 돌아간다.

Accepted direct PlayerUnit target이 실행 전 죽거나 withdraw/ally/air-ineligible 상태가 되면 남은 사용
동작을 환불 없이 취소하며 다른 unit을 찾지 않는다. Unit 기반 TileArea와 PlayerTile area는 acceptance tile을 계속 사용해
target 이동/소멸이나 occupant 변화가 geometry를 바꾸지 않는다. 어느 경우도 cast/resonance를 refund하거나
rollback하지 않는다. `PlayerUnit` 또는 `PlayerTile` selection과 `RetargetOnStep`의 결합은 strict reject다.
직원 수동 파편은 AutomaticUnit selection도 접수한 유닛에 고정하므로 해당 form의 RetargetOnStep을 거절한다.

Range preview는 selection을 `SelfTarget | AutoUnit | ManualUnit | ManualTile`로 투영한다. Manual unit/tile은
Core가 계산한 cast cell을 제공하지만 hover candidate 자체는 authoritative state가 아니다. Skill catalog는
`cast_target.selection`을 포함하며 현재 version은 `3`이다.

`active_skill.effect_preview_cells`는 기존의 단일 cell collection을 유지한다. SelfTarget처럼 현재 preview
입력만으로 spatial delivery를 확정할 수 있는 cast에서는 모든 `DeliveryDef::TileArea.footprint`를 실제
caster/cast-target 기준 tile과 caster facing으로 투영하고 valid-board clipping한 뒤, `(y, x)` 순서의
deterministic deduplicated union으로 제공한다. 이 계산도 runtime TileArea와 같은 typed tile-reach query를
사용한다. 여러 step의 footprint가 다르거나 겹쳐도 collection을 step별로 나누거나 중복 cell을 남기지
않는다.

`cast_cells`는 acquisition/수동 anchor 선택 범위이고 effect footprint가 아니다. `Instant`와 spatial area가
없는 `Projectile` step은 effect cell을 추가하지 않는다. Preview 입력에 없는 retarget/impact-context가 area
anchor를 결정해야 하면 고정 footprint를 꾸며내지 않고 `RequiresRuntimeTarget`로 unavailable을 반환한다.
Projection은 초기 delivery footprint만 읽으며 target filter, occupant, obstacle, effect 적용, persistent
tracking의 미래 이동을 시뮬레이션하지 않고 battle/cast state를 변경하지 않는다. Public DTO field/tag/nesting과
server/Unity serialization은 그대로이며 per-step preview carrier는 현재 계약에 추가하지 않는다.

### Step Targeting Mode

`SkillStepDef.targeting`은 step 실행 시점의 대상 해석 방식을 결정한다.

```rust
pub enum StepTargetingMode {
    ReuseCastTarget,
    RetargetOnStep,
}
```

기본값은 `ReuseCastTarget`이다.

`ReuseCastTarget`은 스킬 시전 시 확정된 cast-level target을 step 실행에도 재사용한다.

- `SkillTarget::SelfUnit`: 항상 시전자 자신을 step target으로 사용한다.
- `SkillTarget::EnemySingle`: direct unit effect는 cast range를 다시 검사하지 않지만 실행 시점에
  대상이 살아 있는 적이고 cast의 air eligibility를 만족하며 해당 step effect가 여전히 유효해야
  한다. 이 조건을 잃으면 direct target 없이 끝난다. `TileArea`는 선택 시 캡처한 target tile을
  재사용하므로 대상이 죽거나 withdraw되어도 그 tile에서 실행된다.
- `SkillTarget::CastTarget`: cast target이 unit이면 unit, tile이면 tile을 그대로 재사용한다. cast target이 없고 cast anchor tile이 기록되어 있으면 그 tile을 사용한다.

`RetargetOnStep`은 예약된 step이 실행되는 시점에 대상을 다시 고르되 같은 skill cast의
`reach`, `target_range`, `air_capable`을 사용한다. Step-local range/air authority는 없다.

- 후속 타격이 첫 타격 대상에 고정되어야 하면 `ReuseCastTarget`을 사용한다.
- 후속 타격이 현재 전장의 유효 대상 중 새 대상을 찾아야 하면 `RetargetOnStep`을 사용한다.
- `RetargetOnStep`은 이전 cast target을 보정하는 모드가 아니라, step-local targeting을 다시 수행하는 모드다.

현재 live RON의 `RetargetOnStep` 사용은 다음 세 autonomous step이다.

- `little_red_hunt_the_prey.hunt_finish`
- `nothing_there_goodbye.goodbye_finish`
- `funeral_butterfly_eulogy_volley.black_funeral`

모두 후속/조건부 타격이 실행 시점의 유효 대상을 같은 cast-level reach/range/air 계약 안에서
다시 고르는 경우다. 직원 Funeral 파편의 quiet lament와 별도 awakened clone은 접수한 target을 재사용한다.

`SkillDef.cast_targeting`과 `SkillStepDef.targeting`의 책임은 다르다.

- `cast_targeting`: target kind/rule, `reach`, `target_range`, `air_capable`, 최초 캡처 tile을 정한다.
- `target`: step이 원하는 대상 종류를 정한다.
- `targeting`: 그 step이 cast target을 재사용할지, step 실행 시 다시 target을 고를지 정한다.
- `DeliveryDef::TileArea.footprint`: 캡처/impact/caster anchor에서 실제 피격 타일을 계산하는 유일한 footprint다.

제거된 `SkillCastTargetingDef::FirstStepTarget` 같은 첫-step 기반 cast target 추론은 legacy다.
Live RON과 internal tests/fixtures는 cast-level target을 명시한다. 작성 편의 helper는 step 목록을
읽어 추론하지 않고 caller가 target, reach, target range, air capability를 직접 넘기는 explicit
constructor여야 한다.

### Skill Cast Interrupt

스킬 시전 방해는 `Silence`와 분리한다.

- `Silence`: 새 수동/자동 스킬 접수를 막는다. 이미 접수한 직원 사용과 pending cast를 취소하지 않는다.
- `InterruptCast`: 대상의 pending cast, 남은 structured flow와 channel을 취소하고 직원의 미방출 use control을 닫는다.

cast start event-log `seq`가 cast epoch/token이다. core는 `ManualCastStart` 또는 `AutoCastStart`를 기록한 뒤 그 `seq`를 pending cast에 저장한다. 예약된 `ManualCastEnd` 또는 `AutoCastEnd`는 자신의 `cause: Parent { seq }`가 현재 pending cast의 start seq와 같을 때만 스킬을 invoke한다.

`InterruptCast`가 성공하면 대상의 pending cast를 제거하고 `SkillCastInterrupted` event를 기록한다. 이후 예전에 예약된 cast-end event가 도착해도 start seq가 맞지 않으므로 stale event로 무시한다.

시전자 자신이 철수해 lifecycle이 `Withdrawn`으로 바뀌는 경우는 외부 방해 효과가 아니므로 `SkillCastInterrupted`를 쓰지 않는다. core는 pending manual/auto cast와 남은 active cast 제어를 취소하고 `SkillCastCancelled { reason: Withdrawn }` event를 기록한다.

정책:

- 성공한 interrupt는 `interrupted_cast_seq`로 취소된 `ManualCastStart` 또는 `AutoCastStart`를 가리킨다.
- interrupt는 pending cast뿐 아니라 아직 방출되지 않은 flow/추가 공격과 channel을 취소한다.
- 이미 방출한 projectile, 독립 area와 적용한 시간제 buff는 각자의 실제 수명을 유지한다.
- 이미 소비된 resonance/focus/cooldown/resource는 환불하지 않는다.
- pending cast가 없으면 `SkillCastInterrupted` event를 만들지 않지만 남은 flow/channel/use control의 취소는 적용된다.
- interrupted cast는 `ManualCastEnd` 또는 `AutoCastEnd` 완료 event를 만들지 않는다.
- withdrawn으로 취소된 cast는 `SkillCastCancelled`를 만들고, 해당 cast에 대한 `ManualCastEnd` 또는 `AutoCastEnd` 완료 event를 만들지 않는다.
- withdrawn caster의 pending/unreleased 제어와 source-bound `FollowCaster`/`FollowTarget` area runtime은 제거된다.
  structural side branch가 있는 cast는 이미 시작된 delivery의 결과 범위·source 문맥을 정산 완료까지 유지한다.
  이 retained cast 문맥은 취소된 제어를 되살리거나 새 미방출 동작을 허용하지 않는다. 그 밖의 active cast는 제거된다.
  이미 release된 skill projectile과 `GroundFixed` area는 launch-time snapshot으로 계속 resolve하며, 이미 event
  log에 기록된 과거 event는 수정하지 않는다.

### Projectile Target Lifecycle

게시 전 delivery contract에서 모든 projectile의 `speed_units_per_ms`는 양수이고, 존재하는
`max_range_tiles`, `max_lifetime_ms`, `max_kills`, `max_pierces`, `collision.max_hits`도 양수여야
한다. Skill은 TargetLocked/DirectionalCollision을 각 정책에 맞게 사용할 수 있지만, 기본 공격과 weapon
profile은 runtime이 실행하는 기본 TargetLocked projectile의 속도만 허용한다. Targetless, directional,
range/lifetime/kill/pierce stop, nondefault collision과 TileArea를 기본 공격에 저작하면 무시하지 않고
객체/필드 경로가 있는 publication error로 거부한다.

기본 공격 projectile은 locked target을 가진다. impact 시점에 locked target이 죽었거나, 철수했거나, 더 이상 적대적 유효 대상이 아니거나, 공중 타격 조건을 만족하지 못하면 `BasicAttackProjectileImpacted { hit: false }`를 기록하고 damage를 적용하지 않는다. 별도 `ProjectileMiss` event는 만들지 않는다.

스킬 projectile은 step/delivery 정책에 따라 대상 또는 타일을 가진다. impact 시 유효한 첫 피격 유닛이 없으면 `SkillProjectileImpacted { first_hit_unit_id: null }`로 끝난다. 이 이벤트가 스킬 projectile의 no-hit/miss 표현이며, Unity는 별도 miss event를 기대하지 않는다.

Directional skill projectile은 accepted impact의 effect를 적용한 뒤 kill stop을 평가한다. 누적
`max_kills`에 도달한 같은 impact가 active projectile을 제거하고 owning step delivery를 정확히 한 번
terminal resolve한다. `max_hits` 또는 lifetime expiry가 같은 시각에 함께 성립해도 하나의 terminal
resolution으로 합성하며, 그 뒤의 collision 후보에는 impact/effect를 만들지 않는다. Append-only
`SkillProjectileImpacted.terminal`은 effect 전 queue 시점의 motion/collision 종료 사실을 유지하고,
post-effect kill-stop outcome은 cast delivery lifecycle에서 합성한다. 과거 event를 수정하지 않는다.

이미 release된 projectile은 전술 이동으로 취소하거나 재발사하지 않는다. Homing은 impact 전 매 tick live
target body를 다시 향하고 Fixed는 launch 때 확정한 aim을 유지한다. 따라서 target의
`TacticalRelocateUnit` 또는 `TacticalSwapUnits`는 Homing의 목적 좌표만 바꾸며 Fixed의 원래 궤적/좌표는
바꾸지 않는다. Swap은 두 participant의 released delivery runtime을 제거하거나 서로 교환하지 않는다.

## Area Delivery

이 장은 unit acquisition과 분리된 `TileArea` footprint, anchor, affected tile과 deferred area 정책을
소유한다.

### TileArea

다중 피격은 DefenseRoute에서 `DeliveryDef::TileArea`를 사용한다.

```ron
SkillStepDef(
    id: "example_sweep",
    target: EnemySingle(rule: Nearest),
    delivery: TileArea(
        footprint: (include_anchor_tile: true, rows: ["XXX", "X@X", "XXX"]),
        area: (
            anchor: CastTarget,
            tile_origin: Anchor,
            hit_targets: Enemies,
            include_caster: false,
        ),
    ),
    effects: [
        Damage(amount: 35, damage_type: Magic),
    ],
)
```

`TileArea.footprint`이 actual affected-tile shape다. Global reach와 footprint는 서로 대체하지 않는다.

`TileArea.area`는 anchor/filter/tick 정책을 담당한다. `CastTarget` anchor는 선택 순간 캡처한
tile을 사용하고 `tile_origin: Anchor`가 필수다. Target/impact-centered footprint는 explicit
orientation 정책이 아직 없으므로 rotation-invariant여야 하며 facing 없는 stationary caster도
동일한 타일을 만든다.

- `anchor`: event-log center, impact context, VFX 기준점, 또는 `tile_origin: Anchor`가 참조할 위치를 결정한다.
- `tile_origin`: `footprint`의 `@`를 caster 또는 resolved anchor 중 어디에 놓을지 결정한다.
- `tracking`: 지속 영역이 시전자/대상/지면 중 무엇을 따라갈지 결정한다.
- `hit_targets`: 범위 안에서 아군, 적, 전체 중 어떤 유닛을 맞출지 결정한다.
- `include_caster`: 시전자가 같은 타일 필터에 걸릴 때 포함할지 결정한다.
- `tick_policy`, `duration_ms`, `tick_interval_ms`: 지속 범위 스킬의 tick 정책을 결정한다.

전술 이동 뒤 `GroundFixed` persistent area는 생성 때 저장한 geometry를 유지한다. `FollowCaster`와
`FollowTarget`은 각각 이동한 caster/target의 live body에서 다음 tick geometry와 포함 대상을 다시 계산한다.
Relocation 자체는 이미 release된 area runtime, tick cadence 또는 source damage snapshot을 취소/재생성하지
않는다.

`ImpactContext`/`ImpactContextStart` 앵커는 이전 projectile 또는 tile area delivery가 만든 impact context를 재사용한다. 이전 spatial delivery가 없는 step에서 사용하면 잘못된 RON으로 간주한다. Cast target tile은 selection 때 저장되므로 대상 소멸은 area anchor 소멸이 아니다.

기본 공격은 `TileArea`를 사용하지 않는다. 기본 공격의 `defense_tile_range`는 DefenseRoute에서 단일 공격 대상 후보를 찾는 사거리 패턴이며, 실제 다중 피격은 스킬 step의 `DeliveryDef::TileArea`만 사용한다.

## Projection And Authoring Boundaries

이 장은 targeting 결과를 event/preview로 전달할 때의 의미, 제거된 schema와 새 range 작성 규칙을
소유한다. 정확한 JSON shape와 message order는 외부 Unity canonical을 따른다.

### Event Log / Unity 표시 계약

DefenseRoute에서 runtime `SkillAreaDeclared` event는 core가 확정한 `tile_pattern.affected_tiles`를 전달한다. Unity는 이 타일을 범위 경고/효과 표시에 사용하고, 배치/선택/스킬 버튼 preview는 `range_previews`의 최종 cell DTO를 사용한다. Circle/Line/Box/Rectangle/Cone을 재해석하거나 authored `defense_tile_range`에서 표시 타일을 추론하지 않는다.

정확한 event JSON shape와 message ordering은 외부 canonical `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`가 소유한다. 이 문서는 어떤 타일이 판정 대상이 되는지와 range/targeting 의미만 소유한다.

### 레거시

다음 계약은 DefenseRoute 공식 live path에서 제거됐다.

- `DeliveryDef::Area(shape: Circle/Line/Box/Rectangle/Cone)` 기반 공식 스킬 판정.
- `SkillTarget` 안에 범위형 아군/적 variant를 두는 방식.
- `target.area`와 `delivery.area.shape`가 동시에 범위를 말하던 이중 source of truth.
- geometric AoE 동작을 고정하는 legacy compatibility test.

`DeliveryDef::Area`, `SkillAreaDeliveryDef`, `SkillAreaShapeDef`, step range fields, 그리고
`WholeFieldValidTiles + TileArea` 혼합 계약은 schema/runtime에 없다.

### 작성 규칙

- 단일 타겟 스킬: core는 cast `reach`/`target_range` 안의 유효 대상 중 하나를 선택한다.
- 범위 스킬: core는 `TileArea.footprint`가 산출한 valid tile 안의 authored hit-filter 대상에게 적용한다.
- live RON cast pattern은 `target_range` 또는 `target_range_preset`을 사용하며 둘을 동시에
  작성하면 실패한다. TileArea footprint는 delivery 안에 concrete pattern으로 존재한다.
- `ModifyDamage`는 같은 step의 `Damage`에만 적용되는 step-local modifier다. `Damage` 없는 step에는 작성하지 않는다.
- 새로운 범위 모양이 필요하면 geometric shape를 되살리지 말고 `TileRangePattern.rows`를 추가한다.

## Authored effect and shared action boundary

Each step still owns target, delivery, condition, repeat and presentation. Its
`effects` now contains explicit `(id, tags, origin, op)` authoring records. Only
`op` determines execution. Effect-level amount edits never alter those shared
carrier fields; unsupported carrier edits return the action and affected effect
addresses. A Damage step and a separate SelfUnit Heal step therefore keep their
independent targets and payloads. Preview continues to read the same compiled
step carrier and SkillId publication.

ModifyDamage is still folded before all Damage operations in its step. Signed
Heal, HP-loss gates, pending/terminal step resolution and queued-step cancellation
retain their prior semantics. The metadata wrapper neither duplicates a projectile
nor changes hit counts or released-projectile/fixed-area lifetime. Concrete RON
and headless support/diagnostic details are in `data_loading_contract.md`.

`DamageExpr`/`HealExpr` reuse the containing step's target and delivery. The validated
winning expression executes through the same Damage/Heal kernel as its literal counterpart.
Snapshot captures release-time inputs; Live reads the admitted effect-consumption frame.
All targets of one effect are evaluated before that effect mutates any target. A following
effect can read the committed prefix, including actual mitigated HP loss. Unsupported
context/member-stat/owner combinations fail publication or execution with an exact site;
there is no literal or zero fallback. Public capability and grammar are owned by
`data_loading_contract.md`.


## Action progress and actual effect settlement

Runtime action observation records Accepted after existing target qualification, Released after
successful invocation, control terminal, and finite EffectsSettled separately. A dispatched
projectile/area step can finish control while its released effects remain. Queued buff application
and explicitly scheduled finite extra attacks also delay settlement; the independent applied buff
lifetime does not. A new repeat/impact/area tick has a new per-target strike, while multiple Damage
effects within one iteration share that strike. Actual HP-loss gates still use their previous
command result, not the new distinct-strike aggregate.

Authored zero Damage still skips command creation. A piercing projectile that had an earlier hit
and then expires without a new final-interval impact closes its native carrier obligation. This
completion closes native pending delivery state without synthesizing damage or resolving an
unsatisfied damage gate. Cleanup polls already-ready deferred work and settles the cast when
its actual finite obligations finish. Withdrawal, HardCC, InterruptCast and death keep their existing distinct
cleanup scopes. For autonomous nonemployee casts, death can leave a pending CastEnd that resets
resonance without a Released milestone. Employee death cancels the accepted use without refund;
its cost was already recorded at admission. See `core_runtime_contract.md` for source, handle, failure and lifetime
ownership. These additions do not alter targeting, repeats, formula, RNG or current carrier policy.

## Typed resource effects (R05)

`Resources(operations: [...])` resolves resource owners separately from skill target selection.
`SourceEmployee` uses the actual caster's persistent employee UUID, `SourceEquipment` uses the
captured persistent equipment-copy UUID, and `TargetEmployee` uses each selected target's actual
employee UUID. A definition UUID, runtime unit ID or runtime item ID is not a persistent owner.
Unknown/missing owners and selector/definition kind mismatches fail closed.

Source operations execute once per authored effect even when multiple targets are selected.
TargetEmployee operations expand over the actual target list. The expanded ordered batch must fit
the resource bound and commits atomically with the enclosing command/timestamp. A later insufficient
cost restores earlier resource changes, queued work, RNG/UUID state and observations in that candidate.
Resources alone do not establish hostile target usefulness; the existing targeting and eligibility
rules continue to select actual payload recipients.

Activation, Delivery and Chain are distinct capabilities. Instant payloads have a finite Delivery;
area ticks/projectile impacts continue their released carrier's Delivery. A carrier may finish its
root while an admitted completion still holds that same scope. Cancelling unreleased control work
does not cancel an already released fixed payload. A closed Activation cannot be reused by its later
tick. Child actions preserve the actual causal Chain across trigger source replacement and release
their own hold at completion. Public source observations omit private resource capabilities.

The full owner/lifetime/Run save contract is in [core runtime contract](core_runtime_contract.md#typed-resources-r05).
Implementation, test evidence and pending validation/audit remain owned by [R05 execution](goals/content_authoring_resource_lifetimes/execution/single_goal/EXPERIMENTS.md).


## R07 fragment source authority

선택된 active 사본·형태의 실제 skill이 manual cast source다. FragmentSourceSnapshot(copy UUID, definition ID, form ID, skill ID)을 cast와 delivery가 공유하며 amount preview/explain 및 trace가 같은 출처를 노출한다. SourceFragment resource selector는 runtime actor에 바인딩된 동일 source capsule의 FragmentCopy owner lease만 허용한다. 형태 변경은 기존 delivery를 다시 해석하지 않는다. 평타/intrinsic 또는 장비 provider 효과를 active 파편의 효과로 귀속하지 않는다.

## U06 ally와 기본 Heal

`AllySingle`은 명시 아군 selector다. Heal은 살아 있는 아군 중 현재 부족 HP가 있는 대상만 유용하며 cast
range/air capability를 동일하게 검사한다. `player_unit`은 skill focus와 exact unit payload가 필요하다.
접수 후 direct 대상이 무효면 재선택하거나 부활하지 않는다. 선택 tile 영역은 기존 geometry 계약을 유지한다.
Weapon 기본 Heal은 `Instant`만 허용하고 Damage/proc를 발생시키지 않는다. footprint 유무가 단일/영역을
결정하며 현재 저지 관계의 적대 기본 공격 범위 예외가 Heal이나 active skill로 확장되지 않는다.

## U07 object placement preview

`CreateObject`의 CastTile은 PlayerTile/CastTarget, SourceTile은 SelfUnit과 결합한다. 각 offset은 실제
battlefield 지형·예약·점유·현재 사거리·source/copy별 상한을 만족해야 한다. preview는 현재 상태에서 예상
정의/진화 ID·위치·HP·HP 비용 또는 top 분리량을 반환한다. 앞선 step의 미래 효과를 미리 실행하지 않는다.
실제 시전은 같은 자격 검사를 쓰고 방출 시 다시 확인한다. 접수 이후 점유·HP·pool 변화로 생성이 실패해도
접수 공명을 환불하지 않으며 별도 생성 비용은 부분 반영하지 않는다. 예산 fault는 기존 sticky rollback이다.

Trap에는 unit 선택/Heal 권한이 없고 Wall에는 Heal·직원 명령 권한이 없다. Wall의 기본 공격 범위 예외는
실제 적대 지상 이동 접촉 관계에 한정하며 active skill의 사거리 예외를 만들지 않는다.
