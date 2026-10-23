# U16-4 독립 재감사 — Not Complete

2026-10-15. GS-CSCB-001 / GOAL-CSCB-01. 사용자가 현재 실행 지시서에 따른 U16-4 독립 감사를 직접 요청하여 수행했다. 이번 실행자는 이전 구현·교정에 참여하지 않았으며, 제품·기존 시험·공식 콘텐츠를 수정하지 않았다. 서브에이전트를 사용하지 않았다.

**판정: Not Complete. F-U164-IA001은 교정 확인으로 닫는다. 새 P2 finding F-U164-IA002와 기존 필수 검증·최종화 잔여가 있다.** 현재 구현과 correction001을 감사한 결과이며, 미완료 GEX-10을 종료된 것으로 인정하거나 최종 동결 후 완료 감사를 대신하지 않는다.

## F-U164-IA002 — P2: 무억제 성공 XP가 레벨업 전에 포화되어 일부 소실

**발생 근거: 범위 안 — 지원 경로.** 정상 RON 작성 → `GameDataBase::load_live_sources` 검증 → 새 게임·직원 선택 → 입장비로 억제력 소진 → 공개 Continue → 합법 배치·실제 처치 → 첫 라운드 성공 철수다. 저장/checksum/내부 상태 변조는 사용하지 않았다.

첫 라운드에 `GrantExperience { amount: 2147483647, target: CombatParticipants }`를 작성하면 현재 스트레스 보상 로더가 승인한다. 고정 2배인 4,294,967,294 자체는 u32에 들어간다. 그러나 공통 성공 정산은 생존 XP 20을 먼저 지급한 다음 이 보상을 지급한다. `src/game/employee.rs:705`는 레벨 비용을 소비하기 **전에** `self.experience.saturating_add(amount)`를 적용하여 합계의 일부를 잘라낸다.

`XP_PROBE.rs`는 지원 성장 RON의 레벨당 비용을 1,000,000,000으로 작성하여 레벨업 반복을 최대 4회로 제한한다. 시작 레벨1/XP0, 생존 기본XP10, 보상량과 성공 조건은 유지한다. 독립 기대값은 `(레벨 - 1) × 1,000,000,000 + 잔여 XP`로 관찰한다.

| 대조 | 기대 총 XP | 실제 총 XP | 관찰 |
| --- | ---: | ---: | --- |
| 정상 성공 철수 | 2,147,483,657 | 2,147,483,657 | 출전 직원 2명 모두 일치 |
| 무억제 성공 철수 | 4,294,967,314 | 4,294,967,295 | breach_guard에서 19 소실, 첫 불일치에서 assertion 실패 |

이는 명령 거절·원자성 실패가 아니다. 성공 결과로 전환된 뒤 실제 레벨5/잔여XP294,967,295가 남으며, 기대 잔여XP는294,967,314다. 무억제 모드의 다른 직원은 첫 실패 이후 검사하지 않았으므로 모두 직접 관찰했다고 주장하지 않는다. 원시 증거는 `probes002.log`의 `AUDIT2_XP`와 실패 assertion이다. 정상 대조에서는 비출전 medic의 XP0도 확인했다.

상위 근거는 `docs/game_rules/growth-and-rewards.md`의 남은 XP 유지·직원별 지급·스트레스 전체전투1회 기본XP·무억제 총량2배 및 Goal A05/A08이다. 큰 수치이지만 지원 제작 입력이며 임의 변조 전용 사례가 아니다. 공식 기본 보상25/50XP가 실패한다고 확대하지 않는다.

**귀속:** `Employee::add_experience_with_policy` 공통 성장 owner의 기존 결함이다. `BASELINE_SOURCE_001.tar.gz`의 U16-4 착수 전 employee.rs와 현재 파일의 SHA-256이 완전히 같다(`XP_OWNER_ATTRIBUTION.json`). correction001이 새로 만든 회귀로 기록하지 않는다. 현재 스트레스 보상 경로가 이 결함을 노출하므로 U16-4 A05/A08의 전체 통과도 주장할 수 없다.

후속 GEX-10은 공통 XP owner에서 총 지급량과 레벨 비용 소비의 정확성을 교정하고, 실제 스트레스 정상/무억제 성공·기본 생존XP·공통 XP 지급 영향 범위를 확인해야 한다. XP를 지갑처럼 단순 총량 제한하는 방식이나 보상 기대값 축소를 감사 결론으로 요구하지 않는다. 이 감사에서는 제품을 교정하지 않았다.

## F-U164-IA001 교정 확인

`StressCombatPolicy::validate`는 같은 라운드의 반복 효과와 전체5라운드를 잔고별로 합산하고 정상/2배 표현 가능성을 검사한다. 가루 전용 효과와 `GrantEquipmentMaterial(equipment_dust)`는 공통 grant에서 같은 잔고를 쓰므로 함께 계산한다. 별도 재료ID는 분리한다. 모든 수량이 비음수여서 전체 합계 경계가 모든 prefix도 포함한다. 이전의 엔케팔린10억×5 승인 및 최종정산 불능 결함은 해소됐다.

교정 원시16개·RON25개·공개 패키징 초과 거절/허용 경계 승인 로그의 해시와 시험 이름을 검산했다. 현재 교정3파일과 보관 archive는 기록된 입력과 일치한다. 이번 사본 실행에서도 기존16개가 통과했다. 이 중 통화 정상2,147,483,647/무억제4,294,967,294의 실제5라운드 정산 대조를 포함한다.

신규 감사 검사는 가루의 두 표기를 섞고 별도 환상체 파츠를 함께 작성하여 각 잔고에 동일 경계값을 지급했다. 정상/무억제 양 모드에서 각각2,147,483,647/4,294,967,294를 받았고 유물은 각1개였다. 중간 미지급, 같은 battle UUID, 노드 소비, 종료 뒤 중복 명령 거절과 자산 불변, Core checkpoint import 후 잔고·유물 사본 보존을 확인했다. 실제 host/영속 저장 검증으로 확대하지 않는다.

따라서 F-U164-IA001의 잔고 누적 결함을 닫는다. XP의 별도 문제는 위 신규 finding으로 분리하며 과거 감사 원문은 보존한다.

## 입력·검토·실행 범위

- HEAD `2f725172edbffd27c5078d29000a1a2ba6ddc4d9`. 기존 dirty는 보존했다. HEAD 전체 diff를 U16-4 성과로 간주하지 않았다.
- `AUDIT_INPUT.json`/`AUDIT_INPUT.tar.gz`에 현재 입력724파일을 고정했다. 이전 감사 입력653파일 대비 변경은 제품1·시험1·계약설명1·PLAN/EXPERIMENTS 인계2파일이다(`CURRENT_DELTA.json`). 추가된 범위는 교정 증거·기존 시험 입력·최신 인계다. 이는 현재 감사 입력이며 Goal 최종 target을 대신하지 않는다.
- 기존 감사와의 실제 delta, 교정 patch, 정책 원문과 A01–A10, 수량 projection→공통배율→grant·지갑/재료·XP 소비자를 직접 읽었다. 변경되지 않은 wave gate/choice/생존XP/후퇴·복원 경계는 이전 판단과 현재 source를 대조해 재사용했다. 별도 reward owner·fallback·fixture 전용 제품 분기는 추가되지 않았다.
- `/tmp/u164-independent-audit002`에 고정 입력과 기존 `REPLAY_SUPPORT.tar.gz`를 복원했다. 복원 직후 모든 고정 입력과의 불일치는0이었다. 사본의 `u164_stress.rs`에만 correction001의 양성replay와 감사 probe를 추가했다. 입력은 실행 중 변경하지 않았다.
- 첫 실행 `probes001.log`:17실행/16통과/1실패/ignore0, exit101. 실패는 공식 유물 목록이 비어 있는데 첫 항목을 가정한 **감사 fixture 준비 오류**다. 실제 전투에 도달하지 않았으며 제품 finding으로 세지 않는다. 원문은 `PROBES.before_fixture_correction.rs`에 보존했다.
- 종료 후 지원 유물 RON fixture를 사용하도록 사본 준비만 교정하고 XP 검사를 추가했다. `probes002.log`:신규2실행/1통과/1실패/ignore0, exit101. 통과는 가루·파츠·유물 대조이며 실패는 위 XP 제품 결함이다. 전체18개를 한 번에 통과한 실행은 없다.
- 교정의 RON25와 공개패키징 양/음성은 새 실행이 아닌 유효 증거 재사용이다. 기존 broad regression의 고유2222/ignore6은 과거 입력의 이력이며 이번 전체 재실행·모든 계약 통과로 바꾸지 않는다. 선행 server fixture 실패1개도 그대로 남는다.
- 실행 세션은 종료했다. 출구에서 고정724파일 불변과 archive 일치를 확인했다(`AUDIT_EXIT_CHECK.json`). 이후 인계 문서 링크 갱신은 별도 비영향 기록으로 남긴다.

## 수용 조건별 현재 판단

| 조건 | 확인/재사용 근거 | 남은 완료 조건 |
| --- | --- | --- |
| A01 입장·첫 보상 | 기존 entry/공개 소진·Continue 양성 재실행, 현재 loader/entry owner | 승인 pack을 사용하는 정상 host |
| A02 선택·보상 공개 | 기존 시간 정지·stale round/명령 거절 재실행, query/capability owner 불변 | 실제 reconnect 및 관련 공개 행동 전체 대조 |
| A03 웨이브·5라운드 | 기존 지연2wave/5라운드와 신규 경계 정산 통과, gate/dependency owner 불변 | 최종 정상 host workload·확장 비용 |
| A04 상태 유지 | 같은 UUID/배치·선택 정지의 기존 시험, 새 검사 같은 UUID 확인 | 실제 쿨다운·전투당 기록·동시각 패배 대조 |
| A05 누적 지급 | IA001 닫음; 통화·가루·파츠 경계, 유물1개·중간 미지급·중복 거절 통과 | IA002 XP 소실, 최종 host 통합 |
| A06 패배 | 기존 실제 보호 대상 패배와 기본XP/누적상실 검사 통과 | 실제 직원 사망·장착유물 회수 통합 |
| A07 수동 후퇴 | 기존 초기/pending 후 후퇴·자산 복원·노드 소비 검사 통과 | 실제 소비품 수명·손실 복원과 paired 저장 |
| A08 무억제 | 정상 소진/Continue, 기본25/50XP·통화150/300·가루15/30 재실행; 신규 파츠2배/유물1개 | IA002, 전 라운드 적 HP/공격력 직접 관측 |
| A09 복원 | 기존/신규 Core export/import 대조 | 실제 reconnect/fresh process/SavePoint/paired IO 실패·동일명령 재시도 |
| A10 공개 경로·비용 | 교정 RON25·공개 패키징 양/음성 재사용, 새 focused 실행 | 정상/확장 시간·heap/RSS peak·회수·한도 판단, 최종 영향 통합/coverage/target/정역복원 |

Validation Dimension의 success/live_data/boundary/retry/producer-consumer는 위 직접 근거가 있는 부분만 확인했다. failure_rollback/disconnect_restart_persistence/ordering_concurrency는 표의 기존 필수 잔여가 남으며, legacy/fallback absence의 변경 비영향 검토를 전체 회귀 PASS로 바꾸지 않는다. `target/`은 비어 있고 `COVERAGE.json`과 GEX-10 최종 종료 기록은 아직 없다. 기존10tick pilot·옛 release build·실패한 instrumented build는 최종 host/비용 증거가 아니다.

## 다음 작업

현재 감사는 종료한다. owner의 별도 GEX-10에서 IA002와 영향 검증을 처리하고 기존 host/저장/수명·동시각/비용 검증을 완성해야 한다. 그 뒤 coverage·최종 target·정역복원을 준비하고 독립 완료 감사를 받는다. 이번 감사 승인으로 U17/U21/Unity/개별최종보스 또는 제품 교정을 자동 시작하지 않았다.

재현 명령과 사본 복원은 [REPRODUCE.md](REPRODUCE.md), 원시 실행 identity는 [RESULT.json](RESULT.json)을 따른다.
