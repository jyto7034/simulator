fn main() {}
/*
• - 무한 루프(시뮬레이터 정지) 가능성 — AutoCastStart가 이동 중이면 같은 tick에 자기 자신을 재스케줄
      - core/src/game/battle/core/sim.rs:440에서 이동 중(MoveState::Moving|Ghost)이면 next_ms =
        caster.move_next_step_ms로 지연시키는데, move_next_step_ms == time_ms(해당 tick에 MoveStep 예
        정)인 경우 AutoCastStart(time_ms)를 다시 넣어 같은 time bucket에서 영원히 재발행될 수 있습니다
        (이벤트 큐 “same-tick drain” 루프가 끝나지 않음).
      - 재현 시나리오: 이동 중인 유닛이 피격으로 공명 만땅 → schedule_pending_autocasts()가
        AutoCastStart(now) 추가 → 위 로직으로 AutoCastStart(now)가 계속 재추가.
      - 관련: core/src/game/battle/core/mod.rs:229
  - 타일 점유 모델 불일치로 “적-아군 겹침 금지”가 깨질 수 있음 (tiles vs unit_positions)
      - 같은 편이 SoftOccupied 위를 “겹쳐서 통과”할 때 타일 상태는 유지되고(SoftOccupied의 소유 유닛이
        고정), 추가로 들어온 유닛은 unit_positions만 갱신됩니다: core/src/game/battle/
        runtime_field.rs:147
      - 이후 소유 유닛이 이동/사망으로 타일을 비우면, 타일이 Empty로 바뀌지만(또는 제거) 그 자리에 “겹
        쳐 있던” 다른 유닛은 그대로 그 좌표에 남아 타일은 빈 것으로 간주됩니다: core/src/game/battle/
        runtime_field.rs:132, core/src/game/battle/runtime_field.rs:239
      - 그 결과: 적이 그 칸을 Empty로 보고 진입/예약할 수 있어, 설계 문서의 “아군-적군 겹침 불가” 규칙
        core/src/game/battle/core/movement.rs:770
  - 투사체 비행시간 계산의 언더플로우 가능성 (speed_units_per_ms == 0일 때)
      - ceil_div(a, b)가 a + b - 1 형태라서 b==0이면 a-1 언더플로우가 납니다(특히 a==0이면 u64::MAX):
        core/src/game/battle/core/commands.rs:397
      - 데이터에서 속도가 0이 절대 안 나온다는 보장이 없다면 방어가 필요합니다.
  - 향후 스킬/어빌리티 확장 시 데미지 산식이 의도와 달라질 위험 (request.base_damage 미사용)
      - calculate_damage()가 request.base_damage를 쓰지 않고 ctx.attacker_attack 기반으로만 기본 데미지
        를 계산합니다: core/src/game/battle/damage.rs:101
      - 현재는 기본공격 중심이라 티가 안 나지만, “스킬 고정 피해/계수 피해”를 같은 함수로 처리하려 하면
        잠재 버그가 됩니다.
  - 덱 배치가 base_uuid 키 기반이라 “동일 유닛 복수 배치”가 사실상 불가능
      - 위치 조회가 positions.get(&unit.base_uuid)라 동일 base_uuid 2개면 같은 좌표를 공유하게 되고, 필
        드 점유에서 실패합니다: core/src/game/battle/core/build.rs:46, core/src/game/battle/core/
        build.rs:123

*/
