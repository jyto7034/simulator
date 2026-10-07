# R06 WP02 단계검토 메모 — action 완료 결과와 release Snapshot

## 구현과 실제 소유권

- `RuntimeFactState::ActiveAction`이 exact `ActionHandle`에 귀속된 `ActionResultWitness`를 소유한다. 일반 literal action의 메모리 비용을 늘리지 않도록 선택적 box로 둔다. `ActionSite`의 definition과 source action/delivery를 확인하고, active action이 정리될 때 witness도 제거한다. 공개 settled receipt에는 복사하지 않는다.
- `execute_ordered_skill_effects`는 첫 effect 전에 skill 전체의 결과 참조와 resource ID를 조사하고 할당·정렬 전에 work/retained 비용을 승인받는다. 각 effect의 전체 대상 평가와 기존 command 실행 후 실제 fact projection을 한 번만 누적한다. action 전체 누적은 다른 step과 delivery를 포함하고, 기존 `EarlierEffect` delivery 내부 prefix는 별도로 유지한다. action 결과 참조가 없는 literal 경로에는 witness가 설치되지 않는다.
- effect 시작의 `CurrentActionPrefix` Live 읽기는 active action, site, delivery, 완료 frontier를 확인한다. 첫 완료 전 읽기와 다른 attempt/소유권은 typed fault다. 완료된 값 0은 부재와 구분한다. release capture는 그 시점에 완료한 같은 witness를 `EvalCapture.snapshot_frame`에 복사하므로 이후 Live 누적으로 바뀌지 않는다.
- 실제 유한 delivery의 result fact에는 ticket이 보존되지 않으므로 관찰 시 실행 중인 source ticket과 fact의 action/site를 함께 결속한다. 이 연결이 없으면 다른 delivery의 완료 결과가 0으로 남는 문제가 직접 시험에서 드러났다.
- 실패는 기존 immediate/projectile/area caller에서 candidate transaction으로 전파한다. 앞에서 확정한 timestamp는 남고 실패한 candidate의 HP, fact, witness, queue 등은 함께 복원된다. compile validator는 앞 step에 완료 가능한 effect가 있는 첫 effect의 action prefix를 허용하며, 실제 완료 여부는 runtime에서 판정한다.

## 이번 checkpoint에서 확인한 경계

- 동일 action의 실제 HP loss 7과 다른 delivery의 3을 누적해 뒤 effect가 10을 읽는다. 다른 action/attempt는 섞이지 않는다.
- release 당시 7을 Snapshot에 고정한 뒤 Live가 10이 되어도 Snapshot은 7이다. 실제 executor 소비로 target HP 87을 확인한다.
- resource 결과의 완료 0과 누적 5, 경계 예산의 성공과 1 byte 부족한 admission 거부, 뒤 effect fault 후 현재 candidate만 복원하는 경로를 직접 시험한다.
- 기존 같은 effect의 전 대상 선평가, 실제 mitigation 이후 damage, finite delivery, literal/resource/clone 회귀는 전체 제품 시험에 포함했다. 기존 거부 전용 `action_prefix_refusal_rolls_back_only_the_current_candidate`는 새 지원 계약에 맞는 성공 시험과 실제 fault rollback 시험으로 전환했다.

## 남은 범위와 중단 이유

Action 완료 결과와 release Snapshot은 같은 witness 소유권과 완료 frontier를 공유하므로 하나의 연결 책임으로 닫았다. 다음 큰 연결 작업인 D04 실제 target aggregate와 D07 preview/explain 전에 단계검토를 위해 중단한다. D04의 identity 중복 제거, owner/tile grouping, 결정적 reduction과 D07의 runtime frame/trace 일치는 아직 구현·검증하지 않았다. WP03의 새 stat/copy producer 의존, WP04, R07 이후, 서버·Unity 변경도 이번 범위 밖이다. production expression publication은 계속 차단된다. 이번 자체 검증은 독립 PASS나 WP02/R06 완료 판정이 아니다.
