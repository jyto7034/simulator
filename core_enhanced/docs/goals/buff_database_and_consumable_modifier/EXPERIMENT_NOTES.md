# BuffDatabase And Consumable Modifier Notes

- 전투 상태이상 buff와 consumable modifier는 같은 runtime으로 합치지 않는다. 전자는 ms/tick 기반 전투 상태, 후자는 직원 단위 다음 전투/런 modifier다.
- `TimelineValidator`에 전체 `GameDataBase`를 넘기면 API 영향이 커진다. validation에 필요한 것은 buff registry뿐이므로 `Arc<BuffDatabase>`를 들도록 하는 편이 더 작고 명확하다.
- `active_consumable_modifier` JSON 필드명으로 Unity-facing 계약을 정리했다. 현재 Unity 클라이언트도 새로 구현 중이고 goal이 명칭 정리를 요구하므로 구 `active_consumable_buff` 호환 필드는 남기지 않는다.
- `GameDataBuilder::empty()`는 기존 테스트와 fixture가 기본 상태이상 buff를 전제하던 흐름을 보존하기 위해 live 기본 buff RON을 포함한다. 빈 buff DB가 필요한 검증은 `with_buff_data(BuffDatabase::new(vec![]))`로 명시한다.
- `cargo test -p game_core ron_loading -- --nocapture`, `skill_refactor_validation`, `skill_test_suite` 필터형 명령은 0개 테스트를 실행했다. 실제 통합 테스트 검증은 `--test ron_loading`, `--test skill_refactor_validation`, `--test skill_test_suite`로 별도 수행했다.
