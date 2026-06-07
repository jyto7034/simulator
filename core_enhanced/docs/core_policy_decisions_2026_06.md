# Core Policy Decisions 2026-06

이 문서는 최근 논의로 새로 확정된 core 구현 정책을 한 곳에 모은다. 기존 `game_rulebook.md`, `skill_target_contract.md`, Unity 계약 문서를 대체하지 않고, 구현 전 확인용 결정 목록으로 사용한다.

Unity-facing 계약/구현 문서의 canonical 위치는 이 저장소의 `docs/`가 아니라 `F:\unity projects\ark\docs`다. Linux/WSL 경로로는 `/mnt/f/unity projects/ark/docs`다. 특히 `unity_core_contract.md`와 `unity_client_implementation_goal.md`는 외부 canonical 문서를 확인하고 갱신한다.

## 구현 원칙

- 정책의 최종 source of truth는 core/server다.
- Unity는 core가 내려준 상태, 경고, 실패 사유, 표시 태그를 보여준다.
- Unity는 방어력, 마법 저항, 타겟팅, 파편 호환성, 피해 피드백을 임의로 재계산하지 않는다.
- 세부 수치 공개보다 전투 중 관찰, 피해 숫자 색상, 짧은 라벨, 노드 경고문으로 플레이어가 상황을 이해하게 한다.

## 이상현상 시도와 퇴각

전투 노드는 단순 전투 장소가 아니라 이상현상이다. 탐사팀은 이상현상에 진입해서 임무를 해결하고 빠져나온다.

- `ConfirmEnterNode`는 전투 노드를 즉시 소비하지 않는다.
- 전투 노드에 진입하면 해당 이상현상의 진입 시도 1회가 시작된다.
- 같은 이상현상은 최대 3번까지 진입 시도할 수 있다.
- `BattleEnd`가 발생하기 전까지 후퇴할 수 있다.
- 모든 아군 유닛이 전투불능이어도 `BattleEnd`가 아직 확정되지 않았다면 후퇴할 수 있다.
- 일시정지 중, 스킬 시전 중, 웨이브 진행 중에도 후퇴할 수 있다.
- `BattleEnd` 이후 결과 처리 상태에서는 후퇴할 수 없다.
- 후퇴하면 현재 진입 시도 1회를 소모한다.
- 남은 시도가 있으면 노드는 소비되지 않고 Safezone/NodeConfirm으로 돌아간다.
- 퇴각 없이 임무 실패 조건이 확정되면 이상현상 노드는 즉시 소비된다.
- 3번의 시도가 모두 사용되면 이상현상은 스스로 사라지고 노드는 소비된다.
- 비보스 DefenseRoute 실패는 즉시 런 실패가 아니다.
- 보스 환상체 전투 패배는 즉시 런 실패다.

재진입 시 초기화/유지:

- 적 HP는 초기화한다.
- 웨이브 진행도는 초기화한다.
- 보호 목표 피해는 초기화한다.
- 퇴각 전에 이미 발생한 전투불능, 트라우마, Run HP 손상은 유지한다.
- 사용된 소비 아이템은 환불하지 않는다.
- active consumable modifier는 같은 이상현상 재진입 동안 유지된다.
- 전투 중 관찰한 위협 정보는 같은 이상현상 재진입 preview에서 유지할 수 있다.
- 구체적인 관찰 정보 저장 범위와 전투 기록 UI는 유보한다.

## 소비 아이템 Modifier

소비 아이템은 전투 진입 시점이 아니라 사용 명령 성공 시점에 소비된다.

- `use_consumable_item`은 `ViewingMap`과 `NodeConfirm`에서만 허용한다.
- Safezone `Item Use`에서 사망자가 아닌 직원에게 드래그앤드롭하면 `use_consumable_item` command를 보낸다.
- command 성공 순간 owned consumable을 inventory에서 제거한다.
- 대상 직원의 `active_consumable_modifier`를 갱신한다.
- 사용된 아이템은 노드 취소, 퇴각, 재진입, modifier 덮어쓰기 상황에서도 환불하지 않는다.
- 직원 1명은 active consumable modifier를 1개만 유지한다.
- 새 아이템을 쓰면 기존 modifier는 환불 없이 덮어쓴다.
- duration은 `NextCombatNode` 또는 `CombatNodes(n)`이다.
- duration은 전투/보스 노드가 해결될 때만 감소한다.
- 같은 이상현상에서 퇴각 후 재진입하는 것은 새 전투 노드 해결로 보지 않는다.
- 남은 시도 횟수가 있는 한 적용된 modifier는 같은 이상현상 재진입에도 유지된다.
- 성공, 퇴각 없는 실패, 3회 시도 소진으로 이상현상이 사라지면 해당 전투 노드가 해결된 것으로 보고 duration을 감소시킨다.

## 노드 미리보기와 위협 경고

세부 적 스탯 숫자나 도감은 만들지 않는다. 노드 진입 전에는 위협 경고문으로 대응 방향만 알려준다.

`CombatPreview`는 단순 `threat_warning_tags` 배열이 아니라 `threat_warnings` 객체 배열을 내려준다.

```text
ThreatWarning
- tag: ThreatWarningTag
- status: Unverified | Disproved
- source: Briefing | Rumor
```

초기 tag 후보:

| tag | 표시 예시 |
| --- | --- |
| `armored_enemy_possible` | 장갑형 적 출현 가능 |
| `high_magic_resist_enemy_possible` | 마법 저항이 높은 적 출현 가능 |
| `air_enemy_possible` | 공중 적 출현 가능 |
| `hard_to_block_enemy_possible` | 저지하기 어려운 적 출현 가능 |
| `shielded_enemy_possible` | 보호막 적 출현 가능 |
| `regenerating_enemy_possible` | 재생 적 출현 가능 |
| `fast_breakthrough_enemy_possible` | 빠른 돌파 적 출현 가능 |

루머성 오경고:

- 최초 진입 전 preview에는 실제 경고와 낮은 확률의 루머성 오경고가 섞일 수 있다.
- 루머성 오경고는 1개까지만 추가한다.
- 루머성 오경고 확률은 우선 20%로 둔다.
- 오경고 문구는 실제 resolved spawn wave 경고에 없는 사전 정의 위협 경고 후보에서 seed 기반으로 선택한다.
- 같은 이상현상에서는 같은 seed 결과를 사용해 재현 가능해야 한다.
- Unity는 오경고를 직접 랜덤 생성하지 않는다.

상태:

- `Unverified`: 아직 관찰로 확인되지 않은 경고.
- `Disproved`: 첫 진입 후 퇴각/재진입 시 core가 false rumor로 판정한 경고.
- `Disproved` 경고는 같은 이상현상 재진입 preview에서 취소선 처리한다.
- 실제 경고는 `Unverified`로 유지한다.

## 피해 타입과 AD/AP

피해 타입은 빌드 색깔과 효율 차이를 만드는 축이다. 클리어 가능/불가능을 잠그는 열쇠로 쓰지 않는다.

- AD는 `Physical`에 대응한다.
- AP는 `Magic`에 대응한다.
- `True`는 고정 피해다.
- `Physical` 피해 숫자는 붉은 계열로 표시한다.
- `Magic` 피해 숫자는 푸른 계열로 표시한다.
- `True`는 별도 고정 피해 색상 또는 neutral/white 계열로 표시한다.
- 일반/정예 적이 특정 피해 타입을 사실상 영구 무효화해 반대 타입 딜러 보유 여부만 검사하는 구조는 피한다.
- 고방어/고마법저항 적은 등장할 수 있지만, 우회 수단을 열어둔다.
- 우회 수단 예시는 관통, 저항 감소, 고정 피해, 제어, 저지, 소비 아이템, 배치/스킬 타이밍이다.
- 보스는 일시 보호막, 기믹 방어, 특정 타이밍 무효를 가질 수 있다.
- 현재 core 기준으로 defense 또는 magic_resist가 50 이상인 preview enemy는 각각 `armored_enemy_possible`, `high_magic_resist_enemy_possible` 브리핑 경고 대상이다.
- 현재 피해 공식은 `minimum_damage > 0`인 피해 요청에서 방어/마법저항 스탯만으로 영구 면역을 만들지 않는다.
- 보스도 영구 단일 타입 면역으로 빌드 전체를 부정하지 않는다.

## 피해 피드백

`HpChanged`에 `feedback_tags`를 추가한다. core가 계산해서 내려주고 Unity는 재계산하지 않는다.

초기 `feedback_tags`:

| tag | 표시 위치 | 표시 예시 | core 판정 기준 |
| --- | --- | --- | --- |
| `critical` | 상단 | 치명타! | 피해 결과의 `critical == true` |
| `mitigated` | 상단 | 경감됨! | `raw_damage > 0`, `final_damage > 0`, `final_damage <= raw_damage * 60%` |
| `fixed_damage` | 상단 | 고정 피해! | 주 피해 타입이 `True` |
| `immune` | 상단 | 면역! | `raw_damage > 0`인데 최종 피해가 0이거나, 상태이상이 완전히 막힘 |
| `piercing` | 좌측 | 관통 | 해당 피해 타입의 관통/저항 관통 modifier가 실제 저항값을 낮춤 |
| `shield` | 좌측 | 보호막 | 피해가 실체 HP보다 보호막/방어막에 먼저 적용됨 |
| `blocked` | 좌측 | 상쇄 | 일회성 방어, 패링, 상쇄 효과가 피해 일부 또는 전부를 지움 |
| `resisted_status` | 좌측 | 저항 | 상태이상이 들어갔지만 지속시간/효과가 감소함 |

표시 규칙:

- 피해 숫자 상단 라벨은 `critical`, `mitigated`, `fixed_damage`, `immune`만 사용한다.
- `weakness` 상단 태그는 초기 계약에서 사용하지 않는다.
- 피해 숫자 좌측 라벨은 그 외 보조 태그에 사용한다.
- 상단 태그가 여러 개면 메인 상단 라벨 1개만 표시한다.
- 상단 우선순위는 `immune > fixed_damage > mitigated > critical`이다.
- `feedback_tags`는 연출/표시 계약이다.
- 피해량, 사망 여부, HP 변화의 source of truth는 `HpChanged.delta`, `hp_before`, `hp_after`, `final_damage`다.

## 무기 아키타입

직원은 고정 직업을 갖지 않는다. 장착한 무기가 현재 전투 역할을 결정한다.

무기 전투 프로필:

```text
range_role: Melee | Ranged
weapon_archetype: Sword | Spear | Shield | Bow | Gun | GrenadeLauncher | Staff | ...
damage_type: Physical | Magic | True
targeting_profile: DefaultForward | AirFirst | LowDefenseFirst | LowMagicResistFirst | SplashClusterFirst
air_capable: bool
defense_tile_range: TileRange
capability_tags: [...]
```

초기 추천 아키타입은 7종이다.

| range_role | archetype | 역할 | 기본 타겟팅 방향 |
| --- | --- | --- | --- |
| Melee | `Sword` | 표준 근접 딜러 | 저지 대상 우선, 없으면 `DefaultForward` |
| Melee | `Spear` | 전방 긴 사거리 근접 | 전방 범위 안 route progress 높은 적 |
| Melee | `Shield` | 저지/생존 | 저지 대상 고정 우선 |
| Ranged | `Bow` | 표준 원거리/대공 | `AirFirst` |
| Ranged | `Gun` | 정밀 원거리 | `LowDefenseFirst` 또는 `DefaultForward` |
| Ranged | `GrenadeLauncher` | 폭발/광역 | `SplashClusterFirst` |
| Ranged | `Staff` | AP/마법 원거리 | `LowMagicResistFirst` |

후속 후보:

- `Axe`: 느린 고화력 근접 또는 소규모 광역.
- `Crossbow`: 느린 정밀/관통 원거리.
- `Shotgun`: 짧은 원거리, 전방 확산.

무기 아키타입은 실제 아이템 이름이 아니라 전투 기능 단위다. 예를 들어 여러 검 아이템은 서로 다른 이름과 스탯을 가질 수 있지만 core의 기본 전투 역할은 `Sword`로 처리한다.

## 타겟팅 프로필

타겟팅 규칙은 무기 아키타입에 직접 하드코딩하지 않고 별도 `targeting_profile`로 분리한다.

초기 프로필:

| profile | 규칙 |
| --- | --- |
| `DefaultForward` | 방어 목표/route 종료 지점까지 남은 route가 가장 짧은 적 우선 |
| `AirFirst` | 공중 적 우선, 없으면 `DefaultForward` |
| `LowDefenseFirst` | 방어력이 가장 낮은 적 우선, 동률이면 `DefaultForward` |
| `LowMagicResistFirst` | 마법 저항이 가장 낮은 적 우선, 동률이면 `DefaultForward` |
| `SplashClusterFirst` | 직접 대상 주변 splash 기대값이 큰 대상 우선, 동률이면 `DefaultForward` |

세부 규칙:

- `DefaultForward`는 직선거리 기준이 아니라 route progress 기준이다.
- 같은 route 안에서는 route progress가 큰 적을 우선한다.
- 여러 route가 섞이면 각 적의 현재 route에서 종료 지점까지 남은 진행 거리를 비교한다.
- 계산 불가능하거나 동률이면 spawn 순서, 그다음 unit id 순서로 처리한다.
- 근거리 기본 공격은 저지 중인 적을 최우선으로 한다.
- 근거리 유닛이 저지 중인 적이 없을 때만 무기 `targeting_profile`을 적용한다.
- 원거리 기본 공격은 무기 `targeting_profile`을 기본 source of truth로 삼는다.
- 스킬이 별도 타겟팅을 명시하면 스킬 타겟팅을 우선한다.
- 스킬이 별도 타겟팅을 명시하지 않으면 장착 무기의 기본 `targeting_profile`을 따른다.

## 스킬 파편 장착 조건

스킬 파편은 직원 고정 직업이 아니라 장착 무기/전투 프로필을 기준으로 장착 가능 여부를 판단한다.

조건 축:

- `range_role`
- `weapon_archetype`
- `damage_type`
- `targeting_profile`
- `air_capable`
- `block_capacity_min`
- `required_capability_tags`
- `incompatible_tags`

정책:

- 근거리 전용 파편과 원거리 전용 파편은 허용한다.
- 모든 파편을 모든 무기 아키타입에 억지로 대응시키지 않는다.
- 유명하거나 스킬 파편으로 녹여내기 좋은 환상체 파편은 특정 아키타입 또는 아키타입 그룹 전용으로 설계한다.
- 환상체 정체성이 약하거나 보조/안정화 역할이 강한 파편은 범용 파편으로 둔다.
- 아키타입 제한은 벌칙이 아니라 빌드 목표다.
- 플레이어는 특정 파편을 쓰기 위해 맞는 무기를 준비하고, 무기 교체로 직원 역할을 바꾼다.
- 최종 장착 가능 여부는 core validation이 source of truth다.

## 유닛 겹침과 저지

아군 배치 겹침은 금지하지만, 전투 중 이동 유닛의 좌표 겹침은 허용한다.

- 아군 배치 위치 겹침은 금지한다.
- `DeployUnit` 시점에 이미 점유된 배치 좌표는 `position_occupied`로 거절한다.
- 적끼리 좌표 겹침은 허용한다.
- 적과 아군 좌표 겹침은 허용한다.
- 저지 중인 유닛끼리도 좌표상 겹칠 수 있다.
- 유닛 간 물리 충돌은 길막과 저지의 source of truth가 아니다.
- 저지와 통과 여부는 `block_state` 같은 명시 전투 상태가 결정한다.
- Unity fan-out, sorting, 체력바 offset은 표시 전용이다.
- Unity 표시 보정은 실제 판정 좌표, 저지, 공격 범위, 피해 판정을 바꾸지 않는다.

저지 규칙:

- 저지는 `block_radius` 기반 논리 상태로 판정한다.
- `block_capacity`가 남은 지상 직원이 `blockable` 적을 반경 안에서 붙잡는다.
- 동시 저지 우선순위는 route progress가 큰 적 우선이다.
- route progress 동률은 먼저 spawn된 적, 그래도 동률이면 unit id 순서로 처리한다.
- 여러 지상 직원이 같은 적을 저지할 수 있으면 가장 가까운 직원이 저지한다.
- 가장 가까운 직원도 동률이면 deterministic id 순서로 처리한다.
- 저지 용량을 초과한 적은 해당 직원을 통과한다.
- 저지된 적은 저지자를 우선 공격한다.
- 저지자는 자신이 저지한 적을 우선 공격한다.
- 저지는 적 사망, 저지자 사망/전투불능, 강제 이동/넉백, `block_radius` 이탈 시 해제된다.

## Movement Backend와 Rapier 책임

Rapier/backend physics는 blocking, targetability, airborne terrain immunity, deploy occupancy, route progress, damage/hit decision의 source of truth가 아니다.

Rapier는 지상 유닛의 terrain/static obstacle correction helper로만 사용한다.

movement backend 정책:

- movement input은 유닛별 movement/collision policy를 명시해야 한다.
- `Ground` 유닛은 static obstacle, walkable/void tile, board bounds 정책을 받는다.
- `Airborne` 유닛은 static obstacle, walkable/void tile, terrain collision correction을 받지 않는다.
- `Airborne` 유닛도 board 밖으로 나가지 않는다.
- 유닛 collider는 movement/blocking source of truth가 아니다.
- 유닛 간 겹침 허용 정책을 Rapier collision으로 되돌리지 않는다.
- block_state가 저지와 통과 여부를 결정한다.
- authored route를 obstacle local avoidance가 조용히 우회하게 만들지 않는다. 우회가 필요하면 명시 policy와 tests를 둔다.
- board bounds는 clamp 또는 Rapier wall 중 하나를 source of truth로 명확히 한다.

## 공중 적

공중 적은 명일방주 드론처럼 지형지물의 영향을 받지 않는 별도 이동 계층이다.

`mobility_kind`가 source of truth다.

```text
Ground
- 지형/장애물/walkable tile/저지 규칙을 받는다.
- blockable 설정 가능.

Airborne
- route waypoint/polyline은 가진다.
- 지형지물, 장애물, walkable tile 여부, 지상 충돌의 영향을 받지 않는다.
- 직선 비행으로 시작점에서 도착점까지 날아가지 않고, authored route waypoint/polyline은 따른다.
- route segment를 따라 이동하되 route 끝을 넘어가지 않는다.
- route progress와 endpoint 도달 판정은 가진다.
- route 끝은 보호 목표를 기본 공격 사거리 안에 둘 수 있어야 한다.
- 지상 배치 타일 점유나 통행 점유에 영향을 주지 않는다.
- 저지되지 않는다.
- 대공 가능 단일 공격/스킬로만 단일 타겟팅 가능하다.
- 광역 스킬에는 지상/공중 구분 없이 피격된다.
```

초기 구현에서는 `target_traits: [Airborne]` 같은 중복 데이터는 만들지 않는다. 필요하면 runtime/snapshot에서 `mobility_kind == Airborne`으로 표시용 trait를 파생한다.

공중 적 이동/공격 규칙:

- 공중 적은 route waypoint/polyline을 따라 이동한다.
- 이동 중 공격 사거리 안에 대상이 들어오면 이동을 멈추고 기본 공격을 수행한다.
- 공격이 끝나면 다음 기본 공격 가능 시점까지 이동한다.
- 다음 공격 가능 시점에 사거리 안 대상이 있으면 다시 공격한다.
- 사거리 안 대상이 없으면 공격하지 않고 route를 따라 계속 이동한다.
- route 끝에 도달하면 그 위치에서 대기한다.
- route 끝에서 다음 공격 가능 시점마다 사거리 안 대상을 다시 찾는다.

공중 적 기본 공격 대상:

1. 사거리 안 보호 목표.
2. 사거리 안 살아있는 player combat unit 중 가장 가까운 대상.
3. 거리 동률이면 deterministic unit id 순서.

보호 목표가 여러 개로 확장되면 거리 우선, 동률이면 deterministic id 순서로 처리한다.

이 보호 목표 우선순위는 공중 적 기본 공격 전용 정책이다. 지상 적 기본 공격/저지 타겟팅은 기존 지상 규칙을 따른다.

대공/피격 정책:

- 기본 공격과 단일 스킬은 공격/스킬 정의에 대공 가능 플래그가 있어야 공중 적을 후보로 삼는다.
- 대공 불가능한 단일 공격은 공중 적을 target candidate에서 제외한다.
- persisted/current target도 공격/스킬 해결 시점마다 대공 가능 여부를 다시 검증한다.
- 광역 스킬은 지상/공중 구분 없이 피격시킨다.
- 광역 스킬의 범위 판정은 공중 유닛의 2D projected world/tile position을 기준으로 한다.
- 기본 공격 splash/폭발이 별도 광역 delivery로 확장되기 전까지, 기본 공격은 단일 공격의 대공 규칙을 따른다.
- `Airborne` enemy route endpoint가 보호 목표를 기본 공격 사거리 안에 둘 수 없으면 data validation에서 거부한다.
- `CombatPreview`는 공중 적 등장 가능성을 `air_enemy_possible` warning으로 표현할 수 있다.
- Unity는 공중 판정/저지/피격/타겟팅을 재계산하지 않고 core DTO를 표시한다.

## 구현 우선순위 제안

정책이 넓기 때문에 한 번에 모두 구현하지 않는다. core 변경은 아래 순서를 권장한다.

1. `HpChanged.feedback_tags` 추가와 피해 피드백 계산.
2. `CombatPreview.threat_warnings` DTO 추가.
3. 이상현상 attempt state와 retreat 처리.
4. consumable modifier duration/재진입 유지 정책 정리.
5. 유닛 겹침/저지 runtime 정리.
6. movement backend/Rapier 책임 정리와 유닛별 movement policy 도입.
7. 공중 적 mobility/저지/피격/타겟팅 정책 도입.
8. 무기 전투 프로필과 targeting profile 도입.
9. 스킬 파편 장착 조건 validation 도입.
10. AD/AP 하드 게이트 방지 validation 도입.

각 단계는 Unity-facing snapshot/command 계약 테스트를 함께 갱신한다.
