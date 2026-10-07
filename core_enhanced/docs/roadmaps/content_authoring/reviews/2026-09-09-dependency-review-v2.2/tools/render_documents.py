from pathlib import Path
import json, os, re
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'roadmap_v2.2-review';REP=ROOT/'report'
M=json.loads((OUT/'GOALS.json').read_text());US={u['id']:u for u in M['units']}
S=json.loads((ROOT/'evidence/source_ledger.json').read_text())['sources'];SS={s['id']:s for s in S}
V=M['version']
def write(p,t):
 p.parent.mkdir(parents=True,exist_ok=True);p.write_text(t.rstrip()+'\n')
def link(p,target,label):return f'[{label}]({os.path.relpath(target,p.parent).replace(os.sep,"/")})'
def refs(p,ids):
 return ' · '.join(f'[{s}]({os.path.relpath(ROOT/"evidence/SOURCES.ko.md",p.parent).replace(os.sep,"/")}#{s.lower()})' for s in ids)
def table(head,rows):
 def esc(x):return str(x).replace('|','∣').replace('\n','<br>')
 return '| '+' | '.join(head)+' |\n| '+' | '.join('---' for _ in head)+' |\n'+''.join('| '+' | '.join(esc(x) for x in row)+' |\n' for row in rows)+'\n'
def ulunit(ids,p):return ', '.join(link(p,OUT/US[i]['card'],i) for i in ids) if ids else '없음'
def header(title):return f'# {title}\n\n**{V} · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**\n\n'
def findings(p):
 out=''
 for f in M['findings']:
  out+=f'## {f["id"]} — {f["title"]}\n\n**관찰.** {f["observation"]}\n\n**설계 판정.** {f["design_judgment"]}\n\n**잘못 나누었을 때.** {f["broken_state"]}\n\n**후속 owner:** {ulunit(f["owners"],p)}. **확인 한계:** {f["limits"]}\n\n근거: {refs(p,f["sources"])}.\n\n'
 return out
# Evidence index: ranges and symbols are the citation substrate, not pretend whole-source hashes.
p=ROOT/'evidence/SOURCES.ko.md'
t=header('현재 저장소 근거 색인')
t+='각 S 번호는 실제 read_file 본문을 읽은 파일과 구간이다. 검색 hit는 탐색에 썼고, 주요 판단은 기재한 연속 본문에 근거한다. 경로와 symbol은 재진입 키이고 줄 번호는 이 관찰 시점의 위치다.\n\n'
t+='**식별 한계:** 전체 source bytes를 내려받아 재해시한 것이 아니다. `source_ledger.json`의 앵커 SHA-256은 그 앵커 문자열만 식별하며 파일 내용 전체의 fingerprint가 아니다. 대부분 `file_sha256=null`이다. 일부 `git_diff` index header의 짧은 Git blob ID는 별도로 기재한다. HEAD·파일 크기·줄 수·앵커만으로 dirty 전체 snapshot을 확보했다고 주장하지 않는다. 기존 P2G 문서에 적힌 source hash는 당시 기록이며 이번 값으로 소급하지 않는다.\n\n'
t+=f'본문 조사 파일: **{len(S)}개**. 이 수는 전체 줄 또는 전체 요구의 완독률이 아니다. 전체 행/정밀 geometry/게임 실행의 한계는 '+link(p,REP/'LIMITATIONS.ko.md','한계표')+'를 따른다.\n\n'
for s in S:
 t+=f'<a id="{s["id"].lower()}"></a>\n## {s["id"]}\n\n`{s["path"]}`\n\n읽은 구간: '+', '.join(f'{a}–{b}' for a,b in s['ranges'])+'행. 유형: '+s['kind']+'.\n\n'
 if s.get('anchors'):t+='재진입 앵커: '+'; '.join('`'+x['text'].replace('`','')+'`' for x in s['anchors'])+'.\n\n'
 if s.get('git_worktree_blob_abbrev'):t+=f'Git 내용 식별자: HEAD `{s["git_head_blob_abbrev"]}` → 관찰된 worktree `{s["git_worktree_blob_abbrev"]}`. {s["git_identifier_evidence"]}.\n\n'
write(p,t)
# All candidate implementation cards from the actual JSON.
for u in M['units']:
 p=OUT/u['card'];t=header(f'{u["id"]} — {u["name"]}')
 t+=f'이 문서는 **로드맵 작업 카드**다. 정식 P2G Goal 또는 GEX activation이 아니다. 기존 CA 대응: {", ".join(u["old_ca"])}. 기본 순서 번호: {u["order_index"]}. 현재 상태: `{u["execution_state"]}`.\n\n'
 if u['id']=='R01':t+='정식 대상은 기존 **GS-CAMEC-P2G50-001 / GOAL-CAMEC-01** 그대로다. 여기의 R01은 그 대상에 대한 로드맵 대응 ID이며 새 Goal ID가 아니다.\n\n'
 t+='## 1. 목적과 먼저 필요한 설계\n\n'+u['design_contracts']+'.\n\n'
 t+='## 2. 구현 선행과 검증 선행\n\n'
 t+='기능상 선행: '+ulunit(u['implementation_dependencies'],p)+'.\n\n검증 선행: '+ulunit(u['verification_dependencies'],p)+'.\n\n진행상 권장: '+ulunit(u['recommended_after'],p)+'.\n\n'
 t+='이 목록은 선행 단위 전체의 모든 기능을 요구하는 것이 아니다. '+link(p,OUT/'DEPENDENCIES.ko.md','관계별 producer output와 consumer')+'가 필요한 최소 계약을 특정한다. 기본 순서는 별도로 보수적인 순차 진행을 택했다.\n\n'
 t+='## 3. 동반 전환 범위\n\n'+u['atomic_changes']+'.\n\n'
 if u['atomic_groups']:t+='최소 동시 전환 묶음: '+', '.join(u['atomic_groups'])+'. 서로 다른 atomic group이나 한 카드의 여러 기능을 모두 단일 runtime transaction으로 합치라는 뜻은 아니다.\n\n'
 else:t+='이 카드가 소비하는 선행 atomic group은 이미 유효해야 한다. 통합 검증 카드는 새 상태 저장을 처음 도입하는 자리가 아니다.\n\n'
 t+='## 4. 재사용·교체·제거\n\n**재사용:** '+u['reuse']+'.\n\n**교체·제거 방향:** '+u['replace_or_remove']+'.\n\n'
 t+='## 5. 직후 유효 상태\n\n'+u['valid_after']+'.\n\n다음 단계를 전혀 수행하지 않아도 이 약속이 충족되어야 한다. 미지원 후속 기능은 publication/descriptor에 명시하고 빈 executor·임시 성공·조용한 fallback으로 완료하지 않는다.\n\n'
 t+='## 6. 검증 owner·입력·기대 결과\n\n'+table(['항목','내용'],[['구현·자체 검증 owner',u['verification_owner']],['기존 수용 기준',', '.join(u['tests'])],['시험 입력',u['verification_input']],['기대 결과',u['expected_result']],['이번 실제 실행','없음. 아래는 향후 구현 owner의 시험 계획이다.']])
 t+='새 RON/schema/opcode를 도입하는 범위는 실제 parser→validation→compiler→executor와 negative fixture를 함께 소유한다. 새 상태는 필요한 save/restore/entry/result·취소·실패 무변경 시험도 이 카드에서 닫는다. 실제 명령·선택된 test·nonzero 건수·exit code·source revision이 없으면 PASS로 기록하지 않는다.\n\n'
 t+='## 7. 조건·미정·범위\n\n정책 gate: '+(', '.join(u['policy_gates']) or '새 정책 gate 없음. 원래 승인 범위와 근거의 현재성은 확인한다.')+'.\n\n'+u['extra']+'\n\n'
 t+='## 8. 실제 코드·정책 근거\n\n'+refs(p,u['sources'])+'.\n\n'+link(p,REP/'LIMITATIONS.ko.md','조사 한계')+' · '+link(p,OUT/'ROADMAP.ko.md','전체 순서')+' · '+link(p,OUT/'GOALS.json','같은 카드의 JSON 원본')+'\n'
 write(p,t)
# CA/M mapping
p=REP/'M_CA_MAPPING.ko.md';t=header('M0–M8 / CA00–CA23 전체 대응')
t+='**원래 M·CA는 누락 방지 추적 ID로 보존하고, 새 R은 구현·검증 작업의 권고 분해다.** CA01 전체 로드맵 범위와 지금 작성된 최소 GOAL-CAMEC-01의 범위는 같지 않다. CA01의 미래 확장을 다른 R에 배정해도 기존 Goal의 범위를 확대하는 것은 아니다.\n\n'
t+=table(['기존 milestone','기존 CA','새 대응 단위'],[[x['id'],', '.join(x['old_ca']),ulunit(x['units'],p)] for x in M['milestone_mapping']])
for c in M['ca_mapping']:
 t+=f'## {c["id"]} — {c["label"]}\n\n'
 t+=f'**직접 대응:** {ulunit(c["units"],p)}. **동반·확장 owner:** {ulunit(c["supporting_owners"],p)}.\n\n{c["disposition"]}\n\n**현재 확인 상태:** {c["current_status"]}.\n\n**제한:** {c["review_limits"]}\n\n근거: {refs(p,c["sources"])}.\n\n'
t+='## COV 기준선 보존 규칙\n\n기존 `records/CA00_NEXT-0002_COVERAGE.json`의 COV/BOSS-COV ID·source·line·status·owner_ca·policy_gates·actual_validation_receipt는 원본으로 보존한다. 본 패키지는 그 행들을 재작성하거나 완료 상태를 덧씌우지 않는다. 각 행의 owner_ca는 위 대응표를 통해 하나 이상의 새 R로 이어진다. 하나의 행이 여러 R을 요구하면 마지막 R 하나에만 몰지 않고 primitive 구현 owner와 통합 검증 owner를 구별한다.\n\n기존 기록의 354 COV/127 BOSS-COV를 이번에 새로 모두 재계수·재검증한 것은 아니다. 원문 공백·도구형 별도 범위·미정 수치가 있는 행은 그대로 남는다. 원문 해시가 달라지면 바뀐 행을 다시 읽고 영향 대응만 갱신해야 한다. 관련 source: '+refs(p,['S005','S006','S078','S075','S076','S096'])+'.\n'
write(p,t)
# Dependencies: all 49 old edges, every new edge, explicit minimum atomic boundaries.
p=OUT/'DEPENDENCIES.ko.md';t=header('실제 코드 기반 의존성 판정과 전환 경계')
t+='## 1. 관계 종류\n\n'+table(['종류','판정'],[['기능상필수','후행의 실제 소비 지점이 선행의 특정 계약/상태/결과를 필요로 한다.'],['동시전환','분리하면 reader/writer·authority·transaction·장착/저장/실행 의미가 불일치한다.'],['검증선행','후행 완료를 증명할 실제 데이터·관찰·실행 증거가 필요하다.'],['진행상권장','독립 구현은 가능하지만 재작업·공용 파일 충돌을 줄이는 순서다.'],['조건부','특정 정책·지원 capability·도입 순서가 정해질 때만 관계가 활성화된다.']])
t+='현행 call/data dependency와 **목표 구조로 전환하기 위한 설계 dependency**는 같지 않다. 아래 판단은 관찰된 source와 승인 목표에서 도출한 설계이며 실제 새 구현의 실행 결과가 아니다.\n\n'
t+='## 2. 기존 49개 선행 관계의 전수 disposition\n\n원본 `GOALS.json`의 CA별 dependency 목록을 기준으로 49쌍을 모두 남겼다. 삭제/축소된 화살표도 이유·현행 재사용·조건이 있다.\n\n'
for e in M['old_dependency_edges']:
 t+=f'### {e["id"]}: {e["from"]} → {e["to"]}\n\n**판정:** {e["disposition"]}. **관계:** {", ".join(e["relationship_types"])}.\n\n**선행 결과 → 실제 후행 소비:** {e["producer_output_to_consumer"]}.\n\n**이유:** {e["reason"]}. **잘못 해석/분리하면:** {e["broken_if_ignored"]}.\n\n**기존 충족/재사용:** {e["already_satisfied_or_reusable"]}. **남은 조건:** {e["remaining_condition"]}.\n\n근거: {refs(p,e["sources"])}.\n\n'
t+='## 3. 새 단위 관계\n\n각 R의 모든 기능이 아니라 producer_output의 계약만 선행한다. 코드/저장/fixture의 동시 경계는 다음 절에서 따로 다룬다. 기본 순차 순서의 인접 단위가 모두 기능상 선행인 것은 아니다.\n\n'
for e in M['new_relationships']:
 t+=f'### {e["id"]}: {e["from"]} → {e["to"]} / {e["type"]}\n\n**산출:** {e["producer_output"]}.\n\n**소비:** {e["consumer"]}.\n\n{e["reason"]}\n\n'
 if e['condition']:t+='조건: '+e['condition']+'.\n\n'
 t+='무시/오해 시: '+e.get('broken_if_ignored','관계별 소비와 유효 상태 참조')+'.\n\n기존 충족/재사용: '+e.get('already_satisfied_or_reusable','source 참조')+'.\n\n남은 정책 gate: '+', '.join(e.get('remaining_policy_gates',[]))+'.\n\n후행이 약속한 유효 상태: '+US[e['to']]['valid_after']+'.\n\n근거: '+refs(p,e['sources'])+'.\n\n'
t+='## 4. 최소 동시 전환 묶음\n\nR 카드가 하나라고 전체 내용을 하나의 transaction이나 하나의 Git commit으로 합치지 않는다. 아래 component 간 공개 유효 상태가 끊기지 않아야 한다. R14 관리 aggregate와 Composer는 구조적으로 분리되어도 좋고, 기본 권고 작업 단위만 합친 것이다.\n\n'
for a in M['atomic_groups']:
 t+=f'### {a["id"]} — {a["owner"]}\n\n**같이 바꿀 경계:** {a["components"]}.\n\n**필요 이유:** {a["reason"]}\n\n**코드/계약 위치:** `{a["paths_and_symbols"]}`.\n\n{a["commit_policy"]}\n\n근거: {refs(p,a["sources"])}.\n\n'
t+='## 5. 순환과 병렬화\n\n원래 최소 Plan의 compiler/catalog/runtime 상호 의존은 **기존 AG-CAMEC-01**로 유지한다. 새 사본/저장의 writer-reader 결합과 효과 계획/지연 소비자 결합도 component 동시 전환으로 해결한다. executor stub·임시 성공으로 순환을 없애지 않는다.\n\nR23 최적화 뒤 R22의 영향받은 독립 제작 증거를 갱신하는 관계는 조건부 **재검증**이지 구현 선행 순환이 아니다. JSON에 `conditional_revalidation=true`를 별도 표시했다. 그 밖의 기능/필수 검증 관계가 기본 순서와 일치하는지는 로컬 정합성 검사 결과를 참조한다.\n\n기본은 순차 진행이다. 예를 들어 R08의 기본 호출은 R06/R07과 기능상 독립일 수 있지만, `ability.rs`/`sim.rs`/`types.rs`/RON fixture와 compile surface를 함께 바꿀 수 있다. R06과 R07도 profile·snapshot·save 경계가 겹친다. 실제 병렬화는 파일·schema·state owner·시험 디렉터리·병합/통합 검증 owner가 분리됐을 때만 별도로 허용한다. 화살표 부재만으로 병렬 가능을 선언하지 않는다.\n'
write(p,t)
# Main roadmap
p=OUT/'ROADMAP.ko.md';t=header('콘텐츠 제작 기반 — 개정 로드맵')
t+='## 1. 결론과 범위\n\n**기존 kernel/게시/상태/보스 graph를 재사용하는 부분 재설계**를 권고한다. 전체 재작성의 필수 근거는 발견하지 않았다. 목표는 제3자가 지원하는 명령·행동·조건·효과·규칙을 RON으로 조합해 보스·장비/무기·파편·유물을 제작하고 등록·교체·등장·획득까지 연결하는 것이다. 새로운 공통 엔진 기능의 추가는 가능하며 모든 미래 기믹을 무한히 표현해야 한다는 요구가 아니다.\n\nRON 조합기 UI, 외부 스크립트/플러그인 실행, game_server·Unity/client 구현은 제외한다. 작성 정보·검증·진단·설명·headless 시험 포트는 포함한다. 코어의 실제 소비/표시 경계는 기록하되 외부 API 불일치를 전체 코어 재작성의 근거로 사용하지 않는다.\n\n'
t+='## 2. 기존 성과 인수\n\nCA00 요구/COV 조사와 CA01 최소 계약의 P2G-60 Handoff를 그대로 인수한다. 현재 다음 카드는 NEXT-0009이나 이번 요청은 실행 요청이 아니다. **현재 GOAL-CAMEC-01은 첫 구현 단위 R01로 유지하는 A 권고**다. 정식 Plan·Goal·Handoff 원본 및 current 선택을 바꾸지 않는다.\n\n'
t+='## 3. 하나의 권장 기본 순서\n\n'+table(['순서','작업 단위','기능 선행','검증 선행'],[[u['id'],link(p,OUT/u['card'],u['name']),', '.join(u['implementation_dependencies']) or '—',', '.join(u['verification_dependencies']) or '—'] for u in M['units']])
t+='R00은 새 게임 구현이 아니라 기존 인수다. R01–R24는 위 순서대로 진행하는 기본 권고이며 모든 인접 관계가 기능상 필수는 아니다. R08 보스 최소 호출과 R09 침식을 큰 후반 기능 묶음에서 먼저 분리했다. R14는 최소 수치 유물을 실제 지연 전달까지 연결하고, R15/R16/R17에서 구조 합성·연쇄·관계를 차례로 확장한다.\n\n'
t+='## 4. 단계마다 독립적으로 유효해야 하는 것\n\n새 기능을 추가하는 owner는 schema/descriptor/parser/validator/compiler/실제 consumer/정상 및 거부 fixture를 함께 소유한다. 새 state는 필요한 snapshot/checkpoint/entry/결과/취소/실패 복원도 같은 시점에 도입한다. R20은 저장의 최초 구현이 아니고, R23은 예산의 최초 도입이 아니다. 기존 A03/A07/CA19/CA22가 이미 밝힌 의도를 현재 코드 경계에 구체화했다.\n\n동일 최신 경로 하나로 전환하되, 기존 kernel로 낮추는 typed adapter를 레거시 shim으로 오해하지 않는다. 최소 Goal에서는 기존 V4 reader/V5 writer를 보존한다. 외부 팩이나 새 사본 저장으로 실제 저장 경계를 전환할 때 해당 owner가 latest-format 전환과 승인된 손실/호환 조건을 소유한다. 이번 연구는 원본 파일 삭제/이동/변환 권한을 행사하지 않는다.\n\n'
t+='## 5. 정책 대기 때의 제한된 분기\n\n'
for b in M['conditional_branches']:
 t+='**조건:** '+b['condition']+'.\n\n'
 if b.get('defer'):t+='유예: '+', '.join(b['defer'])+'. 진행 가능: '+', '.join(b.get('may_continue',[]))+'. 전체 완료 차단: '+', '.join(b.get('blocked_completion',[]))+'.\n\n'
 if b.get('alternative'):t+=b['alternative']+'.\n\n'
 t+=b['note']+'\n\n'
t+='## 6. 검증과 인계\n\nT00–T29, C01–C07, K01–K10, PG00–PG09를 유지하며 '+link(p,OUT/'ACCEPTANCE.ko.md','검증 owner와 입력')+' 및 '+link(p,OUT/'COVERAGE.ko.md','전체 대응')+'를 함께 적용한다. 각 단계의 actual execution receipt와 R22 독립 작성/R24 독립 감사를 요구한다. 이번 정적 조사·문서 검사로 이를 통과했다고 하지 않는다.\n\n'
t+='## 7. 적용 지위\n\n이 로드맵은 후보 권고이며 정식 P2G Plan과 동시에 확정 실행 순서가 되지 않는다. 지금 formal target은 GOAL-CAMEC-01 하나다. 후속 범위는 해당 P2G owner가 이 근거를 소비해 필요할 때 새 formal 산출물로 만든다. 적용 절차는 '+link(p,ROOT/'APPLY.ko.md','별도 적용 안내')+'를 따른다.\n\n근거: '+refs(p,['S001','S005','S006','S009','S010','S065','S066','S080','S082','S085','S086','S088','S089'])+'.\n'
write(p,t)
# Contracts
p=OUT/'CONTRACTS.ko.md';t=header('공통 계약과 구현 owner')
t+='계약명은 목표 구조의 논리 식별자다. 동일한 Rust module/type 이름이 이미 존재한다는 뜻이 아니다. 최소 선언과 실제 capability를 구별하며 새 기능을 빈 descriptor/executor로 선등록하지 않는다.\n\n'
for k in M['contracts']:
 t+=f'## {k["id"]} — {k["name"]}\n\n초기 owner: {ulunit([k["initial_owner"]],p)}. 확장 owner: {ulunit(k["extension_owners"],p)}.\n\n{k["contract"]}\n\n근거: {refs(p,k["sources"])}.\n\n'
t+='## 새 상태의 저장 책임\n\nRun 귀속 state는 도입 owner가 최신 payload/validation/entry/result를 동반한다. Attempt/Deployment/Activation/Delivery/Chain state는 해당 종료·철수·retry/rollback·stale event·live reconnect를 소유한다. compiled index·Arc 주소를 durable identity로 쓰지 않는다. 소유자와 lifetime은 직교한다.\n\n## 공개 작성 지원의 일치 기준\n\n`runtime-supported`라고 공개한 op/property마다 실제 parser·validator·compiler·executor·descriptor·positive/negative 시험이 있어야 한다. 후속 계획 vocabulary와 현재 지원 목록은 별도로 표시한다. production 사용자가 문자열 내부 필드 path나 실행 가능한 코드를 제출하는 API를 만들지 않는다.\n'
write(p,t)
# Policy gates
p=OUT/'POLICY_GATES.ko.md';t=header('보존 정책·조건부 관계·미정 질문')
t+='승인된 사본별 개화, 직원별 공용 3슬롯, 외부 팩/새 런 고정, 경로별 재방문, 최신 침식 방향은 다시 미정으로 돌리지 않는다. 원래 PG 식별자를 유지하고 미정 family만 제한한다. 새 가격·수치·우선순위·손실 의미를 이 문서가 승인하지 않는다.\n\n'
for g in M['policy_gates']:
 t+=f'## {g["id"]} — {g["name"]}\n\nOwner: {ulunit(g["owners"],p)}.\n\n**유지:** {g["preserved"]}\n\n**조건/미정:** {g["open_condition"]}\n\n'
t+='## 침식의 구체적인 남은 질문\n\n1. `world/combat`의 Pause 중 수동 접수와 `sim`의 pending start_seq/focus 종료 사이에서 **한 번의 실제 cast-start 결정**을 어디에 두어야 기존 일반 접수/자원/취소 의미를 유지하는가? 기술 후보는 resume 후 첫 적격 battle-advance의 release 이전 지점이다. 이를 이미 승인된 hook이라고 단정하지 않는다.\n2. 준비 도중 자격/trauma가 변하는 경우 어느 snapshot을 인정하는가? 기존 접수 권한을 보존하면서 침식 조건의 “실제 시작” 의미가 바뀌면 P2G-10의 최소 정책 질문으로 반환한다.\n3. node 완료가 Breakdown 한계와 SavePoint/Gate 회복을 동시에 가져올 때 만료와 회복의 순서를 어떻게 적용하는가? 일반 node 완료당 1, retreat 미증가, trauma<100 즉시 폐기, checkpoint 잔여수 복원은 이미 승인돼 있다.\n\n이 질문들은 R09의 해당 계약을 제한한다. R01에 전체 침식 선구현을 요구하는 근거는 아니다. 근거: '+refs(p,['S018','S040','S044','S046','S059','S060','S061','S077'])+'.\n'
write(p,t)
# Acceptance
p=OUT/'ACCEPTANCE.ko.md';t=header('검증 owner·입력·기대 결과')
t+='**이 파일은 시험 계획이다. 이번 게임 시험은 실행하지 않았다.** 실제 local 문서 검사는 별도 `evidence/DOCUMENT_VALIDATION` 결과만 따른다. 과거 P2G validator PASS나 현재 test 함수의 존재를 이 T00–T29의 이번 PASS로 세지 않는다.\n\n'
for a in M['acceptance']:
 t+=f'## {a["id"]} — {a["label"]}\n\n기준 구현/검증 owner: {ulunit(a["owners"],p)}. 동반 회귀 owner: {ulunit(a.get("supporting_owners",[]),p)}.\n\n입력: {a["input"]}.\n\n기대: {a["expected"]}.\n\n현재: **{a["actual_status"]}**.\n\n'
t+='## 기존 시험 seam의 구체 재사용\n\n`core_enhanced/tests/skill_runtime_contract.rs::hit_gated_self_heal_and_buff_stack_repeat_attack_work_together`는 실제로 4step, opening hit 뒤 자기회복 1회, poison 3stack에 따른 barrage 3회를 assert한다. 이 시험의 원래 의미를 새 schema/compiler/consumer 경로에서 유지하는 것이 R01/R04/R10의 회귀 기반이다. 기존 private builder의 성공만으로 R03/R21 공개 외부 팩 인수를 대체하지 않는다. '+refs(p,['S070'])+'.\n\n정상/거부/취소/종료·중복/stale/저장 왕복 시험은 각 owner가 실제 selector와 nonzero 건수·exit code·revision·fixture fingerprint를 기록해야 한다. 아직 존재하지 않는 새 test 이름이나 명령을 실행된 것으로 표시하지 않는다. 안전·성능은 대상 hardware/build/workload가 없는 숫자를 결과로 쓰지 않는다.\n'
write(p,t)
# Coverage incl all axes preserved
p=OUT/'COVERAGE.ko.md';t=header('목표·대표 사례·전체 표현력 대응')
t+='전체 M/CA owner 대응은 '+link(p,REP/'M_CA_MAPPING.ko.md','전체 대응표')+'를 따른다. 원본 CA00 COV는 보존하며 이 문서의 대표 사례로 대체하지 않는다. **공식 최신 RON 이식**과 **전체 합의 구조의 표현력 대조**는 R21의 서로 다른 제출물이다.\n\n'
for c in M['representative_cases']:
 t+=f'## {c["id"]} — {c["name"]}\n\n기능 owner: {ulunit(c["owners"],p)}. 공식 RON/통합 owner: {ulunit([c["final_owner"]],p)}.\n\n보존할 구조: {c["requirement"]}\n\n조건: {c["condition"]}\n\n현재: {c["actual_status"]}.\n\n'
t+='## 대표 사례 외 구조\n\n고기벽의 segment HP/HP비용, 포자의 원천별 감염/사망 전파/기폭, 흑조의 감쇄 후 저장과 패링, 나비의 타일별 집계, 루돌타의 유한 반복/확률, 직원 Run 성장과 파편 사본 Run 자원을 별도로 추적한다. 원문 COV의 owner_ca에서 새 R로 이어지는 경로를 없애지 않는다. 보스별 actor/blackboard/parameter/signal 요구와 도구형 9종의 scope 후보는 대표 플레이어 파편에 흡수하지 않는다.\n\n## 분해 틀\n\n15개 분해 축은 원문 검토 틀로 유지한다. 단계·분기·병행을 먼저 읽고 개별 효과를 분리한 뒤 대상/전달/범위/계산/자원/사건/수명 등 해당 하위 축을 적용한다. 이 축들을 15개 schema 필드나 서로 독립된 15개 엔진 기능으로 기계적으로 옮기지 않는다. 소환물·변신·공유 자원의 별도 책임도 추적한다. exact 원문 분류·COV 행은 S006/S076/S078을 참조한다.\n\n## 독립 제작 판정\n\nR22에서는 엔진 revision을 동결한 뒤 다른 작성자가 새 ID/새 조합의 팩을 작성한다. 보스·장비/무기·파편·유물 도메인, cross-pack 참조/명시 교체, 둘 이상 스킬과 소환 또는 작업 신호·실패/취소 route를 포함한다. 이름·HP만 바꾼 복사는 부족하다. 공통 기능이 모자라 engine patch가 필요하면 그 기능을 보완한 뒤 재동결/해당 인수를 다시 수행한다. 이번에는 독립 작성자나 실행 receipt가 없다.\n\n근거: '+refs(p,['S006','S054','S057','S058','S075','S076','S078','S084','S087','S096'])+'.\n'
write(p,t)
# Limitations
p=REP/'LIMITATIONS.ko.md';t=header('확인 수준·미검토·조건·재개 위치')
t+='아래 제한은 해당 범위에만 적용한다. 일부 미정/미검토를 이유로 모든 코드 관찰이나 원래 M0 성과를 무효화하지 않는다. 반대로 조사한 관계/문서가 있다는 이유로 게임 구현·전체 콘텐츠·독립 감사가 완료됐다고 말하지 않는다.\n\n'
for l in M['limitations']:
 t+=f'## {l["id"]} — {l["limit"]}\n\n**영향 범위:** {l["affected"]}.\n\n**근거/현재 상태:** {l["basis"]}\n\n**정확한 재개 방향:** {l["resume"]}\n\n'
write(p,t)
print('Rendered cards, mapping, dependency, roadmap, contracts, gates, acceptance, coverage, evidence and limitations')
