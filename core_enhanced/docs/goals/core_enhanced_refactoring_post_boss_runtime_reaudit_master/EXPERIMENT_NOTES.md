# Core Enhanced Refactoring Post-Boss Runtime Reaudit Master Notes

Status: complete

## Approved execution input

- User-approved baseline: 현재 dirty working tree
- Approved scope: 기존 최종 51개 전체 재분류, Boss와 교차한 19개 집중 재검증, 신규 Boss 후보 조사
- Execution: read-only audit only
- Production mutation: forbidden
- Historical documents: 기존 refactoring audit와 Boss Runtime Goal을 수정하지 않고 보존
- Implementation handoff: 새 catalog 승인 뒤 별도 Goal
- Goal name: 사용자 제공 이름이 없어 기존 관례에 따라
  `core_enhanced_refactoring_post_boss_runtime_reaudit_master`로 정함

## Why this is a Master

51개 후보는 live loading/publication, persistence, economy, node/VN, combat setup, battle lifecycle, spatial,
effects, targeting/delivery와 server projection이라는 독립 source-of-truth 경계를 가진다. Boss Runtime은
그중 여러 경계를 관통하지만 각 domain authority를 하나로 합치지 않는다. 하나의 investigation Goal로
실행하면 candidate identity, test ownership, external gate와 완료 evidence가 섞이므로 Master와 순차
Phase가 필요하다.

원래 감사의 Phase 1~10 boundary를 재사용하면 source candidate traceability와 비교 가능성을 유지하면서
Boss 변경분을 해당 authority owner에서 검토할 수 있다. P00은 현재 code map을 읽기 전에 세운 이 구조가
여전히 맞는지 확인하는 유일한 refinement gate다.

## Historical inputs, not current proof

- 2026-08-07 refactoring audit: 53 source candidates → 51 final candidates, production mutation 없음
- Original final normalization: 22 correction/test-correction, 19 behavior-preserving refactor, 10 cleanup
- Original priority: 14 policy-free Tier A corrections, 5 policy/compatibility-gated correctness candidates
- 2026-08-09 Boss completion audit: 19 relevant candidates를 5 prerequisite completed, 2 coimplemented,
  4 deferred/conditional, 8 independent로 기록
- Boss completion validation: Core 930, integrations 3+4+25+13+4, server 56 green

위 수치와 verdict는 P00 이후 current source/hash/test 의미가 확인되기 전까지 historical baseline이다.

## Known reconciliation question

원래 final catalog의 `CRA-P09-C05`는 continuous projectile motion kernel을
`behavior_preserving_refactor`, Tier B priority refactor로 분류한다. Boss completion audit은
`CRA-P07-C03/C04`, `CRA-P09-C05`를 함께 언급하면서 “first three ... cleanup candidates”라고 요약했다.
현재 work type, 원래 target authority와 Boss implementation의 실제 영향을 Phase 9와 Final에서 다시
판정한다. 문구만 고치거나 어느 쪽을 자동 authority로 선택하지 않는다.

## Source-of-truth boundary

- Runtime 사실: current `core_enhanced` source와 authoritative mutation
- Authored 사실: runtime이 실제 소비하는 current production live data
- Durable contract: current `README.md`가 지정하는 가장 가까운 canonical
- Consumer evidence: actual producer types, server mapping, accessible external source/canonical
- Method: current `engineering_workflow.md`, `goal_authoring_and_execution_guide.md`,
  `goal_completion_review_guide.md`
- Historical support only: 기존 Goal, catalog, experiment와 completion audit

## Initial repository and dirty state

- Repository root: `/mnt/f/work/simulator`
- Start commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- Baseline date: 2026-08-09, current dirty working tree
- New Master path did not exist before initialization.
- Current status includes modified Core canonical/source/test, modified live data and server files, untracked original
  audit/Boss campaign documents, untracked Boss/field-work/tactical source and an unrelated server log.
- Start commit alone cannot reproduce the baseline because much of the relevant implementation is dirty or untracked.
- Phase 0 must create a durable exact status/category/content baseline before any boundary investigation.
- All pre-existing changes are user-owned. This Master does not claim, revert, move, delete, format or clean them.
- `game_server/logs/app.log.2026-08-09` is unrelated and explicitly preserved.

## Boss-specific audit seams

P00 and the relevant boundary Phases must distinguish at least these responsibilities without assuming they are
problems:

- embedded Boss source inventory and atomic live candidate publication
- raw RON parse/validation versus immutable compiled definitions and typed setup identity
- definition-time graph/state versus battle-session runtime instance/epoch/task state
- scheduler wake/event ordering, Pause/time, terminal outcome and rollback-safe advance
- target lock, path query, movement recovery and ordinary movement authority
- effect, damage, death, projectile and skill systems retained as leaf authorities
- ordinary encounter opt-out parity versus Boss-specific fallback or parallel path
- private/non-serde runtime state versus public event/checkpoint/server/Unity contract
- current tests that protect behavior versus tests coupled only to private graph/helper placement

New candidate creation requires actual evidence at these seams. The existence of a new module, enum or abstraction is
not sufficient.

## External systems and consumers

- `game_resources` is supporting production-data evidence only; its independent refactor backlog is out of scope.
- `game_server` is read-only supporting evidence for actual public producer/mapping/consumer boundaries.
- External Unity source/canonical is read-only and is inspected only when a current public/wire assertion requires it.
- Inaccessible or unknown public Rust, Unity, standalone Core or deployed consumer remains `unverified`; it is not
  silently treated as absent.
- No external repository or consumer is modified by this campaign.

## Test audit policy

- Historical green counts show that a suite ran at a past final tree; they do not prove current candidate relevance.
- Each assigned family is read as production evidence: setup/action/assertion, exact invariant, reachability, failure
  observation, stronger duplicate and private-shape coupling are recorded.
- `retain`, `strengthen`, `merge`, `remove`, `unverified` are disposition labels, not implementation actions.
- Happy-path tests remain when they uniquely protect a valid success contract.
- Removal requires assertion transfer or a stronger retained/replacement test.
- Narrow runners are used only for a stated evidence gap; broad reruns are not a substitute for source analysis.

## Alternatives not applied

### Reaudit only the 19 Boss-linked candidates

Not applied. Boss implementation touched shared battle/data/setup files and can change dependencies or authority for
the other 32 candidates even without naming their IDs. The remaining 32 receive impact verification, while the 19
receive deeper full-scope review.

### Overwrite the 2026-08-07 final catalog

Not applied. It accurately records its historical baseline and investigation outcome. Rewriting it would destroy the
before/after trace needed to decide what Boss Runtime changed.

### Put all 51 candidates in one investigation Phase

Not applied. Independent authority, test and consumer boundaries would be mixed, and merged candidates could be
accepted from shared-file shape rather than root-cause evidence.

### Implement obvious cleanup while auditing

Not applied. The approved execution mode is read-only and readiness is not authorization. Candidate classification
or external inventory may change before implementation.

### Rerun the entire workspace suite before source inspection

Not applied. The past suite is a baseline, but green count cannot identify authority drift or test meaning. Phase 0
first determines which current evidence must be rerun.

## Open policy decisions

None are required to initialize or perform policy-neutral read-only investigation. If current evidence exposes a
gameplay, DTO, live RON, save/checkpoint, compatibility, balance or consumer rollout choice, the affected conclusion
is marked `policy_decision_required` or gated and the choice is reported to the user. The audit does not infer it.

## Remaining risks

- Large dirty/untracked state makes commit history and start commit insufficient to identify Boss-specific changes.
- Historical catalog evidence may refer to paths or assertions changed by later Boss/tactical work.
- Boss implementation may have closed a candidate only for Boss setup while ordinary or external consumers still use
  the old authority.
- Private Boss state may look disconnected to serialized searches even when runtime lifecycle depends on it.
- External Unity or public consumers may be unavailable or on a different deployment state.
- Full 51-candidate coverage can become document-heavy; Phase artifacts must link to detailed source evidence without
  copying entire historical catalogs.

## Next safe work

1. Review the rebaselined 51-row catalog and 44-open-ID wave/gate plan.
2. Choose one candidate or compatible small wave and authorize a separate `implement_and_verify` Goal.
3. Resolve named policy/external gates before gated corrections, public cleanup or wire/save migration.

## Final outcome

- P00~P10, Final and Master are all `Complete / High`.
- Exact disposition: 7 `fully_implemented`, 44 `still_valid`; no new Boss candidate.
- Boss whole-scope reconciliation: 7 closed, 12 still valid; all other 32 still valid.
- Open work types: 17 corrections, 17 behavior-preserving refactors, 10 cleanup rows.
- Required Final catalog/test/wave/gate/cross-boundary artifacts are complete.
- Production, current runtime canonical and historical Goal content remain unchanged by this campaign. One concurrent
  future-content skill-design note edit occurred after Final; it was impact-reviewed, preserved and does not change
  the current catalog.
