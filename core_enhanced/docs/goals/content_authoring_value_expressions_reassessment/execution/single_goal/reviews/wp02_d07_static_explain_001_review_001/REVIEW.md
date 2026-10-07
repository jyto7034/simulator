# R06 WP02 D07-A 정적 compiled amount 설명 — 독립 단계검토 001

2026-09-27 · **D07-A 부분 범위 PASS** · 수정 요구 없음.

실제 winning compiled amount plan의 정적 설명, 공통 context 분류, 명시적 미평가,
bounded 출력 및 authored expression publication 차단을 확인했다.
이 판정은 실제 frame의 수치/trace preview, D07 전체, WP02/R06 전체 완료 판정이 아니다.
제품 소스와 기존 동결 산출물은 변경하지 않았다.

## 1. 검토 대상과 신원

- [지시서](../../handoffs/WP02_D07_IMPLEMENTATION_REQUEST_20260927.ko.md)
- [구현 인계](../../segments/wp02_d07_static_explain_001/REVIEW_HANDOFF.ko.md)
- [동결 manifest](../../segments/wp02_d07_static_explain_001/target/d07_static_explain_001/MANIFEST.json)
- manifest SHA-256: `ae6609c1e75b711a27cacf8467ac0c42eaa99635a0fc19c5ae486ecd949bb1ac`
- before: 337개, digest `8aa03d6bd04f8d524fa61b465a9bd7a4171062a1190f615b794e20f915190f04`
- after: 338개, digest `5279b8333c6be8f90992ac94a47f09beb083bf82bfcdf02ebbe3e0d179c8f530`
- after 제품: 334개, digest `17e41fa7625d001242fcfcec65834ab533fea43ea74e3f89f7ce78f3bae5acfd`

before는 직전 D04-OWNER-F01 PASS target의 after다. 기존 6개 파일 변경과 `authoring/amount.rs`
추가 1개를 정·역 patch와 실제 코드로 읽었다. 338개 currentness, 기록 98개와 archive 신원,
추가 파일의 부재↔존재까지 포함한 격리 byte 복원을 확인했다.
[진입 검증](ENTRY_CURRENTNESS_AND_RESTORATION.json)과 [종료 검증](EXIT_CURRENTNESS_AND_RESTORATION.json)에 기록한다.

## 2. 코드와 요구 대조

| 요구 | 검토 결과 |
| --- | --- |
| A01 | 기존 Effect query가 definition/action/effect를 찾은 뒤 해당 generation의 catalog entry를 직접 조회한다. Damage/Heal kind, source, moment/unit, fingerprint, requirements, PlanCost, segment ops를 accessor 그대로 설명한다. 기존 literal effect 출력도 유지한다. |
| A02 | 정상 비금액 effect에만 null을 반환한다. 금액 plan 부재는 `authoring_amount_plan_unavailable`, kind/site/source 불일치와 비금액 effect의 잘못된 plan은 `authoring_amount_plan_inconsistent`다. 기존 missing definition/action/effect 경로도 유지한다. |
| A03 | catalog의 `can_evaluate_without_external_context`를 crate 내부에서 공유한다. AuthoredMembers의 bare member와 resolved/runtime reference를 구별한다. 두 context 모두 `performed=false`이며 evaluator를 호출하지 않는다. i128는 typed serializer 경로를 유지한다. |
| A04 | segment index와 borrowed ops를 compiler 순서대로 출력하고 member_segment를 재귀 전개하지 않는다. 실제 Side/Employee/EquippedSlot 관계·필터·중첩 aggregate 시험 assertion을 확인했다. |
| A05 | same-ID 두 winner를 실제 loader로 발행하는 시험이 각 source/fingerprint와 옛 generation의 불변 문서를 대조한다. 추가 효과의 origin source도 기존 경로로 유지한다. |
| A06 | Inspection scope, 길이를 고려한 checked catalog lookup 비용, segment/requirements/source/cost의 계량, fingerprint 64바이트 사전 admission, bounded result writer를 사용한다. work/metadata/result 1 미만 거부와 긴 plan·취소 시험을 확인했다. |
| A07 | borrowed descriptor가 plan/requirements/source/AST를 복제하지 않는다. 새 소유 payload는 고정 길이 fingerprint 문자열이다. 같은 입력의 document/usage 및 publication identity/fingerprint 불변 시험이 유지된다. |
| A08 | `describe`의 지원 정보는 additive다. authored expression publication 차단을 해제하지 않았다. DamageExpr 공개 거부 시험, 기존 HealExpr catalog 거부 시험, 이번 독립 HealExpr CLI 거부를 함께 확인했다. |
| A09 | CLI와 TrialSession이 기존 explain 경로를 사용한다. TrialSession의 direct/delegated document와 usage 비교 시험을 읽었다. boss graph·공간 preview의 제품 구현은 변경하지 않았다. |
| A10 | descriptor에 numeric amount/trace/final_damage를 만들지 않는다. 실제 CLI에서 `performed=false`와 해당 필드 부재, 기존 effect의 literal 60 보존을 확인했다. |

새 설명 경로에 compile/evaluate 호출, runtime frame 생성, action/capture 발급, RNG/resource/target 변경은 없다.
catalog 전체 순회도 없다. `control::metadata_size`는 기존의 무할당 encoded-shape 계량이며,
이 usage를 실제 allocator peak 또는 게임 전투 메모리로 해석하지 않는다.

## 3. 검증 근거와 재실행 구분

앞선 [독립 시간 확인](../wp02_d07_test_timing_001/README.ko.md)에서 직접 실행한 전체 시험은
**1,533 passed / 0 failed, exit 0, 105.35초**였다. 이번 target과 그 실행의 전후 338개 source digest가
정확히 같고 로그 SHA도 일치함을 재확인했다. 소스 변경이 없으므로 전체 시험을 다시 반복하지 않았다.
이 결과는 새 빌드 시간을 포함한 일반 성능 보장이 아니다.

구현자의 최종 영수증 10개는 로그 SHA, exit, source 전후 일치로 확인했다.
authoring 110개, value_expr 88개, 자체 전체 target 1,533개, check/clippy/format 및 CLI/probe 결과는
해당 동결 소스의 증거다. 이번 검토에서 focused/check/clippy를 새로 실행한 것으로 표시하지 않는다.
상세 연결은 [HANDOFF_RECEIPTS_CHECK.json](HANDOFF_RECEIPTS_CHECK.json)에 있다.

이번에 직접 추가 실행한 것은 다음 두 CLI 확인이다.

- 현재 공식 fixture의 Effect explain: exit 0. 동결 CLI 문서와 JSON 구조가 동일하며 source/site,
  fingerprint, publication, 미평가 상태와 literal 보존을 확인했다.
- 공식 fixture를 임시 디렉터리에 복사해 첫 정상 Heal을 같은 값의 `HealExpr`로 바꾼 공개 로더 입력:
  `content_capability_unsupported`로 거부됐다. 원 fixture와 제품 코드는 변경하지 않았다.

명령/결과는 [CLI_RECHECK.json](CLI_RECHECK.json), 출력 대조는 [CLI_DOCUMENT_CHECK.json](CLI_DOCUMENT_CHECK.json),
재현용 입력 변경은 [HEAL_PUBLICATION_MUTATION.json](HEAL_PUBLICATION_MUTATION.json)에 있다.
임시 입력 디렉터리는 정리했으며, base fixture 경로·변경문·전후 해시를 남겼다.

인계의 과거 실패·중단은 [FAILURE_HISTORY](../../segments/wp02_d07_static_explain_001/FAILURE_HISTORY.md)에 보존되어 있다.
최초 단일 스레드 중단은 로그가 없으므로 당시 진행 지점이나 성공 여부를 추정하지 않는다.
앞선 독립 실행의 소켓 EPERM과 승인 환경의 성공을 구별한다. PC36/R05는 이번 변경에 runtime 경로 변화가 없어
지시서대로 재측정하지 않았다. 약 9MB peak 경고 정책이나 다른 한도를 변경하지 않았다.

## 4. 종료와 다음 단계

**D07-A 부분 범위 PASS.** 구현 인계서 제목 아래의 '동결 준비'는 작성 시점 문구이며,
현재 실제 target 동결·검증 상태는 위 manifest와 검증 원자료로 확인했다. 동결 인계서를 고치지 않는다.

다음 작업은 **D07-B의 실제 읽기 전용 평가 문맥과 amount/trace 관측 경계를 코드로 설계하고 구현 지시서를 작성하는 것**이다.
Snapshot release frame과 Live occurrence frame, capture identity, runtime 소비와 같은 evaluator 사용,
판단 불가 수치의 Indeterminate 표현을 구체화해야 한다. 정적 설명에 임의 EvalFrame이나 예상 피해를 붙이지 않는다.

D07-C/WP03의 effective stat/copy 입력·CertifiedInputFingerprint·기존 저장 호환,
WP04의 public 작성→실행→관측 통합은 남아 있다. authored expression production publication 차단도 유지한다.
이번 검토에서 후속 제품 구현, R05 재개, 실험실 로드맵 전환은 수행하지 않았다.
