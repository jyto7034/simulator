# NEXT-0005 — P2G-40 필수 검증기 입력 호환성 문제

P2G-40은 **미완료**다. [계획 후보](P2G_40_PHASE_PLAN_CANDIDATE_001.ko.md)는 작성했으나, 표준 생성기가 exact 현재 Freeze/selector를 해석하지 못해 필수 Receipt를 발행하지 않았다. 마지막 유효 선택은 `WR-CAMEC-P2G20-001` + `FPF-CAMEC-P2G30-001`이다. current Phase Plan·Goal Set은 없다.

## 1. 실제 시도와 영향

[실행 원본](P2G_40_VALIDATOR_ATTEMPT_001.json)에 정확한 argv, 입력 해시, 실제 stdout·exit code와 생성 여부를 기록했다.

```bash
node core_enhanced/docs/tools/validate_p2g40_receipt.mjs --write-receipt \
  --plan core_enhanced/docs/goals/content_authoring_minimal_effect_contract/P2G_40_PHASE_PLAN_CANDIDATE_001.ko.md \
  --freeze core_enhanced/docs/goals/content_authoring_minimal_effect_contract/P2G_30_FREEZE_001.json \
  --receipt core_enhanced/docs/goals/content_authoring_minimal_effect_contract/P2G_40_PHASE_PLAN_VALIDATION_RECEIPT_001.json
```

실제 node process exit `1`, generator `p2g40-receipt-generator/1.1.0`, status `not_written`, validator result `failed`다. 오류는 `REF_FREEZE`, `FREEZE_MEMBERS`, 작업 묶음별 `WP_SELECTOR_EMPTY` 3건이다. 도구가 센 Freeze member는 0이지만 JSON 원본에는 6개가 있다. 도구는 나머지 구조에서 WP 3·edge 4·cyclic SCC 1·Atomic Group 1·final node 2·Phase 1·audit row 1·integration row 6을 읽었다. 이 count는 전체 검사 통과를 뜻하지 않는다.

Receipt 파일은 생성되지 않았다. 따라서 같은 세 입력의 기본 검증 모드는 실행할 Receipt가 없어 실행하지 않았다. 수동 Receipt·대체 검사 결과로 required gate를 통과 처리하지 않았다. 생성 시 입력 Plan·Freeze·validator bytes는 그대로 보존됐다.

## 2. 원인과 authoritative owner

| 문제 | 실제 근거 | 판정 |
| --- | --- | --- |
| JSON Freeze를 읽지 못함 | [검증기](../../tools/validate_p2g40_receipt.mjs)의 `field`(64), `orderedFreezeMemberBlock`(213), `buildReceipt`(245), `validate`(424)는 Markdown bullet/ordered-member block을 regex로 읽는다. 현재 [Freeze](P2G_30_FREEZE_001.json)는 `artifacts[].ref/payload` JSON이다. | 도구의 미지원 직렬화. 현재 artifact가 실제로 없는 것이 아님. |
| stable selector를 이름 패턴으로 가정 | 같은 검증기의 `deriveElementId`(199)는 PER ID에서 PE ID를 추정하고, WP/primary selector는 `ids(..., "PE")`(488, 525)로 찾는다. 실제 `PER-CAMEC-E01-REVIEWED-001`의 payload Stable Element ID는 `CAMEC-E01`이다. 다른 다섯 요소도 동일한 구조다. | prefix/ID 재작성으로 해결할 Plan 오타가 아님. 명시적 Stable Element ID와 exact frozen member 관계를 해석해야 함. |
| 필수 exit 미충족 | [P2G r2](../../policy_to_goal_pipeline_revised_v2.md) P2G-40 생성·검증/종료 조건은 표준 생성기의 Receipt와 기본 검증 `exit 0 / passed / failures empty`를 요구한다. | P2G-40 complete/current 선택 불가. one-off 문서 검사는 이 gate의 대체가 아님. |

P2G §5.4/§8.0/§8.3은 Bound Freeze와 stable Element ID의 owning relation 및 producer-issued exact Ref 보존을 요구한다. 기존 `CAMEC-E..`를 다른 ID로 개명하거나 JSON Freeze를 가짜 Markdown Freeze로 대체하면 이 근거가 달라진다. 사람용 Markdown projection만 추가해도 `PE-` 가정과 revision-name 추론은 해결되지 않으며, 생성 Receipt가 원래 exact JSON 입력을 검증했다고 할 수 없다.

변경되지 않은 현재 코드·정책·Freeze를 stale로 만들 finding은 없다. 새 정책 질문·Change Declaration·P2G-10/20/30 재실행을 임의로 만들지 않았다. classification은 **tooling input contract / unsupported format**, affected claim은 P2G-40의 exact Freeze/selector 검증과 passed Receipt/current Phase Plan 발행이다.

## 3. 승인 범위 안에서 마친 작업

계획 후보는 Review §3–5의 실제 publication/직접 소비 경계를 따라 WP 3개를 설계했다. 새 schema/metadata compiler와 catalog/runtime의 상호 의존 WP 2개는 하나의 Atomic Group으로 묶고, 새 durable 통합 fixture는 standalone 검증 node로 배정했다. 그 node가 future GEX 감사자에게 시험 구현을 미루지 않도록 IVA 6개를 모두 소유한다. 전체 12개 acceptance의 참조와 6개 primary selector, SCC/partition/Phase 관계를 별도 문서 진단으로 확인했다. 실제 결과는 [단계 결과](P2G_40_RESULT_001.json)가 소유한다.

구현자는 정확한 타입·문법·주소 encoding을 고를 수 있지만 publication·source·취소·저장 수명·지원 의미는 기존 Freeze/Review를 보존해야 한다. 새 API·fixture·Phase 구현은 아직 없다. 기존 CRAR/CWBS 감사와 과거 게임 시험 결과를 이번 실행으로 계산하지 않았다.

## 4. 재개에 필요한 변경 범위

다음은 **검증기 교정 제안**이며 이번 요청에서 구현하지 않았다.

1. `validate_p2g40_receipt.mjs`의 입력 해석이 기존 Markdown과 현재 JSON artifact bundle을 구분하고 exact typed Freeze Ref를 보존하도록 한다. JSON member revision은 해당 Topic의 실제 immutable Policy Element payload와 exact-match하여 Stable Element ID를 얻어야 한다. 누락·중복·타입/Ref 불일치·모호한 member는 fail-closed 처리한다.
2. WP/primary selector는 실제 frozen member의 Stable Element ID로 해석하고, PER 이름에서 임의로 새 ID를 만들지 않는다. JSON/Markdown 혼합을 지원한다면 각 근거 파일의 해시와 exact relation을 Receipt의 입력 근거로 남긴다. 구형 Markdown 처리/기존 Receipt 검증 의미를 보존하고 필요한 companion 변경은 실제 소비 영향 범위에서 판단한다.
3. 현재 Topic의 6개 member/primary coverage가 정확히 검증되는 양성 fixture, member/selector 누락·중복·잘못된 Ref와 입력 변경을 거부하는 음성 fixture, 기존 Markdown 경로의 회귀 검사를 추가한다. 생성·기본 검증·동일 Receipt 재생성 무변경·다른 immutable target 덮어쓰기 거부를 확인한다. 새 Receipt 형식이 필요하면 version·fixture·직접 소비자 영향도 함께 검토한다.
4. 교정 후 같은 정책 basis에서 표준 생성과 기본 검증을 실행한다. Ref가 발행된 Plan bytes를 변경해야 하면 candidate successor를 만들고 새 Receipt identity/path를 사용한다. 기존 Freeze·member·Decision·acceptance·이전 후보/시도 증거는 보존한다. 두 모드가 실제 통과한 뒤에만 P2G-40 complete result와 current Plan을 선택한다.

현재 NEXT-0005 카드의 제외 항목은 **“검증기·AGENTS·P2G/GEX 규칙 수정”**이다. P2G-40의 도구 결함 처리도 **“필요한 도구 교정은 별도 승인 범위”**라고 정한다. 따라서 조사·계획 작성·재현을 마친 여기서 멈추며, 필요한 추가 권한은 표준 검증기와 그 직접 companion/회귀 fixture의 이 호환성 교정이다. 게임 코드/RON/store·정책 의미·P2G/GEX 규칙 변경이나 P2G-50 이후 실행은 제안 범위에도 포함하지 않는다.
