# Combat Setup Usage Audit Plan

## Objective

`src/game/combat_setup/` 파일별 실사용 여부를 실제 runtime caller와 live data path 기준으로 감사하고, 스텁 또는 죽은 파일은 제거한다.

## Source Of Truth Order

1. Runtime code and callers
2. Live RON/data load path
3. Unity-facing contract impact
4. Latest policy docs

## Plan

1. `combat_setup` 파일별 public/private item, caller, test coverage를 표로 정리한다.
2. 각 파일을 `keep`, `remove`, `follow-up refactor`로 판정한다.
3. 제거 대상은 module export와 파일을 함께 삭제한다.
4. 유지 대상은 책임과 후속 개선 후보를 기록한다.
5. focused check와 goal 범위의 넓은 검증을 실행한다.

## Completion Conditions

- 파일별 판정 표가 `EXPERIMENT_NOTES.md`에 남아 있다.
- 스텁 또는 죽은 파일은 제거되어 있다.
- 유지 파일의 책임과 후속 refactor 후보가 기록되어 있다.
- behavior/schema/Unity DTO 변경 없이 검증이 통과한다.
- 사용자와 의논하여 정해야 할 정책이 발견되면 goal을 종료한다.

