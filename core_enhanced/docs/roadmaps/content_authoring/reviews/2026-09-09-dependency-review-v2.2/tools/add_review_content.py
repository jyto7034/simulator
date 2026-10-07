from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'roadmap_v2.2-review/GOALS.json';M=json.loads(P.read_text())
K=[
('K01','정의/효과/사본/시전/타격/segment/전달 identity·SourceBinding','R01','R04 R07 R14','정의 주소만 먼저 고정한다. 영구 사본, 전투 actor, cast, 직접 strike, damage segment는 별도 key다.','S016 S018 S027 S031 S064'),
('K02','typed authoring·descriptor·SourceMap·compiled reference','R01','R02 R03 R06 R10 R14 R15','지원 opcode만 공개한다. 외부 pack 원문 소유권/위치는 R02에서 확장하고 AST 내부 경로를 public property로 쓰지 않는다.','S012 S013 S016 S017 S081'),
('K03','ContentGeneration pin·fingerprint·저장 일치','R02','R07 R14 R19 R20','새 런 briefing부터 고정, 기존 런 세대 보존. checksum과 content fingerprint는 구별한다.','S015 S037 S039 S040 S085'),
('K04','ActionHandle·milestone·terminal result·cancel','R04','R08 R10 R11 R18','Accepted는 완료가 아니다. 현재 carrier부터 port를 만들고 미방출 취소/방출 후 존속을 구별한다.','S018 S019 S020 S021 S022 S048'),
('K05','StateOwner×Lifetime·ResourceTxn·lease','R05','R07 R09 R11 R12 R13 R14 R16 R17 R20','owner와 수명은 직교. 새 Run 상태는 도입 단계에서 save/entry/result까지 소유한다.','S027 S034 S038 S039 S044 S082'),
('K06','EventEnvelope·Damage/Heal/Death/CostReceipt·원자성','R04','R09 R12 R16 R18 R20','요청과 확정 사실, 기본공격 트리거와 일반 damage, optional proc와 필수 death/wave를 구별한다.','S023 S024 S025 S026 S028 S071 S072'),
('K07','EffectSelector·PatchPlan·활성/휴면·write conflict','R14','R07 R15 R17 R19','최소 numeric 실행을 먼저, 유한 grant/구조 recipe를 다음에 연결한다. 전용 identity AND 같은 effect의 태그 검사.','S018 S020 S021 S045 S076 S083'),
('K08','ChainContext·fork/join·root 생산자·budget','R16','R04 R10 R13 R15 R17 R23','R04는 인과를 잃지 않을 최소 source/occurrence 포트, R16은 실제 branch-local 재방문/지연 전파를 소유한다.','S026 S027 S071 S072 S082'),
('K09','ScenarioSignal·actor binding·생성/진행 권한','R08','R13 R18 R19','기본 보스 요청은 R08, 객체 adapter는 R13, field-work 신호·다중 actor는 R18. Wave/보상 authority는 보스 내부가 아니다.','S047 S048 S049 S050 S052 S053 S084'),
('K10','진단·TraceReceipt·실제 실행 증거','R01','R03 R04 R08 R14 R18 R21 R22 R23 R24','초기 최소 진단→공개 도구→각 기능의 source trace. 진단 on/off와 gameplay authority는 별개다.','S013 S017 S068 S070 S071 S072 S085')]
M['contracts']=[dict(id=i,name=n,initial_owner=o,extension_owners=e.split(),contract=c,sources=s.split()) for i,n,o,e,c,s in K]
PG=[
('PG00','승인 구조·원문 공백·prototype 수치의 구별','R00 R01 R21 R24','CA00의 354 COV/127 BOSS-COV 기준선과 읽기 기록을 인수한다. 서사·원작·초안·미정·기술 제안을 승인 gameplay로 바꾸지 않는다.','전체 개별행 fresh 재검증/실제 실행은 미완료이며 R21 인수에서 별도 상태를 유지한다.'),
('PG01','typed property·Add/Scale/Replace·비호환','R06 R14 R15','다른 의미의 Percent를 동일 연산으로 합치지 않는다. 같은 effect의 태그와 전용 콘텐츠 조건은 별도 AND다.','미정 수식/우선순위는 해당 property/recipe만 중단. 서로 다른 Replace를 UUID/장착순서로 해결하지 않는다.'),
('PG02','활성 closure·연쇄 root/fork/join','R04 R15 R16 R17','compile-time 유한 활성과 runtime 경로별 (copy,effect) 재방문은 별도다. 형제 경로는 전역 visited가 아니다.','진짜 join·정기 event root의 exact 정책이 없으면 그 family를 미지원/정책대기로 남긴다.'),
('PG03','타격 인정·snapshot·동시 시점·cast 의미','R04 R06 R08 R09 R10 R11 R18','현재 HP-loss gate/basic OnAttack/OnHit는 보존한다. 새로운 실제 타격 인정·같은시각 우선순위는 해당 규칙으로 닫는다.','침식의 실제 cast 시작 vs Pause 중 admission/준비/release, reinput와 timeout, pre-death 관찰의 미정 경계를 명시한다.'),
('PG04','감쇄·pool·HP비용·패링·생존·특수 kill','R12 R13 R17 R18','공유량을 수혜자 수로 복제하지 않는다. HP비용은 Damage로 위장하지 않는다. 침식 특수 kill과 ordinary shield는 구별한다.','개인/공유 보호막 순서·HP1·처형/패링 중재·예외 수치는 임의로 확정하지 않는다.'),
('PG05','사본/형태·자원 수명·성장·저장','R05 R06 R07 R09 R14 R20','개화 해금은 파편 사본별. 소유 직원 이전·form 변화·재배치·attempt 종료는 다른 사건이다.','형태 변경/이전의 자원 보존 세부, 신규 저장 손실/호환 정책은 해당 formal owner가 기존 결정과 대조한다.'),
('PG06','공식 콘텐츠 수치·recipe·획득/선택','R07 R13 R14 R15 R17 R18 R19 R21','기존 승인 구조·동일 사본 독점·직원 3슬롯은 유지한다. 일반/퍼펙트 보상·미정 획득/분쇄량을 발명하지 않는다.','공식 S/지속시간/확률/재료량 미정은 test-only 입력과 분리한다. 정책 대기를 완료로 치환하지 않는다.'),
('PG07','외부 팩 합성·새 런 세대·fingerprint','R02 R03 R19 R20','명시 replace/충돌/최종참조 검증, 새 런 pin, 실패 candidate 미게시. 사용자 live_data 자동 덮어쓰기 없음.','정확한 manifest/fingerprint 표현·symlink 정책은 기술 계약으로 닫되 저장 손실/새 호환 정책을 대신 승인하지 않는다.'),
('PG08','실행 예산·성능 측정·Certified 가중','R02 R03 R04 R06 R10 R13 R16 R19 R23','기능 추가부터 유한성/예산/fault를 소유하고 최종 현실 부하에서 측정한다. 현재 skill 존재 정액을 새 효과 평가로 포장하지 않는다.','안전 상한과 gameplay cap 구별. 목표 장비·측정 기준·새 평가 가중은 근거/정책 없이 채우지 않는다.'),
('PG09','침식된 유물 상태 vs 파편 침식','R09 R14 R15 R21','둘은 다른 기능이다. 파편 침식은 세트4종/별도버튼 없이 실제 시전 조건 대체, 최종 보스 파편 제외다.','공통 trauma threshold·개별 authored 연출 지속시간·침식 유물 세부는 관련 가족만 gate한다.')]
M['policy_gates']=[dict(id=i,name=n,owners=o.split(),preserved=p,open_condition=q,status='승인 의미 유지 / 미정은 family-local gate') for i,n,o,p,q in PG]
TN=['기준선·출처','최소 작성/실행 지원','외부 다중 팩','런 세대 고정','공통 작성 포트','자원/수명','계산식·집계·성장','사건·결과','대상·전달·Flow','가역 상태·집중·재입력','보호막·방어·생존','사본·개화·형태','유물 관리','합성·활성','경로별 연쇄','소환·설치물','관계·감염·공간 집계','보스 제작','등록·획득·평가','침식','저장·결과·복원','대표 공식 RON/표현력','독립 data-only 인수','안전·결정론·성능','최종 독립 감사','상세 설계·근거 정합성','보스 action milestone','effect/carrier/source 경계','closure/연쇄 유한성','게시·저장·fault 경계']
TREC=[
('R00','원문/P2G/HEAD·dirty/old49edge/CA-M 표','모든 요구와 미정의 출처/owner가 있고 기존 완료·NEXT-0009 보존'),
('R01','실제 latest RON 최소 Damage/Heal/Instant+미지원 property','loader→compiler→executor 연결·descriptor 일치·기존 의미 유지'),
('R02','A/B pack, add/replace/remove, conflict/dependency cycle, invalid refs','재컴파일 없는 선택·명시 충돌/지원 불가 거부·성공 candidate만 게시'),
('R02','A런 유지 중 B게시·B실패·같은ID 다른내용 save','A불변·B새런만·fingerprint mismatch 거부'),
('R03','public API/CLI 동일입력·release target·reload failure·readonly query','동일 진단/실제 nonzero 실행·old test session 유지·state/RNG/revision 불변'),
('R05','같은 정의 두copy×deployment/attempt/run resource·비용 실패','잔량/사용기록 분리·owner별 cleanup·거부 원자성'),
('R06','단위 오류/overflow/0나눗셈·empty Max/Sum·weapon+SpellPower','typed거부·snapshot 일관·중복 성장 없음'),
('R04','basic vs skill·strike/segment·negative Heal·lethal pre-death','기존 basic trigger와 새 facts 분리·actual量 정확·필수 death/wave 유지'),
('R10','Sequence/Branch/Parallel/Repeat·result 생산전읽기·nohit·delayed settle','유한 continuation·분기/완료/취소·descriptor/executor 일치'),
('R11','Hello expiry/reinput·channel 입출범위·서로다른 modifier 해제','자기 lease만 정리·natural expiry Goodbye 없음·원형스탯 덮어복원 금지'),
('R12','공유600에감쇄후A400/B350·개인/공유/HP1/비용실패','공유 pool 한번생성·정책순서일치·HP/pool/receipt 원자성'),
('R07','A/B동일definition 중 A개화·이전·form선택·save','B오염 없음·동일copy 이중장착 거부·정상지급/저장'),
('R14','직원3slot·비호환 순서바꿈·다른직원·분쇄확인/취소·무기해제','독립copy·휴면slot유지·무료장착·판매금지·정상회수/기존장비가루'),
('R14 R15','동일 loadout permutation·numeric/structure recipe·새grant강화','개인 immutable plan의 즉시/지연실행 일치·effect선택/전용ID 보존'),
('R16','A1.e→B1.e→A1.e;A2;다른effect;독립root;sibling;delay;logeviction','경로별만 차단·동일ancestry보존·timer/root세탁없음'),
('R13','actor/trap/wall·small→big·비용/점유 실패·withdraw/TTL','서로다른capability·비용/ID 원자성·stale event 무효'),
('R17','붉은실/포자sourceA/B/기폭/타일grouping/진화삭제','원천집합 snapshot/소비·재감염/재발동 방지·유한 전파'),
('R08 R18','boss skill request/failure/cancel/facing→HP형태/fieldsignal/multiactor','실제task 실행·보스명특례없음·phase/wave/reward권한분리'),
('R06 R07 R14 R19','정상획득→선택→장착→전투·같은IDoverride·평가입력','각도입단계정상route·복수보상transaction·미정weight미승격'),
('R09','Pause/실제시전·현재적/미래wave·lastkill·optionalproc·hostclock','특수kill·정상진행·대가once·강제정지 mutation거부·Breakdown복원'),
('R02 R05 R07 R09 R14 R20','checkpoint/entry/attempt/retreat/retry/reconnect 복합','새state가도입부터유효·contentmismatch거부·resultonce'),
('R21','C01–C07 실제 latest 외부 공식RON+전체COV별도대조','builder/prototype만으로대체금지·미정/미지원명시'),
('R22','다른작성자·새ID·4도메인·crosspack·boss2skill+mechanism·data-onlydiff','엔진동결후실제재조합·추가enginecode필요시다시일반기능확장'),
('R02 R04 R10 R13 R16 R23','seed/input고정·컨테이너/pack순서·diag/logcaps·adversarialfanout','기능별초기예산+현실부하측정·typedfault·gameplaycap창작없음'),
('R24','독립reviewer·현재실행receipt·전체coverage/미정/허용외diff','남은필수누락없을때만완료;이보고서는대체아님'),
('R00 R01 R03 R21 R24','관찰/설계/실행구별·K별owner·소스/테스트/본문/JSON','권장순서를필수로과장하지않고모든mapping동일'),
('R04 R08 R18','t0admit/t100release/t300hit·stateexit50/150·stalehandle','Accepted≠Completed·waitmilestone·미방출cancel/방출후수명'),
('R01 R06 R07 R14 R15','AoE피해+1selfheal·같은SkillId개인plan·carrier속성변형','effect와ActionSite분리·sharedtarget/delivery보존·sourcecopy캐시격리'),
('R04 R15 R16 R17','positiveclosure/selfbootstrap/NOTcycle·fork/join/delayedancestry','compile유한성과runtime재방문분리·출처없는새root금지'),
('R02 R03 R05 R20 R23','candidate실패·testreload실패·mismatchsave·publicboundarybudgetfault','기존런/세션보존·미게시state/queue/RNG/ID/log복원·이전commit전체rollback주장없음')]
M['acceptance']=[{'id':f'T{i:02}','label':TN[i],'owners':o.split(),'input':inp,'expected':exp,'actual_status':'이번 게임 시험 미실행','note':'원문 T 식별자를 유지한 개정 owner/recipe; 문서 손계산은 runtime receipt가 아니다.'} for i,(o,inp,exp) in enumerate(TREC)]
C=[
('C01','따뜻한 마음의 나무꾼·깊은 쐐기','R04 R06 R07 R10 R14 R16','3타 순서/마지막 강화, 이번 n번째 직접 타격 보너스. segment 수·대상 수와 strike 수를 혼동하지 않는다.','n타 판정/지속기간/실제 수치 중 미정은 PG03/06'),
('C02','풍운승려 파편·Physical 염주 Staff·장비','R06 R07 R10 R14 R15','범위 bite 뒤 조건부 자기회복 1회. damage footprint만 변형하고 heal/source/횟수 유지. Staff≠Magic/SpellPower 자동분류.','유효 적중의 exact 의미 PG03; example HP 손계산은 test-only'),
('C03','아무 것도 없는 보스·Hello/Goodbye','R08 R10 R11 R18','보스 HP/actor identity 연속·변신 중 피해·임계치 skip. 플레이어 Hello는 기본공격 overlay, 재입력 Goodbye와 자연 종료 분리.','개화 공란/동시임계치 세부를 임의완성하지 않는다.'),
('C04','포큐버스 파편·공유 보호막·단일 전환','R05 R07 R12 R14 R15','한 원천 pool·membership·초기량 한번. 단일 recipient 일반2S recipe는 owner/수명 보존.','S공식/범위·개인공유 순서·침식 비교안 미정. 600/400/350은 감쇄후 test입력.'),
('C05','이름이 없는 태아 집중·단일 전환','R06 R11 R14 R15','regular basic damage/speed만 수혜자에 적용; 채널/lease 유지. 상태별 두property의 교체값을 무조건×2로 바꾸지 않는다.','최대시간·저지·침식 대가 미정은 gate'),
('C06','래티씨아 생성·진화·붉은 실타래','R07 R13 R15 R16 R17','내 source small→big, node진화/삭제/consume와 관계 invalidation, 접근/선통과 반응.','nodeID 유지/교체는 기술선택; 전체/유효node소비·linecrossing상세는 별도조건'),
('C07','포큐버스 보스·전장 작업 억제','R08 R11 R13 R18','잠복/출현·거리조건·Route·wave지원·작업완료→typed signal→억제. player sharedshield 사례로 대신하지 않는다.','거리/수치·작업/전이 정책의 미정만 gate')]
M['representative_cases']=[dict(id=i,name=n,owners=o.split(),requirement=r,condition=c,final_owner='R21',actual_status='이번 실제 RON 이식/게임 실행 미수행') for i,n,o,r,c in C]
# Review findings are the explicit bridge between observations and recommendations.
F=[]
def f(id,title,obs,decision,breakage,owners,src,limit='정적 본문 확인이며 실행 검증은 하지 않았다.'):
 F.append(dict(id=id,title=title,observation=obs,design_judgment=decision,broken_state=breakage,owners=owners.split(),sources=src.split(),limits=limit,classification='현행 관찰 + 목표 전환 설계; 이미 발생한 게임 버그라고 일괄 판정하지 않음'))
f('F01','현재 최소 Goal을 첫 구현으로 유지','Formal Freeze/Plan/Goal/Handoff는 효과 metadata와 literal Damage/Heal/Instant의 실제 연결을 한정하고, 새 사본/generation/save/전체 Flow를 제외한다. 현 executor와 validation/publication은 이미 존재한다.','권고 A. AG-CAMEC-01과 WP03은 재사용하며 미래 계약 전체를 필수 선행으로 더하지 않는다.','metadata 주소를 영구 copy나 runtime occurrence로 사용하거나 전체 후속 기능을 stub로 고정하면 이후 전환이 막힌다.','R01','S007 S008 S009 S010 S011 S065 S066 S092 S093')
f('F02','외부 팩은 기존 게시 경계를 확장하되 런 생성까지 연결','live_ron은 고정 embedded source. data/mod는 모든 검증과 generated setup 뒤 opaque token을 반환한다. GameCore가 Arc DB를 보유하며 PreparedApplicationCommit은 state/seed만 옮긴다.','R01 embedded 최소 slice는 독립 유효하다. R02는 외부 source·publication·briefing부터의 pin·content fingerprint reader/writer를 동시 전환한다.','draft에 B data만 대입하면 state-only commit 뒤 B선택을 잃을 수 있다. 같은ID다른내용 저장을 checksum만으로 허용하면 잘못된 정의로 복원한다.','R02','S012 S013 S014 S015 S037 S038 S039 S040 S085 S094')
f('F03','효과 metadata 도입은 현재 계산·공유 전달 의미를 보존','SkillStep은 target/delivery를 공유하고 effects가 순서대로 처리된다. ModifyDamage는 step의 Damage에 묶이고 signed Heal·HP-loss gate·repeat가 실행 경로에 존재한다.','R01은 주소/descriptor만 연결한다. R10/R15에서 effect와 carrier의 분리·변형을 명시하고 원형 영향을 시험한다.','피해 effect footprint 수정이 자기회복이나 공유 투사체 수·타격 인정까지 바꾸면 승인받지 않은 동작 변경이다.','R01 R10 R15','S016 S017 S018 S019 S024 S070 S097')
f('F04','개인별 합성은 지연 전달의 전역 SkillId 재조회와 함께 전환','projectile과 area 적중 경로는 전역 skill_data.get_by_id로 다시 정의를 읽는다. sim/queued step/preview/readiness에도 정의 기반 소비가 있다.','R14의 최소 numeric effective plan부터 cast·queued·projectile·area·개인 preview/evaluation input이 같은 immutable plan/binding을 소비하게 한다. 전역 catalog 설명 자체는 유지한다.','장착/Explain에는 강화가 보이지만 지연 적중은 원형으로 실행되거나 같은 SkillId의 두 직원 효과가 섞인다.','R14 R15 R16','S018 S019 S020 S021 S062 S063 S064 S091')
f('F05','영구 장비 사본 ID가 전투 source에 그대로 전달되지 않는다','inventory는 OwnedEquipment.instance_uuid를 가진다. player_spawns는 base UUID와 enhancement를 전달하고 build는 전투 owner/slot 기반 RuntimeItem ID를 만든다.','R04에서 실제 장비 copy→draft→runtime binding을 추가한다. 파편 copy binding은 R07이 같은 포트로 확장한다.','base UUID/슬롯을 copy ID로 간주하면 동일 장비 두 사본의 자원·출처·유물 재방문이 합쳐진다.','R04 R07 R14 R16','S034 S064 S022 S027')
f('F06','사건·결과와 기존 basic trigger는 서로 다른 계약','damage_runtime/commands는 HP commit과 결과 기록, triggers는 현재 occurrence별 기본공격 의미, death_runtime은 wave 기록 뒤 optional OnDeath/OnKill 등을 실행한다.','R04에서 request/fact/receipt를 구별하고 R09/R16이 suppression과 새 반응을 소비한다.','OnHit를 모든 Damage로 확대하거나 사망 경로 전체를 억제하면 기존 발동과 wave 진행이 바뀐다.','R04 R09 R12 R16','S023 S024 S025 S026 S028 S053')
f('F07','진단 로그는 새 인과/결과 authority의 저장소가 아니다','BattleEventLog는 개수/바이트/시간 제한의 diagnostic window다. recording은 accumulator 관찰을 append 앞에 두지만 sequence exhaustion에서는 앞서 반환한다.','새 ChainContext/receipt를 retention과 분리한다. log on/off·eviction뿐 아니라 sequence exhaustion 영향도 R04/R23에서 검사한다.','eviction으로 재방문 보호가 사라지거나 진단 출력 여부가 새 결과를 결정하면 재현성과 안전성이 깨진다.','R04 R16 R23','S071 S072 S027','정상 append/retention 경계는 본문 확인. 극단 sequence exhaustion의 전체 게임 영향·게임 테스트는 미검증이며 현재 안전하다고 인증하지 않는다.')
f('F08','파편 사본화는 종류별 장부와 모든 소비자의 동시 전환','SkillFragmentInventory와 loadout/progress는 definition ID를 중심으로 수량·개화·형태를 공유한다. grant/분쇄/profile/snapshot/checkpoint도 이를 읽는다. 장비에는 사본/양방향 loadout transaction이 있다.','R07에서 독립 OwnedFragment와 정상 grant/equip/awake/select/dismantle/death recovery/profile/save를 함께 바꾼다. 장비 장부는 재사용한다.','A사본만 개화해도 B가 바뀌거나 저장/분쇄/장착에서 copy identity를 잃는 중간 상태가 된다.','R07','S031 S032 S033 S034 S035 S036 S038 S041 S042 S064 S091')
f('F09','계산·성장·표시/평가 입력은 같은 owner를 가져야 한다','stat_pipeline이 여러 성장/장비 입력을 합친다. 현재 UnitStats에는 제안된 SpellPower가 없고 Certified는 active skill의 존재를 정액 평가한다.','R06에서 typed 식·스탯/무기성장·projection/평가 입력을 연결하고 R07에서 파편 rank를 제거한다. 새 Certified 가중 모델 완료는 R19/PG08이다.','무기 성장 이중합산, Magic=SpellPower 혼동, 원형 스킬 기준 평가를 개인 effective 평가 완료로 오인한다.','R06 R07 R19','S030 S043 S034 S062 S063 S064 S095')
f('F10','전체 작성 도구는 kernel 기능상 선행이 아니다','Cargo autobins=false이며 명시 target들은 test-fixtures 기반이다. 기존 skill_runtime_contract에는 4step·자기회복1회·buff스택반복3회를 assert하는 실제 kernel fixture가 있다.','R03을 먼저 권장하되 CA03→CA04를 기능 필수에서 권장/공개 인수 검증 관계로 바꾼다. 각 opcode는 descriptor/negative fixture를 함께 소유한다.','CLI가 없다는 이유로 resource/action kernel 자체를 불가능하다고 판정하거나 privatebuilder 성공만으로 외부 제작을 완료 선언한다.','R03 R04 R05','S068 S069 S070 S013 S085')
f('F11','보스 최소 명시 시전은 독립적으로 앞당길 수 있다','BossTaskData와 실제 runtime match에 명시 Skill invoke가 없다. 기존 계층 graph·task result/activation/cancel·bounded microstep은 존재한다. 일반 수동 스킬 경로는 Player 전용이다.','R08에서 raw/compile/action authority/result/facing을 동시에 확장한다. 기존 전달체 handle만 필요하고 전체 shield/Flow/trap은 후속이다.','Player manual API를 그대로 호출하거나 invoke_ability를 직접 호출해 eligibility를 우회하고 Accepted를 Completed로 오인한다.','R04 R08 R18','S047 S048 S018 S019 S022 S084 S094')
f('F12','SpawnKernel은 재사용 가능하나 상위 소환은 보스/적에 결속','SpawnKernel은 batch 배치·idempotent request·검증/원자성 경계를 가진다. summon_runtime은 Boss task와 Side::Opponent, route/actor behavior를 소비한다.','R13은 공통 하위 kernel을 재사용하고 actor/trap/fieldobject capability와 source/lifetime adapter를 분리한다.','아군 함정을 적 보스로 생성하거나 speed=0으로 actor validation을 우회하고 despawn을 kill로 세는 문제가 생긴다.','R13 R18','S049 S050 S051 S052 S053')
f('F13','전장 작업 완료 신호에는 raw→runtime의 동시 확장이 필요','field_work의 실제 완료 효과는 RepairProtectedUnit이며 작업 진행·assignment·완료소모를 소유한다. Wave는 actor death epoch와 binding으로 진행을 평가한다.','R18에서 typed completion signal을 schema/compile/executor/boss 구독까지 연결하고 진행 authority는 유지한다.','작업 완료로 boss 상태만 임의 바꾸거나 소환/보스가 wave/보상을 직접 승인하는 분할이 된다.','R18 R19','S052 S053 S047 S048 S084')
f('F14','침식의 kill 억제와 정상 전투 진행은 함께 보존해야 한다','최신 정책은 최종 보스 파편을 제외하고 실제 시전 시 현재 전장 적만 즉시 kill, 선택적 처치/사망 효과 억제, 필수 wave 진행을 요구한다. 이미 release된 projectile의 기존 수명은 유지한다.','R09는 큰 ordinary Damage가 아니라 special action·mandatory death/suppression·withdraw·result 비용을 같은 전환으로 묶는다. 전체 보스/chain 완성은 선행이 아니다.','OnDeath/OnKill 폭주 또는 미래 required wave 소실/정지, 연출 actor의 잘못된 보상/승리 관여가 생긴다.','R09 R16 R18','S046 S022 S025 S026 S053 S075')
f('F15','침식에는 기존 정책인 Breakdown countdown의 실제 구현이 필요','apply_incapacitation은 trauma와 함께 physical damage도 증가시킨다. 현재 condition/psychology.advance_node는 collapse countdown을 소유하지 않는다. 룰북은 countdown·회복폐기·node완료·save/snapshot·일반사망 회수를 규정한다.','R09에서 trauma100 대가/attempt→result와 새 countdown·노드 회복/사망·asset회수·save/projection을 닫는다. 침식에 기존 incap 함수는 그대로 사용하지 않는다.','침식 때문에 신체손상까지 늘거나 trauma100을 곧바로 사망으로 만들거나 시한부가 영구 정지한다.','R09 R20','S059 S060 S061 S040 S044 S046 S077 S091')
f('F16','Pause 중 cast admission과 침식의 실제 시전 시점은 구별','룰북은 Pause 중 일반 command의 resource/skill cast 시작을 즉시 반영하고 pending resolve는 정지한다. 최신 침식 정책은 Pause input만으로 침식을 시작하지 않으며 일반 command 전체 지연을 요구하지 않는다. sim에는 pending start_seq와 ManualCastEnd 경로가 있다.','R09 기술 후보는 admission을 보존하고 resume 후 첫 적격 battle advance에서 release 이전의 once-only cast-start 결정을 연결하는 것이다. focus 시작/종료·공명 소비 의미는 exact owner에서 확정한다.','UI 접수나 focus 종료를 임의로 실제 시전이라 정하면 발동 시점·자원/취소 의미가 바뀐다. 강제 연출은 모든 mutation/advance 경로를 함께 막아야 한다.','R09','S018 S044 S046 S077','시점의 기술 후보이며 승인된 hook 이름/정책이 아니다. P2G-20은 sim/manual admission·cast end·world/combat clock의 표를, 의미 충돌 시 P2G-10은 최소 질문을 소유한다. R01 선행 blocker는 아니다.')
f('F17','지급 원자성은 공통 Executor 자체가 아니라 바깥 commit 경계','EconomyGrantState가 inventory/fragment/roster/UUID/enkephalin roots를 복제하고 prepare/validate 후 commit한다. GrantExecutor의 순차 mutation 자체를 독립 all-or-nothing으로 볼 수 없다.','새 fragment/relic/resource를 관련 clone/validate/commit/preview/save roots에 포함시킨다.','여러 보상 중 후속 실패로 일부 자산·UUID만 남거나 preview가 실제 상태를 바꾼다.','R05 R07 R14 R19','S035 S036 S037 S038')
f('F18','복수 파편·선택은 등록뿐 아니라 보상 plan/commit/validation 변경','PveEncounterRewardSource는 단일 fragment_id와 source/encounter/fragment uniqueness를 소유한다. 첫 보상 plan은 그 하나를 GrantSkillFragment하고 완료 branch를 재검사하여 commit한다.','R19은 해당 schema와 planner/commit/validators, normal grant와 save/projection을 동시 전환한다. 기본 grant는 R07/R14부터 존재해야 한다.','새 정의만 추가하고 실제 normal reward가 선택을 만들지 못하거나 완료/재지급 분기가 엇갈린다.','R07 R14 R19','S073 S074 S042 S036 S067 S094')
f('F19','저장·복원은 최종 통합에서 처음 붙이는 기능이 아니다','durable checkpoint, battle-entry rollback, attempt roster, live-process reconnect의 상태 수명이 서로 다르다. 기존 A07/CA19도 새 상태 도입 시 저장을 동반하도록 요구한다.','각 R이 자기 상태의 저장/복원/결과를 소유하고 R20은 복합 증거를 닫는다. 기존 의도를 구체 owner/atomic group으로 강화한다.','후퇴/RNG/ID/직원 대가가 다음 단계 전까지 잘못되는 분할을 독립 완료로 볼 수 없다.','R02 R05 R07 R09 R14 R20','S037 S038 S039 S040 S044 S086')
f('F20','대표 RON과 전체 표현력 기준선은 다른 완료 주장','기존 CA00는 세 원문 및 공통정책의 요구/COV와 boss별 행을 만든 조사 기준선이다. A08은 제안 문법/시험값이며 actual fixture가 아니다.','R21은 C01–C07의 실제 외부 RON 실행과 전체 COV의 지원/미지원/정책대기 대조를 따로 제출한다. R22는 다른 작성자의 data-only holdout이다.','몇 개 이름 같은 RON 또는 privatebuilder 통과가 전체 독립 제작/표현력 완료로 둔갑한다.','R00 R21 R22 R24','S005 S006 S054 S057 S058 S078 S087 S096','이번에 354/127 모든 행과 각 콘텐츠 원문을 새로 연속 재검증하지 않았다. baseline 출처/조직을 인수하고 24CA owner를 새 R에 매핑했다.')
f('F21','인접 저장 계약 문구 불일치를 최소 Goal에 과잉 전가하지 않는다','data_loading_contract의 durable 단락은 Core payload V4를 서술하지만 actual active_run은 V5 writer와 V4/V5 reader다. 기존 P2G-50 검토도 이 불일치를 기록했다.','해당 저장/document owner의 인접 정합성 문제로 남긴다. 현재 최소 Goal은 save 비변경·index 비영구성이므로 그 자리에서 모든 reader 제거를 요구하지 않는다.','인접 오래된 문구만으로 새로운 호환 정책이나 전체 최소 Goal 재설계를 대신 결정하게 된다.','R01 R02 R20','S039 S065 S092 S094')
M['findings']=F
LIMITS=[
('L01','실제 게임/컴파일/회귀/성능 시험 미실행','모든 runtime 완료·GEX 자격','연결 도구는 read/search/status/history 전용이며 새 명령 실행 기능이 없다. 과거 PASS는 기존 증거로만 인수했다.','각 R의 owner가 승인된 구현 범위에서 actual nonzero 결과/exit/revision을 남긴다.'),
('L02','dirty 전체 snapshot/hash 미확보','조사 전체의 원자적 현재성','브랜치/HEAD/dirty, 읽은 구간·앵커와 일부 Git blob ID만 식별했다. 앵커 SHA256은 전체 파일 해시가 아니다.','적용/구현 인수 시 영향 경로 내용을 다시 비교한다. 새 변경이 있으면 해당 판단만 다시 검토한다.'),
('L03','354 COV/127 BOSS-COV의 fresh 행별 전수 재검증 아님','전체 개별 콘텐츠 표현력의 완료 주장','기존 CA00 기준선의 구조·출처·일부 head/footer 및 원문 주요 정책을 인수했다. 게임 기능별 실제 전체 row 증거는 없다.','S078 원본 requirements/boss coverage의 각 owner_ca를 M_CA_MAPPING에 연결해 유지하고 R21의 전체 표현력 대조에서 닫는다.'),
('L04','R17 정밀 line-crossing/geometry 알고리즘 미검토','선 통과 판정의 정밀도·성능·exact tie-break','관계/source/소비/수명 선행은 조사했으나 spatial/movement 전체 연속 본문과 geometry 시험까지 확장하지 않았다.','core/battle spatial 및 movement의 segment crossing 생산자/소비자와 R17의 object epoch/tie-break를 추가 확인한다.'),
('L05','미정 정책은 family-local 조건','침식 실제 시전 경계·합류/root·공유/개인 방어·공식 수치/평가','승인된 의미를 보존했지만 보고서가 threshold/방어순서/획득량/가중을 결정하지 않는다.','POLICY_GATES의 exact family/owner에서 확정. 독립 단위는 지정 조건부 분기로 계속 가능하다.'),
('L06','보스 상세 콘텐츠·도구형 9종은 baseline 인수','개별 content authored 완성도','보스 raw/compiler/runtime와 대표 경계는 확인했다. 3266줄 전체를 이번에 다시 읽거나 모든 공식 능력을 검증하지 않았다.','C01–C07 실제 이식과 별개 전체 boss-COV/도구 scope disposition을 유지. 도구 command/획득 신규범위는 사용자 owner.'),
('L07','원본 개별 CA markdown 카드 전량 독립 재독 아님','원본 카드에만 존재할 수 있는 추가 문구','원본 GOALS.json 946줄 전체, ROADMAP/DEP/K/PG/T/C 및 A01–A08을 읽었다. 목표/선행의 주 기준은 이 원본들이다.','원본 별도 카드와 JSON의 기존 불일치가 발견되면 그 행만 보존·수정 owner로 환류; 이를 현재 정식 정책으로 임의승격하지 않는다.'),
('L08','외부 consumer·독립 감사·업로드 미수행','서버/Unity/실제 배포·independent review','코어 측 projection/저장/소비 영향만 기록했다. 독립 reviewer나 제3자 작성자 참여가 없었다. 로컬 산출물이며 저장소 미반영이다.','별도 권한/범위의 후속 작업. 본 문서 검사를 GEX 독립 감사로 세지 않는다.'),
('L09','특정 오래된 문서/기존 소스의 극단 경계는 한정 확인','저장 V4 문구·log sequence exhaustion 전체 영향','구체 불일치와 해당 실제 body는 확인했으나 이를 현 세션에서 수정/회귀 검증하지 않았다.','F07/F21의 지정 owner와 입력으로 확인하고 무관한 최소 Goal까지 범위를 확대하지 않는다.')]
M['limitations']=[dict(id=i,limit=l,affected=a,basis=b,resume=r) for i,l,a,b,r in LIMITS]
P.write_text(json.dumps(M,ensure_ascii=False,indent=2)+'\n')
(ROOT/'report/FINDINGS.json').write_text(json.dumps(F,ensure_ascii=False,indent=2)+'\n')
print('contracts',len(K),'gates',len(PG),'acceptance',len(TREC),'cases',len(C),'findings',len(F),'limitations',len(LIMITS))
