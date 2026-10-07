# D07-B F03 C03 successor 단계검토

2026-09-28 · **PARTIAL_SCOPE_PASS — C03 runtime 경계값·관측 동등성 범위 통과**.

새 시험이 요구한 실제 command 입력과 관측 on/off 동등성을 검증함을 확인했다. 이번 runtime 범위에서 추가 교정 요구는 없다. **C03-OBS01의 RON i32::MIN 입력 문제는 미해결**이며 C15 공개 입력·직렬화 통합 점검의 잔여 의무로 유지한다. C03 부분 PASS는 이 문제나 F03/D07-B 전체의 완료 판정이 아니다.

## 대상과 검토 경계

- [구현 인계서](../../corrections/wp02_d07_b_f03_legacy_001/REVIEW_HANDOFF.ko.md), [동결 manifest](../../corrections/wp02_d07_b_f03_legacy_001/target/d07_b_f03_legacy_001/MANIFEST.json).
- Manifest SHA-256: `23aea02009bb0a850439d39c974908f556eb7ffe9862e0a8876745926ed7df48`.
- before 342개 digest: `f5cbc267ea1e86995e6c85433c8d20fdb81cd59bd64eeddac838cddff985183d`.
- after 343개 digest: `f9883de09a463ad9de856e6cbc3022abc2c27a0b5101cc71f9df9e7c488a7e51`.
- 변경 3개: `commands.rs`, `value_expr_runtime.rs`, `game/data/mod.rs`. 추가 1개: `value_expr_runtime/tests/legacy_amount.rs`. 선행 339개 파일 byte 동일.

원본 구현 대화와 분리된 사본에서 검토했다. 설계·지시서를 공유한 단계검토이며 설계까지 독립된 감사는 아니다. 제품 소스와 기존 동결 자료는 수정하지 않았다.

## 구현·시험 판단

`commands.rs`의 새 tap은 실제 `process_skill_commands_in_scope`에서 source 연결 후 command 소비 직전에 `ApplyDamage.source_snapshot.base_damage`와 `ApplyHeal.flat`을 읽는다. cfg(test) 전용이며 command를 변경하지 않는다. TLS observer scope 안에서만 수집하고 Drop으로 정상·panic 종료 시 해제한다. HP 제한 후 결과만으로 command의 큰 값·부호 보존을 추정하는 시험이 아니다.

허용된 literal은 기존 typed `set_effect_amount`와 정상 database builder/catalog compiler를 거친다. 음수 Damage 두 사례는 공개 setter의 거부와 원본 값 보존을 먼저 확인한 다음, cfg(test) 전용 fixture에서 skill과 catalog를 함께 준비한다. catalog도 실제 `compile_skills`로 생성하며 실행 literal만 몰래 바꾸지 않는다. 기존 공개 Damage 음수 거부를 유지하면서 내부 unsigned_abs 호환 경계를 시험하는 것으로 인정한다. 이 staging은 정상 publication·content generation 일치의 근거가 아니다.

각 입력에서 같은 core의 복사본으로 관측 없음 / 일반 관측만 / 수치 관측 on을 비교한다. 실제 transaction과 ordered executor를 통해 command 입력·적용 수·최종 HP·완료 fact를 독립 기대값으로 검사하고, gameplay snapshot·queue·runtime facts·event log·step result 전체의 설정 간 동등성을 확인한다.

| 입력 | Damage command / HP | Heal command / HP |
| --- | --- | --- |
| i32::MIN | 2147483648 / 0 | -2147483648 / 0 |
| -7 | 7 / 33 | -7 / 33 |
| 0 | command 없음 / 40 | 0 / 40 |
| 7 | 7 / 33 | 7 / 47 |
| i32::MAX | 2147483647 / 0 | 2147483647 / 80 |

Damage 0은 command·damage fact 없이 평가 record/순번 1을 남기며, Heal 0은 command와 delta 0의 완료 fact를 남긴다. 새 시험이 이 차이를 명시한다. 수치 관측 on에서는 독립 기대 amount·literal provenance·단일 Const trace·순번 1, 공통 evaluator 및 preview/explain 재평가를 확인한다. 반복 문서와 usage·query 후 상태 불변도 확인한다. 공개 Latest/Exact 선택·terminal·취소·한도는 이 private encoder 시험으로 닫지 않는다.

## C03-OBS01 — 후속에서 반드시 대조할 입력 문제

동결 실패 로그 `c03_boundaries_001`에서 실제 SkillDatabase의 Damage/Heal MIN fixture가 runtime 진입 전에 `ExpectedInteger`로 실패한 것을 확인했다. 검토자는 현재 빌드된 RON 0.5.1 의존성에 [별도 probe](ron_min_probe.rs)를 연결해 다음을 재현했다.

- `i32::MIN` 직렬화 결과는 `-2147483648`이나 같은 문자열의 i32 역직렬화는 실패한다.
- MIN+1, -7, 0, 7, MAX는 같은 scalar round-trip에 성공한다. MIN 문자열을 i64로 읽는 것도 성공한다.
- 로컬 RON 소스 `parse.rs::signed_integer`는 `-`를 소비한 뒤 같은 타입 T로 양의 크기를 읽고 부호를 바꾼다. T=i32에서 2147483648이 표현 범위를 벗어나므로 음수 변환 전에 실패한다.

[probe 실행 로그](RON_MIN_PROBE.log)는 의존성 scalar 경계의 재현이며 전체 게임 저장 파일 round-trip 성공/실패 시험을 대신하지 않는다. 원래 SkillDatabase 실패와 함께 보면 typed Heal MIN 허용과 RON 입력 경계의 불일치를 후속에서 다뤄야 한다. 이번 시험 전용 변경이 만든 파서 회귀는 아니다.

**판정:** C03은 지시서상 catalog/runtime의 unsigned_abs·signed narrowing·zero 및 관측 동등성 범위이므로 부분 PASS를 허용한다. C03-OBS01은 C15에서 실제 공개 입력과 직렬화 경로를 재현하고 해결·재검증하거나 명시적인 범위/정책 결정을 받기 전까지 열어 둔다. 단순히 알려진 라이브러리 문제라는 이유로 전체 완료 때 누락하지 않는다. 임의로 음수 Damage 공개나 Heal 허용 범위를 바꾸지 않는다.

## 검증·재사용

- 직접 재실행: `cargo test -p game_core --lib d07_f03_c03_ --offline --locked -- --nocapture --test-threads=1`, **2 passed / 0 failed**. 집중 2개만 출력 대조를 위해 한 스레드로 실행했다.
- 직접 출력 대조: 10개 경계값×3설정 **30회**의 amount·command 개수·최종 HP를 독립 산술 기대값과 frozen matrix에 대조했다.
- 최종 영수증 7건의 source 전후 digest·log hash·실제 시험 summary 확인. 구현자 전체 1,552개·authoring 112개·value_expr 101개, check·Clippy·변경 파일 format 통과. 검토자가 전체 suite를 재실행한 것은 아니다.
- 343개 source membership/currentness, 동결 기록 414개, 격리 342→343→342 byte 복원 확인.
- 변경 3개에서 새 cfg(test) module/impl/call만 제외하면 predecessor 원본과 byte 동일함을 별도로 대조했다. 선행 source row와 측정 재사용 자료 hash도 확인했다. PC36/R05 재사용 적용성은 유지되며 이번 새 측정은 없다.
- PC36 성공 31 / typed 거부 5, R05 raw exit 101 및 concurrent peak 9,049,037 bytes / cap 초과 18,849 bytes 단일 경고를 유지한다. 새 수치 관측 비용 보장으로 확대하지 않는다.

근거: [집중 로그](FOCUSED_RECHECK.log), [source·영수증·30행 대조](SCOPE_AND_RECEIPTS_CHECK.json), [시작 확인](ENTRY_CURRENTNESS_AND_RESTORATION.json), [종료 확인](EXIT_CURRENTNESS_AND_RESTORATION.json), [검토 결속](REVIEW_VALIDATION.json).

## 다음 구간

F01·F02·C04/C05 선행 부분 PASS를 유지한다. 남은 **C06–C15 및 C03-OBS01**은 원본 구현 대화에서 사용자 재개 요청 뒤 처리한다. 다음 후보는 C06의 집계 관계·필터·복수 node를 실제 소비와 기록 재평가로 연결하는 검증이다. 의미 있는 구간에서 자체 검증·target 동결·사본 검토를 반복한다. F03·D07-B·WP02·R06 전체 완료는 선언하지 않는다.
