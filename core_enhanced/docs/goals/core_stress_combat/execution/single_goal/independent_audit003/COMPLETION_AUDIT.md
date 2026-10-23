# U16-4 독립 재감사003 — Not Complete

2026-10-15. GS-CSCB-001 / GOAL-CSCB-01. 현재 실행 지시서에 따른 독립 감사라는 사용자 요청을 수행했다. 이번 실행자는 기존 구현·교정에 참여하지 않았고, 제품·공식 콘텐츠·원본 시험을 수정하지 않았다. 서브에이전트를 사용하지 않았다.

**F-U164-IA002(XP 소실)는 교정 확인으로 닫는다. 이전 F-U164-IA001의 종료도 유지한다. 확인한 범위에서 새 제품 결함은 없지만, 필수 검증과 최종화가 남아 Goal 판정은 Not Complete다.** 이번 결과는 correction002 이후 현재 구현의 독립 재감사다. GEX-10 최종 종료와 최종 동결 후 완료 감사를 대신하지 않는다.

## XP 교정의 독립 확인

발생 근거는 이전과 동일한 **범위 안 — 지원 경로**다. 정상 RON 로더 승인 → 새 게임·직원 선택 → 정상 입장 또는 입장비로 억제력 소진/공개 Continue → 합법 배치·실제 처치 → 첫 라운드 성공 철수로 관찰했다. 원래 감사의 XP_PROBE.rs를 입력·행동·기대값 변경 없이 새 사본에서 실행했다.

| 모드 | 출전 직원 각각의 기대 총 XP | 실제 총 XP | 실제 레벨 / 잔여 XP |
| --- | ---: | ---: | --- |
| 정상 | 2,147,483,657 | 2,147,483,657 | 3 / 147,483,657 |
| 무억제 | 4,294,967,314 | 4,294,967,314 | 5 / 294,967,314 |

scout_runner와 breach_guard 모두 일치했다. 비출전 field_medic은 XP0, 라운드 선택 중에는 전 직원 XP0이었다. 수작업 기대값은 생존10+보상2,147,483,647 및 각각의2배다. 비용10억일 때 `(level−1)×10억+잔여`로 지급 총량을 독립 관찰한다. 무억제19XP 소실이 해소됐다.

`Employee::add_experience_with_policy`의 두 u32 입력 합계는 최대8,589,934,590이므로 u64 합계에 정확히 들어간다. `RunPolicyData::validate`는 비어 있거나0인 비용을 거절하고, `xp_required_for_next_level`은 작성 곡선 또는 마지막 양수u32 비용을 반환한다. 매 반복마다 양수 비용을 차감하므로 종료 잔여는 그 비용보다 작고 u32 저장도 정확하다. 기존 level의 saturating 증가 의미는 유지된다. 이 정합성 검토를 큰 입력의 실행 비용 검증으로 해석하지 않는다.

공통 grant의 대상 선택·중복제거·직원별 전액 지급·결과 diff와, 전투 성공 배율→생존 XP→누적 보상 정산 경로를 직접 읽었다. 저장 필드/DTO·보상량·대상·곡선·소유권 변경이나 스트레스 시험 전용 우회 분기는 없다. 상위 근거는 `docs/game_rules/growth-and-rewards.md`의 잔여 유지·직원별 지급·전체 전투1회 기본XP·무억제 총량2배 및 A05/A08이다.

## 실행과 근거 재사용

- 새 사본 `/tmp/u164-independent-audit003`에서 공통 XP·생존XP·U14·U16-4·보상 실패 원자성 선택 **49통과 / 0실패 / 0ignore, exit0**. 실제 검사19.40초. `replay001.log`와 `RESULT.json`이 원시 출력·고유 시험명·종료 코드를 연결한다.
- 영구회귀3개, 대상별 지급/빈 대상/누락 context/level 포화, 작은 정상25/무억제50XP, 패배·후퇴, 실제5라운드 통화 경계와 가루alias·파츠 경계/유물1개를 포함한다. 후자의 Core checkpoint 및 중복 명령 거절도 통과했다. 내부 fixture로 만드는 일부 기존 경계 검사는 정상 host 진입 증거로 세지 않았다. 정상 소진/Continue와 XP 재현은 지원 작성·행동 경로를 사용했다.
- correction002의 수정 전 1통과/2실패 및 수정 후49통과 원시 로그를 해시·시험명으로 검산했다. 교정 전후2파일과 AFTER archive도 일치한다. 이번 새 실행은 과거 green 로그를 재명명한 것이 아니다.
- correction001 로딩25개·공개 패키징 초과 거절/허용 경계 승인 증거는 원시 해시와 현재 로더 비변경을 확인하여 재사용했다. 이전 감사의 IA001 종료 판단은 유지하며 관련 정산 양성도 이번에 재실행했다.
- 이전 broad regression 로그의 해시를 재확인했다. 과거 고유2222/ignore6은 현재 전체 재실행이 아니다. 서버 메시지의 선행 loan_copy_ids fixture 불일치1건은 여전히 역사적 실패이며, 이번 감사가 서버 전체 통과를 확인한 것은 아니다.

## 감사 입력과 소스 검토

`AUDIT_INPUT.json`/`AUDIT_INPUT.tar.gz`는 현재753파일을 식별한다. 이전724파일 대비 변경은 employee.rs(제품+영구회귀), core_runtime_contract.md(설명), PLAN/EXPERIMENTS/HANDOFF(현재 기록)다. 추가29파일은 교정/이전 감사 증거다. 제품·서버 src, Core tests, 공식 data에서 manifest 밖 파일이 없는 것도 확인했다. 기존 dirty를 U16-4 변경으로 합산하거나 되돌리지 않았다.

`CURRENT_DELTA.json`과 correction002/CHANGES.patch를 직접 대조했다. XP owner 외에도 현재 누적 보상 validation/projection→공통 grant, 라운드 gate/선택 명령, 같은 timestamp 처리 후 terminal 판정, 후퇴 Continue 갱신을 읽었다. 변경되지 않은 구현의 기존 판단은 동일성에 근거해 재사용하고, 새로운 실증이 없는 수명·host·동시각 조건은 아래에 남겼다. wave gate가 같은 BattleCore를 유지한다는 코드 사실만으로 모든 쿨다운·전투당 기록 보존을 실증했다고 주장하지 않는다.

사본 복원 직후753파일 해시 불일치0. 감사 probe3파일을 사본에만 추가한 stress 시험 파일은 correction002의 시험 입력과 해시가 같다. 감사 출구에서 원본753파일 불변과 archive 일치를 확인했다(`AUDIT_EXIT_CHECK.json`). 이후 요약·인계 갱신은 별도 `POST_AUDIT_DOCUMENTATION.json`에 귀속한다. `target/`은 비어 있고 `COVERAGE.json`과 최종 GEX-10 종료 기록은 준비되지 않았다.

## 완료 조건별 판단

| 조건 | 확인·재사용 범위 | 남은 필수 검증 |
| --- | --- | --- |
| A01 입장·첫 보상 | 현재 loader/entry, 정상 소진·Continue 양성 재실행 | 승인 pack의 정상 host 통합 |
| A02 선택·보상 공개 | 시간 정지, stale/중복 명령 거절, query owner 비변경 | 실제 reconnect와 공개 행동 전체 대조 |
| A03 웨이브·5라운드 | 지연 다중wave/실제5라운드/경계 정산 재실행 | 최종 정상 host workload와 확장 비용 |
| A04 상태 유지 | 동일 battle UUID/배치·선택 정지, kernel owner 검토 | 실제 쿨다운·전투당 기록, 동시각 패배 대조 |
| A05 누적 지급 | IA001·IA002 닫음; 중간 미지급/통화·가루·파츠·유물/중복 거절 | 최종 host 통합 및 저장 원자성 |
| A06 패배 | 보호 대상 패배·누적 상실·기본 생존XP 재실행 | 실제 직원 사망·장착유물 회수 통합 |
| A07 수동 후퇴 | 초기/pending 이후 후퇴·자산 복원·노드 소비 대조 | 실제 소비품 수명·손실 복원·paired 저장 |
| A08 무억제 | 정상 소진/Continue, 정상·2배 XP와 재료·통화/유물 수량 | 전 라운드 적 HP/공격력 직접 관측 |
| A09 복원 | Core export/import와 후퇴 Continue owner 비변경 | 실제 reconnect/fresh process/SavePoint/paired IO 실패·동일명령 재시도 |
| A10 공개 경로·비용 | 작성/공개패키징 근거 재사용·선택49개 새 실행 | 정상/확장 시간·heap/RSS peak·회수·한도 판단, 최종 영향 통합/coverage/target/정역복원 |

Validation Dimension 중 success/invalid_input/boundary/live_data/retry/producer-consumer는 위 관찰 경계만 인정한다. failure_rollback·ordering_concurrency·disconnect_restart_persistence의 실제 host 경계는 미완료다. legacy/fallback absence의 비영향 소스 검토를 전체 회귀 통과로 확대하지 않는다. Goal Set의5개 IVA와 최종 통합 소유 책임은 그대로 남는다.

위 잔여는 기존 승인 수용 조건의 미검증이다. 외부 임의 변조 방어를 새 finding·필수 시험·완료 차단으로 추가하지 않았다. 이번 범위에서 발생 근거가 확인되지 않은 의심을 제품 결함으로 등록하지 않았다.

## 반환 경계

현재 독립 감사는 종료한다. 다음 owner 작업은 GEX-10에서 기존 host/저장/수명·동시각/비용 검증을 완성하고 coverage·최종 target·정역복원·종료 기록을 준비하는 것이다. 그 고정 입력으로 독립 완료 감사를 받아야 한다. XP 교정 재작업이나 새 제품 finding 교정은 이번 결과로 요구하지 않는다. U17/U21/Unity/개별최종보스는 시작하지 않았다.

재현은 [REPRODUCE.md](REPRODUCE.md), 검산한 기존 증거는 [EVIDENCE_REUSE_CHECK.json](EVIDENCE_REUSE_CHECK.json)을 따른다.
