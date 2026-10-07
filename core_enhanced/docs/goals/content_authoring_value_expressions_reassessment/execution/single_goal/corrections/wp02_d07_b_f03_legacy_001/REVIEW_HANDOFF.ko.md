# D07-B F03 C03 — legacy 경계값과 관측 동등성

2026-09-28 · **C03 부분 자체 검증·단계검토 인계**. F03/D07-B 전체 완료 또는 독립 검토 PASS가 아니다.

## 기준과 범위

[C04/C05 사본 부분 PASS](../../reviews/wp02_d07_b_f03_replay_001_review_001/REVIEW.md) 후 사용자 재개 요청으로
[기존 교정 지시서](../../handoffs/WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md)의 C03/B01 및 이번 C16을 진행했다.
두 구현 템플릿 `2026-09-27-v1`에 따라 [PLAN](PLAN.md), [시도 이력](EXPERIMENTS.md), [현재 판단](EXPERIMENT_NOTES.md)을 갱신했다.

legacy Damage/Heal의 최소·최대·음수·0 입력에서 실제 command에 전달되는 amount, HP 제한 후 결과,
완료 fact 및 관측 record를 대조했다. 공개 입력 정책·제품 수치 계산·보관/한도는 바꾸지 않았다.

## 코드 연결과 준비 경계

- `value_expr_runtime/tests/legacy_amount.rs`: 기존 test fixture에 새 module을 연결한다. 각 입력의 catalog를 실제 compiler로 만들며,
  이미 컴파일한 값과 무관하게 실행 literal만 바꾸는 방식을 쓰지 않는다.
- `commands.rs::amount_command_observation`: **cfg(test) 전용** thread-local observer. actual `process_skill_commands_in_scope`가
  command를 소비하기 직전에 target과 DamageSourceSnapshot.base_damage / Heal.flat을 읽는다. command를 바꾸지 않는다.
  scope 밖에는 자료를 쌓지 않으며 정상 종료·panic 시 해제한다. 서로 다른 시험 스레드의 기록은 섞이지 않는다.
- `GameDataBase::stage_legacy_negative_damage_for_test`: **cfg(test) 전용** 음수 Damage fixture.
  공개 `set_effect_amount`의 음수 거부와 원본 literal 유지 확인 후, unique fixture database에서 skill과 catalog를 함께 준비한다.
  `ValueExprCatalog::compile_skills`를 실제로 사용하며 미리 만든 frame/record를 주입하지 않는다.

**공개 허용값과 내부 호환 사례는 구분한다.** 기존 SkillDatabase는 음수 Damage를 공개하지 않는다.
Heal의 signed i32 및 Damage의0/양수는 기존 typed authoring setter→정상 builder validation→catalog 경로다.
음수 Damage 두 사례(-7, i32::MIN)는 기존 unsigned_abs runtime 호환 의미를 확인하는 내부 staged 시험이다.
공개 validation 우회 기능을 제품에 추가하거나 음수 Damage publication을 허용하지 않았다.

## C03 신규 관측 시험

공통 조건: 대상 HP40/max80, Damage는 True, 실제 action/source와 effect site 존재, actual ordered executor를 transaction 안에서 실행.
동일 core를 복제해 `관측 없음 / 일반 관측만(숫자 off) / 숫자 관측 on`을 각각 실행한다.
두 신규 시험의 접두사는 `game::battle::core::value_expr_runtime::tests::legacy_amount::d07_f03_c03_`다.

| 시험 suffix | literal 입력 | command amount와 최종 HP |
| --- | --- | --- |
| `legacy_damage_boundaries_match_observation_off_and_on` | i32::MIN / -7 / 0 / 7 / i32::MAX | Damage2147483648→HP0 / Damage7→33 / command 없음→40 / Damage7→33 / Damage2147483647→0 |
| `legacy_heal_boundaries_preserve_signed_commands_and_clamping` | i32::MIN / -7 / 0 / 7 / i32::MAX | Heal-2147483648→HP0 / Heal-7→33 / Heal0→40 / Heal7→47 / Heal2147483647→80 |

10개 경계값×3설정=30회 실제 실행이다. 소비 직전 command는 각 독립 기대값과 일치한다.
Damage0은 command·damage fact가 없지만 숫자 관측 on에서는 평가 record/순번1을 남긴다.
Heal0은 command와 요청 delta0의 heal fact를 남기고 HP는 유지한다. Heal 음수도 요청 부호를 유지한다.
큰 값은 최종 HP만 대조하지 않고 command 입력과 requested_delta/실제 delta를 구분한다.
Damage fact의 requested_damage는 downstream formula의 raw_damage이므로 이를 command base_damage의 대용 oracle로 쓰지 않는다.

각 설정의 transaction gameplay snapshot, queue, runtime facts 전체, event log, step result가 동일한지 대조한다.
숫자 관측 on의10개 record에는 독립 기대 amount, literal provenance, 단일 Const trace와 ordinal1을 확인한다.
실제 보관 plan/frame을 공통 evaluator와 preview/explain encoder로 재평가하여 amount/trace 일치,
반복 문서 bytes·usage 및 query 후 gameplay/queue/facts 불변을 확인한다. 숫자 off에는 record/순번 증가가 없다.

명령·환경·source 전후 digest·실행 수와 원문은 [집중 영수증](evidence/c03_boundaries_002.json),
[집중 로그](evidence/c03_boundaries_002.log)에 있다. [원자료 판정표](evidence/BOUNDARY_MATRIX.json)의 `D07_F03_C03` 30행이 각 입력·설정·실제 command/HP를 출력한다.

## 실패·제한과 후속 관찰

최초 `c03_boundaries_001`은 0 pass/2 fail이었다. 처음 i32::MIN을 직접 넣은 RON fixture가 `ExpectedInteger`로 거부되어
실제 소비까지 도달하지 못했다. 당시 source archive와 로그를 보존했고 모든 값을 RON으로 준비할 수 있다는 가정을 폐기했다.
이후 코드 대조에서 기존 음수 Damage 공개 거부 및 Heal의 typed setter 지원을 확인해 위 준비 경계로 교정했다.
제품 파서나 공개 validation은 수정하지 않았다.

**C03-OBS01(후속 관찰):** 원문 RON의 i32::MIN 파싱은 이 checkpoint에서 해결하거나 성공했다고 주장하지 않는다.
Heal MIN의 typed authoring→catalog→소비는 검증됐지만 RON 입력/저장 round-trip까지 검증된 것은 아니다.
원인 분석과 공개 입력·직렬화 경계는 후속 C15 통합 점검 때 이 실패를 다시 대조해야 한다. 음수 Damage 공개 허용 요청은 아니다.

이번 query는 private `encode_record_for_test` 경로다. 실제 public Latest/Exact selector, terminal, 취소·한도 등을 검증했다는 의미가 아니다.
C06–C15와 F03/D07-B 전체 요구는 남아 있다. 단계검토는 C03 runtime 경계와 명시한 준비 경계에 한정한다.

## 최종 검증·재사용·동결

[최종 판정표](evidence/FINAL_VALIDATION.json)에 신규/기존 회귀, 실제 명령·시험 수·소요 시간·source digest를 모았다.
신규2개·authoring112개·value_expr101개·전체1552개(기본 병렬 약125초)가 통과했다. check/Clippy/변경 파일 format도 exit0이다.
Clippy 경고도 원문을 남긴다. 전체를 단일 스레드로 제한하지 않는다.

PC36/R05는 새로 실행하지 않고 [재사용 판정](evidence/EVIDENCE_REUSE.json)의 선행 F02 동결 측정을 사용한다.
직전342개 중339개는 byte 동일하다. 변경3개에서 새 cfg(test) module/impl/call만 제거하면 각각 원본과 byte 동일하며,
신규 파일도 기존 cfg(test) module 아래에만 연결된다. 지시서/검토/이전 재사용 근거와 도구·환경도 대조했다.
제품 admission/소유권에 변화가 없으므로 기존 적용성이 유지된다. 새 관측 비용 보장은 아니다.
PC36 성공31/typed거부5 및 R05 concurrent peak9,049,037 bytes, cap 초과18,849 bytes 단일 경고를 유지한다.

[successor manifest](target/d07_b_f03_legacy_001/MANIFEST.json)는 before342/after343개, 변경3개·신규1개,
전후 archive·정/역 patch·원자료를 결속한다. [격리 복원](target/d07_b_f03_legacy_001/RESTORATION_VERIFICATION.json)과
`python3 freeze.py verify`는 source membership/currentness 및 양방향 byte 복원을 확인한다.
최초 D07-B 대비 누적 delta는 최종 판정표에 기록한다. 다음은 사본 대화의 **C03 단계검토**다.

**부분 구현·자체 검증 종료 / 단계검토 target 동결 / 검토 요청 대기**
