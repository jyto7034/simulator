# U16 최종 종합 감사 — Not Complete

판정: **Not Complete / High confidence**. 지원 작성 입력에서 공통 XP 지급의 미계량 반복 계산을 재현한 **F-U16-FINAL-001 (P2)** 1건이 열려 있다. 기존 기능 회귀 45개와 과거 네 묶음의 증거 검산은 통과했으나, 그것으로 자원 경계의 결함을 면제하지 않는다. 감사 요청은 수행했으며 제품 교정은 수행하지 않았다.

## 요청·범위·고정 입력

2026-10-15 사용자 “CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md 문서를 지침 삼아 u16 최종 종합 감사 작업에 착수” 요청을 인수했다. 네 묶음은 각자 `single_phase` Goal Set이며 U16 전체를 위한 별도 multi-phase Master는 없다. 존재하지 않는 Master/P2G 계약을 만들지 않고 로드맵 U16 카드의 네 묶음·공유 경계·완료 조건을 검토했다. 기존 네 GEX-20과 후속 교정의 현재성을 확인하고 GEX 최종 통합 검토 기준을 적용했다.

현재 실행자는 기존 구현/교정에 비참여이며 단독 수행했다. 제품·기존 시험·공식 데이터·정책·과거 동결/감사는 읽기 전용으로 유지했다. 새 probe는 감사 폴더의 별도 crate와 복제한 지원 RON만 사용한다. 감사의 read-only 범위는 구현 수정을 승인하지 않는다는 GEX Reader Contract execute routing 7에 따른다. U17/Unity/U21/최종 배포 SLO·밸런스와 외부 임의 변조 전용 방어는 범위 밖이다.

현재 입력은 [ENTRY.json](ENTRY.json)의 4,810개 파일과 [현재 소스 보관](CURRENT_SOURCE.tar.gz)에 결속했다. 현재 소스 보관 1,939개 member의 정확한 바이트 동일성을 [ARCHIVE_CHECK](ARCHIVE_CHECK.json)에서 확인했다. HEAD는 `2f725172edbffd27c5078d29000a1a2ba6ddc4d9`이다. 기존 dirty 전체를 이번 감사의 변경으로 귀속하지 않는다. 현재 source 보관은 과거 baseline을 재구성한 것이 아니며, 과거 target의 정역복원은 보관 archive/delta를 검산하고 기존 실제 격리복원 결과를 재사용했다. 출구 확인은 [EXIT](EXIT.json)를 따른다.

| 이전 target | source member / 현재와 다른 수 | evidence member / 달라진 수 | 현재성 판단 |
| --- | --- | --- | --- |
| U16-1 `u16_1_current_001` | 1006 / 54 | 963 / 0 | 기본 보상·딜러·스트레스의 후행 변경과 교정에 귀속. 당시 source와 동일하다고 주장하지 않는다. |
| U16-2 `u16_2_current_001` | 953 / 49 | 774 / 0 | 딜러·스트레스의 shared consumer 변경과 교정에 귀속. 기본 활동 owner/오라클 유지. |
| U16-3 `u16_3_current_001` | 814 / 42 | 639 / 0 | 스트레스/공통 XP 및 후속 검증 교정. 미니게임/담보/상품 owner는 불변. |
| U16-4 `u164_current_001` | 920 / 7 | 550 / 0 | 게임 runtime·정책·콘텐츠는 불변. cfg(test)/fixture 6경로와 선택 runtime-measurement 진행기 1경로. 새 finding은 기존 runtime에 존재한다. |

[전체 차이](PREDECESSOR_DIFF.json), [U16-4 이후 실제 patch](POST_U164.patch). U16-4 manifest의 제품 roots에 추가/삭제된 runtime 파일은 없고 별도로 포함한 서버 examples 6개는 감사의 보조 입력이다. 7개 후속 차이는 서버 메시지 fixture 교정 1개와 실패 테스트 교정 6개의 정확한 postimage에 연결한다. handlers/messages의 변경은 test module 안에 있고 Core fault test는 feature gate 아래다. 선택 playback의 Expedition/loan/초반보상 퇴장은 정상 공개 명령을 사용하며 제품 게임 동작이나 한도를 변경하지 않는다.

## F-U16-FINAL-001 — XP 보상 수치에 비례하는 미계량 레벨업 반복

**P2 / 범위 안 — 지원 경로 / 열린 완료 차단.** [employee.rs](../../../../../src/game/employee.rs)의 `Employee::add_experience_with_policy` 705–714행은 각 레벨 비용을 한 번씩 차감한다. 작성된 곡선 끝에서는 마지막 비용을 계속 사용한다. 이 반복에는 작업량 charge/poll이 없고 입력 수치에 비례해 수행한다. `RunPolicyData::validate`는 양수 XP 비용 1을 허용하며, `StressCombatPolicy::validate`는 본 진단의 양수 보상과 2배 수량을 정상 허용한다. `GrantExecutor::grant_experience`가 실제 전투 완료에서 이 함수를 두 생존 참가자에게 호출한다.

기존 정상 스트레스 원정 RON을 복제하고 **XP 곡선 `[1]`과 첫 라운드 XP 수치만** 바꿨다. 정상 `package_official_content` → `content_pack::load_with_runtime_safety` → `CoreApplication` → 같은 5라운드 완료 명령을 사용했다. 내부 상태/save/checksum 조작, 제품 한도 상향, debug grant는 없다. 새 pack의 CMVA 승인 또는 실제 paired 저장을 주장하지 않으며 이 진단은 지원 로더와 공개 Core 연산 경계의 증거다.

| 첫 라운드 XP | 두 생존자의 실제 레벨 증가 합계 | 마지막 tick의 Core 연산 시간 | 계량 native work | 결과 |
| --- | --- | --- | --- | --- |
| 1,000 | 2,048 | 0.110569초 | 877,508 | 정상 정산 |
| 10,000,000 | 20,000,048 | 0.517545초 | 878,235 | 정상 정산 |
| 100,000,000 | 200,000,048 | 4.396964초 | 878,418 | 정상 정산 |

각 변형은 52개 공개 연산을 실제 실행했고, 두 생존자의 최종 레벨은 `1 + 첫 보상 + 나머지 라운드14 + 기본XP10`, 잔여 XP0이다. 미출전 직원은 레벨1/XP0이며 중간 라운드에는 XP를 지급하지 않는다. 인벤토리·phase·거절 결과는 원래 host trace와 대조했고 모든 변형에서 요청 heap이 정확히 회수됐다. [독립 산술/원시 결과](VERIFIED_RESULTS.json), [입력/관측 설계](XP_PROBE_PLAN.json), [실행 003](XP_RUNS003.json), [확장 대조 004](XP_RUNS004.json).

계량 work를 CPU 시간이나 레벨 증가 횟수와 같은 단위로 간주하지 않는다. 문제의 근거는 **현재 코드의 미계량 수치 비례 반복**, 같은 공개 연산에서 관측된 큰 CPU 비용 증가, 고정 10M work profile에서도 해당 반복을 중간 점검하거나 그 수량만큼 사전 제한하는 경로가 없다는 세 가지다. 위 시간은 dev opt0/관측 allocator 환경의 한 번씩 실행한 연산창이며 배포 SLO 실패나 정밀 성능비율이 아니다. 숫자가 u32로 유한하다는 사실만으로 지침 §6의 조정 가능한 비용 예산을 대체하지 않는다. 허용 최댓값의 실행 시간을 외삽해 실측값으로 제시하지 않는다.

귀속: 레벨별 while 구조는 U16-4 이전에도 존재했다. U16-4의 XP 소실 교정은 u64 합계로 산술을 바로잡았으나 기존 비용 구조를 유지했다. 따라서 이번 finding을 그 교정이 새로 만든 XP 산술 회귀라고 부르지 않는다. 현재 스트레스의 큰 authored 보상이 이 공통 경계를 직접 소비하므로 **GOAL-CSCB-01 / WP-CSCB-VERIFY의 A10 비용·한도 완료 주장**을 다시 닫아야 한다. 실제 교정은 공통 성장 owner `employee.rs`이며 U14 및 다른 XP grant 소비자의 의미/회귀를 함께 확인해야 한다.

권장 기술 교정은 작성 곡선의 유한 구간만 순회하고 반복 tail은 나눗셈/나머지로 처리하여 정확한 u64 합계·level saturating 의미를 보존하는 것이다. 다른 방식이면 유한 배치의 실제 비용을 허용 전에 계량/중단할 수 있어야 하며 실패 원자성을 유지해야 한다. 기존 XP 소실 교정과 한도는 되돌리지 않는다. 교정 이후 같은 세 변형·곡선 경계/큰 잔여/level 포화·일반/무억제/실패 XP와 영향 baseline을 검증하고 새 target의 독립 완료 감사를 인수한 뒤 U16 종합 판정을 갱신한다. 이번 감사에서 수정하지 않았다.

## 수용 조건과 공유 경계 검토

66개 개별 수용 조건은 [CONDITION_REVIEW.json](CONDITION_REVIEW.json)에 원문·기존 evidence·현재 판단으로 연결했다. 현재 판단은 기존 표의 `satisfied`를 복사해 끝내지 않고 다음 source/오라클/원시 증거와 대조했다. 65개의 기능·저장 조건에 새 반증은 없으며 U16-4 A10은 위 finding으로 `Not Complete`다. 기존 40차원·20IVA의 기능 증거는 보존하되 그 합계를 전체 완료로 간주하지 않는다.

| 조건 묶음 | 현재 검토와 증거 |
| --- | --- |
| U16-1 A01–05 | 상품/가격/소진·갱신·자산별 판매 제한·HQ 자격/세트는 기존 owner가 동일하다. `EconomyGrantState`와 공개 application candidate의 비용/사본 commit을 확인했다. 구매/부족/중복/65번째 사본/확장 저장/HQ terms를 이번 집중 시험에서 실행했다. |
| U16-1 A06–12 | Event roll/receipt·직원/장비 증거·무료 침식과 소비품 owner는 기존 감사와 동일하다. 원시 host/check selector, 기존 승패/후퇴 기간 차감과 1회 치료 오라클을 재사용하고 즉시치료는 재실행했다. 스트레스가 추가한 combat 완료/후퇴도 소비품 결과 경계와 대조했다. |
| U16-1 A13–18 | 불완전 노드 입장 전 Continue와 완료 후 결과, IO4/재시도, 작성 오류12, 확장Continue/실제heap 원시를 검산했다. 16MiB 유지와 decoder 비용 증거를 재사용한다. 본 finding의 수치 반복은 정상/확장 상품 저장 비용 증거로 면제하지 않는다. |
| U16-2 A01–10 | `map_content→relic_reward`에서 고정 종류/후보, 다음 등급 후보 검증 후 차액 지불, 여섯 합성 확률/입력 정렬/종류 제외, 공통 grant+노드 완료 transaction을 직접 검토했다. 기존 8개 owner 시험을 포함한 집중 실행과 원시 공개host를 대조했다. |
| U16-2 A11–17 | 투자·소비·결과가 같은 미완료/완료 복원 경계를 사용한다. loader/host/IO/비용 원시 검산 통과. 당시 A01의 ‘U16-3/4 미지원’은 그 단계 범위이며 현재는 후행 Goal의 명시 구현이 대체한다. 과거 정책 파일을 수정하지 않는다. |
| U16-3 A01–09 | 현재 `dealer`의 Offer→Gambit→Ready→Playing/Between 흐름·revision·양수 자격·음수 유지와 VN 선택 연결을 읽었다. 세 게임 내부는 원래 감사 source와 동일해 원시 게임 오라클을 재사용하고 현재 세 게임 public route를 집중 실행했다. |
| U16-3 A10–17 | collateral은 사본을 live inventory에서 escrow로 옮겨 잠그고 회수는 원본 UUID/침식을 반환한다. 포기/상점/Leave의 정산 순서를 확인했다. `action_capability`가 딜러 중 일반 Claim/ExitReward 우회를 차단한다. 반복/다른 rarity/부분회수·포기·구매는 집중 검사와 raw host를 대조했다. |
| U16-3 A18–21 | `RunProgression.dealer_introduced`와 조우 owner의 수명을 분리한다. 미완료 Continue는 중간거래 전체를 되돌린다. 21process153명령/IO4와 새 과거감사 replay, 9회 비용/1152창의 원시 증거를 검산해 재사용했다. |
| U16-4 A01–04 | `map_content/expedition_lifecycle→BattleCore`의 실제 Reward 배정·입장 preview, wave gate/현재 round 전멸, 단일 battle 상태, timestamp bucket 뒤 패배우선, 대기 tick 정지를 검토했다. 기존 same-time/cooldown/protected-target probes와 이번 집중 회귀를 연결했다. |
| U16-4 A05–09 | 성공만 누적 grant, 라운드별 미지급, 무억제 수량·XP 1회/유물 유지, 후퇴 entry restore→노드 소비→stable Continue 발행을 확인했다. 17host192연산/IO4 및 fresh/SavePoint raw 검산을 재사용했다. 이번 XP 변형은 산술 보존도 독립 검증한다. |
| U16-4 A10 | 정상/확장 대표 측정은 유효하지만 지원 XP 수치 비례 반복에 대한 F-U16-FINAL-001을 발견했다. 자원 경계 완료는 미충족이다. |

U13 배정/초반 예약과 각 활동의 진입 조건은 기존 fixture와 공개host의 원정 경로에서 확인한다. U04 continuation은 미완료 노드의 입장 전 root를 유지하고, 완료된 interactive/전투 결과/스트레스 후퇴에서 stable root를 새로 발행한다. U14 입장 비용·무억제와 U15 공통 자산은 별도 활동 엔진에 복제하지 않는다. `prepare_command_with_generation→prepare_publication→publish_prepared_application_bound→commit`의 writer/reader 경계를 읽고 기존 실제 IO failure/retry를 재사용했다. 새 감사가 실제host IO를 다시 실행했다고 표기하지 않는다.

새 제품의 시험 전용 분기, consumer의 보상 재계산, 중복 자산 권한, 숨은 fallback/자동 한도상향은 확인하지 못했다. source-level 검토와 원시 증거 검산은 구별했다. 기존 audited minigame 내부를 새 판본의 외부 규칙으로 재해석하지 않았다.

## 실행·재사용·실패 원본

- 새 집중 실행: `cargo test -p game_core --lib --offline --locked -- u16 dealer relic_reward stress_combat`, **45pass/0fail/0ignore**, exit0. 전체 wall121.00초, 시험32.08초. [로그](focused.log), [시간](focused.time). 원래 전체 workspace가 새로 통과했다는 주장이 아니다.
- 새 공개 XP 대조: 세 supported pack, 각52연산/exit0, 정확 XP/heap 회수. 이것은 finding 재현이며 안전성 PASS가 아니다. 동일 binary의 SHA와 독립 assertions는 [VERIFIED_RESULTS](VERIFIED_RESULTS.json)에 있다.
- 읽기 전용 과거 검산: 네 validator를 현재 감사 출력으로 실행하여 archive/delta/selector/host/IO/작성/비용을 검산했다. [HISTORY_RECHECK](HISTORY_RECHECK.json). U16-2의 historical build locator 검사는 당시 archive에 적용하며 현재 source와 동일하다고 오인하지 않도록 보정했다. 기존 감사 파일을 덮어쓰지 않았다.
- 전체 회귀 재사용: 이전 Core 기본2228pass/ignore6, probe11pass, 후속 서버 기본 고유96pass/ignore22, 최종 계측 영향96pass/0fail/1ignore 및 별도 exporter heap1pass. 후속6파일의 정확한 해시와 named raw 성공96개를 다시 대조했다. 단일 최종 전체green/전체feature 재실행으로 표기하지 않는다. 과거29실패의26교정/3제거 및 첫 실패 로그는 그대로 보존한다.
- 진단 준비 실패: XP 첫 시도는 `official/pack.ron`의 디렉터리 한 단계 누락으로 로딩 실패했다. `content_root/official` 배치로 교정했다. 다음 시도는 `/mnt/f`에 unbuffered 상세 trace를 기록하며 60초에 준비 중 종료됐다. 목표 XP 지급 전이며 제품 실패가 아니다. 동일 binary/input을 `/tmp` 출력으로 실행하니 소형은6.53초에 전체 완료했다. 원래 오류·부분 trace·timeout은 남겼다. 이후 측정창은 원래와 같이 Core 연산만 포함하고 디스크 trace 시간을 제외한다.

## 종료·다음 owner

전체 U16은 아직 완료가 아니다. 기존 U16-1/2/3의 유효 기능 증거와 과거 U16-4 감사004를 삭제하거나 실패로 덮어쓰지 않는다. 현재 U16-4의 A10 긍정 완료와 전체 U16의 완료 주장은 이 새 finding이 닫힐 때까지 재사용하지 않는다. 감사로 승인된 범위는 여기서 종료한다. 다음 작업은 공통 XP 비용 구조의 별도 GEX-10 기술 교정·영향 검증, U16-4 재감사와 U16 종합 판정 갱신이며 U17 구현은 시작하지 않았다.

재현은 저장소 루트에서 `python3 .../u16_final_audit_20261015_001/verify_results.py`로 원시 결과/산술을 검산할 수 있다. 새 실행은 `XP_RUNS003.json`/`XP_RUNS004.json`의 argv와 `xp_probe/Cargo.toml`을 사용하고 결과 경로를 새 `/tmp` 파일로 지정한다. `snapshot.py entry`와 과거 validator의 원본 출력 경로는 재실행하여 덮어쓰지 않는다.
