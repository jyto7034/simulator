# Consumable Item Experiment Notes

## Notes

- `UseConsumableItem`은 Safezone 내부 화면 전환 command가 아니다. core는 `ViewingMap`/`NodeConfirm` 상태에서 사용 가능한 준비 행동만 제공한다.
- `Forbidden` tier는 schema에는 둘 수 있지만 live pool에는 넣지 않는다. 부작용/침식/신뢰도 정책이 아직 확정되지 않았기 때문이다.
- `bound`는 owned instance가 아니라 장비 metadata에 둔다. 획득 경로별 귀속 분기는 만들지 않는다.
- `DeployCostReduction`은 배치 코스트 감소로 연결했다. `InitialSkillCharge`는 전투 시작/배치 시 스킬 게이지(`resonance`) 초기 충전으로 연결했다. `DefenseMitigation`은 schema와 우선순위에는 포함했지만 실제 피해 감소 적용 경로는 아직 구현하지 않았다.
