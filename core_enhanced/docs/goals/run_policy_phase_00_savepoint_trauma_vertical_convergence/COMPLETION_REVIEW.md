# P00 SavePoint And Gate Trauma Vertical Convergence Completion Review

Reviewed: 2026-07-27

Review basis: Goal 문서의 완료 주장을 증거로 사용하지 않고 현재 runtime, live RON, DTO/server
mapping, Unity consumer, canonical과 현재 worktree 검증 결과를 다시 확인했다.

## Audit Table

| Item | Verdict | Long-term fit | 핵심 증거 |
| --- | --- | --- | --- |
| SavePoint category와 legacy flow 제거 | Complete | High | Core/RON/Unity official flow가 SavePoint 하나로 수렴하고 legacy command는 음성 test에서만 존재 |
| 공통 tagged TraumaRecovery와 live 값 | Complete | High | exact enum, strict variant field, SavePoint 15%/Gate 10% live authoring |
| Recovery 대상·계산·preview | Complete | High | Core ceil/saturating 계산, Critical/Breakdown 포함, Dead 제외, Unity는 전후 값만 표시 |
| SavePoint/Gate atomic completion | Complete | High | world transaction rollback, exact-once와 invalid pending failure test |
| Psychology 미생산 | Complete | High | producer/variant/Unity accepted kind 제거 및 condition/psychology 불변 test |
| Canonical·broad validation | Complete | High | Core/server/Unity canonical 정렬, Core 760+integration/server 31/Unity 245+61 통과 |

## SavePoint Category And Legacy Flow

Master item:
- P00 final SavePoint/Gate vertical convergence와 Support/Rest 제거

Subgoal:
- `run_policy_phase_00_savepoint_trauma_vertical_convergence`

Policy source:
- `docs/decisions/save_point_trauma_recovery_policy_draft_2026-07-25.md`

Expected behavior:
- 정식 category와 payload는 SavePoint이며 선택형 support mode, Rest와 ChooseSupport가 없다.

Runtime evidence:
- `src/game/map/types.rs`: `MapNodeCategory::SavePoint`, `MapNodePayload::SavePoint`
- `src/game/resources/selection.rs`: `SavePointSessionState`
- `src/game/world/facility.rs`: SavePoint state/recovery와 facility operation
- `src/game/world.rs`: Headquarters command는 별도 domain으로 routing

Data evidence:
- `game_resources/data/map/generation_policy.ron`: SavePoint category와
  `ensure_save_point_before_boss`
- `game_resources/data/map/node_definitions.ron`: `payload: SavePoint`

External contract evidence:
- Unity router/reader/command/presenter에서 `save_point` 독립 flow를 소비하고
  SupportChoice/Rest route와 command가 제거됐다.

Test evidence:
- game_server removed `choose_support` deserialize rejection
- Core/Unity official legacy symbol audit 0건

Legacy/fallback audit:
- game_server와 RunPolicy에 남은 legacy 문자열은 삭제된 schema를 거부하는 음성 fixture뿐이다.
- HeadquartersContact의 `HeadquartersSupportChoice`와 `headquarters_support` shop context는
  SavePoint mode가 아닌 독립 HQ domain이다.

Long-term direction review:
- Fit: high
- Improvement class: none
- Reason: category, payload, session, route와 live schema가 한 의미를 사용하며 alias/dual-read가 없다.

Verdict:
- Complete

Remaining risk:
- 없음.

## Tagged Recovery, Semantics And Projection

Master item:
- SavePoint/Gate 공통 final tagged recovery와 Core-authoritative preview

Subgoal:
- `run_policy_phase_00_savepoint_trauma_vertical_convergence`

Policy source:
- SavePoint/Trauma decision 2~6, 10~12, 14~15절

Expected behavior:
- `Flat` 또는 `CurrentPercent` 하나만 authoring하고 값은 1..100이다.
- SavePoint 15%, Gate 10%를 현재 Trauma 기준 올림해 saturating 차감한다.
- Critical/Breakdown은 포함하고 Dead는 제외하며 Physical Damage는 바꾸지 않는다.
- Unity는 Core가 계산한 preview를 재계산하지 않는다.

Runtime evidence:
- `src/game/data/run_policy_data.rs`: strict `TraumaRecovery`, validation과 recovery 계산
- `src/game/world/facility.rs`: category별 policy 적용과 직원별 authoritative preview
- `src/game/behavior.rs`: preview DTO, NodePreview/SavePointState/result contract
- `src/game/world/snapshot.rs`: NodeConfirm 및 selected-event projection

Data evidence:
- `game_resources/data/run/policy.ron`: SavePoint
  `CurrentPercent(percent: 15)`, Gate `CurrentPercent(percent: 10)`
- live loader는 legacy support group, mixed tagged fields와 unknown fields를 거부한다.

External contract evidence:
- `TraumaRecoveryPreviewModel.cs`: exact tagged reader와 required Core rows/flags
- `NodeMapSnapshotModel.cs`, `SelectedEventModel.cs`: NodeConfirm/SavePoint preview 소비
- `NodeMapScreenComposer.cs`: mode/value, Physical Damage 불변, checkpoint flag와 직원별 전후 표시

Test evidence:
- tagged parsing/zero/out-of-range/mixed-field rejection 3건
- SavePoint/Gate 정상 경로와 zero/max/Critical/Breakdown/Dead/Physical Damage assertions
- Unity focused EditMode 7/7와 SavePoint PlayMode render 1/1

Legacy/fallback audit:
- flat SavePoint field, Gate 전용 percent field와 Unity local recovery 계산이 없다.

Long-term direction review:
- Fit: high
- Improvement class: none
- Reason: 두 category가 같은 확장 가능한 tagged type과 단일 Core calculation/projection을 공유한다.

Verdict:
- Complete

Remaining risk:
- Balance 값 변경은 RunPolicy authoring 변경이며 schema migration이 아니다.

## Atomic Completion And Psychology

Master item:
- recovery, pending, node/Floor와 process-local checkpoint의 partial-commit 방지 및
  SavePoint Psychology producer 제거

Subgoal:
- `run_policy_phase_00_savepoint_trauma_vertical_convergence`

Policy source:
- SavePoint/Trauma decision 5~6, 10~12절

Expected behavior:
- SavePoint/Gate transition은 실패 시 user-visible 상태를 부분 적용하지 않고 성공 시 한 번만
  recovery한다. SavePoint는 trust/memory/event를 생산하지 않는다.

Runtime evidence:
- `src/game/world/node_flow.rs`: staged recovery와 completion/checkpoint ordering
- `src/game/world/state.rs`: non-battle whole-state transaction rollback
- `src/game/world/facility.rs`: SavePoint/Gate effect만 적용
- `src/game/employee_psychology.rs`: 현재 production event variant가 없고 recovery producer가 없다.

Data evidence:
- Psychology나 per-node override를 만드는 SavePoint/Gate live field가 없다.

External contract evidence:
- Unity Roster reader는 제거된 Rest psychology kind를 허용하지 않으며 recovery preview에
  psychology mutation을 추론하지 않는다.

Test evidence:
- SavePoint/Gate exact-once assertions
- SavePoint와 Gate invalid pending 연구 시 전체 snapshot rollback
- recovery 전후 Psychology equality와 checkpoint replacement/no-replacement assertions

Legacy/fallback audit:
- `RecoveredAtSavePoint`, `RestedAtSupportRest` producer/DTO/consumer/test는 0건이다.

Long-term direction review:
- Fit: high
- Improvement class: none
- Reason: recovery와 transition이 동일 transaction boundary를 사용하고 Psychology와 ownership이
  분리됐다.

Verdict:
- Complete

Remaining risk:
- durable active-run file publish 원자성은 명시적으로 P01 범위다. P00 process-local 의미의 결함이 아니다.

## Cross-Component Findings

- Core live RunPolicy, runtime application, preview DTO, server mapping과 Unity reader/presentation은
  동일한 tagged recovery와 15%/10% 값을 사용한다.
- SavePoint completion result와 root snapshot이 경쟁하는 recovery source를 만들지 않는다.
  Unity는 snapshot/result의 Core projection만 표시한다.
- Gate는 checkpoint를 교체하지 않고 SavePoint만 최종 completion state를 capture한다.
- Canonical에 구 SupportState/Rest/ChooseSupport current contract가 남지 않는다.
- 별도 HeadquartersContact support 명칭은 SavePoint node contract와 타입/route/action이 분리돼 있다.

## Immediate Correction Candidates

- 없음. 감사 중 발견한 mixed tagged-field 허용, stale map rejection fixture와
  `ensure_support_before_boss` 명칭은 최종 verdict 전에 수정하고 재검증했다.

## Follow-up Refactor Candidates

- 없음. Durable active-run, failure decision/recovery restriction과 Treatment는 승인된 P01 기능
  범위이며 P00의 미완료 refactor가 아니다.

## Final Validation

- `cargo check -p game_core`: success
- `cargo check -p game_server`: success
- `cargo test -p game_core --test ron_loading -- --nocapture`: 23 passed
- `cargo test -p game_core`: library 760 passed; integration 3/4/23/13/4 passed
- `cargo test -p game_server`: 31 passed
- Unity full EditMode: 245 passed
- Unity full PlayMode: 61 passed, 4 explicit live opt-in skipped
- Unity compile/refresh: 0 errors

## Policy Questions

- 없음.

## Final Verdict

- Verdict: Complete
- Long-term fit: High
- Improvement class: none
- P01 activation recommendation: 승인. P00 public schema와 recovery 의미를 재교체하지 않고
  durable checkpoint/active-run과 failure decision을 확장할 수 있다.
