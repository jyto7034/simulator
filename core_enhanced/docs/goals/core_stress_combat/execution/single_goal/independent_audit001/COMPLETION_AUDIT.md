# U16-4 현재 구현 독립 감사 — Not Complete

2026-10-15. 대상은 GS-CSCB-001 / GOAL-CSCB-01이다. 사용자의 “CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md 문서를 지침 삼아 u16-4 독립 감사 작업에 착수”, “현재 명령을 지침서보다 더 우선시” 요청에 따른 별도 감사 실행이다. 이번 감사자는 기존 U16-4 구현·교정에 참여하지 않았으며 제품을 수정하지 않았다. 서브에이전트는 소환하지 않았다.

**판정: Not Complete.** 지원 작성 입력에서 최종 정산 실패 1건을 재현했고 필수 호스트·저장·비용 검증이 남아 있다. 이것은 사용자가 지금 요청한 현재 구현 감사다. GEX-10 종료/최종 target/정역복원이 없는 상태를 완료된 것으로 소급 인정하지 않으며, 최종 동결 후 완료 감사를 대체하지 않는다. 구현 교정이나 후속 단계는 이번 감사에서 수행하지 않았다.

## 입력·독립성·현재성

- 근거: 최신 사용자 요청 → 연속 실행 지시서 → HANDOFF → PLAN/EXPERIMENTS → P2G-50 Goal/동결 A01–A10 → RULES 및 `game_rules/relic-reward-nodes.md` 스트레스 전투·저장 규칙. GEX r8의 읽기 전용 감사·발견 귀속·완료 판정 원칙을 적용했다.
- 저장소 `/mnt/f/work/simulator`, HEAD `2f725172edbffd27c5078d29000a1a2ba6ddc4d9`. 누적 dirty가 있으므로 HEAD diff를 U16-4 변경으로 간주하지 않았다. 실제 착수 전 `BASELINE_SOURCE_001`과 대조한 수정 34파일은 `CURRENT_BASELINE_DELTA.json`/`CURRENT_BASELINE_DIFF.patch`에 결속했다. 신규 stress 정책·시험·host도 직접 검토했다.
- `AUDIT_INPUT.json`/`AUDIT_INPUT.tar.gz`는 현재 제품·시험·데이터·계약·Goal/실행 기록 653파일의 감사 입력이다. 구현자가 완료 후 발행한 정식 Audit Target Record는 아니다. 별도 감사 증거와 실행 산출물은 이 입력에서 제외한다.
- 기존 격리 사본 `/tmp/u164-build`의 대응 입력은 현재 원본과 차이 0이었다. 이를 `/tmp/u164-independent-audit001`로 복사한 뒤 해당 사본의 `u164_stress.rs`에만 감사 probe를 추가했다. 제품 코드·정책·공식 데이터·기존 시험 원본은 변경하지 않았다. `ISOLATION.json`, `PROBES.rs`, `DIAGNOSTIC_PROBE.rs` 참조.
- 감사 출구에서 653파일 모두 원본 불변, archive 각 member의 digest 일치를 확인했다(`AUDIT_EXIT_CHECK.json`). 이후 PLAN/EXPERIMENTS에 붙이는 감사 인계 링크는 결과 안내용 비영향 문서 delta로 별도 기록하며, 동결 전 원문은 archive에 보존한다.

## F-U164-IA001 — P2: 개별 수량만 검사하여 지급 불가능한 누적 보상을 승인

**발생 근거: 범위 안 — 지원 경로.** 정상 RON 작성 → 공개 패키징/ValidatedGameData 로더 → 정상 노드 진입·배치·전투 진행 → 5라운드 정산이다. 저장파일/checksum/런타임 메모리 변조를 사용하지 않는다.

`src/game/stress_combat.rs:129`–142는 각 RewardEffect의 `amount.checked_mul(2)`만 검사한다. 그러나 실제 성공 지급은 `src/game/world/combat.rs:1362`–1365에서 완료한 모든 라운드 보상을 하나의 ClaimAll에 모은다. 공통 `src/game/resources/economy.rs:15`–19의 지갑은 u32 checked_add를 사용한다.

재현 입력은 라운드마다 엔케팔린 1,000,000,000이다. 각 효과의 2배는 u32에 들어가지만 5라운드 합계 5,000,000,000은 빈 지갑에서도 저장할 수 없다. 공개 `package_official_content`가 공식 입력의 다섯 수량만 바꾼 pack을 exit 0으로 승인했다(`cumulative_reward_source/`, `cumulative_reward_pack/`, `public-package001.log`, `PUBLIC_PACKAGE_INPUT.json`). 이 승인은 오류 입력의 수용 증거이며 기능 PASS가 아니다.

현재 제품의 격리 실행에서도 같은 수량을 정상 작성한 콘텐츠가 로드됐고 1–4라운드를 실제 처치·계속 선택으로 통과했다. 5라운드 마지막 정산 tick에서 `InvalidAction`이 발생한다. 같은 tick을 재시도해도 실패하고 성공 결과로 전환하지 못한다. candidate/자산은 원자적으로 보존돼 부분 지급은 없으며 수동 후퇴는 가능하지만 누적 보상을 포기하고 노드가 소비된다. 데이터 손상이나 부분 지급 결함으로 확대하지 않는다.

원시 근거:

- `probes001.log`: 기능 검사 13개 중 12통과/1실패, exit 101. 실패는 `u164_independent_cumulative_authored_rewards_are_payable`이며 실제 정산의 `InvalidAction`이다.
- `diagnostic001.log`: exact 진단 1개, exit 0. `round:5`, `completed_before_error:4`, `authored_total:5000000000`, `wallet_max:4294967295`, `retry_failed:true`, `roots_unchanged:true`를 확인했다. **오류 재현 진단의 성공이지 해당 기능의 정상 통과가 아니다.**
- 재현 방법은 [REPRODUCE.md](REPRODUCE.md). 기대값과 실행 전 가정은 [PROBE_PLAN.md](PROBE_PLAN.md).

영향 조건은 A05/A08/A10이다. 교정 owner는 U16-4 GEX-10의 `StressCombatPolicy::validate`와 누적 보상 검증이다. 같은 지급 대상별 누적 수량의 표현 가능성을 정상/무억제 배율에서 확인하고, 지원 불가능한 고정 보상표를 게임 시작 전 거절해야 한다. 경계·정상 대조와 최종 정산을 영향 범위에서 재검증한다. 본 감사의 실제 수량 재현은 엔케팔린에 한정하며, 가루·동일 재료·XP의 누적 경계는 owner가 영향 검토할 항목이다. 일반 보유량 증가 전반을 새 감사 범위로 확장하지 않는다.

## 수용 조건별 확인과 잔여

| ID | 직접 확인한 현재 근거 | 남은 완료 조건 |
| --- | --- | --- |
| A01 입장·첫 보상 | 로더/지원 활동/entry preview/전투 생성 경로, 기존 집중검사 및 신규 공개 행동으로 비용12 소진 후 Continue 진입 | 승인 pack을 소비하는 정상 host 실행 |
| A02 선택·고정 보상 | capability와 UUID/round/waiting 재검사, 기존 대기 시계·자원 불변과 pause/resume/retreat 거절·오래된 round 거절, 보상 projection | 모든 관련 공개 전투 행동 거절, 실제 host reconnect 대조 |
| A03 웨이브·5라운드 | 현재 코드의 미래 라운드 gate, 기존5라운드 성공/미래 actor 의존 거절. 신규 2웨이브 첫 라운드는20초에 미완료,30초 추가 wave 처치 후에만 완료·철수10 | 최종 정상 host workload, 확장 비용 |
| A04 상태 유지 | 같은 BattleCore/UUID/배치/queue 유지 및 선택 중 elapsed 정지. timestamp bucket 처리 후 round/terminal 순서와 패배 우선 코드 확인 | 실제 쿨다운·전투당 사용 제한·동시각 패배 대조 |
| A05 누적 지급 | 신규 정상 통화150/가루15/출전 생존자 XP25, 중간 무지급, 비출전 medic XP0. 기존 첫철수10/중복 거절 | F-U164-IA001; 환상체 파츠·유물 통합, 정상 host |
| A06 패배 | 기존 실제 적의 보호 대상 파괴·누적 상실·기본 생존XP10, common 직원 결과/유물 회수 연결 검토 | 실제 직원 사망·장착유물 회수와 손실의 통합 대조 |
| A07 수동 후퇴 | 기존 초기 후퇴와 신규 정상 무억제 진입→1라운드 미수령→계속→후퇴의 자산 복원·노드 소비·Core checkpoint 복원 | 실제 소비품 활성 기간/손실 복원, paired IO와 실제 재시작 |
| A08 무억제 | 내부 상태 수정 없이 작성 입장비12로 초기 억제력12 소진→공개 Continue.5라운드 통화300/가루30/출전 XP50, 중간 무지급. 일반량 대조 | 전 라운드 적 HP/공격력 실제 관측, 파츠2배·유물 수량 불변, F-U164-IA001 영향 검증 |
| A09 복원 | entry snapshot/완료 후 Continue 갱신·결과 owner 연결 및 기존/신규 Core export/import | 실제 reconnect/fresh process/SavePoint/paired IO 실패/동일명령 재시도 |
| A10 공개 경로·비용·회귀 | 지원 RON·공개 패키징 음성/양성, 기존 영향 회귀 재사용과 새 focused 실행 | 정상/확장 host 시간·heap/RSS peak/종료 후 회수·한도 판단, final coverage/동결/정역복원 |

검증 누락은 이미 PLAN에 남아 있던 필수 항목이며 새로운 전면 시험 행렬을 추가한 것이 아니다. 신규 감사 probe는 일부 Core 검증을 보강했으나 원래 host·영속 저장·최종 baseline 책임을 대신하지 않는다. 공식 기본 편대의5라운드 밸런스나 U21 완료도 주장하지 않는다.

## 증거 재사용과 실행 결과

`RE_REVIEW_RESULTS_001.json`이 가리키는 원시 로그9개의 해시를 모두 확인했다(`EVIDENCE_REUSE_CHECK.json`). 실제 성공 시험 이름의 합집합으로 lib2150 + integration72 = 고유2222를 다시 계산했다(`REGRESSION_REUSE_DETAIL.json`). ignore6은 성공 수에 포함하지 않는다. 집중9는2222에 포함되며 단일 all-targets green도 아니다. 이번 감사의 기존9 재실행도 그 수에 더하지 않는다.

서버 메시지8통과/1실패 중 `loan_copy_ids` 누락은 착수 전 archive의 필수 명령 필드와 동일 시험 입력에서 직접 확인했다. U16-4 새 명령의 parsing 및 setup/update/resync 통과를 전체 서버 PASS로 확대하지 않았다. 이전 release 빌드와 공간 부족 instrumented 실패는 최종 host/비용 증거에서 제외했다. host.rs의 실제 os error21 검사와 원자성 확인 코드는 읽었지만 시나리오 실행 결과는 없다. `normal_suite.json`은10tick pilot이며 전체5라운드·저장·비용 oracle이 아니다.

새 실행은 한 번의 focused cargo 선택(9개 기존+4개 감사)으로13개를 실행해12통과/1실패했다. 빌드1m22s, 시험9.22s. 이후 해당 오류의 exact 진단만 실행해1건 재현을 확인했다(빌드40.10s, 시험4.72s). 공개 패키징1건은 잘못된 입력을 승인했다. 모든 세션은 종료·회수했다. 이 개발 profile 시간은 제품 성능 합격선이나 정상/확장 비용 측정이 아니다.

## 종료·인계

제품 finding 1건과 필수 검증/최종화 미완료로 U16-4는 **Not Complete**다. GEX-10 owner가 F-U164-IA001을 교정하고 필요한 host/저장/비용 증거를 완성한 뒤 coverage·최종 target·정역복원을 준비해야 한다. 이후 별도 읽기 전용 완료 감사에서 currentness와 변경 영향 및 남은 조건을 확인한다. 이번 입력·실패 원문·판정은 보존한다. 기존 U16-3 완료 감사와 다른 dirty 작업은 변경하지 않았다. U17/U21/Unity/개별 최종보스에는 착수하지 않았다.
