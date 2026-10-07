# P2G-10 Policy Closure — 조건부 Wave·보스 소환

이 파일은 `WPD-CWBS-2026-08-15`의 P2G-10 independent artifact와 derived view를 materialize하는 물리
container다. 이 파일 자체는 Artifact Catalog의 독립 artifact가 아니며 Exact Artifact Ref를 갖지 않는다.

## Evaluation Basis

- Selected Stage: `P2G-10`
- Input Working Revision Ref: `Working Revision Ref: WR-CWBS-INITIAL-001; Content Digest: N/A`
- Exact user-approved target scope/change request: `N/A — initial approved scope unchanged`
- Evaluated Policy Intake Snapshot Ref: `Policy Intake Snapshot Ref: PIS-CWBS-INITIAL-001; Content Digest: N/A`
- Exact policy source: `INTAKE-CWBS-001`, source digest
  `sha256:80ca6a1ec917db3b1b9363e120d10fcd45f2c0042ebc2f34a21b8b22211b8f08`
- Exact user approval basis: `USR-CWBS-001`가 위 source를 임시 정책으로 지정했고 `USR-CWBS-004`가 그
  current input에 대한 P2G-10 수행을 요청함
- Raw policy input change: `none`; successor Policy Intake Snapshot을 만들지 않음
- Final Policy Freeze/change basis: `N/A — no predecessor Freeze`

아래 모든 Candidate Policy Element Disposition Record payload는 별도 표기와 무관하게 다음 세 필드를
canonical member로 포함한다.

```text
Evaluated Input Working Revision Ref = WR-CWBS-INITIAL-001
Approved Target Scope/Change Request Ref = N/A
Evaluated Policy Intake Snapshot Ref = PIS-CWBS-INITIAL-001
```

## Candidate Policy Element Disposition Records

### Promoted candidates

| Disposition Record Ref | Candidate ID | Exact source reference | Predecessor record | Result | Evidence/approval | Stable Element ID | Initial open revision |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `CPEDR-CWBS-SP-01-001` | `CAND-CWBS-SP-01` | `INTAKE-CWBS-001#SP-01` | `N/A` | `promoted` | `USR-CWBS-001, USR-CWBS-004` | `PE-CWBS-SP-01` | `PER-CWBS-SP-01-OPEN-001` |
| `CPEDR-CWBS-SP-02-001` | `CAND-CWBS-SP-02` | `INTAKE-CWBS-001#SP-02` | `N/A` | `promoted` | same | `PE-CWBS-SP-02` | `PER-CWBS-SP-02-OPEN-001` |
| `CPEDR-CWBS-WV-01-001` | `CAND-CWBS-WV-01` | `INTAKE-CWBS-001#WV-01` | `N/A` | `promoted` | same | `PE-CWBS-WV-01` | `PER-CWBS-WV-01-OPEN-001` |
| `CPEDR-CWBS-WV-02-001` | `CAND-CWBS-WV-02` | `INTAKE-CWBS-001#WV-02` | `N/A` | `promoted` | same | `PE-CWBS-WV-02` | `PER-CWBS-WV-02-OPEN-001` |
| `CPEDR-CWBS-WV-03-001` | `CAND-CWBS-WV-03` | `INTAKE-CWBS-001#WV-03` | `N/A` | `promoted` | same | `PE-CWBS-WV-03` | `PER-CWBS-WV-03-OPEN-001` |
| `CPEDR-CWBS-WV-04-001` | `CAND-CWBS-WV-04` | `INTAKE-CWBS-001#WV-04` | `N/A` | `promoted` | same | `PE-CWBS-WV-04` | `PER-CWBS-WV-04-OPEN-001` |
| `CPEDR-CWBS-WV-05-001` | `CAND-CWBS-WV-05` | `INTAKE-CWBS-001#WV-05` | `N/A` | `promoted` | same | `PE-CWBS-WV-05` | `PER-CWBS-WV-05-OPEN-001` |
| `CPEDR-CWBS-WV-06-001` | `CAND-CWBS-WV-06` | `INTAKE-CWBS-001#WV-06` | `N/A` | `promoted` | same | `PE-CWBS-WV-06` | `PER-CWBS-WV-06-OPEN-001` |
| `CPEDR-CWBS-WV-07-001` | `CAND-CWBS-WV-07` | `INTAKE-CWBS-001#WV-07` | `N/A` | `promoted` | same | `PE-CWBS-WV-07` | `PER-CWBS-WV-07-OPEN-001` |
| `CPEDR-CWBS-SM-01-001` | `CAND-CWBS-SM-01` | `INTAKE-CWBS-001#SM-01` | `N/A` | `promoted` | same | `PE-CWBS-SM-01` | `PER-CWBS-SM-01-OPEN-001` |
| `CPEDR-CWBS-SM-02-001` | `CAND-CWBS-SM-02` | `INTAKE-CWBS-001#SM-02` | `N/A` | `promoted` | same | `PE-CWBS-SM-02` | `PER-CWBS-SM-02-OPEN-001` |
| `CPEDR-CWBS-SM-03-001` | `CAND-CWBS-SM-03` | `INTAKE-CWBS-001#SM-03` | `N/A` | `promoted` | same | `PE-CWBS-SM-03` | `PER-CWBS-SM-03-OPEN-001` |
| `CPEDR-CWBS-SM-04-001` | `CAND-CWBS-SM-04` | `INTAKE-CWBS-001#SM-04` | `N/A` | `promoted` | same | `PE-CWBS-SM-04` | `PER-CWBS-SM-04-OPEN-001` |
| `CPEDR-CWBS-BR-01-001` | `CAND-CWBS-BR-01` | `INTAKE-CWBS-001#BR-01` | `N/A` | `promoted` | same | `PE-CWBS-BR-01` | `PER-CWBS-BR-01-OPEN-001` |
| `CPEDR-CWBS-BR-02-001` | `CAND-CWBS-BR-02` | `INTAKE-CWBS-001#BR-02` | `N/A` | `promoted` | same | `PE-CWBS-BR-02` | `PER-CWBS-BR-02-OPEN-001` |
| `CPEDR-CWBS-BR-03-001` | `CAND-CWBS-BR-03` | `INTAKE-CWBS-001#BR-03` | `N/A` | `promoted` | same | `PE-CWBS-BR-03` | `PER-CWBS-BR-03-OPEN-001` |
| `CPEDR-CWBS-BR-04-001` | `CAND-CWBS-BR-04` | `INTAKE-CWBS-001#BR-04` | `N/A` | `promoted` | same | `PE-CWBS-BR-04` | `PER-CWBS-BR-04-OPEN-001` |
| `CPEDR-CWBS-BR-05-001` | `CAND-CWBS-BR-05` | `INTAKE-CWBS-001#BR-05` | `N/A` | `promoted` | same | `PE-CWBS-BR-05` | `PER-CWBS-BR-05-OPEN-001` |
| `CPEDR-CWBS-BL-01-001` | `CAND-CWBS-BL-01` | `INTAKE-CWBS-001#BL-01` | `N/A` | `promoted` | same | `PE-CWBS-BL-01` | `PER-CWBS-BL-01-OPEN-001` |
| `CPEDR-CWBS-BL-02-001` | `CAND-CWBS-BL-02` | `INTAKE-CWBS-001#BL-02` | `N/A` | `promoted` | same | `PE-CWBS-BL-02` | `PER-CWBS-BL-02-OPEN-001` |
| `CPEDR-CWBS-BL-03-001` | `CAND-CWBS-BL-03` | `INTAKE-CWBS-001#BL-03` | `N/A` | `promoted` | same | `PE-CWBS-BL-03` | `PER-CWBS-BL-03-OPEN-001` |
| `CPEDR-CWBS-BL-04-001` | `CAND-CWBS-BL-04` | `INTAKE-CWBS-001#BL-04` | `N/A` | `promoted` | same | `PE-CWBS-BL-04` | `PER-CWBS-BL-04-OPEN-001` |
| `CPEDR-CWBS-BL-05-001` | `CAND-CWBS-BL-05` | `INTAKE-CWBS-001#BL-05` | `N/A` | `promoted` | same | `PE-CWBS-BL-05` | `PER-CWBS-BL-05-OPEN-001` |
| `CPEDR-CWBS-PS-01-001` | `CAND-CWBS-PS-01` | `INTAKE-CWBS-001#PS-01` | `N/A` | `promoted` | same | `PE-CWBS-PS-01` | `PER-CWBS-PS-01-OPEN-001` |
| `CPEDR-CWBS-RF-01-001` | `CAND-CWBS-RF-01` | `INTAKE-CWBS-001#RF-01` | `N/A` | `promoted` | same | `PE-CWBS-RF-01` | `PER-CWBS-RF-01-OPEN-001` |

`same`은 바로 위 첫 row와 동일한 exact approval set `USR-CWBS-001, USR-CWBS-004`를 뜻하며 record
payload에서 생략된 값이 아니다.

### Non-element and out-of-scope candidates

| Disposition Record Ref | Candidate ID | Exact source reference | Result | Primary classification | Disposition evidence |
| --- | --- | --- | --- | --- | --- |
| `CPEDR-CWBS-DOC-001` | `CAND-CWBS-DOCUMENT-RESPONSIBILITY` | `INTAKE-CWBS-001#문서-책임` | `not_a_policy_element` | `implementation_finding` | document status/source precedence metadata; `USR-CWBS-001` |
| `CPEDR-CWBS-CONTEXT-001` | `CAND-CWBS-BACKGROUND` | `INTAKE-CWBS-001#논의-배경과-목표` | `not_a_policy_element` | `implementation_finding` | rationale/context only |
| `CPEDR-CWBS-MILESTONE-001` | `CAND-CWBS-M0-M6` | `INTAKE-CWBS-001#잠정-마일스톤` | `not_a_policy_element` | `implementation_finding` | provisional planning input; P2G-40 owns final phase structure |
| `CPEDR-CWBS-CURRENT-DELTA-001` | `CAND-CWBS-KNOWN-DELTAS` | `INTAKE-CWBS-001#알려진-현재-구현-차이` | `not_a_policy_element` | `evidence_gap` | current-tree claims require P2G-20 review |
| `CPEDR-CWBS-GOAL-SPLIT-001` | `CAND-CWBS-GOAL-SPLIT` | `INTAKE-CWBS-001#Goal-문서-분할-방침` | `not_a_policy_element` | `implementation_finding` | proposed planning structure; P2G-40/50 authority |
| `CPEDR-CWBS-DECISION-STATE-001` | `CAND-CWBS-DECISION-STATE` | `INTAKE-CWBS-001#남은-정책-결정-상태` | `not_a_policy_element` | `implementation_finding` | completeness assertion is validated by P2G-10 exit, not a policy element |
| `CPEDR-CWBS-FOLLOWUP-001` | `CAND-CWBS-FOLLOWUP-PROCESS` | `INTAKE-CWBS-001#후속-진행-원칙` | `not_a_policy_element` | `implementation_finding` | process guidance; pipeline owns procedure |
| `CPEDR-CWBS-OOS-UNITY-001` | `CAND-CWBS-OOS-UNITY` | `INTAKE-CWBS-001#header-excluded-unity` | `out_of_scope` | `out_of_scope` | `SCOPE-CWBS-OUT-001` |
| `CPEDR-CWBS-OOS-ALLY-001` | `CAND-CWBS-OOS-ALLY-SUMMON` | `INTAKE-CWBS-001#header-excluded-allied-summon` | `out_of_scope` | `out_of_scope` | `SCOPE-CWBS-OUT-002` |
| `CPEDR-CWBS-OOS-CONTENT-001` | `CAND-CWBS-OOS-PRODUCTION-CONTENT` | `INTAKE-CWBS-001#header-excluded-production-content` | `out_of_scope` | `out_of_scope` | `SCOPE-CWBS-OUT-003` |
| `CPEDR-CWBS-OOS-SKILL-SUMMON-001` | `CAND-CWBS-OOS-SKILL-EFFECT-SUMMON` | `INTAKE-CWBS-001#BR-01` | `out_of_scope` | `out_of_scope` | source explicitly defers `SkillEffect::Summon`; current scope keeps `BossTask::Summon` only |
| `CPEDR-CWBS-OOS-ADJACENT-001` | `CAND-CWBS-OOS-ADJACENT-SYSTEMS` | `INTAKE-CWBS-001#논의-배경과-목표` | `out_of_scope` | `out_of_scope` | new weapon/tool/other undecided gameplay explicitly excluded from this foundation |

All non-element/out-of-scope records also have `Predecessor Candidate Policy Element Disposition Record Ref = N/A`
and no promoted Element ID/open revision.

## Initial Open Policy Element Revisions

각 promoted candidate는 결정 적용 전 다음 immutable initial revision을 가진다.

| Element family | Initial Policy Element Revision Ref pattern | Predecessor | Decision | Acceptance | Intrinsic status |
| --- | --- | --- | --- | --- | --- |
| `PE-CWBS-{SP-01..02,WV-01..07,SM-01..04,BR-01..05,BL-01..05,PS-01,RF-01}` | `PER-CWBS-{same ID}-OPEN-001` | `N/A` | `N/A` | `N/A` | `open` |

각 open revision의 policy statement는 해당 candidate의 exact source section이 결정할 정책 주제라는
statement이며, 구체 decision/acceptance는 아래 decided successor가 소유한다. Brace/range 표기는 view의
축약이고, 위 promoted candidate 표에 각 exact initial revision Ref가 개별적으로 열거되어 있다.

## Decision Revisions

모든 row는 Initial Decision이며 `Predecessor Decision Revision Ref = N/A`다. Approved decision은 source
digest로 고정된 `INTAKE-CWBS-001`의 해당 section 전체 normative bullet set이다. Exact user approval ref는
모든 row에서 `USR-CWBS-001 + USR-CWBS-004`이고, 이 단계에서 만든 Open Policy Question은 없으므로
`Resolves Open Policy Question Ref Set = empty`다.

| Decision Revision Ref | Exact approved decision source | Affected current decided revision |
| --- | --- | --- |
| `DR-CWBS-SP-01-001` | `INTAKE-CWBS-001#SP-01` | `PER-CWBS-SP-01-DECIDED-002` |
| `DR-CWBS-SP-02-001` | `INTAKE-CWBS-001#SP-02` | `PER-CWBS-SP-02-DECIDED-002` |
| `DR-CWBS-WV-01-001` | `INTAKE-CWBS-001#WV-01` | `PER-CWBS-WV-01-DECIDED-002` |
| `DR-CWBS-WV-02-001` | `INTAKE-CWBS-001#WV-02` | `PER-CWBS-WV-02-DECIDED-002` |
| `DR-CWBS-WV-03-001` | `INTAKE-CWBS-001#WV-03` | `PER-CWBS-WV-03-DECIDED-002` |
| `DR-CWBS-WV-04-001` | `INTAKE-CWBS-001#WV-04` | `PER-CWBS-WV-04-DECIDED-002` |
| `DR-CWBS-WV-05-001` | `INTAKE-CWBS-001#WV-05` | `PER-CWBS-WV-05-DECIDED-002` |
| `DR-CWBS-WV-06-001` | `INTAKE-CWBS-001#WV-06` | `PER-CWBS-WV-06-DECIDED-002` |
| `DR-CWBS-WV-07-001` | `INTAKE-CWBS-001#WV-07` | `PER-CWBS-WV-07-DECIDED-002` |
| `DR-CWBS-SM-01-001` | `INTAKE-CWBS-001#SM-01` | `PER-CWBS-SM-01-DECIDED-002` |
| `DR-CWBS-SM-02-001` | `INTAKE-CWBS-001#SM-02` | `PER-CWBS-SM-02-DECIDED-002` |
| `DR-CWBS-SM-03-001` | `INTAKE-CWBS-001#SM-03` | `PER-CWBS-SM-03-DECIDED-002` |
| `DR-CWBS-SM-04-001` | `INTAKE-CWBS-001#SM-04` | `PER-CWBS-SM-04-DECIDED-002` |
| `DR-CWBS-BR-01-001` | `INTAKE-CWBS-001#BR-01` | `PER-CWBS-BR-01-DECIDED-002` |
| `DR-CWBS-BR-02-001` | `INTAKE-CWBS-001#BR-02` | `PER-CWBS-BR-02-DECIDED-002` |
| `DR-CWBS-BR-03-001` | `INTAKE-CWBS-001#BR-03` | `PER-CWBS-BR-03-DECIDED-002` |
| `DR-CWBS-BR-04-001` | `INTAKE-CWBS-001#BR-04` | `PER-CWBS-BR-04-DECIDED-002` |
| `DR-CWBS-BR-05-001` | `INTAKE-CWBS-001#BR-05` | `PER-CWBS-BR-05-DECIDED-002` |
| `DR-CWBS-BL-01-001` | `INTAKE-CWBS-001#BL-01` | `PER-CWBS-BL-01-DECIDED-002` |
| `DR-CWBS-BL-02-001` | `INTAKE-CWBS-001#BL-02` | `PER-CWBS-BL-02-DECIDED-002` |
| `DR-CWBS-BL-03-001` | `INTAKE-CWBS-001#BL-03` | `PER-CWBS-BL-03-DECIDED-002` |
| `DR-CWBS-BL-04-001` | `INTAKE-CWBS-001#BL-04` | `PER-CWBS-BL-04-DECIDED-002` |
| `DR-CWBS-BL-05-001` | `INTAKE-CWBS-001#BL-05` | `PER-CWBS-BL-05-DECIDED-002` |
| `DR-CWBS-PS-01-001` | `INTAKE-CWBS-001#PS-01` | `PER-CWBS-PS-01-DECIDED-002` |
| `DR-CWBS-RF-01-001` | `INTAKE-CWBS-001#RF-01` | `PER-CWBS-RF-01-DECIDED-002` |

## Current Decided Policy Element Revisions

모든 row의 intrinsic status는 `decided`다. 각 predecessor는 같은 Element ID의 `...-OPEN-001`이고,
Decision Revision Ref는 같은 ID의 `DR-CWBS-...-001`이다.

### Spawn Kernel and Wave

| Element ID / current revision | Policy statement | Observable Policy Acceptance |
| --- | --- | --- |
| `PE-CWBS-SP-01` / `PER-CWBS-SP-01-DECIDED-002` | 적 Wave와 보스 소환은 faction-neutral하게 확장 가능한 공용 Spawn Kernel을 사용하되 첫 consumer는 적 Wave/소환물로 제한한다. 요청은 전원 생성 또는 무생성으로 원자적이며 결정론적 identity·위치·event ordering, stable request identity와 one-shot ledger를 갖는다. | 같은 input/state의 동일 request가 정확히 한 번 같은 구성·위치·순서로 전원 spawn되거나 아무도 spawn되지 않는다. invalid reference는 publication/load에서 거부되고, 이미 참인 trigger는 즉시 정산될 수 있으며, spawn 정산은 terminal Victory보다 먼저 끝난다. Wave와 summon provenance/lifecycle은 구별되고 arbitrary expression/아군 소환은 없다. |
| `PE-CWBS-SP-02` / `PER-CWBS-SP-02-DECIDED-002` | Wave는 authored 고정 spawn cell을 사용하고 기존 점유와 중첩 가능하며, summon은 walkable 후보와 명시 occupancy/insufficient-space policy를 사용한다. | Wave는 인원수만큼 서로 다른 authored cell이 없으면 publication 실패하고 runtime 빈자리 탐색·재시도 없이 고정 cell에 생성된다. Summon batch는 각 구성원이 서로 다른 candidate cell을 쓰며 공간 부족 policy에 따라 전원 생성/전원 실패하고 partial spawn·기존 점유자 축출은 없다. |
| `PE-CWBS-WV-01` / `PER-CWBS-WV-01-DECIDED-002` | 모든 Wave는 `required_for_victory`를 명시하고 required Wave만 승리를 gate한다. Summon은 승리를 막지 않는다. | 누락 default는 publication 실패다. Spawn 전 required Wave도 Victory를 보류하며 primary objective와 모든 required group clear 뒤에만 확정된다. 보류 중 Defeat가 성립하면 Defeat가 우선한다. 시간 생존 mission의 Wave는 모두 non-required이고 전투 종료 시 summon은 정리된다. |
| `PE-CWBS-WV-02` / `PER-CWBS-WV-02-DECIDED-002` | 초기 conditional Wave trigger는 절대 시각, 적 group clear, group clear-or-deadline, actor HP threshold, 실제 사망, point 도달, point 도달-or-deadline의 closed set이다. | 각 authored Wave trigger가 지원 set 중 하나로만 표현되고 조건 성립 시 정확히 한 번 발동한다. 일반 압박 Wave는 absolute time 또는 group-clear-or-deadline을 사용할 수 있고, 순수 group-clear는 의도된 순차 encounter에만 명시된다. |
| `PE-CWBS-WV-03` / `PER-CWBS-WV-03-DECIDED-002` | Wave 시간과 trigger는 absolute battle simulation time 및 deterministic same-bucket ordering을 따른다. | Pause는 시간을 멈추고 speed는 simulation time에 반영된다. 같은 시각 trigger는 priority 내림차순·stable trigger ID 오름차순이며 primary와 deadline 동시 충족은 bucket 정산 뒤 primary를 원인으로 기록한다. 한 피해가 통과한 모든 HP threshold와 그 spawn을 정산한 뒤 terminal outcome을 평가한다. |
| `PE-CWBS-WV-04` / `PER-CWBS-WV-04-DECIDED-002` | Enemy group clear는 성공적으로 spawn된 group의 모든 구성원이 active에서 벗어난 상태다. | 사망과 non-kill despawn 모두 clear에 기여하지만 despawn은 kill 통계·보상·on-kill을 만들지 않는다. 비어 있거나 아직 spawn되지 않은 group은 clear가 아니다. |
| `PE-CWBS-WV-05` / `PER-CWBS-WV-05-DECIDED-002` | Actor trigger는 authored Wave `actor_slot_id`에 bind한 runtime actor의 authoritative HP·실제 사망·실제 point 도달만 관찰한다. | Despawn은 defeated가 아니며 reached-point는 이동을 명령하지 않는다. Generated Wave는 group trigger만, summon batch는 `SummonBatchCleared`만 사용한다. 이미 반경 안이면 즉시 충족하고 제거된 actor의 과거 위치는 사용하지 않는다. 영구 불가능 primary는 deadline 유지 또는 허용된 non-required cancel을 따르며 required point trigger는 deadline이 필수다. |
| `PE-CWBS-WV-06` / `PER-CWBS-WV-06-DECIDED-002` | Wave dependency는 정적으로 spawn 가능하고 cycle/deadlock이 없어야 한다. | 선행 source가 먼저 spawn 불가능하거나 cycle/도달 불가능 required 경로가 있으면 publication이 실패한다. 취소된 Wave만 기다리는 deadline 없는 non-required chain은 stable하게 연쇄 취소되고 required 후속은 유효 deadline 없이 그런 경로를 가질 수 없다. |
| `PE-CWBS-WV-07` / `PER-CWBS-WV-07-DECIDED-002` | Wave activation은 closed typed primary trigger 하나와 선택적 absolute fallback `deadline_ms` 하나로 작성한다. | 둘 중 먼저 충족된 원인으로 정확히 한 번 spawn한다. `AtTimeMs`와 별도 deadline, required의 영구 불가능 무기한 경로, generic AND/OR tree, 무의미/역전/overflow time은 publication에서 거부된다. |

### Summoned-unit domain and Boss Runtime

| Element ID / current revision | Policy statement | Observable Policy Acceptance |
| --- | --- | --- |
| `PE-CWBS-SM-01` / `PER-CWBS-SM-01-DECIDED-002` | 적 summon은 직원·침식체·환상체·보스와 다른 독립 catalog/provenance domain이다. | Summoned unit은 definition/summoner/sequence/batch provenance를 가지며 도감·fragment·독립 reward source가 아니다. 자신의 damage/kill attribution과 일반 통계/on-kill은 정상 적용되고 summoner에게 합산하지 않는다. Summon이 boss classification/encounter authority를 얻거나 재귀 summon하지 않는다. |
| `PE-CWBS-SM-02` / `PER-CWBS-SM-02-DECIDED-002` | Summon은 definition base stats/attack/skill과 encounter enemy scaling을 정확히 한 번 사용한다. | Summoner HP/stat/buff를 상속하지 않고 phase별 강도 차이는 숨은 multiplier가 아니라 별도 summon definition ID로 관찰된다. |
| `PE-CWBS-SM-03` / `PER-CWBS-SM-03-DECIDED-002` | 모든 summon definition은 양수 `max_alive`, `max_total`을 가지며 counter는 summoner instance와 definition 조합별 battle lifetime 동안 유지된다. | 어느 한도든 넘는 request는 부분 생성 없이 `LimitReached`이고 실패 request는 total을 쓰지 않는다. 성공 후 death/despawn은 alive만 해제하고 total은 복구하지 않는다. Phase 전환은 counter를 reset하지 않고 summoner instance별로 독립이다. |
| `PE-CWBS-SM-04` / `PER-CWBS-SM-04-DECIDED-002` | Summon authoring은 default 없는 `Persist | Despawn` summoner-removal lifecycle을 명시한다. | Persist는 commit된 summon만 battle end까지 남고 승리를 막지 않는다. Despawn은 summoner 제거와 함께 non-kill despawn한다. 미해결 request는 양쪽 모두 cancel되며 same-time에는 spawn commit과 summoner removal의 authoritative order가 결과를 결정한다. |
| `PE-CWBS-BR-01` / `PER-CWBS-BR-01-DECIDED-002` | 첫 summon consumer는 `BossTask::Summon`이며 task 하나는 definition 하나와 양수 count 하나를 공용 Kernel에 제출한다. | 결과가 `Spawned | LimitReached | NoSpawnSpace | Canceled`로 분기 가능하고 mixed-definition atomic batch나 `SkillEffect::Summon`은 없다. 예고/windup은 기존 Wait 조합으로 표현된다. |
| `PE-CWBS-BR-02` / `PER-CWBS-BR-02-DECIDED-002` | 성공 request마다 deterministic UUID continuation에서 독립 batch UUID와 정확한 membership을 만들고 symbolic pattern-local handle에 bind한다. | 실패 request는 batch/UUID를 만들지 않는다. 모든 member death/despawn 시 `SummonBatchCleared`가 정확히 한 번 발생하고 정확히 bind된 batch만 관찰한다. 같은 handle 이름은 새 pattern scope에 새 UUID로 bind되며 global/latest lookup이 없고 이 조건은 승리 조건이 아니다. |
| `PE-CWBS-BR-03` / `PER-CWBS-BR-03-DECIDED-002` | Summon placement는 task-authored `AroundSummoner` 또는 `SpawnZone`, 필수 occupancy policy와 필수 insufficient-space policy를 사용한다. | Around 후보는 radius 내 walkable cell을 squared-distance/y/x 순으로, SpawnZone은 validated authored order로 선택한다. `RequireUnoccupied | AllowOccupied` 의미와 logical occupancy가 명시대로 적용된다. 공간 부족은 atomic `Fail` 또는 bounded battle-time `WaitUntilAvailable`이며 same-time 공간 확보가 timeout보다 우선하고 cancel/limit을 재검사한다. Partial/eviction/unbounded retry/implicit placement fallback은 없다. |
| `PE-CWBS-BR-04` / `PER-CWBS-BR-04-DECIDED-002` | Summon AI는 별도 scheduler를 만들지 않고 각 unit이 독립 기존 `BossActorBehaviorRuntime`을 재사용한다. | Existing task/query/movement 조합과 명시 route context로 행동하며 summon에는 BossEncounterRuntime·phase·Wave/terminal authority가 없다. Summon graph의 Summon task는 validation 실패하고 unit 제거 시 actor runtime/pending actor task는 끝나되 released detached delivery는 기존 lifecycle을 따른다. |
| `PE-CWBS-BR-05` / `PER-CWBS-BR-05-DECIDED-002` | Summon batch는 optional battle-time lifetime과 verified signal removal을 가지며 각 removal은 명시 `Despawn | Kill` mode를 사용한다. | Despawn은 death/on-kill/reward를 만들지 않고 Kill은 normal death/on-death를 실행하되 기믹 제거에는 killer credit이 없다. Lifetime/signal 중 먼저 성립한 원인만 적용되고 neutral terminal signal은 latch된다. Removal signal은 bind 이후 발생만 구독하며 exact batch만 제거한다. Both modes free alive but not total, member removal은 batch clear에 반영되고 invalid/unproduced/unbound signals는 publication 실패다. Summon은 WV-05 actor-slot trigger 대상이 되지 않는다. |

### Battle lifecycle, persistence, and reference content

| Element ID / current revision | Policy statement | Observable Policy Acceptance |
| --- | --- | --- |
| `PE-CWBS-BL-01` / `PER-CWBS-BL-01-DECIDED-002` | Active player unit 또는 deployable 직원이 0이라는 사실만으로 battle Defeat를 만들지 않는다. | Authored 보호 대상·시간·특수 terminal condition이 없으면 battle은 계속되며, 전투 후 run-failure lifecycle은 별도로만 평가된다. |
| `PE-CWBS-BL-02` / `PER-CWBS-BL-02-DECIDED-002` | 같은 simulation timestamp에는 bucket 전체와 spawn/death/despawn을 안정 상태까지 정산한 뒤 Defeat를 Victory보다 먼저 평가한다. | Defeat가 하나라도 충족되면 패배다. 같은 verdict 후보는 priority 내림차순·stable outcome ID 오름차순으로 하나를 commit한다. 시스템 최대 전투 시간과 암묵 Draw/Defeat는 없으며 authored terminal 또는 Retreat까지 지속된다. |
| `PE-CWBS-BL-03` / `PER-CWBS-BL-03-DECIDED-002` | Encounter terminal time limit는 absolute battle simulation time이고 system maximum이 아니다. | Pause 중 흐르지 않으며 limit와 success가 같은 bucket이면 BL-02에 따라 time-limit Defeat가 우선한다. |
| `PE-CWBS-BL-04` / `PER-CWBS-BL-04-DECIDED-002` | Neutral signal producer, Encounter의 named terminal outcome, Node의 outcome policy가 사건·verdict·보상/진행 권한을 분리한다. | Typed producer는 verdict를 정하지 않는다. Encounter는 reachable Victory를 최소 하나 가진 stable outcome ID/verdict/closed condition/priority를 선언하고 Battle result는 하나의 outcome ID를 commit한다. Node는 모든 outcome에 policy를 정확히 하나 두며 Victory별 reward/bonus/progression을 명시하고 Defeat는 positive reward/progression 없이 기존 retry/failure 흐름을 따른다. Flat dual reward authorities는 제거되고 outcome-specific result consumption은 atomic/idempotent하며 reconnect가 재선택·재지급하지 않는다. BossOmen capstone Victory는 chain/Exit Gate를 반드시 처리한다. |
| `PE-CWBS-BL-05` / `PER-CWBS-BL-05-DECIDED-002` | 영구 도감은 `discovered/defeated`를 폐기하고 canonical ID별 단일 registered set만 가진다. | 실제 spawn된 opponent Abnormality/CorrodedEmployee만 candidate이고 combat node의 final Victory에서 그 attempt의 모든 candidate를 사망/primary 여부와 무관하게 등록한다. Defeat와 Retreat는 변경하지 않는다. Outcome별 compendium credit은 없고 개발 중 기존 profile은 migration/union 없이 빈 새 schema로 재생성된다. |
| `PE-CWBS-PS-01` / `PER-CWBS-PS-01-DECIDED-002` | Trigger/request ledger, summon counter/batch/handle, signals, pending Victory와 committed outcome ID는 authoritative active-battle state다. | Reconnect/resync 또는 동일 active battle 복원에서 재발동·재추첨·counter reset·outcome 재선택이 없다. 이 정책만으로 active battle을 durable SavePoint payload에 새로 저장하지 않는다. |
| `PE-CWBS-RF-01` / `PER-CWBS-RF-01-DECIDED-002` | 조건부 Wave·summon·signal·named outcome·node policy를 증명할 WhiteNight와 분리된 최소 placeholder FinalBoss reference를 둔다. | Placeholder는 대표 vertical path만 증명하고 나머지는 focused fixtures가 검증한다. 실제 lore/balance/production reward를 확정하지 않으며 normal BossOmen pool/cycle, fragment/repeat reward, all-FinalBoss record와 progression에서 제외된다. Production 승격은 별도 content authoring/balance review를 요구한다. |

## Current Policy Element Registry

위 25개 `PER-CWBS-*-DECIDED-002`가 `WR-CWBS-P2G10-001`의 Current Policy Element Revision Map을
구성한다. Owning Phase는 아직 P2G-40 전이므로 모두 `N/A`다. Open revision은 history로 보존하되 current map에
포함하지 않는다.

## Finding Triage Log

| Finding ID | Affected elements/candidates | Finding | Primary classification | Disposition | Evidence/decision ref |
| --- | --- | --- | --- | --- | --- |
| `FIND-CWBS-POLICY-001` | all 25 promoted elements | Named policy sections contain user-visible/runtime contract decisions | `material_policy` | all decided exactly as captured | `DR-CWBS-*-001`, `USR-CWBS-001`, `USR-CWBS-004` |
| `FIND-CWBS-CANONICAL-DELTA-001` | `WV-*`, `BL-*` | Target policy intentionally changes current no-conditional-wave/terminal/compendium meanings | `material_policy` | decided target; canonical publication is downstream, not performed here | source sections and decision refs |
| `FIND-CWBS-IMPLEMENTATION-001` | `CAND-CWBS-KNOWN-DELTAS`, all implementation boundaries | Source lists current implementation differences but P2G-10 cannot verify current code | `evidence_gap` | preserve as unknown until P2G-20 | `GAP-CWBS-CURRENT-TREE-001` |
| `FIND-CWBS-PLANNING-001` | milestone and Goal-split candidates | M0-M6 and Master/Phase split are proposed planning inputs | `implementation_finding` | do not promote; route to P2G-40 after Freeze | candidate disposition refs |
| `FIND-CWBS-OOS-001` | five OOS candidates | Unity, allied/skill-effect summon, production content and adjacent systems are outside current scope | `out_of_scope` | no scope expansion | `SCOPE-CWBS-OUT-*`, source exclusions |

## Open Policy Question Log

- Current Open Policy Question Ref Set: `empty`
- P2G-10 review found no unresolved material choice inside the approved scope. Schema/type/module spelling and exact test
  commands remain technical/code-review matters and do not reopen policy at this stage.

## User Decision Log

The 25 `DR-CWBS-*-001` rows above are the complete current Decision Revision Ref Set. Each is an initial lineage leaf,
references its one affected current decided Policy Element Revision, uses the acceptance owned by that revision, has exact
approval `USR-CWBS-001 + USR-CWBS-004`, and resolves an empty question set. There are no predecessor/successor pairs in
the current set.

## Technical Decision Log

- `empty` — P2G-10 did not choose Rust type/module names, RON field spelling, transport shapes or test commands.

## Evidence Gap Log

| Gap ID | Required fact | Affected claim | Current disposition | Owner/next action |
| --- | --- | --- | --- | --- |
| `GAP-CWBS-CURRENT-TREE-001` | actual current Core/live RON/server behavior and dirty-tree source identities | implementation delta, boundary and feasibility only; does not alter decided target meaning | `unknown` | P2G-20 source plan and code-informed review |
| `GAP-CWBS-CANONICAL-PUBLICATION-001` | exact canonical documents/sections requiring target publication | downstream publication boundary, not current policy closure | `unknown` | P2G-20 identifies delta; P2G-40 assigns work |

## P2G-10 Exit Validation

1. 25 in-scope policy candidates are promoted; all other identified structural/OOS candidates have one disposition.
2. Every finding has one primary classification and disposition.
3. Every material element has one exact current Decision Revision and user approval basis.
4. Current Open Policy Question Ref Set is empty.
5. All 25 decision lineages have exactly one initial current leaf; no predecessor/successor pair coexists.
6. Out-of-scope findings do not expand the approved scope.
7. Evidence gaps remain `unknown` and are not treated as current/safe/pass.
8. Every current in-scope element is `decided`.
9. Every decided revision has non-empty, decision-consistent Observable Policy Acceptance.
10. Every identified candidate has exactly one effective disposition; only promoted candidates have stable Element/open revision IDs.
11. Every disposition records the exact three-part Evaluation Basis stated above.
12. Raw input did not change, so `PIS-CWBS-INITIAL-001` is preserved and no successor snapshot exists.
13. There is no applicable post-Freeze Change because no Freeze exists.
14. This is not a repair run; no P2G-10 obligation leaf exists.
15. Cross-references among candidate, open/decided revision, decision and current Working Revision selection are complete.

Result: `is_p2g10_complete(WR-CWBS-P2G10-001) = true`.

This result does not perform P2G-20 code review, P2G-30 Freeze, P2G-40 planning, P2G-50 Goal authoring,
P2G-60 handoff, activation or implementation.
