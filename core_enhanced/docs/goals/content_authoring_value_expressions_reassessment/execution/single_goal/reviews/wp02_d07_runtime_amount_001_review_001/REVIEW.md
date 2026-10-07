# R06 WP02 D07-B — 단계검토 001

2026-09-28 · **수정 필요 (CHANGES_REQUESTED)** · D07-B 부분 범위 PASS 보류.

실제 평가 결과의 최근 1건 보관, 공통 numerical helper, active/last-terminal 조회와 정상 CLI 연결은 확인했다.
그러나 CLI 인자 누락 panic, 새 legacy 관측 배열의 사전 admission 누락, 필수 관측 회귀의 증거 부족이 있다.
제품 구현과 동결 자료는 수정하지 않았다. 별도 구현 대화의 교정과 새 target 재검토가 필요하다.
이 검토는 계획 담당자의 별도 읽기 전용 단계검토다. 설계 참여까지 배제하는 엄격한 독립 완료 감사라고 주장하지 않는다.

## 1. 대상과 직접 검증

- [구현 지시서](../../handoffs/WP02_D07_B_IMPLEMENTATION_REQUEST_20260927.ko.md)
- [인계서](../../segments/wp02_d07_runtime_amount_001/REVIEW_HANDOFF.ko.md)
- [동결 manifest](../../segments/wp02_d07_runtime_amount_001/target/d07_runtime_amount_001/MANIFEST.json)
- Manifest SHA-256: `7db52e95938de08131f352f7137be8196c5099b150d1b52874ee1d2ae0315f9f`
- after 340개 digest: `ad8021359785e365c3f9560268bcef2a95205495f9e33a22314c3f97743480c4`
- 제품 336개 digest: `6b5e0291b1b6f936c277e2138bc2c2adbbeaaa2b3df5c1b4d5bf31b5acff6807`

[진입](ENTRY_CURRENTNESS_AND_RESTORATION.json)·[종료](EXIT_CURRENTNESS_AND_RESTORATION.json)에서
338→340/340→338 byte 복원과 현재 source, 동결 기록 163개의 일치를 확인했다.
소스 변경 45개와 추가 2개 중 다수 변경은 formatting이며, 관측·실행·조회·CLI 관련 기능 변경을 중심으로 검토했다.

직접 재실행한 검증:

- `cargo test -p game_core --lib d07_b --offline --locked`: **5 passed / 0 failed**, exit 0.
  [집중 시험 로그](FOCUSED_RECHECK.log). 실행 환경은 `CARGO_HOME=/tmp/wp02-cargo-home`,
  `CARGO_TARGET_DIR=/mnt/f/work/simulator/.cache/d07-runtime-amount-product-target`다.
- `cargo build -p game_core --bin author_content --offline --locked`: exit 0. [빌드 로그](CLI_BUILD.log).
- 실제 외부 pack/scenario의 CLI 관측 on, trace off/on: 모두 exit 0, 각 amount 문서 4개,
  출력 byte 동일 및 동결 정상 출력과 동일. [실행 명령·결과](CLI_VALID_RECHECK.json).
- 필수 요청 파일이 없는 CLI: `simulate` exit 101 panic, `explain` exit 2 입력 오류.
  [재현 명령·stderr](CLI_MISSING_ARGUMENT_REPRO.json).

전체 시험 1,538개와 authoring 112/value_expr 91/check/clippy/PC36/R05는 이번 검토에서 다시 실행하지 않았다.
원자료 13개 SHA와 시험 요약 건수를 직접 대조했다. [대조 결과](RECEIPTS_CHECK.json).
이 파일의 exit는 구현자 보고 값이며 이번 재실행 exit와 구별한다. 전체 시험 통과를 아래 누락된 신규 조건의 증명으로 쓰지 않는다.
R05 concurrent peak 단일 초과는 기존 합의대로 **경고**다. 이것은 아래 수정 요구의 이유가 아니다.

## 2. 수정 요구

### D07-B-F01 · P2 · simulate의 필수 REQUEST 누락이 panic으로 바뀜

위치: `core_enhanced/src/bin/author_content.rs:58`, `:85`.

`Arguments::parse`가 `simulate`에 대해서는 `args.len() != expected` 검사를 건너뛴다.
따라서 `simulate unused-root unused-selection`은 3개 인자로 입구를 통과하고
`args[3]`에 접근해 `index out of bounds` panic, exit 101이 된다. 실제 파일을 읽기 전 재현된다.
선행 구현은 해당 길이를 거부했으므로 이번 옵션 parser 변경의 회귀다. 정상 정책은 typed CLI 입력 오류/exit 2다.

교정: operation별 필수 positional 인자 수를 옵션 순회/배열 인덱싱 전에 확인한다.
simulate 3개 인자, trace path 누락, amount/trace 단독·조합·순서, 중복/unknown/과도한 인자를
실제 CLI로 확인한다. 이 결함을 감추는 catch-panic이나 임의 요청 파일 기본값은 추가하지 않는다.

### D07-B-F02 · P2 · 새 legacy 관측 배열에 allocation 전 retained admission이 없음

위치: `core_enhanced/src/game/battle/core/sim.rs:3019`.

관측이 켜진 catalog legacy Damage/Heal 경로는 `Vec::with_capacity(targets.len())`를 곧바로 호출한다.
동일 함수의 expression 경로(`:2960`)에는 `targets.len() * size_of::<RuntimeAmount>()`를 checked 계산한
`admit_retained_growth`가 먼저 존재하지만 새 legacy 분기에는 대응 검사가 없다.
이 배열은 모든 대상의 계산값을 command 생성까지 유지하므로 대상 수에 비례하는 실제 추가 소유 메모리다.

나중의 `commit_amount_evaluation`(`amount_observation.rs:203`)은 battle retained basis와 마지막 record의
extent를 다룬다. 해당 지역 `values` Vec는 그 계산에 포함되지 않으며, 이미 일어난 할당의 사전 검사도 아니다.
따라서 §5.5/B11의 새 관측 소유권 사전 admission 요구를 충족하지 못한다.
이는 OOM을 관찰했다는 판정이나 전체 런타임에 한도가 없다는 뜻이 아니다. 누락된 구체적 할당 경로에 대한 source finding이다.

교정: 두 경로가 같은 checked 배열 크기 계산과 allocation 전 admission을 사용하게 한다.
한도를 늘리지 말고 관측 on legacy 다중 대상에서 정확 경계/한도 부족의 typed 거부와 accepted 상태 보존을 검증한다.
이미 존재하는 guard를 재사용하며 새 전역 메모리 정책을 만들지 않는다.

### D07-B-F03 · P2 · B01–B20 전체 PASS를 뒷받침하지 않는 관측 시험

위치: [자체 판정표](../../segments/wp02_d07_runtime_amount_001/evidence/B01_B20_MATRIX.json),
`value_expr_runtime.rs:7190`, `authoring/trial.rs:2146` 및 연결된 기존 회귀.

확인된 차이:

- **B02/B03/B04:** V17의 Snapshot은 legacy 상수 42이고 Live는 `Const(11)`이다.
  실제 소비·query 연결의 증거는 되지만 release/consume 이후 바뀌는 Attack/HP/resource/result를
  올바른 순간의 입력으로 고정했다는 증거는 아니다. 연결된 기존 Snapshot/Live 시험은 새 관측 설정을 켜고
  record→preview/explain까지 검증하지 않는다. 기존 runtime 의미의 회귀와 신규 관측 회귀를 구별해야 한다.
- **B05:** 기존 관계 집계 시험은 runtime의 Side/Employee/EquippedSlot·필터를 확인한다.
  신규 record가 복수 aggregate view/member trace를 보존하고 두 query로 재생하는지까지 확인하지 않는다.
- **B11:** 판정표는 코드와 기존 전체 회귀를 인용한다. 새 record/교체 peak/terminal 보관/clone 및 숫자 query의
  work/result 경계와 긴 member trace 중간 취소를 실행한 대응 시험은 제시되지 않았다.
  특히 last-terminal은 `Arc<TerminalCapsule>`이고 일반 Arc clone visitor는 핸들만 센다.
  active battle의 record extent 가산만으로 terminal owner의 retained 처리까지 입증하지 말고 실제 경로를 확인해야 한다.
- **B18:** 기존 i128 시험은 정적 설명의 상수다. 신규 runtime 기록의 i128 중간 trace가 API/CLI typed encoder를
  통과한 원문 증거가 없다. 최종 amount는 narrowing 범위 안이고 중간 trace만 큰 정수인 경우를 사용할 수 있다.
- **B01/B06–B10/B12–B14:** 5개 시험은 Damage(-7), zero 두 대상, 후속 effect 실패 후 빈 관측 복원,
  일부 unavailable/Exact, public last-terminal 조회와 receipt/work 보존을 확인한다.
  Heal 경계, piercing 순서, 기록 교체 후 이전 Exact, 손상된 record binding,
  이전 성공 record가 있는 상태의 실패·취소 복원, 다음 전투의 terminal 교체, trace 포화 등은
  기존 시험 이름이나 단일 Arc 구조만으로 일괄 PASS 처리할 수 없다.

교정: 기존 시험을 무조건 복제하지 말고, 실제 관측을 켠 fixture에 필요한 assertion을 추가하거나
공통 fixture로 복수 조건을 검증한다. 조건마다 **runtime 기존 회귀 / 신규 관측 검증 / 코드 근거 / 미검증**을 구별한다.
새 기능을 입증하지 않는 과거 성공을 재사용했다고 표시하지 않는다. 설계상 실제 미지원인 부분이 드러나면
해당 코드를 교정하고 검증해야 하며, 필수 조건을 조용히 제외하거나 판정표 문구만 바꿔 닫지 않는다.

## 3. 확인한 구현과 판정 한계

다음 연결은 소스 및 정상 실행에서 확인했다.

- host opt-in 기본 false, trace 옵션과 독립. 기존 publication 차단 유지.
- runtime의 identity 검사와 frame 선택 뒤 공통 `evaluate_compiled_amount`를 사용한다.
  query는 live action을 새로 만들지 않고 기록된 frame을 재생한다.
- `PendingAmountEvaluation`은 대상별 최신 후보 하나이며 effect 처리·projection·prefix settle 후 commit된다.
  `MandatoryObservation`의 최근 immutable Arc와 ordinal은 candidate 소유다. 기존 transaction rollback에 포함된다.
- private frame/plan은 일반 receipt serde에서 제외된다. 별도 query의 출력은 typed encoder를 사용한다.
- active/last-terminal 조회와 not-retained 등 unavailable 구분, 정적 explain의 `performed=false`를 유지한다.
- 미래 DamageExpr/mixed 수치를 내부 Indeterminate로 구분하며 과거 기록을 미래 예측에 쓰지 않는다.

이 설계 방향을 폐기할 사유는 찾지 않았다. 다만 보관 프레임의 모든 동적 조합·취소·비용 경계가 입증됐다는 뜻은 아니다.
F03 때문에 인계의 B01–B20 전체 PASS를 수용하지 않는다. F01/F02의 교정만으로 D07-B를 자동 PASS 처리하지 않는다.

## 4. 다음 작업

별도 구현 대화에서 F01/F02 교정과 F03 검증 보강을 수행하고, 이번 target을 predecessor로 한 successor를
동결한 뒤 재검토한다. 교정 범위와 중단 지점을 담은 지시서는 별도 요청에서 작성할 수 있다.
교정 중 계획·시도 이력·현재 판단은 기존 실행 문서에 계속 남기며 기존 동결 기록은 수정하지 않는다.

D07-C/WP03, WP04, 전체 R06 완료와 authored expression production 공개는 진행하지 않았다.
판정: **D07-B 수정·검증 보강 후 재검토 필요**.
