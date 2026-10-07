# R06 Goal 실행 PLAN

Current state: `partial implementation validated / staged-review target fixed / review pending`

## 실행 identity와 권한

- Handoff: `HA-CAVER-P2G60-001`
- Goal Set / selected Goal / Phase: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`
- Assigned execution nodes: `AG-CAVER-01`, `WP-CAVER-04`; 구현 순서는 CRA D09의 `WP01 -> WP02 -> WP03 integration -> WP04 evidence`를 유지한다.
- Current basis: `FPF-CAVER-P2G30-001`, `PP-CAVER-P2G40-001`, `PVR-CAVER-P2G40-001`, `CRA-CAVER-P2G20-003`.
- 요청 모드: exact Single Goal 구현 요청에 따른 `goal-run`. 현재 요청은 구현·자체 검증을 허용하고 단계검토를 명시적으로 활성화했지만 GEX-20/30 감사, 다음 구현 구간, R07+ 또는 외부 배포를 허용하지 않는다.
- 실행 문서: 이 파일이 current locator, `EXPERIMENTS.md`가 시간순 명령·결과, `EXPERIMENT_NOTES.md`가 기술 판단·잔여 범위를 소유한다. 동결 P2G 산출물과 기존 audit verdict는 수정하지 않는다.

## 적용 지침 revision과 영향

Planning close는 `2026-09-12-gex-r5` bytes를 고정했다. 이번 인수의 적용 지침은 `core_enhanced/docs/goal_execution_pipeline(1).md`, revision `2026-09-13-gex-r7`이다. 현재 파일 diff에서 r6은 일반 구현 후 감사 인계 경계를 추가했고, r7은 명시적 단계검토의 중단 조건을 구현 전에 연결하되 실제 지점·개수·포함 작업은 구현 중 선택하도록 바꿨다. Goal/Phase/정책 의미와 기존 predecessor verdict는 바뀌지 않는다.

사용자의 “단계검토 과정을 포함” 요청에 따라 다음 조건을 처음 만족한 의미 있는 부분 변경에서 target을 고정하고 mutation을 멈춘다.

1. 최초 baseline 이후 하나의 책임·처리 경로·검증 가능한 동작으로 검토할 만하다.
2. 현재 crate가 빌드되고 해당 부분의 focused validation과 영향받은 공유 계약 회귀가 통과한다.
3. 완료 범위와 아직 연결하지 않은 runtime/stat/save/public 기능을 구분할 수 있다.
4. 복원 자료, 변경 비교, 명령·출력과 current source identity를 안정적으로 고정할 수 있다.

checkpoint를 만들기 위해 WP/Atomic Group/Phase를 새 독립 완료 단위로 바꾸지 않는다. 실제 경계가 성립하면 다음 큰 producer/consumer 연결에 착수하기 전에 종료한다.

## 인수와 현재 기준선

- Repository / branch / HEAD: `/mnt/f/work/simulator`, `cleanup/simulator-dirty-worktree-20260803`, `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- 최초 구현 baseline: `baseline/initial_001/MANIFEST.json` (`R06_IMPLEMENTATION_BASELINE_INITIAL_001`)과 detached `MANIFEST.sha256`.
- 복원 원본: `baseline/initial_001/SOURCE_BEFORE_001.tar.gz`; 307 current files의 실제 bytes를 포함한다. tracked deletion/binary delta는 `PREEXISTING_TRACKED_DIFF.patch`, scoped/global dirty와 untracked identity는 두 status 파일에 보존했다.
- Planning-close 대조: close manifest 469 rows 중 462 unchanged, 7 changed, 0 missing. 변경 7건은 GEX/P2G/운영·roadmap 문서이며 구현 관련 production source/data/test에는 planning close 이후 drift가 관찰되지 않았다. 현재 bytes는 초기 archive에 별도 보존했다.
- 기존 dirty tree 전체를 이 Goal의 변경으로 귀속하지 않는다. initial manifest scope 밖 파일은 구현 대상에서 제외하고 보존한다.

## D10 준비 상태

- R01 `ATR-CAMEC-R01-RESUME-001`, R04 `ATR-CARF-R04-RESUME-001`, R05 `ATR-CARL-R05-RESUME-001`의 report/target/closure를 직접 확인했다. 각 closure는 `Complete / High`, `passed: true`, mismatch empty이며 R06 미착수라고 명시한다. 이 실행은 verdict를 재발행하지 않는다.
- 이전 코드로 생성한 raw V7 fixture와 literal RON fixture를 `evidence/preimplementation/`에 schema 변경 전에 고정했다. 생성용 임시 test는 1건 실행 후 제거했고 해당 test source는 initial manifest hash와 다시 일치한다.
- RON 0.5.1은 public `Deserializer`/Serde seed hook을 제공하지만 deserializer 자체 depth limit은 없다. `InputLimits.max_nesting`은 현재 pack dependency용이다. recursive expression을 authored schema에 연결하기 전 custom bounded `DeserializeSeed`/visitor 또는 동등하게 parse 중 depth/node를 차단하는 구현이 필요하다.
- 현재 `query_action`은 active snapshot도 반환하고 `take_settled_action_result`만 settlement를 강제한다. `with_instant_resource_delivery` callback은 void이며 runtime callers에 early-return이 있다. 이들은 WP02에서 Result/prefix 경로를 실제 call graph와 함께 전환해야 한다.
- `UnitStats`/weapon/save 관련 zero field는 아직 추가하지 않았다. old V7 fixture의 raw checksum과 literal canonical bytes는 WP03 zero omission 검증 입력이다.
- direct server/Unity 배포는 범위 밖이다. server source는 core public DTO 변경의 read-only 영향 확인만 수행한다.

## 구현·검증 계획

승인된 D09 순서를 유지한다. WP01에서 typed expression schema, units, checked integer operations, explicit rounding, bounded compiler/immutable plan/diagnostics와 content compilation seam을 만든다. WP02에서 runtime frame, snapshot/live capture, finite grouping, resource/result prefix와 fallible sequential execution을 연결한다. WP03에서 Attack/SpellPower, weapon/copy enhancement, cadence, save/fingerprint를 같은 shared contract로 전환한다. WP04에서 public RON/CLI/runtime fixtures, canonical 문서, full regression/cost evidence를 닫는다.

구현 중에는 부분 경계를 실제 dependency로 판단한다. 첫 구간의 최소 안전 조건은 production runtime에 미지원 expression을 publish하지 않으면서 typed compiler/evaluator 책임이 완결되고, limits·unit/rounding/overflow/aggregate 진단이 nonzero tests로 입증되며 crate 및 기존 literal authoring/runtime 회귀가 통과하는 것이다. 이 조건이 성립하면 authored catalog/runtime 연결이라는 다음 큰 작업 전에 checkpoint를 선택한다.

## 선택한 단계검토 checkpoint

실제 구현에서 첫 경계를 `R06_STAGE_REVIEW_TARGET_001`로 선택했다. 이 경계는 새 `game::value_expr`의 in-memory typed AST, 반복형 bounded compiler, immutable segmented stack plan, canonical semantic fingerprint, 반복형 checked evaluator, finite grouping, meter/cancel, typed diagnostic/fault와 legacy literal lowering·narrowing까지 한 책임으로 닫는다. `game/mod.rs`는 모듈을 등록하고 기존 `ResourceOwnerSelector`에는 expression reference의 ordered key에 필요한 `PartialOrd/Ord/Hash`만 추가했다.

이 checkpoint는 authored `RON` deserialization, content catalog/publication, battle `EvalFrame` capture, resource lease/result prefix, effect executor 전환, stat/weapon/save/public CLI 연결을 포함하지 않는다. 따라서 미지원 expression이 content/runtime capability로 공개되지 않으며 기존 literal schema·실행·V7 bytes에는 변경이 없다. WP01 전체나 WP/Phase/Goal 완료를 주장하지 않는다.

focused unit은 11건 nonzero PASS이며 unit/rounding/overflow/clamp/dynamic frame/finite empty·grouping/owner domain/canonical order/boundary budgets/cancel/capability/legacy narrowing을 검사한다. 기존 authoring 4건, skill runtime 15건, resource 27건, V7 checksum 1건과 `cargo check --all-targets`가 통과했다. 초기 tree의 Clippy deny 3건을 그대로 확인했고 두 lint만 command-line allow한 전체 Clippy는 exit 0이다. repo-wide fmt는 기존 source 전체 포맷 차이로 실패하지만 새 모듈 focused rustfmt check는 통과한다. 상세 command/exit/output locator는 `EXPERIMENTS.md`와 validation receipts가 소유한다.

## 종료점

검토 entry는 `CHECKPOINT_REVIEW_HANDOFF.md`, target은 `checkpoints/checkpoint_001/MANIFEST.json`이다. initial/current actual-byte archive와 readable binary delta를 고정한 뒤 source·실행 기록·evidence mutation을 멈추고 `부분 구현·검증 종료 / 단계검토 요청 대기`로 종료한다. GOAL-CAVER-01 전체 완료, GEX-10 exit, Audit Target Record 또는 GEX-20 verdict는 선언하지 않는다.
