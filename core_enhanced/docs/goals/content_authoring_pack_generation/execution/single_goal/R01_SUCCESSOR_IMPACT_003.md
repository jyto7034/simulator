# R01 impact after R02 generated source correction003

R02 owner remains `(GS-CAPG-P2G50-001, GOAL-CAPG-01)` / AG-CAPG-01 and WP-CAPG-03. The current repair
implements existing R02 C03/SC01; original R02 audit F01 and R01 audit002 are immutable. R02's first auditor
accepted R01 audit002 as current at ATR002. That earlier acceptance does not silently certify this new delta.

The [exact final comparison](evidence/CORRECTION_IMPACT_003_FINAL.json) has eight changed Rust inputs and
271 unchanged recorded inputs against VI023. Runtime effect compiler, ability/executor/cast/proc/source/log,
world generation commit, save/checksum/profile and product projection owners are unchanged. The public
preview APIs still expose the original GameError from the same generator and validator; only an ephemeral
internal diagnostic carrier retains wave/field/template addresses. LiveRonError adds owned related source
addresses; ordinary R01 errors have an empty list. No authored effect/cast index enters storage or semantic hash.

| R01 obligation | Correction impact and current direct evidence |
| --- | --- |
| IVA02 / supported authoring and addressed rejection | Existing typed opcode/parser/compiler/error identities unchanged; LiveRonError now adds related sources. Existing dynamic-source and unsupported authoring tests run in final core004. New generated003 separately proves exact custom policy and original wave addresses. |
| IVA04 / authored-to-runtime origin | Skill source binder, EffectSource/cast/proc/projectile/area forwarding and fixture bytes unchanged; final core004 executes those original tests. SourceTable now independently relocates each related address; the existing primary skill mapping is retained. |
| IVA05 / validated publication | External load still resolves/canonicalizes/lowers/validates before token creation. Generated rejection now preserves policy/template/boss graph contributors. Three new custom replacement tests and current release negative probes demonstrate no publication and deterministic retry/A preservation. |
| IVA06 / runtime and rollback invariants | Same generator checks, seed/case/wave ordering, preview instance construction, public GameError and setup execution. Original full runtime/withdrawal/signed-Heal/modifier/area/gating/save tests pass in core004; no gameplay or save owner changed. |

The final full core evidence is [1,217 actual passes](evidence/full_core_004.json), including 1,165 library and
52 integration tests, no failed/ignored cases. Intermediate VI025 1,216 passes and both discovery failures remain
historical; they do not certify the final BossPattern contributor behavior. API/observer reuse has exact limits
in [CORRECTION_REUSE_003](evidence/CORRECTION_REUSE_003.json); no new observer timing is claimed.

A separate fresh R01 GEX-20 must read the current stable target, prior audit and these actual changes/evidence,
verify still-applicable acceptance and decide currentness. Planned roles in the original R01 directory are
AUDIT_TARGET_003.json / COMPLETION_AUDIT_003.md / audit-evidence/successor_003. No positive verdict is
prewritten. After that prerequisite review, a distinct fresh R02 GEX-20 judges F01 and all applicable R02 conditions.
