# Experiment Notes — Game Rulebook Full Runtime Alignment Audit

## 초기화 기록

- Created: 2026-08-20
- Goal path: `core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit`
- 경로 충돌: 없음
- Initial Status: `active`
- Goal mode: `audit_only`
- Implementation authorized: `No`
- Audit finding ID: `GRFRA-001`~`GRFRA-010`

## 최종 감사 결과

- Final Status: `complete`
- Coverage: `GR-001`~`GR-050` 전부 disposition, pending 0
- 불일치: 10건 — `canonical_ambiguity` 3건, `runtime_implementation_gap` 7건
- 검증: 성공한 9개 job의 1,213 tests 전부 pass; 별도 최초 incremental compile 1건은 rustc ICE였고
  non-incremental focused/broad 재실행은 pass
- 변경 경계: 이 세 Goal 문서만 신규 작성; production code/data/test와 canonical은 수정하지 않음
- Source basis: revision `f318414fb0156a67e04c5479000c2a2683fc0c09` 위 current dirty working tree

## 구조적 판단

- Deliverable은 룰북 전체의 read-only runtime alignment audit 하나다.
- `PLAN.md`는 현재 감사 계약, `EXPERIMENTS.md`는 실제 실행 증거, 이 문서는 판단·불일치 보고서다.
- 룰북은 gameplay canonical이지만 자신이 인접 canonical에 위임한 세부 알고리즘과 wire shape까지 소유하지 않는다.
- 기존 감사와 Goal history는 조사 단서이며 current runtime/data의 증거가 아니다.
- Finding correction은 이 Goal에서 구현하지 않는다.

## Source-of-Truth 충돌

총 10건을 확인했다. `GRFRA-001`~`003`은 current runtime의 결정적·관찰 가능한 tie-break 또는 계산 척도가
gameplay canonical에 빠져 독립 consumer가 같은 동작을 재구성할 수 없는 `canonical_ambiguity`다.
`GRFRA-004`~`010`은 룰북의 current 규칙에 필요한 producer, DTO 경계, schema/live authoring, persistent state,
recovery 평가, save validation 또는 source-specific transition이 없는 `runtime_implementation_gap`이다.

## 정책 질문

- `GRFRA-001`~`003`: current 알고리즘을 룰북에 고정할지, 인접 owner에 위임할지
- `GRFRA-004`: status 차단 feedback을 구현할지, `immune`을 damage-only로 정의할지
- `GRFRA-005`: active DTO field를 rename/migrate할지, 룰북의 폐기 범위를 legacy semantics로 좁힐지
- `GRFRA-006`: Bonus objective를 current runtime에 구현할지, future authoring으로 재분류할지
- `GRFRA-007`: Breakdown counter/live 값 `2`를 구현할지, future implementation으로 재분류할지
- `GRFRA-008`: HeadquartersContact candidate availability의 exact recovery eligibility
- `GRFRA-009`: invalid V4 scalar의 strict rejection과 기존 save compatibility/migration
- `GRFRA-010`: BossOmen Event source와 Combat source의 exact retry/failure 경계

## 외부 시스템과의 직접 의존성

- `game_resources`: production live RON과 loader reachability 확인 대상
- `game_server`: 룰북 claim에 직접 필요한 command/projection/persistence 경계만 확인
- Unity: 사용자 승인에 따른 명시적 비범위

## 범위 밖 후속 후보

- `GRFRA-001`~`003` canonical clarification
- `GRFRA-004`~`010` runtime/DTO/schema/live-data/save correction

어느 후보도 이 audit Goal에서 구현하거나 별도 Goal로 생성하지 않았다.

## 적용하지 않은 대안과 이유

- 기존 `game_rulebook_runtime_alignment_audit` 재개: 사용자가 fresh 새 경로를 승인했고 과거 actual과 current
  actual을 섞지 않기 위해 사용하지 않는다.
- 과거 audit matrix를 current evidence로 복사: source currentness와 runtime reachability를 증명하지 못하므로
  사용하지 않는다. 실제 기존 matrix는 SHA `c11a0940…`의 1,210줄 룰북과 당시 미구현 정책을 대상으로 하며,
  current SHA `8aa954cc…`의 1,275줄 룰북과 basis가 다르다. Module/test 이름만 조사 locator로 사용할 수 있다.
- 감사 중 production correction: audit-only 권한을 넘으므로 수행하지 않는다.

## Dirty Worktree 주의사항

- Audit Source Basis `GRFRA-ASB-001`: repository revision
  `f318414fb0156a67e04c5479000c2a2683fc0c09` 위의 current dirty working tree.
- 룰북과 인접 canonical, Core battle/world/data, production `game_resources` 및 `game_server`에 광범위한 기존
  변경이 있다.
- Canonical SHA-256: rulebook `8aa954cc…`, core runtime `73bc18d4…`, data loading `58c636ce…`, skill target
  `a35a2024…`.
- 새 Goal 경로만 이번 감사가 추가했다. 기존 변경을 restore, reset, stash, format 또는 정리하지 않는다.
- 조사 중 source identity가 달라지면 영향받은 row를 즉시 stale로 보고 재확인한다.

## 접근하거나 검증하지 못한 소비자

- Unity reader/store/command/presentation: 명시적 비범위
- 배포된 외부 consumer: production rollout 비범위

추가 항목은 발견 즉시 기록한다.

## 생략한 검증과 근거

- Unity 및 배포 consumer 검증: 명시적 비범위
- migration test: 어떠한 migration도 수행하지 않음
- 전체 workspace/repository suite: Core/server library와 production RON integration은 실행했지만, production
  mutation이 없는 audit이고 unrelated package는 claim evidence를 늘리지 않아 확장하지 않음
- finding별 신규 probe/test 작성: production test 변경은 명시적 비범위. Existing test와 source trace만 사용함

## Coverage 및 Finding 기록

룰북의 44개 `###`, 4개 `####` subsection과 직접 disposition이 필요한 상위 경계를 50개 group으로 고정했다.
50개 모두 current verdict가 있으며 pending row는 없다.

| Group | Rulebook section / locator | Initial disposition | Current verdict |
| --- | --- | --- | --- |
| `GR-001` | 문서 책임, 5-33 | boundary/meta | `aligned` (owner boundary; E-017, Unity excluded) |
| `GR-002` | 게임 정체성, 39-51 | current normative | `aligned` (static, E-005) |
| `GR-003` | 게임 모드, 52-99 | current normative + explicit bootstrap | `aligned` (static, E-005) |
| `GR-004` | 핵심 플레이 루프, 100-121 | current normative | `aligned` (static, E-005) |
| `GR-005` | 시작 직원 선발, 122-141 | current normative | `aligned` (static, E-005) |
| `GR-006` | Floor 맵과 노드 선택, 148-205 | current normative + future candidate | `aligned` (static, E-005; future candidates excluded) |
| `GR-007` | Certified 전투력과 Floor 난이도, 206-229 | current normative | `aligned` (static, E-005) |
| `GR-008` | Safe Node와 Safezone, 230-243 | current normative | `aligned` (static, E-005) |
| `GR-009` | Event Node, 244-278 | current normative | `aligned` (static, E-005) |
| `GR-010` | 일반 인벤토리, 279-290 | current normative | `aligned` (static, E-005) |
| `GR-011` | 소비 아이템, 291-303 | current normative | `aligned` (static, E-005) |
| `GR-012` | 상점, 304-320 | current normative | `aligned` (static, E-005/E-013; live path confirmed) |
| `GR-013` | 노드 진입 전 브리핑, 321-337 | current normative/presentation boundary | `aligned` (Core static, E-005; Unity excluded) |
| `GR-014` | 전투 모드, 345-361 | current normative + legacy absence | `aligned` (static, E-008) |
| `GR-015` | 전술 포인트, 362-382 | current normative | `aligned` (static, E-008) |
| `GR-016` | Hard CC와 Silence, 383-404 | current normative | `aligned` (static, E-008) |
| `GR-017` | 철수와 재배치, 405-432 | current normative | `aligned` (static, E-008) |
| `GR-018` | Attack/Defense 태세와 타일 효과, 433-459 | current normative | `aligned` (static, E-008) |
| `GR-019` | 전술 이동과 아군 점유, 460-502 | current normative | `aligned` (static, E-008) |
| `GR-020` | 수동 지정 스킬, 503-532 | current normative + explicit unavailable content | `aligned` (static, E-008) |
| `GR-021` | 현장 작업, 533-560 | current normative + explicit unavailable expansion | `aligned` (static, E-008) |
| `GR-022` | Pause와 직렬 전투 명령, 561-582 | current normative | `aligned` (static, E-008) |
| `GR-023` | 전투 시작과 Unity 경계, 583-596 | Core current + Unity excluded | `aligned` (Core static, E-008; Unity excluded) |
| `GR-024` | 전장과 배치, 597-613 | current normative + content exclusion | `aligned` (static, E-008) |
| `GR-025` | 적 구성과 웨이브, 614-643 | current normative | `mismatch` (`GRFRA-001`, `GRFRA-002`; E-006) |
| `GR-026` | Live Boss Runtime Reference, 644-667 | current bootstrap/reference | `mismatch` (`GRFRA-003`; E-006) |
| `GR-027` | 기본 공격과 스킬 타겟팅, 668-683 | current normative/delegated detail | `aligned` (static, E-008) |
| `GR-028` | 피해 타입과 피드백, 684-699 | current normative + presentation boundary | `mismatch` (`GRFRA-004`; E-007) |
| `GR-029` | Runtime 스탯·HP·사망 전이, 700-718 | current normative | `aligned` (static, E-008) |
| `GR-030` | 보상과 성장, 726-749 | current normative + explicit unavailable producer | `aligned` (static, E-011) |
| `GR-031` | 연구도 폐기와 스킬 파편 획득, 750-787 | current normative + legacy absence | `mismatch` (`GRFRA-005`; E-009) |
| `GR-032` | 영구 도감, 튜토리얼 해금과 개인 기록, 788-819 | current normative | `aligned` (static, E-011) |
| `GR-033` | Endless 환상체 격파와 반복 조우, 820-832 | current normative | `aligned` (static, E-011) |
| `GR-034` | Elite assignment pool, 833-857 | current normative | `aligned` (static, E-011) |
| `GR-035` | NormalBoss assignment pool, 858-866 | current normative | `aligned` (static, E-011) |
| `GR-036` | Elite 모노리스와 강화 재현, 867-914 | current normative + dormant consumer | `mismatch` (`GRFRA-006`; E-010) |
| `GR-037` | 숙달 Elite 소탕, 915-938 | current normative | `aligned` (static, E-011) |
| `GR-038` | 장비, 939-964 | current normative | `aligned` (static, E-011) |
| `GR-039` | 장비 전투 트리거, 965-984 | current normative | `aligned` (static, E-011) |
| `GR-040` | 스킬 파편, 985-1005 | current normative + placeholder disclosure | `aligned` (static, E-011) |
| `GR-041` | SavePoint 노드, 1012-1038 | current normative | `mismatch` (`GRFRA-009`; 나머지 static aligned, E-013/E-015) |
| `GR-042` | Maintenance 노드, 1039-1066 | current normative + limited recipe coverage | `aligned` (static, E-013) |
| `GR-043` | 본사 연락 노드, 1067-1074 | current normative + future extension | `aligned` (static, E-013) |
| `GR-044` | 노드 결과 처리, 1081-1105 | current normative | `mismatch` (`GRFRA-006`; 나머지 static aligned, E-014) |
| `GR-045` | 후퇴, 1106-1114 | current normative | `aligned` (static, E-014) |
| `GR-046` | 직원 condition·전투불능·런 실패, 1115-1159 | current normative + explicit unauthored value | `mismatch` (`GRFRA-007`~`GRFRA-009`; E-012/E-014/E-015) |
| `GR-047` | 직원 Psychology, 1160-1173 | current state + explicit unavailable behavior | `mismatch` (`GRFRA-009`; 나머지 static aligned, E-015) |
| `GR-048` | 후속 콘텐츠·구현 authoring, 1174-1187 | explicit future authoring boundary | `aligned` (future boundary, E-017; Bonus schema gap is `GRFRA-006`) |
| `GR-049` | FinalBoss pool과 BossOmen Chain, 1190-1265 | current normative + hidden-boss future seam | `mismatch` (`GRFRA-010`; 나머지 static aligned, E-016/E-017) |
| `GR-050` | 레거시 제거 원칙, 1266-1275 | current legacy absence | `aligned` (static absence/rejection, E-017) |

## 불일치 보고서

### GRFRA-001 — 같은 timestamp bucket의 Wave trigger tie-break가 룰북에 없다

- Coverage: `GR-025`
- 유형: `canonical_ambiguity`
- 룰북 근거: `game_rulebook.md:630-638`은 stable trigger ID와 priority를 요구하고 boundary/승리 규칙을
  설명하지만, 같은 timestamp bucket에서 priority가 같은 ready trigger들의 처리 순서를 정하지 않는다.
- 구현 근거: `core_enhanced/src/game/battle/core/wave_runtime.rs:155-173`은 ready trigger를 priority 내림차순으로,
  동률이면 trigger ID 오름차순으로 정렬한 뒤 하나씩 settle한다.
- 실제 동작: 같은 boundary에 동시에 준비된 Wave는 높은 priority가 먼저이고, priority가 같으면 stable trigger ID의
  사전식 오름차순이 먼저다.
- 기대 동작: gameplay canonical이 관찰 가능한 same-bucket 순서를 명시하거나, 그 규칙을 소유한 인접 canonical에
  명시적으로 위임해야 한다.
- 영향: authoring과 독립 simulator/consumer가 같은 Wave priority만 보고도 spawn 및 후속 trigger의 결정적 순서를
  재현할 수 없다. 순서에 민감한 binding·승리 판정·event projection의 기대가 갈릴 수 있다.
- 확신도: High — current implementation과 current 룰북을 직접 대조함
- 권장 후속 조치: 별도 correction Goal에서 룰북 Wave 계약에 `priority DESC, trigger ID ASC` tie-break를 명시한다.

### GRFRA-002 — `ActorReachedPoint` 반경의 거리 척도가 룰북에 없다

- Coverage: `GR-025`
- 유형: `canonical_ambiguity`
- 룰북 근거: `game_rulebook.md:632-635`는 bound actor의 지점 도달을 closed primary trigger로 열거하지만
  `radius_cells`의 거리 척도를 정의하지 않는다.
- 구현 근거: `core_enhanced/src/game/battle/core/wave_runtime.rs:286-302`는 actor projected tile과 authored point
  사이의 Chebyshev 거리가 `radius_cells` 이하인지 검사한다.
- 실제 동작: 대각선 이동을 포함한 정사각형 범위(Chebyshev metric) 안에 actor가 들어오면 trigger가 준비된다.
- 기대 동작: gameplay canonical이 Chebyshev metric과 projected tile 기준을 명시하거나 소유 canonical에
  위임해야 한다.
- 영향: Manhattan/Euclidean으로 해석한 authoring 도구나 consumer는 대각선 경계에서 다른 Wave 발동 시점과
  결과를 낸다.
- 확신도: High — current implementation과 current 룰북을 직접 대조함
- 권장 후속 조치: 별도 correction Goal에서 actor 기준 좌표와 `radius_cells` metric을 룰북에 고정한다.

### GRFRA-003 — summon batch의 동시 제거 조건 우선순위가 룰북에 없다

- Coverage: `GR-026`
- 유형: `canonical_ambiguity`
- 룰북 근거: `game_rulebook.md:661-664`는 exact batch와 cleanup signal의 non-kill despawn을 설명하지만,
  summoner removal·signal·lifetime이 같은 commit에서 함께 성립할 때 기록할 removal cause와 적용 mode를 정하지
  않는다.
- 구현 근거: `core_enhanced/src/game/battle/core/summon_runtime.rs:473-531`은 아직 제거되지 않은 batch에 대해
  `SummonerRemoved`(Despawn), `Signal`(authored mode), `Lifetime`(authored mode) 순으로 최초 원인을 선택한다.
- 실제 동작: 같은 reconciliation에서 여러 조건이 참이면 summoner removal이 signal보다, signal이 lifetime보다
  우선하며 선택된 원인과 mode로 exact batch가 제거된다.
- 기대 동작: gameplay canonical이 co-observed removal tie-break를 명시하거나 명시적 소유 canonical에 위임해야
  한다.
- 영향: kill/despawn mode와 removal cause가 다른 reward, outcome, event/projection 소비자의 결과를 바꿀 수 있고
  reference behavior를 독립적으로 재현하기 어렵다.
- 확신도: High — current implementation과 current 룰북을 직접 대조함
- 권장 후속 조치: 별도 correction Goal에서 `summoner removal > signal > lifetime` precedence 및 mode 선택을
  명시한다.

### GRFRA-004 — 상태가 완전히 막힌 경우의 `immune` feedback producer가 없다

- Coverage: `GR-028`
- 유형: `runtime_implementation_gap`
- 룰북 근거: `game_rulebook.md:690-698`은 Core가 계산한 `HpChanged`/`final_damage`를 source of truth로 두고
  공식 `immune` feedback tag의 의미를 “피해 또는 상태가 완전히 막힘”으로 규정한다.
- 구현 근거:
  - `core_enhanced/src/game/battle/damage.rs:531-550`은 raw damage가 0보다 크고 final damage가 0일 때만
    `DamageFeedbackTag::Immune`을 생성한다.
  - Core 전체 `DamageFeedbackTag`/`feedback_tags` 검색 결과 projection은 damage `HpChanged` 경로에만 있고,
    status immunity/resistance가 `Immune`을 생성하는 경로는 발견되지 않았다(E-007).
- 실제 동작: 완전히 방지된 damage는 `immune` tag를 낼 수 있지만, 완전히 차단된 status는 공식 `immune` tag를
  생산하지 않는다.
- 기대 동작: current 룰북대로라면 완전히 차단된 status에도 Core가 공식 `immune` feedback을 생산해야 한다.
- 영향: Core event만 소비하는 presentation은 status immunity를 룰북이 약속한 공식 tag로 표시할 수 없다.
- 확신도: High — tag enum/calculator, HP event commit 및 전체 Core producer 검색을 대조함
- 권장 후속 조치: 별도 correction Goal에서 status rejection에 typed feedback producer를 구현할지, 아니면
  룰북의 `immune` 의미를 damage-only로 축소할지 정책을 결정한다.

### GRFRA-005 — 폐기된 `skill_fragment_progress` 이름이 active inventory DTO에 남아 있다

- Coverage: `GR-031`
- 유형: `runtime_implementation_gap`
- 룰북 근거: `game_rulebook.md:750-754`는 `research_progress`, `skill_fragment_progress`,
  `pending_research_deliveries`와 pending 수령 절차를 모두 폐기한다고 명시한다.
- 구현 근거:
  - `core_enhanced/src/game/behavior.rs:975-985`의 current `InventorySnapshotDto`가
    `skill_fragment_progress: Vec<InventorySkillFragmentProgressSnapshotDto>`를 공식 필드로 노출한다.
  - `core_enhanced/src/game/world/snapshot.rs:639-683,726-735`가 강화·개화 진행을 이 배열에 적극적으로
    생산한다.
- 실제 동작: legacy 연구도 의미는 제거됐지만 exact `skill_fragment_progress` wire 이름은 현재 fragment
  upgrade/awakening progression의 active projection으로 재사용된다.
- 기대 동작: 룰북의 exact removal 선언대로 해당 이름이 runtime/wire에서 제거되거나, 새 의미로 재사용한다면
  canonical이 이전 field와의 identity/compatibility 경계를 명시해야 한다.
- 영향: 소비자는 폐기된 legacy field인지 새 fragment progression인지 이름만으로 구분할 수 없고, 룰북을 따른
  client는 active 필드를 무시할 수 있다. DTO migration/compatibility 판단이 필요할 수 있다.
- 확신도: High — current DTO와 producer를 직접 확인함
- 권장 후속 조치: 별도 correction Goal에서 DTO 이름을 명시적으로 migration할지, 룰북의 폐기 문구를 legacy
  semantics에 한정하고 새 field 계약을 canonicalize할지 결정한다.

### GRFRA-006 — 룰북의 Bonus objective 계약이 current PVE runtime/schema에 없다

- Coverage: `GR-036`, `GR-044`
- 유형: `runtime_implementation_gap`
- 룰북 근거: `game_rulebook.md:898-913`은 encounter-authored Bonus objective, 승리 결과 평가, 전투당 한 번의
  추가 성장 보상, `ClearWithin(time_ms)`와 `DecisiveDamage(percent)`의 계산 규칙을 current gameplay로
  규정한다.
- 구현 근거:
  - `core_enhanced/src/game/data/pve_data.rs:255-281`의 current `PveEncounter`에는 bonus objective field가 없다.
  - 같은 파일 `2349-2375`는 이전 `suppression_research.bonus_objectives` 입력을 제거된 authoring으로
    명시적으로 거부한다.
  - Core/live RON 전체 검색에서 `ClearWithin`, `DecisiveDamage` 또는 bonus objective producer/consumer가
    발견되지 않았다(E-010).
- 실제 동작: live encounter는 Bonus objective를 authoring·평가·투영·보상할 수 없다.
- 기대 동작: 룰북대로 encounter가 두 typed objective를 authoring하고 victory result에서 평가해 authored 성장
  reward를 정확히 한 번 commit해야 한다.
- 영향: 플레이어가 룰북상 기대하는 선택적 전투 목표와 강화 재료/직원별 XP 보상이 전부 발생하지 않는다.
- 확신도: High — schema 부재, 제거 입력 rejection 및 전체 producer 검색이 일치함
- 권장 후속 조치: 별도 correction Goal에서 Bonus objective를 다시 current scope로 구현할지, 룰북에서 future
  authoring으로 내릴지 정책 결정이 필요하다.

### GRFRA-007 — Breakdown collapse node counter가 runtime과 live policy에 없다

- Coverage: `GR-046`
- 유형: `runtime_implementation_gap` (룰북에 명시된 unauthored 상태)
- 룰북 근거: `game_rulebook.md:1131-1150`은 trauma 100 도달부터
  `breakdown_collapse_node_limit`만큼 완료 node를 세고, 회복 시 폐기하며, 만료 시 일반 사망 authority를 적용하고,
  남은 node 수를 snapshot/checkpoint에 보존한다고 규정한다. 동시에 live 값은 아직 authoring하지 않았고 의도값은
  `2`라고 명시한다.
- 구현 근거:
  - `core_enhanced/src/game/data/run_policy_data.rs:11-25`의 current `RunPolicyData`에는 collapse limit field가 없다.
  - `game_resources/data/run/policy.ron`에는 `save_point_trauma_recovery` 등은 있으나
    `breakdown_collapse_node_limit`가 없다.
  - Core 직원/run/snapshot/checkpoint 검색에서 Breakdown remaining-node state나 node completion producer가 발견되지
    않았다(E-012). `employee_psychology.rs:29-73`의 `remaining_nodes`는 Psychology memory duration이다.
- 실제 동작: trauma 100은 현재 Breakdown condition 판정에는 쓰이지만, node 완료에 따라 collapse death로 진행되는
  별도 counter·projection·checkpoint state가 없다.
- 기대 동작: 룰북의 current 계약을 실행하려면 live limit, run/checkpoint 상태, node-completion transition,
  recovery reset, death cleanup 및 snapshot projection이 함께 존재해야 한다.
- 영향: Breakdown 직원을 회복하지 않고 node를 계속 완료해도 룰북상 시한부 사망이 발생하지 않으며 UI가 남은
  node 수를 표시할 authoritative 값도 없다.
- 확신도: High — policy schema/live RON과 관련 Core state/producer를 직접 대조함
- 권장 후속 조치: 별도 correction Goal에서 이 계약을 구현하고 live 값 `2`를 authoring할지, 룰북을 future
  implementation으로 재분류할지 결정한다. 구현 시 checkpoint/save shape 변경 가능성을 별도 평가한다.

### GRFRA-008 — NoDeployable recovery evaluator가 HeadquartersContact 채용을 누락한다

- Coverage: `GR-046`
- 유형: `runtime_implementation_gap`
- 룰북 근거: `game_rulebook.md:1147-1153`은 전투 결과 뒤 deployable 직원이 없을 때 보유 Physical Treatment,
  progression에서 즉시 선택 가능한 SavePoint/Gate 및 실제 후보를 제공하는 HeadquartersContact recruitment를 하나의
  recovery-path evaluator로 계산하고, 제한 중 이 행동들과 checkpoint load만 허용한다고 규정한다.
- 구현 근거:
  - `core_enhanced/src/game/world/combat.rs:1044-1119`의 evaluator는 Treatment 총량과 selectable SavePoint/Gate만
    검사한다.
  - 같은 파일 `1157-1171`은 restriction이 존재하면 selectable node를 `allowed_node_ids`로 거르며, 이 목록에는
    HeadquartersContact가 추가되지 않는다.
  - 반면 `world/headquarters.rs:33-85`와 live node data에는 실제 candidate session을 소비하는 recruitment가 존재한다.
- 실제 동작: 배치 가능 직원이 없을 때 HeadquartersContact가 즉시 선택 가능하고 실제 채용 후보가 있어도 그 경로를
  recovery 가능성으로 인정하지 않고 node를 제한 목록에서 제외한다. 다른 두 경로도 없으면 조기 Run failure가 될 수
  있다.
- 기대 동작: evaluator가 selectable HeadquartersContact와 current active/queue candidate availability를 함께
  계산하고, 실제 채용 가능한 node/action을 recovery restriction에 포함해야 한다.
- 영향: 룰북상 회복 가능한 run이 실패 처리되거나 플레이어가 유효한 채용 복구 경로를 선택하지 못할 수 있다.
- 확신도: High — current evaluator와 실제 HeadquartersContact recruitment producer/live node를 직접 대조함
- 권장 후속 조치: 별도 correction Goal에서 recruitment 후보의 “즉시 실제 가능” 판정과 allowed action/node
  projection을 recovery evaluator에 통합한다.

### GRFRA-009 — V4 checkpoint가 roster condition/trust 범위를 검증하지 않는다

- Coverage: `GR-041`, `GR-046`, `GR-047`
- 유형: `runtime_implementation_gap` (save validation gap)
- 룰북 근거:
  - `game_rulebook.md:1024-1033`은 V4 roster와 progression graph를 runtime 변경 전에 엄격히 검증하고 mismatch
    문서를 mutation 없이 거부한다고 규정한다.
  - `1117-1119`는 `physical_damage: 0..6`, `trauma: 0..100`을 persistent condition authority로, `1163-1166`은
    `trust_score: -100..100`을 Psychology authority로 규정한다.
- 구현 근거:
  - `core_enhanced/src/game/employee.rs:208-220`의 `EmployeeCondition`은 derived `Deserialize`라 `new`의 clamp를
    거치지 않는다.
  - `employee_psychology.rs:47-55`도 raw `i16` trust를 deserialize한다.
  - `employee.rs:684-688`의 roster checkpoint validator는 map key와 employee UUID 일치만 검사하며,
    `world/node_flow.rs:1432-1459`에도 scalar range validation이 없다.
- 실제 동작: checksum/shape가 유효한 V4 payload에 condition 또는 trust의 범위 밖 값이 있으면 strict validator가
  이를 거부하지 않고 roster를 restore할 수 있다. 이후 condition은 `>=` 판정으로 Dead/Breakdown 처리되고 trust는
  범위 밖 값 그대로 snapshot에 투영된다.
- 기대 동작: restore 전에 모든 직원의 condition 및 Psychology authoritative scalar 범위를 검증해 payload 전체를
  mutation 없이 거부해야 한다.
- 영향: 손상되거나 비정상적으로 작성된 local active-run이 룰북상 불가능한 직원 상태를 current runtime에 주입할 수
  있고, strict V4 보장과 UI projection이 깨진다.
- 확신도: High — serde shape와 complete checkpoint validation call path를 직접 대조함
- 권장 후속 조치: 별도 correction Goal에서 roster checkpoint validation에 condition/trust range를 추가하고
  malformed V4의 no-mutation rejection test를 보강한다. Save migration 필요 여부는 별도 평가한다.

### GRFRA-010 — BossOmen Event source 전투가 첫 패배 뒤에도 재시도된다

- Coverage: `GR-049`
- 유형: `runtime_implementation_gap`
- 룰북 근거: `game_rulebook.md:1225-1233`은 provisional 첫 Event source가 시작한 전투의 패배는 즉시 chain
  activation failure이고, active 후속 Event source 전투도 패배하면 같은 step을 다음 Floor에 다시 제안하되 해당
  Event node 안에서는 재시도하지 않는다고 규정한다. Combat source만 모든 node attempt 소진을 기다린다.
- 구현 근거:
  - `core_enhanced/src/game/world/combat.rs:635-674`는 Event combat을 포함한 모든 live battle start에서 공통
    `start_abnormality_attempt`를 호출한다.
  - `world/state.rs:1308-1344`는 monolith 외 모든 node에 공통 3 attempts를 부여한다.
  - `world/combat.rs:2926-2942`는 Defeat 후 attempts가 남으면 먼저 NodeConfirm retry로 돌아가며,
    `boss_omen.handle_source_failure`는 exhaustion 이후에만 호출한다. 다른 production call site는 없다(E-016).
- 실제 동작: BossOmen Event source 전투는 첫·두 번째 패배 후 같은 Event node에서 재시도할 수 있고 세 번째 패배
  뒤에야 provisional chain을 defer하거나 active step을 다음 Floor에 유지한다.
- 기대 동작: source kind가 Event이면 첫 Defeat result commit에서 즉시 source failure를 처리하고 Event node 재시도
  경로를 만들지 않아야 한다. Combat source만 공통 attempt exhaustion을 사용해야 한다.
- 영향: live one-step `white_night` Event omen을 포함해 플레이어가 룰북보다 최대 두 번 더 같은 omen 전투를
  시도할 수 있어 chain 활성화 난이도와 Floor 제안 순서가 달라진다.
- 확신도: High — live chain, battle-start attempt producer, defeat branch와 유일한 failure call site를 대조함
- 권장 후속 조치: 별도 correction Goal에서 BossOmen source kind를 defeat resolution에 전달해 Event와 Combat의
  failure boundary를 분기하고 provisional/active 회귀 test를 추가한다.

## 남은 위험

- Unity reader/store/command/presentation과 배포된 외부 consumer는 승인된 비범위라 검증하지 않았다.
- 대규모 dirty tree이므로 verdict는 released/deployed behavior가 아니라 `GRFRA-ASB-001` current actual에
  귀속된다.
- `GRFRA-004`, `005`, `006`, `007`, `009`의 부재/validation gap을 직접 겨냥한 existing test는 없었다. Source,
  schema, producer 및 validator trace의 High-confidence 판정이지만 correction 전에는 신규 regression test가 필요하다.
- E-022에서 incremental rustc 1.91.1 ICE가 한 번 발생했다. `CARGO_INCREMENTAL=0`의 동일 focused suite와 server
  broad suite는 통과했으므로 runtime finding으로 분류하지 않았지만 toolchain risk는 남는다.
- 이 감사는 불일치 10건을 기록했을 뿐 canonical, runtime, live RON, DTO 또는 save를 수정하지 않았다.
