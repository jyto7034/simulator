# U12 현재 기술 판단

권위는 FPF-CEXL-001의38수용 항목과 CODE_REVIEW T01–T07다. 이 파일은 현재 판단 요약이며 정책 변경 authority가 아니다. 현재 GEX10 부분 구현과 실제 Core/server 표적 실행을 진행 중이다. 정확 입력·선택·통과/실패 범위는 [EXPERIMENTS](EXPERIMENTS.md)가 소유하며 정상 전체 진행·비용·최종 독립 감사는 미완료다. 아래 준비 단계의 미실행 문장은 해당 시점 기록이다.

- 사용자 직접 결정: 기억/최근반응·사건 및 신뢰점수/band 시스템 완전 제거. 다른 personality/condition/영구 진행은 유지. 새 질문이나 보존 대안을 만들지 않는다.
- 메인 코드 기반 기술 결정: prepared draft와 기존 accepted journal을 재사용한다. 감소 가능한 vault ownership은 profile OR/max merge에 넣지 않고 exact lifecycle next state로 다룬다. approved/unbound publish,accept,recovery,sync/fallback 모두 같은 의미를 적용한다.
- 새 원정 전체 root reset은 manager와 authoritative profile을 보존한다. scenario seed는 지도/queue 재현성,instance sequence/namespace는 원정 개인/copy identity를 소유한다. tutorial은 영구 sequence를 올리지 않는다. 같은 원정 복원은 instance를 유지한다.
- 실제 현재 depth≥3 Gate 또는 실제 소진0전이 선택만 성공 귀환 자격이다. 소진 receipt는 최초 입장 소모 뒤 활동 전 결속하고0유지/깊이3도달로 재발생하지 않는다. 개발 원정3층의 실제 Gate/schedule이 필요하다.
- 성공 return→copy transfer/weapon enhancement reset→직원 종료→story pending을 한 draft로 확정한다. fail/abandon은 active만 잃는다. story ack with checkpointNone도 CAS 대상이며 no-change shortcut은 stale 검사 뒤에 둔다.
- same-instance SavePoint는 최신 profile knowledge/story를 되돌리지 않는다. 전체 profile revision equality를 강제해 합법 복원을 막지 않는다. vault∪active 단일 UUID 소유 검증은 dormant/cross-kind/key mismatch까지 다룬다.
- recovery는 실제 실행 가능한 치료/재장비 조합과 유효 채용/도달 SavePoint/Gate/귀환이 출전이나 귀환을 만들 수 있어야 한다. category만 존재하거나 끝까지 쓸모없는 치료는 회복 근거가 아니다. all-dead는 checkpoint failure decision을 유지한다.
- 구체 Rust API/DTO/version/가격/초기 억제력·비용은 기존 위임 범위의 제품 상세다. 공개 수명/권한/저장 의미는 메인이 실제 source로 검토한다. 실제 측정이 필요한 비용 기준은 실행 근거 없이 임의 완화하지 않는다.

적용 guardrail은 goal_execution_pipeline §6.4다. production test-identity 분기,조용한 잘못된 상태 보정,중복 authority,미래용 불필요 추상화,기대값을 위한 의미 완화,무의미한 sequence/delay 증가,근거 없는 golden 갱신,잔존 fallback을 완료로 인정하지 않는다.

## 구체 준비 검토 — 2026-10-12

메인은 product_parts/PREPARATION_001.ko.md를 읽고 현 정책 안의 구체 선택을 수용했다. Tutorial/Expedition enum,선발+required loan_copy_ids 한 출발 commit,typed Dealer/Tank/Healer,player+expedition sequence namespace,authoritative install과 knowledge merge 분리,Continue12/SavePoint13/profile4/active7/journal3 및 catalog6/execution3/expedition_lifecycle_v1을 함께 전환한다. 기존 StartNewGame의 replace_active_run=true는 명시 거절하고 진행 중 교체는 명시 Abandon 뒤 수행한다. terminal 뒤 start는 정상 허용한다. SkipTutorial은 시작/briefing blocking VN guard에서도 실행되고 중간 포기는 profile 불변 Abandon 경로가 남아야 한다.

초기 예산500·억제력12·첫 일반 노드 비용1·HQ 가격100은 승인된 authored tuning의 시작값이며 실제 정상 경로 검증에서 필요한 조절을 판단한다. 새 무제한 권한이나 gameplay cap이 아니다. 구체 type/function 이름 변경은 의미를 유지한 successor preparation과 독립 author 통지로 처리한다.

TEST_PLAN_001의38항목/19IVA/10차원과 파일 소유 후보를 검토했다. 실제 원정에서 사본을 만들어 사용하는 공개 양성과 내부 경계 음성을 구분하고,제품 evaluator를 호출해 같은 결과를 oracle로 삼지 않는 원칙이 적합하다. A-02-1의 새 tutorial 반복은 preset/초기 상태와 같은 노드 재시도 구분으로 관측한다. 정책에 없는 영구 tutorial sequence 또는 반복마다 반드시 다른 tutorial UUID를 추가 수용 조건으로 만들지 않는다. 새 Expedition 개인 identity는 별도 필수다.

baseline collector의 전체 git status30초 timeout은 원시 실패로 보존하고 실제 영향 root/canonical docs로 한정한 조회로 대체했다. 관계없는 역사 자료 전체를 검사해야 한다는 새 조건을 만들지 않는다. actual baseline 복원 증거를 인수했으므로 이 실패는 제품 쓰기 blocker가 아니다.

GEX00 baseline actual 증거 인수와 참조 hash 대조를 마쳤다. GEX_00_RESULT_001에 출구를 고정하고 제품·독립 시험 작성을 시작한다. 이전 baseline 대기는 해소됐으며 기능 PASS는 아직 없다.

## 구현 중 메인 source 검토

새 `game/expedition_lifecycle.rs`의 canonical 사본 상태·단일 UUID 소유·무기 강화 초기화와 `PlayerProfileState`의 knowledge/exact 채택 분리를 직접 읽었다. 구조 방향은 준비 결정과 일치하며 아직 전체 호출 경로 검증이나 기능 통과는 아니다. 구현 중 확인한 decoder 잔여는 vault fragment의 기존 bounded form 규칙과 raw vault map의 중복 UUID key 거절이다. 일반 BTreeMap decode의 last-wins를 ownership 증거로 인정하지 않으며, 제품 담당에게 수정하고 독립 시험 담당에게 raw 문자열 경로로 음성을 작성하도록 전달했다.

제품 mode 치환에서 test-only 두 파일이 잠시 편집된 오류는 `product_parts/MODE_EDIT_CORRECTION_001.json`에 보존했다. 즉시 역전한 두 전체 파일 SHA가 baseline과 일치하는 것을 메인도 재계산했다. retained oracle 변경이나 기능 PASS로 다루지 않으며, 후속 편집은 파일 소유와 production/cfg(test) 경계를 지킨다.

첫 독립 Tutorial oracle가 preset 출발 후 ViewingMap에서 SkipTutorial을 기대하는 것을 발견했다. PREPARATION001의 safe briefing skip과 중간 Abandon 경계를 확인해 test author가 해당 반복 경로를 AbandonRun으로 교정했다. 별도 blocking briefing SkipTutorial 양성은 유지한다. 지원 범위 확대 근거 없이 제품에 맞춰 양성 조건을 추가하지 않는다.

GEX §5 currentness와 GEX20을 직접 확인했다. U11의 mutation 전 Complete/High 인수는 유효하지만 공개 schema·required save consumer가 바뀐 후의 과거 verdict를 자동 current로 승계하지 않는다. 최종 current source에서 실제 영향 조건·현재 승인/저장 증거를 닫고, 필요한 선행 fresh read-only 감사 복원을 U12 전체 감사 전에 수행한다. 변경 영향을 입증한 불변 조건은 exact 이전 근거를 재사용하며 관계없는 선행 전체 감사를 관성적으로 반복하지 않는다.

서버 publisher 두 곳의 `expected_revision=None`이 prepare 뒤 최신 revision을 읽어 채택하는 것을 확인했다. exact lifecycle에서는 준비 중 다른 actor가 대여/종료한 뒤 stale draft가 이를 덮는 통로가 된다. 최초 actor는 prepare 전에 profile→active lock 순서로 profile와 revision을 함께 인수하고 Some을 고정한다. publisher의 None은 fresh nil 기준만 뜻하며 사후 최신 revision 보정이 아니다. no-change도 CAS를 통과한 뒤 결정한다. 제품 담당이 이 의미를 인수했고 독립 author가 A draft 이후 B 실제 출발·A None/Some stale 거절/양쪽 Core·파일 보존 oracle를 추가했다. 아직 실행 결과는 아니다.

NotStarted shell에 authoritative profile의 active 원정 marker가 남아 있으면 Tutorial/Expedition Start 모두 Core에서 거절한다. handler의 저장 guard만으로는 ApprovedHost 직접 소비의 checkpointNone tombstone을 막지 못한다. 기존 원정 Continue→Abandon 뒤 새 시작이 정상 순서이며, 독립 공개 시험은 원래 Continue 양성까지 함께 검사한다. normal host adapter의 force_revision도 실제 mod_launcher와 같이 StartNewGame에만 적용해 no-change/stale 경로를 숨기지 않는다.

## 로드맵 검토 반영 인수 — 2026-10-12

FINDINGS 원본을 보존하고 PLAN의 F01–F06 수정 범위를 mutable 현황/인계·로드맵·지도/조작 규칙에 적용했다. 문서 책임은 U13 가능한 첫 방의 예약/수용 보장→U20 실제 배정/저장, U26-A 전장과 전투 중 고유 선택→B 비전투/전후 선택→C 통합/복구로 연결했다. RON 도입은 각 단계의 기존 계획에서 authored/fixed/invalid/consumer/save/public 범위를 좁힌다. 미래 실제 전조·화면 검증은 해당 단계에 남으며 모든 finding의 제품 완료를 선언하지 않는다. U12는 기존 정책·Goal을 유지하고 인계 의무를 지금 적되 actual code/test 근거는 검증 뒤 연결한다. 동결 P2G/감사·실패 원본은 그대로다.

메인은 이번 mutable 문서 차이와 카드/규칙의 책임·제외·완료 의미를 직접 대조했고 관련9문서의 파일 링크359개가 URL decoding 포함 모두 존재함을 확인했다. 문서 수정 자체는 runtime/fixture/engine identity 입력을 바꾸지 않아 제품 검사를 추가 실행하지 않았다. 원 최신 인계006과 시험 담당 갱신 전007은 동일 bytes로 보존했다.

## 재개 후 저장 경계 검토

`ApprovedPersistence`의 준비 전 profile/revision pair와 no-change 전 CAS, accepted journal의 exact profile 발행/복구를 읽었다. 남은 실제 실행 검증과 별개로 profile-only actor에서는 `active_store=None`의 즉시 Core commit 뒤 `merge_and_persist`가 lifecycle 차이를 거절할 수 있음을 확인했다. 제품 담당은 prepared lifecycle 변경을 commit 전에 명시 거절하고 두 store 없는 memory·지식-only/Tutorial 불변 경로를 유지하기로 했으며 메인이 수용했다. 독립 시험에는 거절의 Core/파일 불변과 정상 대조를 요청했다. 손상 active 명시 폐기의 marker/tombstone 동시 정산은 기존 reset 원자성 잔여이며, 미지원 원본/required mod/accepted intent 보호와 수락 전 불변·수락 후 recovery pending의 실제 코드/시험을 확인해야 한다.

## 표적 실행 후 공통 전제와 남은 회복 판정

지도 슬롯의 고정 배치와 난이도 authored_fixed는 독립이다. 신규 원정의 고정 배치/default_facility 조합에서 writer가 target0을 만들던 실제 Continue 실패는 난이도 profile에 따른 generator 분기로 교정했다. 깊이5 이후 HQ는 채용 후보0의 정상 세션을 유지해 보급/상점 활동이 같은 종료 절차를 소비하도록 연결한다. 관련 시험은 단순 category/ID 존재에 그치지 않고 정확 roster/queue 소유, 실제 merchant 참조와 첫 페이지 상품 수·활동 종료까지 공통 전제를 점검해야 한다. 새 runtime 실패와 fixture 미도달은 EXPERIMENTS에서 분리한다.

정상 승인용 새 U12 probe는 기존 공식 지원 matrix에 root workspace package와 manifest 해시를 등록한다. 실제 Core dependency graph/compiler/target/profile/features/environment 검사를 보존하며 다른 executable의 승인 결과를 재사용하지 않는다. 기존 calibration의 lab16/500/1000명과1/8/8배치·60×1000ms는 gameplay roster10과 구별되는 authoring 부하다. 작은3인 공개 기능 준비가 이 부하를 대신하지 않는다. 새 원정의 HQ 완료 prefix 및 mode/required loan 배열을 현재화하되 측정 부하와 한도는 실제 검토 없이 낮추거나 높이지 않는다.

G07에서는 node 선택 제한과 실제 회복 payload 제한이 아직 함께 완결되지 않았다. 메인은 `helpers`의 ViewingMap/NodeConfirm 소비/장비 guard, `maintenance`의 일부 행동 후 재평가와 `action_capability`의 공용 payload 조회를 읽었다. 살아 있는 직원에 대한 OffenseBoost 등 비치료성 소모품이 회복 제한에서도 단순 Consumable eligibility로 허용될 수 있으므로 A10-1/2의 실제 회복 행동과 불일치를 해결해야 한다. 제품은 조회/명령이 같은 구체 판정을 소비하고 성공 행동 뒤 회복 가능성을 다시 평가하는 방향으로 설계 중이다. 치료 뒤 무기 재장착 같은 합법적인 복합 수순은 유지해야 하며, 매 단계 즉시 출전 가능해야 한다는 단순 조건으로 막지 않는다. 독립 회복 양성/음성 및 all-dead/checkpoint 경계의 실제 실행은 후속 묶음에 남아 있다.

추가 source 검토에서 승리 보상을 적용한 staged roster/inventory의 회복을 판정하면서 HQ 가격 비교는 아직 이전 `self.state.enkephalin`을 읽는 것을 확인했다. 보상 뒤 지갑으로 가능한 유료 채용을 보상 전 지갑 때문에 실패로 만들지 않도록 판정 문맥의 소유자를 맞춰야 한다. 다음 층 completion에서는 지도와 depth/receipt도 같은 시점인지 검토한다. active fragment의 호환 제한을 해제한 뒤 무기를 장착하는 합법 연속 수순은 공개 handler와 실제 planner의 도달 가능한 입력으로 확인한다. 이는 source finding이며 아직 제품 수정·독립 시험 또는 실행 통과를 뜻하지 않는다.

`product_parts/correction005`의 제품7파일과 successor `RESULT_002`를 메인이 실제 source/diff로 읽었다. current/staged 문맥을 명시적으로 빌리고, 허용 payload와 완료 후 단일 회복 재평가를 연결했다. Gate의 다음 층에서도 제한을 재평가하며, Return만 가능한 pending의 부적합 Continue는 공개 publication draft를 거절한다. sibling 모듈의 실패 처리 호출에 필요한 visibility 한 줄은 빌드 전 source 검토로 교정했다. 독립 G07 사례와 실제 검증은 아직 남으므로 source 검토를 기능 합격으로 승격하지 않는다.

공개 driver의 공통 전제는 실제 DTO와 정책을 먼저 대조했다. 공식 tactical 초기20/기본비용10에서는3명을 즉시 배치할 수 없으므로 실제100ms host tick의 자원 회복/eligibility를 읽어 순차 배치한다. 층을 관측한 사실과 ViewingMap에서 fresh application Continue를 수행한 사실을 결속하고, event cursor는 battle UUID가 바뀔 때 초기화한다. 같은 프로세스의 새 application과 새 프로세스 복원을 구분하며, 중간 경계 성공을 전체3층/귀환 완료로 표시하지 않는다. 이 작성 묶음과 회복 제품 변경을 동결한 뒤 compile010 및 공개 작은 준비 경로부터 실행한다.
