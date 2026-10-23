# U16-4 재검토 기록

2026-10-15 사용자의 “지금까지 한 것(U16-4 한정) 모두 재검토” 지시에 따른 root 재검토다. 이전 구현자의 결론을 승계하지 않고 규칙·코드·시험 기대값·원시 결과를 대조했다. 이 검토는 구현에 관여한 root의 검토이며, 원래 요청한 **완수 후 비참여 서브에이전트 감사는 아직 수행하지 않았다**. U16-4 전체 완료 판정은 보류한다.

## 검토 입력과 범위

- 상위 근거: 연속 실행 지시서, ROADMAP U16-4, 룰북 `relic-reward-nodes.md`의 스트레스 전투·저장 규칙, 기존 공통 XP·후퇴·결과/저장 구현.
- 실제 착수 전 `BASELINE_SOURCE_001.tar.gz`의532파일과 현재 파일을 비교하여 변경32파일을 확인했다. 신규 정책·시험·호스트와 별도 probe manifest를 더한36파일의 교정 전 원문은 `RE_REVIEW_INITIAL_001.tar.gz`; 차이는 `REVIEW_U164_DIFF.patch`다. 저장소 전체 git diff는 이전 U00–U16-3 dirty가 섞이므로 U16-4 귀속 근거로 사용하지 않았다.
- 구현 전 경계 검토·P2G00–60·Goal·실행계획·상태수명·ACCEPTANCE, 공식 콘텐츠, 지원 외부 pack과 calibration 시나리오, normal host adapter, 기존 모든 U16-4 실행 로그를 검토했다.
- 호출 경계: layout 로딩 → 활동 지원/입장 안내 → live battle 생성 → kernel wave gate/round 선택 → 명령 권한/행동 자격/DTO → 공통 성공·패배 정산/XP/유물 회수/소비품 기간 → 수동 후퇴 snapshot+node 완료 → Continue/SavePoint/paired host.
- 기존 함수에 연결한 효과의 의미도 확인했다: 판정 시각 bucket의 사건을 정리한 후 round 표시, 패배 우선 terminal 정책, 단일 BattleCore/UUID, 기존 생존자 선별 XP와 성공 배율, 정상 후퇴와 완료 후퇴의 Continue 분기.

## 발견과 교정

| ID | 발견 | 발생 근거·재현 | 조치 |
| --- | --- | --- | --- |
| R01 | 일반 지속 보호막이 스트레스 보호 대상에 적용됨 | 지원 `TileArea/Any` → `BeginSustained` → shield absorption. 기존 검사에는 이 경로가 없었고 새 검사는 기대0 대신 실제50피해를 흡수하여 실패 | 지속 효과 owner에서 아군 보호 대상 제한. 일반 전투는50흡수, 스트레스는0흡수 대조 통과. 명시 수리는 계속 허용 |
| R02 | 무억제2배로 표현 불가능한 후반 보상을 로더가 승인 | 정상 RON의5라운드 보상 u32::MAX를 로딩하면 성공. 이후2배 안내/정산의 checked_mul에서 거절될 입력 | 통화·가루·파츠·XP의 고정2배 표현 가능성을 publication에서 검증. 런타임 overflow 검사도 유지 |
| R03 | actor trigger가 미래 라운드 적을 기다리는 입력 승인 | round1이 round5 actor의 HP100%를 기다리고 round5는AtTime1000. 전역 DAG에는 cycle이 없으며 수정 전 공개 패키징 exit0. round gate가 round5를 잠가 round1 진행 불가 | GroupCleared뿐 아니라 모든 actor trigger의 바인딩 원천 wave도 현재/앞 라운드인지 검사. 앞 라운드 actor 참조 양성 대조도 통과 |
| R04 | 기존 enum match와 명령 개수 검사 갱신 누락 | ron_loading 컴파일 오류, Application/owner 테스트85 기대에 실제86 | StressCombat 분기와86개 exhaustive assertion 반영. 전수 command 매핑 검사는 유지 |
| R05 | 새 wave 그룹 문자열/다음 보상 동적 projection의 사전 비용 경계 누락 | 새 `collect/format` 및 next_rewards clone의 직접 호출 확인 | native allocation admission 추가, 공통 보상 projection 사용. 실제 peak/회수·확장 비용 증거는 아직 필요 |
| R06 | 준비한 host 검사가 단순 명령 거절을 IO 장애로 오인할 수 있음 | fail:true는 is_err만 확인. stop만으로 passed/fresh_process:true 출력 | 실제 주입한 os error21 확인, reject의 disk 비교 추가, 결과는 protocol_completed만 표시. 상위 시나리오가 기능·fresh process 복원을 별도로 판정 |
| R07 | 실행용 수용 ID 불일치와 이전 딜러 calibration 잔재 | ACCEPTANCE의A02–04 순서가 동결 P2G와 다름. dealer_dice_transaction은 지금 StressCombat인8번 노드에서 Offer를 기대 | 기존 파일 보존 후 파생 수용 색인을 동결 기준으로 맞춤. calibration은 첫 pilot만 유지하고 최종 workload 미검증 명시 |

R01–03은 실제 지원 효과/콘텐츠 작성 경로의 문제이며 외부 save/checksum/메모리 임의 변조를 가정하지 않는다. R04는 검사 갱신 누락, R06–07은 검증 도구/기록 문제다. 정적 수정이나 빌드 성공을 기능 합격으로 확대하지 않았다.

## 기존 증거의 재평가

- focused009의6통과는 작은 Core/kernel 검증이다. 인메모리 checkpoint export/import는 실제 파일 저장 및 새 프로세스 Continue 증거가 아니며, snapshot 조회는 실제 reconnect 호출 증거가 아니다.
- 기본 편대의1라운드 성공 철수와 작성된 공격80 편대의5라운드 성공을 구분한다. 후자는 기본 편대의 전체5라운드 밸런스 합격이 아니다. 내부에서 억제력0으로 준비한 것은 배율 로직 대조이며, 정상 소진/계속 선택의 공개 경로 증거는 아직 없다.
- 이전 core001은 컴파일/링크 실패로 검사 실행0; normal-build001은 빌드만 성공; instrumented-build001은 공간 부족 실패다. 정상 host·계측·비용 PASS는 없었다.
- GEX00은 착수 전 소스 보관은 실제로 했지만 정식 P2G/GEX artifact 출력은 제품 변경 후 첫 컴파일 중이었다. 사후 출력으로 착수 순서가 충족된 것처럼 소급 표기하지 않는다. 이번 규칙/코드 재검토를 재개 판단 근거로 남긴다.
- /tmp 고갈로 sandbox 명령 시작도 실패했다. 승인받아 빌드 cache를 작업공간으로 이동했으며 삭제하지 않았다. 격리 빌드의 docs2파일 및 런타임 G11 fixture 누락도 원본·receipt 그대로 보충했다.

## 재검사

원시 로그는 `runs/`에 보존한다. 수치와 범위는 아래 후속 결과란 및 EXPERIMENTS의 최종 기록을 따른다.

- review-focused001: 8실행/6통과/2실패. R01/R02를 교정 전에 검출.
- review-actor-public001: 수정 전 정상 package_official_content exit0. 잘못된 미래 actor 의존을 승인한 음성 증거이며 합격이 아니다.
- review-focused002: 시험의 expect_err/Debug 타입 가정 오류,0실행; assertion API를 교정.
- review-focused003: 9실행/9통과/0실패/0ignore,9.23s. 기존6검사와 R01–03 대조.
- review-core001: lib2143통과/7실패/6ignore,216.75s. 실패는 count2·sandbox socket1·fixture 누락4; integration은 이 명령에서 실행되지 않음.
- review-regression-fixes001: 위 count2·fixture4를 교정한6검사 모두 통과(4.91s).
- review-socket001: 제한 밖 exact1검사 통과(0.08s). 따라서 lib 고유2150통과, ignore6. 단일 전체 green 실행이라고 쓰지 않는다.

## 수용 조건별 잔여

| 동결 ID | 현재 판단 |
| --- | --- |
| A01 입장·첫 보상 | 실제 로더/Core 진입 대조 확인. 정상 host 경로는 잔여 |
| A02 선택·고정 보상 | 시계 동결·기본 자격·오래된 round 거절 확인. 공개 host의 모든 전투 행동 거절 및 배율 반영 전체 보상 검증 잔여 |
| A03 웨이브·5라운드 | 작성5라운드 성공·미래 의존 거절 확인. 라운드당 여러 웨이브 실제 진행 대조 잔여 |
| A04 상태 유지 | 같은 core/UUID·배치 유지·대기 중 kernel/queue/자원 불변 확인. 실제 쿨다운·전투당 사용 제한·동시각 패배 대조 보강 잔여 |
| A05 누적 지급 | 통화150/300·첫 철수10·중복 지급 거절 확인. XP/가루/파츠/유물 통합 대조 잔여 |
| A06 패배 |1라운드 미수령 후 실제 적의 보호 대상 파괴·통화 상실·생존 기본XP10 확인. 실제 직원 사망/유물 회수 통합 잔여 |
| A07 수동 후퇴 | 초기 진행 후 자산 복원·노드 소비·최초비용/receipt·재도전 거절·Core Continue 확인. pending 후 후퇴·실제 소비품 활성 기간·paired IO 잔여 |
| A08 무억제 | 내부 준비 배율·성공 통화·패배 기본XP 확인. 정상 소진 경로와 전 라운드 적 강화, 모든 해당 보상량 및 유물 수량 검증 잔여 |
| A09 복원 | 기존 공통 owner 연결과 Core export/import 대조. 실제 reconnect/fresh process/SavePoint/IO 실패·동일명령 재시도 잔여 |
| A10 공개 경로·비용·회귀 | 이번 영향 회귀는 아래 결과로 결속. 정상/확장 host 비용·실제 peak/종료 후 회수·한도 판단·최종 고정/정역복원·비참여 감사 잔여 |

제품 결함을 찾고 교정한 것과 U16-4 전체 기능을 완수한 것은 구분한다. 정상 호스트와 저장·자원 검증, 최종 target 고정이 끝난 뒤 처음 요청한 비참여 서브에이전트 감사에 착수한다.


## 재검토 종료 결과

`RE_REVIEW_RESULTS_001.json`이 원시 로그의 수치·해시를 연결한다. Core lib2150 + integration72 = **고유2222통과, ignore6**. 집중9는 합계에 포함된다. 오류 원본을 남긴 뒤 필요한 범위만 교정·재실행한 합산이며 단일 all-targets green이 아니다. integration001의 ron_loading 보상 owner 오라클도 실제5라운드 고정보상 확인으로 갱신했고 integration002에서25검사 모두 통과했다. 공개 패키징 도구의 미래 actor 오류 입력도 수정 전수용/수정후거절을 확인했다.

서버 메시지는9개 중8통과/1실패다. 실패는 기존 SelectStarterEmployees fixture가 필수 `loan_copy_ids`를 생략한 것으로, 착수 전 baseline의 명령 필드와 시험 원문에도 동일한 불일치가 있다. U16-4의 새 명령 parsing과 전투 setup/update/resync 경로는 통과했다. U16-4 한정 요청에 따라 선행 fixture를 무관하게 수정하지 않았다. 서버 전체 PASS로 표기하지 않는다.

재검토를 마친 현재 U16-4는 **구현·집중/영향 회귀 교정 진행 상태, 전체 미완료**다. 실제 host/저장·비용·최종고정·비참여감사는 위 잔여 항목대로 이어갈 대상이다. 이전 빌드 결과는 이번 제품 수정 이후의 최종 증거로 승격하지 않는다. 작업 중인 검사는 모두 종료·회수했다.
