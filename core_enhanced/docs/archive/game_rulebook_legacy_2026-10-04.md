# Game Rulebook — 과거 정책 (2026-10-04 보관)

이 문서는 과거 게임 규칙의 비교·보존 자료다. 최신 게임 정책은
[최신 게임 룰북](../game_rulebook.md)와 연결된 `../game_rules/` 상세 규칙, 이후 사용자 합의가 소유한다.
파일 수정 시각은 정책 우선순위를 뜻하지 않는다. 아래 본문의 현재·완료·canonical 표기는 당시 기록으로 읽는다.

2026-10-04 U04/U05 구현 중 혼입된 갱신은 U04 변경 전 보관본으로 복원했다. 이후 새 정책이나 구현 결과를
이 문서에 동기화하지 않는다. [교정 기록](../roadmaps/core_first/execution/rulebook_authority_correction_20261004_001/RECORD.ko.md)을 따른다.

## 문서 책임

| 구분 | 내용 |
| --- | --- |
| 문서 등급 | Historical gameplay reference — 최신 정책 기준 아님 |
| 소유 범위 | 게임 정체성, 모드와 런 흐름, 노드 의미, 플레이어-visible 전투 규칙, 보상·성장·도감·실패·후퇴 정책 |
| 소유하지 않는 범위 | 정확한 타겟팅 알고리즘, runtime 생산자/커밋 불변식, RON loader/schema, Unity JSON shape와 message order, 작업 절차 |
| 함께 확인할 문서 | [skill_target_contract.md](../skill_target_contract.md), [core_runtime_contract.md](../core_runtime_contract.md), [data_loading_contract.md](../data_loading_contract.md), [engineering_workflow.md](../engineering_workflow.md), [README.md](../README.md), 외부 Unity canonical |
| 보존 검증 locator | [runtime_feature_manifest.v1.json](../architecture/runtime_feature_manifest.v1.json) — 이 룰북의 heading을 stable feature ID, current route와 executable evidence에 연결하는 validation manifest이며 gameplay authority는 소유하지 않음 |

세부 구현 계약은 이 문서에 모두 풀어 쓰지 않는다. 경계 주제는 다음 문서를 우선한다.

- 스킬/평타 타겟팅, 범위, 자동 시전, 유효 적대 대상 판정: [skill_target_contract.md](../skill_target_contract.md)
- 전투 runtime source-of-truth, 결정론/RNG, event log, checkpoint, battle record, actor identity 구현 계약: [core_runtime_contract.md](../core_runtime_contract.md)
- live RON authoring, loader, fixture, validation: [data_loading_contract.md](../data_loading_contract.md)
- Unity WebSocket, snapshot, command/result 통합 계약: 외부 canonical `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- 전투 시작, live update, checkpoint, resync, 전투 종료 snapshot 흐름: 외부 canonical `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
- 리팩토링, 레거시 제거, debug 산출물 분류: [engineering_workflow.md](../engineering_workflow.md)
- 문서 지도와 source-of-truth 순서: [README.md](../README.md)

게임 의미의 최신 기준은 `../game_rulebook.md`·연결 상세 규칙·후속 사용자 합의다.
이 과거 문서와 현재 코드의 차이만으로 구현 drift를 판정하거나 코드를 과거 정책으로 되돌리지 않는다.
최신 문서가 명시적으로 계승한 공통 규칙만 그 위임 범위에서 참조한다. 구현 여부는 해당 source와 실행 증거로 확인한다.

> **정책 구현 완료 (2026-08-12):** 연구도 폐기, 전투 기반 스킬 파편 획득, Elite/NormalBoss/
> FinalBoss encounter progression, BossOmen, 고정 Standard 튜토리얼, 영구 PlayerProfile과 숙달 Elite
> 소탕은 Core·live RON·game_server 개발 경계에 구현됐다. 현재 콘텐츠는 게임 구동과 정책 검증에
> 필요한 최소 bootstrap이며 최종 밸런스나 콘텐츠 확정을 뜻하지 않는다. Unity 개발 트리는 이번
> 구현·검증 범위에서 제외됐다.

## 게임 개요

이 장은 플레이어가 접하는 게임의 정체성, 모드, 런의 기본 순서와 시작 roster 구성을 소유한다.
command field나 runtime 저장 구조는 의미를 설명하기 위한 seam이며 정확한 구현 계약은 인접 canonical을 따른다.

### 게임 정체성

이 게임은 환상체를 소유 기물로 쓰는 게임이 아니라, 소수의 직원을 이끌고 봉쇄된 시설을 탐사하는 로그라이크 전술 전투 게임이다.

핵심 판단은 아래 네 가지다.

- 어떤 직원과 장비, 스킬 파편으로 다음 노드에 들어갈지 정한다.
- 노드 브리핑과 위협 경고를 읽고 배치와 전투 계획을 세운다.
- `DefenseRoute` 전투 중 전술 포인트 기반 배치, 철수, 수동 스킬, 후퇴, 일시정지, 배속을 조작한다.
- 보상, 장비/파편 성장, 직원 손실 위험을 관리하며 다음 노드를 고른다.

현재 공식 전투 축은 명일방주식 실시간 배치 방어인 `DefenseRoute` 하나다. 보스 전투도 별도 전투 장르를 되살리기보다 `DefenseRoute` 파생으로 설계하는 방향을 우선 검토한다.

### 게임 모드

게임은 상위 구조로 `Standard`와 `Endless`를 가진다. 두 모드는 같은 전투·직원·장비·스킬 파편
기반 규칙을 사용하지만 서로 독립된 run이며 목적, 맵 생성과 진행 정책이 다르다.

Runtime은 런 생성 시점부터 `GameMode`를 명시적으로 저장해야 한다. `GameMode`는 `StartNewGame`
command의 필수 입력이며, 직원 선발 완료 후 생성되는 `RunProgression`에 그대로 저장된다. `GameMode`는
노드맵 생성, `Gate` 배치, 다음 Floor 전환, 클리어/실패 조건의 source of truth다. 모드별 규칙을
generator나 UI 추론으로 흩어 두지 않는다. 모드 선택 화면에는 Endless를 처음부터 표시하지만,
영구 profile의 튜토리얼 완료 플래그가 없으면 진입을 거부하고 완료 조건을 안내한다.

Standard:

- 본편에 들어가기 전 게임의 대략적인 흐름과 조작을 알려 주는 고정형 튜토리얼이다.
- 랜덤 생성 Floor를 사용하지 않고, 세 Floor 모두 미리 작성된 고정 맵과 진행 순서를 사용한다.
- 전투 난이도도 튜토리얼 RON이 작성한 고정값을 사용한다. Certified 전투력, run peak나 Endless
  Floor anchor로 적 스탯·웨이브·보상을 재조정하지 않는다.
- Floor 1의 핵심 전투는 일반 침식체 `Normal` node다.
- Floor 2의 핵심 전투는 `Elite` 환상체 node다.
- Floor 3의 최종 관문은 간소화된 `NormalBoss` 환상체 node다. `FinalBoss`는 출현하지 않는다.
- Floor 3의 일반 보스를 승리하면 Standard run을 완료하고 영구 튜토리얼 완료 플래그를 기록해
  Endless 진입을 해금한다.
- 튜토리얼 도중 authored script가 임의의 스킬 파편을 지급해 장착·사용·성장을 체험시킬 수 있다.
  지급 시점과 정확한 fragment ID는 튜토리얼 RON이 명시한다. 이 지급은 Endless의 정규 파편 획득
  규칙이 아니며 Standard run 밖으로 전달되지 않는다.
- 튜토리얼 파편도 일반 파편처럼 즉시 분해할 수 있다. 필수 학습 전에 분해해 이후 진행 조건을
  만족하지 못하면 자동 재지급·보호·rollback 없이 Standard를 실패하고 새 run으로 재시도한다.
- Standard의 Elite/Boss 승리는 Endless의 격파 횟수, 숙달, 모노리스와 정규 파편 획득 조건에
  포함하지 않는다.
- 장기 보스 징조, 모노리스 반복 성장, Elite pool 순환과 소탕은 Standard 콘텐츠가 아니다.
- Standard 완료 뒤에도 모드 선택에서 다시 시작할 수 있다. 반복 완료는 추가 전역 보상을 지급하지
  않지만 실제 조우·격파한 침식체와 환상체는 일반 규칙대로 전역 도감에 기록한다.

Endless:

- Standard 완료 뒤 진입하는 본편이며 장기 생존과 심화 콘텐츠를 담당한다.
- Floor가 계속 이어지며, 플레이어는 누적되는 손실과 보상을 관리한다.
- 특정 보스가 `provisional_boss`로 떠오를 수 있고, 기존 노드 위에 보스 징조 source가 얹힐 수 있다.
- 플레이어가 같은 보스 계열의 징조 source를 반복 선택하면 보스 체인이 확정되거나 진행도가 누적될 수 있다.
- 조건이 완성되면 보스 방이 생성되거나, 일부 체인은 다음 노드 선택지를 잠그는 강제 보스 방으로 이어질 수 있다.
- 다음 층 이동 source는 `Gate` 노드다. 끝없는 탐사에서는 `Gate`의 부모가 엘리트, 보스, 일반 노드 중 무엇이든 될 수 있다.
- 기본적으로 한 층에는 `Gate`가 하나지만, 특수 층에서는 여러 `Gate` 후보를 허용할 수 있다.
- 조건부/Locked Gate, 비밀 Gate, 완전 숨김 Gate는 시스템상 미래 확장 후보지만 기본 live data에 임의로 작성하지 않는다. 그런 노드는 명시 정책이 있을 때만 추가한다.
- 고요한 오케스트라 같은 장기 보스 기믹, 특수 보스 파편, 고위험 보상과 누적 리스크는 끝없는 탐사 콘텐츠다. 백야의 상세 체인은 향후 정책부터 다시 승인할 설계 후보다. 개발 중 generic BossOmen end-to-end 검증에는 현재 one-step `white_night` chain과 `suppress_white_night`를 유일한 live bootstrap FinalBoss로 사용하며, 이는 최종 백야 콘텐츠 승인이 아니다. 이 하나를 격파하면 그 시점의 live 집계 대상 FinalBoss 전체 격파 기록을 달성한다.
- 숙달한 Elite, NormalBoss와 FinalBoss의 각 반복 pool은 아래의 cycle 규칙으로 계속 순환한다.
- 자동 `RunComplete`는 없다. 모든 FinalBoss와 숨겨진 보스를 격파해도 Floor를 계속 생성하며,
  플레이어의 자발적 포기 또는 복구할 수 없는 Run 실패만 Endless를 종료한다.

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

Standard에서는 고정된 세 Floor를 순서대로 학습하고 Floor 3의 간소화된 일반 보스를 클리어하는
것이 종료 조건이다. Endless에서는 `Gate`를 통한 다음 Floor 이동, 보스 징조 체인, 누적 손실 관리와
숙달 encounter의 반복 성장이 핵심 진행을 만든다. Endless는 자동 완료 없이 Floor를 계속 생성한다.

전투 노드는 한 번의 전투로 끝나지 않을 수 있다. 일반 전투, 엘리트, 일반 보스, 최종 보스는 같은 이상현상 시도 횟수 정책을 따르되, 최종 보스는 패배 판정이 즉시 런 실패라는 예외를 가진다. 패배 판정 전 전투 포기는 진입 직전 상태를 복원하므로 남은 시도를 줄이지 않는다.

### 시작 직원 선발

런 시작 시 플레이어는 시작 직원 후보 중 출전할 직원을 고른다.

- 모든 시작 후보와 본사 채용 후보는 하나의 authored `EmployeeDefinition` catalog에서 온다. `starter_eligible`은 시작 후보 자격이며, 시작 때 선택하지 않은 starter definition과 모든 non-starter definition은 같은 런의 미합류 채용 queue에 들어간다.
- `employee_definition_id`는 콘텐츠에서 안정적인 authored identity다. roster 합류 뒤 생성되는 `uuid`는 해당 런 안의 직원 instance identity이며 둘을 서로 대신 사용하지 않는다.
- definition은 이름, 역할, 배경, initial loadout, personality의 risk/social 두 축, `CombatAptitude`, 기본 HP/공격/방어/마법 저항, `basic_attack_speed_percent`를 명시한다.
- risk disposition은 `Brave`, `Cautious`, `SurvivalInstinct`, social disposition은 `Obedient`, `Defiant`, `Comradely` 중 각각 정확히 하나다. personality는 UUID나 신뢰 상태에서 추론하지 않는다.
- `CombatAptitude`는 `Balanced`, `Defensive`, `Offensive`의 가시성 정보다. 현재 장비 제한이나 불일치 페널티를 만들지 않는다.
- 직원 기본 공격 속도 100은 중립이다. 무기가 기본 공격 주기를 소유하고 최종 주기는 `ceil(weapon_base_interval_ms * 100 / basic_attack_speed_percent)`다. 비무장 직원은 roster와 safe-zone 장비 관리에는 남지만 battle gameplay source가 될 수 없으므로 기본 직원 공격 주기로 보완하지 않는다.
- specialties는 현재 production schema/runtime/DTO에 없다.
- 미합류 직원 queue 순서는 run seed로 한 번 결정한다. 본사 연락은 queue head에서 authored 후보 수만큼 active session으로 옮기며 snapshot 재요청이나 재접속으로 다시 뽑지 않는다.
- 채용한 definition은 새 run-local UUID와 함께 roster로 이동하고, 선택하지 않은 후보는 제안된 순서 그대로 queue tail에 돌아간다. 응급지원이나 본사 상점을 선택하면 현재 후보 전부가 같은 순서로 tail에 돌아간다.
- 같은 definition은 queue, active recruitment session, roster 중 정확히 한 곳에만 존재한다. checkpoint load는 이 소유권과 queue 순서를 검증·복원하고 Floor 전환도 그대로 유지한다.
- starter-eligible 6명은 일반 보상 pool에 들어가지 않는 `standard_training_sword`와 각 authored initial
  loadout을 받는다. 시작 직원은 이후 런 전체의 핵심 자원이다.
- 직원은 장비, 스킬 파편, run-local Psychology, physical damage, trauma를 통해 장기적으로 관리된다.
- 무기를 장착한 출전 가능 직원이 없고 실제로 출전 가능성을 복구할 경로도 남아 있지 않으면 런 실패다.
  무기를 지급하지 않는 본사 채용은 그 자체로 비무장 roster를 복구하는 전투 경로가 아니다.

## 탐사와 노드

이 장은 플레이어가 Floor를 탐색하고 node content를 선택·해결하는 규칙을 소유한다. Snapshot field,
RON field와 validation 세부는 각각 외부 Unity canonical, [data_loading_contract.md](../data_loading_contract.md),
[core_runtime_contract.md](../core_runtime_contract.md)의 경계 요약으로만 읽는다.

### Floor 맵과 노드 선택

Floor 맵은 봉쇄된 시설의 방과 복도를 탐사하는 지도다. 플레이어는 임의 node로 순간이동하지 않고,
core가 현재 위치와 authored facility connection으로 계산한 도달 가능한 node만 선택한다.

- Standard와 Endless는 같은 facility engine을 사용한다. Standard는 Floor별 고정 authored
  template·assignment를 사용하고 랜덤 pool에서 맵이나 핵심 encounter를 뽑지 않는다. Endless는
  Mode/Floor별 live schedule이 template pool과 assignment profile을 고르며, pool의 deterministic
  순서·cycle·사용 이력은 run/checkpoint 상태다.
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

- Elite/Boss가 Gate 역할을 겸하지 않는다. Standard Floor 1~2의 Floor 전환과 Gate 위치·해금 조건은
  고정 튜토리얼 맵이 직접 작성한다.
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

- 이 절의 적응형 Floor 난이도는 Endless에만 적용한다. Standard는 certified 전투력을 readiness
  표시에 활용할 수 있지만 authored 튜토리얼 전투의 난이도에는 합성하지 않는다.
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
- 장비 강화 및 파편 개화·분쇄는 Safezone이 아니라 Maintenance가 소유한다.

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
- 승리하면 Event가 완료된다. 패배 결과를 commit하면 해당 Event 전투는 즉시 실패로 종료되고 같은
  Event에서 재시도할 수 없다. 결과 확정 전 battle retreat는 공통 rollback 규칙을 따른다.
- Event source를 사용하는 `BossOmen` 전투는 이 일반 Event 완료 규칙을 상속하지 않는다. 첫 omen과
  active chain 후속 omen의 실패 처리는 아래 `FinalBoss pool과 BossOmen Chain` 절이 소유한다.

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
타겟 판정 알고리즘은 [skill_target_contract.md](../skill_target_contract.md), event/checkpoint/result 생산자
불변식은 [core_runtime_contract.md](../core_runtime_contract.md), 정확한 wire shape와 message order는 외부
Unity canonical이 소유한다.

### 전투 모드

현재 live 전투 모드는 명일방주식 실시간 배치 방어인 `DefenseRoute` 하나다.

- 플레이어 유닛은 배치 위치에서 route 적을 공격하고 `block_capacity`만큼 지속 저지한다. 거리
  이탈만으로 engagement가 풀리지 않으며 사망·철수·unblockable 전환·강제 이동 같은 명시
  조건에서만 해제된다.
- 모든 필수 wave·적 처리와 보호 대상 생존이 성공 조건이며 보호 대상 파괴나 scenario 조건이
  실패를 만든다. Pause, 배속과 battle retreat는 server simulation 기준이다.
- 일반 전투와 보스는 같은 transport, 전장, 배치와 wave 모델을 공유한다.
- 생존 전투는 별도 `Encirclement` mode가 아니라 `DefenseRoute / Defense + survive_timer_ms`로
  작성하며 timer 도달 시 남은 적과 무관하게 승리한다.

`DefendPoint`, `Encirclement` mission variant, `Recovery`, `DefendAndEscape`,
`TacticalGroupPlan`과 formation AI는 현재 live 계약이 아니다. `SplitRoom`은 전투 mode가 아니라
필요할 때 battlefield template로 표현한다.

### 전술 포인트

플레이어 표시명은 `전술 포인트`, code/DTO 이름은 `tactical_resource`다. Active battle마다 player-side
전역 pool 하나를 만들며 첫 live 값은 current `20`, maximum `99`, authoritative battle time 1초당 `1`
자연 회복과 최초 배치 base cost `10`이다.

- Current는 항상 `0..=maximum`이고 maximum은 battle start에 고정된다. 자연 회복과 명시 skill effect는
  초과분이나 감소분을 저장하지 않고 clamp한다. Maximum 변경 effect, encounter별 override, debt, 부분
  지불과 암묵적인 환경 drain은 없다.
- 최초 배치와 회수/전투불능 뒤 재배치는 같은 pool에서 전액을 보유할 때만 비용을 한 번 지불한다. 실패한
  command는 resource, deployment, salt, cooldown, event와 revision을 바꾸지 않는다. 철수는 비용도 환불도
  없다.
- Defense 직원의 전술 이동은 `5`를 한 번 지불하며 직원별 battle-time `10,000ms` cooldown을 시작한다.
  `DeployCostReduction`은 이 비용에 적용하지 않는다.
- 개별 skill은 `ModifyTacticalResource(amount)`로 signed current delta만 명시할 수 있다. DefenseFocus와
  Rest는 전술 포인트를 생산하지 않는다.
- 자연 회복은 simulation battle time의 온전한 1초 cadence를 따른다. Pause wall time에는 회복하지 않고
  Resume 뒤 catch-up하지 않으며 maximum에서 넘친 회복도 bank하지 않는다.
- Checkpoint/resync와 root InBattle snapshot은 같은 battle-local current/maximum/recovery를 반환한다.
  전투 종료와 Retreat는 pool을 폐기하고 다음 battle은 initial `20`에서 다시 시작한다.

### Hard CC와 Silence

전투 제어 제한은 command마다 `Stun`/`Freeze` 이름을 다시 나열하지 않고 현재 active buff의 분류에서
파생한다. 현재 `Stun`과 `Freeze`는 Hard CC, `Silence`는 Silence capability를 제공한다. 서로 다른 active
instance가 같은 capability를 제공하면 하나라도 남은 동안 제한이 유지되며, Hard CC 해제 시각은 남은
Hard CC의 가장 늦은 expiry다.

- Hard CC는 새 automatic 기본 공격과 automatic/manual skill 시작을 막는다. 적용 순간 아직 release되지
  않은 기본 공격 windup과 skill cast/focus는 취소하고 해당 action이 소유한 lock도 함께 해제한다.
- 이미 release된 기본/skill projectile, delayed skill step, persistent area, buff 적용과 확정된 파생
  effect는 caster의 Hard CC와 무관하게 기존 lifecycle을 계속한다. 이미 소비한 resonance와 시작한 proc
  cooldown은 환불하거나 되감지 않는다.
- Hard CC 동안 보행을 멈추고 새 이동을 막으며 blocker 또는 blocked enemy가 Hard CC이면 engagement를
  즉시 해제한다. Positive resonance gain도 중단한다. 피해, periodic effect, passive/reactive trigger와
  battle-time cooldown은 계속 흐른다.
- Silence는 새 automatic/manual skill만 막는다. 이미 시작한 skill과 기본 공격, 이동, block, damage,
  resonance gain과 철수는 그대로 유지한다.
- Manual skill의 typed 거부 reason은 Hard CC에 `hard_crowd_control`, Silence에 `silenced`를 사용한다.
  둘이 겹치면 Hard CC reason이 우선한다. 실패한 시작 요청은 resonance, cooldown, event와 battle revision을
  바꾸지 않는다.
- 자발적 철수는 두 capability와 무관하게 허용하며 아래 철수 lifecycle을 그대로 따른다.

### 철수와 재배치

Active 배치 player unit은 태세, 기본 공격 windup, skill cast/focus, Hard CC, Silence와
`FieldWorking` 여부에 관계없이 자발적으로 철수할 수 있다. 사망·전투불능·이미 철수한 unit은
철수할 수 없고, 실패한 요청은 전투 상태나 revision을 바꾸지 않는다.

- 철수는 전술 자원을 소비하거나 이미 지불한 배치 비용을 환불하지 않는다. 해당 runtime instance를
  즉시 `Withdrawn`으로 전환해 official field/checkpoint에서 제거하고 engagement를 해제한다.
- Release 전 기본 공격과 skill cast/focus는 취소한다. 이미 발사된 기본/skill projectile과 이미 생성된
  `GroundFixed` area는 독립 delivery로 원래 lifecycle을 계속한다. 철수 unit을 향해 비행 중인 hostile
  projectile은 target이 inactive이므로 miss한다.
- `FollowCaster`, `FollowTarget`, aura, channel/focus처럼 source runtime에 종속된 효과는 철수와 함께
  종료한다. 철수 unit이 적용했거나 자신에게 적용된 buff/debuff는 각각 `CasterWithdrawn` 또는
  `TargetWithdrawn`으로 종료한다.
- 철수 재배치 HP는 철수 순간
  `min(max_hp, current_hp + floor(max_hp * 30 / 100))`으로 잠근다. 대기 중 회복이나 후속 condition/max HP
  변화는 이 값을 소급 변경하지 않는다.
- 전투불능은 runtime instance마다 physical damage `+1`과 authored trauma를 정확히 한 번 attempt-local
  condition에 반영한다. 최신 condition이 combat eligible일 때만 재배치 후보가 되며 시작 HP도 그
  condition에서 다시 계산한다.
- 재배치는 같은 employee identity의 새 runtime instance를 만들고 현재 persistent condition,
  확정 loadout/fragment, 유효한 battle-scoped modifier와 배치 초기 효과를 다시 구성한다. 이전 instance의
  buff, resonance, target, cooldown, cast/action, block과 aura는 상속하지 않는다. `InitialSkillCharge`와
  authored on-deploy effect는 매 배치 다시 적용한다.
- 자발적 철수/전투불능 대기는 각각 battle time `30초`/`90초`다. 재배치 base cost는
  `ceil(base_deploy_cost * 150 / 100)`이고 `DeployCostReduction`은 최초 배치와 재배치에 모두 적용한다.
  Pause 중 대기는 진행하지 않으며 checkpoint/resync는 잠긴 HP, ready time과 cost를 그대로 투영한다.

### Attack/Defense 태세와 타일 효과

Active 배치 직원은 authoritative `Attack | Defense` 태세 하나만 저장한다. 최초 배치와 모든 재배치는
`Attack`으로 시작하고 그 battle time부터 직원별 `1,500ms` 전환 cooldown을 시작한다. 양방향 전환은
비용이 없고 같은 cooldown을 사용한다.

- 같은 태세 요청은 `already_active`, cooldown 중 반대 태세 요청은 `stance_cooldown`, Hard CC 중 전환은
  `hard_crowd_control`로 거부한다. Silence, 기본 공격 windup과 skill cast/focus는 전환 자체를 막지 않는다.
  실패는 태세, 효과, cooldown, 전술 포인트, event와 revision을 바꾸지 않는다.
- `Attack`은 평상시 기본 공격과 automatic/manual skill 시작을 허용하지만 즉시 공격을 강제하지 않는다.
  `Defense`는 새 regular 기본 공격과 모든 automatic/manual skill 시작을 막는다. 전환 전에 시작한 windup,
  cast/focus, projectile, delayed/proc/multihit와 `ExtraAttack`은 취소하지 않고 완료하며 passive/reactive
  triggered effect도 계속 동작한다.
- `Defense`의 효과는 별도 저장 state가 아니라 현재 tile과 장착 무기 profile에서 파생한다. Ground에서는
  `DefenseFocus`, Platform에서는 `Rest`이고 `Attack`이면 어느 tile에서도 파생 효과가 없다.
- Ground DefenseFocus는 authored base block에 `block_capacity_bonus`를 더하고 general incoming damage
  reduction을 type별 reduction과 가산한 뒤 최종 `0..=100%`로 clamp한다. 해제 즉시 base 값으로 돌아간다.
  Capacity 감소 시 기존 engagement는 route progress 내림차순, spawn order, unit ID 순으로 남기며 새 적이
  기존 적을 밀어내지 않는다. Hard CC 동안 effective block은 0이지만 damage reduction은 유지된다.
- Platform Rest는 Defense 진입 뒤 온전한 `1,000ms`마다 자신의 HP `3`, resonance `5`를 각각 최대치로
  clamp 회복한다. full interval 전 첫 tick, recently-hit 제한, revive, 전술 포인트 회복과 catch-up은 없다.
  Attack 전환이나 Rest 조건 상실은 즉시 중단하고 다시 조건을 얻으면 새 full interval을 요구한다.
- 같은 timestamp의 모든 확정 피해가 Rest보다 먼저다. 그 피해로 사망하면 Rest가 되살리지 않는다. Hard CC는
  HP/resonance Rest를 모두 건너뛰되 cadence는 유지하고, resonance gain lock은 resonance만 건너뛴다. Skipped
  tick은 이후 보상하지 않으며 실제 변화가 0이면 HP/resonance event를 만들지 않는다. Pause는 battle clock과
  cadence를 함께 동결한다.

### 전술 이동과 아군 점유

`TacticalRelocateUnit`은 Defense 상태인 active 배치 직원의 같은 runtime instance를 player deployment
zone 안의 새 position/facing으로 즉시 옮긴다. 철수·재배치, 보행, pathfinding, 예약 또는 도착 state가 아니다.

- Source는 Defense여야 한다. Hard CC와 release 전 기본 공격 windup/skill cast-focus는 막지만 Silence와
  현재 block engagement는 막지 않는다. 같은 position+facing은 `already_active`, 같은 position에서 facing만
  바꾸는 요청은 정상 이동과 같은 비용/cooldown으로 허용한다.
- Destination은 valid/non-obstacle player deployment tile이고 현재 무기 `deployment_affinity`와 맞아야 한다.
  Active player employee와 defense object는 각자 logical tile 하나를 점유하므로 최초 배치, 모든 재배치와
  전술 이동은 다른 아군이 점유한 tile을 `position_occupied`로 거부한다. 이동 source 자신의 현재 tile은
  검사에서 제외하고 enemy와의 tile overlap은 허용한다. Occupied tile을 자동 push/swap하지 않는다.
- 성공은 모든 검증 뒤 position/facing, 직원별 `10,000ms` cooldown, 전술 포인트 `5`, engagement/target와
  tile effect를 한 번에 반영한다. HP, buff, resonance, Attack/Defense, stance cooldown, runtime identity와
  이미 시작된 action/delivery cooldown은 보존한다. 실패는 이 상태와 event/revision을 전혀 바꾸지 않는다.
- 기존 block engagement를 해제한 뒤 새 위치에서 deterministic하게 다시 계산한다. Range, aura/area 포함
  관계도 새 위치를 사용한다. 이미 release된 homing projectile은 이동한 target을 계속 추적하고 fixed
  projectile과 `GroundFixed` area는 원래 좌표를 유지하며 `FollowCaster`/`FollowTarget`은 새 좌표를 따른다.
- Ground↔Platform 이동은 두 tile profile을 모두 가진 `any` 무기만 가능하다. Defense를 유지한 채 old
  DefenseFocus/Rest를 제거하고 새 효과를 원자 적용한다. Platform block은 0이고 Ground 복귀는 authored
  base+bonus로 재계산한다. 새 Platform Rest는 이동 시점부터 full `1,000ms`를 기다리며, 같은 tile의
  facing-only 이동은 기존 Rest cadence를 재시작하지 않는다.
- 철수/전투불능 뒤 재배치는 새 runtime instance이므로 이전 instance의 전술 이동 cooldown을 상속하지 않는다.

`TacticalSwapUnits`는 서로 다른 active 배치 직원 두 명의 현재 position을 별도 destination payload 없이 한
transaction에서 맞바꾼다. Occupied relocation을 swap으로 바꾸거나 swap 실패를 relocation으로 바꾸는 fallback은 없다.

- 두 직원 모두 Defense이고 각자의 전술 이동 cooldown이 준비되어야 하며, Hard CC, release 전 기본 공격
  windup/skill cast-focus와 exclusive movement segment가 어느 한쪽에라도 있으면 전체를 거부한다. Silence와
  기존 block engagement는 허용한다.
- 각 직원은 상대의 current tile에 대해 player zone, valid/static tile, weapon affinity와 Ground/Platform
  profile을 전부 통과해야 한다. Occupancy 검사에서는 두 participant만 함께 제외하므로 그 tile의 제3 active
  player employee/defense object는 전체 swap을 `position_occupied`로 거부한다.
- Command는 각 직원의 새 facing을 명시한다. Server는 기존 facing을 자동 복사하거나 서로 교환하지 않는다.
  성공은 전술 포인트 `5`를 command당 한 번 지불하고 두 직원의 cooldown을 같은 `time+10,000ms`로 설정하며
  event와 battle revision도 각각 한 번만 증가시킨다.
- 양쪽 engagement를 먼저 해제하고 두 body/facing/tile effect를 모두 commit한 뒤 block/target/range/area를
  한 번 재계산한다. 따라서 중간 duplicate/empty occupancy, sequential relocation, walking/reservation state는
  관찰할 수 없다. HP, buff, resonance, stance, action cooldown, runtime identity와 release된 delivery는 각
  직원에게 남는다. Ground↔Platform swap도 Any profile의 effect/Rest epoch를 양쪽 최종 tile에서 계산한다.
- 어느 preflight나 resource 검사가 실패해도 양쪽 position/facing/effect/cooldown, resource, event와 revision은
  전혀 바뀌지 않는다.

### 수동 지정 스킬

모든 live skill은 effect 대상인 `SkillTarget`과 별도로 선택 주체 `SkillCastSelection` 하나를 필수로
명시한다. 허용 값은 `self`, `automatic_unit`, `player_unit`, `player_tile`이며 weapon archetype, 첫 step,
effect target에서 선택 주체를 추론하거나 default/fallback하지 않는다.

- `self`는 caster 자신, `automatic_unit`은 Core의 authored target rule을 사용하며 별도 targeting focus 없이
  `ActivateSkill`을 즉시 제출한다. 이 두 mode에 explicit target을 보내면 `skill_target_type_mismatch`다.
- `player_unit`과 `player_tile`은 각각 explicit `Unit`/`Tile` payload가 필요하다. 누락은
  `skill_target_required`, 종류 불일치는 `skill_target_type_mismatch`이고 다른 target kind나 automatic
  target으로 보정하지 않는다.
- PlayerUnit은 active hostile lifecycle, air eligibility, effect usefulness와 최종 reach/range를 검사한다.
  PlayerTile은 valid battlefield tile과 reach/range만 검사하며 occupant를 요구하지 않는다. Caster의 manual
  mode, Attack stance, action lock, Hard CC/Silence와 full resonance도 target보다 먼저 모두 통과해야 한다.
- PlayerUnit/PlayerTile 선택은 authoritative tactical focus를 시작한다. Unpaused battle은 focus 동안 effective
  speed `X0_5`, 이미 Pause인 battle은 frozen 상태를 유지한다. Focus는 purpose/caster/skill과 종료 때 복원할
  playback만 저장하며 hover와 미제출 candidate를 저장하지 않는다.
- `PreviewSkillTarget`은 current revision에서 candidate 하나를 읽기만 하고 event/revision을 만들지 않는다.
  최종 `ActivateSkill`은 같은 payload를 다시 검증한다. 실패는 focus, playback, resonance, event와 revision을
  전부 유지하고 Cancel은 비용 없이 focus를 끝내며 이전 playback을 정확히 복원한다.
- 성공은 captured Unit/Tile로 manual cast를 시작하고 focus 종료/playback 복원을 같은 revision commit에
  반영한다. 이후 selected unit이 무효가 된 direct effect는 no-op하며 다른 unit을 찾지 않는다. Unit 기반
  TileArea와 PlayerTile area는 acceptance 시점 tile을 유지한다. 이후 무효화나 빈 tile에도 refund/rollback은
  없고 `PlayerUnit|PlayerTile + RetargetOnStep` authoring은 거부한다.
- Checkpoint/resync는 focus와 restore playback을 그대로 반환한다. Pause에서 시작한 focus는 성공/취소 뒤에도
  Pause다. Active-skill 전용 추가 cooldown이나 수동 기본 공격은 없다.
- 첫 live PlayerUnit path는 `fragment_freischutz_black_round`와 awakened progression의
  `freischutz_magic_bullet`뿐이다. 기존 range/focus/damage/delivery tuning은 유지하며 live PlayerTile skill은
  아직 없다.

### 현장 작업

현장 작업은 tactical point, static defense object 또는 tile kind를 재사용하지 않는 별도 setup object다.
Incomplete object의 authored work cell을 active Player 직원이 최종 점유하면 별도 command나 비용 없이 같은
mutation에서 자동 assignment된다. 직접 배치와 재배치도 work cell이면 즉시 `Defense + FieldWorking`으로
시작한다.

- Progress는 직원이 아니라 object가 소유하며 interruption policy는 `Preserve` 하나뿐이다. 각 eligible worker는
  흐른 battle time `1ms`마다 `1ms`를 기여하고 여러 cell의 기여는 합산된다. Required work에서 clamp하며 worker가
  0명이 되어도 줄거나 초기화되지 않는다.
- `FieldWorking`은 regular 기본 공격과 automatic/manual skill 시작, block, 태세 전환, Ground DefenseFocus와
  Platform Rest를 막는다. Worker는 피해를 정상적으로 받고 enemy overlap/attack은 assignment를 끊지 않는다.
  자발적 철수, 전술 이동과 자리 교대는 계속 허용된다.
- Hard CC는 assignment와 누적 progress를 보존하되 active 동안 그 worker의 기여만 멈춘다. 해제 뒤 놓친 시간을
  보충하지 않는다. Pause는 simulation clock 자체를 멈추므로 기여도 진행하지 않고 Resume 뒤 catch-up하지 않는다.
- 철수·사망·이동·교대는 모든 position commit이 끝난 최종 점유에서 assignment를 한 번 reconcile한다. Cell을
  떠난 직원은 `Ready + Defense`와 현재 tile의 일반 DefenseFocus/Rest로 복귀하고, 새 적격 직원은
  `FieldWorking`이 된다. Completed cell은 더 이상 assignment하지 않고 underlying tile로 돌아간다.
- 최초 required 도달 update에서 object는 정확히 한 번 Completed가 되고 typed effect를 한 번 실행한다. Full HP
  repair도 완료·소모되며 재사용하지 않는다. Assignment start/stop과 completion만 semantic event이고 현재
  progress/status/worker mapping은 checkpoint/reconnect가 소유한다.
- 첫 live object는 `suppress_freischutz`의 `black_box_emergency_repair`다. Presentation은
  `EmergencyRepairTerminal`, object와 단일 Ground work cell은 `(2, 6)`, required work는 `10,000ms`, completion은
  보호 대상 `black_box_device` HP를 최대 `100` 회복한다.

`AssignFieldWork`, `CancelFieldWork`, 별도 work cost, Reset mode, generic scripting/status framework와 추가 live
object는 현재 계약이 아니다.

### Pause와 직렬 전투 명령

Pause는 전투 simulation clock만 정지한다. 현재 배치, 철수, 태세 전환, 전술 이동/교대, 수동 스킬 시작, 배속 변경, 전투 후퇴와
read-only 상태 요청은 Pause 여부만으로 금지되지 않으며, frozen battle time의 최신 상태에서 각 명령의
평상시 eligibility를 그대로 검사한다.

- 플레이어 mutation은 한 번에 하나씩 처리한다. 앞 command의 authoritative response와 최신
  `battle_state_revision`을 받은 뒤 다음 command를 보낸다. Local command plan/batch와
  `ResumeBattleWithPlan`은 없다.
- 성공한 command의 배치/철수, 자원 소비, skill cast 시작과 stored speed 변경은 Pause 중에도 즉시
  반영되고 revision을 1 증가시킨다. 실패는 state, resource, event와 revision을 바꾸지 않는다.
- 뒤 command가 실패해도 앞 command의 성공을 되돌리지 않는다. 같은 frozen time의 순서는 server actor
  처리 순서, revision과 event sequence가 소유한다.
- 적 이동/공격, pending cast resolve, buff duration, 재배치 대기와 자연 회복처럼 battle-time 진행이
  필요한 동작은 멈춘다. Resume 뒤 Pause 시간을 소급하거나 비축하지 않는다.
- Pause 중 배속 변경은 clock을 진행시키지 않고 Resume 뒤 사용할 stored speed만 바꾼다.
- State/resync와 preview query는 최신 Pause state를 읽되 revision을 증가시키지 않는다. Retreat 성공은
  active battle과 Pause state를 함께 종료한다.
- 연결이 끊기면 server가 active battle을 자동 Pause하고 시간 제한 없이 같은 process에 보존한다. 재접속은
  동일 상태를 paused로 받으며 명시적 Resume 전에는 진행하지 않는다. Server process가 종료되면 active
  battle을 복구하지 않고 마지막 완료 SavePoint로 돌아간다.

### 전투 시작과 Unity 경계

전투는 live battle state를 사용한다.

- Scene construction은 `battle_setup_snapshot`, 연출 timing은 `battle_update.events_delta`,
  현재 상태와 복구는 `battle_update.checkpoint`, 결과 통계는 `selected_event.result_stats`가
  source of truth다.
- Active event/result는 전투 종료 뒤 player history로 누적하지 않는다.
- Core가 공격·스킬 판정과 preview를 계산하며 Unity는 authored metadata에서 재계산하지 않는다.

생산자 불변식은 [core_runtime_contract.md](../core_runtime_contract.md), targeting은
[skill_target_contract.md](../skill_target_contract.md), JSON shape와 message order는 외부 Unity
canonical을 따른다.

### 전장과 배치

Node마다 route, 장애물, 배치/출현 구역이 다른 비사각형 전장을 사용할 수 있다.

- `battle_setup_snapshot`이 실제 scene source이고 `combat_preview`는 진입 전 briefing이다.
- 배치와 range 후보는 tile 기반, 이동과 충돌은 2D 연속좌표 기반이다.
- Player tactical slot의 logical allied occupancy는 Core active player combatant/defense-object position이
  소유하며 배치 preview와 최종 deploy/redeploy/relocation commit이 같은 검사를 사용한다.
- 크기는 `Small`, `Medium`, `Large`이며 Boss는 기본적으로 Large다.
- Archetype은 `OpenHall`, `Corridor`, `ChokePoint`, `Ambush`, `Surrounded`,
  `SplitRoom`, `ObstacleRoom`, `BossArena`의 데이터 분류다. `SplitRoom`은 현재 live
  encounter 배정 대상이 아니다.
- ASCII template/route authoring은
  [data_loading_contract.md](../data_loading_contract.md#battlefield-template-authoring), route와
  movement projection은
  [core_runtime_contract.md](../core_runtime_contract.md#route-and-battlefield-projection)가 소유한다.

### 적 구성과 웨이브

일반 wave의 주력은 `CorrodedEmployee`이고 `Abnormality`는 Elite/Boss/특수 encounter의 중심
위협이다. `FacilityEntity`는 아직 live 대상이 아니다.

Encounter 분류:

- `Normal`: 순수 침식 직원 전투. Primary abnormality와 환상체 bonus objective가 없다.
- `Elite`, `NormalBoss`, `FinalBoss`: 정확히 하나의 `primary_abnormality_id`를 가지며
  각각 elite, 일반 boss, final-boss급 encounter에 사용한다. Standard Floor 3은 간소화된
  `NormalBoss`이며 `FinalBoss`가 아니다.
- `ReferenceFinalBoss`: FinalBoss vertical path를 검증하는 명시적 nonproduction class다. Normal map,
  BossOmen pool/cycle, 파편/repeat reward, 모든 FinalBoss 기록과 일반 progression에 들어가지 않는다.
- 환상체 `threat_class`는 최소 Elite이고 Boss 환상체만 Boss다. 실제 spawn은 encounter
  classification이 아니라 wave data가 소유한다.

Wave는 `SpawnWave`로 관리한다.

- 각 Wave는 stable trigger ID, priority, closed primary trigger와 optional deadline을 명시한다. Primary는
  absolute battle time, 선행 group clear, bound actor HP/사망/지점 도달 중 하나다.
- 조건이 최초 binding 시 이미 참이면 같은 boundary에 한 번 발동하고, actor death는 binding 뒤 실제 death만
  관찰한다. Same-process reconnect는 trigger epoch를 유지해 재발동하지 않는다.
- `required_for_victory: false` wave는 승리 필수 대상이 아니다.
- Spawn 전 required Wave도 Victory를 보류하며 모든 required group이 실제 spawn·처리된 뒤에만 승리할 수
  있다. Deadline과 primary가 같은 시각이면 primary 원인을 사용하고 Wave 간 dependency cycle은 data 오류다.
- Manual source는 typed `CorrodedEmployee(profile_id, tier, count)` 또는
  `Abnormality(abnormality_id, tier, count)` entry를 사용한다.
- Preset 생성도 허용하지만 결과는 CombatPreview 때 `enemy_entries`로 확정하며 전투 중
  재생성하지 않는다.

#### Live Boss Runtime Reference

`suppress_little_red`는 production content에 opt-in한 교체 가능한 Boss Runtime reference다. 별도
`reference_cwbs_final_boss`는 conditional Wave, Summon, neutral signal, named outcome와 Victory registration을
연결하는 nonproduction reference다. 둘 다 최종 캐릭터 설계나 lore 계약이 아니다.

- `primary_abnormality` wave의 spawn index 0만 `little_red_reference_encounter`의 같은 이름 actor slot에
  bind한다. 이 wave는 기존 5초 timing과 구성에 `defense_main` route만 추가한다.
- Little Red는 spawn 뒤 reachable active hostile 중 minimum path-cost target 하나를 고정하고,
  `AcquireTarget -> MoveToTarget -> RejoinForwardRoute`를 한 번 실행한 뒤 기존 `RouteCombat`으로 돌아간다.
  Target 없음, invalidation, path failure와 timeout은 모두 authored rejoin/route exit를 따르며 implicit
  retarget, hidden retry 또는 두 번째 pursuit를 만들지 않는다.
- 기존 `f-01-57_little_red` stats, support wave, reward, 5초 wave timing과
  `little_red_hunt_the_prey` leaf skill은 이 reference 때문에 바뀌지 않는다. Automatic skill의 step-level
  targeting은 기존 skill runtime authority이고 Boss target lock을 재작성하지 않는다.
- 전용 bounty, Big Bad Wolf 연계, art/animation/VFX와 public Boss lifecycle은 없다. Existing opponent
  capsule, battle event와 checkpoint projection이 presentation/transport 경계다.
- CWBS reference의 summon은 독립 definition/limit/provenance와 exact batch를 가지며 cleanup signal로
  non-kill despawn한다. Summoned unit은 Wave/Boss/보상/파편/도감 authority가 아니고 승리를 막지 않는다.
- CWBS reference는 ID/class/display name으로 nonproduction임을 드러내며 authored reward, node bonus와
  progression을 모두 비활성화한다. Production 승격은 별도 content authoring/balance review가 필요하다.
- 다른 production encounter는 `boss_runtime`에 opt-in하지 않으며 기존 ordinary planner와 authored combat
  semantics를 그대로 사용한다.

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
[skill_target_contract.md](../skill_target_contract.md)다.

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

### Runtime 스탯·HP·사망 전이

전투 중 스탯 modifier는 causal unit이 있는 skill/장비 effect와 system/직접 command처럼 source가 없는
경우를 구분한다. Persistent battle-entry stat 조합은 이 runtime event 경로를 사용하지 않는다.

- 모든 runtime modifier는 산술과 현재 HP clamp를 한 번 계산하고 `StatChanged`를 한 번 기록한다. Skill은
  caster, 장비 trigger effect는 trigger owner를 `source_instance_id`로 보존하며 source 없는 command는
  `None`을 유지한다.
- MaxHealth 변경으로 current HP가 실제로 줄어들면 `StatChanged` 다음에 damage metadata가 없는
  `HpChanged(reason: Command)`를 정확히 한 번 기록한다. HP delta가 없으면 `HpChanged`를 만들지 않으며,
  MaxHealth 증가가 current HP를 자동 회복시키지 않는다.
- Clamp 결과가 0이면 같은 전이 안에서 기존 death finalization을 실행한다. Active 0 HP를 terminal state로
  남기지 않고 movement/action, buff, graveyard와 `UnitDied`, `OnDeath`/`OnKill`/`OnAllyDeath`를 일반 사망과
  같은 authority에서 한 번만 처리한다.
- Causal unit이 있으면 그 unit이 killer이고 일반 OnKill credit을 받는다. Source가 없는 system/직접
  modifier 사망은 killer와 OnKill credit이 없다.
- 관찰 가능한 인과 순서는 `StatChanged → HpChanged → 기존 death sequence`다. MoveSpeed modifier의 runtime
  body speed 갱신과 다른 스탯의 기존 산술·rounding은 그대로 유지한다.

## 성장, 스킬 파편과 도감

이 장은 전투와 node resolution 뒤 플레이어가 얻거나 누적하는 보상, 성장, 장비, 스킬 파편과
도감의 gameplay 의미를 소유한다. 수량 commit과 runtime transaction은
[core_runtime_contract.md](../core_runtime_contract.md), authored schema와 catalog validation은
[data_loading_contract.md](../data_loading_contract.md)가 소유한다.

### 보상과 성장

일반 Combat reward는 통화, 장비, 강화 재료와 직원 성장 보상을 지급할 수 있다. 스킬 파편은
아래의 환상체별 전투 조건 또는 Standard의 명시적 튜토리얼 script로만 지급한다. Ordinary Combat,
Reward node, Shop, HeadquartersContact와 일반 authored reward pool은 스킬 파편을 지급하지 않는다.
`ResearchProgress`는 폐기 대상이며 새 보상에 작성하지 않는다. Live encounter의 일반 자동 보상은
`ClaimAll`만 허용하며 `ChooseOne`은 현재 계약이 아니다.

직원 성장:

- 새 직원은 level 1, 현재 level XP 0에서 시작한다.
- XP requirement, 생존 XP와 battle tier는 `run/policy.ron`이 소유한다. 현재 모든 level은
  100 XP, 전투 결과 생존 직원은 승패와 무관하게 10 XP, tier는 level 1/3/6부터 I/II/III다.
- 같은 직원이 여러 번 배치돼도 생존 XP는 한 번만 지급하고 retreat에는 지급하지 않는다.
- 한 grant가 여러 level을 넘을 수 있으며 남은 XP는 다음 level에 유지된다. Battle tier는 현재
  stat multiplier가 아니다.
- Authored `GrantExperience.amount`는 분배할 total pool이 아니라 대상 직원 한 명당 지급량이다. 예를
  들어 `amount: 20`, 대상 직원 3명이면 각자 20 XP, 총 60 XP를 받으며 quotient/remainder 분배는
  존재하지 않는다. `AliveRoster`는 살아 있는 모든 직원, `CombatParticipants`는 명시된 전투 참가자,
  `SelectedEmployee`는 선택 직원, `AllRoster`는 생사와 무관한 모든 직원을 대상으로 하고 각 대상이
  전액을 받는다. 정확한 대상 집합과 staging은
  [core_runtime_contract.md](../core_runtime_contract.md#reward-and-employee-growth-boundary)가 소유한다.
- `GrowthStack`은 XP/level과 별도다. 현재 production producer는 없고 level-up으로 증가하지 않는다.

### 연구도 폐기와 스킬 파편 획득

기존의 기본 연구도, 환상체 대응 연구, 본사 분석·송신, `research_progress`,
`skill_fragment_progress`, `pending_research_deliveries`와 pending 수령 절차는 모두 폐기한다.
연구 완료를 스킬 파편 지급이나 Endless 종료 조건으로 사용하지 않는다.

게임에서 별도 수식어 없이 말하는 “파편”은 스킬 파편 하나만을 뜻한다. 환상체와 파편의 관계는
`환상체 1 : 스킬 파편 N`이다.

- 모든 환상체는 확장에 따라 두 개 이상의 고유 스킬 파편을 가질 수 있다.
- 각 스킬 파편은 정확히 하나의 환상체에만 대응한다.
- 일반적인 환상체는 파편 하나만 가진다. 파편 지급 source는 RON에 정확한 `fragment_id` 하나를
  명시하며 플레이어 선택, 무작위 추첨이나 대응 파편 일괄 지급을 하지 않는다.
- 미래에 한 환상체가 여러 파편을 가지더라도 각 획득 조건은 RON이 가리킨 파편 하나만 지급한다.
- 같은 run에서 해당 reward source의 파편 지급 완료 ledger가 이미 기록된 뒤 같은 조건을 다시
  만족하면 RON의 명시적 `repeat_reward`를 적용한다. 강화 재료, XP 등의 effect 또는 `None`을 직접
  작성하고 암묵적 대체 보상은 만들지 않는다.
- 유일한 예외는 숙달 Elite **소탕**이다. 해당 source의 지급 완료 ledger는 남아 있지만 RON의
  `fragment_id` 보유 count가 0이면 `repeat_reward` 대신 같은 파편 한 개를 재지급한다. 한 번의 소탕은
  재획득과 `repeat_reward`를 함께 지급하지 않는다. Count가 1 이상이면 기존 `repeat_reward`를 지급한다.
  직접 전투, NormalBoss와 FinalBoss에는 이 재획득 예외를 적용하지 않는다.
- 마지막 사본을 분쇄하고 이후 숙달 Elite를 소탕하는 분쇄→재획득 cycle은 횟수 제한 없이 허용한다.
  재획득은 지급 완료·숙달 ledger를 reset하지 않으며 identity별 강화·개화 진행도도 보존한다. Cycle의
  실제 빈도는 숙달 repeat pool 배정과 소탕 가능 조건이 제한한다.

Endless의 정규 획득 trigger는 encounter 등급으로 결정한다.

- `Elite`: 같은 run에서 동일한 Elite 환상체를 세 번째로 격파한 승리 결과에서 파편 지급 trigger가
  성립한다. 최초 3회는 직접 전투해야 하며 소탕으로 달성할 수 없다.
- `NormalBoss`: 해당 보스 node의 승리 결과에서 파편 지급 trigger가 성립한다.
- `FinalBoss`: 해당 보스 node의 승리 결과에서 파편 지급 trigger가 성립한다.
- `ReferenceFinalBoss`: 파편과 repeat reward trigger가 없으며 production reward source로 authoring할 수 없다.
- 지급 또는 `repeat_reward`는 전투 결과 commit에 포함하며 본사 수령을 기다리지 않는다.
- 파편과 획득 진행은 run-local이다. 한 run의 savefile은 다른 run의 파편, 격파 횟수와 숙달에
  영향을 주지 않는다.
- Standard가 중간에 지급하는 임의 파편은 사용법 학습을 위한 명시적 예외이며 위 trigger나 다른
  run의 소유권을 만들지 않는다.

### 영구 도감, 튜토리얼 해금과 개인 기록

게임은 플레이어가 승리한 attempt에서 실제로 조우한 환상체와 침식 프로필의 로컬 영구 도감을 유지한다.
이는 전투 history나 재생 기록이 아니라 canonical ID의 단일 `registered_ids` set이다.

- 전장에 실제 spawn된 적군 `Abnormality`/`CorrodedEmployee`만 후보다. 노드 preview, briefing, 아직 spawn되지
  않은 Wave와 `SummonedUnit`은 도감에 올라가지 않는다.
- Final Victory commit만 attempt의 모든 후보를 등록한다. 후보의 사망·생존, primary 여부와 개별 credit은
  관계없으며 Defeat, retry exhaustion, Retreat와 process 종료는 등록하지 않는다.
- 커밋된 set은 local-install profile의 단조 union이며 checkpoint load, 새 run, RunFailed, 일반
  non-battle transaction 실패와 active-run transaction rollback으로 사라지지 않는다.
  Run/checkpoint/transport player UUID와 분리되고, title의 확인된 전체 profile reset만 삭제할 수 있다.
- profile에는 ID set만 저장하고, 표시 이름·위험 등급과 authored 순서는 현재 live catalog에서 해결한다. 현재 catalog에서 사라진 dormant ID는 profile에 보존하지만 current snapshot에는 투영하지 않는다.
- `RunSnapshotDto.encounter_compendium`은 현재 catalog 전체를 authored 순서로 투영한다. 미등록 항목은 ID와
  `registered:false`만 가진 `???` placeholder이고 등록 항목만 metadata를 공개한다.
- 범위 안 consumer는 Core projection과 host의 `player_profile_persistence` 상태만 표시하며 battle event,
  preview, local catalog로 발견·격파·튜토리얼 완료·개인 기록을 재계산하지 않는다.
- 도감과 별도로 `standard_tutorial_completed`에 해당하는 영구 플래그 하나를 유지한다. Standard
  Floor 3의 간소화된 NormalBoss 승리를 최종 commit할 때만 false에서 true가 되며, true이면 Endless
  진입을 허용한다. 이 플래그는 run 성장이나 파편 소유권이 아니다.
- 영구 profile은 Endless에서만 집계하는 순수 기념용 개인 기록으로 `최고 도달 Floor`, `최장 무사망
  연속 Floor`, `한 run에서 모든 FinalBoss 격파`를 보존한다. Standard는 세 기록에 기여하지 않는다.
- 수치 기록은 기존 최고값을 넘는 즉시 갱신하고, 모든 FinalBoss 격파는 한 Endless run에서 달성
  시점의 non-empty BossOmen chain registry를 모두 완료하면 영구 달성한다. `ReferenceFinalBoss`는 이 집계에
  포함하지 않는다. 이후 새로운 production FinalBoss가
  추가되어도 이미 달성한 기록을 취소하지 않는다.
- 개인 기록은 active run savefile과 분리해 조건 달성 즉시 profile에 commit한다. 과거 checkpoint나
  run savefile load, Run 실패와 새 run으로 되돌리지 않으며 title의 확인된 전체 profile reset만
  삭제할 수 있다. 기록은 능력치, 장비, 재료, 재화, 파편과 콘텐츠 해금을 지급하지 않는다.
- 게임 전역에 보존되는 gameplay profile은 도감, 튜토리얼 완료/Endless 해금과 위 개인 기록뿐이다.
  직원, 장비, 재화, 파편, 격파 횟수, 숙달, 모노리스와 pool 상태는 active run savefile에만 속한다.

### Endless 환상체 격파와 반복 조우

Endless는 Elite/Boss/FinalBoss 환상체별 격파와 파편 지급 상태를 run-local ledger로 보존한다.
연구 point, requirement와 completion percentage는 갖지 않는다.

- Elite의 세 번째 승리는 해당 run의 숙달 조건을 만족한다.
- 숙달은 파편 보유와 분리된 격파 ledger다. 지급받은 파편을 장착·소비·분쇄하거나 더 이상 보유하지
  않아도 같은 run의 숙달과 소탕 자격은 유지한다.
- 파편 지급을 해결한 뒤 동일 환상체를 다시 격파하는 목적은 추가 성장 보상이다.
- 모든 환상체의 ledger를 채우거나 모든 FinalBoss를 격파해도 Endless를 완료하지 않는다.
- run-local pool, active chain, 숙달, 완료, 파편과 보상은 checkpoint에 포함한다. 과거 checkpoint를
  load하면 함께 rollback하고 전역 도감, 영구 개인 기록과 단조 증가하는 `loads_used`는 되돌리지 않는다.

#### Elite assignment pool

Elite 자연 배정은 `미숙달 pool`과 `숙달 반복 pool`을 명시적으로 구분한다.

- 미숙달 pool은 아직 자연 승리하지 않은 모든 Elite identity로 시작한다. 각 Floor의 Elite slot은
  same-Floor 중복 금지를 적용한 미숙달 후보로 먼저 채운다. 현재 Floor에 배정 가능한 서로 다른
  미숙달 후보 수가 Elite slot보다 적을 때만 남은 slot을 숙달 반복 pool 후보로 채운다.
- 자연 출현은 `Stage 1`이다. 자연 승리는 해당 identity를 미숙달 pool에서 제거하고 모노리스 진행을
  연다. 모든 node attempt를 소진해 현재 node가 패배 완료돼도 영구 실패로 기록하지 않고 미숙달
  pool에 남긴다. 같은 Floor 중복 금지는 유지하며 이후 Floor에서 자연 승리할 때까지 다시 후보가 된다.
- Floor에 배정됐지만 방문하지 않은 identity는 해결된 것이 아니며 다음 Floor의 미숙달 pool 후보로
  되돌린다.
- 자연 승리 뒤 모노리스 Stage 2·3까지 총 세 번 승리하면 숙달 반복 pool에 추가한다.
- 숙달 반복 pool은 첫 숙달 identity가 생기면 활성 상태를 유지하되 위 미숙달 우선 규칙 뒤의 남은
  slot에만 사용한다. 미숙달/숙달 양쪽에서 유효한 고유 후보가 없으면 authored Elite slot을 일반
  침식체 Combat으로 바꾼다.
- 숙달 반복 pool은 한 cycle에서 각 identity를 정확히 한 번씩 run seed 기반 결정론 순서로 배정하고,
  전부 소진하면 다시 채운다. 최근 Floor 제외는 없으며 cycle 경계에서 같은 identity가 연속 Floor에
  등장해도 허용한다.
- 숙달 identity도 Floor에 배정됐지만 직접 전투나 소탕으로 해결하지 않고 Gate를 통과하면 현재
  repeat cycle 후보로 되돌린다.
- 같은 Floor에는 동일한 Elite identity를 둘 이상 배정하지 않는다. cycle reset 뒤에도 현재 Floor에
  이미 배정한 identity를 다시 쓰지 않으며, 서로 다른 숙달 후보가 부족한 나머지 Elite slot은 일반
  침식체 Combat으로 바꾼다.

#### NormalBoss assignment pool

- NormalBoss pool은 한 cycle에서 각 authored identity를 한 번씩 해결한 뒤 전부 다시 채운다.
- 승리하면 현재 cycle에서 소진하고 해당 run의 지급 완료 ledger가 없으면 RON의 `fragment_id`, 이미
  있으면 `repeat_reward`를 지급한다. NormalBoss에는 소탕을 적용하지 않는다.
- 모든 attempt를 소진한 패배는 현재 cycle에서만 소진된 것으로 본다. 다음 cycle에서는 다시 후보가
  되어 파편 획득에 재도전할 수 있다.
- Floor에 배정됐지만 방문하지 않은 NormalBoss는 해결된 것이 아니며 현재 cycle pool로 되돌린다.

### Elite 모노리스와 강화 재현

Endless의 모노리스는 이미 자연 출현한 Elite를 더 강한 상태로 재현해 성장 보상과 세 번째 격파의
파편 지급 trigger를 노리는 선택형 콘텐츠다. 모든 Elite 모노리스를 수행할 의무는 없으며 플레이어가
필요한 성장과 파편에 따라 대상을 고른다. Standard에는 모노리스 규칙을 적용하지 않는다.

- 자연 Elite 최초 승리만 해당 환상체의 모노리스 구매 자격을 연다. 한 identity는 run당 한 번만
  구매할 수 있고 3 uses를 가진다.
- Endless Floor 1~3에는 떠돌이 상인이 없다. Floor 4+의 전용 상인은 첫 page 3칸에 eligible
  모노리스를 최대 2개 배치하고 나머지는 고위험 consumable로 채운다. 한 번의 유료 refresh는 page
  전체를 item-only 3칸으로 바꾼다.
- NodeConfirm은 순수 Available Normal Combat에서만 `모노리스 사용 (N)`과 환상체별 카드를 투영한다.
  최종 확정은 node 구조를 유지하고 encounter/difficulty를 다음 stage로 바꾸면서 1 use를 차감한다.
- 강화 재현은 Stage 2와 3이며 각 node에서 최대 2회 시도한다. 첫 패배는 같은 node를 재시도하고 모든
  attempt를 소진한 패배는 해당 identity를 run-local 영구 실패로 기록한다. 영구 실패 뒤에는 다음
  stage, 파편 지급, 숙달 반복 pool과 소탕을 열지 않는다. Retreat는 attempt를 소비하지 않는다.
  자연 Elite, Stage 2, Stage 3의 승리를 같은 identity의 1·2·3번째 격파로 세며 Stage 3 승리는 RON이
  명시한 `fragment_id`의 지급 trigger와 숙달을 함께 성립시킨다.
- 변환된 difficulty는 typed stage를 소유하고 monolith ledger의 stage와 반드시 일치한다. Stage 1
  자연 출현을 기준으로 Stage 2의 초기 스탯 가중치는 150%, Stage 3은 200%다. 두 값은 개발 과정의
  플레이 검증으로 조절 가능한 RON tuning이며 gameplay 구조 정책을 다시 열지 않고 변경할 수 있다. 어떤 스탯과 encounter
  구성에 가중치를 합성할지, 정확한 반올림·웨이브·보상 값은 각 RON이 직접 작성하고 공통 규칙으로
  추론하지 않는다.
- Stage 보상은 Monolith role multiplier와 먼저 합성한 뒤 원래 authored 수량에 한 번만 적용하고
  한 번만 반올림한다. Unity에는 exact multiplier 대신 `enhanced`/`final_enhanced` tier만 투영한다.
- stage 3 승리 시 use가 정확히 1 남으면 source identity를 가진 잔재로 전환한다. 서로 다른 경로에서
  모은 잔재가 네 개가 되면 다음 Floor의 Shop 하나를 보장한다. 상인은 Shop보다 먼저 일회성 VN으로
  oldest four 교환을 제시하며 수락은 run-local 변질된 모노리스를 주고 거절은 같은 run에서 비가역적이다.
- 변질된 모노리스는 현재 runtime에서 consumer가 없는 dormant run-local key다. 숨겨진 보스,
  관리자 연출과 지속형 capstone의 target gameplay는 아래 FinalBoss 정책을 따르며 구현은 후속이다.

Bonus objective:

- Encounter가 명시한 objective만 존재하며 primary abnormality가 없는 Normal encounter에는 없다.
- 승리 결과에서 완료 event log/result stats로 평가하고 objective별 전투당 한 번만 추가 성장
  보상을 지급한다. 각 objective의 RON이 강화 재료와 `GrantExperience`를 포함한 reward effect와
  정확한 수량을 직접 명시하며 여러 effect를 함께 가질 수 있다. 스킬 파편 획득 조건에는 영향을
  주지 않는다.
- Bonus `GrantExperience.amount`는 나눠 갖는 total pool이 아니라 직원별 지급량이다. 배치, 무장과
  현재 전투 가능 여부에 관계없이 결과 commit 시 roster의 모든 살아 있는 직원이 각자 전액을 받고
  `Dead` 직원만 제외한다. 직접 level을 지급하지 않고 기존 XP/level-up 규칙을 따른다.
- `ClearWithin(time_ms)`는 battle start부터 victory까지의 시간을 검사하며 늦게 spawn하는
  boss에는 사용하지 않는다.
- `DecisiveDamage(percent)`는 primary abnormality에게 한 공격 원천이 준 non-DoT 실효 피해가
  최대 HP의 지정 비율 이상인지 정수 비교한다. 여러 tick을 합산하지 않고 마지막 일격일 필요도 없다.
- “환상체 부위”는 별도 스킬 파편이 아니라 bonus 강화 재료를 표현하는 presentation으로 사용할 수
  있다.

### 숙달 Elite 소탕

소탕은 이미 파편 획득 단계까지 직접 전투한 Elite를 같은 run에서 다시 만났을 때 반복 성장의
피로를 줄이는 빠른 전투다.

- 현재 run에서 동일한 Elite 환상체를 이미 세 번 격파한 경우에만 소탕 후보가 된다.
- 최초 세 번의 격파, Standard, NormalBoss와 FinalBoss에는 소탕을 적용하지 않는다.
- 숙달 Elite node에서는 `직접 전투`와 `소탕`을 함께 제시하고 플레이어가 선택한다. 소탕도 일반
  전투 진입과 마찬가지로 출전 가능하며 무기를 장착한 직원이 최소 한 명 있어야 한다.
- 플레이어가 소탕을 선택하면 실제 전투 setup/update simulation을 시작하지 않고 전투와 독립된
  typed 소탕 결과를 만든다.
- 소탕도 해당 node를 해결하는 명시적 선택이며, 결과 확인과 최종 commit 경계를 우회하지 않는다.
- 직접 승리와 같은 authored 기본 보상을 지급한다. Source fragment 보유 count가 0이면 같은 파편 한
  개를 재지급하고 `repeat_reward`는 지급하지 않으며, count가 1 이상이면 RON `repeat_reward`를 지급한다.
  분쇄→재획득 cycle은 의도적으로 허용한다. 기본 전투 XP도 배치·무장·condition과 관계없이 roster의
  살아 있는 모든 직원에게 각자 전액 지급하고 Dead만 제외한다.
- 실제 전투가 없으므로 physical damage, trauma, 전투불능과 사망을 만들지 않고 모든 직원 condition을
  그대로 유지한다. `ClearWithin`/`DecisiveDamage`를 평가하거나 bonus 보상을 자동 지급하지 않는다.
- 일반 승리처럼 활성 소비 아이템의 node 단위 duration을 한 번 차감하고 결과 commit에서 node를
  `Completed`로 소비한다.
- battle record와 전투 통계는 만들지 않는다. 첫 command는 기본 보상, 재획득 파편 또는
  `repeat_reward` 중 하나와 직원별 XP를 담은 동일한 typed 결과를 stage하고, 확인 command가 그 저장된
  계획을 재계획 없이 원자 commit한다. 정확한 DTO와 transient state 경계는 runtime contract가 정한다.

### 장비

장비는 일반 장비와 E.G.O 장비로 나뉘며 무기, 방어구, 악세서리 슬롯을 사용한다.

- 무기는 기본 공격·range·targeting과 block 수치를 제공한다. 각 무기의 필수
  `weapon_profile.block_capacity`가 직원의 기본 저지 수를 단독으로 결정하며, 직원 class/
  `CombatAptitude`나 방어구는 이 값을 변경하지 않는다. 무기가 없으면 0이다.
- 각 무기는 `deployment_affinity`와 usable tile별 profile을 직접 소유한다. `ground_only`는
  `ground_defense_focus`만, `platform_only`는 `platform_rest`만, `any`는 둘 다 명시해야 하며 runtime은
  archetype, employee profile, effective block 또는 default에서 값을 보완하지 않는다.
- First-live 30개는 Staff 16, Bow 1, Gun 1, Sword 10(training 포함), Spear 1, Shield 1이다. Staff는
  PlatformOnly/base block 0/Rest HP 3·resonance 5, Bow/Gun은 GroundOnly/base 0/DefenseFocus +0·10%,
  Sword/Spear는 GroundOnly/base 1/+1·10%, Shield는 GroundOnly/base 3/+1·10%다. 이 authored
  DefenseFocus/Rest 수치는 위 Attack/Defense 타일 효과 계약으로 전투 중 적용된다.
- Fragment의 `block_capacity_min`과 certified combat power는 타일 효과가 더해진 값이 아니라 위
  `weapon_profile.block_capacity`만 사용한다. 방어구는 생존 stat과 utility를, 악세서리는 보조 stat과
  utility를 제공할 수 있다.
- 비무장 직원은 safe-zone roster와 equipment target으로 유지된다. condition상 이미 출전 불가능하면
  그 condition reason이 우선하고, condition상 가능하지만 무기가 없으면 battle command/capability는
  `weapon_required`로 거부한다. 배치, 재배치, 배치 범위 preview와 그 이후 battle gameplay는 fallback
  profile을 만들지 않는다.
- 강화 level은 definition이 아니라 `OwnedEquipment` instance에 저장하며 전투 시작 draft에
  snapshot한다.
- 강화는 장비 가루를 사용한다. 장비 제작은 현재 Maintenance action이 아니다.
- 장비 가루와 skill fragment dust 경제는 분리하며 서로의 강화에 사용할 수 없다.

#### 장비 전투 트리거

장비 runtime instance는 소유 유닛의 `OnBattleStart`, `OnAttack`, `OnHit`, `OnKill`, `OnDeath`,
`OnAllyDeath` 발생을 하나의 typed occurrence 경로로 처리한다.

- 같은 발생에서는 장비 instance UUID 오름차순으로 정의를 모으며 일반 effect를 ability activation보다
  먼저 실행한다. 여러 유닛이 참가하는 전투 시작과 아군 사망 처리는 runtime unit UUID 오름차순이다.
- `OnBattleStart`는 effect와 activation을 모두 실행한다. 따라서 `parasite_seed`의 `AllAllies` Attack
  `+6%`처럼 이미 출판된 시작 effect도 살아 있는 아군 각각에게 정확히 한 번 적용된다.
- `OnAttack`과 `OnHit`는 현재 기본 공격의 확정 hit에만 적용한다. 추가 피해와 피해 보정은 해당 hit의
  명시 입력으로 유지하며 일반 skill, buff tick, area damage가 이 두 장비 trigger를 새로 발생시키지 않는다.
- `OnDeath`는 사망한 장비 소유자의 activation에만 dead-caster 실행을 허용한다. `OnKill`은 killer,
  `OnAllyDeath`는 같은 편의 살아 있는 나머지 유닛에게 발생하며 각 effect는 자기 occurrence의 activation보다
  먼저 실행된다.
- `OnHit + CounterpartUnit + BonusDamage`의 counterpart는 공격자다. 이 조합은 피격자의 수신 피해를
  증폭하지 않고, 확정된 원 hit snapshot에서 authored 추가 피해를 한 번 계산해 effect 소유자 attribution으로
  공격자에게 반격한다. authored effect마다 한 번만 적용하며 이 반격 피해는 `OnAttack`/`OnHit` 장비 trigger를
  재수집하지 않아 상호 반사 loop를 만들지 않는다. 공격자가 없거나 이미 죽었으면 재지정하거나 원 hit를
  증폭하지 않는다.

### 스킬 파편

직원은 기본 고유 active skill 대신 장착한 skill fragment의 대표 active skill을 사용한다.

- 한 환상체는 두 개 이상의 서로 다른 skill fragment를 가질 수 있고 각 fragment는 정확히 하나의
  원본 환상체를 가리킨다. 따라서 schema와 UI는 환상체당 단일 fragment를 전제로 만들지 않는다.
- 기본 active fragment slot은 1개다. 환상체 원본 skill과 직원용 모방 skill은 별도 data다.
- Fragment는 passive 장비가 아니며 평타 강화, 일시 stat, buff와 공명 변화도 active skill 결과로
  표현한다.
- 흑조형 패링 파편은 개별 예외로, 장착한 실제 사본의 직원이 배치되어 있는 동안 공명과 무관한
  피해 경감·확률 반사를 제공할 수 있다. 기존 active 슬롯 하나를 그대로 사용한다. 모든 형태에
  SelfUnit/Instant 패링이 있어야 하며 상시 경감과 유한 패링 경감을 중복 작성하지 않는다.
  철수·전투불능·전투 종료 시 상시 효과가 사라지고 재배치 시 장착 사본에서 다시 파생한다.
  실패 빈틈은 해당 사본의 경감만 해제한다. 반사는 직접 적 타격의 실제 HP 손실을 기준으로
  대상별 첫 양수 segment에서 한 번 판정하며, 반사·받아치기·자기 피해·HP 비용은 입력에서 제외한다.
  패링/무적에 막힌 타격은 반사하지 않는다. 공격자 소실 또는 반사 소유자의 전투불능 시 반사를
  실행하지 않으며, 새 대상을 선택하지 않는다. 수치는 콘텐츠별 검증 데이터이며 fixture 수치를
  공식 밸런스로 간주하지 않는다.
- 위 두 항목은 정식 fragment 정책이다. 현재 abnormality-derived 28개와 concept starter 1개의
  직접 `BasicAttackModifier`는 dedicated active skill 제작 전의 temporary placeholder이며 정식
  정책이 구현 완료됐다는 근거가 아니다. 이 correction에서는 해당 live data를 변환하지 않는다.
- 같은 정의도 개별 UUID 사본으로 소유한다. 직원 active 슬롯은 사본 하나를 장착하며, 이전해도 사본의 해금·선택 상태는 유지한다. 같은 사본의 이중 장착과 occupied 슬롯으로의 이전은 거부한다.
- 일반 파편 강화는 없다. Maintenance에서 exact copy의 authored recipe(통화와 환상체 부위 재료)로 형태를 해금하고 별도 명령으로 선택한다. 중복 해금과 동일 선택은 비용을 소비하지 않는다. 선택한 형태는 공통 요구와 형태별 무기 요구를 모두 만족해야 한다.
- 명시 분쇄 recipe가 있는 사본만 분쇄 가능하다. 장착 중이면 확인 후 exact 해제·삭제·산출을 한 번 commit한다. 사망 회수는 소유 사본을 삭제하지 않고 장착만 해제한다.
- 기존 dust 보상 잔액은 보존하지만 새 개화 비용으로 소비하지 않는다. recipe 없는 기존 정의와 active skill 미구현 placeholder는 해당 기능 unavailable로 표시한다.
- 장착 조건은 직원 직업명이 아니라 장착 무기의 combat profile을 사용한다. Melee/ranged 또는
  특정 archetype 제한을 허용하되 모든 fragment를 모든 무기에 강제 호환하지 않는다.
- 최종 장착 가능성은 core validation이 소유한다. 콘텐츠 설계와 live 목록은 `docs/skills/`를
  함께 확인한다.

## 비전투 노드

이 장은 SavePoint, Maintenance, HeadquartersContact에서 플레이어가 얻는 결과를 소유한다.
세션 identity, capability, atomic commit과 snapshot projection은 runtime/wire 경계의 요약이며 세부
구현은 [core_runtime_contract.md](../core_runtime_contract.md)와 외부 Unity canonical을 따른다.

### SavePoint 노드

- 완료 시 run당 단일 checkpoint를 덮어쓰고 Safezone으로 복귀한다.
- Load는 최대 3회이며 Config에서 core가 `LoadRunCheckpoint` capability를 enabled로 투영할 때만
  실행한다. Checkpoint metadata 자체는 실행 가능성을 뜻하지 않는다.
- 생존 직원 trauma를 RunPolicy `save_point_trauma_recovery`에 따라 줄인다. Live 값은
  `CurrentPercent { percent: 15 }`이며 현재 trauma의 15%를 올림한다. Critical과 Breakdown도
  생존 직원으로 포함하고 Dead는 제외한다. Physical damage는 바꾸지 않는다.
- Recovery, node completion과 checkpoint capture는 승인된 최종 상태를 기준으로
  원자적으로 한 번만 적용한다. Host의 durable active-run publish까지 실패하면 이 transition을
  성공으로 보고하지 않는다. SavePoint는 psychology/trust/memory/event를 생산하지 않는다.
- 마지막 완료 SavePoint는 versioned local-install active-run current file과 rolling backup의
  continue source다. 이어하기는 load 횟수를 소비하지 않으며 transport player identity와 영구
  조우 도감은 payload 밖에서 주입·보존한다.
- SavePoint는 active battle, battle clock, 배치, Wave/summon state와 attempt-local 도감 후보를 저장하지
  않는다. Process restart 뒤 이어하기는 마지막 완료 SavePoint 상태에서 시작하며 중단된 전투를 resync하지
  않는다.
- 현재 V8 payload의 seed, map/progression, roster, inventory/equipment ownership, skill fragment와 encounter
  progression graph는
  이어하기나 명시적 load가 runtime을 바꾸기 전에 엄격히 검증한다. 불일치 문서는 state, load 횟수와
  active-run export를 바꾸지 않고 거부하며 repair, rebinding 또는 암묵적 구 schema migration을 수행하지 않는다. V7는 별도 명시 변환 API로 원본을 검증한 새 V8 문서를 만들 수 있다.
  V2/V3와 unknown/missing/extra/research/pending shape는 strict reject하며, 명시적 load는 payload 내부 seed
  일치에 더해 현재 live run seed와도 일치해야 한다.
- `loads_used`는 checkpoint payload와 분리된 run-scoped monotonic metadata다. 성공한 명시적 load만
  정확히 1 증가시키고 SavePoint 덮어쓰기, checkpoint 복원, process restart와 backup 복구로
  감소시키지 않는다.

### Maintenance 노드

Maintenance는 분쇄·강화·개화를 반복 실행한 뒤 완료하는 독립 Safe Node다.

- 파편 개화는 authored 통화·재료 recipe를 사용하고 장비는 equipment dust 경제를 사용한다.
- Bloom은 현재 skill fragment 전용이며 장비에서는 disabled다.
- 장착 중인 대상도 확인 후 자동 해제해 분쇄할 수 있고 Maintenance 안에서 loadout 교체도 가능하다.
- Core가 대상별 가능 여부, disabled reason, 비용/획득/before-after preview와 확인·자동 해제 여부를
  계산한다.
- 파편은 `AwakenSkillFragment`/`SelectSkillFragmentForm`/`DismantleSkillFragment`, 장비는 `DismantleEquipment`/`EnhanceEquipment` capability를 사용한다. Maintenance 밖에서는 관리 action을 생략한다. 형태별 DTO의 can_unlock/can_select와 recipe/불가 이유가 exact copy의 실행 조건을 나타낸다. 일반 파편 강화 action은 제거됐다.
- Enabled capability는 모든 대상이 가능하다는 뜻이 아니다. 선택한 대상은 실행 시 Core가 최신
  보유·장착·recipe·재료 상태를 다시 검증하며 실패한 command는 부분 mutation을 남기지 않는다.
- 장비 강화 recipe에서 같은 material ID가 반복되면 Core는 첫 등장 순서를 유지한 checked 합계 하나로
  검증·소비·diff를 만든다. 합계 부족이나 overflow는 재료와 장비 level을 바꾸기 전에 거부하고, 정확한
  합계는 논리 material stack을 한 번 소비해 강화 level을 정확히 한 번 올린다.
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
- 본사 연락은 스킬 파편 연구도, pending 지급과 파편 수령 경로가 아니다.
- 본사 연락은 추후 정보 확인, 채용, 보급, 특수 지시 수령 같은 별도 선택지로 확장할 수 있다.

## 결과와 실패

이 장은 node 소비, 전투 결과 확정, 후퇴, 직원 상태와 run failure가 플레이어에게 갖는 의미를
소유한다. Runtime instance ledger, rollback snapshot, capability preflight와 commit 순서는
[core_runtime_contract.md](../core_runtime_contract.md)가 소유한다.

### 노드 결과 처리

CombatResult는 성공/실패, 직원 condition 변화, 통계, 자동·bonus 성장 보상, 파편 지급 trigger와
숙달 변화를 표시한 뒤 플레이어 확인으로 commit한다.

- Encounter의 named terminal outcome 하나가 stable outcome ID와 Victory/Defeat verdict를 commit한다. System
  time cap/implicit Draw는 없고 시간 종료가 필요하면 encounter가 `TimeReached` outcome을 작성한다. 같은
  boundary에서 Defeat와 Victory가 함께 성립하면 Defeat가 우선한다.
- `CompleteCombatResult`는 condition, consumable duration, reward/XP, 파편/격파 ledger, node
  진행과 session 정리를 하나의 원자적 transition으로 적용한다. 실패하면 user-visible state를
  부분 commit하지 않는다.
- Attempt 중 condition은 working state에 누적하고 결과 확인 때 한 번 roster에 반영한다. 여러
  runtime instance의 통계는 직원별 한 행으로 합치며 마지막 instance가 최종 HP/생존을 대표한다.
- 결과 완료 capability와 preflight는 Core가 소유하고 Unity는 context에서 추론하지 않는다.
- 패배도 살아남은 직원에게 생존 XP를 지급하지만 성공 보상, bonus objective 보상, 파편 지급과
  승리 격파 횟수는 지급·증가시키지 않는다.

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
- 일반 사망 시 roster record는 남기되 장비와 exact active fragment 사본를 원자적으로 회수하고
  consumable modifier를 제거한다. Death-prevent가 발동하면 회수하지 않는다.

Breakdown은 안정 상태가 아니라 시한부다. Trauma 100은 그 자체로 사망이 아니지만, 방치하면 사망으로
확정된다.

- Trauma가 100에 도달한 시점부터 RunPolicy가 소유한 `breakdown_collapse_node_limit`만큼의 node
  완료를 센다. 그 안에 trauma를 99 이하로 낮추는 조치가 없으면 해당 직원은 사망한다.
- Counter는 node category와 무관하게 완료한 node마다 정확히 1 증가한다. 전투 attempt 소진, retreat와
  같은 미완료 결과는 증가시키지 않는다.
- Trauma가 99 이하로 내려가면 counter를 즉시 폐기한다. 이후 다시 100에 도달하면 새 counter를 처음부터
  시작하며 이전 진행도를 이월하지 않는다.
- Breakdown 사망은 일반 사망과 같은 authority를 사용한다. Roster record를 남기고 장비와 shared active
  fragment를 원자적으로 회수하며 consumable modifier를 제거한다. 전투 밖 node transition이므로
  battle death-prevent의 대상이 아니다.
- 남은 node 수는 snapshot으로 투영한다. Unity는 node 진행이나 trauma 수치에서 재계산하지 않는다.
- Counter는 run/checkpoint 상태다. Checkpoint load는 저장 시점의 남은 node 수를 그대로 복원한다.
- `breakdown_collapse_node_limit`의 live 값은 아직 authoring하지 않았다. 현재 정책 의도값은 `2`다.

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

## 후속 콘텐츠·구현 authoring

이 정책 범위의 gameplay 결정은 완료됐다. 다음 값은 공통 fallback으로 추론하지 않고 개별 RON에서
작성한다.

- encounter가 지급할 정확한 `fragment_id`와 보유 시 `repeat_reward`
- `ClearWithin`/`DecisiveDamage`의 강화 재료·직원 XP effect와 수량
- BossOmen chain의 minimum Floor, optional required fragment와 개화 필요 여부, step/encounter 및
  encounter-driven 단일 fragment의 최초 source와 단계별 형태
- Standard scripted fragment의 identity와 지급 시점

Schema 이름, DTO shape, validation, migration과 live 수치는 구현 계획에서 정하되 이 룰북의 확정된
gameplay 의미를 바꾸지 않는다.

## 고급 정책과 레거시

### FinalBoss pool과 BossOmen Chain

FinalBoss는 일반 Boss node를 직접 pool에서 배정하지 않는다. Endless의 authored BossOmen chain
pool에서 징조 체인을 고르고, 체인을 완료한 뒤 final-boss급 capstone encounter를 강제한다. 한 번에
active chain은 최대 하나다.

FinalBoss pool cycle:

- 개발 bootstrap의 authored 일반 chain membership은 one-step `white_night` 하나다. 이 chain은 generic
  eligibility/cycle/source-failure/capstone과 영구 all-FinalBoss record의 live end-to-end 검증을 위해
  활성화하며 최종 백야 chain 설계로 간주하지 않는다.

- 한 cycle은 모든 authored 일반 FinalBoss chain을 구성원으로 가진다. 각 chain RON은 minimum Floor,
  optional required fragment와 그 fragment의 개화 필요 여부를 직접 명시한다. 모든 FinalBoss에
  “아무 개화 파편 하나”를 요구하는 공통 조건은 두지 않는다.
- Chain identity, step/encounter reference, 선행 조건과 의존 관계는 live data 초기화에서 강하게
  검증한다. 알 수 없는 reference, 순환 의존성과 authored progression에서 달성 불가능한 조건은
  초기화를 거부하며 fallback chain, 조건 무시 또는 자동 해금으로 복구하지 않는다.
- Eligibility는 새 Floor를 생성할 때만 평가한다. 현재 Floor 도중 파편 획득 등으로 조건이 열려도
  이미 확정한 map에 징조를 추가하지 않고 다음 Floor부터 반영한다.
- 조건을 만족하지 못한 chain은 현재 cycle의 미완료 상태로 남되 후보에는 나오지 않는다. 완료된
  chain을 먼저 재사용하지 않으며, 남은 미완료 chain이 모두 잠겨 있으면 조건이 열릴 때까지
  BossOmen이 출현하지 않는다.
- 모든 authored 일반 FinalBoss chain을 한 번씩 완료해야 cycle을 다시 채운다. Cycle reset은
  Endless 완료가 아니며 다음 Floor들에서 같은 chain이 다시 등장할 수 있다.
- 흉탄의 사수 같은 별도 숨겨진 보스는 이 일반 FinalBoss pool cycle의 구성원이 아니다.

Provisional omen과 active chain:

- Eligible chain 하나를 provisional로 고르고 RON step의 Event 또는 Combat source를 현재 Floor의
  같은 category node 위에 overlay한다. 적합한 node가 없으면 다른 category로 fallback하지 않고
  다음 Floor로 미룬다.
- 첫 omen source를 방문하지 않고 Gate로 이동하면 chain을 확정하거나 현재 cycle에서 소진하지 않는다.
  같은 chain은 현재 제안 순서의 뒤로 보내 다른 eligible chain을 한 번씩 먼저 제안한 뒤 다시 후보가
  될 수 있다.
- 첫 omen의 Event source가 시작한 전투에서 패배하거나 Combat source의 모든 node attempt를 소진하면
  chain 활성화에 실패한다. 일반 Event 완료 규칙으로 omen step을 소비하지 않고, chain을 현재
  cycle에서 제거하지 않은 채 제안 순서의 뒤로 보낸다. 이후 Floor에서 다시 첫 omen 후보로 선택될
  수 있다.
- 첫 source를 성공적으로 해결하면 active chain으로 확정한다. 그 뒤 보스 승리 또는 Run 실패 전까지
  다른 FinalBoss chain을 시작하거나 현재 chain을 pool로 되돌릴 수 없다.
- Active chain 후속 step의 Event source가 시작한 전투에서 패배하거나 Combat source의 모든 node
  attempt를 소진해도 Run을 끝내거나 step을 진행하지 않는다. Active 상태와 같은 step을 유지하고
  다음 Floor에 다시 제안한다. Event source 전투는 해당 Event node 안에서 재시도하지 않는다.
- 모든 step을 완료하면 현재 탐사 map을 authored `boss_omen_capstone`으로 원자 교체한다. Capstone은
  Boss room과 미리 배정된 concealed Exit Gate room, `Boss -> Exit Gate` ForwardOnly connection을
  사용한다. Boss가 유일한 진행 node이고 이전 탐사 room으로 돌아갈 수 없으며 진입 전 Safezone
  loadout은 허용한다.

FinalBoss resolution:

- 실제 FinalBoss는 소탕할 수 없다. 패배 결과를 commit하면 남은 일반 node attempt 없이 즉시 Run
  실패 후보가 된다. 결과 확정 전 battle Retreat는 허용하며 유효한 checkpoint가 있으면 공통 실패 전
  load/확정 선택을 제공한다.
- 승리는 RON의 파편 또는 `repeat_reward`, authored 보상, chain 종료와 현재 pool cycle 완료를 함께
  commit한다. Capstone에 미리 배정된 Exit Gate와 connection을 공개하며 새 node/edge를 만들지 않는다.
- Boss 승리 자체는 Floor를 넘기거나 Endless를 완료하지 않는다. 공개된 Gate를 확인할 때 기존
  Gate transition과 trauma 회복을 적용하고 다음 Floor를 계속 생성한다.
- “미방문 FinalBoss를 pool로 반환”하는 규칙은 capstone Boss node가 아니라 provisional 첫 omen
  source에만 해당한다. Active chain 뒤의 capstone Boss는 건너뛸 수 없다.
- Active/completed chain, pool cycle, 파편과 보상은 checkpoint state다. 승리 뒤 과거 checkpoint를
  load하면 함께 rollback해 같은 chain을 다시 진행할 수 있다. 전역 도감, 영구 개인 기록과
  `loads_used`는 되돌리지 않는다.

숨겨진 보스:

- 변질된 모노리스 등 별도 RON 조건으로 발현하는 흉탄의 사수형 숨겨진 보스는 현재 timeline에서
  run당 한 번만 발현한다. 승리 completion ledger가 남아 있으면 다시 조건을 만족해도 재등장하지 않는다.
- 숨겨진 보스 승리도 `RunComplete`를 만들지 않는다. 보상·파편을 commit하고 Exit Gate를 공개해
  Endless의 다음 Floor로 계속 진행한다. 따라서 Exit Gate 없는 구 single-terminal hidden capstone과
  특별 완료 정책은 폐기 대상이다.
- 패배는 일반 FinalBoss와 같은 공통 Run 실패 후보·Retreat·checkpoint 규칙을 따른다.
- 승리 전 checkpoint 또는 승리 뒤의 과거 checkpoint를 load하면 발현·승리·보상·조건 소비가 함께
  rollback되므로 다시 발현해 도전할 수 있다. 이는 새 보상을 중복 보유하는 두 번째 발현이 아니라
  checkpoint가 복원한 timeline이며 `loads_used`는 감소하지 않는다.

### 레거시 제거 원칙

공통 compatibility, fallback, 구 테스트와 사용자 확인 원칙은
[engineering_workflow.md](../engineering_workflow.md#레거시와-문서-정리)가 소유한다. 이 룰북은
gameplay domain에서 다시 만들지 않을 계약만 열거한다.

- 구형 `timeline_delta`/`battle_delta` 전송을 다시 만들지 않는다.
- 제거된 전투 모드와 RON wrapper는 필요하다는 이유만으로 부활시키지 않는다.
- 제거된 `Support`, `Rest`, `ChooseSupport`, support choice mode와 관련 Psychology 사건은
  compatibility alias나 fallback으로 부활시키지 않는다.


### R06 구현 수치 입력 보충 (2026-09-29)

스킬의 Damage/Heal은 기존 정수 literal 또는 검증된 typed expression으로 계산한다. Snapshot은 방출 시, Live는 효과 소비 시 입력을 읽는다. SpellPower 기본값은0이며 무기 수치·성장·Permanent 효과·실제 장비 사본 강화와 함께 단일 stat pipeline에서 합성한다. AttackSpeedPoints는 basic attack 간격에만 적용하며 다른 스킬/투사체/영역 timer를 바꾸지 않는다. 이 입력 fingerprint 추가는 기존 Certified 전투력의 공식 weight 변경을 뜻하지 않는다. 실제 amount preview/explain은 실행 기록을 조회하며 게임 상태나 자원을 변경하지 않는다. R07 사본/형태 선택은 같은 stat·amount pipeline에 exact 출처를 전달한다.
