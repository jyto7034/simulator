# Skill Contract Surface Refactor Experiment Notes

- 작업 원칙: 문서보다 현재 코드와 live RON을 기준으로 판단한다.
- `BuffDatabase`는 즉시 통합하기엔 호출부가 넓어 보인다. BattleCore 생성자/validator까지 흔들면 별도 goal로 분리해야 한다.
- `manual_activation_allowed`는 caster/resource 상태를 의미하고, target 상태는 별도 field로 내려야 Unity가 명일방주식 타일/대상 선택 흐름을 만들 수 있다.
- Range preset은 런타임까지 새 개념을 퍼뜨리지 않고 RON loading 단계에서 `defense_tile_range`로 normalize하는 구조가 가장 단순하다.
- BuffDatabase 통합은 이번 goal에서 무리하게 적용하면 source of truth를 줄이려다 constructor/API 복잡도를 키울 위험이 있다.
