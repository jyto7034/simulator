# Game Rulebook

이 문서는 현재 게임 규칙의 최상위 룰북이다. 플레이어가 어떤 흐름으로 런을 진행하고, 각 노드와 전투가 어떤 의미를 가지며, 보상과 손실이 언제 확정되는지를 설명한다.

## 문서 책임

| 구분 | 내용 |
| --- | --- |
| 문서 등급 | Gameplay canonical |
| 소유 범위 | 게임 정체성, 모드와 런 흐름, 노드 의미, 플레이어-visible 전투 규칙, 보상·성장·연구·실패·후퇴 정책 |
| 소유하지 않는 범위 | 정확한 타겟팅 알고리즘, runtime 생산자/커밋 불변식, RON loader/schema, Unity JSON shape와 message order, 작업 절차 |
| 함께 확인할 문서 | [skill_target_contract.md](skill_target_contract.md), [core_runtime_contract.md](core_runtime_contract.md), [data_loading_contract.md](data_loading_contract.md), [engineering_workflow.md](engineering_workflow.md), [README.md](README.md), 외부 Unity canonical |

세부 구현 계약은 이 문서에 모두 풀어 쓰지 않는다. 경계 주제는 다음 문서를 우선한다.

- 스킬/평타 타겟팅, 범위, 자동 시전, 유효 적대 대상 판정: [skill_target_contract.md](skill_target_contract.md)
- 전투 runtime source-of-truth, 결정론/RNG, event log, checkpoint, battle record, actor identity 구현 계약: [core_runtime_contract.md](core_runtime_contract.md)
- live RON authoring, loader, fixture, validation: [data_loading_contract.md](data_loading_contract.md)
- Unity WebSocket, snapshot, command/result 통합 계약: 외부 canonical `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- 전투 시작, live update, checkpoint, resync, 전투 종료 snapshot 흐름: 외부 canonical `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
- 리팩토링, 레거시 제거, debug 산출물 분류: [engineering_workflow.md](engineering_workflow.md)
- 문서 지도와 source-of-truth 순서: [README.md](README.md)

문서보다 실제 runtime code와 live RON/data가 우선이다. 문서와 코드가 충돌하면 코드를 먼저 확인하고, 게임 정책 판단이 필요한 부분은 사용자와 의논한다.

## 게임 개요

이 장은 플레이어가 접하는 게임의 정체성, 모드, 런의 기본 순서와 시작 roster 구성을 소유한다.
command field나 runtime 저장 구조는 의미를 설명하기 위한 seam이며 정확한 구현 계약은 인접 canonical을 따른다.

### 게임 정체성

이 게임은 환상체를 소유 기물로 쓰는 게임이 아니라, 소수의 직원을 이끌고 봉쇄된 시설을 탐사하는 로그라이크 전술 전투 게임이다.

핵심 판단은 아래 네 가지다.

- 어떤 직원과 장비, 스킬 파편으로 다음 노드에 들어갈지 정한다.
- 노드 브리핑과 위협 경고를 읽고 배치와 전투 계획을 세운다.
- `DefenseRoute` 전투 중 안정화치 기반 배치, 철수, 수동 스킬, 후퇴, 일시정지, 배속을 조작한다.
- 보상, 연구 진행도, 장비/파편 성장, 직원 손실 위험을 관리하며 다음 노드를 고른다.

현재 공식 전투 축은 명일방주식 실시간 배치 방어인 `DefenseRoute` 하나다. 보스 전투도 별도 전투 장르를 되살리기보다 `DefenseRoute` 파생으로 설계하는 방향을 우선 검토한다.

### 게임 모드

게임은 상위 구조로 `일반 모드`와 `끝없는 탐사 모드`를 가진다. 두 모드는 같은 직원, 장비, 스킬 파편, 노드, `DefenseRoute` 전투 규칙을 공유하지만, 런 길이와 보스 체인 정책이 다르다.

Runtime은 런 생성 시점부터 `GameMode`를 명시적으로 저장해야 한다. `GameMode`는 `StartNewGame` command의 필수 입력이며, 직원 선발 완료 후 생성되는 `RunProgression`에 그대로 저장된다. `GameMode`는 노드맵 생성, `Gate` 배치, 다음 Floor 전환, 최종 클리어/실패 조건의 source of truth다. 모드별 규칙을 generator나 UI 추론으로 흩어 두지 않는다.

일반 모드:

- 기본 모드이자 튜토리얼 성향의 짧은 모드다.
- 플레이어가 런 시작, 직원 선발, Floor 맵, 노드 선택, Safezone, `DefenseRoute` 전투, 후퇴/실패/보상, 장비/스킬 파편 성장 흐름을 익히는 것이 목적이다.
- 일반 모드는 3 Floor 고정 구조다.
- Floor 1~2는 엘리트 노드 클리어 후 열리는 `Gate` 노드를 통해 다음 Floor로 이동한다.
- Floor 3은 최종 보스가 최종 관문으로 등장하며, 최종 보스를 클리어하면 런 클리어다.
- 최종 보스의 패배 판정은 즉시 런 실패다. 엘리트와 일반 보스 실패는 런 실패가 아니라 해당 노드 실패로 처리한다.
- 고요한 오케스트라 같은 장기 보스 체인과 특수 보스 파편 리스크는 기본적으로 일반 모드의 핵심 콘텐츠가 아니다. 백야의 장기 체인/사도/특수 파편 세부 설계는 아직 미래·미정이며 현재 모드 규칙으로 해석하지 않는다.
- 일반 모드에서도 스킬 파편, 장비, 연구 진행도는 사용할 수 있지만, 끝없는 탐사용 장기 징조 체인을 끝까지 밀어 파편을 얻는 구조는 다루지 않는다.

끝없는 탐사 모드:

- 장기 생존과 심화 콘텐츠를 담당하는 모드다.
- Floor가 계속 이어지며, 플레이어는 누적되는 손실과 보상을 관리한다.
- 특정 보스가 `provisional_boss`로 떠오를 수 있고, 기존 노드 위에 보스 징조 source가 얹힐 수 있다.
- 플레이어가 같은 보스 계열의 징조 source를 반복 선택하면 보스 체인이 확정되거나 진행도가 누적될 수 있다.
- 조건이 완성되면 보스 방이 생성되거나, 일부 체인은 다음 노드 선택지를 잠그는 강제 보스 방으로 이어질 수 있다.
- 다음 층 이동 source는 `Gate` 노드다. 끝없는 탐사에서는 `Gate`의 부모가 엘리트, 보스, 일반 노드 중 무엇이든 될 수 있다.
- 기본적으로 한 층에는 `Gate`가 하나지만, 특수 층에서는 여러 `Gate` 후보를 허용할 수 있다.
- 조건부/Locked Gate, 비밀 Gate, 완전 숨김 Gate는 시스템상 미래 확장 후보지만 기본 live data에 임의로 작성하지 않는다. 그런 노드는 명시 정책이 있을 때만 추가한다.
- 고요한 오케스트라 같은 장기 보스 기믹, 특수 보스 파편, 고위험 보상과 누적 리스크는 끝없는 탐사 콘텐츠다. 백야의 상세 체인은 향후 정책부터 다시 승인할 설계 후보이며 현재 구현 계약이 아니다.

### 핵심 플레이 루프

기본 런 흐름은 아래 순서다.

```text
런 시작
-> 시작 직원 선발
-> Floor 맵 생성
-> 노드 선택
-> 노드 진입 전 확인과 준비
-> 전투/지원/정비/상점/보상/본사 연락 처리
-> 노드 결과 반영
-> 다음 노드 선택
-> 모드별 종료 조건 확인
```

일반 모드에서는 Floor 1~2의 `Gate`를 통한 다음 Floor 이동과 Floor 3 최종 보스 클리어가 핵심 진행/종료 조건이다. 끝없는 탐사 모드에서는 `Gate`를 통한 다음 Floor 이동, 보스 징조 체인, 누적 손실 관리가 핵심 진행/종료 조건을 만든다.

전투 노드는 한 번의 전투로 끝나지 않을 수 있다. 일반 전투, 엘리트, 일반 보스, 최종 보스는 같은 이상현상 시도 횟수 정책을 따르되, 최종 보스는 패배 판정이 즉시 런 실패라는 예외를 가진다. 패배 판정 전 전투 포기는 진입 직전 상태를 복원하므로 남은 시도를 줄이지 않는다.

### 시작 직원 선발

런 시작 시 플레이어는 시작 직원 후보 중 출전할 직원을 고른다.

- 모든 시작 후보와 본사 채용 후보는 하나의 authored `EmployeeDefinition` catalog에서 온다. `starter_eligible`은 시작 후보 자격이며, 시작 때 선택하지 않은 starter definition과 모든 non-starter definition은 같은 런의 미합류 채용 queue에 들어간다.
- `employee_definition_id`는 콘텐츠에서 안정적인 authored identity다. roster 합류 뒤 생성되는 `uuid`는 해당 런 안의 직원 instance identity이며 둘을 서로 대신 사용하지 않는다.
- definition은 이름, 역할, 배경, initial loadout, personality의 risk/social 두 축, `CombatAptitude`, 기본 HP/공격/방어/마법 저항, `basic_attack_speed_percent`를 명시한다.
- risk disposition은 `Brave`, `Cautious`, `SurvivalInstinct`, social disposition은 `Obedient`, `Defiant`, `Comradely` 중 각각 정확히 하나다. personality는 UUID나 신뢰 상태에서 추론하지 않는다.
- `CombatAptitude`는 `Balanced`, `Defensive`, `Offensive`의 가시성 정보다. 현재 장비 제한이나 불일치 페널티를 만들지 않는다.
- 직원 기본 공격 속도 100은 중립이다. 무기가 기본 공격 주기를 소유하고 최종 주기는 `ceil(weapon_base_interval_ms * 100 / basic_attack_speed_percent)`다. 무기가 없을 때도 같은 공식을 기본 직원 공격 주기에 적용한다.
- specialties는 현재 production schema/runtime/DTO에 없다.
- 미합류 직원 queue 순서는 run seed로 한 번 결정한다. 본사 연락은 queue head에서 authored 후보 수만큼 active session으로 옮기며 snapshot 재요청이나 재접속으로 다시 뽑지 않는다.
- 채용한 definition은 새 run-local UUID와 함께 roster로 이동하고, 선택하지 않은 후보는 제안된 순서 그대로 queue tail에 돌아간다. 응급지원이나 본사 상점을 선택하면 현재 후보 전부가 같은 순서로 tail에 돌아간다.
- 같은 definition은 queue, active recruitment session, roster 중 정확히 한 곳에만 존재한다. checkpoint load는 이 소유권과 queue 순서를 검증·복원하고 Floor 전환도 그대로 유지한다.
- 시작 직원은 이후 런 전체의 핵심 자원이다.
- 직원은 장비, 스킬 파편, run-local Psychology, physical damage, trauma를 통해 장기적으로 관리된다.
- 출전 가능한 직원이 없고 회복 가능 노드도 남아 있지 않으면 런 실패다.

## 탐사와 노드

이 장은 플레이어가 Floor를 탐색하고 node content를 선택·해결하는 규칙을 소유한다. Snapshot field,
RON field와 validation 세부는 각각 외부 Unity canonical, [data_loading_contract.md](data_loading_contract.md),
[core_runtime_contract.md](core_runtime_contract.md)의 경계 요약으로만 읽는다.

### Floor 맵과 노드 선택

Floor 맵은 봉쇄된 시설의 방과 복도를 탐사하는 지도다. 플레이어는 임의 node로 순간이동하지 않고,
core가 현재 위치와 authored facility connection으로 계산한 도달 가능한 node만 선택한다.

- Standard와 Endless는 같은 facility engine을 사용한다. Mode/Floor별 live schedule이 template pool과
  assignment profile을 고르고, pool의 deterministic 순서·cycle·사용 이력은 run/checkpoint 상태다.
- Facility template는 semantic room slot, slot별 허용 category, tag, typed terminal과
  `Bidirectional`/`ForwardOnly` connection을 소유한다. Runtime은 별도 node edge graph를 저장하거나
  depth/lane 좌표로 topology를 재생성하지 않는다.
- Floor 생성 시 모든 slot의 category, kind/content와 seed를 한 번 확정한다. Profile의
  min/max/exact와 slot allowed category를 만족하는 완전 배정이 없으면 category repair나 구 generator
  fallback 없이 content error다.
- Node는 Combat, SavePoint, Maintenance, Shop, Reward, HeadquartersContact, Boss, Gate 등의
  단일 category를 가진다.
- Start는 실제 node이며 Floor 시작 시 `Completed + Revealed` 상태와 현재 위치가 된다.
- 완료 node는 content를 다시 실행하지 않는 transit 지점으로 사용할 수 있다. Core는 완료된
  transit path와 connection direction을 반영한 최종 `map_navigation.selectable_node_ids`를 투영한다.
- Connection은 `Bidirectional` 또는 `ForwardOnly`다. Unity는 graph와 선택 가능성을 재계산하거나
  transit 이동을 별도로 연출하지 않고 core가 허용한 목적지를 즉시 선택한다.
- Combat/Boss 선택은 `NodeConfirm` preview를 열 뿐 현재 위치를 바꾸지 않는다. 실제 진입과
  node 소비는 출전 가능한 직원이 한 명 이상인 `ConfirmEnterNode` 성공 뒤 확정된다.
- 직원 카드 표시 순서는 Unity-local 설정이며 core gameplay, checkpoint와 battle retreat의
  복원 대상이 아니다.

Node state와 visibility의 gameplay 의미:

- `Available`: 선택 가능한 미해결 node.
- `Completed`: 해결됐고 transit으로만 사용할 수 있는 node.
- `Locked`/`Unavailable`: 조건 또는 현재 progression 때문에 선택할 수 없는 node.
- `Revealed`: 종류와 preview를 표시하는 기본 상태.
- `Obscured`: 종류·보상·상세를 숨길 수 있지만 core가 selectable로 투영하면 선택 가능하다.
- `Concealed`: Unity에서 숨기며 선택할 수 없다. 명시적인 비밀방/조건부 기믹에만 사용한다.

Gate는 다음 Floor로 이동하는 독립 node다.

- Elite/Boss가 Gate 역할을 겸하지 않으며 Standard Floor 1~2에서는 Elite 뒤에 별도 Gate가 열린다.
- Gate는 보이는 것과 선택 가능한 것이 다르다. Unity-local 확인을 거친 확정 command에서 core가
  현재 선택 가능성을 다시 검증한다.
- Gate는 SavePoint가 아니다.
- 전환 시 생존 직원의 trauma를 RunPolicy `gate_trauma_recovery`에 따라 줄인다. Live 값은
  `CurrentPercent { percent: 10 }`이며 현재 trauma의 10%를 올림한다. Critical과 Breakdown도
  생존 직원으로 포함하고 Dead는 제외하며 physical damage와 checkpoint는 바꾸지 않는다.
- Floor가 바뀌어도 roster condition, 장비, 스킬 파편, 소비 아이템과 resource는 유지된다.

시설 지도 좌표는 core의 `map_template_id + slot_id`와 Unity-local template가 연결한다. Slot이
없을 때 임의 좌표로 fallback하지 않는다. 차단 구역·오염·경고 같은 분위기 표현은 선택과 진행에
영향을 줄 때만 core node가 되고, 그 외에는 Unity template/art가 소유한다.

조건부 보너스 방은 Floor 생성 때 slot/category/content/seed까지 미리 확정된 `Concealed` node다.
Authored prerequisite slot들을 순서와 무관하게 모두 성공 완료한 transaction에서만 같은 map의
authored connection과 함께 한 번 `Revealed`로 발현한다. Retreat, 실패, 취소와 단순 방문은 발현을
일으키지 않으며 reconnect/checkpoint load는 content를 재추첨하지 않는다. 발현 방은 선택형 콘텐츠만
허용하고 Gate, SavePoint, 필수 Boss 같은 진행 필수 room은 대상이 될 수 없다.

### Certified 전투력과 Floor 난이도

- Core는 지금 전투에 진입할 수 있는 직원(`alive && physical_damage <= 4 && trauma < 100`)의 level,
  영구 combat stat, 장착 장비/강화, 장착 skill fragment, 공격 주기, block과 공격 범위만으로 versioned
  provisional certified power를 계산한다. 이 경계 안의 현재 HP 및 physical damage/trauma 크기,
  active consumable과 전투 중 임시 효과는 raw power 계수가 아니라 readiness 경고에만 반영한다.
  Critical, Breakdown, Dead 직원은 합산하지 않으며 roster의 eligible power가 0이면 상대 위협은
  `VeryHigh`다.
- 첫 Floor anchor는 starter/loadout commit 뒤 certified power와 authored 최소값의 최댓값이다. 이후
  실제 Combat 진입 때만 run peak를 올리고, 새 Floor anchor는 authored 최소값, 이전 anchor + 최소
  상승분, run peak의 최댓값으로 한 번 확정한다. 같은 Floor의 편성 변경은 node를 재스케일하지 않는다.
- Ordinary Combat는 Floor 생성 transaction에서 Easy/Standard/Hard quota를 결정론적으로 나눈다.
  Elite/NormalBoss/FinalBoss/BossOmen/MonolithRemanifestation은 ordinary quota와 분리된 role이다.
- 확정 난이도는 target power, enemy stat, generated-wave budget과 수량형 reward budget을 함께 소유한다.
  Hard와 특수 role의 reward multiplier는 Standard보다 커야 한다.
- Unity에는 exact certified/anchor/target/multiplier를 공개하지 않는다. 확정 band, 현재 certified 대비
  상대 위협과 Trauma/physical damage/출전 가능 인원 readiness만 표시한다.
- Run peak는 checkpoint load, retreat, restart와 active-run backup 복구로 감소하지 않는다. Current Floor
  anchor와 power model version은 checkpoint의 map/node difficulty와 같은 세대로 저장·복원한다. 같은
  checkpoint에서 같은 Gate를 재진행하면 같은 run peak/live policy에서 같은 다음 anchor를 만들며 load가
  상승분을 누적하지 않는다. 구 Floor-number scaling과 overflow clamp는 authority가 아니다.

### Safe Node와 Safezone

`Safe Node`는 맵에서 선택하고 한 번 소비하는 비전투 node이며 `Safezone`은 node 사이의 준비
UX다.

- Safe Node 계열은 SavePoint, Maintenance, HeadquartersContact, Shop, Reward다.
- SavePoint는 선택지를 고르는 중간 mode 없이 `SavePointState`와
  `selected_event.type == "save_point"`를 쓰는 독립 통과형 node다. Maintenance도 별도 상태를
  쓰는 독립 node다.
- Safezone은 node를 소비하거나 보상·회복·정비 효과를 자동 지급하지 않는다.
- Safezone에서는 node 탐색, 소비 아이템 사용과 원자적 loadout 교체를 할 수 있다. 장비 이동과
  같은 type slot 교체는 전부 성공하거나 전부 실패하며 장비를 소비·조합하지 않는다.
- 파편/장비의 강화·개화·분쇄는 Safezone이 아니라 Maintenance가 소유한다.

### Event Node

Event는 ordered illustration/text panel, 직원 판정과 분기 결과를 소유하는 독립 node category다.
Global Visual Novel이나 다른 node category의 fallback이 아니며 Standard와 Endless 기본 Floor에
최소 하나가 생성된다.

- Core는 event/scene identity, ordered panel id, interaction, eligibility, committed check와 진행
  상태를 소유한다. Unity는 image/text asset과 panel cursor만 로컬로 처리한다.
- 선택지가 없는 scene도 command로 진행하며 Unity가 다음 scene이나 eligibility를 계산하지 않는다.
- 직원은 저장된 Profile에 Medical, Security, Analysis, Reconnaissance, Containment,
  Maintenance 중 Event aptitude를 정확히 하나 가진다. 선택지의 aptitude와 일치하면 +3이고,
  성격·신뢰·기억·컨디션 수치·전투 스탯은 범용 보정치가 아니다.
- checked choice가 `aptitude`, DC 8/12/16/20과 optional key를 소유한다. 같은 aptitude에 적용되는
  현재 직원의 장착·소유 장비 중 가장 높은 +1/+2 하나만 더하며 동률은 장비 instance UUID
  오름차순이다. Inventory-only 장비는 적용하지 않고 key는 소비하거나 보정치를 암시하지 않는다.
- Core는 Event session의 choice/performer와 독립적인 pure 1d20을 한 번만 확정한다. Natural 1은
  critical failure, natural 20은 critical success이며 2..19는
  `raw_roll + employee_modifier + equipment_modifier >= dc`면 성공한다.
- 확정한 choice, performer, raw roll, typed modifier/key provenance와 outcome은 exact duplicate,
  reconnect/checkpoint/retreat 뒤에도 바꾸거나 다시 굴릴 수 없다. Invalid identity, condition,
  missing key와 stale preview는 상태·RNG를 변경하지 않는다.
- 실패는 command error가 아니라 authored failure scene이며, 모든 reference와 effect는 검증 뒤
  원자적으로 commit한다.
- 한 Event에는 reachable employee-check scene이 최대 하나다. 현재 첫 live checked Event는
  `sealed_archive_recovery_01`이고 Analysis DC16/key/80 Enkephalin과 Maintenance DC12/no-key/30
  Enkephalin 선택을 제공한다. `white_night_confession_01`은 advance-only다.

Event choice는 기존 combat pipeline을 시작할 수 있다.

- Event category를 Combat으로 바꾸지 않고 choice가 명시한 encounter를 사용한다.
- 승리하면 Event가 완료된다. Retry 가능한 패배나 retreat 뒤에는 committed check를 유지한 채
  같은 전투를 이어가며 choice/employee를 다시 선택하지 않는다.
- 도전 횟수를 모두 소진하면 Event가 완료된다.
- BossOmen Event도 같은 규칙을 따르고 Event 완료 시점에만 omen step을 소비한다.

### 일반 인벤토리

일반 인벤토리는 장비 instance와 소비 아이템 definition stack을 보유하며 gameplay admission cap이
없다.

- 장비 instance 하나와 서로 다른 소비 아이템 stack 하나가 각각 한 슬롯을 점유한다. 재화,
  장비 재료, 스킬 파편과 파편 가루는 포함하지 않는다.
- 소비 아이템은 수량 상한 없는 `definition_id` stack이며 수량 증가는 새 슬롯을 요구하지 않는다.
- `reserved_slot_count`는 50에서 시작해 새 슬롯 commit이 경계를 넘을 때 10단위로 증가하고
  제거 시 줄지 않는 표시용 high-water 값이다. Collection capacity나 overflow 저장소가 아니다.
- 보상, Event, Shop과 admin 지급은 슬롯 수 때문에 실패하거나 지연되지 않는다.

### 소비 아이템

소비 아이템은 `ViewingMap`과 `NodeConfirm`에서 사용하는 run 자원이다.

- Core가 대상과 효과를 검증하고 성공 시 definition stack 수량을 정확히 1 줄인다.
- 직원의 단일 active modifier가 이미 있으면 성공한 command가 기존 modifier를 즉시 제거하고 새
  modifier로 교체한다. 환불, 차단, 별도 overwrite 확인 modal은 없다.
- Metadata의 static UUID가 definition-backed stack의 wire selector이며 copy별 owned UUID는 없다.
- 적용한 직원 modifier duration은 전투 결과 commit마다 한 번 감소한다. Retreat와 non-combat
  node에서는 감소하지 않는다.
- 전투·결과·Safe Node·Shop·Reward 내부에서는 사용할 수 없으며 사용한 아이템은 되돌리지 않는다.
- Unity는 authoritative quantity와 command/snapshot 결과를 표시하고 local count를 추론하지 않는다.

### 상점

Shop은 Enkephalin으로 authored merchandise를 구매하고 허용된 보유품을 판매하는 Safe Node다.
Headquarters와 Unknown Shop은 같은 transaction/session 모델을 사용하되 서로 다른 merchandise
profile을 사용한다.

- Core가 offer, price, refresh와 sell eligibility를 계산하며 Unity는 재추첨하거나 재계산하지 않는다.
- Offer는 session 내 고유 identity, typed merchandise, 양수 quantity와 총 구매가를 가진다.
  장비 구매는 새 owned instance를 만들고 소비 아이템은 definition stack에 수량을 더한다.
- 구매한 자리는 자동 보충하지 않으며 non-repeatable 상품은 같은 session의 refresh에 재등장하지 않는다.
- Refresh는 authored 비용과 횟수를 원자적으로 소비해 page 전체를 교체한다. 비용·횟수·후보가
  부족하면 상태를 바꾸지 않는다.
- Sell은 현재 owned/착용 상태, tag, quantity와 quote를 검증한다. 귀속·장착 장비와 0원 거래는
  허용하지 않는다.
- 구매가는 base price percentage를 올림, 판매가는 내림하며 live validation은 same/cross-Shop
  arbitrage와 산술 overflow를 거부한다.

### 노드 진입 전 브리핑

전투 노드 미리보기는 제한 자원을 소비해 업그레이드하지 않는다. 기본 브리핑은 항상 전투 진입에 필요한 최소 정보를 제공해야 한다.

기본 브리핑에는 아래 정보가 포함된다.

- 전투 모드와 임무 변형
- 전장 템플릿과 배치 가능 구역
- 주요 route 또는 예상 진입 방향
- 위험도와 보상 후보
- 주요 적 또는 핵심 위협 요약
- 위협 경고와 그 신뢰도

정확한 웨이브 수, 모든 등장 타이밍, 숨은 기믹의 세부 값은 기본적으로 전투 중 관찰과 리트라이 학습 영역으로 둔다. 단, 플레이어가 납득 가능한 배치를 할 수 없을 정도로 정보가 부족하면 별도 소비 자원 없이 기본 브리핑 자체를 보강한다.

`EnemyBriefing`은 플레이어에게 보여줄 정보이며 실제 스폰 수량/종류의 source of truth가 아니다. 실제 전투 스폰은 `SpawnWave.enemy_entries`가 담당한다.

## 전투

이 장은 플레이어가 경험하는 전투 진입, 배치, 적 구성, 공격과 피해 피드백의 의미를 소유한다.
타겟 판정 알고리즘은 [skill_target_contract.md](skill_target_contract.md), event/checkpoint/result 생산자
불변식은 [core_runtime_contract.md](core_runtime_contract.md), 정확한 wire shape와 message order는 외부
Unity canonical이 소유한다.

### 전투 모드

현재 live 전투 모드는 명일방주식 실시간 배치 방어인 `DefenseRoute` 하나다.

- 플레이어 유닛은 배치 위치에서 route 적을 공격하고 `block_capacity`만큼 지속 저지한다. 거리
  이탈만으로 engagement가 풀리지 않으며 사망·철수·unblockable 전환·강제 이동 같은 명시
  조건에서만 해제된다.
- 철수/전투불능 유닛은 필드에서 제거되고 각각 30초/90초 뒤 새 runtime instance로 재배치한다.
  철수 재배치는 기존 HP에 최대 HP 30%를 회복하고, 전투불능 재배치는 갱신된 직원 condition으로
  시작 HP를 다시 계산한다. 이전 buff/cast/target/cooldown state는 상속하지 않는다.
- 철수한 유닛을 향해 비행 중인 hostile projectile은 miss로 끝난다.
- 모든 필수 wave·적 처리와 보호 대상 생존이 성공 조건이며 보호 대상 파괴나 scenario 조건이
  실패를 만든다. Pause, 배속과 battle retreat는 server simulation 기준이다.
- 일반 전투와 보스는 같은 transport, 전장, 배치와 wave 모델을 공유한다.
- 생존 전투는 별도 `Encirclement` mode가 아니라 `DefenseRoute / Defense + survive_timer_ms`로
  작성하며 timer 도달 시 남은 적과 무관하게 승리한다.

`DefendPoint`, `Encirclement` mission variant, `Recovery`, `DefendAndEscape`,
`TacticalGroupPlan`과 formation AI는 현재 live 계약이 아니다. `SplitRoom`은 전투 mode가 아니라
필요할 때 battlefield template로 표현한다.

### 전투 시작과 Unity 경계

전투는 live battle state를 사용한다.

- Scene construction은 `battle_setup_snapshot`, 연출 timing은 `battle_update.events_delta`,
  현재 상태와 복구는 `battle_update.checkpoint`, 결과 통계는 `selected_event.result_stats`가
  source of truth다.
- Active event/result는 전투 종료 뒤 player history로 누적하지 않는다.
- Core가 공격·스킬 판정과 preview를 계산하며 Unity는 authored metadata에서 재계산하지 않는다.

생산자 불변식은 [core_runtime_contract.md](core_runtime_contract.md), targeting은
[skill_target_contract.md](skill_target_contract.md), JSON shape와 message order는 외부 Unity
canonical을 따른다.

### 전장과 배치

Node마다 route, 장애물, 배치/출현 구역이 다른 비사각형 전장을 사용할 수 있다.

- `battle_setup_snapshot`이 실제 scene source이고 `combat_preview`는 진입 전 briefing이다.
- 배치와 range 후보는 tile 기반, 이동과 충돌은 2D 연속좌표 기반이다.
- 크기는 `Small`, `Medium`, `Large`이며 Boss는 기본적으로 Large다.
- Archetype은 `OpenHall`, `Corridor`, `ChokePoint`, `Ambush`, `Surrounded`,
  `SplitRoom`, `ObstacleRoom`, `BossArena`의 데이터 분류다. `SplitRoom`은 현재 live
  encounter 배정 대상이 아니다.
- ASCII template/route authoring은
  [data_loading_contract.md](data_loading_contract.md#battlefield-template-authoring), route와
  movement projection은
  [core_runtime_contract.md](core_runtime_contract.md#route-and-battlefield-projection)가 소유한다.

### 적 구성과 웨이브

일반 wave의 주력은 `CorrodedEmployee`이고 `Abnormality`는 Elite/Boss/특수 encounter의 중심
위협이다. `FacilityEntity`는 아직 live 대상이 아니다.

Encounter 분류:

- `Normal`: 순수 침식 직원 전투. Primary abnormality와 그 연구·bonus objective가 없다.
- `Elite`, `NormalBoss`, `FinalBoss`: 정확히 하나의 `primary_abnormality_id`를 가지며
  각각 elite, 일반 boss, Standard run-ending boss에 사용한다.
- 환상체 `threat_class`는 최소 Elite이고 Boss 환상체만 Boss다. 실제 spawn은 encounter
  classification이 아니라 wave data가 소유한다.

Wave는 `SpawnWave`로 관리한다.

- 현재 live authored wave는 전투 시작 5초 뒤 등장한다. 다른 timing은 필요할 때 명시한다.
- `required_for_victory: false` wave는 승리 필수 대상이 아니다.
- Manual source는 typed `CorrodedEmployee(profile_id, tier, count)` 또는
  `Abnormality(abnormality_id, tier, count)` entry를 사용한다.
- Preset 생성도 허용하지만 결과는 CombatPreview 때 `enemy_entries`로 확정하며 전투 중
  재생성하지 않는다.
- HP/사망/웨이브 전멸/지점 도달 기반 조건부 spawn event는 아직 정규 계약이 아니다.

### 기본 공격과 스킬 타겟팅

타겟팅의 상위 원칙은 단순하다.

- 어떤 공격이든 deterministic expected final damage가 0보다 크면 유효 후보로 본다.
- 피해가 0이어도 선택한 적 또는 AoE 안의 적에게 적용되는 적대적 target-applied effect가 있으면 유효 후보로 본다.
- 피해도 없고 적대적 대상 효과도 없으면 자동 평타/스킬 후보에서 제외한다.
- 후보가 여럿이면 기본 공격은 무기의 `targeting_profile`, 스킬은 스킬 자체의 명시
  `UnitTargetRule`과 deterministic fallback/tie-break 규칙을 따른다. 스킬이 무기 profile을
  상속하는 경로는 없다.
- 피해 가능 대상과 효과만 가능한 대상 사이에 별도 우선순위는 두지 않는다.

적대적 효과의 분류, 타겟 filter 평가 시점, blocker-first, route enemy 재이동, cast/step retarget,
projectile lifecycle, `TileArea`, range authoring의 세부 source of truth는
[skill_target_contract.md](skill_target_contract.md)다.

### 피해 타입과 피드백

`Physical`, `Magic`, `True`는 빌드 효율 차이를 만들지만 영구 단일 타입 면역으로 특정
딜러 보유만 검사하지 않는다. 높은 방어/저항과 일시 기믹은 허용하되 관통, 저항 감소, 고정 피해,
제어, 저지, 아이템과 timing 같은 복수 counterplay를 둔다.

Core가 계산한 `HpChanged`와 `final_damage`가 피해·HP·사망의 source of truth다. Unity는 다음
`feedback_tags`만 표시하고 방어 계산을 재구현하지 않는다.

- `critical`: 치명타
- `mitigated`: 크게 감소한 피해
- `fixed_damage`: 고정 피해
- `immune`: 피해 또는 상태가 완전히 막힘

`piercing`, `shield`, `blocked`, `resisted_status`는 현재 공식 피해 feedback tag가 아니다.

## 성장과 연구

이 장은 전투와 node resolution 뒤 플레이어가 얻거나 누적하는 보상, 성장, 연구, 장비와 스킬
파편의 gameplay 의미를 소유한다. 수량 commit과 runtime transaction은
[core_runtime_contract.md](core_runtime_contract.md), authored schema와 catalog validation은
[data_loading_contract.md](data_loading_contract.md)가 소유한다.

### 보상과 성장

Combat reward는 `Currency`, `Equipment`, `SkillFragment`, `ResearchProgress` effect를 자동
지급한다. Live encounter는 `ClaimAll`만 허용하며 `ChooseOne`은 현재 계약이 아니다. Reward
node는 authored pool에서 XP를 제외한 후보 하나와 그 effect를 node seed로 고정한다.

직원 성장:

- 새 직원은 level 1, 현재 level XP 0에서 시작한다.
- XP requirement, 생존 XP와 battle tier는 `run/policy.ron`이 소유한다. 현재 모든 level은
  100 XP, 전투 결과 생존 직원은 승패와 무관하게 10 XP, tier는 level 1/3/6부터 I/II/III다.
- 같은 직원이 여러 번 배치돼도 생존 XP는 한 번만 지급하고 retreat에는 지급하지 않는다.
- 한 grant가 여러 level을 넘을 수 있으며 남은 XP는 다음 level에 유지된다. Battle tier는 현재
  stat multiplier가 아니다.
- Authored `GrantExperience.amount`는 total pool이며 대상 직원에게 결정론적으로 균등 분배한다.
  정확한 staging과 remainder 규칙은
  [core_runtime_contract.md](core_runtime_contract.md#reward-and-employee-growth-boundary)가 소유한다.
- `GrowthStack`은 XP/level과 별도다. 현재 production producer는 없고 level-up으로 증가하지 않는다.

### 연구 완료와 본사 송신

스킬 파편 연구도 보상은 파편 자체를 즉시 지급하지 않고, 해당 파편의 `research_progress`를 올린다. 연구도는 개화 진행도와 별개의 본부 분석/블랙박스 회수 진행도다.

기본 정책:

- 연구 진행도가 완료 기준에 도달하면 pending 연구 완료 항목으로 등록한다.
- node 활동 중에는 pending 상태를 유지한다. SavePoint, 본사 연락, Shop, Reward, Maintenance,
  Event를 포함해 어떤 node도 진입만으로 지급하지 않는다.
- 해결된 node가 권위적으로 완료되어 지도/planning으로 돌아오는 순간, Unity가 Safe Zone으로
  표현하는 그 복귀와 함께 모든 pending을 원자적으로 자동 수령한다.
- 전투 재도전, 전투 후퇴, node 선택 취소, checkpoint load, reconnect/snapshot 재표시,
  RunComplete, RunFailed는 Safe Zone 복귀 지급이 아니다.
- Event에서 시작한 전투는 Event node 전체가 해결될 때까지 보류한다. Gate는 다음 Floor의
  planning에 도착할 때 지급한다.
- pending 연구 완료 수령은 node 행동권을 추가로 소모하지 않으며, 같은 복귀를 다시 표시해도
  중복 지급하지 않는다.
- 인벤토리 스냅샷은 보유 파편 목록과 별도로 `skill_fragment_progress`, `pending_research_deliveries`를 노출한다.

지급 권위는 Core의 planning-return transition 하나다. Unity 화면 진입이나 node category
allowlist로 대체하지 않는다.

### 영구 환상체·침식 도감

게임은 플레이어가 실제로 조우한 환상체와 침식 프로필의 로컬 영구 도감을 유지한다. 이는 전투 history나 재생 기록이 아니라 canonical ID의 `discovered`/`defeated` set이며 `defeated ⊆ discovered`다.

- 전장에 실제 spawn된 적군 `Abnormality`/`CorrodedEmployee`만 후보다. 노드 preview, briefing, 아직 spawn되지 않은 wave는 도감에 올라가지 않는다.
- 승리하거나 최종 패배로 해당 combat node가 소비될 때만 실제 spawn ID를 발견으로 커밋한다. 재도전이 남은 패배와 전투 포기는 발견을 커밋하지 않는다. Player 승리는 primary 환상체를 발견·격파하고, 같은 승리 attempt에서 실제 `UnitDied`한 non-primary 환상체/침식만 추가 격파한다. 패배와 생존한 non-primary는 격파하지 않는다.
- 커밋된 두 set은 local-install profile의 단조 union이며 checkpoint load, 새 run, RunFailed, 일반
  non-battle transaction 실패와 active-run transaction rollback으로 사라지지 않는다.
  Run/checkpoint/transport player UUID와 분리되고, title의 확인된 전체 profile reset만 삭제할 수 있다.
- profile에는 ID set만 저장하고, 표시 이름·위험 등급과 authored 순서는 현재 live catalog에서 해결한다. 현재 catalog에서 사라진 dormant ID는 profile에 보존하지만 current snapshot에는 투영하지 않는다.
- `RunSnapshotDto.encounter_compendium`은 현재 catalog 전체를 authored 순서로 투영한다. 미발견 항목은 ID와 `discovered:false`, `defeated:false`만 가진 `???` placeholder이고 발견 항목만 metadata를 공개한다.
- Unity는 Core projection과 host의 `encounter_compendium_persistence` 상태만 표시하며 battle event, preview, local catalog로 발견·격파를 재계산하지 않는다.

### 끝없는 탐사 환상체 대응 연구

Endless run은 스킬 파편 연구 및 영구 도감과 별도로 Elite/Boss/FinalBoss 환상체의 run-local 대응
연구를 보존한다. 도구형 환상체, 침식 직원, 시설 entity와 일반 wave 적은 대상이 아니다.

- 각 대상은 research point/requirement, completion, suppression count, 최근/완료 Floor와 unique
  fragment 지급 여부를 가진다. Point가 판정 source이고 percentage는 UI 파생값이다.
- 현재 live 기본값은 requirement 100, 승리 20 point, 반복 제압 dust가 Elite 5/Boss 10/FinalBoss
  20이다. 모든 수치는 `run/policy.ron`에서 조절한다.
- 최초 완료 시 metadata가 지정한 skill fragment를 한 번 지급한다. 이후 같은 환상체 승리는
  fragment를 중복 지급하지 않고 threat class별 dust를 지급한다.
- 모든 대상 환상체의 대응 완료가 Endless 완료 조건이다.

반복 조우는 허용하지만 일반 non-omen 선택에서 최근 1 Floor의 같은 환상체를 제외하고, 완료
환상체의 현재 weight를 25%(미완료 100%)로 낮춘다. 후보가 비면
`ResetSoftRepeatConstraints`가 최근 제외와 weight 감소만 초기화하며 hard constraint를 깨지
않는다. BossOmen·강화/각성·FinalBoss 조건은 이 일반 weight를 명시적으로 우회할 수 있다.
`last_encountered_floor`는 실제 결과 처리 때만 갱신한다.

### Elite 모노리스와 강화 재현

Endless 자연 Elite는 같은 run에서 승리 또는 세 번째 패배가 확정되면 다시 자연 배정되지 않는다.
같은 Floor에도 같은 Elite identity를 둘 이상 배정하지 않으며, 모든 자연 Elite가 해결되면 이후 Elite
slot은 authored 일반 Combat으로 바뀐다. 미방문 배정은 해결이 아니며 Standard에는 이 규칙을 적용하지
않는다.

- 자연 Elite 최초 승리만 해당 환상체의 모노리스 구매 자격을 연다. 한 identity는 run당 한 번만
  구매할 수 있고 3 uses를 가진다.
- Endless Floor 1~3에는 떠돌이 상인이 없다. Floor 4+의 전용 상인은 첫 page 3칸에 eligible
  모노리스를 최대 2개 배치하고 나머지는 고위험 consumable로 채운다. 한 번의 유료 refresh는 page
  전체를 item-only 3칸으로 바꾼다. Standard의 일반 exploration Shop은 별도다.
- NodeConfirm은 순수 Available Normal Combat에서만 `모노리스 사용 (N)`과 환상체별 카드를 투영한다.
  최종 확정은 node 구조를 유지하고 encounter/difficulty를 다음 stage로 바꾸면서 1 use를 차감한다.
- 강화 재현은 stage 2와 3이며 최대 2회 시도한다. 첫 패배는 같은 node를 재시도하고 두 번째 패배는
  무보상 완료한다. Retreat는 attempt를 소비하지 않는다. 승리만 stage를 올리고 stage 3은 authored
  고유 skill fragment를 정확히 한 번 지급한다.
- 변환된 difficulty는 typed stage를 소유하고 monolith ledger의 stage와 반드시 일치한다. Stage는
  target power와 적 최대 HP/공격/방어에만 합성하고 wave budget은 바꾸지 않는다. 현재 stage 2는
  HP 165%·공격/방어 145%, stage 3은 HP 207%·공격/방어 182%이며 둘 다 wave budget 200%다.
- Stage 보상은 Monolith role multiplier와 먼저 합성한 뒤 원래 authored 수량에 한 번만 적용하고
  한 번만 반올림한다. Unity에는 exact multiplier 대신 `enhanced`/`final_enhanced` tier만 투영한다.
- stage 3 승리 시 use가 정확히 1 남으면 source identity를 가진 잔재로 전환한다. 서로 다른 경로에서
  모은 잔재가 네 개가 되면 다음 Floor의 Shop 하나를 보장한다. 상인은 Shop보다 먼저 일회성 VN으로
  oldest four 교환을 제시하며 수락은 run-local 변질된 모노리스를 주고 거절은 같은 run에서 비가역적이다.
- 변질된 모노리스는 현재 dormant run-local key다. 숨겨진 보스, 관리자 연출, 전용 capstone, 기존
  Endless 최종 완료 변경은 콘텐츠 정책이 확정될 후속 phase 전까지 존재하지 않는다.

Bonus objective:

- Encounter가 명시한 objective만 존재하며 primary abnormality가 없는 Normal encounter에는 없다.
- 승리 결과에서 완료 event log/result stats로 평가하고 objective별 전투당 한 번만 연구 bonus를
  더한다. 총 증가는 기본 승리 point와 만족한 bonus의 합이다.
- `ClearWithin(time_ms)`는 battle start부터 victory까지의 시간을 검사하며 늦게 spawn하는
  boss에는 사용하지 않는다.
- `DecisiveDamage(percent)`는 primary abnormality에게 한 공격 원천이 준 non-DoT 실효 피해가
  최대 HP의 지정 비율 이상인지 정수 비교한다. 여러 tick을 합산하지 않고 마지막 일격일 필요도 없다.
- “환상체 부위”는 별도 item이 아니라 bonus 연구 증가를 표현하는 presentation이다.

### 장비

장비는 일반 장비와 E.G.O 장비로 나뉘며 무기, 방어구, 악세서리 슬롯을 사용한다.

- 무기는 기본 공격·range·targeting과 block 수치를 제공한다. 각 무기의 필수
  `weapon_profile.block_capacity`가 직원의 기본 저지 수를 단독으로 결정하며, 직원 class/
  `CombatAptitude`나 방어구는 이 값을 변경하지 않는다. 무기가 없으면 0이다.
- 현재 authoring 기준은 Shield 3, Sword/Spear 1, Bow/Gun/Staff 0이다. 이 값은 모든 live weapon
  RON에 명시하며 runtime이 archetype으로 추론하거나 fallback하지 않는다. 방어구는 생존 stat과
  utility를, 악세서리는 보조 stat과 utility를 제공할 수 있다.
- 강화 level은 definition이 아니라 `OwnedEquipment` instance에 저장하며 전투 시작 draft에
  snapshot한다.
- 강화는 장비 가루를 사용한다. 장비 제작은 현재 Maintenance action이 아니다.
- 장비 가루와 skill fragment dust 경제는 분리하며 서로의 강화에 사용할 수 없다.

### 스킬 파편

직원은 기본 고유 active skill 대신 장착한 skill fragment의 대표 active skill을 사용한다.

- 기본 active fragment slot은 1개다. 환상체 원본 skill과 직원용 모방 skill은 별도 data다.
- Fragment는 passive 장비가 아니며 평타 강화, 일시 stat, buff와 공명 변화도 active skill 결과로
  표현한다.
- 위 두 항목은 정식 fragment 정책이다. 현재 abnormality-derived 28개와 concept starter 1개의
  직접 `BasicAttackModifier`는 dedicated active skill 제작 전의 temporary placeholder이며 정식
  정책이 구현 완료됐다는 근거가 아니다. 이 correction에서는 해당 live data를 변환하지 않는다.
- 강화·개화·분쇄는 Maintenance가 소유하며 전투/결과/보상 중에는 교체하지 않는다.
- 장착 조건은 직원 직업명이 아니라 장착 무기의 combat profile을 사용한다. Melee/ranged 또는
  특정 archetype 제한을 허용하되 모든 fragment를 모든 무기에 강제 호환하지 않는다.
- 최종 장착 가능성은 core validation이 소유한다. 콘텐츠 설계와 live 목록은 `docs/skills/`를
  함께 확인한다.

## 비전투 노드

이 장은 SavePoint, Maintenance, HeadquartersContact에서 플레이어가 얻는 결과를 소유한다.
세션 identity, capability, atomic commit과 snapshot projection은 runtime/wire 경계의 요약이며 세부
구현은 [core_runtime_contract.md](core_runtime_contract.md)와 외부 Unity canonical을 따른다.

### SavePoint 노드

- 완료 시 run당 단일 checkpoint를 덮어쓰고 Safezone으로 복귀한다.
- Load는 최대 3회이며 Config에서 core가 `LoadRunCheckpoint` capability를 enabled로 투영할 때만
  실행한다. Checkpoint metadata 자체는 실행 가능성을 뜻하지 않는다.
- 생존 직원 trauma를 RunPolicy `save_point_trauma_recovery`에 따라 줄인다. Live 값은
  `CurrentPercent { percent: 15 }`이며 현재 trauma의 15%를 올림한다. Critical과 Breakdown도
  생존 직원으로 포함하고 Dead는 제외한다. Physical damage는 바꾸지 않는다.
- Recovery, pending 연구 지급, node completion과 checkpoint capture는 승인된 최종 상태를 기준으로
  원자적으로 한 번만 적용한다. Host의 durable active-run publish까지 실패하면 이 transition을
  성공으로 보고하지 않는다. SavePoint는 psychology/trust/memory/event를 생산하지 않는다.
- 마지막 완료 SavePoint는 versioned local-install active-run current file과 rolling backup의
  continue source다. 이어하기는 load 횟수를 소비하지 않으며 transport player identity와 영구
  조우 도감은 payload 밖에서 주입·보존한다.
- `loads_used`는 checkpoint payload와 분리된 run-scoped monotonic metadata다. 성공한 명시적 load만
  정확히 1 증가시키고 SavePoint 덮어쓰기, checkpoint 복원, process restart와 backup 복구로
  감소시키지 않는다.

### Maintenance 노드

Maintenance는 분쇄·강화·개화를 반복 실행한 뒤 완료하는 독립 Safe Node다.

- Skill fragment는 fragment dust, 장비는 equipment dust 경제를 사용한다.
- Bloom은 현재 skill fragment 전용이며 장비에서는 disabled다.
- 장착 중인 대상도 확인 후 자동 해제해 분쇄할 수 있고 Maintenance 안에서 loadout 교체도 가능하다.
- Core가 대상별 가능 여부, disabled reason, 비용/획득/before-after preview와 확인·자동 해제 여부를
  계산한다.
- Root capability는 `UpgradeSkillFragment`/`AwakenSkillFragment`/`DismantleSkillFragment`를 각각
  skill-fragment 대상의 `enhance`/`awaken`/`dismantle` operation에,
  `DismantleEquipment`/`EnhanceEquipment`를 equipment 대상의 `dismantle`/`enhance` operation에
  대응시킨다. Maintenance 밖에서는 이 다섯 action을 생략한다. Maintenance 안에서는 대응
  `can_execute`가 하나라도 true면 enabled이고, 하나도 없으면 `NoEligiblePayload`로 disabled다.
- Enabled capability는 모든 대상이 가능하다는 뜻이 아니다. 선택한 대상은 실행 시 Core가 최신
  보유·장착·recipe·재료 상태를 다시 검증하며 실패한 command는 부분 mutation을 남기지 않는다.
- 장비 분쇄/강화는 대상 definition의 explicit RON dismantle/enhancement recipe가 있을 때만
  실행한다. Recipe가 없으면 preview의 `can_execute`가 false이고 기존 typed
  `disabled_reason`(`cannot_dismantle_equipment`/`cannot_enhance_equipment`)을 표시한다. Generic
  비용/획득 fallback은 없다.
- 현재 live equipment recipe coverage는 `standard_armor` 하나뿐이다. 다른 무기/방어구/
  악세서리 recipe와 수치는 후속 콘텐츠·밸런스 authoring 범위다.
- 완료 전에는 여러 작업을 반복할 수 있고 완료하면 node를 소비한다. Safezone Loadout은 이
  정비 효과를 제공하지 않는다.

### 본사 연락 노드

본사 연락 노드는 런 중 본부와 연결되는 별도 Safe Node다.

- `SavePoint`, `Maintenance`와 역할이 다르므로 별도 노드 계열로 다룬다.
- pending 연구 완료는 본사 연락 진입 시 수령하지 않는다. 본사 연락 행동이 node를 해결하고
  planning으로 돌아갈 때 다른 node와 같은 자동 수령 규칙을 사용하며 추가 행동권을 소모하지 않는다.
- 본사 연락은 추후 정보 확인, 연구 송신, 특수 지시 수령 같은 별도 선택지로 확장할 수 있다.

## 결과와 실패

이 장은 node 소비, 전투 결과 확정, 후퇴, 직원 상태와 run failure가 플레이어에게 갖는 의미를
소유한다. Runtime instance ledger, rollback snapshot, capability preflight와 commit 순서는
[core_runtime_contract.md](core_runtime_contract.md)가 소유한다.

### 노드 결과 처리

CombatResult는 성공/실패, 직원 condition 변화, 통계, 자동 보상, 연구 진행과 pending 완료를
표시한 뒤 플레이어 확인으로 commit한다.

- `CompleteCombatResult`는 condition, consumable duration, reward/XP, research/pending, node
  진행과 session 정리를 하나의 원자적 transition으로 적용한다. 실패하면 user-visible state를
  부분 commit하지 않는다.
- Attempt 중 condition은 working state에 누적하고 결과 확인 때 한 번 roster에 반영한다. 여러
  runtime instance의 통계는 직원별 한 행으로 합치며 마지막 instance가 최종 HP/생존을 대표한다.
- 결과 완료 capability와 preflight는 Core가 소유하고 Unity는 context에서 추론하지 않는다.
- 패배도 살아남은 직원에게 생존 XP를 지급하지만 성공 보상과 연구 진행은 지급하지 않는다.

전투 node는 첫 진입을 포함해 기본 최대 3회 도전한다.

- 승리/패배 result commit이 한 회를 소비하고 retreat는 진입 상태를 복원해 소비하지 않는다.
- Normal/Elite/NormalBoss는 3회를 모두 소진하면 보상 없이 Completed transit node가 된다.
- 모노리스 강화 재현 node는 예외적으로 최대 2회이며 두 번째 패배에 보상 없이 Completed가 된다.
- FinalBoss 패배는 run failure 후보이며 결과 확정 전 retreat는 허용한다. 유효하고 횟수가 남은
  checkpoint가 있으면 다른 gameplay failure와 같은 사전 선택을 거치고, 없으면 즉시 확정된다.

### 후퇴

`RetreatBattle`은 개별 유닛 철수와 달리 BattleEnd/패배 판정 전 전투 전체를 포기하는 rollback이다.

- Pause, cast와 wave 진행 중에도 가능하지만 명시된 강제 encounter는 금지할 수 있다.
- Core는 NodeConfirm 진입 직전의 attempt, RNG/UUID continuation, roster condition, consumable,
  inventory/resource, node/session/VN/checkpoint 상태를 정확히 복원한다.
- CombatResult, 보상·패널티, node 완료와 시도 소진을 만들지 않는다.

### 직원 condition, 전투 HP, 전투불능, 런 실패

Persistent condition의 authority는 `physical_damage: 0..6`와 `trauma: 0..100`뿐이다.

- Physical damage 0/1~3/4/5/6은 Normal/Minor/Major/Critical/Dead이고 trauma 100은 살아 있는
  Breakdown이다.
- Combat/Event/high-risk 참여는 physical damage 0~4와 trauma 0~99에서만 가능하다. Critical과
  Breakdown은 loadout/치료가 가능하지만 사망 직원은 모든 action에서 제외한다.
- Battle HP는 runtime 값이며 persistent condition으로 되쓰지 않는다. 전투불능 instance마다
  attempt-local physical damage 1과 authored trauma를 정확히 한 번 더한다.
- Physical damage는 명시적 Physical Treatment만 줄인다. 살아 있는 직원은 현재 physical damage가
  0이어도 구조적으로 유효한 target이며, 선택한 Treatment는 실효 회복량과 무관하게 소비된다.
  SavePoint/Gate는 trauma만 회복하고 사망을 되돌리지 않는다.
- 일반 사망 시 roster record는 남기되 장비와 shared active fragment를 원자적으로 회수하고
  consumable modifier를 제거한다. Death-prevent가 발동하면 회수하지 않는다.

살아 있는 직원이 없으면 run failure 후보이다. 전투 결과 뒤 deployable 직원이 없으면 현재 보유한
Physical Treatment, 현재 progression에서 즉시 선택 가능한 SavePoint/Gate, 실제 후보를 제공하는
HeadquartersContact recruitment를 하나의 recovery-path evaluator로 계산한다. 제한 중에는 이 행동과
checkpoint load만 허용하고 snapshot의 `combat_recovery_restriction` reason/허용 node 집합을 Unity가
표시한다. 허용 행동 하나가 끝날 때마다 전체 deployability와 경로를 다시 계산하며 더 이상 실제
경로가 없으면 failure 후보가 된다.

모든 gameplay failure 후보(`NoLivingEmployees`, `NoDeployableEmployees`, `BossDefeated`)는 유효하고
횟수가 남은 checkpoint가 있으면 `AwaitingRunFailureDecision`에 먼저 들어간다. 플레이어는 checkpoint
load 또는 명시적 failure 확정을 선택하며, checkpoint가 없거나 loadable하지 않으면 즉시
`RunFailed`로 확정된다. `RunComplete`, 확정된 `RunFailed`, `AbandonRun`은 durable 이어하기를
무효화한다. 유효한 이어하기를 새 run으로 교체할 때도 한 번의 명시적 확인이 필요하다.

### 직원 Psychology

Authored personality는 변하지 않는 risk/social 두 축이고 Psychology는 별도의 run-local 신뢰와
기억 상태다.

- `trust_score`는 -100~100이며 band는 Distrust(-100~-51), Uneasy(-50~-11), Neutral(-10~25),
  Trusting(26~70), Devoted(71~100)로 파생한다.
- 현재 production Psychology mutation은 없다. SavePoint/Gate trauma recovery는 신뢰, 기억 또는
  최근 사건을 만들지 않는다.
- Durable memory는 같은 kind/subject를 병합하고 optional duration을 node 완료마다 줄인다. 최근
  반응은 별도 최대 8개 receipt다.
- Psychology는 roster와 checkpoint/retreat에 보존되며 사망해도 남는다.
- 신뢰 기반 명령 거부, combat modifier와 대사·표정 cue는 아직 gameplay 계약이 아니다.

## 고급 정책과 레거시

### BossOmen Chain

BossOmen은 Endless에서 final-boss급 encounter를 예고하고 확정하는 단일 active run-local chain이다.

- 후보는 authored minimum Floor, awakened fragment 보유, active chain 부재를 요구하며 abnormality
  metadata의 `omen_chain_id`가 별도 chain definition을 가리켜야 한다.
- Step은 Event 또는 Combat source로 기존 같은 category node에 overlay한다. 적합한 node가 없으면
  잘못된 category로 fallback하지 않고 provisional boss와 step을 다음 Floor로 미룬다.
- 첫 omen source를 무시하고 Gate를 통과하면 boss를 pool에 돌려보내며 chain을 확정하지 않는다.
  Source를 해결하면 chain이 확정되고 node가 Completed될 때 step을 소비한다.
- 모든 step 뒤에는 현재 탐사 map을 authored `boss_omen_capstone` map으로 원자 교체한다. Capstone은
  Boss room과 미리 배정된 concealed Exit Gate room, `Boss -> Exit Gate` ForwardOnly connection을
  사용하며 이전 탐사 room으로 돌아갈 수 없다. Boss 진입 전 Safezone loadout은 허용한다.
- Forced boss 패배도 result commit 뒤 공통 failure-decision 규칙을 사용한다. BossOmen 전용
  post-`RunFailed` load 예외는 없으며 FinalBoss와 다른 failure reason도 같은 checkpoint 사전 선택을
  공유한다.
- 승리는 authored 보상과 chain 종료를 commit하고 capstone에 이미 배정된 Exit Gate와 connection을
  공개한다. 새 node/edge를 만들지 않으며 Floor advance와 Gate trauma 회복은 boss 승리가 아니라 그
  Gate 확인 때 적용한다.

### 레거시 제거 원칙

공통 compatibility, fallback, 구 테스트와 사용자 확인 원칙은
[engineering_workflow.md](engineering_workflow.md#레거시와-문서-정리)가 소유한다. 이 룰북은
gameplay domain에서 다시 만들지 않을 계약만 열거한다.

- 구형 `timeline_delta`/`battle_delta` 전송을 다시 만들지 않는다.
- 제거된 전투 모드와 RON wrapper는 필요하다는 이유만으로 부활시키지 않는다.
- 제거된 `Support`, `Rest`, `ChooseSupport`, support choice mode와 관련 Psychology 사건은
  compatibility alias나 fallback으로 부활시키지 않는다.
