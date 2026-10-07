# Test and Risk Traceability

- 기준선: `B-001`
- 최초 broad 실행: E-002, `cargo test --all-targets --all-features`
- 결과: 10 binaries, 1,131 passed, 0 failed, 0 ignored
- 원칙: test count 자체가 아니라 각 구조 위험에 대한 증명/공백을 연결한다.

| risk/contract | executed evidence | what it proves | what it does not prove |
| --- | --- | --- | --- |
| action route/scope/authority | `every_action_kind_has_one_characterized_route_scope_and_authority_contract` | every current action has one expected route/scope/authority tuple | handler transaction correctness |
| result transport completeness | 47-case `every_behavior_result_has_exact_old_command_projection_contract` | exhaustive outer result type/payload/battle-side routing | domain/wire type separation or protocol versioning (RUNTIME-AUDIT-008) |
| shop transaction | sale/purchase/refresh behavior tests + E-007 static staging | checked eligibility and representative success/error behavior | property coverage for every integer/data combination |
| battle advance/tactical atomicity | future-cursor mutation/finish advance and five staged tactical command tests | safe families preserve staged authority on cursor/projection errors | six playback/target/activate commands; GAP-001/RUNTIME-AUDIT-005 |
| StartNewGame invalid trigger | `missing_run_start_visual_novel_trigger_fails_start_new_game` | missing trigger returns typed `InvalidStaticData` | full before/after equality; GAP-002/RUNTIME-AUDIT-006 |
| deterministic map generation | same seed/state full generated-value equality | required A/B probe for IDs/assignment/output | separate-process/platform stability |
| event ordering | all 720 relevant heap insertion permutations | exact finite-event drain order independent of insertion | non-finite Eq/Ord law; RUNTIME-AUDIT-004 |
| Rapier/movement order | reversed unit insertion, exact outputs/positions/event-log JSON | same-process movement observable stability | cross-platform Rapier result identity |
| RNG/log identity | observation-only log insertion tests | gameplay RNG/projectile/proc identities are not based on log seq | exhaustive future RNG caller discipline |
| official live data | live loader/schema/path/unknown/malformed/parity/integration tests | current embedded bundle parses, validates, compiles and publishes as tested | alternate public builder safety; RUNTIME-AUDIT-007 |
| checkpoint syntax/graph | strict V4 field/version/graph mutation matrix | malformed continue/load is rejected without mutation | broad post-continue command equivalence; GAP-003 |
| checkpoint behavior | recapture semantic identity and checkpoint gate replay | representative restored RNG/map continuation | canonical multi-domain original-vs-continued sequence; GAP-003 |
| adjacent durable store | tests inspected in E-012, not executed by E-002 | static recovery/atomicity cases exist | actual result on this dirty adjacent server tree |
| error identity | `GameError::code` characterization plus adjacent adapter tests inspected | current variants map to stable codes/messages | backward protocol version evolution |
| battle log growth | E-021 `/tmp` focused actual-method probe | recent-tail cost grows ~linearly with retained N | permanent CI budget/real battle distribution; GAP-004 |

## Reliability verdict

- Positive: no ignored tests in E-002; high-risk data, deterministic ordering, checkpoint corruption and many atomic families
  have specific contract tests using public or near-root behavior.
- Negative: the S1 failed-command mutation family has no before/after regressions, StartNewGame error test is error-only,
  continuation equivalence is representative, and resource growth has no permanent budget test.
- Final criterion implication: C11 `FAIL`; the suite is broad and valuable, but risk-proportional failure-state coverage is
  incomplete for confirmed high/medium findings.
