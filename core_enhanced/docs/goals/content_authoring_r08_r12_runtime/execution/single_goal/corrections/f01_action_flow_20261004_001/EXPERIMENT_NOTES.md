# U01 현재 판단

새 정책 변경은 없다. F01의 기존 취소 계약을 구현한다. `CancelUnreleased`와 actor 전체 interrupt는 별개다.
pending 준비가 끝나도 action의 continuation은 남을 수 있다. ActionHandle의 attempt/occurrence identity를
보존해 actor ID만으로 취소하지 않는다. 이미 방출한 child는 원래 종료까지 실행하고 source/lease를 유지한다.

기존 U00의 구저장 호환 제외·보상·도구 결정은 후반 owner의 차기 정책이며 이 교정의 기존 Goal을 변경하지 않는다.
이번 pass가 구형 converter를 삭제하거나 후속 정책을 소급 적용하지 않는다.

원 runner와 공개 입력은 현재 저장소에 존재한다. 과거 ZIP의 누락 표시는 U00에서 해소했다.
과거 PASS는 재실행으로 표시하지 않으며 current source와 실제 검사 결과를 연결한다.

- focused_001: 새 회귀 3개 및 동일 이름의 기존 F01 시험 포함 14 PASS. 취소 6경계, Detach, fault 6지점 복구/재시도 확인.
- focused_002: 5 PASS/1 FAIL. 새 same-timestamp 시험이 초기화 이전 분기 두 개의 Debug 문자열 HashMap 순서를 비교했다. 실제 차이는 scenario unit_refs와 result_stats unit_sources 표시 순서뿐이었다. 시험을 동일한 초기화 완료 상태에서 분기하도록 정정했다. 제품 코드를 이 실패에 맞춰 변경하지 않았다. 원 실패 log/receipt는 보존한다.

- all_default_001은 사용자 turn 중단 시 프로세스가 종료되어 완료 receipt 없음. 원 partial log와 interruption 기록을 보존하고 all_default_002에서 전체 1684 PASS를 새로 확보했다.
- r05_current_001은 별도 도구 Cargo cache의 zmij 패키지 부재로 offline 해석 중 exit101(0 workload), r05_compare_001은 raw 부재를 감지해 실패했다. 성능 경고나 PASS로 분류하지 않았다. --locked fetch 후 별도 _002 경로에서 동일 원 검증을 다시 실행한다.

## 실제 소유권·실패 경계 판단

- `release_finished_boss_waiters`의 TargetInvalidated/TimedOut+CancelUnreleased 및 state 이탈의 `detach_boss_action` 경로를 교정했다. 정상 Released·명시 Detach는 기존 result detach만 수행한다.
- 기존 actor-wide `cancel_boss_owned_work`는 이동 epoch·basic windup·모든 structured flow를 취소한다. 정확 task 취소에서는 이를 호출하지 않고 actor와 opaque ActionHandle이 모두 일치하는 owner만 제거한다. 이 변경은 준비 중 취소에도 같은 격리 계약을 적용한다.
- ActionHandle equality는 ordinal이 아니라 occurrence capability의 Arc identity다. 같은 ordinal·다른 capability를 가진 내부 대조 cast의 owner·wake 및 actor 상태를 보존하는 시험으로 실수를 방지했다. 공개 후속 state는 기존 marker 팩으로 확인했다.
- SkillStep/SkillFlowWake만 큐에서 제거한다. 이미 현재 timestamp bucket으로 추출된 이벤트는 owner 부재 검사로 no-op이며, 새 회귀에서 stale wake와 legacy delayed step 재삽입을 각각 확인했다. 다른 cast의 예약 이벤트는 retain된다.
- projectile/area는 독립 source snapshot과 delivery ticket을 갖는다. cast owner를 제거해도 completion path가 ticket을 닫는다. 지속 area는 취소 뒤 3회 피해와 최종 outstanding delivery 0을 확인했고, projectile은 공개 oracle의 5885ms HP 손실을 확인했다.
- `runtime_facts.terminal`은 미실행 control steps와 bound resource를 정리하되 outstanding delivery를 보존한다. 이미 terminal인 action은 overwrite하지 않는다. queue scan·owner 제거·terminal의 실패를 Result/poll로 전달하며 기존 후보 transaction이 전체 경계를 복구한다. native 한도 시험은 기존 전체 suite, 새 취소 mutation 경계는 6지점 fault/retry 시험으로 연결했다.
- 제품 변경은 두 파일의 메서드 추가/호출 대체다. schema, action/resource 구조체 크기, wire/save version, RON enum, gameplay 정책은 바꾸지 않았다. 이번 새 정책 결정(U00)을 이 기존 Goal에 소급 적용해 V7/V8 converter 등을 제거하지 않았다.
- 공개 취소 5팩 trace on/off의 실제 battle event가 전부 같았다. 기존 공개 9팩45명령의 gameplay/amount parity도 별도로 확인했다.
