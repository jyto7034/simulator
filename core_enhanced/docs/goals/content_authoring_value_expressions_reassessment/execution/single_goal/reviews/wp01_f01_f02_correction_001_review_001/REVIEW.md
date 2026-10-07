# R06 WP01 F01/F02 correction 001 재감사

검토일: 2026-09-13  
판정: **부분 범위 PASS — WP01-F01/F02 해소, 새 확정 findings 0건**.

대상: `R06_WP01_CONTENT_INPUT_COMPILE_CORRECTION_TARGET_001`  
Manifest SHA-256: `679336ad025f9dfaa6b44345ca3e494fd694e5903d49786a91d0ff270dbd642f`  
Correction baseline SHA-256: `a431db07b318c56f572b1799f9ce186903ecc667d3bddb898fbf78f8c7eb635e`  
Predecessor target SHA-256: `6f55cfa0fae0d88971eba6cb3c47e39ab5d12a2e99b2656a40eb2b51e7cf76de`.

## 범위와 독립성

사용자가 요청한 F01/F02 교정 재감사다. 이전 [검토 보고서](../wp01_content_input_compile_001_review_001/REVIEW.md), 현재 correction 인계서·PLAN·EXPERIMENTS·EXPERIMENT_NOTES, source delta 4파일과 관련 compiler/evaluator 경로, actual-byte archive와 validation receipts를 확인했다. 적용 절차는 GEX gex-r7 §6.4의 부분 단계검토다.

이 reviewer는 이전 결함 발견과 교정 방향·지시서 작성에 참여했지만 이번 제품 코드 구현에는 참여하지 않았다. 완전한 blind review로 표시하지 않는다. 제품 수정 없이 새 review directory와 격리 임시 fixture에만 진단 자료를 추가했다.

## WP01-F01 — 해소

[parse.rs:400](/mnt/f/work/simulator/core_enhanced/src/game/value_expr/parse.rs:400)의 WireValueExpr와 [mod.rs:188](/mnt/f/work/simulator/core_enhanced/src/game/value_expr/mod.rs:188)의 ValueRef에 enum-level `deny_unknown_fields`가 적용됐다. 기존 AmountExpr/TypedInteger/AuthoredMember/GroupOwner/ProjectedTile의 strict 설정도 유지된다.

- 기존 unknown Mul 필드 재현은 직접 파서에서 오류로 거부된다.
- 같은 입력의 실제 외부 pack load도 `ron_parse`를 반환하며 definition/action/effect 식별자를 유지한다.
- 구현자 unit의 Mul/Ratio/Clamp/Aggregate, 모든 struct형 ValueRef payload, 정상 입력 대조군을 직접 재실행해 통과했다.
- 추가 reviewer matrix 6사례: 첫/중간 위치의 unknown 필드, 중첩 payload, AuthoredMember의 owner 내부 unknown 필드도 모두 `ron_parse`와 unknown-field 원인으로 거부됐다.
- 변경은 기존 Deserialize의 필드 검증 강화다. 전체 RON 재파싱이나 사전 AST 복제, admission 제한 확대는 추가되지 않았다.

## WP01-F02 — 해소

[catalog.rs:345](/mnt/f/work/simulator/core_enhanced/src/game/value_expr/catalog.rs:345)의 `can_evaluate_without_external_context`는 resolved target 요구가 없고 모든 참조가 정확히 ValueRef::AggregateMember인 경우만 기존 evaluator로 정적 검증한다. 참조가 없는 기존 상수 식도 포함된다.

이 판단의 전제도 코드에서 확인했다.

- [compiler lexical 검사](/mnt/f/work/simulator/core_enhanced/src/game/value_expr/compile.rs:1368)가 Aggregate 바깥 member read를 거부한다.
- ResolvedTargets source는 compiler의 별도 requirement flag에 누적된다.
- Stat/HitPoints(subject: AggregateMember)는 로컬 member value와 별개의 ValueRef이므로 이 조건을 통과하지 않는다.
- compiler/evaluator/비용·fingerprint 알고리즘은 이번 delta에 포함되지 않는다.
- 정적 평가 시 기존 ExprMeter, evaluate, Damage/Heal narrowing을 그대로 사용한다. 별도 산술 구현이나 임의 frame 값 대입이 없다.

기존 authored -1 Sum Damage는 이제 실제 외부 pack에서 `value_expr_overflow`로 거부된다. 구현자 unit의 Heal i32 초과, 정상·중첩 집계, member 값 0에 따른 divide-by-zero, source/member stat 및 resolved target 대조군도 통과했다.

추가 reviewer 외부 pack matrix 7사례:

| 사례 | 실제 결과 |
| --- | --- |
| ByOwner 집계 결과 -1 | value_expr_overflow |
| 중첩 authored 집계 결과 -1 | value_expr_overflow |
| 빈 authored Max | value_expr_unavailable |
| 빈 authored Sum | content_capability_unsupported |
| Source stat을 분모로 쓰는 유효한 동적 식 | content_capability_unsupported |
| local member + Source stat 혼합 | content_capability_unsupported |
| authored 집계 안의 ResolvedTargets 집계 | content_capability_unsupported |

정상/동적 식의 마지막 오류는 현재 WP02 미연결 publication gate의 의도된 동작이다. 동적 식을 빈 frame으로 평가하거나 0으로 대체해 생긴 오류가 아니다. 모든 사례의 definition/action/effect 식별자도 검증했다. 격리 fixture의 literal 원본을 복구한 뒤 정상 load가 성공했다.

## 직접 실행한 검증

- `cargo build -p game_core --lib --offline --locked`: PASS, dead_code warning 6건. 현재 library로 외부 probe를 재링크했다.
- [ValueExpr unit](UNIT_RERUN.log): **36/36 PASS**.
- [콘텐츠 파이프라인](CONTENT_PIPELINE_RERUN.log): **59/59 PASS**. initial tool 출력 일부가 잘렸음을 로그에 명시했으며 최종 결과 요약은 완전하다.
- [기존 리뷰 probe 재실행](REVIEW_PROBES.log): **3 PASS / 0 FAIL / 1 ignored**. 과거 probe의 assertion은 유지하고 fixture 경로만 새 임시 디렉터리로 변경했다.
- [확장 reviewer suite](BOUNDARY_PROBES.log): **5 PASS / 0 FAIL / 1 ignored**. 기존 3개 테스트를 포함하며 추가 2개 테스트가 위 7-case 외부 경계와 6-case strict parsing matrix다. 두 suite의 중복 테스트를 독립 coverage로 합산하지 않는다.
- [별도 depth probe](DEPTH_PROBE.log): **1/1 PASS**, 실제 RON depth 64 / nodes 127 / 2 MiB worker stack에서 정상 반환.
- [진입](TARGET_CURRENTNESS_ENTRY.json)·[종료](TARGET_CURRENTNESS_EXIT.json) currentness 및 독립 archive/delta 검증: PASS.

재현용 코드는 [review_probes.rs](review_probes.rs), [boundary_probes.rs](boundary_probes.rs), [verify_target.py](verify_target.py)에 보존했다. 기존 리뷰 원본은 수정하지 않았다. Rust probe는 `rustc --edition=2021 --test`에 현재 `target/debug/libgame_core.rlib` 및 `target/debug/deps`를 연결해 빌드했으며 실행에는 `--test-threads=1 --nocapture`를 사용했다. depth test는 `--ignored --exact parser_default_depth_on_worker_stack`로 별도 실행했다. 이후 재사용 시 probe의 fixture 출력 경로를 새 임시 디렉터리로 지정한다.

현재 target에 결속된 receipt/output 해시를 확인해 재사용했으며 이번 reviewer가 재실행하지 않은 항목:

- authoring 15, skill runtime 15, checkpoint/save 18.
- all-target check, focused rustfmt, 동일 baseline allows의 Clippy.
- 구현자의 수정 전 두 결함 재현과 invalid fixture setup 시도 기록.

이번 감사에서 전체 all-target test나 독립 before/after fingerprint 3종 비교를 새로 수행했다고 주장하지 않는다. fingerprint 관련 기존 unit 회귀는 ValueExpr 36건 재실행에 포함된다.

## 보존 및 복원 확인

[독립 검증기](verify_target.py)는 읽어 확인한 구현자 verifier에 더해 다음을 검사했다.

- 요청한 target SHA-256 일치; baseline source가 predecessor target source 313개와 정확히 일치.
- 현재 source scope 313개; 실제 변화는 0 added / 4 modified / 0 deleted.
- 참조 hash 검사 503회(중복 포함); records 6 / evidence 47 / validation receipts 23.
- archive 7개: source before/after 각 313, 기존 문서 4, review inputs 11, predecessor records 14, correction records 6, evidence 47. member set·중복·실제 bytes/hash 일치.
- 새 임시 디렉터리에 before source archive를 복원한 뒤 `git apply --check -p4` 및 `git apply -p4`로 SOURCE_DELTA.patch를 적용했다. 복원된 전체 313개 파일의 set/hash가 after와 정확히 일치했다.
- 기존 source 외 동결 기록·이전 리뷰 입력·실패 증거의 currentness도 유지됐다.

content_pack/tests.rs에는 기존 코드의 rustfmt 변경이 함께 포함돼 있다. 확인한 delta에서 제품 동작 변경이나 기존 assertion 약화로 이어지는 수정은 발견하지 못했다.

## 판정의 경계와 다음 행동

이전 WP01-F01/F02는 현재 correction target에서 해소한다. 이번 WP01 콘텐츠 입력·컴파일 부분 범위에 추가 교정을 요구할 확정 결함은 발견하지 못했다.

이는 전체 RON 입력 공간에 대한 무결함 증명, R06/WP/Phase/Goal 완료, 실제 Snapshot/Live 전투 실행 검증 또는 GEX-20/30 PASS가 아니다. 향후 참조/source 종류나 compiler requirement 의미가 바뀌면 정적 평가 분류도 함께 재검토해야 한다. 현재 authored expression publication 차단은 유지된다.

부분 범위 PASS와 다음 구현 승인은 다르다. 새 사용자 구현 요청 전에는 다음 구간을 시작하지 않는다. 재개 시 currentness와 남은 범위·의존을 확인하고 새 구현 전 baseline을 고정한다.

제품 코드, 기존 실행 문서, 동결 target·baseline·evidence는 수정하지 않았다.
