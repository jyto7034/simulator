# R06 WP02 EEP-F01~F03 교정 재검토 002

## 판정: 부분 범위 PASS

**추가 수정 요구 없음. EEP-F01~F03은 이번 교정 범위에서 해소된 것으로 판단한다.**
이 판정은 아래 고정본의 세 지적 교정에만 적용하며 WP02/R06/Phase/Goal 완료나 production publication 해제를 뜻하지 않는다.

- 대상: [correction_001/MANIFEST.json](../../corrections/wp02_eep_f01_f03_001/target/correction_001/MANIFEST.json)
- SHA-256: `abe5a96830d8090f77e63fde7c6d7ff79675f701a382437d2492e0c66e9aa60a`
- Goal: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`
- 일자: 2026-09-26 KST.
- 요청: 현재 대화에서 계획·검토, 다른 대화에서 구현. 사용자의 후속 검토 요청에 따라 수행했다.
- 검토자: 이 대화의 assistant. 대상 제품 구현·교정에 참여하지 않았으며, 이전 턴에는 재개 위치 확인 문서만 작성했다.
- 근거: [원 검토 EEP-F01~F03](../wp02_effect_execution_prefix_001_review_001/REVIEW.md),
  [교정 인계](../../corrections/wp02_eep_f01_f03_001/REVIEW_HANDOFF.ko.md),
  [전후 delta](../../corrections/wp02_eep_f01_f03_001/target/correction_001/SOURCE_DELTA.patch).

직접 코드·전후 변경·시험 본문을 검토하고 동결 자료와 성공 로그의 결속을 검사했다.
**이번 검토에서 Rust 시험을 직접 실행한 건수는 0이다.** Linux `cargo`/`rustc`가 설치된 경로를 찾지 못해
재실행 준비를 완료하지 못했다. 아래 시험 결과는 구현자의 고정 증거를 재사용한다.
GEX §6.4의 증거 재사용 규칙에 따라 원 실행의 출처·범위를 유지하며 독립 재실행으로 표시하지 않는다.

## 지적별 판단

### EEP-F01 — 버퍼 할당 전 청구: 해소

`value_expr_runtime.rs::build_value_expr_delivery_capture`의 release target 임시 Vec가 제거됐다.
이미 소유한 target slice의 원소를 읽고, capture Arc slots는 cardinality에 따라 structural admission에 포함한다.
effect ID의 String slots·문자열, action site 문자열도 복제 전에 비용을 계산한다.

result resource ID는 borrowed prepass로 슬롯과 문자열의 보수적 상계를 계산한 뒤 청구하고
`Vec::with_capacity`/clone을 수행한다. `sort_unstable`/`dedup`은 입력 의미를 유지하며
중복 제거 후에도 예약 capacity의 청구를 되돌리지 않는다. sort work를 사전 청구하고 각 scan/clone에서
work·취소를 확인한다. retained admission 자체도 native/authoring work·fault와 연결됨을 확인했다.

`EffectResultPrefix::from_capture`는 settled slots, cumulative resource 배열, site 문자열을 먼저 청구한다.
각 effect의 projection 배열은 `begin_effect_result_projection`이 실제 생성 직전에 한 번 청구하고,
literal 경로의 빈 Arc도 이 admission 이후에 생성한다.
`sim.rs::execute_ordered_skill_effects`의 target별 evaluated amount Vec 역시 `with_capacity`보다
앞에서 checked byte 계산과 admission을 수행한다. 비용 상한 및 누적 미환급 의미는 바꾸지 않았다.

재사용한 allocator·경계 시험의 실제 출력:

- 512 target, 추가 allowance 6,000 bytes: 거부 전 최대 단일 요청 4,096 bytes, 관측 요청 합계 4,568 bytes.
- prefix 필요량 2,296 bytes: 정확한 한도에서 성공. 2,295 bytes에서는 관측 allocation 0으로 거부.
- capture 전체 필요량: 12,249 bytes 성공, 12,248 bytes 거부.

이는 해당 fixture의 사전 청구와 경계 회귀 증거다. 실제 PC RSS 한도나 모든 스킬 조합의 성능 보증이 아니다.
많은 resource ID를 실제 authored catalog에서 수집·중복 제거하는 전체 조합은 별도 allocator 실행 증거가 없다.
해당 경로는 checked 상계·capacity 보존·할당 순서의 코드 검토로 판단했으며 추가 실행을 했다고 주장하지 않는다.

### EEP-F02 — 대상 수 중복 집계: 해소

`build_ordered_skill_effect_commands`는 per-effect 결과에 resolved target 수를 넣지 않는다.
그 호출자인 `execute_ordered_skill_effects`가 consumption 시작에 `targets.len()`을 한 번 기록한다.
다른 effect 결과와 signal 결과는 기존 필드 의미대로 merge한다. `SkillStepResult::merge`의
별도 delivery/반복 결과 합산은 유지됐다. affected builder의 호출 위치도 확인했다.

제품 회귀 시험은 literal 다중 효과 × 두 대상 × 두 consumption에서 각 2, 합계 4를 확인하고,
empty target은 0, Damage와 EarlierEffect HealExpr 혼합에서는 대상 수 1과 실제 HP 복원을 확인한다.
원 검토 probe의 legacy builder와 비교도 성공 기록이 있다. effect 개수에 따라 대상 수가 늘던 경로는 제거됐다.

### EEP-F03 — delivery-local 값을 action 전체 값으로 반환: 해소

`EffectResultPrefix::value`는 현재 effect 앞 frontier가 미완료이면 `NotSettled`,
그 외 `CurrentActionPrefix`도 action-wide witness가 없으므로 `Unavailable`을 반환한다.
delivery-local cumulative를 action 전체 결과로 반환하지 않고 단일 delivery 예외도 추정하지 않는다.

`EarlierEffect`는 source action/delivery, ActionSite, 현재 effect ID, 자기/미래 여부를 검사한 뒤
정확히 앞 effect들만 완료된 frontier인지 확인한다. 유효 0과 witness 부재를 구별하는 기존 시험도 유지됐다.

원 cross-step probe는 불완전한 정상값을 거부하는지 검사하지만 모든 오류를 허용하는 부분이 있다.
이를 단독 근거로 삼지 않았다. 제품 시험 `action_prefix_refusal_rolls_back_only_the_current_candidate`가
정확히 `Unavailable` 오류인지, candidate snapshot이 복원되는지, 이미 확정된 앞 step 피해 7과
HP 73은 보존되고 healing_delta는 0인지 함께 확인한다. effect projection context 복원과
바깥 strike scope 복원 경로도 열람했다.

이 교정은 action-wide witness의 구현 완료가 아니라 부분 구현에서 잘못된 성공값을 내지 않게 하는 교정이다.
후속 R06 구현에서 해당 기능을 연결해야 한다는 최종 요구는 남는다.

## 증거의 현재성과 재사용

검사 방법은 [verify.py](verify.py), 진입/종료 결과는 [entry.json](entry.json), [exit.json](exit.json)에 있다.
manifest 해시, source 333개, records 44개와 연결 artifacts, source/evidence archive 내용,
양방향 delta 복원을 직접 확인했다. 현재 `core_enhanced/src` 아래 Rust 파일 중 manifest에 없는 파일도 없었다.
이는 해당 범위 검사이며 저장소 전체 모든 비소스 파일의 완전성을 주장하지 않는다.

| 검증 | 출처와 판정 범위 |
| --- | --- |
| 제품 all-target 1,487 PASS | `evidence/all_tests_002.*` 재사용. lib 1,435 + integration 52. 원시 로그에서 건수를 다시 집계했으며 0-test binary를 성공 시험 수에 더하지 않음 |
| 검토·경계 probe 7 PASS | `evidence/review_probes_002.*` 재사용. 원 검토의 5개 test body가 유지됐음을 확인. 기존 제품 시험을 호출하는 control도 포함하므로 독립 coverage 7개로 확대하지 않음 |
| all-target check / Clippy | `check_final.*`, `clippy_final.*` exit 0 재사용. 기존 `never_loop`, `eq_op` allow 두 개 유지; warnings-free 주장이 아님 |
| 변경 영역 format | `changed_format.*` 재사용. 기존 sim 범위 밖 포맷 차이는 별도 |
| PC fixture | `pc_001.*` 및 비교 기록 재사용. 36조건 중 31완료/기존 5거부. 새로운 순차 expression 조합 전체나 새 PC 플랫폼 검증은 아님 |
| R05 guard | `r05_guard_001.*`와 summary 재사용. concurrent peak 8,945,386 / 기존 cap 9,030,188 bytes. 상한 변경 없음 |

위 evidence 경로는 [교정 evidence 디렉터리](../../corrections/wp02_eep_f01_f03_001/evidence/) 기준이다.
모든 성공 receipt의 시작/종료 source 333개 해시가 대상과 일치한다.
receipt 실행기는 격리 실행 때도 **원 작업공간**의 해시를 기록하므로 이것만으로 격리 소스 일치를 주장하지 않았다.
격리 준비·최종 검사 스크립트, `isolate_final_identity.json`과 보존된 두 probe 파일의 해시를 추가로 확인했다.
당시 격리 디렉터리는 현재 없어 그 디렉터리를 직접 재검사한 것은 아니다. test-only 두 hook의 검증은
고정된 최종 검사 증거를 재사용한 것이며, 현재 새 격리 사본에서 probe를 실행하지 않았다.

## 중단된 이전 재검토와 이번 환경의 한계

이전 `review_001/product_all_targets_002.log`의 **1,370 PASS / 65 실패**는 그대로 실패 이력이다.
실패 본문 64개가 ReadOnlyFilesystem을 명시하고, 나머지 하나는 debug trace 파일 부재다.
trace 출력은 CARGO_MANIFEST_DIR 아래 `target/combat_traces`를 사용하므로 환경 영향 가능성이 있지만,
한 건의 정확한 실패 원인을 이번에 재현·확정하지 않았다. 65개를 PASS로 재분류하지 않는다.

이번에는 target archive에서 새 격리 사본을 만들고 기존 supplemental compile-time 문서 두 개의 해시도 확인했다.
그 후 `rustc --version` 확인이 FileNotFoundError로 종료되어 cargo 빌드·시험은 시작하지 않았다.
현재 Linux cargo/rustc 검색 결과는 entry/exit에 남겼다. Windows 도구로 Unix 포함 시험을 대체하거나
새 toolchain을 설치하지 않았다. 따라서 현재 환경의 전체 시험 재현 성공을 주장하지 않는다.

이번 부분 PASS는 현재 코드 검토와 동일 고정 source의 유효한 기존 성공 증거에 근거한다.
나중에 source·시험 입력 또는 관련 실행 환경이 바뀌면 영향받는 검증을 다시 수행해야 한다.

## 다음 작업

현재 대화에서 다음 WP02 구현 계획을 구체화할 수 있다. 잔여 책임은 action 전체 완료 prefix,
release 이전 결과 Snapshot, ResolvedTargets 집계 및 남은 수치 preview/explain 연결이다.
WP03/04 통합과 최종 감사도 남아 있다. publication 차단은 코드에서 유지됨을 확인했다.

다른 대화의 구현은 이 검토 target을 선행 기준으로 삼고 실제 mutation 직전 baseline을 새로 고정한다.
기존 단계검토 경계를 유지하여 구현·자체 검증 후 별도 target과 인계를 만들고 현재 대화의 검토로 돌아온다.
이 보고서 자체는 다음 구현 실행 요청이 아니며 제품 코드·기존 target·과거 판정을 변경하지 않았다.
