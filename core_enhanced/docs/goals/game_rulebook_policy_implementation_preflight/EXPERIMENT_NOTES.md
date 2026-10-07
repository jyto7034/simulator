# Game Rulebook Policy Implementation Preflight — Notes

## Structural Judgment

- Single Goal을 선택했다. 구현 package 자체는 여러 독립 경계지만 Preflight 산출물은 production을
  변경하지 않는 하나의 execution-input dossier이므로 Master/Phase가 필요하지 않다.
- Merged `policy_implementation_readiness_review_guide.md` lifecycle에서 이 Goal의 owner stage는
  `ATI-05 — Execution Preflight`다. Prior ATI-01~04 artifacts는 entry pointer이고 current runtime proof는
  PF-01 drift reconciliation이 소유한다.
- 이 Goal은 readiness audit를 재수행하지 않는다. Audit 이후 canonical/code/data/runner delta와 구현 시작에
  필요한 test contract만 재확인한다.
- 실제 implementation Goal 생성 또는 activation은 Preflight 완료 뒤 사용자의 별도 요청이 소유한다.

## Source-Of-Truth Boundary

- Gameplay 의미는 `docs/game_rulebook.md`와 `GRB-PD-001`~`010`이 소유한다.
- Readiness artifacts는 current boundary/package order/bootstrap candidate evidence를 소유한다.
- Preflight artifacts는 acceptance matrix와 실행 evidence를 소유할 뿐 runtime type/schema의 durable
  authority가 아니다.
- Current implementation과 approved target의 불일치는 구현 대상이며, Preflight가 어느 쪽을 조용히
  덮어쓰거나 current behavior를 target으로 재정의하지 않는다.
- Bootstrap manifest의 마지막 summary에는 PD005 Option B라는 stale 문구가 남아 있지만 같은 문서의
  detailed resolution, decision log와 rulebook은 one-step WhiteNight `A-bootstrap`으로 일치한다. 이는 새
  policy decision이 아니라 `PF-01` documentation drift disposition 대상이다. 사용자의 지시에 따라
  completed historical Goal은 수정하지 않는다.

## Activation And Rewind Judgment

- Goal exit에는 `Complete / High`, lifecycle `ready_to_activate`와 package handoff `ready_for_wp002`가 모두
  필요하다.
- 위 verdict는 implementation Goal 생성 또는 production mutation을 자동 승인하지 않는다.
- Policy drift는 ATI-02, authority/save/consumer drift는 ATI-03, dependency/atomic boundary drift는 ATI-04,
  current code/data/runner drift는 이 ATI-05로 돌린다.
- Rewind 필요 시 과거 Goal을 현재 사실에 맞게 개작하지 않고 이 Goal에 새 evidence와 activation denial을
  기록한다.

## Dirty Worktree Warning

Repository는 Goal 생성 전에 이미 매우 넓게 수정돼 있었다. 다음 범주가 포함된다.

- `core_enhanced` Core/runtime/test와 canonical 문서의 다수 tracked modification/deletion
- `game_resources` live RON modification
- `game_server` protocol/runtime/store modification
- 다수의 untracked Goal directory와 새 runtime/data file

이 변경은 사용자 소유다. Preflight는 전체 dirty 상태를 정리하거나 되돌리지 않는다. `PF-01`은
WP002/001/003/004 영향 경로의 diff overlap만 분류하고, 안전한 attribution이 불가능하면 해당 package를
`not ready`로 표시한다.

## External Systems And Consumers

- game_server는 같은 workspace에서 확인할 대상이다.
- Readiness audit에서 adjacent Unity consumer가 확인됐지만, 사용자는 현재 Unity 개발 버전의 drift와
  별도 정비 범위가 크므로 이번 Preflight에서는 Unity를 검사하지 않도록 명시했다.
- Unity current tree, EditMode/PlayMode와 live client probe는 `explicitly_excluded_by_user`다. 이번
  Preflight는 Unity consumer 일치나 test 통과를 주장하지 않으며, 해당 미검증은 `ready_for_wp002`
  판정의 blocker로 사용하지 않는다.
- 실제 production package에서 Unity를 같은 atomic cutover에 포함할지 별도 Goal로 분리할지는 그
  package activation 입력에서 다시 확정한다. 이번 제외는 ignored field, fallback 또는 version skew를
  승인하는 근거가 아니다.

## Alternatives Not Applied

- 곧바로 `WP002` production 값을 수정하지 않았다. Goal 문서 작성 요청은 production mutation 승인이
  아니다.
- 모든 implementation package를 하나의 Master로 생성하지 않았다. 사용자는 Preflight 문서만 요청했고
  구현 Goal activation은 아직 승인하지 않았다.
- Final content roster, WhiteNight detailed chain, tutorial polish나 29 fragment skill design을 Preflight
  입력으로 발명하지 않는다.
- Development-only라는 일반 사유만으로 consumer correctness를 없앤 것은 아니다. 이번에는 사용자의
  명시적 지시로 Unity current-tree 검증만 제외했고 Core/live RON/game_server baseline은 유지한다.

## Omitted Validation At Initialization

- No Core/server/Unity test runner was invoked.
- No production RON loader or reference validator was invoked.
- No runtime/code/live-data/canonical diff was yet inspected beyond initial Git status.
- These are planned Preflight work units, not initialization completion evidence.

## Initial Risks

- Broad dirty overlap may invalidate historical readiness anchors or make baseline attribution difficult.
- Canonical documents and Goal guides themselves are dirty, so current content must be treated as the active user
  workspace state and compared carefully rather than assumed byte-identical to the readiness audit baseline.
- The largest risk remains `WP001`: research removal, acquisition replacement, cycles, save V4 and cross-layer
  result consumers form one atomic cutover.
- Sweep reacquisition intentionally permits a dust-producing dismantle cycle; tests must prove branch exclusivity
  and progress preservation without treating the economy effect as a bug.
- Preflight에서 실행한 baseline은 implementation activation 시점의 변경 전 baseline을 대체하지 않는다.
  Dirty tree와 dependency가 계속 변할 수 있으므로 각 package Goal이 current baseline을 재실행해야 한다.

## Out-Of-Scope Follow-Up Candidates

- Final content authoring and balance pass after runtime foundation
- Dedicated fragment active-skill campaign (`WP005`)
- Any unrelated dirty-tree cleanup or refactor discovered during Preflight

## Executed Preflight Judgments

- ATI-01~04 entry evidence remains materially valid. Current research/Standard/BossOmen/save differences are the
  approved target gap; they do not change policy, ownership or dependency topology.
- WP002 shares two dirty files with user work, but the current diff does not touch its exact Monolith fields or
  validation block. This is sufficient for a `ready_to_activate` handoff with a mandatory activation-time hunk
  recheck, not permission to replace either file wholesale.
- Monolith `safe-range` means every derived percentage is exactly representable without saturation. No arbitrary
  gameplay maximum was inferred from the user's initial 150/200 tuning decision.
- The one-step WhiteNight bootstrap uses no required fragment. No named fragment condition was approved, the global
  any-awakened gate is explicitly removed, and a reachable live fixture is required. Multi-chain DAG/cycle/error
  coverage belongs in synthetic test-only data, not new production boss content.
- The readiness phrase “sweep item” does not authorize a dedicated consumable or cost. The rulebook requires normal
  active-consumable node-duration decrement after sweep; existing `NextCombatNode`/`CombatNodes(n)` fixtures cover
  this generic participant. A special item would reopen content policy.
- Standard F3 completion and tutorial flag need a crash-recoverable host publication boundary. Ordinary permanent
  records remain monotonic and outside active-run rollback. WP003 may select the mechanism but cannot expose only
  one half of Standard completion.
- All 29 encounter/abnormality/fragment ID triples exist. Three tutorial reward pools are intentionally new exact
  WP003 authoring, and target source/repeat fields are exact WP001 authoring; neither is a missing-reference blocker.
- Unity exclusion applies only to this Preflight evidence. Any later package that changes a public contract must
  explicitly decide and verify its consumer scope; no compatibility fallback was approved.

## Package Handoff Judgment

- `GRB-WP-002`: `ready_to_activate`.
- `GRB-WP-001`: `ready_after_wp002_completion_audit`.
- `GRB-WP-003`: `ready_after_wp001_completion_audit`.
- `GRB-WP-004`: `ready_after_wp001`, serialized after WP003 under current file ownership.
- `GRB-WP-005`: `future_or_dormant`.

These verdicts do not create a Goal or authorize production mutation. The user must separately activate the next
implementation Goal.
