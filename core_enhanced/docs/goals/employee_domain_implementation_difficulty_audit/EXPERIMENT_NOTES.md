# Employee Domain Implementation Difficulty Audit Notes

## Initial State

- Goal 경로는 신규이며 이름 충돌이 없었다.
- Core 작업 트리와 상위 simulator 저장소에는 매우 많은 기존 수정·삭제·untracked 파일이 있다.
- 직원 관련 파일을 포함한 여러 production 파일과 `docs/game_rulebook.md`, `docs/README.md`도 이미 수정 상태다.
- `/mnt/f/unity projects/ark` 역시 매우 큰 dirty worktree다.
- `game_resources`와 `game_server`는 Core와 같은 상위 git worktree에 속하며 별도 `.git` 저장소가 아니다.
- 이번 Goal은 새 Goal 문서와 현재 `docs/decisions/employee_domain_policy_2026-07-17.md`로 승격된 당시 정책 논의 문서만 수정한다. 다른 기존 변경은 사용자 소유로 보존한다.

## Path Collision

- `docs/goals/employee_domain_implementation_difficulty_audit/`는 생성 전 존재하지 않았다.
- 필수 문서 세 개 모두 신규 생성했다.

## Structural Judgments

- `StarterEmployeeCandidate`가 definition, starter offer DTO와 recruitment offer DTO 역할을 동시에 맡고 있지만 authored identity는 roster에 전달되지 않는다.
- 현재 `EmployeeRoster`는 run UUID key의 HashMap이며 `add()`는 같은 run UUID만 덮어쓸 뿐 definition 중복을 검증할 수 없다.
- 목표 Queue/NodeSession/Roster 단일 소유권은 새 run-persistent queue state와 session 반환 transaction을 요구한다. 현재 node-seeded shuffle은 deterministic하지만 run-wide finite ownership 모델이 아니다.
- personality와 combat aptitude/base stats는 통합 definition schema의 필수 필드가 되므로 live loader, strict validation, starter projection과 roster snapshot을 함께 바꿔야 한다.
- 직원 공격 속도 percent는 weapon interval을 기준으로 적용해야 하므로 현재 weapon direct overwrite 이후, equipment permanent modifier 이전의 명시 pipeline 단계가 필요하다.
- trust score/band는 목표 정책과 이미 대체로 일치하지만 trait, memory, reaction DTO는 목표와 크게 다르다.
- battle employee condition은 persistent roster를 직접 mutate하지 않고 attempt roster clone에서 누적한 뒤 combat result transaction으로 commit한다. 목표 physical damage도 이 구조를 재사용할 수 있다.
- retreat atomicity는 `BattleEntrySnapshot`이 roster/inventory/skill fragments/UUID/checkpoint까지 복원하므로 새 queue와 employee condition이 기존 captured fields 안에 포함되는지 명시 검증이 필요하다.
- 현재 `LiveBattleDeploymentDto`는 condition-invalid 직원을 projection에서 제외해 typed reason을 전달하지 못한다. 목표 evaluator는 후보 projection과 payload validation 양쪽을 재구성해야 한다.
- death asset recovery는 employee, inventory equipment reverse reference와 skill fragment loadout을 한 transaction에서 갱신해야 한다. 기존 maintenance auto-unequip 경로는 참고할 수 있지만 death source를 소유하지 않는다.
- Core checkpoint는 serde save가 아니라 in-memory clone 구조다. 새 employee fields는 roster clone으로 따라가지만 recruitment queue는 payload/RunState carry-forward에 별도 추가가 필요하다.
- game_server는 employee payload 의미를 소유하지 않고 shared Core DTO를 serialize하는 transport다. 독립 server domain migration보다 contract fixture 갱신이 중심이다.
- Unity `RosterModel`과 여러 composer가 HP/life/availability를 직접 소비하며 unknown life state를 living으로 취급한다. 목표 typed eligibility authority를 위해 fallback과 local inference를 함께 제거해야 한다.

## Source-of-truth Conflicts

- 목표 정책은 하나의 authored catalog를 요구하지만 runtime/live data는 starter/recruitment 두 catalog를 독립 source로 사용한다.
- 목표 정책은 definition ID와 run UUID를 분리하지만 현재 roster는 definition ID를 잃는다.
- 목표 정책은 두 축 authored personality를 요구하지만 현재 단일 trait를 run UUID에서 파생한다.
- 목표 정책은 직원 공격 속도 보정을 보존하지만 현재 weapon profile이 interval을 완전히 덮어쓴다.
- 목표 정책은 trauma 100을 Breakdown으로 두지만 현재 trauma 100은 Run HP 0과 함께 영구 Dead 전이 조건이다.
- 목표 정책은 치명상/Breakdown에서도 치료와 장비 관리를 허용하지만 현재 loadout validator는 combat availability를 요구한다.
- 목표 정책은 purpose-specific typed eligibility를 요구하지만 Event, consumable, loadout, battle deployment가 서로 다른 boolean 조건과 범용 error를 사용한다.
- 목표 정책은 physical damage Treatment를 요구하지만 현재 live effect/schema와 실제 recovery capability에는 구현된 Treatment가 없다.

## External Dependencies

- `/mnt/f/unity projects/ark`의 실제 reader/presenter/test에 접근해 조사했다. 다만 worktree가 매우 dirty하여 배포 버전 일치는 검증하지 않았다.
- `game_server`는 Core와 같은 simulator git worktree 안에서 접근 가능했다.
- 별도의 durable save service/database consumer는 현재 검색 범위에서 확인하지 못했다.

## Out-of-scope Follow-ups

후속 구현 Goal은 이번 조사 결과에 이름과 경계 후보만 기록하며 자동 생성하지 않는다.

## Skipped Verification

- Rust, game_server와 Unity test runner는 실행하지 않았다. 이번 Goal은 production behavior/schema를 변경하지 않는 read-only audit와 Markdown 갱신이며, 관련 테스트의 위치와 보호 계약만 조사했다.
- Markdown formatter는 실행하지 않았다. repository 표준 formatter가 확인되지 않았고 관련 없는 dirty 문서를 건드리지 않기 위해 대상 파일의 heading/whitespace/path를 직접 검증했다.

## Remaining Risks

- 대규모 기존 dirty 상태 때문에 관련 파일의 현재 내용이 clean baseline과 다를 수 있다. 이번 평가는 현재 작업 트리의 실제 상태를 기준으로 한다.
- Unity 소비자는 접근 가능하지만 dirty 상태이므로 배포 버전과 일치한다고 단정하지 않는다.
- 별도의 durable save/database consumer는 현재 저장소 검색에서 확인되지 않았다. 후속 Goal에서 외부 persistence가 새로 발견되면 난이도와 migration 범위를 재평가해야 한다.
- 대상 직원 문서 앞부분은 2026-07-17 초안이고 이번 승인 결정을 모두 본문에 canonical하게 재작성한 상태가 아니다. 이번 절은 최신 승인 목표를 명시해 난이도를 평가했으며 본문 정합화는 별도 policy canonicalization 후속이다.
