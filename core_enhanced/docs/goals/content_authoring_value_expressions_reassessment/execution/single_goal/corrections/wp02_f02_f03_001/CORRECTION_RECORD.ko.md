# R06 WP02 F02/F03 교정 기록

상태: **교정·자체 검증 종료 / 단계검토 타깃 동결 / 재검토 요청 대기**. 이전 F01-R01 동결 타깃과 증거는 수정하지 않았다.

## F02 — action 실행 정보의 결속

`RuntimeFactState::validate_action_source`가 유효 action handle에 더해 소유 action snapshot의 actor, 수락 당시 확정된 equipment snapshot, 보관된 cast 실행 정보가 전달 source와 일치하는지 확인한다. equipment snapshot은 definition UUID·runtime instance UUID·persistent copy UUID를 함께 비교하므로 같은 definition의 다른 사본은 통과하지 않는다. action 수락 후 child/proc가 provider를 얻어 snapshot에 provider가 없던 경우에는 BattleCore의 실제 runtime item이 해당 actor 소유인지, 전체 snapshot이 정확히 일치하는지 확인한다. 따라서 persistent copy UUID가 없는 실제 scenario item도 허용한다. effect별 authored metadata와 `RuntimeFactSource` 전체 구조는 비교하지 않는다. 기존 delivery의 action handle·ordinal 검사도 유지한다. 검증용 mutable 상태, 공개 API, 직렬화 필드, token 신뢰 경계는 추가하지 않았다.

capture 때 검증된 Snapshot은 이후 source가 퇴장해도 적법한 released delivery가 살아 있으면 저장한 값을 사용한다. 이 테스트는 실제 `withdraw_unit` 경로를 쓴다. action/delivery 자체가 무효가 된 경우의 기존 `Unavailable` 처리는 변경하지 않았다.

## F03 — Live resource 읽기 가용성

ValueExpr frame의 Resource 분기에서 SourceEmployee·SourceEquipment는 Source unit을, TargetEmployee는 Target unit을 해석해 `value_expr_unit`의 active 검사를 resource lease 조회 **전에** 실행한다. 그 뒤 기존 owner/lease/lifetime 검사가 그대로 적용된다. inactive/missing은 `MissingContext` typed fault로 거부하며 balance·revision·reservation을 읽기/실패 과정에서 바꾸지 않는다. 공유 resource mutation resolver는 변경하지 않았고, Run/Attempt의 사망 후 저장 lease와 Deployment 종료 규칙도 유지된다. capture 때 만든 Snapshot frame은 이후 lifecycle 변화와 분리한다.

## 직접 실행 증거

- 원본 감사의 수정 전 probe를 구현 전 archive에 그대로 연결해 36개 중 33 PASS·3 FAIL을 재현했다. 실패는 actor 바꿔치기, equipment copy 바꿔치기, 실제 death producer 이후 TargetEmployee Live read였다. 첫 격리 실행의 test hook 범위 오류로 인한 컴파일 실패도 별도 로그로 보존했고 재시도 결과를 재현 근거로 삼았다.
- 최종 source에서 **동일 원본 probe**를 다시 실행해 36/36 PASS. 원본 F02 두 건은 capture 때 거부, F03 한 건은 실제 사망 뒤 `MissingContext`로 거부된다. 정상 catalog 대조군도 통과한다.
- 새 제품 회귀는 actor 및 definition/runtime/copy 바꿔치기, 다른 delivery action, 실제 scenario의 persistent copy 없는 장비, child delivery 정상 경로, 실패 후 상태 불변·정상 재시도를 확인한다. F03는 세 owner selector의 active/dead/withdrawn/missing, Source/Target 구분, Snapshot frame 보존, 실제 death/withdraw producer, Run/Attempt/Deployment의 target lifetime 차이를 확인한다. missing unit은 실제 공개 lifecycle producer가 없어 테스트 내부에서만 제거해 확인했고 이를 제품 lifecycle PASS로 세지 않는다.
- 최종 source에서 all-target check, focused rustfmt, 기존 `clippy::never_loop`·`clippy::eq_op` allow만 적용한 Clippy가 통과했다. sandbox 밖 all-target test는 lib 1,417 + integration 52 = **1,469 PASS**. 최초 sandbox 실행은 Unix socket bind가 `Operation not permitted`여서 1개 실패했고 로그를 보존한 뒤 동일 테스트를 sandbox 밖에서 재실행했다.
- F01-R01 검토자의 allocator·Snapshot probe 7개를 최종 source에 재연결해 7/7 PASS. 기존 full suite에는 F01 선청구·취소·bounded frame·rounding·fingerprint와 R05 finite delivery/resource lifetime 회귀가 포함된다. 앞선 PC 36조건과 R05 별도 allocation guard는 **재사용 증거**이며 이번 교정에서 다시 측정하지 않았다.

미검증: 다른 OS/allocator의 실측, SourceEquipment의 Run/Deployment별 전체 lifecycle 조합, 실제 producer가 제거한 missing unit 조합, CAP 부하 수용력. SourceEquipment resource는 persistent copy 소유권이 필요한 기존 계약 때문에 copy UUID가 없는 경우를 resource read 성공 사례로 세지 않는다. F02/F03의 이번 단계는 자체 검증이며 재검토 PASS나 WP02 완료 선언이 아니다.
