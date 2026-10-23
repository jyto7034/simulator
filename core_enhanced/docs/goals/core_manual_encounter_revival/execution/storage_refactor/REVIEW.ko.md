# 저장·복원 후속 구현 자체 검토

이 문서는 구현자의 코드·계약 검토다. 완료 감사나 비참여 감사 판정이 아니다. 실제 실행 결과는 `CURRENT_RESULTS.json`, 비용은 `COST_REVIEW.json`, 실패 원본과 교정 이유는 `EXPERIMENTS.md`가 소유한다.

## 선택한 구조와 변경 경계

기존 exhaustive `RunCheckpointPayload`를 유지한다. 새 영속/임시 타입 계층을 만들면 현재 필드별 capture·CloneShape와 wire migration을 중복시키므로, 이번 문제에 직접 필요한 독립 복사/설치와 완료 수명 경계를 공통화했다. Continue/SavePoint/거래 rollback/후퇴의 정책 차이는 `PLAN.md` 표에 명시했다. 게임 정책·V17·기본 한도·공개 명령을 바꾸지 않았다.

- `RunCheckpointPayload::{preflight_copy,copy_owned,from_shared}`가 자원·유물·난수·이력의 독립 복사 예약을 함께 적용한다. 단독 Arc는 이동하고 공유 Arc만 복사한다. 최종 reader는 cached SavePoint를 함께 보유하므로 실제 설치에서는 공유 복사가 필요하다. 단독 이동만의 절감으로 최종 비용 개선을 설명하지 않는다. current와 SavePoint alias는 독립 gameplay root를 유지한다.
- 부활 source와 stage의 해석은 `HatchedEncounter::stage_policy`로 모았다. 결과 producer와 Continue reward resolver가 같은 콘텐츠 정책을 참조한다. 기존 선택 후보 생성 helper도 그대로 공유한다.
- `retire_completed_node_setup`은 완료된 node의 preview·후퇴 상태·일반 Event session을 회수한다. 유물 Event receipt와 미수령 보상 owner는 보존하며 미완료 노드는 이 경계를 지나지 않는다. 이미 교정된 완료 preview 문제를 새 finding으로 세지 않는다.
- 저장 계량은 유물 존재와 무관하다. typed decode, 독립 Value, normalization, encoded Vec, Arc와 checkpoint clone을 별도로 예약한다. 기존 유물 native owner seed를 유지하되 전체 RSS와 동일하다고 주장하지 않는다.
- canonical writer는 지원 serde_json BTreeMap 순서를 직접 쓴다. checksum은 같은 writer로 streaming한다. envelope에 Value를 이동하며 매 저장에 전체 reader를 실행하지 않는다.
- reader는 checksum·content·domain 검증을 마친 immutable continuation을 보유한다. gameplay current는 독립 복사하고 SavePoint도 독립 설치한다. Continue 직후 동일 내용을 다시 capture/encode하지 않는다. application의 직원/유물/자원 reconcile·loadout/profile 검증은 계속 실행되며, 정상 writer가 발행한 같은 콘텐츠 상태의 불변 조건을 확인한다. fresh application의 전체 공개 snapshot/다음 명령 비교와 실제 process 소비가 이 재사용 경계를 검증한다.
- 자원 validation은 authored 순서를 요구하지 않는다. 별도 borrowed index를 정렬해 lookup하고 원본 catalog 순서는 보존한다. 기존 store fixture가 Run/Attempt/Deployment 등 lifetime 순서로 섞인 catalog를 사용하므로 사전 정렬을 가정한 잘못된 변경을 검출한다. owner/lifetime/cap/duplicate/basis 검증은 유지한다.

## 비용 bound 검토

작은 JSON 객체의 B-tree root를 객체별로 계산한다. 객체마다 11필드까지는 unsplit root1개이며 큰 객체는 nonroot 최소5항목을 사용한 보수적 node 상한이다. 고정128 depth stack은 지원 decoder의 nesting 범위와 맞춘다. 문자열 안의 괄호·콜론·escape를 구조로 세지 않는 chunked 시험과 작은 객체3500개/11·12필드 경계가 있다.

typed RunResourceEntry만 닫힌 schema에 따라 별도 계상한다. record object와 externally tagged ResourceOwner object의 두 node 대신 두 배 entry 크기로 Vec 성장을 예약한다. Employee/EquipmentCopy/FragmentCopy는 모두 UUID newtype variant여서 같은 구조다. 문자열·배열·encoded scratch 예약은 남긴다. generic owned/profile projection의 기존 넓은 bound는 바꾸지 않는다. 이 계산은 현재 toolchain과 serde_json 기본 BTreeMap 표현에 종속되며 의존성을 바꾸면 다시 검토해야 한다.

## 검증 범위와 재사용

전체 기본 Core와 관련 runtime-observation 검사를 새 입력에서 실행했다. server persistence는 실제 16 crash 지점을 포함해 재실행했다. default Core의 소켓 EPERM은 권한 재실행으로 분리하고 원시 실패를 보존했다. 기존 무관한 서버/exporter·Unity·다른 카드 전체 시험을 재실행한 것으로 표시하지 않는다.

정상/확장 공통 공개 runner는 이전 `single_goal`에서 읽기 전용으로 재사용하며 새 binary/config/issuer와 새 출력에 결속한다. 실제 writer→새 process→후속 행동, 미완료 entry rewind와 확정 후퇴, 결과 전/선택 대기·선택·수령/SavePoint, 실제 IO거절/동일재시도를 구별한다. 귀환용 depth3 제작 입력은 별도이며 비용 비교 입력을 바꾸지 않는다. AbandonRun은 명시적 실패 정산이다. 전투 패배를 대신 검증했다고 표기하지 않는다.

기존 U17 `u17_current_001`과 P2G/Goal 계약은 그대로 둔다. 이 후속은 정책을 보존하는 일반 변경이므로 새 formal Goal/P2G 동결을 만들지 않는다. `target/u17_storage_001`에는 실제 변경 전 bytes, 새 source/증거 manifest와 격리 정역 복원 결과를 별도로 고정한다. 감사와 U18은 요청 대기 경계 밖이다.
