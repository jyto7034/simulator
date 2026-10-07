# 터미널 전투 시연

그래픽 클라이언트 없이 실제 core를 실행해 개발 진척을 보여주는 테스트다.
기존 환상체 전투 시험은 결과 검증과 JSON 내보내기를 제공한다. 이 시연은 그 runner와 전투 데이터를 재사용하고, 첫 스킬 시전의 사건만 사람이 읽기 쉽게 출력한다.

저장소 루트 또는 `core_enhanced`에서 실행한다.

```sh
cargo test -p game_core --test battle_showcase -- --nocapture
```

처음에는 컴파일 시간이 걸릴 수 있다. `--nocapture`가 있어야 성공한 시험의 설명도 보인다.
의존성이 이미 준비된 환경에서는 `--offline --locked`를 `-- --nocapture` 앞에 추가할 수 있다.

시연은 「아무것도 없는」의 Goodbye 첫 시전을 보여준다.

1. 반격하지 않는 HP 2000 표적을 옆 타일에 배치한다.
2. 실제 스킬 실행에서 첫 피해, 시전자 공격력 변화, 마무리 두 타격을 읽는다.
3. 사건의 실제 시각·HP 전후·공격력 전후와 해당 스킬의 실제 피해 합계를 출력한다.

설명 문구만 미리 적었으며 시각과 결과 수치는 실제 `BattleEventLog`에서 가져온다. 같은 로그의 원인 연결을 따라 해당 시전의 효과만 고르고, 단계 순서·대상·피해·강화 값이 기대와 다르면 시험이 실패한다. 다른 공격을 스킬 피해 합계에 섞지 않는다.

이것은 기존 콘텐츠를 훈련용 배치에서 실행한 시연이다. 최종 게임 밸런스, 그래픽 화면, 신규 R06 수식의 공개 지원이나 전체 전투 승리를 보여주는 것은 아니다. 첫 시전 이후의 사건은 터미널 요약에서 생략한다. 기존 runner는 전체 JSON 로그도 `core_enhanced/target/debug_event_log_exports/skill_test/`에 내보낸다.

관련 파일: [시연 테스트](../tests/battle_showcase.rs), [기존 환상체 검증](../tests/skill_test/nothing_there.rs), [공유 전투 runner](../tests/skill_test/common/run.rs).
