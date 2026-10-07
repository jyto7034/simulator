# U05 실행·검증

P2G40/50 표준 구조/quality2 PASS, 의미 검토와 P2G60 인계 완료.
GEX00 actual dirty before archive와 실제 추출 hash 확인. Product mutation 없음.
초기 full baseline은 exact U04 closed evidence 재사용; U05 acceptance 실행은 아직 없음.

## 2026-10-04 GEX-10 definition / first runtime pass

Detailed mutation readiness is recorded in PLAN before the first product edit. Initial changes add required
fragment-form max and explicit skill use descriptor, strict employee profile serde, base20% then existing
consumable composition, Rest HP-only schema, public form max projection and current source/fixture migration.
Historical evidence directories were not traversed or rewritten. This is an incomplete atomic WP sequence.

- `../../evidence/definition_check_001.json`: `cargo check -p game_core --all-targets --features test-fixtures`,
  exit0, source unchanged during execution; all-target schema compilation. Existing non-feature warnings are
  preserved in raw log; this is not the final warning-clean clippy configuration.
- `../../evidence/definition_ron_001.json`: `cargo test -p game_core --test ron_loading`, exit0, 25 passed,
  source unchanged during execution. Official catalog actual parser/loader passes at this intermediate source.
  Neither receipt proves pending runtime, typed sustained fixture migration, persistence-version changes or
  final U05 lifecycle acceptance. Subsequent runtime edits invalidate their use as final-source evidence.
- Runtime first pass now connects typed use credential, input consumption, 1000ms battle-second pulse,
  no employee basic-attack gains, separate remaining flow control / effect ownership, cancellation and targets.
  `runtime_check_001` is the first build attempt; its actual receipt/log owns the outcome.

Runtime attempts (all command receipts record unchanged source during the attempt):
- runtime_check_001 exit101: two complete RuntimeFactSource initializers omitted the new private use_control flag;
  fixed in commands/queue, corrosion now-unused imports removed.
- runtime_check_002 exit0: all Core targets compile with test-fixtures.
- runtime_focus_001 exit101 before tests: new test helper used u32 instead of existing u64 ApplyBuff duration.
- runtime_focus_002,003 each exit101: 5/6 pass; new self-shield fixture first used self_unit instead of public self,
  then retained invalid global self reach. Fixed the fixture to self + pattern; runtime assertions were unchanged.
- runtime_focus_004 exit0: 8/8 public-RON-derived native command cases pass: admission consume/cost once,
  duplicate rollback, zero-start, delayed action lock, slow projectile final EffectsSettled independence,
  applied timed shield vs HardCC overlap/missed pulses, Silence after admission, Stun/Freeze cancellation,
  employee gain removal/opponent retained gains, empty tile/wrong-kind failure, three extra attacks/CC no resume,
  direct target withdrawal immediate cancel/no refund. This remains focused intermediate evidence.
- core_lib_interim_001 started as the first broad unit regression; actual log/receipt owns outcome.

- core_lib_interim_001 exit101: 1648 passed /37 failed /0 ignored,106.70s test runtime. Full raw failure output is
  preserved. Groups: missing use_lifetime on inline/reinput fixtures; old recharge-duration assumptions in public
  trial helpers; intended Rest/initial-charge/empty-tile/control cancellation expectation changes; original V7
  employee resonance migration still missing; and an actual SelfUnit+CastTarget step gate regression.
- Actual defect found: admitted_skill_target_is_valid treated a SelfUnit selection whose leaf uses CastTarget as
  an enemy requirement. It canceled legitimate self actions/corrosion at start. Fixed with explicit self identity
  and active lifecycle validation; world_corrosion_001 is the focused existing public-world recheck.

- world_corrosion_001 exit0: existing r09_world_pause_start_host_speed_terminal_employee_cost_and_save
  passes after the self-target correction; source unchanged. The remaining36 initial broad failures have not yet
  been individually corrected/revalidated; some share this fixed self-target cause. No broader pass is inferred.

- checkpoint_transition_001 exit101 at compile: new conversion helper omitted its closing brace. Corrected.
- checkpoint_transition_002 exit101:22/23 pass; a multiline version assertion still expected10. Corrected to11.
- checkpoint_transition_003 exit0:23/23 checkpoint validation tests pass, including immutable original V7 hash,
  explicit V7/V8 conversion, foreign identity/checksum/graph rejection, fresh-process restore, valid-checksum
  removed employee field rejection, previous/unsupported envelope versions, and custom legacy policy rejection.
  Source unchanged; raw log and exact source receipt are in evidence/checkpoint_transition_003.*.
- host_checkpoint_transition_001 running: server active-run store tests include current/backup protection for
  checksummed CoreV10 originals against publish and invalidate. Outcome must be read from the receipt.

- host_checkpoint_transition_001 exit0:13/13 real-file host store tests pass; CoreV10 in either current or backup prevents publish/invalidate and preserves exact originals. Source unchanged.
- core_lib_interim_002 started after regression fixture changes: persistent employee charge via existing InitialSkillCharge100%; public amount trial selected form max25 (base5 +20 whole-second pulses reaches full before23s); empty tile admission; unreleased delayed action cancellation; accepted corrosion credential/stale event. Actual outcome pending.

- core_lib_interim_002 exit101:1680/1685 pass,5 fail,111.21s. Remaining failures: duplicate Hello now
  returns action_locked; old shield-overlap recast/attack charge expectations; two remaining Rest bonus
  assertions; and an actual regression where new target validation intercepted nonemployee pending death
  before its original cast-end cost/failure. Restricting the target-cancel branch to employee use preserves
  autonomous/nonemployee ownership; the existing unchanged native death test is the regression oracle.

- descriptor_regression_001 exit0:103/103 boss/runtime/setup cases pass under required finite-effect descriptors.
- runtime_focus_005 exit101:10/11 pass; new Sequence/Parallel test reused one authored step under two producers,
  which the existing public Flow compiler forbids. Replaced the second producer with its own hit2 step.
- runtime_focus_006 exit101 compile: new tests used ron::from_str instead of this repository's ron::de::from_str.
- runtime_focus_007 exit0:13/13 U05 focused tests, adding strict removed employee/Rest authoring,
  max1/4/5/99/100 +bonus40 via actual granted/equipped/awakened/selected copy, Flow branches/repeats,
  employee death/withdraw pending/pulse cleanup. Source unchanged.
- runtime_focus_008 exit101 compile: new actual-work fixture omitted required FieldWorkCellDef.id. Fixed.
- runtime_focus_009 exit101:15/17 pass; employee deployment fixture lacked equipped weapon; overflow ExtraAttack
  test had attack0 and was correctly refused as NoValidTarget. Fixed test attack1 and began real equipment setup.
- runtime_focus_010 exit101:16/17 pass; field_work_core's minimal live_defaults builder contains no weapon.
  Added test helper data injection and used full validated official content for the employee field-work scenario.
- runtime_focus_011 exit101:16/17 pass; full live loader exposed one actual unmigrated manual fragment
  RetargetOnStep (fragment_quiet_lament). Changed only that fragment step to explicit ReuseCastTarget;
  the three opponent autonomous RetargetOnStep entries remain. runtime_focus_012 is running.
- New focused assertions already passing at010/011: positive command gate/negative change/cap pulse discard;
  typed pulse-deadline and extraattack counter overflow rollback of full state/queue/execution with retry;
  native clone admission failure and controlled clone preservation of actual accepted credential/readiness.

- runtime_focus_012 exit101:16/17 pass; official awakened butterfly form also referenced an autonomous retargeting skill. Added an explicit employee awakened skill with fixed accepted target and migrated that form/catalog manifest; preserved the abnormality original. runtime_focus_013 rechecks the full official loader and actual work lifecycle.

- runtime_focus_013 exit0:17/17; actual official-data work/Silence/relocation/stance/withdraw/redeploy pulses pass.
- host_use_reconnect_001 exit101 compile: four new test API references corrected (FacingDirection module, infallible builder, read_model, nested deployment).
- core_lib_interim_003 exit101:1693/1694 pass,108.61s. The remaining passive-guard negative fixture replaced Parry with Heal but left its finite-effect descriptor. Changed that variant to Action so it reaches and still rejects at the intended fragment guard check. All previous runtime failures pass.

- host_use_reconnect_002 exit101 compile: JSON command macro needs concrete serializable values, not unconstrained .into()/None. Corrected string/Null.
- runtime_focus_014 exit101:17/19 pass; two new fixtures violated existing SelfUnit-overlay producer and rotation-invariant footprint contracts. Corrected fixtures to SelfUnit and square footprint.
- runtime_focus_015 exit0:20/20. Adds actual independent DoT/Control/permanent/area, all selected finite effect families and mixed two finite leases with independent area still unsettled after use ends.
- host_use_reconnect_003 exit101 at paused-time assertion: the test called explicit simulation advance, which deliberately advances even paused. Switched probe to the actual host-tick port used by PlayerGameActor (playback-aware); this is test wiring, not a runtime semantic change.

- host_use_reconnect_004 exit101: paused time now stays1000 correctly; aggregate resource_spent includes initial deployment10, so changed the assertion to the actual before/after delta1 plus unchanged subsequent totals.
- runtime_focus_016 exit101:20/21 pass; selected finite projectile fixture had a hostile negative modifier, correctly forbidden as a beneficial use lease. Uses explicit positive modifier plus hostile damage in the same legal public projectile so admission remains useful and the selected lease has the intended category.

- host_use_reconnect_005 exit0: same actor paused admission retains consumed1, duplicate rollback, detach/reattach exact readiness, resume/cast execution with no second cost. Actual host tick port and public commands used.
- runtime_focus_017 exit101:20/21 pass; new fixture text insertion accidentally matched the descriptor's effects list too. Restricted it to the operation entry.
- Live RON manifest cutover is version5 (required form capacity/use descriptor and removed employee/Rest authoring); previous version4 rejects at exact schema_version path. Current continuation remains11 and explicit legacy checkpoint conversion12. Formatting precedes final integrated validation.

- finite_projectile_018 exit101 only final value8 vs7: actual collision contact arrives before tile-center distance, so its lease already ended before the2000 pulse. Changed the oracle to the actual published expiry, checking immediately before/at expiry and exactly the next whole-second pulse. Earlier launch/delivery/active-lease gates pass.
- official_catalog_002 exit101: live catalog audit4/4 and ron_loading24/25 pass. The exact cast-selection catalog expectation still listed25 skills. Added the explicit employee awakened clone/current_target and AutomaticUnit count20 (total26); no selector expectation removed.


### Integrated001 and public cutover probes

- integrated_fmt001 and integrated_core_default001 passed on the unchanged first integrated basis; Core default all-targets took697.66s. integrated_release001 built observed stock Core/server bins successfully in339.95s.
- public_inputs001:13/15 accepted; the two R07 unnamed serialized skill tuples missed use_lifetime. input_revision_003 archives both failed bytes and explicit Action additions. public_inputs002 accepts15/15 with originals unchanged. Seven manual trial prototypes use max25/base5 with their unchanged23s setup; official form capacity stays100.
- public_schema001/002 each passed10/11; persistent finite-producer negative fixture hit earlier shared-shield and HealthCost Instant-only diagnostics. Corrected only this isolated fixture to personal shields/modifiers/Heal so the intended lifetime boundary is reached. public_schema003 passes all11 actual release-reader checks, including manual AutomaticUnit retarget rejection and the independent persistent finite-producer rejection.
- public_continuation001 passed78 messages/7 actual process starts; public_rewind001 passed114 messages/7starts, retaining stale-claim refusal, no mutation on refusal and independent rewind behavior under version11.
- compat_prepared_public_001 and compat_prepared_boundary_001 completed successfully before orchestration was stopped. They remain bound to the pre-correction source.
- independent_attack_area001/002 are new real public RON/CLI executions, not completion passes:002 samples101ms after admission and proves an independent area retains acting/skill_use through its generated ExtraAttacks. INDEPENDENT_AREA_COUNTEREXAMPLE.json records expectedNone vs actualuse and exact source/log.
- Stopped remaining integrated/compat orchestration for this real defect. integrated_core_fixtures001 was deliberately interrupted with SIGINT(raw -2, source unchanged); no tests from it count as a successful suite. INTEGRATED_001_INTERRUPTION.json records scope/reason. All child runners closed before product correction.
- runtime_focus019 now checks corrected source and the expanded24 U05 filters. Independent area attack provenance is excluded from creator use while normal authored combo attacks remain bound through last release. New tests cover actual first/later area attacks, combo projectile last-launch unlock and three native scan fuel boundaries.

Final-source public rebind after independent-area correction: public_inputs003, public_schema004, public_independent_area003, public_continuation002. All returned exit0 with unchanged source. The same frozen101ms independent-area counterexample now projects no use or SkillUse block; schema reader positive+10 negative cases pass; stock Continue covers78 messages and7 process starts. Full integrated and remaining consumers are still running; no GEX-10 completion or audit claim.

Integrated current002 Core default completed1758 passed/0failed, including1701 lib tests; exact receipt records724.29s and unchanged product source. public_rewind002 passed114 messages/7 starts; compat functional prepared/boundary/defaults/oracle/trace/CLI/profile/boundary all exit0 so far. Fixture/observation/host/check/heap and API/fault/performance remain open.

compat_public_api_002 completed21 commands,13CLI cases and6 negative external compile consumers, exit0/source unchanged. All ten functional compatibility procedures now pass; actual five-mode fault/stock WebSocket and full fixture/observation/host/performance remain in progress or pending.

public_functional_matrix completed successfully: final_fault_001 has exact five-mode native results, selected observer cleanup, queue saturation/drop observation and99-message stock WebSocket recovery. Both observed and unobserved example builds are current-source receipts. No performance measurement ran concurrently with full validation. PC isolation preparation matches current source; compile-only prebuilds are in progress to avoid later compilation delays.

All three integrated Core source002 configurations completed: default1758, test-fixtures1758, runtime-observation1767;0failures and unchanged product source in each receipt. Host default/observation and final check/clippy/heap continue. Compile-only PC and both performance guard prebuilds also passed; no measurements yet.

public_guards001 and allocator001 pass with unchanged original caps: ordinary snapshot p95 221310ns /500000,49375bytes/65536; export18376/32768. PC compat_pc_002 executed576 rows successfully but original compare_pc_v32 failed96 after-log hashes (six literal-battle conditions ×16 samples), while status/error-code/HP/facts/queue/time/log-count oracles match. The unchanged private draft helper constructs TestFixture from employee_default, whose approved base changed0/100/1000→20/100/0. The captured U05 SOURCE_BEFORE proves that implicit old input. Explicit pinning of only the isolated TestFixture helper restores original0/100/1000 without changing product Employee behavior or weakening log oracles. compat_pc_prepare_003 creates new isolate002 with that hook; compat_pc_003 is in progress. Original isolate001/raw002/failed comparison remain preserved.

R05 compat_r05_002 raw101 and summarize_002 pass: only the previously approved concurrent peak exception applies (9497009 against9030188, excess466821). Other hard guards and deterministic log parity pass. Event-heavy ops44194/70671, retained263238/276174, peak267527/276174. Concurrent ops370928/389240 and retained4196928/4196928. Allocation-instrumented latency is recorded but not a latency gate; no cap or exception was enlarged.

PC003: 원래 비직원 fixture의 공명0/100/1000을 명시한 isolate에서576행 측정(137.67초, compile 포함541.09초), source 불변. 원래 compare_pc_v32 36조건(31성공/5typed refusal)와 전투 로그·HP·facts·queue 통과. strict unchanged-cost compat_pc_immediate_002는 예상대로 exit1이며 원본 보존. compat_pc_immediate_003은 명시한 상태64bytes/unit 및 source traversal 비용 증가를 모든 행에서 대조한 뒤 시간 외 모든 필드를 완전 비교해 통과. 실제 비용 동일을 주장하지 않는다. PC_RECONCILIATION.json에 원본·입력 교정·정량 비용·hash를 결속했다. 기존 R05/public 상한과 예외는 바꾸지 않았다.

최종 자체 검증: GEX_10_RESULT.json. 기본/fixture/관측 Core·host와 전체 검사 및16 public 소비자, 실제 다섯 fault 관측 모드·stock WebSocket 영속/재접속·선택권 동시성·SavePoint 재전투를 현재 source에서 검증했다. R05 raw exit 101 / warning_concurrent_peak_only는 승인된 해석과 원자료를 분리 보존한다. 새 독립 완료 판정은 포함하지 않는다.
