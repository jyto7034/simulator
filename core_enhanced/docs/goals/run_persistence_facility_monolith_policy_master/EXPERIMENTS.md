# Run Persistence, Facility, And Monolith Master Experiments

Status: complete

Implementation complete: true

이 문서는 master-level 조사, phase gate 검증과 최종 broad validation의 실제 실행 기록만 보존한다.
Child 구현 실험은 해당 child `EXPERIMENTS.md`에 기록하고, master에는 gate verdict와 재현 evidence만
요약한다.

## 2026-07-27 — Master Goal 초기화

### 가설

다섯 decision은 서로 다른 계약 경계를 가지지만 순환 seam이 있으므로, 하나의 production Goal로
합치는 대신 ordered phase와 completion review gate를 가진 master campaign이 필요하다.

### 수행한 조사

- `core_enhanced/docs/README.md`, `engineering_workflow.md`,
  `goal_completion_review_guide.md`를 확인했다.
- 기존 `core_unimplemented_policy_implementation_master`,
  `core_followup_policy_implementation_master`, `endless_mode_progression_master`의 master/phase 관례를
  확인했다.
- 다섯 decision record와 관련 Core, live RON, game_server, 외부 Unity runtime/canonical을 읽었다.
- master Goal 경로 충돌과 관련 저장소 dirty 상태를 확인했다.

### 변경한 내용

- 신규 master Goal 문서 3개만 생성했다.
- Runtime code, live data, server, Unity와 canonical 문서는 수정하지 않았다.

### 실행한 정확한 확인 명령

```text
test -e core_enhanced/docs/goals/run_persistence_facility_monolith_policy_master && echo exists || echo available
git status --short -- core_enhanced/docs/goals/run_persistence_facility_monolith_policy_master core_enhanced/docs/decisions core_enhanced/src/game/map/types.rs game_resources/data/map/generation_policy.ron game_server/src/game/player_game_actor/messages.rs
git -C "/mnt/f/unity projects/ark" status --short
```

### 실제 결과

- 생성 전 master 경로는 존재하지 않았다.
- `docs/decisions/`는 현재 untracked이며 사용자 소유 변경이다.
- Core, game resources, game_server와 외부 Unity 모두 대규모 dirty 상태다.
- 확인 명령은 test runner가 아니므로 test 수를 보고하지 않는다.

### 판단

- Child Goal은 phase 활성 시 lazy creation한다.
- P00만 최초 active 후보로 두고 P01-P06은 dependency gate 뒤에 둔다.
- 현재 dirty 변경을 초기화하거나 정리하지 않고 대상 파일별 diff audit 후 병합해야 한다.

## 2026-07-27 — 구현 전 현재 기준선 조사

이 기록은 master 작성 전에 현재 dirty worktree에서 실제로 실행한 read-only baseline을 master 입력
근거로 옮긴 것이다. Goal 문서 생성 뒤 production 변경은 없었다.

### 가설

현재 코드는 새 policy 미구현 상태일 뿐 아니라 `Support/Rest -> SavePoint` vertical migration이
중단돼 있어 production check와 전체 test baseline이 다를 것이다.

### 실행한 명령과 결과

| 명령 | 실제 실행 test 수 | Pass | Fail | Skip | 실제 결과 |
| --- | ---: | ---: | ---: | ---: | --- |
| `cargo check -p game_core` | runner가 test를 실행하지 않음 | 해당 없음 | 해당 없음 | 해당 없음 | 성공 |
| `cargo check -p game_server` | runner가 test를 실행하지 않음 | 해당 없음 | 해당 없음 | 해당 없음 | 성공 |
| `cargo test -p game_core --no-run` | 0 | 0 | 0 | 0 | test binary compile 성공, 실행 없음 |
| `cargo test -p game_core --test ron_loading -- --nocapture` | 23 | 1 | 22 | 0 | 실패 |
| `cargo test -p game_server --no-run` | 0 | 0 | compile 실패 | 0 | 실패 |

### 실패 원인

- `ron_loading`: `game_resources/data/map/generation_policy.ron`의
  `normal_safe_categories`/`pre_boss_safe_categories`가 삭제된 `Support` variant를 포함한다.
- `game_server --no-run`: test module이 삭제된 `game_core::game::map::SupportNodeType`와
  `PlayerBehavior::ChooseSupport`를 참조한다.

### 변경/되돌림

- baseline 조사 중 production 변경이나 되돌림을 수행하지 않았다.

### 판단

- 이 두 failure는 P00 범위의 명시적 baseline이며 P00 완료 시 제거돼야 한다.
- `cargo check`만으로 현재 vertical contract가 정상이라고 판단할 수 없다.
- 각 child 시작 시 관련 worktree가 바뀌었는지 확인하고 baseline을 재실행한다.

## 2026-07-27 — Master 문서 자체 검증

### 가설

신규 master 문서가 필수 3파일, phase registry, decision link, status와 dirty-worktree 기록을 빠뜨리지
않고 production 파일 변경 없이 생성됐다.

### 실행한 정확한 명령

```text
wc -l core_enhanced/docs/goals/run_persistence_facility_monolith_policy_master/PLAN.md core_enhanced/docs/goals/run_persistence_facility_monolith_policy_master/EXPERIMENTS.md core_enhanced/docs/goals/run_persistence_facility_monolith_policy_master/EXPERIMENT_NOTES.md
rg -n "^Status:|^Implementation complete:|^## [0-9]+\\.|^\\| P0|policy_decision_required|Support/Rest|Phase 00|Phase 06" core_enhanced/docs/goals/run_persistence_facility_monolith_policy_master
rg -n "[ \\t]+$" core_enhanced/docs/goals/run_persistence_facility_monolith_policy_master
git status --short -- core_enhanced/docs/goals/run_persistence_facility_monolith_policy_master
git diff --check -- core_enhanced/docs/goals/run_persistence_facility_monolith_policy_master
```

Decision record 다섯 경로에는 각각 `test -f` 존재 확인을 수행했다.

### 실제 결과

- `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`가 모두 존재하고 끝까지 읽을 수 있다.
- P00-P06 registry, status, dependency, completion review와 policy stop rule이 문서에 존재한다.
- 다섯 decision record 경로가 모두 존재한다.
- trailing whitespace 검색 결과가 없었다.
- scoped status에는 신규 master 디렉터리만 `??`로 나타났다.
- 이 검증은 test runner를 실행하지 않았으므로 test 수를 보고하지 않는다.

### 변경/되돌림

- 검증 중 production 변경이나 되돌림은 없었다.

### 판단

- Master 문서 초기화는 완료됐다.
- Production 구현과 child Goal 생성은 아직 시작하지 않았다.

## 2026-07-27 — 구현 순서 독립 검토와 계획 보정

### 검토 범위

- Master `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`와 다섯 decision record를 다시 대조했다.
- Core SavePoint/RunPolicy/checkpoint/compendium/map generator, live map/run/PVE RON, game_server test
  mapping과 외부 Unity contract/runtime/mock/test의 관련 표면을 표본 감사했다.
- 계획이 기록한 두 baseline failure를 현재 worktree에서 직접 재현했다.

### 발견한 계획 위험

- P00이 final tagged recovery 이전의 flat/percent 혼합 상태를 독립 public/canonical 기준선으로
  완료하면 P01에서 같은 계약을 다시 migration하게 된다.
- P02가 defeated storage/API만 만들고 실제 기존 encounter producer를 P05까지 미루면 P02 완료
  시점의 사용자-visible defeated profile이 gameplay에서 채워지지 않는다.
- P02 뒤 P03 실행은 dirty worktree를 위한 선호 직렬 순서이지 hard dependency가 아니다.
- P03의 실제 facility/Unity template authoring readiness와 P04 이전 dangling difficulty reference
  금지 규칙이 명시돼 있지 않았다.
- P01/P05는 단일 patch보다 내부 milestone별 focused 검증이 필요한 범위다.
- local profile/active-run의 local-install scope를 transport `player_id` actor map과 명시적으로
  구분할 exit gate가 필요했다.

### 변경한 계획

- P00을 최종 SavePoint/Gate tagged Trauma recovery와 Support/Rest/psychology 제거 phase로 재정의했다.
- P01을 failure/checkpoint/active-run으로 좁히고 세 내부 milestone을 추가했다.
- P02에 기존 encounter의 실제 discovered/defeated producer와 local-install scope exit gate를 추가했다.
- P03의 hard prerequisite를 P01로 바로잡고 P02 뒤 실행은 선호 직렬 순서로 분류했다.
- Phase 진입 readiness gate, P03/P04 reference 경계, P04의 P03 regression review와 P05 내부 milestone을
  추가했다.
- Runtime code, live RON, server와 Unity production 파일은 수정하지 않았다.

### 실행한 명령과 결과

| 명령 | 실제 실행 test 수 | Pass | Fail | Skip | 실제 결과 |
| --- | ---: | ---: | ---: | ---: | --- |
| `cargo test -p game_core --test ron_loading -- --nocapture` | 23 | 1 | 22 | 0 | 예상한 삭제된 `Support` live RON failure 재현 |
| `cargo test -p game_server --no-run` | 0 | 0 | compile 실패 | 0 | 예상한 `SupportNodeType`/`ChooseSupport` test compile failure 재현 |

두 실패는 새 문서 변경이 만든 회귀가 아니라 P00이 제거해야 할 기존 baseline이다.

## Phase Gate Log

Phase별 구현과 완료 검토 결과를 누적한다. P07까지 완료되어 현재 master가 닫혔다.

| Phase | Child status | Completion review | Broad validation | Next action |
| --- | --- | --- | --- | --- |
| P00 | complete | Complete/High | Core 760+47, server 31, Unity Edit 245/Play 61(+4 skip) | P01 활성화 |
| P01 | complete | Complete/High | Core 762+47, server 40, Unity Edit 246/Play 61(+4 skip) | P02 활성화 |
| P02 | complete | Complete/High | Core 765+47, server 51, Unity Edit 251/Play 62(+4 skip) | P03 활성화 |
| P03 | complete | Complete/High | P04 final broad에 포함 | P04로 전달 완료 |
| P04 | complete | Complete/High | Core 775, server 52, Unity Edit 254/Play focused 1 | P05로 전달 완료 |
| P05 | complete | Complete/High | Core 786, live RON 52, server 52, Unity Edit 258 | P07로 전달 완료 |
| P06 | current master에서 제외 | 없음 | 없음 | 콘텐츠 개발 시 별도 후반 Goal |
| P07 | complete | Complete/High | Core 787+47, RON 23/52, server 52, Unity Edit 259/Play 62(+4 skip), live probe 2종 | master 완료 |
## 2026-07-27 — P00 completion과 P01 활성화

- Child: `run_policy_phase_00_savepoint_trauma_vertical_convergence`
- Completion review: `Complete / High`
- Core final validation: library 760 passed; integration 3/4/23/13/4 passed
- game_server final validation: 31 passed
- Unity final validation: EditMode 245 passed; PlayMode 61 passed, explicit live opt-in 4 skipped
- production live bundle은 SavePoint/Gate exact tagged recovery로 strict load된다.
- official Core/server/Unity flow의 Support/Rest와 SavePoint Psychology producer는 제거됐다.
- P00 exit gate를 충족해 P01을 active로 전환했다.

## 2026-07-27 — P01 completion과 P02 활성화

- Child: `run_policy_phase_01_failure_checkpoint_active_run_persistence`
- Completion review: `Complete / High`
- Core final validation: library 762 passed; integration 3/4/23/13/4 passed
- game_server final validation: 40 passed
- Unity 원본 Editor MCP final validation: EditMode 246 passed; PlayMode 61 passed,
  explicit live opt-in 4 skipped, 실패 0
- 공통 recovery/failure decision, Treatment, checkpoint monotonic round-trip과 local-install active-run
  current/backup/checksum/lock/continue/invalidation이 수직 수렴했다.
- active-run payload와 transport actor identity에서 local compendium profile이 분리된 것을 감사했다.
- P01 exit gate를 충족해 P02를 active로 전환했다.

## 2026-07-27 — P02 completion과 P03 활성화

- Child: `run_policy_phase_02_local_compendium_persistence`
- Completion review: `Complete / High`
- Rust final validation: Core library 765, integrations 3/4/23/13/4, server 51, 실패 0
- Unity 원본 Editor MCP final validation: EditMode 251 passed; PlayMode 62 passed,
  explicit live opt-in 4 skipped, 실패 0; 재기동 후 refresh/compile error 0
- local-install singleton discovered/defeated profile, dormant-preserving projection, actual encounter producer와
  별도 current/backup/checksum/lock/dirty retry lifecycle이 수직 수렴했다.
- active-run/profile transaction과 rollback이 분리되고 transport UUID별 durable authority가 제거됐다.
- P02 exit gate를 충족해 선호 직렬 순서의 P03을 active로 전환했다.

## 2026-07-27 — P05 readiness audit와 정책 입력 대기

- P03과 P04 completion review가 각각 Complete / High임을 확인하고 P05 child를 생성했다.
- Unity MCP `ark@b49d32f6f2f70bbe`는 Unity 6000.3.4f1 editor ready, compile pending 없음,
  Console error 0으로 재연결됐다.
- Live에는 세 target FinalBoss와 generic Shop-tag/capstone/difficulty/compendium seam이 있다.
- 전용 `흉탄의 사수` abnormality/skill/encounter/capstone 및 두 VN authored content는 없다.
- 기존 concept note도 실제 encounter를 `추후 미정`으로 두며 master readiness gate는 Der Freischutz
  재사용, empty placeholder와 temporary encounter 완료를 금지한다.
- P05를 `policy_decision_required`로 전환했다. Production code/data/Unity 파일은 수정하지 않았다.
- 다음 action: 사용자 authored content 제공 또는 이 child의 전용 gameplay/content 설계 권한 승인.

## 2026-07-27 — P05 콘텐츠 구현 보류 확정

- 사용자는 보스 정책을 포함한 콘텐츠 정책이 모두 정해진 뒤에만 구현한다고 결정했다.
- 현재는 `흉탄의 사수` 세부 구현을 skip하고, 작업자가 제안한 전투 방향도 승인 정책으로 간주하지
  않는다.
- Master/P05는 `policy_decision_required`를 유지하며 P05-A/B/C production migration을 시작하지 않는다.
- 완성된 콘텐츠 정책과 명시적인 구현 재개 요청이 새 입력으로 필요하다.

## 2026-07-27 — P05/P06 범위 분리와 모노리스 구현 재개

- 사용자는 `2번`을 선택해 모노리스 시스템 전체를 지금 구현하고 실제 보스 구현은 뒤로 미루도록 했다.
- P05 terminal을 변질된 모노리스 획득으로 재정의하고 status를 `active`로 전환했다.
- 실제 hidden boss actor/encounter/skill, 관리자 VN, capstone, FinalBoss 재분류와 special completion은 새
  P06 `run_policy_phase_06_hidden_boss_capstone`에 `deferred_by_user`로 등록했다. Lazy child 원칙에 따라
  P06 문서는 아직 생성하지 않았다.
- 기존 final review는 P07로 이동했다.
- Scope 문서 변경 시점까지 production runtime/data/Unity 파일은 수정하지 않았고 test도 실행하지 않았다.

## 2026-07-27 — P05 completion과 P06 사용자 보류 유지

- Child: `run_policy_phase_05_elite_monolith_remanifestation`
- Completion review: `Complete / High`
- Core final validation: library **786 passed**, live RON filtered **52 passed**, 실패 0
- game_server final validation: **52 passed**, 실패 0; monolith snake_case command fixture 포함
- Unity 원본 Editor MCP final validation: focused monolith EditMode **4/4**, broad EditMode **258/258**,
  script/compile errors 0
- 자연 Elite hard exclusion, Floor 4+ 전용 방랑 상인, typed monolith 구매, 일반 Combat node의 stage 2/3
  전환, 두 attempt, 고유 fragment, ordered residual, next-Floor Shop guarantee, accept/decline 교환 VN과
  dormant corrupted key가 Core/live RON/server/Unity/persistence에서 수직 수렴했다.
- Standard 일반 Shop과 Endless wandering merchant를 별도 ID로 분리했고 기존 Endless final-boss
  classification/completion은 변경하지 않았다.
- Hidden boss actor/skill/encounter, 관리자 VN, 전용 capstone과 special completion production identifier가
  대상 변경에 없음을 scoped search로 확인했다.
- P06은 사용자 결정대로 `deferred_by_user`이며 콘텐츠 정책 확정과 명시적 재개가 필요하다. P07은
  P06 dependency 때문에 계속 `blocked_by_dependency`다.

## 2026-07-27 — P05 이후 master 재개 감사

- 사용자 제공 재개 지침과 master 필수 문서 세 개를 다시 끝까지 읽고 phase registry, 완료 조건과
  현재 다음 작업을 대조했다.
- P00~P05 completion review가 모두 존재하고 각 final verdict가 `Complete / High`임을 확인했다.
- P06 child가 존재하지 않는 것은 lazy-child 및 `deferred_by_user` 규칙과 일치한다. P07 child도 P06
  dependency가 닫히기 전 생성하지 않는 현재 registry와 일치한다.
- P05 이후 보스 콘텐츠와 독립적으로 실행 가능한 잔여 production phase는 없다. P06의 actor/skill/
  encounter/VN/capstone/FinalBoss 재분류/특수 완료나 P07 감사를 선행하는 것은 승인 범위를 위반한다.
- 마지막 P05 검증 뒤 production code/data/consumer 변경이 없으므로 broad test를 반복하지 않았다.
  현재 재개 감사는 문서/dependency 상태 확인이며 test runner를 실행하지 않았다.
- Master는 완료가 아니다. 다음 전이는 보스 콘텐츠 정책 확정과 사용자의 명시적인 P06 재개다.

## 2026-07-27 — P06 master 분리와 P07 활성화

- 사용자는 실제 hidden-boss 작업이 상당히 후반에 수행될 콘텐츠 개발 범위이므로 현재는 README에
  함께 수행할 작업으로만 남기고, 현재 master에서는 P06을 제외한 채 P07을 진행하도록 확정했다.
- Master dependency와 완료 조건을 P00-P05 -> P07로 변경했다. 과거 P06 범위는 추적성을 위해
  `current master에서 제외 / future_content_goal`로만 남기며 child Goal은 만들지 않는다.
- README의 후반 작업 표식은 별도 콘텐츠 Goal의 actor/skill/encounter/VN/capstone/도감/persistence/
  Core/server/Unity/canonical 동기화 범위를 보존한다.
- P00, P03, P04 decision record의 오래된 `미구현` disposition을 실제 completion review pointer가 있는
  구현 이력 상태로 정정했고, 모노리스 decision은 P05 구현 범위와 후반 hidden-boss 범위를 분리했다.
- P07 `run_policy_master_final_completion_review`를 lazy 생성하고 P00-P05 결과를 독립적으로 감사한다.

## 2026-07-27 — P07 completion과 master 종료

- Child: `run_policy_master_final_completion_review`
- Completion review: `Complete / High`
- P00~P05 runtime/data/persistence/server/Unity seam을 과거 review와 독립적으로 직접 감사했다.
- 세 correction을 완료했다: 외부 probe/canonical의 SavePoint drift, Unity NodeConfirm legacy alias,
  authored facility 밖 admin SavePoint fixture.
- Rust final validation: Core library 787, integrations 3/4/23/13/4, filtered live RON 52, server 52, 실패 0.
- Unity MCP final validation: EditMode 259/259, PlayMode 62 pass + explicit live 4 skip, 실패 0,
  refresh/compile error 0.
- localhost admin non-combat 및 authored SavePoint save/load probe가 통과했다.
- hidden-boss actor/skill/encounter/VN/capstone/FinalBoss 재분류/특수 완료는 구현하지 않았다.
- 현재 master의 완료 조건을 모두 충족해 Status를 `complete`로 변경했다.
