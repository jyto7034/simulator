# R05 실행 기록

## 2026-09-12 · GEX-00 초기 인수

새 exact Goal/Topic에서 prior execution 역할과 경로가 없음을 확인해 `execution/single_goal` 한 경로를 정했다. [P2G-60](../../P2G_60_RESULT_001.json) current exact chain/Not Activated 상태를 인수하며, 실행 권한은 기존 active user request에서 별도로 확인했다.

기준선333파일/8,551,924bytes와 original dirty status/diff/archive를 evidence에 보존했다. GEX-00에서 source mutation, Cargo build/test, gameplay probe는 수행하지 않았다. rustc/cargo1.91.1 및 local source/data/tool 접근과 공간을 확인했다.

기존 R04 verifier는 read-only prefix만 재사용해 currentness를 재측정한다. 원래 script/report/target/registry/outputs를 수정하거나 기존 reviewer identity의 새 audit를 발행하지 않는다. 완료한 출력과 실제 unexpected drift 판정을 인수 result에 추가한다.

## 2026-09-12 · GEX-00 완료

currentness reader session11382 exit0:6,946distinct files,20raw mismatches,unexpected0. R04 target003와3선행 target/외부 inventories/바이너리/47선행 출력 및18 R04 봉인 출력의 actual을 확인했다. 기존5경로 중4 actual hash는 P2G 인수와 같고 OPERATING_RULES의 이번 single-R 인수 단락을 제거하면 과거 exact hash가 복원된다. [결과](GEX_00_RESULT_001.json)가 정상 GEX-10 신규 인수를 기록한다. 새 audit/game test/source mutation은0이다.

## 2026-09-12 · GEX-10 상세 설계 시작

Active fixed-R05 goal-run의 별도 구현 pass. accepted upstream/dirty baseline을 유지한다. SC01–03의 실제 자료형·수명/causal/예약·Run overlay/result/save·catalog/consumer와 유한 비용을 구체화한 뒤 production을 수정한다. 아직 patch/게임 시험 없음.

## 2026-09-12 · 선언 구현 / 시험001

`game::resource`에 strict typed 선언·selector 검증·native CloneShape를 추가했다. 새 모듈은 crate 내부이며 loader/executor capability는 아직 게시하지 않았다. `rustfmt --edition 2021 core_enhanced/src/game/resource/mod.rs` exit0. [실제 시험001](evidence/resource_definitions_001.json): `cargo test --locked --offline -p game_core --lib game::resource::` exit0, 3 passed / 0 failed / 1311 filtered. local Cargo target와 scoped evidence log에만 출력했고 외부 다운로드/개인 store 접근은 없다. 내부 선언 seam만 검증했으며 여섯 수명의 실제 소비·Run AT05·public 인수는 미완료다.

## 2026-09-12 · store/registry 구현 / 시험002–003

실제 owner token, 여섯 scope registry, finite Chain hold, sparse value/revision, reserve/commit/abort 및 grant/decay/consume, typed 내부 fault와 보수적인 retained/clone 비용을 추가했다. 시험002는 내부 `Result` alias가 기존 CloneShape macro 반환형을 가린 compile error와 test generic 추론 실패로 exit101(0실행)했다. alias를 ResourceResult로 바꾸고 test 반환형을 명시했다. 별도로 native-only operation도 clone 전에 직접 meter하도록 보완했다. 같은 focused command 시험003 exit0,18passed. 두 command의 immutable input snapshot을 미리 잡지 않았으므로 diagnostic history로만 보존하고 final exact-source 증거로 사용하지 않는다. 후속 retained walk의 선행 charge 수정으로 영향 seam을 재검증할 예정이다.

## 2026-09-12 · Run projection / 시험004 실패

RunResourceState의 strict durable key/owner/value projection, overlay capture, full basis/revision 검사, 모든 result의 revision 증가 및 owner 삭제/terminal closure를 내부 코드로 추가했다. 아직 실제 RunState/save writer/reader/result에는 연결하지 않았다. 시험004 exact input manifest/archive를 실행 전에 고정했다. 결과 exit101,22passed/1failed: native clone limit1이 copy를 막지 못했다. 원인은 기본 CloneMeter가 native allowance를 켜지 않는 데 있다. for_native_scan과 별도 scan/charge_clone을 사용하도록 교정하며 시험 통과로 표기하지 않는다.

## 2026-09-12 · native clone 교정 / 시험005 통과

[시험005](evidence/resource_store_005.json): exact input 전체 목록/변경 source archive를 미리 보존하고 같은 focused command를 실행했다. exit0,23passed/0failed/1311filtered, 실행 중 source change0. 네이티브 전용 CloneMeter::for_native_scan으로 clone 전에 allowance를 확인하고 scan work와 clone slots를 별도 charge했다. 기존 실패004를 유지한다. 이는 선언·store/registry·durable projection 내부 증거이며 실제 battle/Run AT05/catalog/public/전체 비용·감사는 남아 있다.

## 2026-09-12 · 실제 resource catalog 연결 / 시험006–008

Resources/Resource를 strict catalog parser, resolver/assembly, final validation, GameDataBase/fixture identity, public definition explain/describe 및 official 빈 RON에 연결했다. catalog schema는3으로 올려 기존2를 명시 거부하고 bundle producer를 함께 갱신했다. 아직 resource effect opcode/executor capability는 추가하지 않았다.

시험006은 public definition exhaustive match 누락으로 compile exit101/0tests. 해당 borrowed projection을 연결했다. 시험007은48passed/7failed: source inventory의 Resources 순서 불일치,25→26 file-count 기대 갱신 누락, 기존 Unix socket fixture의 sandbox EPERM을 확인했다. 목록 순서·직접 영향 count를 수정했다. socket은 임시 fixture 안에서만 생성하며 새 external network는 사용하지 않는다. sandbox 밖 재실행의 도구 승인을 받아 [시험008](evidence/resource_catalog_008.json)을 실행했고 exit0/55passed, source change0. 기존7실패 원본을 보존했다.

세 개 새 외부 시험은 실제 선택 pack의 initial/cap 변경→semantic identity·옛 generation 유지, strict/cap/owner/lifetime negative의 원본 source 위치, public authoring validate/explain/describe와 구 catalog contract 거부를 검증했다. 실제 RON→자원 지출/Run AT05 전체 인수는 아직 남아 있다.

## 2026-09-12 · typed runtime fault·비용 보강 / 시험009–010

새 fault를 기존 RuntimeFaultCode::Resource와 structured error port에 결속했다. 내부 오류를 GameError로 명시 변환하면 native operation에 latch하므로 void cleanup이 반환값을 무시해도 enclosing transaction이 거부된다. shared authoring/native work charge 및 BTree node의 spare slots·빈 root까지 포함하는 retained byte 상한을 추가했다.

시험009는 전역 From<ResourceError> 구현이 기존 boss/summon closure의 오류 타입 추론을 모호하게 만들어 compile exit101/0tests였다. 기존 boss/summon 코드를 변경하지 않고 자원 경계의 명시 into_game_error로 바꿨다. [시험010](evidence/resource_store_010.json) exit0,26passed/source change0: 무시된 오류 latch, authoring/native budget, retained byte admission까지 직접 확인했다.

시험005–011의 command recorder는 [보존 도구](evidence/RUN_TEST_TOOL_001.py)와 같은 bytes(SHA256 bc0cbc65b4dd63b68c8954ca627c2e8bad8724e58a261332087f1e09ad7ad38d)다. source manifest/기준선 대비 변경 archive/실제 stdout·stderr/exit·전후 currentness를 기록하는 helper이며 콘텐츠를 실행하는 별도 runner가 아니다. 실제 검증 대상은 기록된 Cargo command와 production loader/API다.

## 2026-09-12 · live RON 회귀 / runtime adapter 착수

[시험011](evidence/resource_live_ron_011.json)은 live RON owner의81tests exit0/source change0다. 현재 catalog schema3·25catalog source·strict public projection을 인수했다.

ResourceContext/ResourceDelivery와 RuntimeResources를 실제 R04 action accept/terminal/settled, delivery admission/close, startup/transaction, deployment spawn/withdraw/death에 연결했다. resource가 없는 official catalog는 registry를 생성하지 않는다. source observation은 resource capability를 제거하고 실질 source replacement는 parent Chain context를 보존한다. 순수 Source completion의 action 없는 finite hold도 별도로 닫는다. 아직 resource effect opcode와 World Run root/save/owner binding을 연결하지 않았고 실제 여섯 수명 public 시험이 없다.

all-target compile012는 exhaustive source literal2곳의 새 field 누락으로 exit101/0tests였다. explicit source propagation과 source-free static을 수정한 [compile013](evidence/resource_runtime_compile_013.json)은 exit0/source change0다. compile은 게임 시험 PASS가 아니다.

## 2026-09-12 · Run AT05 및 현재 저장 계약

RunState/체크포인트에 strict typed_resources를 함께 추가하고 실제 inventory·roster owner graph를 battle startup reset 뒤·시간0 처리 전에 mount했다. terminal overlay는 battle core 제거 전에 result에 capture하고 기존 결과 candidate에서 full basis/revision을 검사해 반영한다. 공통 application transaction은 진입 시 값을 검증하고 성공한 owner 변경·terminal closure를 같은 candidate에서 reconcile한다. latest-only 저장 형식 V7로 writer/reader/checksum/validator를 함께 전환했다. [compile014](evidence/resource_at05_compile_014.json)은 exit0/source change0다.

체크포인트 시험015는16passed/1failed였다. 이전 unknown-version 음성 시험의7이 현재 유효 V7이 되어 거부되지 않았으며, 해당 입력만8로 교정한 [시험016](evidence/resource_checkpoint_016.json)은17passed/0failed/source change0다. 구4/5/6 거부와 현재 키 목록, 독립 역사 corpus projection을 보존했다. 아직 nonempty Run 자원의 실제 결과·후퇴·owner 전환 증거를 대신하지 않는다.

## 2026-09-12 · RON Resources 실행 및 공개 borrowed 관찰

SkillEffectDef::Resources의 explicit selector/Grant·Decay·Consume 묶음을 actual owner와 Run/Attempt/Deployment/Activation/Delivery/Chain capability에 연결했다. 전개 수4096을 넘기기 전에 거부하고 authored 순서의 store batch를 단일 candidate에 적용한다. Consume은 내부 synchronous reserve/commit을 이용하며 임의 지연 예약 grammar는 추가하지 않았다. 실제 common Cost/ResourceChange에는 typed ID·owner·lifetime·diagnostic scope·before/after·출처를 기록한다. [compile017](evidence/resource_executor_check_017.json)은 exit0/source change0다.

borrowed ResourceView는 catalog·actual owner·활성 scope·sparse entry·reservation count를 노출하고 mutation capability는 노출하지 않는다. CoreApplication과 authoring observation에 Run projection/active overlay를 연결했다. current execution contract2와 typed_resources_v1 descriptor를 실제 opcode에 맞춰 추가했다. [compile018](evidence/resource_public_check_018.json)은 exit0/source change0다. 이는 전체 R05 지원 완료 판정이 아니며 AG 구현·실제 public 시험·감사가 계속 진행 중이다.

실제 external pack에서 직원·장비 자원과 OnAttack skill binding을 읽어 World 전투로 소비하는 시험019는 테스트 RON API 경로/ProviderFactSummary field 오용으로 compile exit101/0tests였다. production을 바꾸지 않고 fixture/관찰 assertion을 수정해020을 실행 중이다. immutable 실패 원본을 유지한다.

## 2026-09-12 · 실제 여섯 수명 소비 시험020–027

020/021은 fixture raw RON 필수 delay_ms/name/focus_time_ms 누락으로 실행2tests 실패했다. 필수 값을 명시했다. 022/023은 실제 근접 사거리 밖 배치로 authored OnAttack 소비자에 도달하지 못했고,024는 점유된 route endpoint 배치로 실패했다. 외부 시험 장비에 명시적 넓은 사거리를 주고 실제 미점유 칸을 선택했다.025는 전술 배치 비용10을 typed 비용으로 잘못 센 assertion을 교정했다. 공식 RON의 사거리·경제·자원 선언은 바꾸지 않았다.

026은 shortage whole-timestamp rollback 시험1passed, 여섯 수명 시험1failed였다. 직접 소비를 통해 기존 batch spawn 경로에 Deployment hook이 빠져 있음을 확인했다. wave member와 별도로 실제 ordinary/deploy batch에 scope를 열고 즉시 완료되는 step에도 synchronous Delivery scope/Chain hold를 연결했다. projectile/area의 released delivery는 기존 별도 completion을 유지한다. borrowed read는 missing sparse entry의 explicit initial을 materialization 없이 반환한다.

[시험027](evidence/resource_external_world_027.json)은 exit0/2passed/source change0다. 한 시험이 실제 외부 pack을 각 Run/Attempt/Deployment/Activation/Delivery/Chain으로6회 작성·로드·World 시작·실제 장비 OnAttack으로 소비했다. 직원/사본 owner, cap grant15→decay12→consume10, 출처, reservation0, 자기 수명 종료 후 entry/scope 소멸 및 Run overlay 미선반영을 확인했다. 다른 시험은 두 번째 비용99의 부족을 실제 timestamp에서 발생시키고 whole application/battle/RNG/UUID/revision/queue/관측 및 resource view의 정확한 전후 동등성을 확인했다. 이 증거는 실제 성공/즉시 종료/부족 경계이며 released projectile/area 취소·재배치·Run 결과/save/public default-release·전체 비용·fresh 감사는 여전히 미완료다.

028은 현재 source에서 전체 world owner 회귀를 실행 중이다. 결과가 나오기 전 PASS로 세지 않는다.

[World 회귀028](evidence/resource_world_regression_028.json)은 exit0,325passed/0failed/source change0로 완료됐다. 공식 빈 catalog의 world·economy·owner·기존 result/entry/restore 회귀와 새 두 resource external 시험을 포함한다. 새 nonempty resource의 나머지 인수 조건이나 전체 core/default-release/fresh 감사를 대신하지 않는다.

## 2026-09-12 · 실제 Run 경계029–031

Consume-only 및 명시적 timed Victory 외부 fixture를 추가했다.029는3passed/2failed: time-Victory의 required wave 불일치가 정상 strict validator에 거부됐고 재배치 cooldown이 정상 거부됐다. fixture wave를 해당 terminal 계약에 맞추고 cooldown 동안 실패한 재배치의 자원 무변경을 확인한 뒤 실제 시간 진행으로 해제했다.030은4passed/1failed였으며 마지막 실패는 NPC를 포함한 첫 Deployment ordinal을 직원의 실제 scope로 잘못 선택한 assertion이다. 소모 entry에 기록된 actual employee scope를 기준으로 교정했다.

[시험031](evidence/resource_run_boundaries_031.json)은5passed/0failed/source change0다. 실제 철수/재배치에서 Run·Attempt는7→5→3, Deployment는 새 발생의7→5로 구분되며 종료된 직원 scope 조회는 실패한다. 실제 후퇴 후 Run 원본과 새 attempt의 빈 overlay를 확인했다. 실제 terminal capture 후 잘못된 보상으로 CompleteCombatResult를 두 번 실패시키면 Run/기존 grant roots가 복원되고, 교정 재시도는 값5/revision+1을 한 번 반영하며 중복 접수는 거부된다. 실제 SavePoint에서 V7을 export하여 새 CoreApplication으로 continue하면 비어 있지 않은 owner/값/순서/revision이 같다.

032는 그 실제 committed Run 값을 가진 사본의 해제·직원 이전·강화·판매·분해 및 실패 보존을 추가 검증 중이다. 아직 PASS로 세지 않는다.

032는 test metadata field 명칭 오류로 compile exit101/0tests였다. 실제 OwnedEquipment.meta를 사용하도록 교정했다.033은 가격0인 공식 훈련검의 정상 InvalidQuote 거부로1failed였고 timed owner fixture에만 price100을 명시했다. armor의 기존 유지보수 recipe 형태를 시험용 훈련검에도 선언했다. 공식 가격/recipe는 변경하지 않았다. [시험034](evidence/resource_owner_boundaries_034.json)1passed는 실제 결과에서 지출한 사본의 해제/직원 이전/강화 후 값 보존, 거부된 판매/강화의 원본 보존, 판매의 기존35% quote만 지급 및 분해에서 copy key만 삭제/employee key5 보존을 확인했다.

[시험035](evidence/resource_world_lifetimes_035.json)은 현재 fixture6tests 모두 passed/source change0다. 잘못된 owner·resource ID·cap·duplicate key·구 execution contract의 checksum-correct 저장을 fresh CoreApplication이 거부하고 source/목적 Run을 게시하지 않으며, 뒤이은 정상 continue는 같은 nonempty root를 복원하는 음성 시험도 포함한다. 새 resource 파일들만 rustfmt했고 기존 대규모 dirty를 재포맷하지 않았다.

resource reference validation의 새 catalog lookup/skill/step/effect walk를 authoring/native 공통 fuel에 charge하도록 보강했다.036은 GameError에 Display가 없는 diagnostic adapter compile 오류로 exit101/0tests였다. 기존 code() port로 교정하여037 내부 resource 회귀를 실행 중이다. Reference lookup의 실제 structured budget 거부와 released carrier/Chain·추가 owner/acquisition/Run 시나리오·default-release·전체 cost/regression/audit는 아직 남아 있다.

[내부037](evidence/resource_store_regression_037.json)은26passed/0failed/source change0다. [외부 World038](evidence/resource_owner_sources_038.json)은8passed/0failed/source change0로 source/selector 음성과 같은 정의 두 실제 사본·TargetEmployee 분리를 추가했다. 미지원 resource reference와 owner selector가 official/data/skills/base.ron의 정확한 object/field 위치를 유지하며 파일을 변경하지 않음을 확인했다. 두 직원/두 사본의4개 Run entry가 각각5이고 definition UUID는 같아도 persistent copy UUID/관측 source가 다르다.

039는 fixed area의 실제 multi-target Delivery/Chain이 controller 철수 이후에도 유지되고 자신의 종료에서 사라지는지, Activation을 같은 방식으로 사용하면 철수 이후 stale로 거부하면서 이미 성공한 철수를 보존하는지를 실행 중이다. 아직 PASS로 세지 않는다.

## 2026-09-12 · Released carrier039–044와 causal Chain045

039는 area Caster anchor와 Anchor tile origin을 섞은 fixture의 strict validation으로2failed였다. Caster origin으로 교정한040은 Activation stale/철수 보존1passed, Delivery 지속1failed였다. 실제 tick마다 별도 Delivery를 만들고 종료하여 영역 잔량이 첫 틱 뒤 사라지는 구현 결함을 확인했다. 유한 carrier의 queued tick/impact가 carrier의 Delivery scope를 유지하는 명시적 continuation hold를 도입했다. root 종료는 이미 승인한 completion hold를 기다리고 마지막 completion이 값을 닫는다. Chain의 별도 hold와 Activation 종료 규칙은 유지한다. [041](evidence/resource_released_area_041.json)은2passed/source change0다.

042는 projectile fixture에 없는 UnitLocked enum을 사용하여2passed/1failed,043은 TargetLocked에 directional 전용 종료 필드를 지정하여2passed/1failed였다. 기존 strict 계약에 맞춰 TargetLocked와 speed만 명시했다. [044](evidence/resource_released_carriers_044.json)은3passed/0failed/source change0다. 실제 fixed area Delivery/Chain은 controller 철수 후 추가 소비하고 영역 종료에서 소멸한다. 닫힌 Activation을 참조한 후속 tick은 resource_stale_scope로 timestamp 전체를 복원하며 성공한 철수는 보존한다. 실제 projectile은 발사 후 철수한 뒤 충돌 시7→6을 소비하고 motion root/queued impact 모두 완료되면 scope/entry를 제거하며 취소된 미출시 제어 step은 실행하지 않는다.

045는 실제 authored ExtraAttack→OnAttack child cast가 부모 Chain의5에서3으로 소비하고 전체 인과 작업 종료에 정리되는지를 실행 중이다. 결과 전 PASS로 세지 않는다. AUTORUN §8의 읽기 위임에 따라 별도 agent가 Goal 수용 조건과 현재 시험의 누락만 읽기 대조했다. 구현 단계 보조이며 fresh GEX20 감사로 세지 않는다. 그 결과 default-release typed 소비자, nonempty event/defeat/floor/new Run/추가 acquisition 및 기존 destination 복원 음성, 비용/회귀/문서/독립 감사가 여전히 필수 미완료임을 재확인했다.

045–047은 각1failed/source change0다. 046에서 max trigger 횟수를 typed fixture로 명시했고047에서 actual ExtraAttack→OnAttack→child AbilityCast 발생을 확인했다. child는 부모의5가 아니라 별도 Chain 초기7에서5를 소비했다. actual damage의 unsourced activation command batch가 RuntimeFactSource를 교체하며 private resource context를 지우는 결함이었다. 동기 process_command_batches에서 명시적 context가 없는 batch만 현재 causal context를 승계하도록 교정했다. 기존 관측 attribution과 별도 Action/Delivery 정체성은 유지하며 실제 finite parent hold가 살아 있는 동안 child를 admit한다.048은 교정 재검증 중이다.

[048](evidence/resource_causal_chain_048.json)은1passed/source change0다. 실제 external authored ExtraAttack→OnAttack→child cast가 같은 Chain 잔량5→3을 사용하고 유한 인과 작업 종료 후 entry/reservation이 남지 않는다.049는 실제 첫 결과의 nonempty5에서 두 번째 전투 지출3·반복 후퇴5/정확한 Run snapshot·UUID 재현·두 번째 결과3·floor 보존·Abandon/NewRun 초기화를 실행 중이다. 아직 PASS로 세지 않는다.

049는 신규 Run의 pre-run briefing 직후 아직 Run이 없는 정상 상태를 fixture가 unwrap하여1failed였다. 실제 VN 종료/직원 선택까지 완료하도록 교정한 [050](evidence/resource_nonempty_run_050.json)은1passed/source change0다. 첫 결과5→다음 전투3→반복 후퇴5 및 exact Run snapshot/재진입 UUID 일치→두 번째 실제 결과3/revision+1→actual Gate floor 보존→Abandon/새 Run 직원 선택 후 빈 open root를 확인했다.051은 외부 event StartCombat과 synthetic timed Defeat outcome을 실제 실행하여 result의 once commit·retry basis·retreat 보존을 검증 중이다.

051은 event employee check의 실제 실패 분기에 도달하여1failed/source change0다. 기존 event-combat fixture에서 성공 경로를 검증하는 seed123을 명시하고도 actual ResolveEventChoiceCheck를 생략하지 않는052를 실행 중이다. 공식 DC/확률·실패 의미는 변경하지 않았다.

[052](evidence/resource_event_defeat_052.json)은1passed/source change0다. 실제 StartCombat 이벤트와 authored timed Defeat에서 기존 Run5를 overlay3으로 소비하고 실제 terminal result가 지연 반영되며 Complete가3/revision+1을 한 번만 commit한다. 중복 Complete는 거부하고 event retry는3→1, retreat는 확정3을 보존한다.053은 현재 core all-target 회귀를 실행 중이다. 새 resource 파일/시험만 rustfmt했으며 기존 dirty의 전체 재포맷은 하지 않았다. Current fixture manifest producer 두 곳을 catalog3/execution2로 맞췄고 current V7 테스트명/주석을 정렬했다. 아직 default-release 실행 증거는 아니다.

053은 lib1351passed/3failed/source change0이며 all-target 전체 완료가 아니다. 실패는 sandbox Unix socket EPERM, Describe의 과거24/42 count, 공식 빈 Resource catalog를 모든 definition kind 인수로 잘못 기대한 Explain이었다. Describe는25/43을 명시하고 Explain은 외부 synthetic resource를 실제 게시해 모든43종을 조회하도록 교정했다. 읽기 보조의 비용 검토에서 catalog duplicate scan, Run result basis equality/basis retained walk, 새 after projection의 clone admission 누락을 확인했다. catalog charge와 비교/extent charge, destination vector/ID를 할당 전에 native/authoring CloneMeter로 인수하는 borrowed projection shape를 추가했다. empty before basis에서 새 nonempty after projection이 낮은 clone cap에 거부되고 원본이 보존되는 focused 시험을 포함했다.

054는 sandbox의 Unix socket 제한 때문에 승인된 좁은 escalated 동일 all-target 명령으로 재실행 중이다. 결과 전 PASS로 세지 않는다. 이 구현 단계 읽기 보조를 GEX20 감사로 세지 않으며 독립 최종 reviewer는 구현에 참여하지 않은 새 agent가 필요하다.

[054](evidence/resource_core_alltargets_054.json)은exit0/source change0로 완료됐다. lib1355와 integration4+4+25+15+4의 실제1407tests passed/0failed/0ignored다. 세 binary harness의0tests는 실행 인수로 세지 않는다. 새 catalog work/Run after projection clone cap 거부와 원본 보존을 포함하며 snapshot socket도 승인된 환경에서 통과했다. unused trait import 하나만 제거한 뒤055는 실제 admin grant·직원 condition 유지·HQ recruit로 새 owner를 얻고 외부 authored resource를 소비하는 추가 시험을 실행 중이다. 아직 전체 R05 완료나 default-release/fresh audit 증거는 아니다.

[055](evidence/resource_acquisition_status_055.json)은1passed/source change0다. 실제 관리자 직원 condition 변경(중상/사망/회복)에서 같은 employee UUID 자원을 유지하고, 실패한 grant의 다섯 기존 root/typed root를 보존한다. 새 admin 지급 사본과 실제 HQ 채용 직원은 소비 전 sparse entry를 만들지 않고, 장착/배치/OnAttack 후 각 초기7→5를 별도 key로 기록하며 이전 owner 값은 유지한다.

056은 RON0.5 내부 tagged enum serializer가 shop merchandise type을 quoted string으로 출력하여 strict raw parser에 거부된1failed다. fixture가 authored identifier 표기로 출력하도록 교정한 [057](evidence/resource_shop_reward_057.json)은1passed/source change0다. 실제 PurchaseItem은 정확한 offer 가격만 차감하고, Reward Claim의 GrantEquipment도 새 copy UUID를 얻는다. 두 경로 각각 새 copy7→5/이전copy5유지/직원5→3 및 actual source copy UUID/최종3entries를 확인했다.

058은 missing-root 음성을 만들면서 strict typed checksum helper를 호출해 fixture 자체가 누락 field를 거부한1failed였다. Required field 누락은 codec의 typed parse가 checksum 전에 거부하는 경로로 분리하고059를 재실행 중이다. 추가 증거는 prior destination 보존·기존 Run 덮어쓰기 거부·load count2→3/최대 거부·peak high-water·같은 ID cap 변경의 fingerprint 거부다. 결과 전 PASS로 세지 않는다.

[059](evidence/resource_checkpoint_boundaries_059.json)은1passed/source change0다. required root 누락·checksum 손상·기존 Run 덮어쓰기·최대 load 거부 및 source/목적 root 보존, load2→3와 peak high-water, 같은 resource ID의 cap 변경에 따른 content fingerprint 거부를 확인했다. current canonical 계약 세 문서를 catalog3/execution2/active-run V7과 실제 typed owner·수명·원자성으로 갱신했다.

Default-release public 소비자 준비로 ordinary CoreApplication replay example과 typed external probe를 추가했다. 새 tools 파일이 원래 recorder001의 추가 파일 검색 밖에 있으므로 원본을 보존하고 [recorder002](evidence/RUN_TEST_TOOL_002.py)에 tools의 rs/py/toml/lock 수집을 추가했다. 이후 실행은 이 successor로 입력과 변경 archive를 보존한다. 아직 빌드/공개 소비자 PASS가 아니다.

[060](evidence/resource_release_build_060.json) 기본 release CLI3종과 [061](evidence/resource_public_build_061.json) default-feature external combat/typed_resources/cost examples가 exit0/source change0로 빌드됐다. [062](evidence/resource_public_positive_062.json) 및 [공개 원문 결과](evidence/TYPED_RESOURCE_EXTERNAL_001/RESULT.json)는12cases passed/source change0다. 현재 공식 pack을 실제 생산/소비하고 외부 RON으로6life/initial/cap/operation/newID·독립Chain대Run누적을 확인했다. API/CLI receipts, ordinary CoreApplication의 actual owner/value/terminal facts가 같으며 Run trace on/off/gap/repeat도 일치한다. 이전 R04fixture는 수정하지 않고 현재 producer pack에 synthetic equipment/skill 두 파일만 복사했다.

읽기 보조가 pending terminal의 CompleteCombatResult 직접 CombatBattleState clone을 확인했다. typed before/after의 추가 native/authoring clone 및 terminal retained admission을 보강하고 실제 fault rollback을 검증한다.062는 현재 gameplay 공개 통과이며 이 비용 보강 이후의 final currentness 증거로 미리 재분류하지 않는다.

063 all-target cargo check는exit0/source change0다. Pending RunResourceResult에 당시 native limits를 private 보존하고, application의 pending command도 같은 limits로 인수한다. resource before/after 실제 vector/ID 보유량과 기존 공유 facts snapshot의 저장 extent를 합쳐 시작/최종 검증하며 facts를 재생성하지 않는다. terminal resource projection은 첫 새 vector 전에 cumulative retained growth를 인수한다. World source/entry/checkpoint/nonbattle rollback 및 Complete의 새 resource 소유 그래프 복사를 native preflight에 연결하고 Complete의 실제 CombatBattleState 복사는 authoring controlled_clone으로 바꿨다.064는 실제 nonempty pending result에 native clone/retained 및 authoring clone/work 경계 거부·원본/UUID/profile/결과 보존·정상 재시도 once를 실행 중이다. 결과 전 PASS로 세지 않는다.

064는 test unwrap_err가 Debug 없는 private provenance를 요구하여 compile exit101/0tests였다. production provenance를 변경하지 않고 test의 err().expect()로 교정한065를 실행한다.

[065](evidence/resource_pending_budget_065.json)은1passed/source change0다. nonempty pending native clone/retained와 authoring clone/work 거부가 Run·UUID·profile·pending 결과를 보존하며 정상 재시도는 한 번만 반영한다. measured pending retained bound59728bytes, Complete authoring work30152/clone peak3201이다.066은16passed/1failed/source change0로 새 분해 보상 assertion이 잘못된 시험 장비 ID를 lookup한 오류였다. 실제 사본 meta.id를 사용해067 owner seam을 재검증한다.

[067](evidence/resource_dismantle_boundary_067.json)은1passed/source change0다. 실제 지출 사본의 분해 보상이 overflow하면 기존 사본/Run/다섯 grant roots가 보존되고 정상 분해에서는 기존 recipe의 정확한 material 수량만 추가된다. 읽기 보조가 tactical staging의 별도 entry 복사, floor/Run 완료의 직접 typed_resources clone, continue/load의 checkpoint 추가 복사를 확인했다. 해당 복사를 native/authoring 사전 인수로 연결하고 fallible floor builder는 기존 transaction으로 오류를 전달하도록 변경했다.068은 이 보강의 all-target compile 확인 중이다. 공개 output/fuel fault example과32/256 resource workload도 준비 중이며 아직 실행 PASS는 아니다.

[068](evidence/resource_clone_boundaries_check_068.json) all-target check exit0 및 [069](evidence/resource_world_budget_regression_069.json) 현재 자원 World17tests passed/source change0다. floor/checkpoint/tactical 추가 복사 교정 이후 실제6life/owner/carrier/Chain/Run/event/save/획득·분해/pending budget 경로를 재검증했다. 공개 보강은 실제 두 사본·32/256resource workload·13종 strict negative·terminal/Complete receipt와 total-work 거부를 준비했다. current package output에서catalog25+manifest26files를 확인하여 기존 producer probe count도26으로 정렬했다.070부터 release 재빌드 후 실행한다.

[070](evidence/resource_release_build_070.json)/[071](evidence/resource_public_build_071.json) current default release bins/examples 빌드exit0/source change0다.072는 기존12cases를 다시 통과한 뒤 두 번째 사본을 defense object의 실제 점유칸(4,6)에 배치하여 position_occupied로 실패했다. checkpoint의 실재 점유/빈칸을 읽고 새 fixture만(2,6)으로 교정한다. [073](evidence/resource_public_negative_073.json)은13종 missing/type/range/nonfinite/owner/lifetime/opcode/reference 음성이 API/CLI에서 두 번씩 같은 source-aware 진단으로 거부되고 이전 valid session/입력파일을 보존했다. [074](evidence/resource_public_limits_074.json)는 terminal/Complete의 receipt/total-work 네 경계에서 반복 거부·accepted observation/receipt 보존·정상 replay를 통과했다. 공개 부족/queue/clone 및확대 workload/비용 측정을 계속한다.

[075](evidence/resource_limits_build_075.json) expanded example build exit0.076의 기존12cases는 다시 통과했지만 두 번째 사본은 기본 짧은 사거리 밖에 있어 실제 attack/cost가 없었다. actual provider에 두 번째 copy UUID가 있고 cost None임을 확인했다. 두 사본 fixture에만 이미 World 시험에서 쓰는 명시적7×7사거리를 선언하여 실제 둘 다 OnAttack하도록 한다. 수정 없는12cases를 반복하지 않도록 probe에 explicit case selector를 추가하고 two_copies/resource_32/resource_256만080에서 실행한다. [077](evidence/resource_pack_regression_077.json)10commands는 current 공식producer/소비자/A→B→A통화500/733/500와거부/입력보존을 통과했다. [078](evidence/resource_cost_allocator_build_078.json) 별도 allocator example build exit0이며 plain071 executable은 /tmp/r05_cost_plain_001에 복사 보존했다. [079](evidence/resource_public_limits_079.json)는8cases passed/source change0로 first cost·terminal·Complete receipt/fuel 및 actual queue/clone fault에서 반복 거부·accepted prefix 보존·정상 replay를 확인했다.

080의 two_copies와resource_32는API/CLI/native 실제결과를 통과했으며 원문 progress에 보존됐다. resource_256은 accepted11 이후 긴60000ms 단일clock에서 native clone limit1048576/requested1050033/fact_clone_slots로 거부됐다. 이를 default 성공으로 세지 않고 별도 adversarial probe로 반복거부/accepted prefix 보존 및 명시적5000ms clock 분할의 완료를 검증한다.081/082 cost는 Complete 뒤 active battle 없는 reconnect의 정상 InvalidAction을 unwrap하여 각각exit101이었다. 기존 cost probe가 active ending만 가정한 것이며 새 terminal/Complete 입력에서는 거부를 명시적으로 검증하고 typed readonly query의 실제 무할당도 측정하도록 보완한다. 모두 source change0이며 성능 PASS가 아니다.

083/084의 current cost/adversarial examples는 default release로 빌드됐다. [085](evidence/resource_public_adversarial_085.json)는256자원의 native clone 거부 두 번과 accepted state 보존, 기존한도에서 명시적5000ms×4clock의 actual27450ms Victory/256entries 완료·재현을 통과했다. [086](evidence/resource_public_bounded_086.json)은같은 거부/분할실행을API/CLI/ordinary CoreApplication에서 대조해 receipt/facts/Run values가 일치하고 native실패before/after도 같았다. [087](evidence/resource_cost_plain_087.json)445samples/5repeats와[088](evidence/resource_cost_plain_large_088.json)580samples/5repeats의 비계측 비용 suite가통과했다. allocator089빌드 후091부터 별도 측정한다. [090](evidence/resource_checkpoint_profile_090.json)은1passed로 nonempty personal profile(최고99층/17연속/튜토리얼완료)을 가진목적지의 저장음성·동일ID변경거부·정상continue/explicitload에서 profile이되감기지 않음을 추가확인했다.

[091](evidence/resource_cost_allocator_091.json)/[092](evidence/resource_cost_allocator_large_092.json)는 각각267/348samples, 3repeats allocator suite를 통과했다. [비용 요약](evidence/COST_SUMMARY_001.md)/[정확한 집계](evidence/COST_SUMMARY_001.json)는 plain1025/allocator615samples와 실제 typed query99회 무할당·두 입력별12generation drop의 정확한 회수를 기록한다. System requested bytes는 RSS가 아니며 R04 RuntimeFactUsage bound는 새 typed store 전체를 포함하지 않는다. Complete의 순보유 증가를 회수로 오해하지 않는다.

093 typed_refusal default-release build exit0.094는 팩 선택 순서의 API/CLI/native 일치를 통과한 뒤 declaration 순서 변경도 같은 fingerprint라는 probe 가정에서 실패했다. 기존 resolver는 base source_order를 semantic input 순서에 포함한다(resolver.rs semantic_bytes 및 slot). 해당 가정을 교정하여 지문은 다르고 contract·최종 Run·actual outputs·terminal facts는 동일함을 검증한 [095](evidence/resource_order_095.json)/[원문](evidence/TYPED_RESOURCE_ORDER_002/RESULT.json)은 exit0/source change0다. 실제 두 번째 copy Consume99의 반복 부족에서 첫 employee Consume2도 취소되고 accepted state/receipts/native before-after가 같으며 CLI 진단도 API와 일치한다. 구현/기존 지문 규칙을 변경하지 않았다.

096은 원래 R01–R04 public/kernel/native 성능 oracle을 그대로 새 R05 evidence에서 재실행 중이다. 원래 baseline과 9,030,188bytes peak cap을 유지한다. 이전 native67은 고정 기대 횟수가 아니라 binary search 실행 수이므로 현재 actual count를 보고한다. 기존 guards script/과거 evidence는 변경하거나 덮어쓰지 않는다. Fresh predecessor audits 및 R05 GEX20은 아직 미완료다.

[096](evidence/resource_legacy_guards_096.json)은 exit1/source change0다. Public plain/allocator, kernel plain, native exact/one-below는 통과했으나 kernel allocator concurrent peak가9,307,156bytes로 원래9,030,188상한을 넘었다. 원래 cap/실행/실패 원문은 유지했다. R05의 RuntimeFactSource inline context16+delivery56+ticket.resource56=128bytes 증가가 일반 전투에도 반복 저장됨을 확인했다. 세 private immutable capability를 Arc로 압축하고 새 할당 전에 retained admission을 하며 source별 pointee와 scope/hold tokens를 계속 보수 계상한다. 명시적 scope 종료/hold 취소 권한과 store의 단일 writer는 그대로다. 098 all-target check는 production compile 성공 뒤 stale-ticket 시험이 Arc에 직접 ordinal을 쓰려 해 compile101이었다. 의도적으로 위조하는 시험용 ticket value만 as_ref().clone()으로 복사하여099 실제World17개를 실행 중이다. 새 Arc로 인해 기존 비용·release 증거의 currentness를 재검증해야 하며 091/092를 최종 current라고 표시하지 않는다.

[097](evidence/resource_predecessor_fixture_097.json)은 현재 실제 packager에서 새 predecessor fixture를 생산하고 이전 synthetic gameplay 차이 equipment/skills만 재적용했다. 과거 COMBAT_EXTERNAL_001 전체 입력 해시가 유지됨을 검증했다. source-receipt consumer의 파일 기대는 catalog schema3의25catalog+manifest26으로 정렬했다. 나머지 원래 source/rollback/privacy assertions는 유지한다.

[099](evidence/resource_compact_sources_world_099.json)은17passed/source change0다. [100](evidence/resource_legacy_guards_100.json)은 공개·plain kernel·native boundary가 통과했지만 allocator peak9,044,661로 기존상한을14,473bytes 초과하여 exit1이다. 감소 자체를 성공으로 세지 않는다. 자원 source의 context/delivery 두 포인터를 하나의 optional immutable capability bundle로 합쳐 일반 source의 추가 slot도 없앤다. 새 bundle은 resource 없는 source에 할당되지 않으며 모든 생성은 native retained admission 후다. source/command/cast/finite delivery에서 같은 context와 hold만 전달하고 observation은 bundle을 제거한다.101 all-target compile 확인 후102에서 같은 original guards를 다시 실행한다.

[101](evidence/resource_compact_bundle_check_101.json) all-target check와 [102](evidence/resource_legacy_guards_102.json)/[guards 원문](evidence/ORDINARY_GUARDS_003/RESULT.json)이 exit0/source change0로 통과했다. Public/kernel plain·allocator와 native 세 차원의 exact/one-below를 유지하며 concurrent peak9,025,562bytes < original9,030,188이다.103은 앞서 확인한 sandbox socket EPERM 때문에 동일 필수 all-target 범위를 승인된 환경에서 재실행 중이다. 당시 source090의 profile 보강과 최종 capability bundle을 포함한다. R05 전체/선행 fresh 감사는 아직 미완료다.

[103](evidence/resource_core_alltargets_103.json)은 exit0/source change0로 실제1410tests passed/0failed/0ignored다(lib1358 + integration4+4+25+15+4). 0-test binary harness는 수용 건수에 포함하지 않는다. current capability bundle·World17·profile 저장 경계·원래 literal/skill/owner 회귀가 포함된다.104 current default-release bins 재빌드를 진행한다. 이전 allocator executable은 /tmp/r05_cost_alloc_002에 보존했다.

[104](evidence/resource_release_build_104.json)/[105](evidence/resource_public_build_105.json)은 current default-feature bins 및 standalone public examples 빌드 exit0/source change0다. [106](evidence/resource_public_final_106.json)/[공개 최종15cases](evidence/TYPED_RESOURCE_EXTERNAL_006/RESULT.json)는 교정 source에서6life/선언값/새ID/독립Chain·Run/두 actual copies/32resource/256resource의 명시적 거부 및 clock 분할 완료를 모두 통과했다. 이전 source의 공개 증거를 소급 수정하지 않는다.

[108](evidence/resource_predecessor_diagnostics_108.json)은 원래 generated policy missing spawn-zone 및 boss summon zone/route source 진단의10commands를 current schema3/execution2에서 통과했다. 이전 pack을 수정하지 않은 successor fixture에서 반복 오류/빈 실패 출력/원문 복원/정상 재시도 출력 동등을 확인했다.107의 원래 CLI/API·Damage18/36·공명17/31·Buff·delivery·preview·source·runtime facts 소비자는 진행 중이며 현재 CLI21commands/6privacy compile negatives가 통과했다.109 strict typed negatives는 진행 중, [110](evidence/resource_public_limits_110.json)8limits 및 [111](evidence/resource_order_111.json)순서·원자적 shortage는 current release로 통과했다. 원래 kernel guard102와 alltarget103을 포함해 최종 current map을 준비하되 fresh 감사는 아직 수행하지 않았다.

[107](evidence/resource_predecessor_consumers_107.json)/[13묶음 원문](evidence/PREDECESSOR_CONSUMERS_001/RESULT.json)은 current release에서 모두 통과했다. CLI21commands/privacy6, ordinary API runtime5/privacy6, original Damage18/36·공명17/31·Buff·producer invalid13(29commands)·delayed/repeat/projectile/area·preview·R04 facts/source·failed/successful replacement·source receipt·ordinary application terminal parity를 원래 assertions로 실행했다. [109](evidence/resource_public_negative_109.json)은13 typed negatives의 반복 API/CLI source/phase 진단·valid session/입력 보존을 통과했다. [112](evidence/resource_cost_plain_112.json)은 교정 source의445plain samples/5repeats 비용 suite를 통과했고113 확대256resource explicit-clock plain suite를 별도 실행 중이다.

[113](evidence/resource_cost_plain_large_113.json)580samples/5repeats, [114](evidence/resource_cost_allocator_build_114.json) allocator feature build, [115](evidence/resource_cost_allocator_115.json)267samples/3repeats와 [116](evidence/resource_cost_allocator_large_116.json)348samples/3repeats 모두 exit0/source change0다. [최신 비용 요약](evidence/COST_SUMMARY_002.md)/[정확한 집계](evidence/COST_SUMMARY_002.json)는 final source의 plain1025/allocator615와99typed reads 무할당, 각 입력12generation의 retained→drop 회수를 확인한다. 원래 긴256clock의 bound refusal와 명시적 분할 완료는106으로 그대로 성립하며 한도를 올리지 않았다.

[현재 구현 입력](evidence/IMPLEMENTATION_INPUT_001.json)/[귀속](evidence/ATTRIBUTION_001.json)은349 source/data/contract/probe files를 보존한다. Captured dirty baseline 대비49modified+16added이며 recorder가 추가로 수집한64개 옛 build-generated files는 initial full status에도 존재한 재사용 입력으로 분리했다. 새 R05 구현으로 잘못 귀속하지 않는다. SOURCE_AFTER_001.tar.gz와 IMPLEMENTATION_DIFF_001.patch는 clean HEAD가 아닌 원래 dirty archive에 대한 정확한 delta다. Source는 final guard102·alltarget103·release104/105·공개106–111·비용112–116에서 바뀌지 않았다. 이제 요구별 대조/필수 선행 fresh 감사 복원/R05 GEX20 대상 고정을 준비하며 아직 Phase/Goal 완료 verdict는 없다.

[수용 대조](ACCEPTANCE_EVIDENCE_001.md)/[원시 map](ACCEPTANCE_EVIDENCE_001.json)은 원래33acceptance·12IVA·10차원·C01–08을 실제 passing witness와 연결했다. 첫 Markdown link 검사는 core_enhanced 기준 상위 경로가4단계여서 실패했다. source나 기준을 바꾸지 않고5단계로 고친 뒤51개 링크를 모두 확인한 [receipt](evidence/ACCEPTANCE_MAP_VALIDATION_001.json)를 보존했다. AUDIT 항목과 필수 선행 복원은 pending으로 명시하며 구현 검증을 독립 감사로 세지 않는다. PLAN/NOTES/Working Policy는 최종116까지의 현재성 및 source mutation 중단을 반영했다.

[선행 currentness002](evidence/PREDECESSOR_CURRENTNESS_002.json)는4대상/6,884고유파일에서 예상 밖 변경0과 historical external 변경0을 확인했다. Raw228/고유57의 차이를 [영향 판정](PREDECESSOR_IMPACT_001.md)으로 귀속했다. Audit restoration은 R01→R02→R03→R04→R05이며 아직 실제 fresh verdict는 없다. 전체 R05 문서 용량을 확인하려던 read-only recursive walk는 ignored build cache까지 포함하여 불필요하게 길어 중단했다. rg --files는5,979 evidence/planning files와 target cache0을 확인했다; 시험 실행 또는 실패가 아니다.

[선행 보충 currentness](evidence/PREDECESSOR_CURRENTNESS_SUPPLEMENT_001.json)는 별도 sealed audit registry/output68개와56개 external directory의 exact membership을 확인했고 mismatch/missing/extra0이다.002는 named manifest 파일 내용 검사이며 이 보충 검사가 디렉터리 목록 및 target 밖 historical audit 증거를 함께 고정한다. 이제 같은 stable source/evidence manifest의 R01target007/R02target007/R03target005/R04target004/R05target001을 작성하고 새 구현 비참여 reviewer에 순서대로 인계한다.
