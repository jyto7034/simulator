# U15 실행 기록

P2G-40 candidate001은 SCC 멤버 표기 오류로 실패했다. 실제 COPY/PAY/AUTHOR↔STORE 의존성을002로 교정하여 receipt 생성·검증 exit0/passed, P2G50 v2 exit0/passed. 원시 실패·성공은 상위 로그에 보존했다. 이는 문서구조 검사이며 Rust 기능시험0이다.

GEX00: U14 independent002와 current Core full001/socket002 근거 인수. BASELINE_SOURCE_001 archive member 검증 후 GEX10 착수. 제품변경 전 원본과 dirty 상태를 보존한다.

check001 exit127: shell PATH 공백 quoting 오류로 Cargo 미실행. check002 exit101: 새material type의 wire label match1누락. check003 exit101: 기존 fixture9곳의 필수필드 누락. 모두 원문 runs/에 보존했고 원인별 교정 후 u15_ 집중 테스트001을 시작했다. 빌드결과를 기능PASS로 세지 않는다.

tests001: u15_ 선택5/실행5/통과5/실패0/ignore0, exit0. build111초·test0.87초. 혼합결제/내림환급/확인/실패불변/SavePoint3회/Continue/귀환재대여를 owner fixture로 확인. 새 프로세스와 Normal host는 당시 미검증. raw runs/tests001.log.


## Core001 회귀 실패와 correction001
전체 Core all-targets(exit101): lib2031 PASS/70 FAIL/ignore6/socket별도1, 나머지 integration은 ron_loading24PASS/1FAIL 외 통과. 원문 runs/core001. 주요 제품 원인: 새 warehouse_debits map을 RunProgression serialized_leaf에 남겨 계량 활성 clone 경로에서 unreviewed native map 거절. 실제 필드별 CloneShape로 교정한다. fixture 원인: 옛 재료종류/방어구강화/무확인분쇄/버전15, 새 공식 recipe 중복 삽입. 원래 oracle 목적을 유지해 갱신한다. G11 고정 이전입력은 새 공개 귀환 결과 기반 successor 준비 후 전환한다. 기존 고정자료는 보존한다. fragment 기본+투자 환급을 재료별 합산해 query와 실행의 단일 수량을 맞춘다.


## Tests002–003 / shared currentness
Tests002(exit101)5PASS/1FAIL: 추가 rollback oracle에서 token 캡처가 profile을 default로 비워두는 기존 경계를 검출했다. restore만 바꾸는 것으로 부족하므로 transaction capture도 profile 자산을 포함해 비용과 함께 복제하도록 교정했다. 원문 tests002.log. SavePoint/Continue는 별도 delta 방식 유지.
Check004/005는 CloneShape 등록과 새 read-only query의 exhaustive 공개목록/fixture 필드 누락을 검출(exit101); 중복 GameMode 등록 제거, PendingContinuation Copy shape, 공개 command kind/정적 배열길이/DTO fixture를 함께 교정했다. Consumer-build001은 새 외부 harness의 SkillFragmentId/String 비교 컴파일 오류(exit101)이며 as_str로 교정했다.
Tests003(exit0)11실행/11통과/1.07초: 새로운 exact-source quote의 불변성, 부활촉매 공통 seam, 원장환급·장착확인·실패원자성·한도·미확정Continue·SavePoint이전소비·recipe가격변경 후재대여를 포함한다. 내부 owner fixture 증거이며 아직 외부 host 증거를 대체하지 않는다.
서버 actual writer의 opaque Core 허용목록에16을 추가했다. 구 profile4/5는 D01에 따라 현재6으로 묵시 해석하지 않고 header에서 미지원거절한다. 외부 source build002 진행 후 public pilot로 정상 동선의 선행조건을 먼저 확인한다.


## Tests004 / canonical dust correction
Tests004 exit0/11실행11통과/1.06초. 공개 phase2에서 typed GrantEquipmentDust 잔액과 옛 equipment_materials 잔액의 분리를 실제 검출했다. 공급 fixture를 바꾸지 않고 RunCraftingResources를 예약 equipment_dust id의 단일 owner로 연결했다. generic material grant도 이 owner로 들어가며 강화·환급·정비/Inventory projection·SavePoint/Continue는 같은 잔액을 사용한다. custom authored dust id는 기존 stack owner를 유지한다. 테스트 자산 불변 비교에 crafting_resources를 포함했다.
regression001의62PASS/4FAIL 원문을 보존한다. profile 전체복제 대신 transaction token에 lifecycle 자산만 추가해 지식 snapshot을 제외하는 기존 비용 계약을 지킨다. 나머지는 잘못된 RON weapon profile, version15 기대, 공식 무기 투자환급 기대를 교정했다. 이전 full-profile 캡처는 최종 설계가 아니다.


## Core002 / focused005 / public route
Core002 all-targets exit101: lib2097PASS/6FAIL/ignore6/filtered5(소켓1·G11 4); integration72PASS/0FAIL. 원문 runs/core002. 세 명령열거81개 기대, 두 checkpoint crafting_resources 필드 기대, U14 base dust3+salvage4와 bonus3+5가 같은잔액에 합산되는 기대를 교정했다. 정상값을 임의 대입하지 않고 보상 작성량과 개별 목적을 보존한다.
Focused005 exit0/37실행37PASS/20.01초: 실패6개, U14와 U15 전체, 새 현재 G11 default4 포함. 이 실행은 current source이며 원본 전체2097PASS+integration72는 비영향 경로로 재사용한다. 소켓001은 sandbox PermissionDenied 환경실패; 허용환경002는1실행1PASS. 두 원문 보존.
Public phase1–4_002 PASS: public Event 공급, depth3Gate 선택회수, 다음run 혼합결제+강화, 미완료 freshContinue, 분쇄+SavePoint, 완료 freshContinue, 투자유지 귀환을 실제 commandtrace로 증명했다. phase5_002는 정비중 Abandon 자격을 잘못 가정한 진행기실패였다. 명시 CompleteNode를 추가한 phase5_003과 phase6_003은 PASS: 재대여 실제원장환급/중복분쇄거절과 확정 소비 후 포기 미환급. 범위를 임의 건너뛰지 않는다. 원 consumer source build005/006, 후자는 기존 정상경로에 영향없는 reserved dust purpose loader검사와 시험교정, 진행기교정만 추가했다.
Authoring001 exit0: valid1 + 지원작성오류9를 public package/load에서 대조. 예약dust 목적변경, Armor강화, 파츠강화, Defense보정, 가루/촉매개화, 미지파츠, 0·중복합산overflow를 각각 거절. runs/authoring001/RESULT.json 및 작성입력/패키지를 보존한다.

## 최종 기능·일반 호스트 검증

Server001은 persistence 관련29실행29통과(exit0). G11 native001은 runtime-observation 기능으로 실제 예산 실패·취소·폐기·복원4실행4통과(exit0,5.68초)다. 기본 G11 4개는 focused005에 포함되어 중복 합산하지 않는다. 현재 기본 Core의 고유 통과 coverage는2180(전체2097+integration72+교정6+기본G11 4+소켓1), 기존 ignore6이다. 한 번의 전체 실행이 전부 초록이었다는 주장은 하지 않는다. 실패6개의 정확한 이름과 재실행 대조는 BASELINE_REUSE/TEST_RESULTS_INDEX에 있다.

Normal host/worker는 기본 release 그래프로 실제 빌드했다. instrumented-build001과 before-cost-build001은 F드라이브 용량 소진으로 실패했고, 소유한 새 산출물/캐시만 /tmp로 이동한 successor002에서 성공했다. 원문·receipt는 artifacts/와 runs/에 보존했다. 소스 컴파일 결속2961bda67387c7a3928ea7ff46fb349c2f52059991412d2d7f772b316fdfc8f3, dependency graph8f1ee6c6148407e4740fe7081e633d687efa05f369f5fc8ca9cb9d8e37ff68ae.

Calibration001은 필수 Event2를 건너뛴 준비 오류(no_eligible_payload),002는 seed13031에서 slot1이 Event인데 Combat을 기대한 오류다. public scenario pilot001이 실제 노드 종류를 드러냈다. 검증된 seed12077로 바꾼 pilot002는 이전 배치좌표(1,7)가 새 전장 배치영역 밖임을 검출했다. 공개 combat_preview의 ground cell(4,4)로 교정한 pilot003은24준비단계와10개1초 clock을 모두 통과했다. 교정 전 suite/driver/pilot 결과와 worker 보고서는 보존했다. 이는 준비 전략의 수정이며 목표 workload나 한도 완화가 아니다.

Normal calibration003/review003/adoption003은 실제 최종 host identity에서 모두 exit0. Plain/instrumented 각3반복, load/prepare/execute/restore 총12측정씩 완료했다. 기존 시간6초·native233000000·heap1342177280·RSS1610612736 한도를 유지했다. Plain 최대0.676초, instrumented 최대0.779초/native8445816/요청heap26194097/RSS54018048. 누락 counter는 null로 남겼다.

Normal replay001: 새 프로세스7개, 실제 명령165개, 모두 exit0/PASS. phase2 혼합개화·phase3 SavePoint복구·phase4 귀환에서 실제 journal 임시파일 경로를 디렉터리로 막아 IO실패3건을 만들었다. 각각 memory/disk 불변 확인 후 장애를 제거하고 같은 요청을 재시도하여 결과를 대조했다. phase3 뒤 실제 저장소 사본으로 phase6 포기·phase7 빈 회수선택 귀환을 따로 실행했다. 명령으로 획득한 공개 결과와 inventory/lifecycle를 정확히 비교한다. 저장 원문은 normal_host001.stores.tar.gz, 판정은 runs/normal_host001/RESULT.json.

Old-save001은 다른 normal replay와 승인 캐시를 동시에 사용해 cache.lock 경합으로 조기거절됐으므로 구버전 검증으로 세지 않는다. 순차 successor002는 실제 U14 version5 저장 원본의 사본을 현재 host로 읽어 명시적 unsupported version5를 확인하고 active/profile 본문·backup4파일 SHA가 모두 그대로임을 확인했다. 임의 버전/checksum 변조나 묵시 이행은 없다. 실제 원본 사본도 old_save_reject002.originals.tar.gz로 보관했다.

## 비용과 한도 검토

모든 컴파일/호스트 측정이 끝난 뒤 current_cost001 9개, paired_cost001 18개, 실제 투자원장 보충 paid_cost001 6개의 새 프로세스를 순차 실행했다. 모두12연산단계와 종료 시 요청할당량 정확회수를 통과했다. 현재 입력은0/6장비+6파편/384+384, 비교 입력은 전후 동일0/6+3/384+192, 투자 보충은 실제 두번째 귀환12+12(유료파편1)와64배768+768(유료파편64)다. 확장 fixture는 명시 I 준비이며 실제 게임 저장 변환이 아니다.

전후 바이너리는 모두 dev opt0/debug0/incremental0, 같은 공개 Tutorial/profile 연산을 쓴다. Before는 실제 U15 이전519파일 보관본과 변경 없는 외부 의존성으로 별도 빌드했다. After의 필수 빈 필드와 유효 catalog 변경은 비교 조건에 명시했다. 정상 command_prepare의 요청heap 증가720bytes, 확장16896bytes이고, 확장 Continue는15547bytes 증가했다. 시간의 작은 증감은 원시3반복과 함께 기록하며 최적화로 주장하지 않는다.

현재 확장 최대단계0.00571초/요청heap상승1220208bytes, 유료 확장0.00766초/2586673bytes. 실제 allocator peak·release와 RSS, 내부 native/retained 회계를 구분한다. 세부 max/median·원문·종료코드는 COST_REVIEW와 각 batch result에 있다. 정상 여유와 지원 실패 경계 근거로 기존 한도를 유지했으며 최종 배포환경 성능SLO로 확대하지 않는다.

Report001은 유료배치에서 실행하지 않은 empty-case placeholder(null)까지 max 계산에 넣은 보고도구 오류로 중단했다. 측정·기대값은 바꾸지 않고 sample_count0을 집계에서 제외한 report002가 통과했다. 부분 산출물과 실패 이유는 runs/report001_partial 및 report001.result.json에 보존했다. 기능시험을 다시 돌리지 않았다.

## 종료 준비

21수용·10차원·5통합을 COVERAGE.json에 결속했다. 구현자 코드 검토는 IMPLEMENTATION_REVIEW이며 정식 GEX20 감사가 아니다. 최종 고정은 test_parts/finalize.py가 SOURCE 한 번, archive equality, 격리경로 정역복원을 수행한다. 그 실제 결과는 target/u15_current_001/AUDIT_TARGET_RECORD.json과 RESTORE.json이 권위다. 이후 제품/시험/증거 편집 없이 GEX10 결과와 mutable 인계 locator만 기록한다. 자동감사·U16·Goal Complete는 수행하지 않는다.
