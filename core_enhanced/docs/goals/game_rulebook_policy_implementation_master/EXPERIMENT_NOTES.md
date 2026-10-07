# Game Rulebook Policy Implementation Master — Notes

## Structural Judgment

- Master를 선택했다. Monolith RON tuning, run-local progression/V4 cutover, fixed Standard/permanent profile,
  mastered sweep는 서로 다른 source-of-truth와 migration 수명주기를 가진다.
- Master는 `orchestration_only`이며 production mutation authority를 갖지 않는다.
- 실제 구현은 한 번에 하나의 separately activated Phase Goal이 소유한다.
- Preflight는 completed external entry gate다. 이를 다시 P00 child로 만들거나 재실행하지 않는다.
- Phase Goal directory는 실제 activation 때 필수 문서 3개와 함께 생성한다. 미래 Phase placeholder
  state/schema 또는 빈 child directory를 미리 만들지 않는다.

## Phase Boundary Judgment

- P01은 `GRB-WP-002`만 소유한다. 값/validation 경계가 독립적이고 public DTO/save migration이 없으므로
  첫 vertical slice가 될 수 있다.
- P02는 `GRB-WP-001` 전체를 하나로 유지한다. Research removal, source ledger, cycle, BossOmen, V4 save,
  server mapping을 나누면 어느 중간 상태도 approved target으로 runnable하지 않기 때문이다.
- P03은 Standard run terminal과 permanent profile을 함께 소유한다. F3 completion/tutorial flag가 서로 다른
  수명주기를 교차하므로 crash-recoverable host publication을 같은 Phase에서 닫는다.
- P04는 P02 ledger를 소비하지만 P03 뒤에 직렬화한다. 이는 semantic dependency 추가가 아니라 Preflight가
  확인한 shared public-file/current ownership 충돌을 줄이기 위한 실행 순서다.
- WP005는 별도 future campaign이다. 이번 Master가 완료되어도 29 placeholder fragment skill은 남는다.

## Source-Of-Truth And Cross-Phase Impact

- Target meaning은 rulebook/closed PD가 소유하고 current fact는 activated Phase가 조사한 code/live data/
  producer/store가 소유한다.
- P01은 P02가 사용하는 Monolith policy data를 먼저 안정화한다. P02가 P01의 reward/wave 의미를 다시
  바꾸면 boundary violation이다.
- P02의 V4 encounter/mastery/source authority는 P03 all-FinalBoss record event와 P04 sweep branch의 필수
  input이다.
- P03의 profile은 run-local rollback 밖에 남는다. P04 sweep은 profile을 새로 정의하지 않는다.
- Phase가 canonical을 갱신하면 다음 Phase는 historical Master text보다 current canonical/runtime을 먼저
  확인하고 material drift를 Master에 기록한다.

## Activation And Completion Judgment

- Initial status는 P01 `ready`, P02~P04 `pending`, active Phase 없음이다.
- `ready`는 production mutation 승인이 아니다. 사용자가 별도로 activation하면 child를 만들고 Master
  registry에서 하나만 `active`로 전환한다.
- Completed Preflight baseline Core 1063/server 63/focused 17 green은 runner availability evidence다. 어떤
  Phase의 activation baseline이나 completion evidence도 대신하지 않는다.
- 다음 Phase는 선행 child의 독립 `Complete / High` audit와 Master handoff 기록 없이는 시작하지 않는다.
- Master final audit는 Phase status 합계를 반복하지 않고 combined runtime/data/save/DTO/consumer/canonical을
  다시 검증한다.

## Dirty Worktree Warning

- Repository는 Master 생성 전부터 broad dirty 상태였고 completed Preflight directory도 기존 untracked
  input이다.
- Master는 existing diff를 restore/reset/stash/format하지 않았다.
- P01의 target Monolith fields는 Preflight 당시 user diff와 겹치지 않았지만 activation 때 exact target
  hunk를 다시 확인해야 한다.
- P02~P04는 Core/RON/server에 넓게 겹치므로 각 child가 current ownership과 merge safety를 처음부터
  기록한다. Attribution이 불가능하면 다음 변경을 진행하지 않는다.

## External Systems And Consumers

- Core, live RON과 game_server는 이 campaign의 확인 대상이다.
- 사용자는 Unity 개발 트리의 큰 drift 때문에 Preflight 검사를 제외했다. Master authoring도 Unity source,
  EditMode/PlayMode나 live-client probe를 실행하지 않았다.
- 이 제외는 P01 blocker가 아니며 Unity correctness, ignored field, fallback 또는 version skew를 승인하지
  않는다.
- Public DTO/wire가 바뀌는 P02~P04 activation은 당시 consumer boundary를 명시한다. Unity가 계속 제외되면
  그 Phase는 Core/live-RON/game_server development boundary만 claim하고 Unity 통합은 future risk로 남긴다.

## Applied Policy Clarifications

- WhiteNight는 sole live one-step placeholder FinalBoss chain이다. Multi-chain DAG/cycle/error는 synthetic
  test fixtures로 증명하며 final content를 발명하지 않는다.
- Sweep의 consumable duration은 이미 활성화된 generic modifier participant다. Dedicated sweep item/cost는
  승인되지 않았다.
- 마지막 fragment 분쇄 후 mastered sweep으로 재획득하는 사이클은 의도된 정책이다. 반복 횟수 제한을
  새로 추가하지 않는다.
- F3 Nothing There는 stat만 reduced되고 pattern은 기존 쉬운 damage-meter 용도다. 별도 pattern redesign은
  P03 범위가 아니다.
- Initial Monolith 150/200은 개발 중 조절 가능하지만 live-RON authority와 representability validation을
  유지한다.
- 개발 중 프로젝트이므로 public release gate는 없다. Internal atomicity, persistence와 runner gate는
  그대로 유지한다.

## Alternatives Not Applied

- 네 package를 한 single Goal로 합치지 않았다. Baseline과 save/profile/DTO migration evidence가 섞인다.
- WP001을 연구 제거, cycle, BossOmen, V4와 server mapping으로 나누지 않았다. Partial runnable authority와
  compatibility path를 만들 위험이 있다.
- Master completion audit만 별도 production Phase로 만들지 않았다. Master가 orchestration-only final audit를
  직접 소유하고 material correction은 owning Phase로 돌린다.
- 과거 readiness/Preflight Goal을 새 registry에 맞춰 수정하지 않았다. Completed history는 evidence input으로
  보존한다.
- Unity 정비와 WP005를 campaign에 흡수하지 않았다.

## Omitted Validation At Master Authoring

- Runtime/source/live-data 재조사: 0. Completed Preflight를 authoring input으로 사용했다.
- Test runner/probe: 0. Phase activation 때 current baseline을 새로 실행한다.
- Unity inspection/runner: 0, user-approved Preflight exclusion을 유지했다.
- Production modification: 0.

## Remaining Risks And Follow-Up Candidates

- P02는 가장 큰 atomic cutover다. Broad dirty overlap, V4 graph, no-research migration, BossOmen과 server
  publication을 하나의 runnable Phase로 닫아야 한다.
- P03의 run-terminal/profile cross-store commit은 crash/restart failpoint가 없으면 completion을 주장할 수
  없다.
- P04의 synthetic result가 기존 battle side effect를 우회하면서도 normal reward/XP/duration/checkpoint
  transaction을 재사용해야 한다.
- Public consumer integration, final content/balance, dedicated fragment skills와 unrelated cleanup은 별도
  future Goals 후보다.

## P01 Completion Impact

- P01 completed `Complete / High` on 2026-08-12.
- Live Monolith stat tuning is now 150/200 and all profile/stage target-power/HP/attack/defense compositions are
  checked before publication and at resolution.
- Reward/wave/rounding semantics remain unchanged. P02 consumes this seam and must not revert it while editing the
  shared run-policy boundary.
- P02 is the only ready Phase; no production Phase is active at this transition.

## P02 Completion Impact

- P02 completed `Complete / High` on 2026-08-12.
- Core/live RON/game_server now share one exact Active Run V4 and one run-local encounter progression graph; removed
  research/pending and V3 compatibility must not return in P03.
- Exact 29 source rows and the sole live WhiteNight chain are stable inputs. The direct source reward and terminal
  Floor completion path now use the same post-victory graph.
- P03 is the sole active Phase. It owns fixed Standard plus the successor permanent profile/store; P04 remains
  pending and no sweep placeholder is allowed.

## P03 Completion Impact

- P03 completed `Complete / High` on 2026-08-12.
- Standard is now the exact fixed three-Floor ST1 bootstrap with F3 Nothing There `NormalBoss`, actor-only
  `720/53/36/33`, three approved reward pools and no adaptive/FinalBoss authority.
- One monotonic PlayerProfile V2 owns the four compendium sets, tutorial flag and Endless-only records. Bounded V1
  migration preserves all four sets and defaults new fields false/0; only full profile reset removes them.
- Standard completion uses a durable intent journal and shared `completion_recovery_pending` fail-closed state until
  profile publication, Active Run invalidation and journal cleanup all finish. Startup and typed retry recover it.
- Active Run remains exact V4 with required run-local `record_progress`; permanent profile state remains outside
  checkpoint rollback.
- P04 is the only ready Phase. It must consume P02 mastery/source ledgers and P03's stable save boundary without
  adding profile state, fake battle history or an item/cost.

## P04 Completion Impact

- P04 completed `Complete / High` on 2026-08-12.
- Current Endless mastered Elite NodeConfirm exposes Core-owned direct/sweep resolution. One command stages a
  battle-independent immutable result; a second revalidates current authority and atomically commits the stored plan.
- Existing base rewards, per-living full XP, active duration, source-exclusive reacquire/repeat, assignment and node
  completion share existing transaction roots. Condition, combat history, BossOmen, monolith and profile remain
  unchanged.
- Last-copy dismantle followed by sweep reacquisition preserves fragment progress, source completion and mastery and
  may repeat without a new counter.
- Active Run remains exact V4 and PlayerProfile exact V2. The staged plan is actor-transient and committed effects
  rollback through existing checkpoints.
- No production Phase remains. Master completion now depends only on the independent final cross-Phase audit.

## Final Remaining Scope

- Unity integration is unverified by explicit user direction; this Master claims only Core/live-RON/game_server.
- Current Standard, WhiteNight and fragment entries are the approved minimum bootstrap. Final content, balance,
  tutorial polish and WP005 dedicated fragment skills remain separate future work.
- Broad pre-existing dirty work remains user-owned and is not a campaign cleanup target.

## Master Final Verdict

- All four production Phases and the independent cross-Phase review are `Complete / High`.
- Final current versions are Active Run Core/host V4 and PlayerProfile V2. No V3 compatibility or research/pending
  gameplay authority remains.
- The final Core/live-RON/game_server tree passes 1074/72 broad tests plus all current policy-focused filters.
- No production Phase, correction item or policy question remains inside this Master.
- Future work remains intentionally separate: Unity integration, final content/balance/playtesting, richer live boss
  content and WP005 dedicated fragment skills.
