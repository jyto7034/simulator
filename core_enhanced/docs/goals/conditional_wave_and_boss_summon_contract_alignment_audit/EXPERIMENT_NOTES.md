# Experiment Notes — Conditional Wave and Boss Summon Contract Alignment Audit

## 초기화 기록

- Created: 2026-08-17
- Goal path:
  `core_enhanced/docs/goals/conditional_wave_and_boss_summon_contract_alignment_audit`
- 경로 충돌: 없음
- 초기 Status: `active`
- Goal mode: `audit_only`
- Implementation authorized: `No`
- 감사 finding ID: 없음

## 초기 Dirty Worktree 주의사항

신규 Goal 문서 생성 전 다음 read-only 확인만 수행했다.

```bash
git -C core_enhanced rev-parse --show-toplevel
git -C core_enhanced status --short
```

- repository root는 `/mnt/f/work/simulator`다.
- 다수의 modified/deleted/untracked 사용자 변경이 존재한다.
- 감사와 직접 겹칠 가능성이 높은 `docs/game_rulebook.md`, Core battle/runtime/data, live RON,
  `game_server`와 기존 conditional-wave policy artifacts도 dirty 또는 untracked 상태다.
- 이 시점에는 기존 diff의 의미, 작성자 또는 완결성을 판정하지 않았다.
- 기존 변경을 restore, reset, stash, format 또는 정리하지 않는다.
- 감사 evidence에는 조사 당시 exact path와 relevant dirty state를 함께 기록한다.
- 조사 중 대상 diff가 변하면 기존 observation을 자동 재사용하지 않고 영향받은 row만 재확인한다.

## 구조적 판단

- 이 Goal의 deliverable은 하나의 read-only alignment audit다.
- 임시 정책 노트는 확장 후보를 찾는 입력이며 승인된 정책 source가 아니다.
- 기존 current Final Policy Freeze가 승인된 목표 의미의 우선 근거다.
- `game_rulebook.md`는 durable gameplay 표현이고, runtime/live RON은 current actual 근거다.
- 감사 finding의 correction은 서로 다른 source-of-truth 경계를 가질 수 있으므로 이 Goal에서 구현하지 않는다.
- 구현이 필요한 finding은 후속 후보로만 기록하며 새 Goal을 자동 생성하지 않는다.

## Source-of-Truth 충돌

ASB-002 current source에서 25개 Freeze member를 대조했다. runtime/live RON와 Freeze가 충돌하는 행은
발견하지 않았다. 다만 current runtime이 이미 갖는 세 가지 authored/user-visible 의미가
`game_rulebook.md`에 충분히 명시되지 않은 canonical-document finding으로 남았다.

- `CWBS-AUD-001` / `WV-03`: 같은 timestamp의 Wave ordering은 runtime에서 priority 내림차순, stable trigger
  ID 오름차순인데 rulebook은 field 존재만 말하고 ordering을 명시하지 않는다.
- `CWBS-AUD-002` / `WV-05`: `ActorReachedPoint.radius_cells`의 거리 metric은 runtime에서 Chebyshev인데,
  Freeze/rulebook은 ``반경``이라고만 표현한다.
- `CWBS-AUD-003` / `BR-05`: 이미 commit된 batch에 signal/lifetime/summoner-removal 조건이 같은 reconciliation
  boundary에 함께 관찰될 때 runtime은 `Despawn summoner removal → signal → lifetime` 순으로 하나를 택한다.
  Freeze/rulebook은 first-cause 원칙만 말하고 이 tie-break를 명시하지 않는다.

이 세 finding은 runtime 구현 결함이 아니라 canonical 문서가 current actual을 충분히 표현하지 못한 경우다.
현재 user delegation에 따라 각각 runtime의 deterministic rule을 권장 canonical value로 제안한다. 이 Goal은
audit-only이므로 문서를 수정하지 않는다.

## Audit Source Basis — ASB-001

- Basis date: 2026-08-17
- Repository root: `/mnt/f/work/simulator`
- Working Policy SHA-256:
  `ef8b18ebb55cb3c60b8b9031f12d102308622f3a7a29903a540644bfff6e7ceb`
- Current Final Policy Freeze:
  `FPF-CWBS-P2G30-002` /
  `4c94407a83e3252635d2e033fe792e98dbafa6ce202b0cfc8f52d910780838e8`
- `docs/game_rulebook.md` SHA-256:
  `3aeec0e8e3eab5c57002083e0cead4f29003c31b39c00f5f877cbb40f08ac086`
- `docs/core_runtime_contract.md` SHA-256:
  `73bc18d42f11b54a69f6d861c675cda7d35f6e2090f6b34cb23e539e3d8364f6`
- `docs/data_loading_contract.md` SHA-256:
  `58c636ceacb990a328644de8bfd44cb5ca85bf744d5ef680eaba00dc081b5c7b`
- Source disposition: current dirty workspace actual; clean HEAD 또는 Released/Deployed claim이 아님
- Freeze currentness evidence:
  `WORKING_POLICY.ko.md:67-85, 320-342, 358-398`와
  `P2G_30_FINAL_POLICY_FREEZE_002.ko.md:78-151, 239-276`
- Freeze coverage: exact 25 members / exact 25 acceptance rows

## Audit Source Basis — ASB-002 (재개)

- Basis date: 2026-08-20
- Repository root: `/mnt/f/work/simulator`
- Working Policy SHA-256:
  `ef8b18ebb55cb3c60b8b9031f12d102308622f3a7a29903a540644bfff6e7ceb` (ASB-001과 동일)
- Current Final Policy Freeze:
  `FPF-CWBS-P2G30-002` /
  `4c94407a83e3252635d2e033fe792e98dbafa6ce202b0cfc8f52d910780838e8` (ASB-001과 동일)
- `docs/game_rulebook.md` SHA-256:
  `8aa954cc215dccc63b736f884327ca03375f82895b5e86eb2252e597e8fcbf6f`
  (ASB-001 이후 변경됨)
- `docs/core_runtime_contract.md` SHA-256:
  `73bc18d42f11b54a69f6d861c675cda7d35f6e2090f6b34cb23e539e3d8364f6`
  (ASB-001과 동일)
- `docs/data_loading_contract.md` SHA-256:
  `58c636ceacb990a328644de8bfd44cb5ca85bf744d5ef680eaba00dc081b5c7b`
  (ASB-001과 동일)
- Source disposition: current dirty workspace actual; clean HEAD 또는 Released/Deployed claim이 아님
- 재개 판단: rulebook 변경 때문에 모든 행의 Rulebook clause를 ASB-002에서 다시 확인한다. Freeze currentness,
  runtime/data canonical basis는 digest만으로 ASB-001과 동일하다고 확인했다.

## 임시 정책 노트 Section Disposition

다음 25개 policy section은 모두 exact same-ID current Freeze member에 포함된다.

- `SP-01`, `SP-02`
- `WV-01`, `WV-02`, `WV-03`, `WV-04`, `WV-05`, `WV-06`, `WV-07`
- `SM-01`, `SM-02`, `SM-03`, `SM-04`
- `BR-01`, `BR-02`, `BR-03`, `BR-04`, `BR-05`
- `BL-01`, `BL-02`, `BL-03`, `BL-04`, `BL-05`
- `PS-01`, `RF-01`

임시 노트의 M0~M6는 역사적 milestone이고 별도 policy element가 아니다. `알려진 현재 구현 차이`는 구현 전
기준선이므로 current actual로 재사용하지 않는다. Goal 분할 방침·후속 진행 원칙도 실행 history이며 새
정책 section이 아니다.

## 정책 질문

초기에는 없음. 감사 중 사용자-visible 의미를 새로 선택해야 하는 경우 질문과 영향을 기록하고 Goal Status를
`policy_decision_required`로 변경한다.

## 외부 시스템과 직접 의존성

- `game_resources`: production live RON이 실제 loader 경로에 선택되는지 확인할 때만 조사
- `game_server`: Core 결과의 server-visible projection이 해당 policy row에 직접 관련될 때만 조사
- Unity: 명시적 비범위이며 검증한 것으로 표현하지 않음

## 적용하지 않은 대안과 이유

- P2G-10부터 P2G-60까지 즉시 재수행: 현재 목적은 새 정책 authoring이 아니라 current alignment audit이므로
  선택하지 않았다. Material policy finding이 확인되면 그 finding만 P2G-10 후보로 보고한다.
- 기존 완료 Goal을 재개해 수정: 과거 execution history와 현재 감사 결과를 혼합하므로 선택하지 않았다.
- 전체 test suite 선실행: production 변경이 없고 row별 current actual 확인을 직접 증명하지 않으므로 생략한다.
- 임시 정책 노트를 expected behavior로 직접 사용: non-canonical 문서이므로 금지한다.

## 접근하거나 검증하지 못한 소비자

- Unity reader/store/command/presentation: 사용자 승인 비범위
- 배포된 외부 consumer: 존재 및 배포 상태를 현재 확인하지 않음

필수 consumer로 판명되면 접근 가능성을 재평가한다. 접근 불가가 핵심 verdict를 막으면
`externally_blocked`, 필수가 아니면 `evidence_unavailable`과 남은 위험으로 기록한다.

## 생략한 검증과 근거

- Core 및 game_server 전수 test: read-only 감사 초기화 단계이며 production 변경 없음
- Unity 검증: 명시적 비범위
- migration/compatibility test: migration과 compatibility 변경 없음
- production 콘텐츠 양적 검사: 콘텐츠 설계 비범위

정적 근거가 불충분한 개별 row의 focused 검증까지 자동 생략하는 근거는 아니다.

## Alignment Matrix

| Row ID | Policy element / observable behavior | Notes ref | Freeze ref | Rulebook ref | Runtime entrypoint → mutation/result | Live RON/loader | Consumer/test seam | Verdict | Impact / follow-up |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `SP-01` | Wave/Summon의 stable request, exact-once settlement, terminal 전 spawn 정산 | tmp `SP-01` | `PER-CWBS-SP-01-CODE-006` | `§적 구성과 웨이브` 630-642 | `spawn_kernel.rs:218-393`; `sim.rs:1844-1876` | loader `live_ron.rs:144,268-312`; ref encounter 2756-2847 | E-003 Wave/Kernel 20 pass | `aligned` | follow-up 없음 |
| `SP-02` | Wave fixed-cell 및 Summon value-bearing occupancy/all-or-none | tmp `SP-02` | `PER-CWBS-SP-02-CODE-006` | `§적 구성과 웨이브` 630-642; detailed runtime contract | `spawn_kernel.rs:57-130,235-385`; `summon_runtime.rs:629-749` | loader/compiled schema; ref uses `RequireUnoccupied` | E-003/E-004; catalog validation seam | `aligned` | live content는 모든 enum variant를 사용할 필요 없음 |
| `WV-01` | required Wave spawn/clear 전 Victory 보류, Defeat 우선 | tmp `WV-01` | `PER-CWBS-WV-01-CODE-003` | `§적 구성과 웨이브` 632-638; `§노드 결과 처리` | `sim.rs:1217-1260,1847-1876` | ref two required groups 2808-2847 | E-003 `primary_wins...` | `aligned` | follow-up 없음 |
| `WV-02` | closed primary trigger set과 one-shot activation | tmp `WV-02` | `PER-CWBS-WV-02-CODE-003` | `§적 구성과 웨이브` 632-635 | `pve_data.rs:107-140,1483-1539`; `wave_runtime.rs:145-304` | `encounters.ron` activation payload | E-003; pve validator seam | `aligned` | follow-up 없음 |
| `WV-03` | same-bucket Wave order, primary-over-deadline, threshold-before-terminal | tmp `WV-03` | `PER-CWBS-WV-03-CODE-003` | `§적 구성과 웨이브` 632-638 (ordering omission) | `wave_runtime.rs:141-173,247-304`; `sim.rs:1844-1876` | applicable generic schema; ref HP trigger 2827-2845 | E-003 same-bucket/primary/threshold tests | `canonical_document_gap` | `CWBS-AUD-001`: rulebook에 priority↓/trigger ID↑를 명시 |
| `WV-04` | death와 non-kill despawn이 clear에 기여; despawn은 kill credit 없음 | tmp `WV-04` | `PER-CWBS-WV-04-CODE-003` | `§Live Boss Runtime Reference` 661-662 | `wave_runtime.rs:306-319`; `summon_runtime.rs:604-625` | cleanup signal ref 137-235 | E-003 group clear, E-004 removal tests | `aligned` | reference와 Core contract가 generic rule을 뒷받침 |
| `WV-05` | bound actor HP/death/point observer, bind-time evaluation | tmp `WV-05` | `PER-CWBS-WV-05-CODE-006` | `§적 구성과 웨이브` 632-635 (metric omission) | `wave_runtime.rs:209-304` (`Position::chebyshev`) | schema `pve_data.rs:123-128,1525-1539` | E-003 HP/point/death tests | `canonical_document_gap` | `CWBS-AUD-002`: `radius_cells`가 Chebyshev tile distance임을 canonicalize |
| `WV-06` | impossible/cyclic required dependency rejection, optional cancellation cascade | tmp `WV-06` | `PER-CWBS-WV-06-CODE-003` | `§적 구성과 웨이브` 637-638 | `pve_data.rs:1216-1368`; `wave_runtime.rs:176-205,321-359` | publication validator | E-003 cancellation test; pve validator seam | `aligned` | follow-up 없음 |
| `WV-07` | primary/deadline first-cause, invalid time authoring rejection | tmp `WV-07` | `PER-CWBS-WV-07-CODE-003` | `§적 구성과 웨이브` 637-638 | `wave_runtime.rs:247-260`; `pve_data.rs:1483-1539` | ref has HP primary plus 5s deadline | E-003 primary/deadline test | `aligned` | follow-up 없음 |
| `SM-01` | independent summon provenance, no Boss/Wave/reward/compendium authority | tmp `SM-01` | `PER-CWBS-SM-01-CODE-003` | `§Live Boss Runtime Reference` 661-662 | `summon_runtime.rs:270-470`; `spawn_kernel.rs:37-49` | `summoned_units/base.ron:3-31` | E-004 `successful_batch...` | `aligned` | follow-up 없음 |
| `SM-02` | summon definition owns stats; no summoner inheritance/hidden phase multiplier | tmp `SM-02` | `PER-CWBS-SM-02-CODE-003` | `§Live Boss Runtime Reference` 661-662 | `summoned_unit_data.rs`; `summon_runtime.rs:270-470` | `summoned_units/base.ron:5-30` | E-004 independent profile test | `aligned` | follow-up 없음 |
| `SM-03` | per-summoner max alive/total; failed request consumes neither, clear frees alive only | tmp `SM-03` | `PER-CWBS-SM-03-CODE-003` | detailed runtime/data contract | `summon_runtime.rs:151-268,536-573` | `summoned_units/base.ron:27-29` | E-004 limit/counter tests | `aligned` | rulebook detail is not independently player-facing |
| `SM-04` | Persist/Despawn after commit; same-timestamp unresolved removal cancels request | tmp `SM-04` | `PER-CWBS-SM-04-CODE-006` | `§Live Boss Runtime Reference` 661-662 | `summon_runtime.rs:151-193,473-534` | minion `on_summoner_removed: Despawn` | E-003 removal-first; E-004 Persist/Despawn | `aligned` | follow-up 없음 |
| `BR-01` | typed Summon task and `Spawned/LimitReached/NoSpawnSpace/Canceled` result | tmp `BR-01` | `PER-CWBS-BR-01-CODE-003` | `§Live Boss Runtime Reference` 646-665 | `boss_runtime_data.rs:193-280`; `summon_runtime.rs:151-268` | summon task/transitions 137-203 | E-004 result path | `aligned` | follow-up 없음 |
| `BR-02` | pattern-scoped UUID batch handle; exact once `SummonBatchCleared` | tmp `BR-02` | `PER-CWBS-BR-02-CODE-003` | `§Live Boss Runtime Reference` 661-662 | `boss_runtime.rs` active pattern handles; `summon_runtime.rs:576-599` | `reference_batch` 147-203 | E-004 exact batch-clear test | `aligned` | follow-up 없음 |
| `BR-03` | Around/Zone ordering, occupancy, atomic fail/bounded wait, removal-first | tmp `BR-03` | `PER-CWBS-BR-03-CODE-006` | detailed runtime/data contract | `summon_runtime.rs:151-268,629-749`; `boss_runtime_data.rs:1180-1325` | live reference Around radius 2 / RequireUnoccupied | E-004 placement/waiter test | `aligned` | follow-up 없음 |
| `BR-04` | existing task/movement graph only; summon has no encounter/Wave/terminal authority | tmp `BR-04` | `PER-CWBS-BR-04-CODE-003` | `§Live Boss Runtime Reference` 650-665 | `boss_runtime.rs:520+`; compiler rejects summon recursion | live compiled actor behavior | Boss runtime/data test seam (static) | `aligned` | no live actor lifecycle DTO claim |
| `BR-05` | lifetime/signal/summoner removal, mode, nonretroactive exact batch cleanup | tmp `BR-05` | `PER-CWBS-BR-05-CODE-003` | `§Live Boss Runtime Reference` 661-662 (tie-break omission) | `summon_runtime.rs:473-625`; `boss_runtime.rs:1530+` | lifetime/signal data 147-235 | E-004 lifetime/signal test | `canonical_document_gap` | `CWBS-AUD-003`: current tie-break `summoner removal → signal → lifetime`를 canonicalize |
| `BL-01` | authored terminal 없으면 battle 지속; post-battle run failure separate | tmp `BL-01` | `PER-CWBS-BL-01-CODE-003` | `§노드 결과 처리` 1081-1105 | `sim.rs:1217-1260`; `world/combat.rs:1246-1293` | ref terminal outcomes 2774-2802 | terminal source seam | `aligned` | follow-up 없음 |
| `BL-02` | Defeat-first terminal selection, priority then stable outcome ID | tmp `BL-02` | `PER-CWBS-BL-02-CODE-003` | `§노드 결과 처리` 1086-1089 | `sim.rs:1217-1238` | ref named outcomes 2774-2802 | terminal source seam | `aligned` | follow-up 없음 |
| `BL-03` | Pause stops simulation time; time limit/Victory same bucket Defeat first | tmp `BL-03` | `PER-CWBS-BL-03-CODE-003` | `§노드 결과 처리` 1086-1089 | `world/combat.rs:1494-1529,1691-1775`; `sim.rs:1217-1238` | N/A — content chooses time outcome | server pause test seam; static | `aligned` | server runner ICE recorded in E-006 |
| `BL-04` | named outcome policy, atomic/idempotent result consumption | tmp `BL-04` | `PER-CWBS-BL-04-CODE-003` | `§노드 결과 처리` 1081-1105 | `world/combat.rs:1344-1406,1494-1605` | ref outcome policies 2788-2802 | Core World/server source seam | `aligned` | Unity consumer is out of scope |
| `BL-05` | attempt candidates survive death/despawn/reconnect; Victory-only registration | tmp `BL-05` | `PER-CWBS-BL-05-CODE-006` | `§영구 도감` 788-799 | `spawn_kernel.rs:432-455`; `world/combat.rs:729-749` | ref contains only nonproduction candidates | E-003 ledger test; Core World static | `aligned` | process death discards active battle per PS-01 |
| `PS-01` | process-alive disconnect Pause/reconnect; process death checkpoint restore; load use | tmp `PS-01` | `PER-CWBS-PS-01-CODE-006` | `§SavePoint 노드` 1012-1036 | `world/combat.rs:1691-1775`; server handlers 26-150 | active battle excluded from checkpoint; loader N/A | existing server test; E-006 compiler ICE | `aligned` | no server test pass claim; static evidence remains |
| `RF-01` | nonproduction reference is vertical-path only, excluded from normal pool/rewards/progression | tmp `RF-01` | `PER-CWBS-RF-01-CODE-003` | `ReferenceFinalBoss` 625-626; `Live Boss Runtime Reference` 646-665 | PVE selector/validation; compiled reference runtime | encounter 2756-2847; boss/summon RON | E-005 live publication test | `aligned` | promotion requires separate content review |

## Canonical-document Finding Detail

### CWBS-AUD-001 — Same-bucket Wave ordering

- Current actual: ready Wave triggers sort by priority descending and then stable trigger ID ascending before each
  settlement (`wave_runtime.rs:155-173`).
- Existing approved policy: exact same ordering is in `PER-CWBS-WV-03-CODE-003`.
- Canonical gap: `game_rulebook.md` names `priority` and stable trigger ID but does not state the tie-breaking order.
- User-visible/authoring impact: same-time Wave authoring can otherwise produce an unknowable spawn order.
- Recommended correction Goal: add that one sentence to the Wave rulebook clause; do not change runtime or RON.

### CWBS-AUD-002 — Actor point trigger metric

- Current actual: `ActorReachedPoint.radius_cells` uses Chebyshev distance from the actor projected tile to the
  authored tactical point (`wave_runtime.rs:286-302`).
- Existing approved policy: actor-point trigger is closed but its metric is not named.
- Canonical gap: `game_rulebook.md` only calls this a point/radius trigger.
- User-visible/authoring impact: diagonal cells differ from Euclidean/Manhattan interpretations.
- Recommended correction Goal: state that `radius_cells` is Chebyshev tile distance. This follows current runtime and
  needs no schema or compatibility change.

### CWBS-AUD-003 — Committed batch removal tie-break

- Current actual: if a committed Despawn-on-summoner-removal batch sees multiple removal conditions at one
  reconciliation boundary, runtime chooses summoner removal, then signal, then lifetime
  (`summon_runtime.rs:500-519`).
- Existing approved policy: Freeze defines first-cause behavior but not this co-observed boundary tie-break.
- Canonical gap: neither runtime contract nor game rulebook exposes this deterministic order.
- User-visible/authoring impact: different signal/lifetime modes can make the resulting death versus non-kill despawn
  observable.
- Recommended correction Goal: explicitly adopt current order `summoner removal → signal → lifetime`; it preserves the
  already implemented deterministic behavior and requires no migration.

## 범위 밖 후속 후보

다음 finding은 현재 Goal과 독립적으로 수정·검증할 수 있으므로 후속 후보로만 기록한다.

- `CWBS-AUD-001`: `game_rulebook.md` same-bucket Wave ordering 문장 보강. 문서 단일 책임, policy decision
  required 아님.
- `CWBS-AUD-002`: `game_rulebook.md` point radius metric 문장 보강. 문서 단일 책임, current runtime
  Chebyshev를 canonicalize.
- `CWBS-AUD-003`: `game_rulebook.md`와 필요한 runtime canonical에 committed batch removal tie-break 문장 보강.
  문서 단일 책임, current runtime precedence를 canonicalize.

새 Goal이나 Master Goal은 자동으로 만들지 않는다.

## 남은 위험

- 감사 대상 source가 이미 대규모 dirty 상태이므로 clean HEAD가 아니라 ASB-002 current working tree actual을
  판정했다. 대상 source가 다시 바뀌면 이 결과는 stale해진다.
- Unity reader/store/command/presentation과 배포된 외부 consumer는 이번 Goal에서 확인하지 않았다.
- game_server reconnect focused test는 Rust compiler ICE로 실행되지 않았다. Core/server static trace는 남지만,
  server test current pass 증거는 없다.
- `CWBS-AUD-001`~`003` canonical 문서 correction이 완료되기 전에는 game_rulebook-only 독자가 세 semantics를
  완전하게 재현할 수 없다.

## 최종 감사 결론

- Final Freeze coverage: 25/25 Matrix row, 누락/중복 없음.
- Final verdict: `aligned` 22개, `canonical_document_gap` 3개, `runtime_implementation_gap` 0개,
  `live_data_gap` 0개, `policy_undefined` 0개, `evidence_unavailable` 0개.
- `canonical_document_gap`: `CWBS-AUD-001` (`WV-03` ordering), `CWBS-AUD-002` (`WV-05` Chebyshev metric),
  `CWBS-AUD-003` (`BR-05` committed removal tie-break).
- No automatic correction Goal was created. 이 Goal은 audit-only이며 user-authorized production/canonical edit가
  없다.
