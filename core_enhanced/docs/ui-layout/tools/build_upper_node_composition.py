from pathlib import Path
from fractions import Fraction as F
from itertools import product
from collections import defaultdict,Counter
from html import escape
import csv,json,hashlib
ROOT=Path(__file__).resolve().parent.parent
BASE='facility-upper-node-composition-v1'
source=ROOT/'node'/'data'/'facility-map-review-v2-data.json'
mapdata=json.loads(source.read_text())
rooms=sorted((n for n in mapdata['rooms'] if n['kind']=='explore' and 1<=n['depth']<=3),key=lambda n:(n['depth'],n['wing'],n['id']))
WINGS={'A':'기록동','B':'중앙본부동','C':'추출동'}
EVENT={6:{0:55,1:45},9:{0:30,1:60,2:10},12:{0:20,1:55,2:25}}
REWARD={6:{0:70,1:30},9:{0:45,1:50,2:5},12:{0:30,1:55,2:15}}
ELITE={(1,6):{0:100},(1,9):{0:100},(1,12):{0:100},(2,6):{0:85,1:15},(2,9):{0:75,1:25},(2,12):{0:65,1:35},(3,6):{0:75,1:25},(3,9):{0:65,1:35},(3,12):{0:50,1:50}}
SUPPORT={0:40,1:45,2:15}
RESERVED_SUPPORT={1:85,2:15}
SUPPORT_TYPES={'정비':30,'상점':45,'본사 연락':25}

def fnum(x):return float(x)
def fmt(x):return f'{float(x):.2f}'
def type_choices(fixed,total):
 needed=total-len(fixed)
 out=[]
 def rec(chosen,p):
  if len(chosen)==total:out.append((chosen,p));return
  eligible={k:v for k,v in SUPPORT_TYPES.items() if k not in chosen};weight=sum(eligible.values())
  for k,v in eligible.items():rec(chosen+[k],p*F(v,weight))
 rec(list(fixed),F(1))
 assert sum(p for _,p in out)==1
 assert all(len(c)==len(set(c))==total for c,_ in out)
 return out

results=[];exact={};all_cases=0
for n in rooms:
 cap=n['nodes'];depth=n['depth'];res=n['reserved'];id=n['id']
 encounter=1 if n['encounter'] else 0
 fixed_reward=int(res=='초반 유물 보상');save=int(res=='SavePoint')
 fixed_support=[res] if res in ('정비','본사 연락') else []
 fixed_non_support=encounter+fixed_reward+save
 min_normal=2+int(depth==3)
 support_dist=RESERVED_SUPPORT if fixed_support else SUPPORT
 cases=[];marginal=defaultdict(F);support_types=defaultdict(F)
 for s,sw in support_dist.items():
  free=cap-fixed_non_support-s
  admissible=[]
  for (e,ew),(r,rw),(l,lw) in product(EVENT[cap].items(),REWARD[cap].items(),ELITE[depth,cap].items()):
   normal=free-e-r-l
   if normal>=min_normal:admissible.append((e,r,l,normal,F(ew*rw*lw)))
  total_weight=sum(z[-1] for z in admissible)
  assert total_weight>0,(id,s)
  for e,r,l,g,w in admissible:
   p=F(sw,100)*w/total_weight
   cases.append({'support':s,'event':e,'reward_extra':r,'elite':l,'normal':g,'probability':p})
   marginal['support']+=s*p;marginal['event']+=e*p;marginal['reward_extra']+=r*p;marginal['elite']+=l*p;marginal['normal']+=g*p
   marginal['event_present']+=int(e>0)*p;marginal['reward_present']+=int(r+fixed_reward>0)*p;marginal['elite_present']+=int(l>0)*p
  for kinds,p in type_choices(fixed_support,s):
   for kind in kinds:support_types[kind]+=F(sw,100)*p
 assert sum(c['probability'] for c in cases)==1
 for s,sw in support_dist.items():assert sum(c['probability'] for c in cases if c['support']==s)==F(sw,100)
 for c in cases:
  assert fixed_non_support+c['support']+c['event']+c['reward_extra']+c['elite']+c['normal']==cap
  assert c['normal']-int(depth==3)>=2
  if id=='A3-02':
   # If the prerequisite is unmet, the conditional encounter slot remains an ordinary corrosion encounter in this proposal.
   assert (fixed_non_support-1)+c['support']+c['event']+c['reward_extra']+c['elite']+(c['normal']+1)==cap
 assert sum(support_types.values())==marginal['support']
 fixed_count=fixed_non_support+len(fixed_support)
 if n['encounter']:role='조건부 조우 갈래' if id=='A3-02' else ('선행 조우 갈래' if n.get('encounter_kind','').startswith('선행') else '일반 보스 선택 갈래')
 elif res=='SavePoint':role='저장·회복 거점'
 elif res=='정비':role='성장 적용 거점'
 elif res=='본사 연락':role='초반 채용 거점'
 elif res=='초반 유물 보상':role='초반 합류·유물 기회'
 elif '연결' in n['name'] or id.endswith('-01'):role='연결·진입'
 else:role='선택 탐사 갈래'
 reserved=[]
 if n['encounter']:reserved.append(n['encounter']+' 1'+(' (조건부)' if id=='A3-02' else ''))
 if res:reserved.append(('단순 선택 유물 보상' if res=='초반 유물 보상' else res)+' 1')
 normal_min=min(c['normal'] for c in cases);normal_max=max(c['normal'] for c in cases)
 results.append({'id':id,'wing':n['wing'],'depth':depth,'name':n['name'],'role':role,'activity_nodes':cap,'reservations':reserved,'reserved_slots':fixed_count,'remaining_assignment_slots':cap-fixed_count,'omen_compatible_slots':int(depth==3),'fixed_support':fixed_support,'fixed_encounters':encounter,'fixed_reward':fixed_reward,'fixed_save':save,'normal_without_omen':{'min':normal_min,'max':normal_max,'mean':float(marginal['normal'])},'normal_with_omen':{'min':normal_min-1,'max':normal_max-1,'mean':float(marginal['normal']-1)} if depth==3 else None,'wolf_inactive_normal_delta':int(id=='A3-02'),'expected':{k:float(v) for k,v in marginal.items()},'expected_support_types':{k:float(support_types[k]) for k in SUPPORT_TYPES},'probability_sum':1.0,'candidate_count':len(cases),'probability_cases':[{**c,'probability':float(c['probability']),'probability_exact':str(c['probability'])} for c in cases]})
 exact[id]=marginal
 all_cases+=len(cases)

assert len(results)==33 and sum(r['activity_nodes'] for r in results)==258
assert sum(r['omen_compatible_slots'] for r in results)==12
assert sum(r['fixed_save'] for r in results)==3
assert sum(r['reserved_slots'] for r in results)==12
by_floor=[]
for depth in [1,2,3]:
 subset=[r for r in results if r['depth']==depth]
 avg=sum(exact[r['id']]['normal'] for r in subset)
 by_floor.append({'depth':depth,'rooms':len(subset),'slots':sum(r['activity_nodes'] for r in subset),'reserved_slots':sum(r['reserved_slots'] for r in subset),'assignment_slots':sum(r['remaining_assignment_slots'] for r in subset),'normal_mean_no_omen':float(avg),'normal_mean_max_omens':float(avg-(3 if depth==3 else 0)),'normal_min_sum_no_omen':sum(r['normal_without_omen']['min'] for r in subset),'normal_max_sum_no_omen':sum(r['normal_without_omen']['max'] for r in subset)})
totals={k:sum(exact[r['id']][k] for r in results) for k in ['normal','support','event','reward_extra','elite']}
assert totals['normal']+totals['support']+totals['event']+totals['reward_extra']+totals['elite']+8==258
agg={'rooms':33,'activity_slots':258,'reserved_slots':12,'remaining_assignment_slots':246,'normal_mean_no_omen_wolf_active':float(totals['normal']),'normal_mean_three_omens_wolf_active':float(totals['normal']-3),'normal_mean_no_omen_wolf_inactive':float(totals['normal']+1),'normal_mean_three_omens_wolf_inactive':float(totals['normal']-2),'expected_support':float(totals['support']),'expected_event':float(totals['event']),'expected_reward_including_fixed':float(totals['reward_extra']+1),'expected_elite':float(totals['elite']),'fixed_save':3,'fixed_general_family_encounters_wolf_active':4,'global_omen_max':3,'omen_compatible_rooms':12,'enumerated_compositions':all_cases}
meta={'status':'review_proposal_not_canonical','date':'2026-10-07','scope':'map v2 depths 1-3 only; whole upper-map capacity, not one-expedition visits','source_map':'facility-map-review-v2-data.json','supported_map_room_ids':[r['id'] for r in results],'policy_basis':['../game_rules/run-and-map.md#노드-종류-배정','../game_rules/run-and-map.md#방향별-징조와-퍼즐-발현','../game_rules/relic-reward-nodes.md','../skills/abnormality_design_notes.ko.md#붉은-두건의-용병'],'new_proposals':['Event/reward/elite count weights by size/depth','fixed-support rooms: 1 total 85%,2 total 15%','support type weights Maintenance30 Shop45 HQ25 without replacement','minimum 2 ordinary corrosion nodes after optional omen','one generic compatible corrosion slot in each depth3 room; maxone omen perwing','unmet wolf prerequisite: conditional encounter slot remains one ordinary corrosion encounter','capacity-safe event/reward/elite combinations normalized within each already-drawn support count'],'existing_initial_policy':{'support_count_distribution':SUPPORT,'ordinary_difficulty_percent':{'Easy':20,'Standard':50,'Hard':30}},'proposal_parameters':{'event_count_weights':EVENT,'extra_reward_count_weights':REWARD,'elite_count_weights':{f'{d}F_{s}slots':v for (d,s),v in ELITE.items()},'reserved_support_count_percent':RESERVED_SUPPORT,'support_type_weights':SUPPORT_TYPES,'min_normal_after_omen':2},'aggregate':agg,'by_floor':by_floor,'rooms':results}
(ROOT / 'node' / 'data' / f'{BASE}.json').write_text(json.dumps(meta,ensure_ascii=False,indent=2),encoding='utf-8')
with (ROOT / 'node' / 'data' / f'{BASE}.csv').open('w',encoding='utf-8-sig',newline='') as f:
 w=csv.writer(f);w.writerow(['방ID','동','층','방이름','역할','전체활동수','고정예약','고정예약수','일반배정자리','전조후보자리_총량아님','일반전최소_전조없음','일반전최대_전조없음','일반전평균_전조없음','일반전평균_이방이전조대상','늑대미발생시일반전추가','이벤트평균','보상평균_예약포함','엘리트평균','지원합계평균_예약포함','정비평균','상점평균','본사연락평균','이벤트유효발현률','보상유효발현률','엘리트유효발현률'])
 for r in results:
  g=r['normal_without_omen'];ex=r['expected'];st=r['expected_support_types']
  w.writerow([r['id'],WINGS[r['wing']],r['depth'],r['name'],r['role'],r['activity_nodes'],' / '.join(r['reservations']) or '없음',r['reserved_slots'],r['remaining_assignment_slots'],r['omen_compatible_slots'],g['min'],g['max'],fmt(g['mean']),fmt(r['normal_with_omen']['mean']) if r['normal_with_omen'] else '',r['wolf_inactive_normal_delta'],fmt(ex['event']),fmt(ex['reward_extra']+r['fixed_reward']),fmt(ex['elite']),fmt(ex['support']),*[fmt(st[t]) for t in SUPPORT_TYPES],*[fmt(100*ex[k])+'%' for k in ['event_present','reward_present','elite_present']]])

md=['# 공용 구간 방별 노드 구성 검토안','',f'기록일: 2026-10-07. 대상은 [시설 지도 시안 02](facility-map-review-v2.html)의 1~3층 33개 방이다. 총 258개 활동 자리에 예약과 배정 초안을 적용하면, 늑대 조우가 발생하고 3층 전조가 세 계열에 모두 배정되는 조건에서 일반 침식전은 평균 {fmt(totals["normal"]-3)}개다. 이 수량은 공용 구간 전체의 배치 용량이며 한 원정의 방문·전투 횟수가 아니다.','',
'| 구분 | 내용 |','| --- | --- |','| 성격 | 전장 수요 산정을 위한 검토안. 확정 게임 정책이나 구현 완료 판정으로 사용하지 않는다 |','| 유지한 지도 | 방 33개, 위치와 연결, 방별 활동 수 6·9·12, 고정 조우 위치 후보 |','| 새 검토값 | 활동 개수 가중치, 고정 지원 방의 분포, 지원 종류 가중치, 최소 일반전 2개, 미발생 늑대 자리의 처리 |','| 계산 산출물 | [구성표 CSV](facility-upper-node-composition-v1.csv), [확률과 모든 유효 구성 JSON](facility-upper-node-composition-v1.json), [구성표 화면](facility-upper-node-composition-v1.html) |','',
'## 적용 범위와 수량을 세는 기준','',
'시작과 Gate는 활동 수에서 제외한다. 고정 조우·초반 유물 보상·정비·본사 연락·SavePoint는 방의 활동 수에 포함한다. 정비·상점·본사 연락은 합계 0~2개의 지원 묶음이고, 고정 지원도 이 상한에 포함한다. SavePoint와 유물 보상은 지원 상한에 포함하지 않는다. 같은 본사 연락의 상점 기능을 별도 노드로 더하지 않는다. [기존 배정 규칙](../game_rules/run-and-map.md#노드-종류-배정)','',
'이 시안의 고정 예약은 용병·여왕·기사·조건부 늑대 조우 4개, 정비 3개, SavePoint 3개, 본사 연락 1개, 초반 유물 보상 1개다. 총 12자리를 먼저 잡고 나머지 246자리를 일반 배정 대상으로 둔다. 위치는 지도 시안의 후보를 사용한다.','',
'용병의 선행 Event/Story는 최초 전투가 아니다. 여왕·기사도 각각 동행·계약의 시작점으로 세며 후속 전투를 여기서 추가 노드로 중복 집계하지 않는다. 늑대는 이번 원정의 용병 선행 완료 후에만 발생한다. 호출권 구매는 발생 조건이 아니다. 이 구성안에서는 선행 미완료 시 조건부 늑대 자리 하나를 일반 침식전으로 유지하는 방안을 제안한다. 해당 자리의 미발생 처리 방식은 새 검토값이며 기존 선행 규칙 자체와 구분한다. [용병과 늑대](../skills/abnormality_design_notes.ko.md#붉은-두건의-용병)','',
'절망의 기사 계약은 중앙본부동 B2-02 배전관리실에 둔다. 계약 후 서로 다른 전투 10회 승리를 시도할 여유를 확보하고, 계약은 이후 어느 동을 탐사해도 원정 전체에서 유지한다. 파고드는 천국은 추출동 C4-02 배관점검실의 선택 조우다. 4층 정비 후 도전할 여유와 추출동 독립 구간의 황금가지 후보 위치를 확보한다. 실제 황금가지 유무는 사전 작성 배치안에 따른다. 추출동 C3-02와 기록동 A4-05는 일반 탐사 방으로 사용한다. 이 배치로 각 동의 일반 보스 계열 고정 조우는 4개씩이다. 최종보스와 비전투 선행 조우를 구분하며, 실제 보스전 횟수가 동마다 같다는 뜻은 아니다.','','## 전조를 받을 자리와 실제 전조 수','',
'3층의 12개 방마다 일반 침식전 후보 자리 하나를 전조로 바꿀 수 있게 준비한다. 실제로는 각 계열에서 처음 진입한 방에 적격 미완료 전조가 있으면 하나를 배정하고, 그 계열의 같은 층 다른 방에는 추가 배정하지 않는다. 따라서 후보 자리는 12개지만 실제 전조는 3층 전체에서 최대 3개다. 그 방의 일반전 수에서만 1개를 빼며, 정비·저장·고정 조우를 덮어쓰지 않는다. 전조가 없으면 그 자리는 일반 침식전으로 남는다. [전조 배정](../game_rules/run-and-map.md#방향별-징조와-퍼즐-발현)','',
'3층 진입 방이 원정의 이동 경로에 따라 달라지므로 표에는 전조 없는 경우와 해당 방이 전조 대상인 경우를 함께 적었다. 전조 대상 열을 12개 방 모두에 적용해 합산하지 않는다.','',
'## 원정별 활동 배정 초안','',
'일반 방의 지원 합계는 기존 초기값인 0개 40%·1개 45%·2개 15%를 유지한다. 고정 정비 또는 본사 연락이 있는 방은 총 1개 85%·2개 15%를 새 검토값으로 둔다. 이는 예약 1개에 추가 지원을 15%로 붙이는 방식이며, 예약 지원을 두 번 세지 않는다. 초반 확정 접근 기회를 유지하면서 추가 편의 노드가 매번 붙지는 않도록 한 출발점이다.','',
'지원 종류의 새 가중치는 정비 30·상점 45·본사 연락 25다. 예약된 종류를 제외하고 가중치에 따라 중복 없이 선택한다. 이는 개별 노드의 최종 발현률이 아니다. 예를 들어 지원이 두 개인 방의 두 번째 종류는 남은 후보의 가중치를 다시 나눈다. 초기 채용 기회를 보완할 본사 연락은 남기되, 구매와 구성 보완 역할의 상점에 조금 더 비중을 둔다.','',
'다음 표는 남은 자리에 배정할 활동 개수의 **원가중치**다. 최소 일반전 수를 지킬 수 있는 조합 안에서 정규화하므로, 실제 방별 발현률은 CSV와 JSON의 유효 발현률을 따른다. 보상 수는 초반 예약 보상 외에 추가되는 수다.','',
'| 활동 자리 | 이벤트 0개 / 1개 / 2개 | 추가 보상 0개 / 1개 / 2개 |','| --- | --- | --- |','| 6개 방 | 55 / 45 / 0 | 70 / 30 / 0 |','| 9개 방 | 30 / 60 / 10 | 45 / 50 / 5 |','| 12개 방 | 20 / 55 / 25 | 30 / 55 / 15 |','',
'작은 방은 짧게 통과할 여지를 두고, 큰 방에는 일반전 외의 선택 활동도 더 넣는다. 이벤트와 보상을 모든 방에 보장하지 않는다.','',
'| 깊이 | 6개 방 엘리트 0개 / 1개 | 9개 방 엘리트 0개 / 1개 | 12개 방 엘리트 0개 / 1개 |','| --- | --- | --- |','| 1층 | 100 / 0 | 100 / 0 | 100 / 0 |','| 2층 | 85 / 15 | 75 / 25 | 65 / 35 |','| 3층 | 75 / 25 | 65 / 35 | 50 / 50 |','',
'초기 3명 편성과 기본 장비를 정리하는 1층은 엘리트를 배정하지 않는 초안이다. 2층부터 선택형 고위험 조우를 넣고 3층에서 조금 늘린다. 깊이별 수치와 상한은 확정 정책이 아니다.','',
'후보 구성은 다음 순서로 계산한다.','',
'1. 고정 예약과 조건부 늑대 자리의 예산을 먼저 확보한다.','2. 지원 총수를 위 분포로 정하고, 예약을 뺀 추가 종류를 선택한다.','3. 이벤트·추가 보상·엘리트 개수 조합 중 일반전이 최소 2개 남는 구성을 후보로 작성한다. 3층은 전조 치환 뒤에도 2개가 남도록 치환 전 최소 3개를 확보한다.','4. 유효 후보 안에서 세 활동의 원가중치 곱으로 추첨한다. 지원 총수의 확률은 바꾸지 않는다.','5. 남는 자리를 일반 침식전으로 둔다. 전조가 배정된 방만 호환 자리 1개를 치환한다.','6. 마지막 일반전 수에 기존 Easy 20%·Standard 50%·Hard 30% 초기 비율을 적용한다. 재방문이나 불러오기로 배정을 다시 추첨하지 않는다.','',
'자리 부족이 생긴 뒤 예약 노드를 삭제하는 방식은 사용하지 않는다. 유효한 개수 조합을 사전에 작성하며, 어느 지원 총수에서도 후보가 없다면 구성 오류로 본다. 이번 33개 방은 모든 지원 총수에서 후보가 존재한다.','',
'## 층별 계산 결과','',
'아래 기대값은 조건부 늑대가 발생하는 경우다. 모든 방에 가능한 추첨을 정확히 열거한 평균이며 플레이 결과나 원정당 전투 수가 아니다.','',
'| 층 | 방 수 | 전체 활동 | 고정 예약 | 일반 배정 자리 | 일반전 평균 전조 없음 | 일반전 평균 전조 최대 배정 |','| --- | ---: | ---: | ---: | ---: | ---: | ---: |']
for f in by_floor:md.append(f'| {f["depth"]}층 | {f["rooms"]} | {f["slots"]} | {f["reserved_slots"]} | {f["assignment_slots"]} | {fmt(f["normal_mean_no_omen"])} | {fmt(f["normal_mean_max_omens"])} |')
md += [f'| 합계 | 33 | 258 | 12 | 246 | {fmt(totals["normal"])} | {fmt(totals["normal"]-3)} |','',f'늑대가 발생하지 않으면 위 일반전 합계에 1개를 더한다. 전조가 실제로 k개 배정되면 일반전 기대값은 **{fmt(totals["normal"])} − k + 늑대 미발생 시 1**이다. k는 0~3이며 각 계열의 방문·적격 후보 조건을 따른다.','',
'| 활동 | 공용 구간 전체 기대 수량 |','| --- | ---: |',f'| 일반 침식전 전조 최대 및 늑대 발생 | {fmt(totals["normal"]-3)} |',f'| 일반 이벤트 | {fmt(totals["event"])} |',f'| 보상 예약 1개 포함 | {fmt(totals["reward_extra"]+1)} |',f'| 엘리트 | {fmt(totals["elite"])} |',f'| 지원 정비·상점·본사 연락 예약 포함 | {fmt(totals["support"])} |','| SavePoint | 3 |','| 일반 보스 계열·선행 조우 늑대 포함 | 4 |','| 전조 세 계열 모두 배정되는 경우 | 3 |','',
'## 방별 구성표','',
'일반전 범위는 그 방에서 가능한 최소·최대이며 95% 구간이 아니다. 평균은 여러 원정의 배정 기대값이다. 늑대 방은 선행 완료 상태를 기준으로 표시하며, 미완료 시 일반전을 1개 더한다.','',
'| 방 ID와 이름 | 역할 | 전체 | 예약 | 남은 배정 자리 | 일반전 범위 및 평균 전조 없음 | 이 방이 전조 대상일 때 평균 |','| --- | --- | ---: | --- | ---: | --- | ---: |']
for r in results:
 g=r['normal_without_omen'];o=r['normal_with_omen'];reserved=' / '.join(r['reservations']) or '없음'
 md.append(f'| {r["id"]} {r["name"]} | {r["role"]} | {r["activity_nodes"]} | {reserved} | {r["remaining_assignment_slots"]} | {g["min"]}~{g["max"]} / {fmt(g["mean"])} | {fmt(o["mean"]) if o else "—"} |')
md += ['',
'## 전장 수요로 이어지는 다음 판단','',
'258개의 활동 자리가 모두 일반전은 아니며, 배정된 일반전을 플레이어가 모두 방문하는 것도 아니다. 방 내부의 빠른 통과 경로와 추가 탐사 갈래를 그린 뒤 실제 선택 전투량을 추정한다. 그다음 여러 방에서 같은 전장 도면을 공유할 범위와 원정 안의 반복 허용 정도를 정해 고유 전장 수를 산출한다.','',
'일반 이벤트의 전투 분기, 엘리트·전조 전투, 여왕·기사의 후속 전투, 유물 보상의 스트레스 전투는 위 일반 침식전 수에 포함하지 않았다. 이들은 별도 전장 수요 또는 공용 전장 재사용 후보로 검토해야 한다. 특히 유물 보상은 현재 단순 선택·자원 투입·도박·유물 투입 가챠·스트레스 전투의 다섯 유형이며, 이 안은 유형별 비중을 새로 정하지 않는다. 초반 B1-01 예약만 단순 선택형을 유지한다. [유물 보상 규칙](../game_rules/relic-reward-nodes.md)','',
'정비·저장으로 접근하는 실제 활동 수와 억제력 소모, 초기 채용 비용을 감당할 재화, 각 일반전의 길이와 보상은 방 내부 연결을 그릴 때 함께 확인한다. 지원 종류가 두 개 있어도 두 번 모두 이용해야 하는 구성으로 만들지는 않는다.','']
md_text='\n'.join(md).replace('](facility-', '](../node/html/facility-').replace('](../game_rules/', '](../../game_rules/').replace('](../skills/', '](../../skills/')
md_text=md_text.replace('../node/html/facility-upper-node-composition-v1.csv','../node/data/facility-upper-node-composition-v1.csv').replace('../node/html/facility-upper-node-composition-v1.json','../node/data/facility-upper-node-composition-v1.json')
(ROOT/'md'/f'{BASE}.ko.md').write_text(md_text,encoding='utf-8')

# A compact, offline review table with scenario controls. The underlying values are generated above.
brief={k:v for k,v in meta.items() if k not in ['rooms']};brief['rooms']=[{k:v for k,v in r.items() if k!='probability_cases'} for r in results]
html='''<!doctype html><html lang="ko"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>공용 구간 방별 노드 구성 검토안</title><style>*{box-sizing:border-box}body{margin:0;background:#f5f7fb;color:#26364c;font:15px/1.6 'Malgun Gothic',sans-serif}header,main{max-width:1450px;margin:auto;padding:24px}header{padding-bottom:8px}h1{margin:0;font-size:28px}p{margin:8px 0;color:#56667b}.links{display:flex;gap:10px;flex-wrap:wrap;margin:16px 0}a{color:#265e91}.links a,button,select{background:white;border:1px solid #cbd6e5;border-radius:7px;padding:8px 12px;text-decoration:none;color:#28476a;font:inherit}.cards{display:grid;grid-template-columns:repeat(4,1fr);gap:14px}.card{padding:20px;background:white;border:1px solid #dce4ee;border-radius:12px}.card b{display:block;font-size:28px;color:#234e7a}.card span{color:#65758a;font-size:13px}.controls{display:flex;flex-wrap:wrap;gap:18px;padding:18px;background:#e8eff7;border-radius:10px;margin:20px 0}label{display:flex;align-items:center;gap:8px}.note{background:#fff7e6;border-left:4px solid #c18b2e;padding:14px 18px;margin:18px 0}.scroll{overflow:auto;border:1px solid #d4dfec;border-radius:10px;background:#fff}table{border-collapse:collapse;width:100%;min-width:1200px}th,td{padding:11px 12px;text-align:left;border-bottom:1px solid #e4eaf1;vertical-align:top}th{background:#eaf0f7;color:#344c69;white-space:nowrap;position:sticky;top:0}tbody tr:hover{background:#f2f7fc}.id{font-weight:bold;white-space:nowrap}.muted{color:#758296;font-size:12px}.num{text-align:right;white-space:nowrap}h2{font-size:20px;margin:28px 0 12px}.summary{min-width:700px}small{color:#677890}@media(max-width:800px){.cards{grid-template-columns:repeat(2,1fr)}header,main{padding:16px}h1{font-size:23px}}</style>
<header><h1>1~3층 공용 구간 노드 구성 검토안</h1><p>33개 방 · 활동 자리 258개. 방 구조와 수량을 유지하며 고정 예약과 원정별 배정 초안을 계산했습니다.</p><div class="links"><a href="facility-map-review-v2.html">시설 지도</a><a href="facility-lower-node-composition-v1.html">4층 이후 구성표</a><a href="../data/facility-upper-node-composition-v1.csv" download>방별 CSV</a><a href="../../md/facility-upper-node-composition-v1.ko.md">계산 기준 문서</a></div></header><main><div class="cards"><div class="card"><b>258</b><span>공용 구간 전체 활동 자리</span></div><div class="card"><b>12</b><span>고정·조건부 예약 예산</span></div><div class="card"><b>246</b><span>예약 후 일반 배정 자리</span></div><div class="card"><b id="mean"></b><span>전체 33개 방의 일반 침식전 기대 수량</span></div></div>
<div class="controls"><label>3층 실제 전조 수 <select id="omens"><option>0</option><option>1</option><option>2</option><option selected>3</option></select></label><label><input id="wolf" type="checkbox" checked>이번 원정 용병 선행 완료로 늑대 발생</label></div>
<div class="note">전체 구간의 배치 용량이며 한 원정의 전투 횟수가 아닙니다. 3층의 전조 후보 방은 12개지만 실제 배정은 계열별 첫 진입 방 하나씩, 최대 3개입니다. 아래 ‘전조 대상’ 열을 모든 방에 합산하지 않습니다.</div>
<h2>층별 배치 용량</h2><div class="scroll"><table class="summary"><thead><tr><th>층</th><th>방 수</th><th>전체 활동</th><th>예약</th><th>배정 자리</th><th>일반 침식전 기대 수량</th></tr></thead><tbody id="floors"></tbody></table></div>
<h2>방별 구성</h2><div class="controls"><label>층 <select id="floor"><option value="all">전체</option><option value="1">1층</option><option value="2">2층</option><option value="3">3층</option></select></label><label>동 <select id="wing"><option value="all">전체</option><option value="A">기록동</option><option value="B">중앙본부동</option><option value="C">추출동</option></select></label><span id="shown"></span></div>
<div class="scroll"><table><thead><tr><th>방 ID / 이름</th><th>역할</th><th>활동 수</th><th>예약 활동</th><th>남은 자리</th><th>일반전 범위<br>전조 없음</th><th>일반전 평균<br>전조 없음</th><th>일반전 평균<br>이 방이 전조 대상</th><th>이벤트 / 보상 / 엘리트<br>각 기대 수량</th><th>지원 합계<br>예약 포함</th></tr></thead><tbody id="rows"></tbody></table></div>
<p><small>수치와 새 배정 가중치는 검토값입니다. 시작·Gate 제외, 예약·고정 조우 포함. 범위는 가능한 최소·최대입니다. 보상 평균은 예약 보상을 포함하며, 전투형 보상·이벤트의 전투 분기는 일반 침식전 합계에 포함하지 않습니다.</small></p>
<h2>이번 구성안의 기준</h2><p>일반 지원 묶음은 기존 0개 40% · 1개 45% · 2개 15%를 유지합니다. 예약 지원 방은 총 1개 85% · 2개 15%를 제안합니다. 이벤트·보상·엘리트는 방 규모와 깊이에 따른 후보 가중치로 배정하되, 전조 치환 후에도 일반전 2개가 남는 구성만 후보로 둡니다.</p><p>늑대 선행 미완료 시 그 조건부 자리 하나를 일반 침식전으로 유지하는 것은 이번 초안의 제안입니다. 늑대 조우가 선행 없이 발생하지 않는 기존 규칙은 그대로 따릅니다.</p></main>
<script>const DATA=__DATA__;const $=s=>document.querySelector(s),f=x=>x.toFixed(2);function cell(row,t,cls=''){const x=document.createElement('td');x.textContent=t;x.className=cls;row.append(x)}function render(){const k=Number($('#omens').value),inactive=!$('#wolf').checked,mean=DATA.aggregate.normal_mean_no_omen_wolf_active-k+Number(inactive);$('#mean').textContent=f(mean);const fs=$('#floors');fs.replaceChildren();for(const a of DATA.by_floor){const tr=document.createElement('tr');[a.depth+'층',a.rooms,a.slots,a.reserved_slots,a.assignment_slots,f(a.normal_mean_no_omen-(a.depth===3?k:0)+(a.depth===3&&inactive?1:0))].forEach(v=>cell(tr,v));fs.append(tr)}const filter=DATA.rooms.filter(r=>($('#floor').value==='all'||r.depth===Number($('#floor').value))&&($('#wing').value==='all'||r.wing===$('#wing').value));$('#shown').textContent=filter.length+'개 방 표시 / 합계 카드는 전체 33개 방 기준';const host=$('#rows');host.replaceChildren();for(const r of filter){const tr=document.createElement('tr'),delta=inactive?r.wolf_inactive_normal_delta:0,g=r.normal_without_omen,e=r.expected;tr.dataset.room=r.id;cell(tr,r.id+' · '+r.name,'id');cell(tr,r.role);cell(tr,r.activity_nodes,'num');cell(tr,r.reservations.join(' / ')||'없음');cell(tr,r.remaining_assignment_slots,'num');cell(tr,(g.min+delta)+'~'+(g.max+delta),'num');cell(tr,f(g.mean+delta),'num');cell(tr,r.normal_with_omen?f(r.normal_with_omen.mean+delta):'—','num');cell(tr,[e.event,e.reward_extra+r.fixed_reward,e.elite].map(f).join(' / '),'num');cell(tr,f(e.support),'num');host.append(tr)}}['omens','wolf','floor','wing'].forEach(id=>$('#'+id).addEventListener('change',render));render();</script></html>'''
(ROOT / 'node' / 'html' / f'{BASE}.html').write_text(html.replace('__DATA__',json.dumps(brief,ensure_ascii=False).replace('</','<\\/')),encoding='utf-8')
print(json.dumps({'aggregate':agg,'by_floor':by_floor},ensure_ascii=False,indent=2))

# Refresh the existing map inspector from the same calculated composition.
map_file=ROOT / 'node' / 'data' / 'facility-map-review-v2-data.json'
map_current=json.loads(map_file.read_text())
by_id={r['id']:r for r in map_current['rooms']}
for c in results:
 assert by_id[c['id']]['nodes']==c['activity_nodes']
 by_id[c['id']]['upper_composition']={k:v for k,v in c.items() if k not in ('probability_cases','expected_support_types')}
viewer=ROOT / 'node' / 'html' / 'facility-map-review-v2.html'
viewer_text=viewer.read_text()
start=viewer_text.index('const data=')+len('const data=')
end=viewer_text.index(';const rooms=',start)
viewer_text=viewer_text[:start]+json.dumps(map_current,ensure_ascii=False).replace('</','<\\/')+viewer_text[end:]
map_file.write_text(json.dumps(map_current,ensure_ascii=False,indent=2),encoding='utf-8')
viewer.write_text(viewer_text,encoding='utf-8')
