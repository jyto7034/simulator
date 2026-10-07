# R06 WP01 콘텐츠 입력·컴파일 단계검토

검토일: 2026-09-13  
판정: **CORRECTION_REQUIRED — 확정 findings 2건(P2 2건)**.

대상: `R06_WP01_CONTENT_INPUT_COMPILE_TARGET_001`  
Manifest SHA-256: `6f55cfa0fae0d88971eba6cb3c47e39ab5d12a2e99b2656a40eb2b51e7cf76de`  
Before manifest SHA-256: `e884bc38014d1ce05dcede78a495c9ae6c888b2e9d99ca10c1d58ba4a22fe6ec`

## WP01-F01 [P2] 표현식 내부 unknown field가 조용히 폐기된다

위치: [parse.rs:400](/mnt/f/work/simulator/core_enhanced/src/game/value_expr/parse.rs:400), `WireValueExpr`의 Deserialize 정의.

외부 SkillDatabase RON의 DamageExpr에 다음 root를 넣으면 `SkillDatabase::from_ron`이 성공한다.

```ron
Mul(
    value: Const((value: 1, unit: Magnitude)),
    scalar: Const((value: 2, unit: Scalar)),
    typo_parameter: 123,
)
```

`WireValueExpr`에 unknown-field 거부 설정이 없어 마지막 필드를 무시한다. 동일 입력을 실제 외부 pack 로더에 넣어도 parse/compile을 통과하고 마지막 `content_capability_unsupported`에 도달한다. 단순한 저수준 파서 전용 경로의 문제가 아니다.

[현재 입력 계약](/mnt/f/work/simulator/core_enhanced/docs/data_loading_contract.md:646)은 unknown fields/variants 거부를 약속한다. 오타나 지원하지 않는 저작 속성이 성공한 표현식에서 사라지므로 이 계약에 어긋난다. 현재 publication gate가 실행을 막고 있어 잘못된 전투 결과까지 발생한 사례는 아니다.

교정 방향:

- WireValueExpr의 struct variant를 strict하게 역직렬화하고, Read 등에서 도달하는 nested authored 타입도 같은 계약인지 점검한다.
- 정상 입력 대조군과 함께 Mul/Ratio/Clamp/Aggregate 및 참조 payload의 unknown-field 회귀를 추가한다.
- unknown field를 검사하기 위해 전체 AST나 RON을 먼저 생성·재파싱하지 않는다. 기존 byte/depth/node/member admission 및 source 진단을 유지한다.

## WP01-F02 [P2] 정적 authored 집계가 최종 범위 검사를 건너뛴다

위치: [catalog.rs:331](/mnt/f/work/simulator/core_enhanced/src/game/value_expr/catalog.rs:331), `validate_result_contract`.

현재 코드는 `requirements.references.is_empty()`인 경우에만 실제 evaluate와 final narrowing을 수행한다. 그러나 `AggregateMember`는 외부 runtime 값이 아니라 authored 집계에서 제공되는 로컬 값일 수 있다. 이 참조가 있다는 이유만으로 정적 계산 대상에서 빠진다.

실제 외부 pack에서 다음 root를 DamageExpr로 사용했다.

```ron
Aggregate(
    source: AuthoredMembers([
        (identity: "a", value: (value: -1, unit: Magnitude),
         owner: None, projected_tile: None),
    ]),
    grouping: WholeSet,
    reduction: Sum,
    outer_reduction: None,
    member: Read(AggregateMember(unit: Magnitude)),
)
```

외부 frame 없이 결과가 -1로 결정되며 Damage의 u32 범위를 벗어난다. 대조군 `Const(-1 Magnitude)`는 `value_expr_overflow`로 거부되지만, 이 집계식은 정상 compile 취급 후 `content_capability_unsupported`를 반환한다. 즉 이번 단계가 맡은 정적 계산·narrowing 진단을 누락한다.

교정 방향:

- 참조 유무와 외부 context 필요 여부를 구분한다. authored member로 충족되는 로컬 참조까지 일괄 dynamic으로 분류하지 않는다.
- 정적으로 평가 가능한 계획에는 공통 bounded evaluator와 Damage/Heal narrowing을 적용한다. 동적 식을 임의의 기본값으로 계산해 검증하지 않는다.
- 모든 member 관련 참조가 정적으로 해결된다고 가정해서도 안 된다. 실제 외부 stat/resource 등이 필요한 경우와 순수 AggregateMember 값을 구분한다.
- 음수 Damage, Heal i32 범위 초과, 중첩 authored 집계, 정적 계산 fault 및 진짜 dynamic 식 대조군을 추가한다.
- WP02 미연결 publication gate는 그대로 유지한다.

## 재현과 검증

[검토용 public-API probes](review_probes.rs), [외부 pack 전체 사례 로그](EXTERNAL_COMPILE_PROBES.log).

외부 pack 사례별 결과:

| 사례 | 기대 | 실제 |
| --- | --- | --- |
| Const 음수 Damage | value_expr_overflow | 동일 |
| authored member -1의 Sum | value_expr_overflow | content_capability_unsupported |
| Mul의 unknown field | ron_parse | content_capability_unsupported |
| 정상 Const 표현식 | content_capability_unsupported | 동일 |

네 사례 모두 실제 definition/action/effect 식별자를 유지했다. 격리 fixture의 literal 원본을 복구한 뒤 정상 pack load도 성공했다. 저장소의 game_resources는 변경하지 않았다.

직접 실행한 검증:

- `cargo build -p game_core --lib --offline --locked`: PASS. 현재 library로 probes를 링크했다.
- [ValueExpr unit](UNIT_RERUN.log): 29/29 PASS.
- [콘텐츠 파이프라인](CONTENT_PIPELINE_RERUN.log): 59/59 PASS.
- 최종 review suite: 1 PASS / 2 FAIL / 1 ignored. 두 실패는 위 결함을 검출하도록 작성한 assertion이다. [최종 출력 tail](FINAL_SUITE_TAIL.log); 외부 사례 전체 출력은 별도 로그에 보존했다.
- ignored depth probe는 [별도 실행](PARSER_DEPTH_PROBE.log): PASS. 실제 RON depth 64, 127 nodes, 2 MiB worker stack에서 typed parsing과 반환이 정상 종료했다. 모든 AST 형태의 stack 안전성을 증명한 것은 아니다.
- [진입](TARGET_CURRENTNESS_ENTRY.json)·[종료](TARGET_CURRENTNESS_EXIT.json) currentness 및 독립 archive 검증: PASS.

구현자의 target-bound evidence/receipts를 해시 대조해 확인했으나 이번 reviewer가 재실행하지 않은 항목: authoring 15, literal runtime 15, checkpoint/save 18, V7 checksum 1, all-target check, 동일 baseline allows의 Clippy, focused rustfmt, 보존 fixture 3종. 이를 직접 재실행한 테스트 수에 합산하지 않는다. 이번 검토는 전체 all-target test 재실행이 아니다.

재현(repository root):

```sh
cargo build -p game_core --lib --offline --locked
rustc --edition=2021 --test core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp01_content_input_compile_001_review_001/review_probes.rs --extern game_core=target/debug/libgame_core.rlib -L dependency=target/debug/deps -o /tmp/r06-wp01-review.J309ox/review_probes
/tmp/r06-wp01-review.J309ox/review_probes --test-threads=1 --nocapture
/tmp/r06-wp01-review.J309ox/review_probes --ignored --exact parser_default_depth_on_worker_stack --test-threads=1 --nocapture
```

Probe의 pack 경로는 이번 리뷰 전용 임시 경로로 고정돼 있다. 다른 환경에서 재사용할 때는 새 임시 디렉터리로 바꾸며 실제 콘텐츠 디렉터리를 출력 대상으로 지정하지 않는다.

## 동결·범위·인수인계

읽기 전용 verifier에 더해 before/after archive의 exact member set·중복·실제 bytes/hash, 기록/evidence archive, source scope, 변경 목록을 독립 대조했다.

- Source before 311 / after 313; added 2 / modified 17 / deleted 0.
- Records 14 / evidence 51 / validation receipts 25.
- 참조 해시 검사 441회(중복 포함).
- SOURCE_DELTA.patch는 결속된 해시를 검사했다. 이번 감사에서 patch를 적용한 별도 복원 실행까지 했다고 주장하지 않는다.

borrow 기반 컴파일, lexical 실패 위치 탐색, final winner만의 catalog compile, authored expression publication 차단 경로를 확인했다. 기존 AST clone/전체 RON 재파싱 문제가 재도입된 근거는 발견하지 못했다. WP02 미구현 자체는 이번 범위의 결함으로 세지 않는다. 두 findings 모두 현재 gate를 우회해 publication에 성공한 사례가 아니다.

이 reviewer는 기존 검토·방향 논의에 참여했으나 이번 제품 구현에는 참여하지 않았다. 완전한 blind review로 표시하지 않는다. 검토의 발견 범위는 이번 WP01 구간이며 전체 입력 공간에 대한 안전성 보증이 아니다.

제품 source, 기존 실행 기록, 동결 manifest/archive/evidence는 수정하지 않았다. 추가한 것은 동결 scope 밖의 이번 review 자료와 격리 진단 fixture뿐이다. 다음 구현이나 GEX-20/30, WP/Phase/Goal 완료 선언은 수행하지 않았다.

후속 교정 시 WP01-F01/F02를 대상으로 새 before baseline을 보존하고 교정·회귀·새 target 동결 후 재검토를 요청한다. 기존 target과 이번 실패 증거를 덮어쓰지 않는다. 이 문서는 구현 명령이나 다음 기능 진행 승인이 아니다.

