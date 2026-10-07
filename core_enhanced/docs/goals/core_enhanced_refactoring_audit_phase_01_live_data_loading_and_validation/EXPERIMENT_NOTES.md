# Core Enhanced Refactoring Audit Phase 01 — Experiment Notes

Status: complete

## 구조적 판단

- Phase 1의 authority family는 embedded production file set, RON parse/validation과 typed database
  publication이다.
- Individual gameplay policy consumption은 Phase 2~9의 독립 경계다. Loader host file을 공유한다는 이유로
  domain behavior 후보를 Phase 1에 합치지 않는다.
- Source-count/canonical drift는 조사 신호이며 runtime defect나 refactor candidate 자체가 아니다.

## Direct dependency and activation

- Parent Master: `core_enhanced_refactoring_audit_master`, CRA-01~CRA-13
- Direct predecessor: `core_enhanced_refactoring_audit_phase_00_surface_inventory`, `Complete / High`
- Boundary proposal: Phase 1 `embedded live data loading/validation`
- 2026-08-07 사용자 지시: current proposal로 시작하고 앞으로의 Child/후속 Goal 이름은 작업자가 결정
- Naming 위임은 material scope 또는 gameplay/data/compatibility 정책 결정 위임이 아니다.

## Source-of-truth conflict known at activation

Phase 0은 다음을 확인했다.

- production live/reference file set은 각각 23 files로 exact match
- code/test는 manifest 외 22 sources를 표현
- current data canonical은 21 sources라는 문구를 포함

어느 표현이 stale인지, count constant/document가 독립 authority인지와 후보 work type은 아직 확인하지
않았다. Runtime/live data를 우선하고 canonical은 durable contract intent와 drift owner를 확인한다.

## Dirty worktree precautions

- Repository root: `/mnt/f/work/simulator`
- Start commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- Baseline: 2026-08-07 current dirty working tree
- Core source/test/docs, production live data와 direct server paths에 기존 사용자 변경이 있다.
- Master/Phase 0과 이 Phase Goal artifacts만 audit-owned로 취급한다.
- 기존 code/data/test/canonical diff를 audit 변경으로 주장하거나 restore/format/정리하지 않는다.

## External systems and consumers

- `game_resources/data/**`: Core production loader가 실제 포함하는 파일만 confirmed supporting evidence다.
- `config/game_balance.toml`: Phase 0에서 workspace consumer가 확인되지 않아 `unverified`다.
- `game_server`: startup loader call evidence에 직접 필요한 범위만 read-only 확인할 수 있다.
- Unity와 다른 external consumers: 명시적 비범위/미검증이다.

## Applied and rejected alternatives

### Data files를 domain별 별도 Phase로 즉시 분할

적용하지 않는다. Phase 1은 공통 loading/validation/publication lifecycle을 먼저 감사하고 domain policy
consumption은 후속 authority Phase가 소유한다.

### Count drift를 즉시 canonical typo로 판정

적용하지 않는다. Runtime reference/live set, test owner, document intent와 history를 확인하기 전에는 stale
owner나 correction type을 확정할 수 없다.

### Audit 중 obvious 문구나 constant 수정

적용하지 않는다. Master와 Child는 production/canonical mutation이 승인되지 않은 read-only campaign이다.

## Omitted validation at initialization

Phase 문서 초기화 전에는 repository-wide search, runtime/data investigation, test, probe와 formatter를
수행하지 않았다. 따라서 loader correctness, validation completeness, fallback 존재, focused test coverage
또는 candidate 수를 주장하지 않는다.

## Remaining risks

- Large dirty baseline 때문에 Git history/change-frequency evidence가 current accumulated work를 대표하지 않을 수 있다.
- Generated/embedded path와 authored live file path가 macro/build semantics를 거치면 source trace만으로
  publication timing을 전부 입증하지 못할 수 있다.
- External authoring tools/consumers가 비범위라 cleanup readiness가 `unverified`로 남을 수 있다.
- Count drift가 behavior-neutral docs cleanup인지 future schema policy signal인지 evidence가 필요하다.

## Next safe work

1. 새 Phase의 필수 문서 3개와 Master/Phase 0 direct dependency를 다시 읽는다.
2. scoped dirty baseline과 exact Phase 1 file surface를 고정한다.
3. startup→loader→validation→publication path와 focused safety net을 조사한다.
4. candidate/disposition artifacts와 별도 completion audit를 작성한다.

## Current investigation decisions

### Source inventory authority

Current inclusion is correct: 23 embedded references and 23 production files, no difference. The defect signal is
not a missing file today; it is the proven parallel ownership between the actual source set, an isolated count
constant, a tautological literal test, and stale canonical prose. This is classified as behavior-preserving
ownership consolidation rather than a schema or file-format migration.

### Generated preview gate

The post-assembly candidate gate protects atomic publication, but only with baseline tuning. Runtime consumes
manifested difficulty that can add/scale waves. Because the current canonical explicitly promises every stage,
closing this gap is a `correctness_correction`, not a speculative refactor or new gameplay policy. It is a
priority candidate due to correctness and whole-run/startup blast risk, but implementation must add a later-stage
negative characterization test first.

### Fixture versus production lifecycle

`GameDataBuilder`, `RunPolicyData::builtin`, cached map catalogs, and the live-buff convenience are intentionally
test-oriented and validated. Their separate panic semantics are not competing production publication authority.
They are rejected as a merge candidate unless future evidence shows drift or an actual consumer problem.

### Private publication mapping

Parsed and published catalog field mapping is mechanically repeated. A private-only consolidation can reduce
change amplification without changing public fixture APIs. Removal of `GameDataBaseParts` or builder APIs is not
authorized because other Rust consumers are unknown.

### Config disposition

`core_enhanced/config/game_balance.toml` exists, but workspace code contains no path/symbol/generic Core loader
evidence. It remains `unverified`, not obsolete. External authoring/tool consumer evidence is required before any
cleanup candidate can be confirmed.

### Validation limits handed to later Phases

Phase 1 confirms strict loading and current publication. It does not claim that every gameplay default is the
right domain policy, nor that every runtime consumer interprets a catalog correctly. Those meanings remain with
Phases 2–9. Phase 2 may rely on exact current file inclusion but must retain the C02 caveat that startup preview
generation is baseline-only.
