# R06 단계검토 인계 — checkpoint 001

State: `부분 구현·검증 종료 / 단계검토 요청 대기`

## Identity와 검토 입구

- Handoff: `HA-CAVER-P2G60-001`
- Goal: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`
- Execution basis: `FPF-CAVER-P2G30-001`, `PP-CAVER-P2G40-001`, `PVR-CAVER-P2G40-001`, `CRA-CAVER-P2G20-003`
- Applied GEX revision: `2026-09-13-gex-r7`
- Stage target: `checkpoints/checkpoint_001/MANIFEST.json` (`R06_STAGE_REVIEW_TARGET_001`)
- Initial baseline: `baseline/initial_001/MANIFEST.json` (`R06_IMPLEMENTATION_BASELINE_INITIAL_001`)
- Command ledger: `EXPERIMENTS.md`; design/remaining decisions: `EXPERIMENT_NOTES.md`; current execution contract: `PLAN.md`

이 문서가 첫 단계검토의 entry다. 이 target은 Audit Target Record나 GEX-20 진입물이 아니며 WP/Phase/Goal 완료를 주장하지 않는다.

## 이 checkpoint에서 완료한 책임

`core_enhanced/src/game/value_expr/`에 production runtime과 authored schema로부터 격리된 값 표현식 커널을 추가했다.

- `AmountExpr/ValueExpr`, Snapshot/Live, typed units와 stat/HP/resource/time/result/member reference
- signed i128 checked Add/Mul/Ratio/Clamp, 네 rounding mode, compile-time constant fold와 final damage/heal narrowing
- 명시적 반복형 postorder compiler, immutable multi-segment stack plan, node/depth/stack/member/group/work/retained limits
- finite authored/resolved-member aggregate, WholeSet/owner/projected-tile grouping, Count/Sum/Max, explicit empty semantics
- stable member order, duplicate/conflict 처리, side/employee/equipment-copy owner domain 혼합 거부
- capability/requirements admission, closed diagnostic/fault code, evaluation meter/cancel, deterministic trace
- compiler version+typed plan+EffectSite 의미를 hashing하는 canonical binary fingerprint
- legacy Damage unsigned-absolute 및 Heal signed literal lowering

`core_enhanced/src/game/mod.rs`가 모듈을 등록한다. 초기 baseline에서 이미 존재하던 `core_enhanced/src/game/resource/mod.rs`의 `ResourceOwnerSelector`에는 ordered reference key에 필요한 `PartialOrd`, `Ord`, `Hash` derive만 추가했다.

## 의도적으로 남긴 범위

- bounded custom RON deserializer와 `DamageExpr/HealExpr` authored surface
- content compile/catalog/publication diagnostics 및 generation carrier
- battle state의 Snapshot/Live `EvalFrame`, 실제 target/copy/action identity capture
- resource lease read, completed-effect prefix witness, sequential fallible effect execution과 rollback
- Attack/SpellPower, weapon copy enhancement, cadence, save zero omission, Certified fingerprint
- public validate/describe/explain/simulate, integration fixtures, V01–V21 전체 검증과 비용 측정
- 독립 GEX-20 감사와 R07+

이 범위가 남아 있으므로 현재 production RON과 runtime은 새 expression capability를 소비하지 않는다. 기존 literal schema/behavior와 V7 save serialization은 이 checkpoint에서 바뀌지 않았다.

## 검증 증거

| 검증 | 결과 |
|---|---:|
| value_expr focused unit | 11 passed, 0 failed |
| authoring regression | 4 passed, 0 failed |
| skill runtime contract | 15 passed, 0 failed |
| resource regression | 27 passed, 0 failed |
| V7 checksum regression | 1 passed, 0 failed |
| `cargo check -p game_core --all-targets` | exit 0 |
| focused rustfmt check | exit 0 |
| Clippy with two baseline lint classes allowed | exit 0 |
| post-scope-restore value_expr/all-target check | 11 passed / exit 0 |

원시 출력과 해시는 `evidence/validation/`에 있다. allow 없는 Clippy는 initial bytes와 동일한 `boss_runtime_data.rs`/`world/tests/combat.rs`의 기존 deny 3건으로 exit 101이다. repo-wide rustfmt도 initial tree의 기존 광범위한 포맷 차이로 exit 1이다. 두 실패 실행을 숨기거나 PASS로 세지 않았고 `EXPERIMENTS.md`에 exact path와 분리 근거를 남겼다.

Pre-schema old-build 입력은 `evidence/preimplementation/PREIMPLEMENTATION_FIXTURES.json`에 있다. V7 raw fixture SHA-256은 `47af09db635ab1691dc61d7094d6b2414237562c27cd6e1d1b6ce9a4e49024c9`, literal RON은 `1bd51a81d6b528ea4a80895bf95348c8417fd7113c77a73c6a2543d7821cf8c0`이다.

## Reviewer focus

1. typed unit algebra, negative rounding, i128 overflow와 legacy narrowing이 D01 의미를 정확히 보존하는가.
2. explicit task machines가 recursive compile/evaluate stack을 만들지 않고 node/depth/stack/work admission과 nested member product를 정확히 적용하는가.
3. finite grouping의 identity normalization, owner domain, empty Count/Sum/Max와 canonical trace/fingerprint가 D04와 맞는가.
4. fingerprint encoder가 source-file 경로를 제외하고 EffectSite semantics·moment·provenance·compiler version과 plan 의미를 빠짐없이 담는가.
5. authored Deserialize/catalog/runtime 노출을 미룬 현재 경계가 parser-depth와 unsupported public capability를 만들지 않는가.
6. 다음 구간에서 실제 runtime ID/capability를 `EvalFrame`에 연결할 때 현재 String identity adapter를 좁고 fallible하게 유지할 수 있는가.

## 복원과 비교

- Before actual bytes: `baseline/initial_001/SOURCE_BEFORE_001.tar.gz`
- After actual bytes: `checkpoints/checkpoint_001/SOURCE_AFTER_001.tar.gz`
- Readable/recoverable comparison: `checkpoints/checkpoint_001/CHECKPOINT_DELTA.patch`
- Before/after per-file SHA-256와 added/modified/deleted 목록: target manifest
- Dirty/untracked preservation: initial/current scoped·global status files와 initial tracked binary diff

두 archive를 별도 임시 디렉터리에 풀어 비교해야 하며 기존 dirty worktree에 덮어쓰면 안 된다. 검토 전에는 이 target의 source, 실행 기록, evidence를 변경하지 않는다.
