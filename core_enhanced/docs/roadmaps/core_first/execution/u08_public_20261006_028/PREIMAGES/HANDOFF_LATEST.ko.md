# Core 우선 로드맵 최신 인수인계

갱신 UTC 2026-10-06T12:36:57.751502+00:00. U08 / GOAL-CRNV-01 / GEX-10 검증을 **계속 실행**한다. 기존 연속 권한은 유효하며 U08 전체 완료·독립 감사·U09 진입은 아직 아니다. Core 우선·Unity 후속·개별 최종보스 고유 구현 제외를 유지한다. 이전 최신본은 [HANDOFF_20261006_004.ko.md](HANDOFF_20261006_004.ko.md)에 exact 보존했다.

현재 source438/`380e7408c647444127f818afb1fc722a6d3602b660ecc57b38d9df74ddd89171`. [root 인수027](u08_public_20261006_027/RECORD.ko.md)와 실제 이후 receipt를 우선한다. 현재 환경은 danger-full-access/network enabled/approval never다. **sandbox_permissions 인자를 전달하지 않는다.** 과거 EPERM 환경장애와 require_escalated 절차는 이전 환경의 이력이며 현재 실행 지시가 아니다.

## 최소 읽기 순서

1. [인수027](u08_public_20261006_027/RECORD.json), 같은 디렉터리 SOURCE_CURRENT.json/SOURCE_DELTA_FROM003.json/FOCUSED_READBACK.json/HARNESS_READBACK.json.
2. [PLAN](../../../goals/core_relic_numeric_variants/execution/single_goal/PLAN.md), [NOTES](../../../goals/core_relic_numeric_variants/execution/single_goal/EXPERIMENT_NOTES.md), [EXPERIMENTS](../../../goals/core_relic_numeric_variants/execution/single_goal/EXPERIMENTS.md), [STATE](../../../goals/core_relic_numeric_variants/execution/single_goal/STATE_INVENTORY.md). 초기 문단보다027과 이후 실제 receipt를 우선한다.
3. [harness004](../../../goals/core_relic_numeric_variants/evidence/verify_harness_handoff004/HANDOFF.json), 같은 디렉터리 MANIFEST/EXECUTION_PLAN/SELF_CHECKS. 최종화 도구는 별도 evidence/finalizer_prep002이며 아직 실행 전이다.
4. [단일 현황](../../content_authoring/ROADMAP_STATUS.md), [Core 로드맵](../ROADMAP.ko.md), [연속 요청](CONTINUOUS_REQUEST_20261004.ko.md), 현재 정책·CODE_REVIEW_001·P2G_60_HANDOFF_001의 필요한 참조. 적용 지침은 docs/policy_to_goal_pipeline_revised_v2.md, docs/goal_execution_pipeline(1).md, content_authoring/guides/policy_to_goal_pipeline_roadmap_usage_v2_1.ko.md. 최신 사용자 합의와 game_rulebook.md/game_rules를 따른다.

## 구현·검증 경계

U08 현재 검증 인수027: 공개 RON Normal 실제 HP 누락과 nonfinite 입력 누락을 보강했다. source438/380e7408에서 scoped fmt와 Normal/E009/nonfinite exact3시험이 PASS이며 Normal452→272=180, Corrupt422→212=210, 비정상 입력16건을 실제 확인했다. 제품 변경 없이 u08_relics.rs에 이전14줄/이번200줄을 추가했고 기존 E00936 입력은 그대로다. 현재 전체 기능 PASS나 독립 완료는 아니다.

harness004의1322입력(기존122+실제 읽히는 추가1200), archive78+44+1200, 외부archive2개/필수8member, 현재source438·정확archive를 root가 검토했다. 49기능+8성능의57필수 역할을 새006으로 실행한다. source/input writer0이며 Sol /root/u08_product_runner_resume가 실제 before 결속 후 단독runner를 맡는다. 두 Astra 작성자의 후속 문서 작업은 실행 입력 밖에서만 허용한다. 현재 fullaccess/network enabled/approval never 지시에 따라 sandbox_permissions 인자는 사용하지 않는다. 과거 EPERM·승인 요청 방식과005 부분 실행은 원본 이력으로 보존한다.

정확 실행 인계는 evidence/verify_harness_handoff004/HANDOFF.json, root 인수는 core_first/execution/u08_public_20261006_027/RECORD.json, 역할은 u08_roles_20261006_004/RECORD.json이다. 소스·도구 before/after 실제UTC, 모든 비성능49receipt를 포함한 PC/R05 각각의 실제 quiescence, fresh release/debug 선행 빌드와 binary 결속을 유지한다. 최종 순서는 selection→191+31 impact→adaptation→32문장/10차원/3node/16seam coverage→자체검증→PLAN/NOTES→동결/격리복원→전체U08 비참여 Astra 감사다. U17 실제 부활촉매 교차는 미구현 후속 경계이며 16.2 독립 감사는 대기다.

source003의873 대비 단일 u08_relics.rs에 nonfinite14줄과 Normal200줄이 추가됐다. 새Normal test는 full authoring loader·정상reward/equip·Event decline·실제battle 명령을 거쳐 HP452→272를 확인한다. E009는 새 owned export에서35payload+manifest36 및 fresh process/HP422→212를 다시 확인했고 원본 export를 덮어쓰지 않았다. nonfinite16행은 같은최종소스에서 다시 통과했다. 세 집중시험은 각1/1/1884filtered이고 원시 receipt는 evidence/relic_public_normal001, relic_public_normal_e009001, relic_public_normal_nonfinite001이다. RESULT는 evidence/test_public_normal001/RESULT.json에 있다. 전체 변경97경로의 기존 근거는 public025/SOURCE_DELTA_FROM_GEX00.json, 이후단일파일 delta는027을 연결한다. 기록patch를 다시 적용하거나 과거hash로 되돌리지 않는다.

통합005는 옛873에서 fmt/Core1957 PASS 뒤 추가검증 보강을 위해 중단했다. evidence/execution_harness_binding005/ABORTED_INTERVAL.json과 원raw/receipt를 유지하며47기능은 미시작이었다. 새006은 별도 실제interval이다. 초기nonfinite001은 사용자중단으로receipt가 없으며 소급생성하지 않는다. nonfinite002의9af 성공도 별도 과거source다.

## 역할·입력 소유

root gpt-6-astra/high는 계획·설계·읽기·문서·조정만 맡고 제품/시험작성·최종독립감사를 겸하지 않는다. /root/u08_product_runner_resume gpt-6.1-sol/xhigh는 제품교정·통합fmt/build/기능/성능 및 raw 수집을 맡으며 시험·fixture·oracle·comparator를 변경하지 않는다. /root/u08_test_author_resume gpt-6-astra/high는 현재Normal시험과 coverage003 담당, /root/u08_harness_author_resume gpt-6-astra/high는 harness004와 predecessor/finalization 준비 담당이다. 세실제역할은 [roles004](u08_roles_20261006_004/RECORD.json)에 기록했다. 과거Sol/Astra 원작성 이력을 바꾸지 않는다. 두Astra는 U08독립감사자격이 없고 최종별도감사자는 아직 미배정이다. 옛에이전트이름만으로 가용성/독립성을 추정하지 않는다.

027 인수 시 모든 source/harness input writer0, functional runner0다. 이후Sol이 단독runner를 인수한다. 두Astra는1322입력밖 새문서에서만 병행한다. 실제실행프로세스는 각receipt와runner보고를 확인하며 /proc 가시성만으로 host전체정지를 주장하지 않는다. 기능실행과PC/R05측정은 겹치지 않는다.

## 정확한 다음 행동

Sol이004와닫힌027을읽고 source438/1322files/pathorder·archive78+44+1200·external2/8·실제UTC·writer0를 before기록한뒤004의 exactargv로 integrated_matrix006→verify_remaining006을순차실행한다. 인수당시167미래경로가비어있었다. freshrelease/debug build+binarybinding이소비자보다선행한다. PC/R05는49기능role전체receipt가닫힌뒤각각source/actualUTC/writers0/functionalrunners0/49exactrefsquiescence를남겨순차실행한다. /tmp/u08-r05-target006은시작전없어야하며CLI의내부이름은CLI_FINAL_SUMMARY_002.json이다. 실패가있으면원본보존·유형분류·담당자교정·새label로진행한다.

최종57필수역할뒤실제after1322+external2/8+source와receipt기간을닫는다. 준비시각을실행시각으로대체하지않는다. 현재coverage003의32문장/10차원/3node/16seam 및predecessor초안191+31을실제결과로결속해 selection→impact→adaptation→coverage→자체검증→PLAN/NOTES→DIRECT UNWRAPPED freeze/격리정역복원→U08비참여AstraHigh전체감사를진행한다. Complete/High/열린finding0+exitcurrentness뒤U09로계속한다.

유물catalog immutablegeneration/runcopyUUID·variant·3slots·resource소유, 정상grant/equip/cost/save/checkpoint/Continue원자성, aura뒤reliconce·원source/provider/generation/capture수명을유지한다. 실제사망정산은terminal,CompleteCombatResult는acknowledgement다. Weak strong0은payload drop이며 backingstorage전체해제/heap상계/admission환급이아니다. PC8MiB·31성공5거절·workload/guard, R05원cap9030188의기존단독peak경고만유지한다.20분stress제외·공유hosttiming한계도보존한다.U17실제부활촉매교차는후속미구현경계다.
