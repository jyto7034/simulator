# R06 correction 001 R01–R03 재검토 인계

Status: `교정본 재검토 준비 완료 / 재검토 요청 대기`

이번 인계는 R01–R03 교정, 자체 검증, successor target 동결까지만 다룬다. 다음 R06 기능, GEX-20/30, WP/Phase/Goal 완료를 선언하지 않는다.

## 동결 경계

- 수정 전 baseline: `R06_CHECKPOINT_001_CORRECTION_BASELINE_002`
- baseline manifest: `core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/correction_001_r01_r03/baseline/pre_fix_002/MANIFEST.json`
- baseline SHA-256: `96eb47695ea346627b5c0c6aedbd75953cf496353ad00b03b04d37bd80232bda`
- 수정 후 target: `R06_CHECKPOINT_001_CORRECTION_TARGET_002`
- target manifest: `core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/correction_001_r01_r03/target/correction_002/MANIFEST.json`
- target SHA-256: `1bedd79cf745bb489db18172b3c29bc3477f728fa9849d2b4e36f357224ff85a`
- source before/after actual-byte archive와 binary-capable source delta, 변경 문서 before/after actual-byte archive와 delta, validation evidence archive를 target에 보존했다.

## R01

compile/evaluate가 member scan과 sort work helper를 공유한다. AuthoredMembers의 raw count, identity/owner/value payload, sort/group/reduction 및 중첩 반복을 checked arithmetic으로 plan에 합성한다. 정적 plan은 같은 `static_work` meter에서 실행 가능하며, compile 시 알 수 없는 frame/resolved payload는 `runtime_dynamic_work`와 승인 ceiling으로 분리했다.

검증은 재감사의 1-member 6/7 실패가 7/7 성공으로 바뀐 것, authored 15개 조합의 bound==actual, empty/long/group/nested 및 bound-1/bound/bound+1을 포함한다.

## R02

recursive `segment_shape`를 dependency 순서의 iterative bottom-up 분석으로 교체하고 scratch storage를 할당 전에 admission했다. 동시 value stack과 task frame 의미를 유지했으며 새 hard cap이나 worker stack 증가는 없다.

기본 depth 64/65, 확장 depth 512, 재감사 depth 2,048/2 MiB worker subprocess를 검사했다. deep probe는 `ManuallyDrop`으로 AST 파괴 재귀를 분리하며 수정 후 정상 typed 결과로 종료한다.

## R03

요구된 과거 PLAN을 실제 bytes로 복구했다: `evidence/r03/HISTORICAL_PLAN_PRE_FIX_001.md`, 4,460 bytes, SHA-256 `661dd456d923e323cfd5704dd520e322251ba9a603303a4b73c295e6c13412c7`. 원본은 Codex session JSONL line 1075의 apply-patch result이며 raw event와 recovery record도 보존했다.

predecessor baseline은 수정하지 않았다. 당시 실제 PLAN bytes를 archive하지 못한 역사적 결함과 successor에서의 원본 복구를 별도 세대로 표시한다. verifier도 current live PLAN과 recovered historical snapshot을 분리한다.

## 변경 파일

제품 source는 다음 네 파일만 변경됐다.

- `core_enhanced/src/game/value_expr/compile.rs`: `cada1346e2fc09f5661f0129c1d2e437ff5694ebbaf1c0d5075dd6384951ddbf` → `d4bb639c4abc749ababd983c24c723296ed95d553084de283ac751a5c9af9a79`
- `core_enhanced/src/game/value_expr/eval.rs`: `1591c19f86d5394f56641b0351970e994f712e0917c61d0a7e524704fb9158e4` → `961abbe0119a053c419edfae20550cfbfa45a6f63a385669c38c6a6574e56582`
- `core_enhanced/src/game/value_expr/mod.rs`: `4678673e201084c91aab00f5470adc0283c5ca51fd42328d387626c3f44d9ba2` → `b52e69da5c12155fbe69b7c95f3f872c686106cab9f28bb609520c14f9dac8f2`
- `core_enhanced/src/game/value_expr/tests.rs`: `8ae882c6e953cfdcf9fad7b81371dcfb591349f3c39bef39d663d17da6da382b` → `74b92b1caa0b3901088631e19133c49af3103866959f92af86c349f9290a9a19`

기록·보존 도구 변경은 `core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/correction_001_r01_r03`와 동일 goal의 `r06_correction_002_*`, `r06_run_correction_002_validation.py`, `r06_recover_historical_correction_plan.py`로 한정했다.

## 검증

이번 pass 실행:

- value_expr unit 22/22, predecessor correction probe 7/7
- 재감사 probe 10/10와 격리 deep probe 1/1
- all-target check PASS; all-target test lib 1,382 + integration 52 PASS
- focused rustfmt PASS
- Clippy PASS with predecessor baseline allow `clippy::never_loop`, `clippy::eq_op`
- pre-fix archive와 corrected source fingerprint 3/3 byte-identical

재사용:

- predecessor target source/record/evidence/archive currentness hash 검증
- predecessor의 Unix socket EPERM 환경 분류; 이번 전체 test는 승인된 환경에서 직접 PASS

보존된 실패는 pre-fix R01 exit 101, pre-fix R02 SIGABRT(-6), 구현 중 invalid ResourceId test-fixture exit 101이다. 마지막 실패는 유효한 100-byte fixture로 교정한 뒤 unit과 전체 suite를 통과했다.

## 남은 제약

- runtime-dynamic payload는 compile 시 알 수 없어 runtime meter가 제한한다.
- retained-byte 모델은 보수적 kernel-owned allocation model이며 RSS 보장이 아니다.
- predecessor baseline이 생성 당시 PLAN 실제 bytes를 누락한 사실은 유지된다. successor evidence가 원본을 복구했으며 과거 baseline을 소급 PASS로 바꾸지 않는다.
- 실제 lease 검증과 다음 R06 기능은 범위 밖이다.

이 target과 인계서를 재검토 입력으로 사용한다. 재검토 결과가 돌아오기 전까지 correction source·records·evidence를 변경하지 않는다.
