# U14 실행 — GEX-10 구현 pass001

현재(2026-10-14 교정001): 두 제품 결함의 교정·자체검증을 마쳤으며 비참여 재감사 대기다. 마지막 교정001 절과 `correction/u14_correction_001/RESULT.json`을 따른다. 앞선 구현pass의 최종 표시는 당시 이력이다.

대상 GS-CEDR-001 / GOAL-CEDR-01 / PHASE-CEDR-01. HA-CEDR-001, FPF-CEDR-001, PP-CEDR-001와 P2G40/50 통과를 인수한다. 사용자 U14 착수 요청 및 부활/무억제 곱셈 “1번” 선택으로 구현·자체 검증을 수행한다. 적용 지침은 P2G2026-10-04-r8/GEX2026-10-04-gex-r8 및 현행 상시 지시서다. 단독 직접 수행, 서브에이전트 없음. 중간 단계검토 opt-in 없음. 최종 구현·자체검증·target 동결 뒤 감사요청 인계에서 종료한다. 감사/U15 자동 착수 없음.

GEX-00: 이번 대화에서 처음 만든 exact Goal이며 이전 구현이력 없음. U13 audit/u13_independent_002 Complete/High/open0와 관련 선행 근거를 인수했다. U14 제품 수정 전 현재 dirty 포함 source baseline을 archive로 고정하고 모든 member hash/목록을 검증했다. 제품변경·Rust시험0. P2G 구조검사는 gameplay 검증이 아니다.

문서 역할: PLAN이 현재 범위/설계/검증위치, EXPERIMENTS가 실제 명령·실패·결과, EXPERIMENT_NOTES가 판단·미해결, STATE_INVENTORY가 소유·수명을 소유한다. 상위 CODE_REVIEW와 Goal Set의31수용 조건/10차원/10IVA를 따른다.

구현 순서: (1) 작성비용·규정유예4·입장 사망/자산회수·소진 순서·Continue/BattleEntrySnapshot, (2) 작성 난이도와 전투 진입 HP/ATK·보상 자격 snapshot/승리 whitelist2배·XP·실제 보너스, (3) authored 황금가지plan·봉인/후퇴/승리/지도·저장, (4) 실제 public/host/RON·회귀·비용 측정과 문서. 이들은 같은 atomic Phase 안에서 통합하며 부분 작업을 U14 완료로 보고하지 않는다.

첫 변경 설계: 비용은 노드 작성 입력에서 해석하되 first-entry receipt는 U12 단일 owner를 유지. Breakdown의 remaining_nodes 의미를 remaining_cost로 교체하고 입장 소비 함수만 감소시킨다. 완료노드 소비 호출은 제거하며 trauma변화의 reconcile과 사망 자산회수는 유지한다. Continue는 application이 미리 확보한 입장전 snapshot을 유지하고 수동후퇴 snapshot은 비용/유예 확정후 봉인해제전 capture. 새 shape/version을 지원하지 않는 구개발저장은 원본보존·명시거절한다.

초기 검증 준비: 세직원 중한명을 trauma100으로 만들고 두명은 출전가능으로 유지하여 단독 사망과 전체출전불가를 구분한다. 정상 authored node cost0/2, balance1/0에서 cost→receipt→grace→choice→activity를 관측. 치료·SavePoint 노드에서 grace2/cost2면 치료전 사망·자산회수이고 same-node repeat 무소모다. 종료Continue는 입장전, manual retreat는 입장후 상태를 기대한다. 새 저장/첫입장 상태와 재시도 상태의 선행을 분리한다. 공개 전투의 생존XP는 실제 참가자를 확인한 뒤 기대한다.

검증 운영: bounded loader/state roundtrip→실제전투 작은 oracle→normal application/host 프로세스복원/IO실패→관련회귀·측정. source/test/data 입력은 실행 중 고정한다. 명령·선택/실행/통과/실패/skip·원문·exit를 기록하며 0시험/빌드만성공은 PASS가 아니다. 캐시 CARGO_HOME=/tmp/wp02-cargo-home과 기존 target을 가능한 재사용한다. 최초 빌드 소요 미확정, 60초마다 CPU/log로 진단하고 출력없음만으로 취소하지 않는다. 동일 입력 반복검사는 실패/변경/의문이 있을 때만 한다. 최종 한 target과 격리 정역복원으로 감사인계를 준비한다.

pass001 전투 검증 준비: HP/ATK는 difficulty 뒤 encounter multiplier를 적용하며 exact override도 마지막 배율을 보존한다. 실제 enemy spawn 및 late spawn을 관측하고 속도/방어 등 비대상은 대조한다. 보너스는 runtime damage mutation의 실효HP/해당 최대HP 비율의 최댓값 하나만 소유하며 DOT/여러 타격 합산을 배제한다. 생존XP는 실제 result employee의 unique identity와 alive를 확인한다. 봉인 테스트는 normal-boss 클래스의 지원 authored fixture, 원래 고정slot과 부활slot을 구분한다. 후퇴 시 비용후 snapshot, Continue 시 비용전 snapshot, Victory 시 자동정산된 snapshot을 관측한다. 공식 미지원 본체는 강제로 실행가능하게 만들지 않는다. 초기 작성plan은 A5/B4/C4 또는 A5/B7/C4, 각50가중치, 보충8이며 모든 후보 wing을 포함한다.

봉인 실제승리 fixture 준비 교정: Boss 승리는 required enemy 전멸이며 TimeReached는 Defense 전용이다. 직원은 자동 추격하지 않고 전술 이동은 작성 deployment cell에만 가능하다. 정상 RON 접촉 시험에서 arena의 일반 floor를 배치 가능 cell로 작성하고 spawn/obstacle은 유지한다. 배치→5초→공개 tactical preview의 인접범위/자격 확인→이동→실제공격/승리 순서로 원래 보상/회복/Continue oracle에 도달한다. 공식 미지원보스 본체나 공식 arena 배치규칙을 변경하지 않는다.


최종 상태: GEX10 구현·자체 검증 종료, target/u14_current_001 동결·격리 정역복원 후 감사 요청 인계. 최종 기능 실행 뒤 Core/data/server 편집 없음. canonical 문서의 writer 설명과 실행 기록만 정리했다. 현재 WPD 선택은 footer의 WR-CEDR-004이며 머리의003과 시험0은 P2G 작성 당시 이력이다. upstream 동결 원문은 변경하지 않는다.

재현: workspace root에서 CARGO_HOME=/tmp/wp02-cargo-home, CARGO_TARGET_DIR=.cache/u12-vault-cost-target, /home/blast/.cargo/bin/cargo를 사용한다. focused011의 선택은 raw log의82개 test name이며 `cargo test -p game_core --lib --offline --locked <filters> -- --skip u12_vault`로 실행했다. G11은 U12_G11_INPUTS=test_parts/g11_inputs001/manifest.json의 절대경로와 u12_vault 필터, 계측판은 --features runtime-observation. server는 game_server persistence:: 필터다. 외부 consumer/host/measurement는 각 *.result.json argv와 test_parts의 드라이버를 따른다. 과거 issuer key 재사용 없이 새 private0700 root로 actual host의 --cmva-calibrate→review_calibration.py→--cmva-adopt-calibration 순서를 수행한다. 이 순서와 비용 한도는 일반 승인 경계를 유지한다.


## 2026-10-14 GEX-10 correction001 — F-U14-001

사용자 U14 교정 요청으로 재개한다. 기존 감사 Not Complete는 당시 target001의 원문으로 보존한다. 감사 source704/evidence980 출구 확인과 현재 편집 대상3개 hash 일치를 인수하며 target001 archive를 정확한 preimage로 재사용한다. 현재 실행자는 이전 감사 후 이번 교정에 참여하므로 교정본의 비참여 독립 재감사를 수행하지 않는다. 교정·자체 검증·successor 동결 뒤 재감사 인계에서 종료한다.

설계: 봉인 조회에 room_id와 실제 node를 전달하는 내부 helper를 두고, 현재 전투/정산은 기존 current-map wrapper를 유지한다. node entry preview도 현재 방 wrapper와 명시적 room context를 구분하며 전체 시설 view는 순회 중인 방 context를 전달한다. receipt/cost·collected·monolith 제외·배율 결정은 기존 owner를 사용한다. 새 상태·저장버전·공개DTO·정책 변경은 없다.

검증 준비: 기존 감사의 지원 package/loader fixture를 분리된 correction probe로 재사용한다. 같은 슬롯의 boss가 두 방에 있어도 plan은 B1-01에만 귀속한다. 억제 초기12는 지원2/5/8/G 왕복 뒤6, 무억제 초기3은 첫방3비용의 소진 Continue 후0이다. 보스는 선택적이고 Gate 접근을 막지 않는다. 봉인180/170, 무억제150/150으로 작성해 중첩/오판정을 구별한다. 실제 회수 사례는 기존 weak-boss·public tactical contact 전략으로 slot1 승리→보충8→결과ack→지원경로→옆방을 사용한다. 같은 node/plan/marker와 양쪽 공개DTO, 조회 무변경, hidden identity, 새프로세스 Continue 뒤 상태를 확인한다. 구 감사 관측과 바이너리는 보존한다.

Core은 기존 U14/U13/U12 및 변경 영향의82선택을 재사용하고 새 경계 assertion을 포함해 묶는다. 별도 feature G11은 입력동일/조회만변경의 영향으로 재사용 여부를 판정한다. 정상 host의 journal/reward/codec 메커니즘은 수정하지 않아 기존 IO 증거를 재사용하며 새 Normal 승인을 주장하지 않는다. 공개 view 비용은 기존6표본 runner로 동일 workload를 새 binary에서 비교하고 한도 변경은 하지 않는다. 빌드 예상1–3분(직전 release81초), 첫진단60초,5분초과시CPU/log확인. 공개 반례는 수초,실제승리는 기존2초대,집중82시험은22초대 근거다. 실행 중 source/test/fixture를 수정하지 않는다.

교정 중 발견 C-U14-002: public002의 실제 수집은12→SavePoint11→보스10→보충18→Gate16으로 정상 진행하지만 새프로세스 Continue가 초기값12 상한 검사로 거절했다. 지원 경로 제품 결함이며 범위 안이다. node_flow checkpoint 검증의 공급 상한을 초기값+해당 snapshot에서 이미 회수한 가지수×보충량으로 계산한다(넓은 정수의 saturating 계산). 실제 지급 checked_add·수량·저장형식은 유지한다. 기존 실제승리 Core 사례에 balance12를 추가해 초기값 초과 복원을 검증하고, 공개 왕복 뒤 두 번째 새프로세스 복원으로 수집 후 SavePoint도 포함한다. source/node_flow의 target001 preimage 일치를 추가 확인했다.

교정001 최종 상태: F-U14-001의 방별 봉인 공개 조회와 정상 승리 보충 저장에서 추가 발견한 C-U14-002를 교정했다. core00299/99, G11기본4/4·계측4/4, public0033/3(왕복 전후 새프로세스 복원6회), 계량6/6이 통과했다. 제품3파일+회귀1파일이며 새 상태/정책/저장버전/공개DTO 변경은 없다. 기존 Core138실패와 미지원 범위를 유지한다. correction/u14_correction_001/RESULT.json, EXECUTION_INDEX.json 및 MEASUREMENT_COMPARISON.json에서 명령·실패·수치·재사용을 구분한다. successor target/u14_current_002의 최종 snapshot/정역복원을 수행하고 비참여 재감사 인계에서 끝낸다. target001의 Not Complete 판정을 자체검토로 닫지 않으며 U15를 착수하지 않는다.

재현: correction/run.py의 EXECUTION_INDEX.json 명령과 환경을 사용한다. public003는 artifacts/u14_correction_probe correction-visited <game_resources 절대경로> <새 출력디렉터리>이며 정상 package/loader와 실제전투를 포함한다. 각 결과 디렉터리의 saved.json/returned_saved.json은 같은 packaged input을 U14_PROBE_INPUTS로 지정해 resume INPUT OUTPUT으로 별도 프로세스 재현할 수 있다. 모든 출력명은 기존 것을 덮어쓰지 않는 새 경로를 쓴다. G11은 기존 test_parts/g11_inputs001/manifest.json 절대경로를 U12_G11_INPUTS로 전달한다.
