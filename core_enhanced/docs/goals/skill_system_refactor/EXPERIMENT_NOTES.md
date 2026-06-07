# Skill System Refactor Experiment Notes

## 2026-06-02

- 사용자의 정책은 명확하다. DefenseRoute에서는 스킬 범위가 곧 실제 타격 가능 범위다.
- `defense_tile_range`와 AoE 피격 범위를 분리하지 않는다.
- 기존 코드의 `DeliveryDef::Area(shape)`는 지금 정책과 반대 방향이다.
- 레거시 테스트는 ignored로 보존하지 않고 삭제 또는 최신 focused test로 교체해야 한다.
- 작업트리가 이미 매우 dirty하므로 기존 변경을 되돌리지 않고 필요한 파일만 부분 패치한다.
- official live skill RON은 `base.ron`만 로드된다. `base.generated.ron`은 참조가 없으므로 legacy 이름으로 분리했다.
- live RON 변환 후에도 runtime에는 geometric `DeliveryDef::Area` 타입이 남아 있다. 현재는 official DefenseRoute data validation으로 차단하고, 코드 타입 제거 여부는 전체 geometric tests 정리와 함께 별도 확인이 필요하다.
- `TileArea.anchor`는 DefenseRoute 공식 피격 타일 기준점이 아니다. 실제 피격 타일은 시전자 tile과 facing에 투영한 `defense_tile_range`이며, anchor는 연출/impact context/후속 step 보조 정보다.
