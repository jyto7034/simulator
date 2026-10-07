# A battle projection과 unavailable 상태 — 부록003

실제 BattleUnitDraft::combat_profile(types.rs725)는 weapon/profile의 행동 구성을 적용하지만 성장·영구아이템·scenario scale·condition HP를 완성하지 않는다. BattleUnitDraft::effective_stats(types.rs808)→stat_pipeline::effective_stats_for_draft가 실제 final battle projection이다. 신규 plan의 base_battle_stats는 이 계산 결과, effective_battle_stats는 그 뒤 relic penalty 적용 결과다. relic-bearing roster effective projection도 이 경로를 사용한다. certified/persistent power와 noRelic 기존 roster 동작은 보존한다.

동시에 employee combat_profile는 WeaponRequired guard를 유지한다. 정상 game player의 무기 해제 상태를 가짜 unarmed 실행 성공이나 fake0 능력치로 표시하지 않는다. mandatory UnitStats는 이 상태를 표현하지 못하므로 다음과 같이 구체화한다.
- base_battle_stats/effective_battle_stats: Option<UnitStats>.
- stats_unavailable_reason: Option<RelicNumericPlanUnresolvedReason>.
- 정상 profile: Some/Some/None. JSON 능력치 객체 형상은 기존 확정 의미와 같다.
- 실제 WeaponRequired: None/None/Some(ProviderUnavailable), targets 빈 목록. 아직 없는 실행 source를 발명하지 않는다. 이미 아는 penalty 정보는 보존한다.
- InvalidStaticData/native budget 등 다른 오류를 availability로 삼키지 않는다.

Basic.weapon Option은 실제 equipment provider 부재를 표현할 수 있는 DTO 타입일 뿐 현 employee WeaponRequired를 우회하지 않는다. root가 실제 stat pipeline을 읽은 뒤 Sol과 Astra 양측에 동일 계약을 전달했다. Astra의 준비된 A6시험에 무기 해제 조회·불변 반례를 추가하도록 시험 파일만 재해제했다. 아직 모든 실행은 HOLD다.
