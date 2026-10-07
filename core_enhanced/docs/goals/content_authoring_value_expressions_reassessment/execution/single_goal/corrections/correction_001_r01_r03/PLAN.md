# R06 correction 001 R01–R03 교정 PLAN

Current state: `자체 검증 완료 / correction_002 동결 입력`

## Identity와 허용 범위

- Handoff / Goal: `HA-CAVER-P2G60-001 / GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`
- Input review: `reviews/correction_001_review_001/REVIEW.md`, verdict `수정 필요`, findings R01–R03.
- Frozen predecessor: `R06_CHECKPOINT_001_CORRECTION_TARGET_001`, manifest SHA-256 `87feab8cdfc20b43d7c49c2f6238bd1e684cb7e1daf4f993efe2f193f01dc45e`.
- Pipeline: `2026-09-13-gex-r7`.
- 허용 범위는 R01/R02 제품 코드 교정, R03 보존 절차 보강, 자체 검증, 새 target 동결과 재검토 인계까지다.

기존 checkpoint, correction 001 target/baseline, 두 review 디렉터리와 그 증거는 수정하지 않았다. 다음 R06 기능, authored catalog/battle 연동, GEX-20/30, WP/Phase/Goal 완료는 수행하지 않는다.

## 단계검토 결과

1. **지침·입력 확인** — pipeline과 재감사 보고서, 같은 디렉터리의 probe·실행 로그·entry/exit currentness, predecessor handoff/manifest를 읽었다.
2. **진입 currentness** — predecessor target manifest의 detached hash, source 311개, records 10개, evidence 30개, before/after archive를 확인했다. source와 current target record에는 새 drift가 없었다. reviewer verifier의 유일한 실패는 이미 보고된 과거 PLAN 참조와 현재 live PLAN의 불일치였고 R03으로 분리했다.
3. **수정 전 baseline** — `baseline/pre_fix_002`에 source 311개의 실제 bytes, 구현 전 이 디렉터리의 변경 예정 문서 3개, predecessor records 10개, review inputs 11개를 별도 archive로 고정했다. manifest SHA-256은 `96eb47695ea346627b5c0c6aedbd75953cf496353ad00b03b04d37bd80232bda`다.
4. **수정 전 재현** — reviewer probe에서 R01은 8 pass / 2 fail / 1 ignored, R02 격리 probe는 2 MiB worker native stack overflow/SIGABRT를 재현했다. deep probe는 `ManuallyDrop`로 입력 AST 파괴 재귀를 분리했다.
5. **구현 검토** — compile/evaluate 공유 work 함수와 authored 정적 비용 합성, runtime-dynamic 비용 표기, bottom-up execution shape 계산을 적용했다. 기본 예산, 표현식 계산 순서, 지원 depth, 공개 평가 결과와 fingerprint 형식은 바꾸지 않았다.
6. **경계·회귀 검토** — 1-member 7/7, 15개 authored 조합, empty/long identity·owner·resource, 전체/group/tile grouping, 중첩 집계, work 및 shape scratch의 직전·동일·초과 경계를 검사했다. 기본 depth 64/65와 확장 depth 512/2,048를 검사했다.
7. **전체 자체 검증** — value_expr 22 tests, predecessor correction probe 7 tests, 재감사 probe 10 pass / 1 ignored와 별도 deep pass, all-target check/test, focused rustfmt, 기존 두 allow를 명시한 Clippy, 동결 전 fingerprint 3종 비교를 통과했다.
8. **보존 검토** — R03 원본 4,460 bytes를 Codex session의 해당 apply-patch 결과에서 찾아 immutable evidence로 복구했다. predecessor baseline은 수정하지 않고 successor recovery record로 연결했다.

## 구현 판단

### R01 — 정적 비용과 실행 비용의 같은 정의

- `aggregate_member_scan_work`와 `aggregate_sort_work`를 compile/evaluate가 공유한다.
- raw member count, identity, primary value payload, owner key, associated resolved value payload, canonical sort/group/reduction을 checked arithmetic으로 계산한다.
- `AuthoredMembers`는 compile 시 알려진 전체 preprocessing 비용을 plan work에 합성하고 중첩 aggregate는 member 반복 횟수를 곱해 합성한다.
- `PlanCost`는 `static_work`, `runtime_dynamic_work`, `worst_case_work`를 구분한다. frame/resolved payload가 없는 정적 plan의 `static_work == worst_case_work`는 같은 meter work에서 실행 가능한 상한이다. 미확정 payload가 있는 plan은 dynamic flag와 승인된 `max_work` ceiling을 표시하며 구조 비용을 전체 worst case로 오표시하지 않는다.
- frame preprocessing은 plan이 요구하고 frame에 존재하는 non-member reference만 순회·계측한다. 관계없는 frame data를 표현식 처리 비용으로 계산하지 않는다.

### R02 — native recursion 제거

- recursive `segment_shape`를 segment 생성 의존 순서의 역순 bottom-up 계산으로 교체했다.
- `Vec<Option<ExecutionShape>>` scratch의 보수적 저장 공간을 할당 전에 compile retained budget에 admission한다.
- 동시 value stack, 누적 stack storage, task frame 요구량 계산은 유지한다. 새 depth hard cap이나 worker stack 증가는 도입하지 않았다.

### R03 — 과거 PLAN 실제 bytes 복구와 세대 분리

- 요구된 SHA-256 `661dd456d923e323cfd5704dd520e322251ba9a603303a4b73c295e6c13412c7`, 4,460-byte 원본을 `evidence/r03/HISTORICAL_PLAN_PRE_FIX_001.md`에 보존했다.
- provenance는 2026-09-13 Codex session JSONL line 1075의 PLAN apply-patch 결과다. 원시 JSONL event 자체도 별도 보존했고 추출한 content의 byte/hash를 검증했다.
- 기존 `baseline/pre_fix_001`은 손대지 않았다. 당시 baseline의 `historical_r03_plan_bytes_preserved=false` 사실도 바꾸지 않는다. 새 recovery record가 후속 세대에서 누락 bytes를 보완한다.
- successor verifier는 historical snapshot을 immutable recovered copy와 provenance에서 검사하고, current live PLAN은 predecessor target record로 따로 검사한다.

## 계약과 종료 경계

typed AST → compiled immutable plan → evaluation, 정상 operand/group 순서, 중복 제거, 단위, signed rounding, checked overflow, legacy narrowing과 fingerprint 의미를 유지한다. 예산을 늘리거나 runtime charge를 제거하지 않았다.

이 문서와 검증 evidence는 새 target 생성 입력이다. 새 target을 read-only 검증하고 handoff에 manifest hash를 연결하면 상태는 `교정본 재검토 준비 완료 / 재검토 요청 대기`가 된다. 그 지점에서 mutation과 다음 구현을 중단한다.
