# NEXT-0006 — 자동 검사 범위 정리와 P2G-40 재개

## 요청·범위

사용자가 검증기 유지 필요성을 물었고, “참조·누락/중복·의존성·검증 담당의 최소 검사만 자동화하고 입력 형식 의존을 분리하자”는 제안에 **“응 그러면 진행해줄래?”**라고 답했다. 이를 NEXT-0006의 도구 교정과 같은 최소 계약의 P2G-40 재개 권한으로 인수했다. 이전 NEXT-0005가 제외했던 검증기 변경은 이 새 요청으로 승인됐으며 같은 허락을 다시 요구하지 않았다.

게임 코드/RON/store, AGENTS/P2G/GEX 규칙, 동결 정책·수용 기준은 변경하지 않았다. 실제 종료점은 P2G-40이고 새 Topic의 P2G-50/60·GEX는 실행하지 않는다. `Source Design`으로 이전 candidate를 연결한 JSON successor는 같은 WP/Phase/검증 ownership을 명시적으로 직렬화한 기술적 교정이다.

## 최소 검사와 구현 변경

[도구 설명](../../tools/README.md)이 입력 지원 범위와 자동/수동 검토의 경계를 소유한다. 자동화 대상은 exact 입력 참조·hash, 정책 담당의 누락/중복, WP/group partition과 dependency/SCC/Phase 관계, 검증 담당 배정과 Receipt 무결성이다. 설계 이유·정책 승인·실제 코드 currentness·중간 상태의 의미는 문서와 코드 근거로 별도 검토한다. 구조 검사 통과를 게임 구현 성공으로 해석하지 않는다.

| 변경 | 결과·직접 영향 |
| --- | --- |
| [공통 입력 해석](../../tools/p2g_inputs.mjs) | 새 JSON Freeze의 exact member를 Topic의 artifact bundle에서 해석하고 payload의 Stable Element ID를 사용한다. 실제 참조한 bundle hash도 Receipt에 남긴다. JSON Plan은 명시적인 selector·edge·owner 배열을 읽는다. 기존 Markdown 표현 규칙은 호환 adapter로 분리했다. |
| [P2G-40 validator 1.2.0](../../tools/validate_p2g40_receipt.mjs) | 생성과 검증이 같은 입력 모델과 관계 계산을 사용한다. Receipt의 계산 결과 전체를 대조하며 잘못된 기존 target을 덮어쓰지 않는다. 기존 v1 Receipt와 method 1.0/1.1 경로를 보존하고 새 method의 typed Ref/source hash 근거를 추가했다. |
| [P2G-50 직접 소비자 1.3.0](../../tools/validate_p2g50_goal_set.mjs) | 같은 Freeze/Plan 해석을 사용하고 Receipt의 source/hash·관계를 재검증한다. 기존 Goal Quality Contract 2와 Goal 자체의 작성 규칙은 바꾸지 않았다. 이 Topic의 Goal은 만들지 않았다. |
| [회귀 검사](../../tools/p2g_validation.test.mjs) | 실제 Topic·임시 synthetic fixture·기존 Receipt 4개와 과거 완성 Goal Set을 사용한다. 명시적 ID, 누락/중복/잘못된 참조, cycle/owner, source 변경, 불변 Receipt와 직접 소비자 호환성을 검사한다. |

형식을 바꾸기 위해 기존 Freeze·Policy Element를 개명하거나 고치지 않았다. `PP-CAMEC-P2G40-001`과 실패 시도는 그대로 보존하고 [JSON successor](P2G_40_PHASE_PLAN_002.json) `PP-CAMEC-P2G40-002`와 새 Receipt `PVR-CAMEC-P2G40-002`를 사용했다. 기존 candidate의 정책 담당·WP/group/Phase 관계와 12개 acceptance 참조 의미를 변경하지 않았다.

## 실제 검증과 남은 한계

[실행 근거](P2G_40_TOOLING_VALIDATION_001.json): 회귀 검사 **37/37 통과**, 표준 생성과 같은 입력의 기본 검증 모두 실제 `exit 0 / passed / failures []`. 최종 생성 호출은 이미 생성한 동일 Receipt를 `unchanged`로 확인했다. JSON member 6, WP 3, edge 4, cyclic SCC 1, Atomic Group 1, final node 2, Phase 1, integration row 6이다.

첫 실행에서는 Node 자식 프로세스가 sandbox에서 EPERM으로 차단되어 승인된 외부 실행으로 재검증했다. 호환 검사에서 inline-code integration row와 과거 Current Decision 목록의 대체 표기를 발견해 adapter를 교정했다. 실패한 실행을 통과 횟수로 계산하지 않았다. 기존 Receipt 4개는 생성·검증 뒤 bytes가 그대로였다.

새 Phase Plan의 책임·전환·중간 상태는 [기존 코드 검토](CODE_REVIEW_001.ko.md) §3–5와 [이전 계획 후보](P2G_40_PHASE_PLAN_CANDIDATE_001.ko.md)의 설계 판단을 인수했다. descriptor/효과 주소·runtime/catalog 동시 전환은 AG-CAMEC-01, 새 durable 통합 fixture는 WP-CAMEC-03이 소유하며 같은 Phase 안에서 완료해야 한다. 기존 publication/SkillId/command/checkpoint는 실제 선행이고 새 API·fixture·저장 상태가 이미 있다고 가정하지 않는다.

P2G r2와 배치된 GEX gex-r1의 revision/해시는 유지했다. 새 도구가 현재 source를 읽는다는 사실은 runner routing·서버/Unity·배포 검증을 뜻하지 않는다. 과거 Markdown은 기존 template 범위의 호환 지원이며, 임의 문서 전체의 의미를 자동 판정하지 않는다. 개인 저장 데이터나 모든 ignored/asset 파일을 조사하지 않았다.

B40-001의 JSON Freeze/Stable Element ID 호환성 blocker는 해소됐다. P2G-40 complete result와 current selection은 [단계 결과](P2G_40_RESULT_002.json)가 소유한다. 다음 P2G-50은 새 작업 카드로만 남기고 이번 요청에서 실행하지 않는다.
