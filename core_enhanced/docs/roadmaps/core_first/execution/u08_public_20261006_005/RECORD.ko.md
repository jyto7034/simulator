# U08 D 공개 수명주기 3건 회수 및 전투 검사 착수 — 2026-10-06/005

`relic_public_d001`은 실제 3/3 PASS, exit0, 20.608940초이며 source427/ad76cd8138b96630bb45f47784bad2df9a9cab36608213a3df6c7303b1696756이 실행 전후 동일하다. root가 raw, receipt, SHA와 전후 전체 source maps를 직접 확인했다. 별도 format001은 rustfmt raw exit0이며 의도한 lifecycle.rs 문법 포맷만 변경했다. wrapper99는 소스 변경을 표시한다. 모든 원본은 보존했다.

직원 A에서 B로 Normal/Corrupt 복사본을 옮길 때 UUID/variant/material은 그대로이고 수치·페널티·슬롯 소유가 이동한다. 공개 copy-only Unequip은 새 소유자의 슬롯을 지운다. 독립 lifecycle001의 정상 Map 선택/입장으로 시작한 미완료 Event는 fresh CoreApplication Continue에서 진입 전 copy/history로 돌아가며 진입 전에 이미 지불한 catalyst 비용은 유지한다. Event를 완료하면 현재 층 이력이 durable Continue에서도 유지된다. 이들은 실행된 범위이며 실제 전투/CLI나 별 OS 프로세스 복원을 대신하지 않는다.

새 원정 검사의 정확한 기존 계약은 active StartNewGame(true) 거부와 전체 공개 roots 불변 → 정상 AbandonRun → NotStarted → StartNewGame/시작 flow → copy/slot/catalyst/equipment_dust 초기화 → fresh Continue다. Core world/action_capability.rs의 StartNewGame gate는 NotStarted만 허용하며 world.rs의 dispatch는 replace_active_run을 우회 권한으로 사용하지 않는다. game_server player_game_actor/handlers.rs의 해당 flag는 durable store 교체 확인에 쓰인다. 제품 gate를 변경하지 않았고 이 D 검사가 host durable replacement까지 검증했다고 주장하지 않는다. 초기 계획의 replacement 표현은 이 실제 계약으로 구체화한다.

이전 BC002 source424/2cfcdfe…에서 이번 format 전까지의 전체 delta는 신규 lifecycle001 RON2, 신규 lifecycle.rs, integration root의 module 등록뿐이다. format은 lifecycle.rs 하나만 바꿨다. 기존 support/public001 7 catalogs/제품/기존 A7+B3+C3/owner 시험은 그대로다. 따라서 이전 공개13, query owner3, Event owner4 증거를 명시 비영향으로 유지한다. 새 D3까지 집중23을 회수한 상태이며 전체 U08 완료나 최종 현재 성능 판정이 아니다. 지금 public16을 중복 실행하지 않고 마지막 필수 통합에서 포함한다.

Sol이 owned runner0을 회수한 뒤 Astra에게 신규 src/game/world/tests/u08_lifecycle.rs와 tests/mod.rs module 등록1줄의 단독 writer를 넘겼다. 2개 검사는 실제 Withdraw→clock/cooldown→Deploy 후 새 actor의 numeric/penalty 재소비와 실제 damage→Dead→생존자가 있는 Victory→CompleteCombatResult의 slot 회수/copy+variant 보존/반복 확정 거부를 목적으로 한다. 시작 fixture는 attempt-local physical_damage=5이며 실제 incapacitation의 +1로 임계6을 넘어야 한다. 앞 협의의 trauma5/6 표현은 잘못이며 trauma100 Breakdown과 구분한다. 기대결과 직접 주입이나 reconcile helper 호출만으로 충족하지 않는다. 기존 공용 fixture를 재사용하고 별도 제품 API/RON 변경은 없다. 아직 이 2건은 작성 중이고 PASS가 아니다.

root는 계획/설계/증거 현재성/조정만, Sol6.1/xHigh는 제품·포맷·합의된 실행·EXPS/STATE, Astra/High는 시험·fixture·oracle를 담당한다. 제품과 나머지 시험 및 모든 실행은 HOLD다. 문서 밖 마지막 검증경계 ad76은 지금 writer가 열린 실시간 소스의 해시라는 뜻이 아니다. 실제 carrier/trial/CLI, 최종 기능/성능, current evidence selection→impact→adaptation→coverage→자체검증→동결/복원→비참여 독립 감사가 남았다. 사용자 연속 권한을 유지한다.
