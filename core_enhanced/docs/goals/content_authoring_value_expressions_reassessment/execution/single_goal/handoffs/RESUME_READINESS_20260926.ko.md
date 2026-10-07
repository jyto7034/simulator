# R06 재개 확인 — 계획·검토와 구현 대화 분리

확인일: 2026-09-26 KST. 대상: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`.

사용자 요청: R06을 계속 진행하되 **구현 계획과 검토는 현재 대화, 제품 구현은 다른 대화**에서 수행한다.
이번 확인은 다음 작업을 정하기 위한 기록 조사·교정 delta 및 관련 코드 열람·동결 검증이다.
독립 단계검토의 최종 판정이나 새 구현 승인으로 해석하지 않는다.

## 현재 기준점

가장 최근 구현 결과는 [EEP-F01~F03 교정 인계](../corrections/wp02_eep_f01_f03_001/REVIEW_HANDOFF.ko.md)다.
대상은 [correction_001 manifest](../corrections/wp02_eep_f01_f03_001/target/correction_001/MANIFEST.json),
SHA-256 `abe5a96830d8090f77e63fde7c6d7ff79675f701a382437d2492e0c66e9aa60a`다.

직접 실행한 명령:

```sh
python3 core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_eep_f01_f03_001/freeze.py verify
```

exit 0. 현재 파일 333개와 고정본 일치, 기록 44개 및 연결 artifact hash 검사,
before→after와 after→before delta 복원 일치를 확인했다.
교정 제품 변경은 `sim.rs`, `value_expr_runtime.rs` 두 파일이다. 이것은 지정된 manifest 범위의 검증이며
저장소 전체의 신규 파일·미추적 파일 완전성이나 의미 검토 PASS를 뜻하지 않는다.
branch는 `cleanup/simulator-dirty-worktree-20260803`, HEAD는 `f318414fb0156a67e04c5479000c2a2683fc0c09`다.
다수의 기존 dirty 변경이 있으므로 HEAD를 구현 기준선 대신 사용하지 않는다.

교정 내용은 할당 전 예산 청구, delivery당 대상 수 1회 집계,
action 전체 결과를 보증하지 못하는 `CurrentActionPrefix`의 명시적 거부다.
인계서의 제품 시험 1,487건 통과는 구현자 기록이며 이번 확인에서 재실행한 결과가 아니다.

## 당장 이어갈 작업: 교정 재검토 종료

[기존 재검토 디렉터리](../reviews/wp02_eep_f01_f03_correction_001_review_001/)에는
동결 검사·독립 archive 검사·시험 시도 기록이 있지만 `REVIEW.md`와 최종 판정은 없다.
`product_all_targets_002.log`에는 **1,370 통과 / 65 실패**가 남아 있다.

- 실패 64개는 실패 본문에 `ReadOnlyFilesystem / Read-only file system`을 명시한다.
- 나머지 1개는 `live_ron_defense_route_playable_path_runs_to_combat_result`의 debug trace 파일 부재다.
  trace는 `CARGO_MANIFEST_DIR/target/combat_traces`에 기록된다. 환경 영향 가능성은 있으나 로그만으로
  동일 원인이라고 확정하거나 PASS로 바꾸지 않는다.
- 최초 `product_all_targets_001`은 exit 101이다. 이후 보충한 compile-time 문서 입력 두 개는
  `supplemental_test_inputs.json`에 별도 기록되어 있다. 이들을 frozen source로 잘못 표시하지 않는다.
- 당시 격리 경로 `/tmp/r06-eep-independent-mb8k5yse`는 현재 존재하지 않는다.

따라서 다음 검토는 현재 대화에서 다음을 수행한다.

1. 위 exact correction target을 새 검토 pass에 연결한다. 기존 실패·실행 로그는 보존한다.
2. [원 지적 EEP-F01~F03](../reviews/wp02_effect_execution_prefix_001_review_001/REVIEW.md),
   [교정 delta](../corrections/wp02_eep_f01_f03_001/target/correction_001/SOURCE_DELTA.patch),
   비용 소유권과 frontier/집계의 실제 호출 경로를 대조한다.
3. 필요한 재실행은 새 격리 사본과 전용 build 디렉터리에서 수행한다. 임시 fixture 생성 위치와
   debug trace 위치가 쓰기 가능한지 확인한다. 샌드박스에 막힌 명령은 정해진 승인 절차를 따르며,
   시험 삭제·제품 코드 수정으로 환경 실패를 숨기지 않는다.
4. 동일 입력·source hash에 결속된 기존 성공 증거는 재사용할 수 있다. 재현이 필요한 실패와
   선청구·다중 효과 대상 수·cross-step prefix 거부·rollback probe에 검증을 집중한다.
5. 현재 파일 identity를 다시 확인하고 부분 범위 PASS / 수정 필요 / 판정 불가 및 근거를 남긴다.
   이는 WP02나 R06 전체 완료 감사가 아니다.

## 재검토 이후 구현 계획 후보

현재 소비자 코드와 인계서에서 확인되는 잔여 책임은 아래와 같다.
순서와 세부 경계는 교정 재검토 결과 및 해당 호출 경로의 상세 설계를 바탕으로 확정한다.
이 표 자체를 다음 구현 대화의 실행 지시로 사용하지 않는다.

| 후보 | 계획에서 확정할 책임·직접 검증 |
| --- | --- |
| WP02 action 전체 완료 결과와 Snapshot | action/step/delivery occurrence·완료 frontier를 결속한 bounded witness. 앞 step 실제 피해 7 + 현재 step 피해 3이면 같은 action prefix는 10. release 이전 완료 결과 Snapshot은 이후 결과에 따라 변하지 않음. 미완료·다른 attempt·evicted는 오류, 유효 0은 유지. candidate 실패 시 witness도 복원. D03/D06이 기준 |
| WP02 실제 대상 집계 | executor가 확정한 유한 target set의 identity 중복 제거 및 owner/tile grouping. 미확정 release 대상의 Snapshot은 거부. tile0의 5/3, tile1의 2에서 그룹별 Max 후 Sum=7, 그룹별 Count 후 Max=2. D04가 기준 |
| WP02 남은 수치 소비자 | runtime/preview/explain의 공통 evaluator와 명시적 frame. context 부재를 0/false로 바꾸지 않고 prospective와 committed Snapshot 구분. D07이 기준 |
| WP03 | Attack/SpellPower·무기 수치 단일 합성, 실제 사본별 enhancement, cadence, old V7 호환·nonzero 저장 및 fingerprint 통합. D05/D07이 기준 |
| WP04 | 외부 RON부터 공개 실행까지 연결, 27 acceptance와 V01–V21 증거·문서·비용·회귀 마감. 최종 감사는 별도 |

원 설계는 [CODE_REVIEW_003 D01–D10](../../../CODE_REVIEW_003.ko.md),
최종 요구는 [Goal Set](../../../P2G_50_GOAL_SET_001.ko.md)이 소유한다.
현재 `CurrentActionPrefix`의 typed refusal과 `ResolvedTargets`의 UnsupportedMoment는
부분 구현 경계다. R06 최종 요구에서 이 기능을 삭제한 것으로 해석하지 않는다.
`content_capability_unsupported` publication 차단은 여전히 존재한다.

다른 대화로 넘길 최종 구현 계획에는 선행 검토 판정·exact target, 변경 책임과 데이터 수명,
처리 순서·오류/rollback·비용 소유권, 직접 시험 기대값, 변경 전 baseline과 종료 target 규칙을
연결한다. 구현 대화는 구현·자체 검증 후 동결하여 현재 대화의 검토로 돌아온다.
기존 단계검토 옵션을 유지하며, 구현 중 의미 있는 책임과 관련 검증이 닫히는 시점에 중단한다.

## 오래된 안내를 읽을 때

`ROADMAP_STATUS.md`의 “계획 인계 완료 / 구현 요청 대기”, 상위 실행 `PLAN.md`의 최초 커널 상태,
[기존 구현 인계서](IMPLEMENTATION_RESUME_20260926.ko.md)의 frame/capture 상태는 최신 구현 위치가 아니다.
이미 구현된 순차 effect 실행·EarlierEffect 소비·오류 전파를 처음부터 다시 구현하지 않는다.
이번에는 새 확인 기록만 추가하며 옛 동결 문서·manifest·verdict·제품 코드를 변경하지 않았다.
