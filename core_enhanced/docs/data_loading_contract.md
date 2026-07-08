# Data Loading Contract

이 문서는 core live data loading의 현재 계약을 짧게 고정한다.
탐색용 지도는 [maps/data-and-content.md](maps/data-and-content.md)를 보고, RON authoring strictness와 runtime source-of-truth 원칙은 [core_runtime_contract.md](core_runtime_contract.md)를 함께 확인한다.

## Source Of Truth

실제 source of truth는 runtime code와 live RON/data다. 이 문서는 깨진 링크를 막기 위한 별도 계획 문서가 아니라, 현재 코드가 이미 따르는 data loading 경계를 요약하는 계약 문서다.

공식 production live data entrypoint는 [src/game/data/mod.rs](../src/game/data/mod.rs)의 `GameDataBase::load_live_embedded()`다.

- `game_server` startup은 `GameDataBase::load_live_embedded()`로 위임한다.
- live RON을 읽는 integration/helper test도 같은 entrypoint로 위임한다.
- loader는 `include_str!` 기반 embedded bundle을 사용한다. RON 수정은 재컴파일이 필요하며, 이 경로는 hot reload나 filesystem data-pack API가 아니다.
- live bundle 구성 뒤 `GameDataBase::new()`가 index/reference validation을 수행한다.
- 공식 loader는 추가로 generated combat preview contract validation을 실행한다.

## Main Bundle

`GameDataBase::load_live_embedded()`가 소유하는 main live bundle은 gameplay data를 `GameDataBase` 안으로 모으는 경로다.

현재 main bundle에 포함되는 주요 도메인:

- abnormalities
- artifacts
- boss omen chains
- buffs
- consumables
- corroded employees
- corroded wave presets
- employees
- equipment
- events
- PvE encounters
- rewards
- run policy
- shops
- skills
- skill fragments

세부 live file 위치와 관련 module은 [maps/data-and-content.md](maps/data-and-content.md)의 Live RON Groups 표를 기준으로 찾는다.

## Domain-Owned Builtins

모든 embedded RON reader가 반드시 `GameDataBase::load_live_embedded()` 안에 있어야 하는 것은 아니다.
다음처럼 특정 runtime domain이 직접 소유하는 builtin catalog/policy는 명시적인 domain loader로 남을 수 있다.

- map node definitions
- map generation policy
- battlefield archetypes/templates
- run policy defaults used by minimal fixtures
- test-only direct schema/audit loads

단, domain-owned loader는 다음 조건을 만족해야 한다.

- owning domain이 분명해야 한다.
- production live startup과 충돌하는 별도 gameplay source of truth가 되면 안 된다.
- validation 또는 focused test가 있어야 한다.
- 새 loader를 추가할 때는 왜 `GameDataBase` main bundle이 아닌지 문서나 코드 주석으로 설명한다.

## Fixture Builders

`GameDataBuilder::empty()`와 fixture helper는 테스트용 최소 DB를 만드는 도구다.
`GameDataBuilder::live_defaults()`는 embedded live buff data가 필요한 fixture convenience이며, production live bundle loader가 아니다.

Production/server code는 live data가 필요하면 `GameDataBase::load_live_embedded()`를 사용한다.

## Validation Gates

Live data loading은 gameplay 시작 전에 authoring mistake를 잡는 gate다.

- 각 database constructor/index validation은 duplicate id와 local shape 문제를 잡는다.
- `GameDataBase::new()`는 cross-reference validation을 수행한다.
- `GameDataBase::load_live_embedded()`는 generated combat preview contract validation까지 수행한다.
- `tests/ron_loading.rs`와 live catalog tests는 실제 embedded bundle이 runtime 계약을 만족하는지 확인한다.

Live RON schema를 바꿀 때는 보통 다음을 확인한다.

```text
cargo test -p game_core --test ron_loading -- --nocapture
cargo check -p game_core
```

변경이 server startup까지 닿으면 `cargo check -p game_server`도 확인한다.
