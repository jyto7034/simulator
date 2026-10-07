# Core 우선 로드맵 최신 인수인계

갱신 UTC 2026-10-06T11:38:01.422769+00:00. 현재 U08 / GOAL-CRNV-01 / GEX-10 부분 구현·검증. 사용자 권한 변경 뒤 승인 실행에서 Unix/로컬TCP 소켓 생성·bind가 모두 PASS하여 **승인 실행으로 남은 통합 검증을 재개한다.** 기존 연속 권한을 유지하고 U08 최종화·독립감사와 U09 진입은 미완료다. Core 우선·Unity 후속·개별 최종보스 고유 구현 제외를 유지한다.

[재개 인수026](u08_public_20261006_026/RECORD.ko.md)와 실제 이후 runner receipt를 먼저 확인한다. 기본 sandbox는 workspace-write/network restricted이고 require_escalated 승인 요청이 가능해졌다. 아래025 EPERM은 권한변경 전 실제 실패 이력이며 새 socket probe는 제품/host 테스트를 대신하지 않는다. 재개 전 source438/8734111c와harness122입력 변경0을 확인했다. Sol은 실제 before 결속 뒤 통합005부터 단독 실행하고 두 Astra작성자는HOLD/입력writer0을 유지한다.

직전 기술 인수인계는 [HANDOFF_20261006_002.ko.md](HANDOFF_20261006_002.ko.md)에 원본 보존했다. 그 이전 사용자의 두줄/CRLF 원문은 [HANDOFF_20261006_001.ko.md](HANDOFF_20261006_001.ko.md)에 보존했다.

## 먼저 읽을 자료

1. [현재 장애·교정·root 인수025](u08_public_20261006_025/RECORD.ko.md), 같은 디렉터리 RECORD.json/REFERENCES.json/SOURCE_CURRENT.json/SOURCE_DELTA_FROM_GEX00.json.
2. [PLAN](../../../goals/core_relic_numeric_variants/execution/single_goal/PLAN.md), [EXPERIMENT_NOTES](../../../goals/core_relic_numeric_variants/execution/single_goal/EXPERIMENT_NOTES.md), [EXPERIMENTS](../../../goals/core_relic_numeric_variants/execution/single_goal/EXPERIMENTS.md), [STATE_INVENTORY](../../../goals/core_relic_numeric_variants/execution/single_goal/STATE_INVENTORY.md). 처음의 초기 실행 설명보다025 현재 안내를 우선한다.
3. [현재 검증 도구/정확한 명령 인계003](../../../goals/core_relic_numeric_variants/evidence/verify_harness_handoff003/HANDOFF.json), 같은 디렉터리 MANIFEST.json/SOURCE_REBIND.json/ENVIRONMENT_BLOCK.json, [교정 결과](../../../goals/core_relic_numeric_variants/evidence/test_integrated_corrections001/RESULT.json), [환경 진단](../../../goals/core_relic_numeric_variants/evidence/env_socket_probe001/RESULT.json).
4. [단일 현황](../../content_authoring/ROADMAP_STATUS.md), [Core 로드맵](../ROADMAP.ko.md), [연속 요청](CONTINUOUS_REQUEST_20261004.ko.md). 최신 사용자 요청의 역할·연속 권한을 적용한다.
5. U08 [CODE_REVIEW_001](../../../goals/core_relic_numeric_variants/CODE_REVIEW_001.ko.md), [P2G_60_HANDOFF_001](../../../goals/core_relic_numeric_variants/P2G_60_HANDOFF_001.json)에서 필요한 정책/Goal/phase/수용문장/선행 감사 참조만 읽는다. 적용 지침은 docs/policy_to_goal_pipeline_revised_v2.md, docs/goal_execution_pipeline(1).md, content_authoring/guides/policy_to_goal_pipeline_roadmap_usage_v2_1.ko.md다. 최신 사용자 합의와 game_rulebook.md/game_rules가 게임 정책이다.

## 현재 source·검증·남은 실패

현재 runner 입력은438파일, aggregate `8734111ce349bc36de5534aff831f89ccf8c9034b7e802e120d87cee5d0e248b`다. SOURCE_CURRENT와438원본 source_versions archive(SHA9de94d559b6a92836f8167fe7b197d5890ad228d9018c5c538a37aa288960d6d)가 실제 bytes를 소유한다. GEX00 398파일 대비40추가57수정0삭제의97경로/전후해시는 public025/SOURCE_DELTA_FROM_GEX00.json에 있다. 이는 인수용 목록이며 최종 감사 target이 아니다. 개별 product/test 변경 patch는 각 execution 기록에 보존한다. 마지막 제품 주석 교정은 evidence/product_catalog_doc001/CHANGE.patch, 마지막7시험파일 교정은 evidence/test_integrated_corrections001/final/의7diff다. 기록 patch를 workspace에 재적용하거나 source를 과거 hash로 되돌리지 않는다.

통합004는 source438/ce0e0f2f에서 fmt PASS 뒤 Core default **1,872 PASS/12 FAIL**이었다. evidence/integrated_core_default004.json/raw log와 integrated_matrix004.json, execution_harness_binding004/ABORTED_INTERVAL.json이 원실행을 소유한다. 이 source에 후속 주석1곳/시험7파일 교정을 적용했으므로 과거 성공을 현재 전체 suite PASS로 올리지 않는다. 11개 catalog·capability exact selector는 현재 source에서 각각1/1 PASS다. 첫 fixture namespace compile 실패는 시험0으로 보존했다. 마지막 Sol post_correction_fmt001은 현재 source에서 PASS/변경0이다.

나머지 실패는 UnixListener bind EPERM이다. 10:39:49–53 UTC 환경 진단에서 AF_UNIX socket은 생성됐으나 /tmp bind EPERM(1), AF_INET은 socket 생성부터 EPERM(1)이었다. 당시 sandbox workspace-write/network restricted/approval never에서 필요한 소켓·host·네트워크 검증을 완료할 수 없었다. 현재는026의 승인 실행 경로가 가능하다. 허가 우회/다른 socket 경로 반복/시험 skip으로 해결하지 않는다. 이 환경 장애는 소켓 fixture 변경으로 숨기지 않았다. 전체 통합·G7 release CLI·성능·최종화·독립감사는 미완료다.

G1–G6 국소 증거는 public016–024와 EXPS/NOTES에 보존했다. 마지막 G6 lifetime0021/1(실제3회전투)·carrier_module00112/12는 sourcece0e0f2f의 성공이다. G7은 외부RON 정의·describe/explain·실제피해·trace oracle 작성까지만 준비됐으며 현재 release 실행은 아직 없다. U17 실제 부활촉매와의 교차시험은 미구현 후속 한계로 유지한다.

## 실제 역할·현재 소유

- root: gpt-6-astra/high, 계획·설계·수용 조건·조정·읽기·문서. 제품/시험 코드를 작성하지 않았고 최종 독립 감사자가 아니다.
- /root/u07_implementation: gpt-6.1-sol/xhigh, 제품 구현·교정·통합 포맷/빌드/기능/성능 실행·raw 수집. 이번 마지막 주석 교정/환경 진단/통합004/포맷을 담당했다. 과거 Sol 작성 시험의 원 작성 이력은 보존한다. 현재 계약에서는 시험/fixture/oracle/comparator를 편집하지 않는다.
- /root/u08_test_author: gpt-6-astra/high, 시험·fixture·공개 probe·기대값·작은 집중 실행과 교정. 마지막7파일11selector 교정 담당이며 독립 감사 자격이 없다.
- /root/u08_finalizer_author: gpt-6-astra/high, 보조 검증/성능/finalizer 도구·G7 준비·harness003 작성 참여. 역시 독립 감사 자격이 없다.
- U08 최종 독립 감사는 아직 배정/실행하지 않았다. 과거 /root/u07_independent_audit 이름만으로 현재 가용성/독립성/모델을 추정하지 않는다. 최종화 뒤 현재 실제 가용성과 U08 비참여를 확인한 별도 gpt-6-astra/high에게 전체 Goal을 맡긴다.

025 장애 인계 시 위3작성자는 모두 completed/HOLD, 제품/test/harness writer0·runner0이었다. 현재026 재개에서는 Sol이 preflight 뒤 단독runner를 맡고 두 Astra작성자는HOLD/전체입력writer0을 유지한다. 실제 상태는 public025/QUIESCENCE.json과 각 담당 결과를 대조한다. 프로세스 가시성은 sandbox namespace에 한정되어 호스트 전체 무작업을 보증하지 않는다. 입력 소스는 현재 서로 일치하지만 감사 동결 target이나 실행 interval로 승격하지 않는다. 다음 재개 시 가용성·현재 source·진행 프로세스·미검증 delta부터 다시 확인한다. 같은파일 순차편집과 기능/성능 실행 분리를 유지한다.

## 유지할 코드 계약과 정정된 전제

유물 catalog는 immutable generation 소유, run은 copy UUID/variant/3slot·crafting resources 소유다. 정상 grant/equip/variant/cost commit은 기존 staging/rollback·save/checkpoint/Continue 경계와 함께 작동해야 한다. 최신 typed 계약은 CODE_REVIEW TD01–06와 PLAN에서 확인한다. Event 객체·target 권한·수명은 public002/003의 OwnedHashMap 및 현재층 Event 정정을 따른다. Core StartNewGame은 NotStarted만 허용하며 replace_active_run은 host 확인과 구별한다.

Numeric은 기본 aura 반영뒤 relic once, 원 source_attack·generation/provider/site 의미를 보존한다. literal/Expr/basic과 capture/queue/projectile/area 재결속을 모두 검사하며 일반증폭을 모든 피해 helper에 무차별 적용하지 않는다. 실제 사망 정산은 terminal, CompleteCombatResult는 acknowledgement다. Complete 후 최근 terminalfacts/binding cache가 남고 다음 terminal에서 교체될 수 있다. Weak 검사상 strong0은 T/nested payload drop의 증거이며 backing allocation 전체 해제·heap 상계·누적 admission 환급을 뜻하지 않는다. cap 정확검사는 admission enforcement이며 실제 heap 상계를 대신하지 않는다.

PC 8MiB/workload/31성공5거절·회귀 oracle/hardguard를 유지한다. R05 원 cap9,030,188의 concurrentpeak 단독 기존 경고만 보존하고 다른 실패에 예외를 추가하지 않는다. 한도 상향·workload 축소·기대 완화·계량 누락은 허용하지 않는다. 20분stress 제외와 공유호스트 timing 한계를 그대로 기록한다. 외부 fixture가 성공했다는 이유로 공식 콘텐츠나 U17 교차 동작이 검증됐다고 주장하지 않는다.

## 정확한 다음 행동

026에서 승인 실행 소켓 기능을 확인했다. 기존 EPERM을 sandbox 안에서 반복하지 말고 필요한 실행명령에 require_escalated와 구체적인 justification으로 승인 요청을 보낸다. 아래 순서를 현재성 확인부터 재개하며 before binding에는 기존025rootreview와026환경변경을 함께 참조한다. 기존 연속 권한 범위의 일반 교정에 별도 재승인을 추가하지 않는다.

1. source438와 harness003의122입력·외부archive/필수2member를 실제 bytes로 다시 대조한다. 새변경은 인수하고 과거 source로 되돌리지 않는다. 필요하면 현재 입력으로 새 manifest와 고유 label을 만든다. 아직 사용되지 않은005 label/출력 존재도 확인한다. 003의 root review null은 준비 당시 상태이며 닫힌 root review는 public025/RECORD.json이다.
2. Sol이 첫 필수 실행 전에122files_before(정확한 manifest 순서), 실제UTC/source/전체writer0/rootreview와 external_inputs_before를 남긴다. 003/HANDOFF.json argv 그대로 integrated_matrix.py **005 0**, 이후 verify_remaining.py **0 005**를 run.py 새 receipt로 단독 실행한다. 현재005는 미실행 계획이다. fresh release build/bind가 release/public보다, fresh debug measurement build/bind가 playback26/lifecycle10/host4×2보다 선행한다.
3. 전체49기능묶음을 실제로 닫은 후 각 PC/R05 직전에 source/UTC/functional receipts/writer0/runner0을 별도 기록하고 순차 측정한다. R05 target은 /tmp/u08-r05-target005이며 시작전 존재하지 않아야 한다. public CLI의 디렉터리는 cli_current_005, 내부 이름은 CLI_FINAL_SUMMARY_002.json이다. 실패원본은 보존하고 새label로 교정한다.
4. 마지막 필수 receipt 뒤122files_after·external_inputs_after·실제UTC·각receipt기간을 닫는다. before==after 및 required receipt가 실제 interval 안에 있는지 검증한다. prepare 시각을 실행 시각으로 소급하지 않는다. finalizer_prep002의 CONFIG_TEMPLATE/CLAIM_SCHEMA_EXAMPLES를 실제 closed refs로 채운다. 준비 도구가 있다는 사실은 finalization PASS가 아니다.
5. evidence selection → predecessor impact → adaptation 현재 결속 → coverage → 자체검증 → PLAN/NOTES → exact target 동결/격리 정·역복원 → 비참여 Astra의 **전체 U08** 독립감사를 수행한다. 32수용문장/16군·10차원·191선행+31R06·3node/16seam를 연결한다. finding은 담당자 교정·새증거/target·재감사로 닫는다. Complete/High/열린finding0과 exit currentness 뒤 U09로 진행한다.
