from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'roadmap_v2.2-review/GOALS.json'
m=json.loads(P.read_text())
for a in m['acceptance']:
 a['supporting_owners']=[u['id'] for u in m['units'] if a['id'] in u['tests'] and u['id'] not in a['owners']]
P.write_text(json.dumps(m,ensure_ascii=False,indent=2)+'\n')
from render_documents import ROOT,OUT,REP,M,US,S,SS,V,header,write,link,refs,table,ulunit,findings
# Exact Goal impact report
p=REP/'GOAL_CAMEC_IMPACT.ko.md';t=header('기존 M0·GOAL-CAMEC-01 영향 판정')
t+='## 1. 권고: A — 현재 Plan과 최소 Goal을 첫 구현 단위로 유지\n\n현재 조사에서 이 최소 Goal의 **전제·범위·필수 선행을 수정해야 할 오류는 발견하지 않았다.** 외부 팩 전체, 파편 사본화, 공유 보호막, 완전한 Flow, 보스 확장, 유물 chain, 침식을 이 Goal 앞에 구현해야 할 기능 의존도 확인되지 않았다.\n\n이것은 이번 조사 범위의 권고이며 **정식 current/stale 판정, GEX 실행 자격, Goal activation, 구현 승인**을 대신하지 않는다. 사용자가 후속 구현을 요청할 때 기존 GEX 인수에서 그 시점의 current basis를 확인해야 한다. 이번에는 NEXT-0009를 실행하지 않았다.\n\n'
t+='## 2. 범위를 고정한 실제 근거\n\n'+table(['판정 질문','확인 결과','근거'],[
['embedded 최소 계약이 독립 유효한가?','현재 loader/compiler/실행 kernel과 검증 후 게시가 존재한다. metadata를 현재 경로에 연결하는 최소 slice는 외부 팩 게시와 독립이다.',refs(p,['S012','S013','S017'])],
['기존 executor를 재사용하면 곧 폐기할 구조를 고정하는가?','literal Damage/Heal/Instant와 현 타격/취소 의미를 보존하는 adapter는 재사용 가능하다. 전체 후속 Flow를 현 step 구조로 강제한다는 승인 내용은 아니다.',refs(p,['S016','S018','S019','S020','S021','S086'])],
['정의 주소를 사본/시전 ID와 혼동하는가?','Freeze/Goal은 작성 EffectSite/ActionSite/SourceMap과 runtime caster/cast/proc를 구별한다. 새 영구 사본과 generation/save는 제외한다.',refs(p,['S009','S065','S066','S092'])],
['계산/타격 수/공유 delivery/취소 의미가 보존되는가?','현재 ModifyDamage folding·signed Heal·HP-loss gate·released projectile/fixed area 존속을 직접 확인했다. 최소 Goal의 보존 조건과 맞는다.',refs(p,['S018','S022','S024','S065','S097'])],
['Atomic Group 범위가 충분한가?','WP01 compiler/descriptor와 WP02 latest RON/catalog/direct consumers가 기존 AG01로 묶이고 WP03의 durable oracle은 같은 Single Goal이 소유한다.',refs(p,['S010','S011','S009'])],
['저장을 먼저 다시 써야 하는가?','새 영구 state/format을 만들지 않는 최소 Goal이다. 현 V5 writer/V4·V5 reader와 metadata index 비영구성 확인을 보존한다.',refs(p,['S038','S039','S065','S092'])]])
t+='## 3. 그대로 재사용할 exact artifact\n\n공통 Topic 경로는 `core_enhanced/docs/goals/content_authoring_minimal_effect_contract/`다. 아래 모든 원본은 수정하지 않았다.\n\n'
rows=[
('WORKING_POLICY.ko.md','WPD-CAMEC-20260909 / WR-CAMEC-P2G20-001','정책 의미·범위/미정 분리 유지','없음. 새 정책 의미가 필요할 때만 P2G-10'),
('CODE_REVIEW_001.ko.md','기존 code-informed Review','기존 source/재사용 근거 인수; 이번 조사 F01–F21은 별도 보고서','기존 Review 덮어쓰기 없음. 실제 새로운 코드 사실이 전제를 깨면 P2G-20'),
('P2G_30_FREEZE_001.json','FPF-CAMEC-P2G30-001','literal slice/계산·취소 보존/사본·저장 제외 유지','이번에는 불필요. 승인 의미 수정이 생기면 가장 이른 정책 owner부터'),
('P2G_40_PHASE_PLAN_002.json','PP-CAMEC-P2G40-002','AG-CAMEC-01(WP01+WP02) + WP03 단일 Phase 유지','이번에는 불필요. 실제 atomic/dependency/scope 수정 필요 시 P2G-40'),
('P2G_40_PHASE_PLAN_VALIDATION_RECEIPT_002.json','PVR-CAMEC-P2G40-002','기존 receipt를 당시 증거로 인수','새 검사라고 세지 않음. 실제 Plan successor가 있어야 대응 검증'),
('P2G_50_GOAL_SET_001.ko.md','GS-CAMEC-P2G50-001 / GOAL-CAMEC-01','현재 최소 Single Goal·검증 owner/원문 acceptance 유지','이번에는 불필요. 상위 Plan/Goal 내용 변경 때 P2G-50'),
('P2G_60_HANDOFF_001.json','HOF-CAMEC-P2G60-001','현재 선택된 basis와 한계/비범위 인수','이번에는 불필요. 변경된 upstream 인계가 필요할 때 P2G-60'),
('P2G_50_AUTHORING_REVIEW_001.ko.md / P2G_60_RESULT_001.json','기존 작성 검토/complete result','당시 표준 검증과 선택 상태를 인수','이 연구가 당시 해시·PASS를 새 값으로 재발행하지 않음')]
t+=table(['원본 파일','identity','재사용/영향','최소 수정 owner'],rows)
t+='그 밖의 CA00 intake/requirements/COV, 기존 Decision·감사 원본과 단계별 history는 그대로 보존한다. 이전 전체 의존성 검토가 미완료였다는 사실은 기존 M0 성과가 없다는 뜻이 아니다.\n\n'
t+='## 4. 지금 닫은 질문과 후속 구현의 상세 선택\n\n이번 조사에서는 실제 loader/publication, shared step semantics, direct cast/queued/impact consumers, 취소 뒤 payload 수명, durable 저장에서 정의 index의 비영구성이라는 핵심 경계를 조사했다. 따라서 “나중에 구현자가 필수 소비자를 알아서 찾아야 한다”는 막연한 B 권고로 돌리지 않았다.\n\n정확한 Rust 타입·source locator 소유 방식·ID 표현과 fixture 함수 이름은 기존 AG의 상세 구현 책임이다. 이 선택은 지금 최소 계약을 정확히 구현하기 위한 기술 상세이며 외부 팩·사본·Flow 전체의 선구현 승인이 아니다.\n\n현재 최소 Goal이 처음으로 해야 할 전환은 **metadata/compiler + 최신 RON/catalog + 실제 consumer**의 결속이다. source locator는 외부 팩에서 owned source로 확장될 수 있고, 개인 합성은 미래 R14에서 explicit effective-plan reference를 필요로 한다. 이 비용을 기록하되 지금 미래 상태를 phantom ID/빈 executor로 미리 실행하지 않는다.\n\n'
t+='## 5. 인접 이슈와 별도 owner\n\n**저장 문구:** `data_loading_contract.md`의 durable 단락 V4와 actual V5 writer/V4·V5 reader의 차이는 기존 작성 검토에도 기록돼 있다. 최소 Goal의 저장 비변경 범위를 확장하지 않는다. 해당 canonical 저장 owner가 정합성을 닫되, 이번에는 원문을 교정하거나 old reader를 제거하지 않았다. '+refs(p,['S039','S092','S094'])+'.\n\n**침식 시점/Breakdown:** 이 보고서는 실제 미구현 boundary와 구체 정책 질문을 R09로 분리했다. 그것을 minimum metadata의 선행으로 끌어올리지 않는다. '+refs(p,['S018','S040','S044','S046','S059','S060','S077'])+'.\n\n**외부 wire 소비:** 코어 projection의 영향을 기록했지만 서버/Unity 계약 불일치를 이 Goal의 전체 재작성 선행으로 사용하지 않는다. '+refs(p,['S090','S091','S095','S097'])+'.\n\n'
t+='## 6. A 권고가 바뀌는 조건\n\n현재 허용한 최소 metadata 전환만으로 새 persistent-state·구형/신형 이중 실행·계산/취소 의미 변경이 불가피하다는 **새 실제 근거**가 발견되면, 그 최초 문제 지점의 owner에 반환해야 한다. 정책 의미면 P2G-10, 코드 사실/전제면 P2G-20, Work Package·Atomic Group·Phase 구조면 P2G-40이 가장 이른 수정 owner다. 그때 exact artifact와 필요한 최소 successor를 지정하고 필요한 후속만 다시 검증한다. 지금은 그 조건이 확인되지 않았으므로 C 권고/정식 successor/재실행을 발행하지 않았다.\n\n최종 답: **GOAL-CAMEC-01을 첫 구현 단위로 유지해도 된다는 A 권고다. 별도 구현 요청 전에는 여전히 미활성·미승인이다.**\n'
write(p,t)
# Main substantive investigation report
p=REP/'REVIEW.ko.md';t=header('Simulator 콘텐츠 제작 기반 — 전체 의존성 검토·개정 보고서')
t+='## 결론\n\n**기존 M0 성과와 GOAL-CAMEC-01은 유지하고, 후속 로드맵은 필요한 최소 계약 중심으로 재분할하는 안을 권고한다.** 기존 `ValidatedGameData` 게시 경계, 계산/전달 kernel, 장비 사본/loadout transaction, 세계 상태 복제/commit, 보스 graph, SpawnKernel과 wave bookkeeping은 재사용할 수 있다. 현재 근거로 코어 전체를 다시 써야 한다고 판단하지 않는다.\n\n가장 큰 변경은 **보스 최소 호출·침식을 앞당기고**, **개인별 유물 합성의 지연 실행 소비자를 동시 전환에 포함하며**, **새 상태의 저장/복원을 도입 단계에 명시적으로 배정**한 것이다. R20/R23은 각각 복합 저장·최종 부하 검증이며 최초 저장/예산 도입이 아니다.\n\n'
t+='## 제출물과 확인 수준\n\n'+table(['결과','내용'],[['A. 권장 순서',link(p,OUT/'ROADMAP.ko.md','R00 인수 + R01–R24 기본 순서')],['B. 전체 대응',link(p,REP/'M_CA_MAPPING.ko.md','M0–M8 / CA00–CA23 전체 대응·재사용·한계')],['C. 관계 근거',link(p,OUT/'DEPENDENCIES.ko.md','기존 49개 edge disposition + 새 76개 typed 관계 + 16개 최소 동시 전환')],['D. 단계별 전환',link(p,OUT/'GOALS.json','실제 개정 JSON')+' 및 '+link(p,OUT/'goals/R01.ko.md','R00–R24 작업 카드')],['E. 최소 Goal 영향',link(p,REP/'GOAL_CAMEC_IMPACT.ko.md','A 권고·exact artifact·보존/수정 owner')],['F. 개정 배포본',link(p,ROOT/'APPLY.ko.md','별도 경로 적용 안내')+' · '+link(p,ROOT/'ROADMAP_STATUS_ADDENDUM.ko.md','현황 추가 기록안 — 미적용')]])
t+=f'현재 저장소 **{len(S)}개 파일의 기재된 구간**을 본문으로 읽었다. 전체 파일/전체 저장소/전체 COV의 완독률은 아니다. 기존 GOALS 946줄, A01–A08, 로드맵 계약/검증/정책 gate, 선택된 formal 원본과 핵심 code/RON 경계를 대조했다. **게임 테스트·빌드·runtime probe·성능·독립 감사는 실행하지 않았다.** 실제 수행한 검사는 배포 문서/JSON/그래프 정합성 검사이며 결과는 별도 receipt에 둔다.\n\n'
t+='읽은 source·범위·앵커/일부 Git 식별자는 '+link(p,ROOT/'evidence/SOURCES.ko.md','근거 색인')+', 중요한 조사 묶음은 '+link(p,ROOT/'evidence/RESEARCH_LOG.ko.md','조사 기록')+'에 있다. '+link(p,REP/'LIMITATIONS.ko.md','미검토·조건·재개 위치')+'의 제한을 함께 적용한다.\n\n'
t+='## 현재 작업공간과 권한\n\n대상은 연결된 simulator의 dirty 작업 트리다. 첫 확인은 branch `cleanup/simulator-dirty-worktree-20260803`, HEAD `f318414`, dirty=true였다. `ROADMAP_STATUS.md`는 v1.9 / 기록 0010이며, CA01의 P2G-60이 완료되고 다음 카드는 NEXT-0009다. 원래 자료의 당시 값만 되풀이한 것이 아니라 현재 본문을 읽어 인수했다. '+refs(p,['S001','S002','S066','S093'])+'.\n\n도구는 현재 workspace의 read/search/list/status/diff와 과거 실행 기록 조회만 제공한다. 코드 실행 기능 또는 repository write 기능은 없었다. 따라서 이 배포본은 **로컬 산출물**이며 저장소에 기록했다고 주장하지 않는다. 사용자 dirty 변경·정식 원본·production/RON/save는 수정하지 않았다. 전체 dirty 파일의 원자적 snapshot/hash는 없으므로 일부 source 식별자로 전체 현재성을 인증하지 않는다.\n\n'
t+='## 하나의 권장 기본 순서\n\n'+table(['묶음','권장 순서','주요 이유'],[
['인수/최소 전환','R00 기존 M0 인수 → R01 현 최소 Goal','현재 게시·실행 kernel을 이용하는 metadata slice는 독립 유효하다.'],
['제작 입력/관찰','R02 외부 팩·세대 → R03 공개 도구','새 런/저장과 실제 외부 RON 검증 포트를 먼저 닫는다. 도구 전체가 kernel 기능 선행인 것은 아니다.'],
['공통 의미/상태','R04 사건·결과·ActionHandle → R05 자원/수명 → R06 계산/성장 → R07 파편 사본','현 출처·실제 결과를 바탕으로 owner와 상태/계산을 분리하고 사본의 정상 지급/저장까지 닫는다.'],
['큰 후반 묶음에서 분리','R08 보스 최소 호출 → R09 침식','전체 Flow/shield/trap/유물 chain을 기다릴 이유가 없다. 침식 정책 미정 때는 해당 분기만 유예한다.'],
['실행 표현력','R10 Flow/대상/전달 → R11 가역/집중/기본 공격 → R12 보호막/방어 → R13 매개체','각 primitive가 실제 실행·종료·실패를 소유하며 후속 recipe를 위한 안정 계약을 만든다.'],
['유물과 공간 반응','R14 관리+최소 numeric 실행 → R15 closure/구조 recipe → R16 연쇄 → R17 관계/감염/집계','개인 plan과 async provenance를 먼저 닫고 compile-time closure와 runtime chain을 구별한다.'],
['통합/제작 인수','R18 보스 고급 확장 → R19 등록/획득/평가 → R20 결과/저장 통합 → R21 공식 RON/표현력 → R22 독립 제작 → R23 안전/부하 → R24 독립 감사','각 단계의 초기 정상경로/저장을 다시 통합 확인하며 독립 증거로 마무리한다.']])
t+='기본은 **순차 진행**이다. 기능상 필수 DAG와 위 전체 순서는 다르다. 정책/기능별 대안과 병렬화 조건은 DEPENDENCIES 및 각 카드에 따로 기재했다. R09가 미정이면 R10–R19의 독립 기능은 진행 가능하지만 R20 이후 관련 전체 완료는 주장할 수 없다.\n\n'
t+='## 반드시 같이 바꿔야 하는 핵심 경계\n\n'+table(['동시 경계','분리하면 생기는 문제','owner'],[
['metadata/compiler + latest RON/catalog + 직접 consumer','parser만 새 정의를 받고 runtime은 잘못된 기존 값/위치를 소비','R01 / 기존 AG-CAMEC-01'],
['외부 게시 + 새 런 데이터 선택 + fingerprint save/restore','state-only commit에서 세대 선택 소실, 같은 ID 다른 내용 저장 오복원','R02'],
['사본 aggregate + grant/equip/form/dismantle/recovery + profile/snapshot/save','개화/소유는 사본인데 장착·분쇄·저장에서는 종류로 합쳐짐','R07'],
['개인 effective plan + cast/queued/projectile/area + 개인 preview/평가입력','투사체 적중 때 전역 SkillId 원형으로 되돌아감','R14'],
['침식 대체/kill/suppression + withdraw/attempt 비용 + clock + 결과/Breakdown/save','마지막 적 처치 후 대가 누락, wave 중단, physical damage 잘못 증가','R09'],
['복수 보상 schema + plan/commit/validator + normal grant/save','등록만 되고 실제 선택·재지급·완료 기록의 의미가 엇갈림','R19']])
t+='위 표는 최소 경계를 요약한 것이며 전체 16개 묶음은 DEPENDENCIES에 있다. RelicCopy 관리와 Composer는 도메인 책임을 분리한다. 관리-only를 먼저 구현하는 분할도 유효하나 그 단계의 완료를 “효과 합성 실행 완료”로 부르지 않아야 한다. 본 기본안은 공용 코드 재작업을 줄이고 유물의 실제 동작까지 닫기 위해 R14에서 함께 인수한다.\n\n'
t+='## 세 가지 의존성의 구별\n\n**현행 호출 의존**의 예는 projectile impact가 SkillId로 정의를 다시 조회하는 것이다. **목표 전환 의존**은 개인 plan을 도입할 때 그 impact consumer도 plan reference를 소비하도록 같이 바꾸는 것이다. **진행상 권장**은 공개 CLI를 먼저 준비해 뒤의 RON 인수를 일관되게 하는 것이다. 이 셋을 같은 굵기의 필수 화살표로 표현하지 않았다.\n\n기존 CA03→CA04, CA08→CA10, CA09/13/16→CA18 등은 전체 기능 선행으로 유지하지 않았다. 반대로 source binding·지연 실행·저장 reader/writer·직원 대가처럼 분리하면 무효가 되는 부분은 명시적인 최소 계약/동시 묶음으로 남겼다.\n\n'
t+='## 상세 조사 결과\n\n아래 “관찰”은 기재된 현재 본문, “설계 판정”은 목표에 대한 권고다. “잘못 나누었을 때”는 전환 위험이며 모두 현재 게임에서 발생한 버그라는 뜻은 아니다.\n\n'+findings(p)
t+='## 기존 성과·검증의 재사용\n\nCA00의 요구/COV 기준선, CA01의 Working Policy·Review·Freeze·Phase Plan·검증 Receipt·Goal Set·Handoff는 새로 만들지 않는다. 기존 기록의 표준 검사 결과와 감사 PASS는 그때의 근거로만 보존한다. 실제 source의 변경 영향이 있으면 해당 판단·시험만 다시 확인해야 하며, 단순히 세션이 바뀌었다고 전체 P2G/GEX를 반복하지 않는다.\n\n현재 최소 Goal이 독립 유효한 이유를 조사했지만, 새 로드맵의 나머지 R들이 정식 Plan/Goal로 작성되거나 구현 승인됐다는 의미는 아니다. 후속 source/계약을 바꿀 때 가장 이른 owner의 정상 절차를 거쳐야 한다.\n\n'
t+='## 이번에 완료하지 않은 범위\n\n전체 COV/BOSS-COV 행 및 세 콘텐츠 원문을 이번에 다시 전수 재검증하지 않았다. 정밀 line-crossing geometry/성능, 모든 original CA markdown 카드의 별도 재독, 모든 old audit의 fresh 실행/해시 확인, 게임/부하/제3자 제작/독립 감사도 하지 않았다. 원문 승인/미정과 현재 실행 지원을 혼동하지 않고 해당 제한을 따로 표기했다. 이 때문에 본 배포본은 **코드 기반 개정 권고**이지 “전체 콘텐츠 제작 기반이 완성됐다”는 인증이 아니다.\n\n그 제한이 CA00/01 인수를 무효화하거나 모든 의존성 판정을 불가능하게 만들지는 않는다. 기존 49개 관계·24 CA·9 M의 disposition과 새 단위별 계약/검증 owner는 실제 개정 파일에 모두 제공했다.\n\n'
t+='## 요청의 마지막 두 질문에 대한 답\n\n**1. 어떤 순서로, 어디까지 함께 바꾸어야 하는가?** 위의 R00→R24 순서를 기본으로 삼되, 각 단계가 소비하는 최소 계약을 구별한다. 핵심 동시 범위는 metadata/실행 연결, 세대/new-run/save, 사본/장착/지급/save, 개인 plan/지연 소비자, 침식/death bookkeeping/직원 비용/clock/result, 복수 보상 plan/commit이다. 각각의 저장·거부·종료 경로는 나중 단계가 없어도 유효해야 한다.\n\n**2. GOAL-CAMEC-01을 첫 구현으로 유지해도 되는가?** **A 권고: 유지해도 된다.** 이번 실제 source 대조에서 먼저 수정해야 할 필수 Plan 오류는 발견하지 않았다. 정확한 영향과 보존할 artifact는 별도 영향 보고서에 있다. 이 답은 실행 승인이 아니며 NEXT-0009는 실행하지 않았다.\n'
write(p,t)
# Main navigation/handoff
p=OUT/'HANDOFF.ko.md';t=header('개정안 인수 안내')
t+='이 패키지는 simulator dirty 작업 트리의 **조사·개정 권고**다. source repository가 들어 있지 않으며 P2G/GEX 정식 원본을 복제해 새로운 authority를 만들지 않는다. 실제 저장소 접근과 현재 dirty 내용을 별도로 유지해야 한다.\n\n'
t+='먼저 '+link(p,REP/'REVIEW.ko.md','조사 보고서')+'와 '+link(p,REP/'GOAL_CAMEC_IMPACT.ko.md','최소 Goal 영향')+'를 읽고, '+link(p,OUT/'ROADMAP.ko.md','권장 순서')+'·'+link(p,OUT/'DEPENDENCIES.ko.md','관계/동시 전환')+'·'+link(p,REP/'M_CA_MAPPING.ko.md','전체 대응')+'·'+link(p,REP/'LIMITATIONS.ko.md','한계')+'를 함께 인수한다.\n\n'
t+='현재 관찰된 formal basis는 WR-CAMEC-P2G20-001 / FPF-CAMEC-P2G30-001 / PP-CAMEC-P2G40-002 / GS-CAMEC-P2G50-001 / GOAL-CAMEC-01 / HOF-CAMEC-P2G60-001이다. ROADMAP_STATUS는 v1.9 / 기록 0010 / NEXT-0009였고 구현 권한은 없었다. 새 R 번호는 기존 Goal ID를 개명한 것이 아니다.\n\n'
t+='후속 요청이 **검토**이면 조사/문서 범위만 이어간다. 별도 **구현 요청**이 있을 때만 그 exact target과 기존 GEX 인수 규칙에 따라 현재성을 확인한다. 이 파일 또는 NEXT-0009가 권한을 만들지는 않는다. 다른 후속 R은 아직 정식 Plan/Goal이 아니다. 필요할 때 담당 P2G owner가 이 근거를 사용해 형식화한다.\n\n'
t+='sourceledger의 전체 파일 해시는 대부분 비어 있고 일부 앵커/Git 짧은 ID만 있다. 따라서 새 작업의 source 변화는 영향 범위를 다시 읽어 확인한다. L03/L04/L05의 미검토/정책대기를 완료로 바꾸지 않는다. 이전 sandbox 경로가 다른 세션에 그대로 있다고 가정하지 않고 제공된 ZIP/실제 파일을 인수한다.\n\n'
t+='적용은 '+link(p,ROOT/'APPLY.ko.md','APPLY')+'와 '+link(p,ROOT/'ROADMAP_STATUS_ADDENDUM.ko.md','추가 기록안')+'을 따른다. 이번 배포본은 저장소에 적용하지 않았다.\n'
write(p,t)
# Revised architecture. Preserves logical design, with explicit concrete cutovers.
ARCH=[
('01_SYSTEM.ko.md','A01 — 시스템 구성·단계별 책임','S080','''## 1. 제작 입력과 실행 도메인

외부 source selection → snapshot/pack resolve → typed validation/compile → ValidatedGeneration → 새 런 pin → immutable definition/개인 effective plan → 각 도메인의 공통 executor 흐름을 유지한다. Flow는 일반 effects를 연결하고, Boss graph는 상태/task/progress를, World/Economy는 등장/획득/장착/결과를 소유한다. 하나의 무제한 VM이 모든 도메인 authority를 대신하지 않는다.

## 2. 현재 기반과 목표 변화

현재 `load_live_sources`의 검증 후 공개, GameCore/BattleCore의 데이터 참조, world clone/commit, skill carrier, boss compiled graph와 SpawnKernel은 재사용한다. R01은 현 경로의 metadata, R02는 외부 게시/new-run/save, R04는 사실/handle/source, R05 이후는 새로운 상태/표현력이다. R14가 개인 plan을 도입할 때 모든 지연 consumer를 같은 source binding으로 전환한다.

## 3. Identity와 권한

definition address, compiled index, persistent copy, employee, runtime unit, cast/action, strike, damage segment, delivery/activation/chain을 구별한다. immutable template은 개별 직원/사본 UUID를 품지 않고 binding을 받는다. World result authority, wave/victory, boss task, object lifetime, acquisition은 별도다.

## 4. 재사용의 조건

typed request를 기존 damage/heal/spatial kernel에 낮추는 adapter는 유효하다. 구형 parser/신형 parser 또는 서로 다른 의미의 이중 executor를 영구 병행시키는 것을 기본 해법으로 하지 않는다. 현재 최소 Goal의 old save reader 보존은 새 전투 의미 이중 실행과 다르다.

## 5. 관찰·검증

R03 공개 포트와 각 R의 positive/negative actual RON, source-aware trace를 연결한다. R21 대표 실행과 전체 COV 표현력, R22 독립 제작, R23 측정/회귀, R24 독립 감사를 구별한다. 미래 공통 기능을 확장할 수 있지만 현재 미지원 op는 publication error여야 한다.
'''),
('02_AUTHORING_AND_COMPILER.ko.md','A02 — 작성·compiler·지원 descriptor','S081','''## 1. 의미 단위

EffectSite는 개별 효과, ActionSite는 실제 target/delivery/완료 milestone을 소유하는 행동 주소다. tags/property는 타입과 capability가 있는 지원 표면이며 내부 Rust/JSON field path가 아니다. 같은 effect의 태그 조건과 전용 definition/form identity 조건을 구별한다.

## 2. 최소 전환과 확장

R01의 AG는 literal Damage/Heal/Instant metadata를 현 parser/catalog/runtime에 연결한다. 현재 ModifyDamage는 step-local이며 step 공유 target/delivery·효과 순서·signed Heal을 보존한다. R02에서 static source path를 owned pack/file/source span으로 확장한다. R06의 Expr와 R10 Flow는 실제 지원하는 새 typed vocabulary와 executor/시험을 함께 추가한다.

## 3. Compile 흐름

source bytes와 locator 확보 → raw schema/타입 → pack dependency/명시 합성 → 최종 참조 → effect/Flow/Boss domain compile → bounded graph/작성 지원 검사 → generated setup 검증 → 불변 세대 게시. 최종 loser 정의의 더 이상 쓰이지 않는 참조로 유효한 resolved catalog를 막지 않되 raw syntax/manifest 검증을 건너뛰지 않는다.

## 4. 식·Flow·결과 검증

값의 단위/부호/범위·산술·overflow·empty aggregation·평가 시점을 검증한다. result는 생산된 범위에서만 참조하고 무한 loop/자기부트스트랩을 허용하지 않는다. 자유 스크립트/일반 bytecode/임의 JSON patch로 타입과 authority 경계를 우회하지 않는다. 15개 분해 축을 그대로 15개 schema field로 만들지 않는다.

## 5. SourceMap과 지원 정보

source locator, definition/effect/action 주소와 compile index 매핑을 보존한다. runtime-supported는 parser/compiler/consumer와 음성 시험까지 존재하는 조합에만 붙인다. unsupported를 warning 뒤 무시하거나 빈 success handler로 공개하지 않는다. 현재 예제 RON은 채택된 schema로 실제 fixture가 된 뒤에만 실행 증거가 된다.
'''),
('03_RUNTIME.ko.md','A03 — 실행·상태·수명·실제 결과','S082','''## 1. Owner와 수명

StateOwner와 Lifetime은 직교한다. 직원/장비사본/파편사본/유물사본/activation/object가 소유하는 상태가 Run/Attempt/Deployment/Activation/Delivery/Chain 중 어느 종료를 따르는지 별도 선언한다. 공유 pool은 하나의 owner가 소유하고 수혜자 membership만 여러 개다. 정의 ID나 transient unit ID로 모든 상태를 묶지 않는다.

## 2. Action과 continuation

request admission, Accepted, Released, FlowFinished, EffectsSettled, Canceled/Failed를 구별한다. R04는 현재 carrier의 handle부터 제공하고 R08 boss adapter와 R10 structured Flow가 이를 소비한다. stale state/task generation의 결과가 새 state를 전이시키면 안 된다. 미방출 cancel과 이미 방출된 projectile/fixed area의 독립 수명을 구별한다.

## 3. 실제 사실과 반응

Damage/Heal/Cost request와 actual HP/pool/회복/처치 receipt는 다르다. pre-death 관찰이 필요한 상태는 정의한 시점의 snapshot을 남기고, mandatory death/wave bookkeeping과 optional OnDeath/OnKill/proc dispatch를 분리한다. 기본공격 OnAttack/OnHit의 현 의미를 일반 Damage로 무단 확대하지 않는다.

## 4. Modifier·membership·channel

R11은 source별 ledger/lease를 합성하고 종료 시 자기 항목만 제거한다. 반대 숫자를 더하거나 옛 stat 전체를 덮어 복원하지 않는다. channel과 기본 공격 overlay는 행동 lock/수혜자 membership/재입력/timeout을 명시한다. 이미 release한 payload snapshot은 이후 profile 변경과 구별한다.

## 5. 원자성·예산·관찰

현재 공개 operation의 후보 상태·resource·queue·RNG·ID·log가 실패에서 일관되게 commit/abort되어야 한다. 이미 완료된 이전 timestamp의 모든 상태를 무조건 되감는다는 계약은 만들지 않는다. 연쇄/소환/반복/graph microstep/원문 크기마다 도입 시점부터 bounded budget과 typed failure를 둔다. 최종 R23에서 현실 부하를 측정한다. 로그 retention/출력 여부는 새 인과 state의 authority가 아니다.

## 6. 저장

Run 상태는 도입 R이 checkpoint/entry/result와 함께 소유한다. live-process reconnect와 durable save를 구별한다. compiled index·pointer를 저장하지 않는다. R20은 여러 상태를 섞은 결과/후퇴/복원의 통합 인수다.
'''),
('04_COMPOSITION.ko.md','A04 — 효과 합성·활성 closure·연쇄','S083','''## 1. 관리와 합성

RelicCopy/직원별 3슬롯/독점/비호환/이전·회수·분쇄는 관리 aggregate다. Composer는 지원된 원형 효과와 loadout binding으로 개인 immutable effective plan을 만든다. 요구 효과가 없으면 장착 거부가 아니라 휴면으로 남긴다. 다른 직원의 조합을 직원 내 비호환으로 막지 않는다.

## 2. 최소 numeric slice

R14는 관리와 최소 numeric 실행을 한 작업 단위로 인수하되 도메인 책임은 분리한다. current cast/queued step뿐 아니라 projectile/area impact가 전역 SkillId를 다시 읽는 경계를 전환한다. 정의 catalog 설명은 유지하고 개인 preview/explain/평가 입력은 effective binding을 받아야 한다. 같은 SkillId의 직원/사본/형태/세대가 cache에서 섞이면 안 된다.

## 3. Selector와 단계화

전용 콘텐츠/형태 identity AND 해당 effect의 role/tags/property capability를 검사한다. 파편 전체 태그 합집합으로 매칭하지 않는다. 긍정 grant의 유한 최소 closure → activation/dormancy → 구조/태그/수치 변경의 명시 단계 → 최종 graph 검증 순으로 처리한다. 의미 없는 자기부트스트랩과 negative cycle은 거부하거나 승인된 층화만 허용한다.

## 4. Write/preserve 계약

Add/Scale/Replace의 타입·단위·합성 순서는 별도다. incompatible Replace는 마지막 장착/UUID 순서로 해결하지 않는다. 구조 recipe는 input/output 및 owner/source/lifetime/채널/영향받지 않는 효과의 preserve set을 선언한다. 포큐버스 2S와 태아 상태별 두 property의 교체값은 공통 무조건 ×2가 아니다. generated effect 주소는 provider copy/effect/input/local identity로 안정화한다.

## 5. Compile closure와 runtime chain

R15의 활성 closure는 runtime 반응의 visited 집합이 아니다. R16은 branch-local (copy_id,effect_id) ancestor를 즉시/지연 자식에 보존한다. A1.e→B1.e→A1.e만 차단하고 A2/다른 effect/독립공격은 구별한다. 진짜 join·정기 root 권한은 명시 정책이 없으면 해당 family를 미지원으로 남긴다. timer마다 새 root를 주어 순환 제한을 우회하지 않는다.

## 6. 관계/감염 recipe

R17이 object lifecycle·source-scoped membership·relation graph와 GroupByTile를 결합할 때 composer descriptor/validation/실행도 같이 확장한다. 설치물 연결망과 감염 집합을 같은 이름 때문에 하나의 global graph로 강제하지 않는다.
'''),
('05_BOSS_AUTHORING.ko.md','A05 — 보스 제작·task 포트·진행 권한','S084','''## 1. 유지할 구조

기존 Encounter→ActorBehavior→Pattern/State→Task→Result/Transition 구조, typed parameter/call frame, cooldown/priority·loop 검증, bounded microstep와 state activation을 재사용한다. 보스 전체를 플레이어 skill Flow에 흡수하거나 보스 이름별 실행 분기를 추가하지 않는다.

## 2. 조기 최소 호출 R08

raw BossTask·compile·공통 내부 Action admission·result mapping·target/facing·state exit cancel을 함께 추가한다. Player 전용 ManualSkill 요청을 그대로 호출하거나 void invoke로 실패/자격을 우회하지 않는다. Accepted/Released/EffectsSettled의 waiter를 구별하고 모든 정상 실패·취소·timeout을 graph에서 처리한다. release 뒤 payload는 기존 수명에 따른다.

## 3. 고급 확장 R18

R18은 상태 profile/가역 modifier·targetability/거리 조건·HP 연속성·임계치 skip, 실제 Priority/Weighted 선택자, 여러 actor·field-work signal을 확장한다. 선택자의 enum만 추가하고 FixedSequence로 fallback하지 않는다. cooldown은 후보 검토와 실제 admission의 소비 시점을 구별한다.

## 4. Blackboard·signal·권한

변수/신호는 타입·scope·writer를 선언한다. RON signal이 무제한 world mutation이나 보상 authority를 가지지 않는다. field_work의 RepairProtectedUnit-only 완료를 확장할 때 raw/validator/executor/boss 구독을 동시 전환한다. wave activation/required-for-victory/loot authority는 별도 owner다.

## 5. 소환과 실패

공통 SpawnKernel의 atomic preflight·idempotent request·placement를 재사용한다. 현재 boss/opponent 상위 adapter를 일반 actor/object 생성 포트와 분리한다. recursive spawn·zero-time graph·stale task completion은 유한성/세대 검사로 제어한다. 잘못된 task 요청은 정상 성공/fallback state로 숨기지 않는다.

## 6. 인수

R08의 기본 시전과 R18의 C03/C07를 별도 검증한다. C03은 HP/actor identity를 재생성하지 않고 phase skip/변신 중 death를 검사한다. C07은 작업완료→typed signal→억제와 Route/wave를 검사한다. 두 경우 모두 R21에서 실제 최신 공식 RON, R22에서 다른 작성자의 새 보스 조합으로 연결한다.
'''),
('06_PACKS_AND_TOOLS.ko.md','A06 — 외부 팩·세대·작성 포트','S085','''## 1. Source와 실행 입력

`game_resources/data`는 공식 authoring 원본, `live_data`는 선택된 runtime pack 입력이다. 자동 상호 덮어쓰기를 하지 않는다. 파일 감시기는 reload 요청을 만들 수 있을 뿐 진행 중 전투의 정의를 바꾸지 않는다. effect마다 파일을 다시 읽지 않는다.

## 2. Manifest와 명시 합성

pack ID/version·engine authoring/capability·dependencies·file inventory·명시 add/replace/remove를 검증한다. pack dependency 순서가 충돌 winner를 정하지 않는다. replacement chain 또는 명시 선택이 필요하다. 기본 full-definition Replace와 좁은 typed patch를 구별하고 최종 resolved catalog의 참조를 검사한다.

## 3. 입력 일관성과 안전

source bytes/path/hash를 확보하고 파일 수/크기·parser 깊이·경로 정규화/루트 탈출·절대경로·symlink·case 충돌 정책을 검증한다. 느슨한 디렉터리 읽기 여러 번이 원자적 파일시스템 snapshot은 아니다. 동시 변경은 bounded retry/명시 실패, 필요하면 immutable directory+manifest 교체를 택한다.

## 4. Publication/new-run/test reload

R02는 validate→compile→generated setup 뒤 불변 세대만 게시한다. 새 런은 briefing/starter부터 pin한다. 현재 world commit은 state/seed만 옮기므로 generation-aware factory/commit을 명시해야 한다. B실패 때 기존 A런은 유지하되 B요청을 A로 몰래 성공시키지 않는다. 시험 reload는 B검증과 새 시험 session 생성이 모두 성공한 뒤 교체한다.

## 5. Fingerprint와 저장

source bytes hash와 최종 의미 fingerprint를 구별한다. save는 최신 schema/contract/resolved content fingerprint를 기록하고 같은ID다른내용을 거부한다. 기존 세대의 메모리 강한 참조 유지와 구형 parser 이중 지원은 다른 문제다. 오래된 저장을 이어갈 수 있는 내용 구성을 사용자가 복원하거나 새 런을 시작할 수 있지만 파일을 임의 삭제/변환하지 않는다.

## 6. Describe/Validate/Explain/Simulate

R03의 library API를 단일 의미 구현으로 두고 CLI는 얇은 adapter다. Cargo의 autobins=false를 고려해 실제 public binary를 명시 등록한다. schema/API에 없는 fixture-only builder를 제작 SDK로 포장하지 않는다. source/expected/actual/관련정의/지원 capability를 진단하고 Explain/Preview는 state/RNG/revision을 바꾸지 않는다.

## 7. Trace와 비용

Trace는 source map, semantic occurrence, action milestone, resource before/after, composition decision/차단 chain을 설명한다. diagnostic history의 retention과 실제 ChainContext를 분리한다. retained generation/cache와 runtime execution budget을 각 도입 owner가 계측하고 R23에서 최종 측정한다.
'''),
('07_CUTOVER_AND_REUSE.ko.md','A07 — 현재 코드 전환·재사용·유효 중간 상태','S086','''## 1. 기본 원칙

이 개정은 현재 본문으로 확인한 producer/consumer·저장/종료 경계를 바탕으로 R 단위와 최소 atomic group을 제시한다. 기존 v2.1의 탐색 지도/전환 후보를 구현 완료로 간주하지 않는다. 각 R의 actual patch/test는 아직 미래 작업이다.

## 2. 주요 전환

R01은 기존 AG01 그대로 compiler/metadata/latest RON/direct consumer를 전환한다. R02는 fixed embedded source inventory를 외부 snapshot/resolver로 옮기고 run pin/save fingerprint까지 동반한다. R04는 equipment copy provenance와 receipts/handle, R07은 fragment count/progress 장부의 전체 reader/writer, R14는 모든 개인 plan consumer, R09는 침식/직원/clock/result를 묶는다.

## 3. 현재 kernel 재사용

damage/heal 산술·현재 carrier·spatial query·boss graph·SpawnKernel·wave bookkeeping·EquipmentLoadoutTransition·EconomyGrantState·world prepare/commit를 재사용한다. 같은 책임을 가진 영구 이중 시스템을 만들지 않는다. 코드 이름에 adapter가 있다는 이유로 legacy shim으로 간주하지 않는다.

## 4. 상태·표시·평가·시험

새 사본/form/resource/modifier는 도입 시 snapshot/save/entry/result와 private/public 실제 fixture를 함께 소유한다. 정의 catalog와 개인 effective projection을 구별한다. Certified 가중은 승인된 모델만 쓰고 새로운 입력의 정책대기를 숨기지 않는다. 유지할 회귀와 승인 변경으로 바뀔 의미를 따로 기록한다.

## 5. 최신 경로와 미지원

미지원 고급 op는 명시 거부한다. 아직 내용이 부족한 최소 playable profile을 공식 콘텐츠 완성으로 이름만 바꾸지 않는다. 구형 grammar/alias/이중 executor를 기본 해법으로 남기지 않는다. 최소 Goal의 저장 비변경 조건과 이후 실제 schema cutover는 서로 다르다.

## 6. 적용과 재검증

기존 dirty 변경을 보존하고 이 보고서는 새 별도 경로로만 적용한다. shared type/source 의미가 바뀌면 관련 receipt만 갱신한다. R23 최적화가 지원 의미를 바꾸는 경우 R22 독립 인수의 영향 부분을 재실행한다. 현재 문서 검사를 게임 독립 완료 감사로 세지 않는다.
'''),
('08_WORKED_EXAMPLES.ko.md','A08 — 예제의 보존 의미와 실제 검증 owner','S087','''아래는 기존 E01–E06의 의미를 유지한 개정 시험 설계다. **현재 parser가 받는 새 RON이나 실행한 fixture가 아니다.** 원문 제안 문법과 숫자는 시험용이며 공식 밸런스 승인값이 아니다.

## E01 — 범위 피해 후 자기 회복 한 번

Damage(bite)의 actual HP loss 결과를 만든 뒤 조건을 충족하면 Heal(self)을 한 번 실행한다. 예제 Attack100/자기500/1000/적2명500·방어0에서 피해200씩과 자기100회복을 기대하는 손계산은 기존 산술과 대조할 fixture 입력이다. 피해 footprint만 바꾸어도 heal의 scope/source/횟수는 변하지 않아야 한다. “유효 적중” 전체 의미를 이 예제의 HP-loss 조건 하나로 확정하지 않는다. owner: R06/R10/R14/R15, 실제 공식 연결 R21/C02.

## E02 — 보스 사격의 release와 settled

시험 clock t0 접수/t100 release/t300 impact에서 Released waiter는 t100에 다음 state, EffectsSettled waiter는 t300 뒤 결과를 소비한다. t50 state exit은 미방출을 취소하고, t150 exit은 승인된 detach-released 정책에 따라 날아가는 payload를 보존한다. 이전 state generation의 늦은 결과는 새 state를 전이시키지 않는다. owner: R04/R08, 확장 R18.

## E03 — 공유 보호막의 단일 전환

하나의 effect instance가 pool을 소유하고 여러 recipient는 membership을 가진다. 단일 recipe는 기존 적격 수혜자 중 하나로 제한하고 초기량을 2S로 바꾸되 source/수명/소진 종료를 보존한다. 600 pool에 감쇄후 A400/B350 순서면 B HP 손실150이라는 값은 입력이 정해진 시험 산술이다. 수혜자 loop에서 초기량을 여러 번 키우지 않는다. S식/개인공유 우선순위/침식 비교안은 policy gate. owner: R05/R12/R15/R21-C04.

## E04 — Hello/Goodbye

일반 시전이 Hello activation과 basic PiercingDelivery override lease를 만든다. 자연 만료는 lease 제거 후 Idle이며 Goodbye가 없다. 승인된 재입력은 잔여시간을 snapshot하고 Goodbye skill action을 만든 뒤 lease를 정리한다. Hello 직접 기본공격 출처와 Goodbye 직접 skill 출처를 구별한다. old weapon 전체를 snapshot했다가 복원하지 않는다. owner: R10/R11/R21-C03.

## E05 — 태아 집중과 단일 전환

channel activation이 행동 lease와 aura membership을 소유한다. 정규 기본 공격의 피해/속도만 수혜자에게 적용하며 provider Attack 전체나 summon/fragment damage로 확대하지 않는다. 입장/이탈/종료, release snapshot, windup 경계를 시험한다. 상태별 +20/+40/+80%와 +15/+20/+30은 각 property의 교체값이지 모두 스칼라 곱이 아니다. owner: R06/R11/R15/R21-C05.

## E06 — 래티씨아와 원천별 전파

내 source가 만든 small이 있는 eligible tile에서 비용 commit과 big 형태 전환을 함께 처리한다. node ID 유지/교체는 기술 선택이지만 old edge/event가 잘못 적용되면 안 된다. 연결망은 접근/선통과 사건에서 정해진 순서로 attack→consume하고 consumed/stale node를 재발동하지 않는다. 별도 감염 fixture는 source별 살아 있는 숙주와 위력을 snapshot→consume→damage 요청하며 기폭의 재감염/사망전파 suppression을 유지한다. 감염 membership과 trap graph를 억지로 하나로 합치지 않는다. owner: R13/R16/R17/R21-C06.

## 실제 증거로 승격하는 조건

각 E를 채택된 latest RON·expected trace/state·정상/실패/취소/종료 case로 만들고 현재 parser/compiler/executor와 공개 로딩 포트에서 실행해야 한다. 원문 E01–E06과 별도로 C07의 보스 작업 신호 및 전체 COV 표현력 대조도 필요하다. 코드/게임 실행 없는 손계산이나 문서 도식은 PASS가 아니다.
''')]
for fn,title,src,body in ARCH:
 p=OUT/'architecture'/fn;write(p,header(title)+'원래 논리 설계와 승인 목표는 유지하고 새 R owner/현재 코드 전환 경계를 반영했다. 새 문법·타입·포트 이름은 실제 구현 존재의 증거가 아니다.\n\n'+body+'\n원문 설계 근거: '+refs(p,[src])+'. 현재 코드 근거와 각 단계의 상세 동시 전환은 '+link(p,OUT/'DEPENDENCIES.ko.md','DEPENDENCIES')+' 및 '+link(p,REP/'REVIEW.ko.md','조사 결과')+'를 따른다.\n')
print('Rendered main report, Goal impact, handoff and all 8 revised architecture documents')
