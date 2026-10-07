# R06 WP02 D04 실제 대상 집계 — 단계검토 인계

상태: **부분 구현·자체 검증 완료, 독립 검토 대기**. 선행 action 결과·release Snapshot target `b16697bfe8c518944f0bddcf800b0f1e005ed639edd6fe915e4d96c726a6e1f4`와 그 부분 PASS 위에서 진행했다. 이번 target은 `target/d04_resolved_target_001/MANIFEST.json`이다. 제품 source 최종 digest는 `4c63ca018cd39d50e4a896711debaaf7970dc2636841a77eb74457f83313582f`다.

## 검토 단위와 호출 경계

- executor `execute_ordered_skill_effects`의 확정 target set이 `prepare_resolved_target_members`에 전달된다. canonical member payload는 별도 복사본으로 만들고 실제 command target 순서는 그대로다. `AggregateMember`는 active RuntimeUnit의 명시 Attack·HP reference와 해당 moment의 projected tile을 읽는다. Source·현재 Target·action result prefix는 별도 frame context다.
- release carrier는 확정 set이 있는 Snapshot의 identity·값·tile을 Arc payload에 고정한다. target별 frame이 이 payload를 공유한다. Live는 effect마다 새로 읽고 동일 effect의 모든 대상 amount를 mutation 전에 평가한다. impact와 area tick은 해당 occurrence의 set을 사용한다.
- raw 입력과 정렬·member 읽기 전에 checked work/native retained/authoring admission을 적용한다. evaluator는 기존 member/group/nested·typed reduction·trace 검사를 그대로 담당한다. 오류는 caller의 candidate transaction까지 전파된다.

| delivery | Snapshot resolved WholeSet/Tile | Live resolved WholeSet/Tile | 실제 근거 |
| --- | --- | --- | --- |
| Instant | release에 확정 set 제공 | 소비 effect의 확정 set | `ordered_effect_uses_one_live_member_set_before_mutation_then_rebuilds_it`, `released_member_values_and_tiles_remain_snapshot_while_live_advances`, `actual_resolved_members_canonicalize_identity_and_read_projected_tiles` |
| target-locked projectile | release에 확정 target 있을 때 제공 | impact의 확정 set | `target_locked_projectile_uses_release_members_for_snapshot_and_impact_members_for_live` |
| directional projectile | release set 미확정: `UnsupportedMoment` | impact의 확정 set | catalog의 `actual_finite_set_capability_follows_delivery_moment`, projectile executor 결합, 전체 회귀 |
| tile area | release set 미확정: `UnsupportedMoment` | 매 tick의 확정 set | `resolved_area_tick_rebuilds_live_members_for_each_occurrence` |

빈 확정 set의 WholeSet Count/Sum은 0, Max는 `Unavailable`이다. set 자체가 없으면 `UnsupportedMoment`다. ByProjectedTile은 실제 `UnitBody::projected_tile`을 쓰며 A/B가 같은 tile, C가 다른 tile일 때 inner Max→outer Sum=7과 inner Count→outer Max=2를 확인했다. `[C,A,B,A]`와 `[B,C,A]`의 집계 결과·trace는 동일하다. ordered damage의 두 대상 HP 10/10은 첫 Live effect에서 각 5, 다음 effect에서 새 합계로 각 2를 받아 최종 HP 3/3이다.

## 검증과 복원

`EXPERIMENTS.md`와 `evidence/`에 각 명령, 전후 source bytes hash, full log, 실제 시험 수를 보존했다. 최종 revision에서 check, value-expression focused 74, 승인된 환경 전체 1,500, Clippy, 변경 영역 format 모두 통과했다. 샌드박스 전체 시험의 Unix 소켓 EPERM 1건과 동일 명령의 승인된 환경 재실행도 각각 보존했다. PC 36조건은 이전 target과 동일하게 성공 31/typed refusal 5, gameplay oracle·오류 코드 차이 0이다. R05 allocation observer의 concurrent peak 8,997,016 bytes는 원 cap 9,030,188보다 33,172 bytes 낮고 guard를 통과했다. 별도 D04 workload의 raw/canonical/group/work/retained 수치는 `d04_workload_001.log`에 있다.

`baseline/entry_001`은 시작 실제 333개 파일을, 새 target은 최종 333개 source archive·변경 9개 경로·양방향 patch·증거 archive·manifest를 보존한다. `freeze.py verify`로 currentness와 before→after 및 after→before byte 복원을 확인한다. 시작 dirty/untracked 파일은 제거하거나 정리하지 않았다.

## 잔여 범위와 검토 요청

- resolved `ByOwner`에는 현재 AST `Grouping::ByOwner`의 Side/Employee/EquipmentCopy domain 선택 인자가 없다. 임의 side/employee/equipment owner를 추론하지 않고 `InvalidOwner`로 거부한다. authored ByOwner는 기존 의미를 유지한다. 원 대화에서 domain 표현과 schema·plan 변경 범위를 정해야 한다.
- 실제 unit의 `Read(AggregateMember { unit })`는 어떤 scalar인지 정하지 못해 `MissingContext`다. WP03 SpellPower/cadence 등 아직 producer 없는 reference도 typed refusal이다. 미확정 release set의 Snapshot은 빈 set이나 impact set으로 대체하지 않는다.
- D07 preview/explain, WP03/04, WP02 전체 통합·감사·publication 해제, R07 이후, 서버·Unity 작업은 미실행이다. 이번 자체 검증을 독립 PASS나 WP02/R06 완료 판정으로 표시하지 않는다.

WholeSet·ByProjectedTile의 실제 target frame, 직접 의존한 capture/consume·예산·rollback, 관련 전체 회귀·cost guard가 닫힌 첫 단계검토 지점이므로 여기서 중단한다. 원 대화에서 독립 검토와 다음 범위를 결정해 달라.
