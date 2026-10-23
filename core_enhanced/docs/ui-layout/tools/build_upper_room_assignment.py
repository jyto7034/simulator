"""Assign room sizes, route templates and protected slots to the 33 upper rooms.

Creates a separate review layer over the facility graph. The source map and old
composition tables remain reproducible comparison baselines. No runtime is edited.
"""
from collections import Counter, deque
from copy import deepcopy
from itertools import product
from pathlib import Path
import csv
import heapq
import json
import xml.etree.ElementTree as ET

import build_room_activity_allocation as allocation
import node_diagram_art as art

ROOT=Path(__file__).resolve().parent.parent
NODE=ROOT/'node'
BASE='facility-upper-room-assignment-v1'
SVG_NS='http://www.w3.org/2000/svg'
ET.register_namespace('',SVG_NS)
TAG=lambda name:'{'+SVG_NS+'}'+name
WINGS={'A':'기록동','B':'중앙본부동','C':'추출동'}
# Template, fixed reservation slot (if any), omen-compatible slot (3F only), role.
PLAN={
 'A1-01':('6A',None,None,'짧은 진입과 방향 선택'),
 'A1-02':('9A',None,None,'첫 곁방 탐사'),
 'A1-03':('6C',None,None,'짧은 순환과 하강 연결'),
 'B1-01':('9C','2',None,'첫 분기 전 유물 기회'),
 'B1-02':('6B','2',None,'초반 유료 채용 거점'),
 'B1-03':('6C',None,None,'동측 이동 전 짧은 탐사'),
 'C1-01':('6A',None,None,'짧은 진입과 방향 선택'),
 'C1-02':('6B',None,None,'작은 곁방 탐사'),
 'C1-03':('9B',None,None,'작은 순환 구역 탐사'),
 'A2-01':('9A',None,None,'기록동 연결과 선택 탐사'),
 'A2-02':('9B','5',None,'두건 선행을 선택하는 곁방'),
 'A2-03':('6A','2',None,'짧은 정비 거점'),
 'A2-04':('12C',None,None,'기록 회랑의 추가 탐사'),
 'B2-01':('9B',None,None,'본부동 순환 연결'),
 'B2-02':('9A','2',None,'초기에 접근하는 기사 계약'),
 'B2-03':('9C','5',None,'정비와 다른 활동을 담는 거점'),
 'B2-04':('12B',None,None,'두 구역을 선택 탐사'),
 'C2-01':('9C',None,None,'중앙 복귀형 연결 방'),
 'C2-02':('9B','5',None,'여왕 동행을 선택하는 곁방'),
 'C2-03':('6C','2',None,'순환 통로의 짧은 정비 거점'),
 'C2-04':('12A',None,None,'여러 곁방의 선택 탐사'),
 'A3-01':('12B',None,'6','두 기록 구역을 선택 탐사'),
 'A3-02':('15B','9','14','조건부 늑대와 깊은 곁방'),
 'A3-03':('9A','4','8','저장과 선택 활동을 함께 배치'),
 'A3-04':('6A',None,'5','하강 전 짧은 연결 방'),
 'B3-01':('9B',None,'5','본부동 순환 연결'),
 'B3-02':('18B',None,'8','중앙 통로와 부속 구역 탐사'),
 'B3-03':('9C','5','3','중앙 통로의 저장 거점'),
 'B3-04':('6C',None,'5','하강 전 짧은 순환 연결'),
 'C3-01':('12C',None,'7','추출동 회랑 연결'),
 'C3-02':('15A',None,'4','중앙 홀과 부속 구역 탐사'),
 'C3-03':('9B','4','5','순환 통로의 저장 거점'),
 'C3-04':('6A',None,'5','하강 전 짧은 연결 방'),
}
SHORT_LABELS={'붉은 두건의 용병':'두건 선행','절망의 기사':'기사 계약','증오의 여왕':'여왕 동행',
              '커다랗고 나쁠 늑대':'늑대 조우','초반 유물 보상':'유물 보상','본사 연락':'본사 연락',
              '정비':'정비','SavePoint':'저장'}


def graph(template):
    g={n['id']:[] for n in template['nodes']}
    for e in template['edges']:
        g[e['a']].append(e['b'])
        if not e.get('one_way'):g[e['b']].append(e['a'])
    for neighbors in g.values():neighbors.sort(key=lambda n:int(n) if n.isdigit() else 1000)
    return g


def shortest(g,start='S',goal='G',blocked=()):
    blocked=set(blocked)
    if start in blocked or goal in blocked:return None
    q=deque([[start]]);seen={start}
    while q:
        path=q.popleft()
        if path[-1]==goal:return path
        for n in g[path[-1]]:
            if n not in seen and n not in blocked:
                seen.add(n);q.append(path+[n])
    return None


def activities(path):
    return len({n for n in path if n not in ('S','G')})


def full_walk(g,quick):
    seen={'S'};walk=['S']
    def visit(n):
        for nb in g[n]:
            if nb!='G' and nb not in seen:
                assert n in g[nb]
                seen.add(nb);walk.append(nb);visit(nb);walk.append(n)
    visit('S');walk+=quick[1:]
    assert all(b in g[a] for a,b in zip(walk,walk[1:]))
    return walk


def reservation(source,slot):
    label=source['encounter'] or source['reserved']
    if not label:
        assert slot is None
        return []
    assert slot is not None
    category='encounter' if source['encounter'] else 'save' if label=='SavePoint' else 'reward' if label=='초반 유물 보상' else 'support'
    return [dict(node=slot,label=label,short_label=SHORT_LABELS.get(label,label),category=category,
                 condition=source.get('prerequisite') if source['id']=='A3-02' else None)]


def validate_position_capacity(room,template,profile):
    """Construct a witness placement for every admissible count tuple.

    Omen placeholder remains an ordinary node when no omen is assigned; the fixed
    encounter budget remains reserved when testing the active encounter branch.
    """
    fixed={r['node']:r['category'] for r in room['reservations']}
    if room['omen_node']:fixed[room['omen_node']]='normal'
    slots=[n['id'] for n in template['nodes'] if n['id'].isdigit()]
    sd=allocation.support_distribution(profile['support'])
    elite=allocation.ELITE_ONE[room['depth']][room['size']]
    minimum=2+int(room['depth']>=3)
    fixed_other=profile['encounter']+profile['save']+profile['reward']
    checked=0
    for (s,sw),(e,ew),(r,rw),(l,lw) in product(sd.items(),allocation.EVENT[room['size']].items(),allocation.REWARD[room['size']].items(),{0:100-elite,1:elite}.items()):
        normal=room['size']-fixed_other-s-e-r-l
        if sw*ew*rw*lw==0 or normal<minimum:continue
        placement=dict(fixed)
        if l:
            assert room['elite_candidates']
            placement[room['elite_candidates'][0]]='elite'
        free=[n for n in slots if n not in placement]
        for kind,count in [('support',s-profile['support']),('event',e),('reward_extra',r)]:
            assert len(free)>=count
            for _ in range(count):placement[free.pop(0)]=kind
        for n in free:placement[n]='normal'
        counts=Counter(placement.values())
        assert len(placement)==room['size']
        assert counts['normal']==normal and counts['support']==s and counts['event']==e and counts['reward_extra']==r and counts['elite']==l
        assert all(placement[n]==kind for n,kind in fixed.items())
        assert all(placement.get(n)!='elite' for n in room['safe_access_nodes'])
        checked+=1
    assert checked==room['composition']['composition_count']
    return checked


def assign(source,template,plan=None):
    tid,fixed_slot,omen,role=(PLAN if plan is None else plan)[source['id']]
    g=graph(template);reserved=reservation(source,fixed_slot)
    encounter_nodes=[r['node'] for r in reserved if r['category']=='encounter']
    quick=shortest(g,blocked=encounter_nodes)
    assert quick and activities(quick)==template['validation']['shortest_activity_path']
    fixed_ids={r['node'] for r in reserved}
    assert omen not in fixed_ids and (bool(omen)==(3<=source['depth']<=7))
    safe=set(quick)-{'S','G'}
    for r in reserved:
        path=shortest(g,goal=r['node'],blocked=[n for n in encounter_nodes if n!=r['node']])
        assert path
        r['access_path']=path;r['access_activities']=activities(path)
        r['position']=next({k:n[k] for k in ('x','y')} for n in template['nodes'] if n['id']==r['node'])
        safe.update(set(path)-{'S','G'})
        if r['category']=='encounter':assert shortest(g,blocked=[r['node']])
    omen_path=shortest(g,goal=omen,blocked=encounter_nodes) if omen else None
    if omen:
        assert omen_path
        safe.update(set(omen_path)-{'S','G'})
    elite_candidates=sorted(set(g)-safe-fixed_ids-{omen,'S','G'},key=int)
    assert elite_candidates or source['depth']==1
    if source['depth']==1:elite_candidates=[]
    reach_without_encounter=[n for n in g if n.isdigit() and shortest(g,goal=n,blocked=encounter_nodes)]
    assert len(reach_without_encounter)==template['activity_count']-len(encounter_nodes)
    profile=dict(id=source['id'],label=source['name'],encounter=int(bool(source['encounter'])),
                 save=int(source['reserved']=='SavePoint'),reward=int(source['reserved']=='초반 유물 보상'),
                 support=int(source['reserved'] in allocation.SUPPORT_TYPES))
    result=dict(id=source['id'],name=source['name'],wing=source['wing'],depth=source['depth'],role=role,
                previous_size=source['nodes'],size=template['activity_count'],template_id=tid,template_title=template['title'],
                reservations=reserved,omen_node=omen,omen_access_path=omen_path,elite_candidates=elite_candidates,
                safe_access_nodes=sorted(safe,key=int),quick_path=quick,quick_activity_count=activities(quick),
                full_activity_count=template['activity_count'],additional_full_activity_count=template['activity_count']-activities(quick),
                reachable_activities_without_fixed_encounter=len(reach_without_encounter),
                full_walk=full_walk(g,quick),composition=allocation.enumerate_room(template['activity_count'],source['depth'],profile),
                diagram_svg=f'../svg/upper-room-assignment/{source["id"]}.svg')
    assert activities(result['full_walk'])==result['size']
    result['validated_position_compositions']=validate_position_capacity(result,template,profile)
    if source['id']=='B1-01':
        assert quick[1]==fixed_slot and shortest(g,blocked=[fixed_slot]) is None
    return result


def facility_access(source_map,rooms):
    by_id={r['id']:r for r in rooms};g={r:[] for r in by_id};g['ENTRY']=[]
    for e in source_map['connections']:
        a,b=e['a'],e['b']
        if a not in g or b not in g:continue
        g[a].append(b)
        if e['kind'] in ('lateral','cross'):g[b].append(a)
    # Distance is the number of new activity slots before reaching a room's Start.
    distances={'ENTRY':0};paths={'ENTRY':['ENTRY']};q=[(0,'ENTRY')]
    while q:
        cost,rid=heapq.heappop(q)
        if cost!=distances[rid]:continue
        leave=0 if rid=='ENTRY' else by_id[rid]['quick_activity_count']
        for nb in g[rid]:
            new=cost+leave
            if new<distances.get(nb,float('inf')):
                distances[nb]=new;paths[nb]=paths[rid]+[nb];heapq.heappush(q,(new,nb))
    assert set(distances)==set(g)
    services=[]
    for r in rooms:
        r['entry_access_activities']=distances[r['id']]
        r['entry_room_path']=paths[r['id']]
        for p in r['reservations']:
            if p['category']=='encounter':continue  # Conditional encounters require their own progress state.
            p['from_facility_entry_activities']=distances[r['id']]+p['access_activities']
            services.append(dict(room=r['id'],name=r['name'],label=p['label'],node=p['node'],
                                 local_activities=p['access_activities'],entry_activities=p['from_facility_entry_activities'],
                                 room_path=paths[r['id']],local_path=p['access_path']))
    returns=[(distances[r['id']]+r['quick_activity_count'],r['id']) for r in rooms if r['depth']==3]
    best=min(returns)
    return dict(services=services,first_3f_gate_activities=best[0],first_3f_gate_room=best[1],
                first_3f_gate_room_path=paths[best[1]],all_upper_rooms_reachable=True,
                scope='Shortest structural access from facility Entry, using boss-bypassing room routes. Counts include destination activity, exclude Start/Gate, and are not battle counts or deterrence costs.')


def room_diagram(room,template,directory='upper-room-assignment'):
    tree=ET.parse(NODE/'svg'/f'{template["base"]}.svg');root=tree.getroot()
    root.find(TAG('title')).text=f'{room["id"]} {room["name"]} · {room["template_id"]} 배정안'
    text=root.findall(TAG('text'))
    text[0].text=f'{room["id"]} {room["name"]} · {room["template_id"]}'
    text[1].text='고정 활동과 전조 후보 위치 배정안 · 초록 테두리는 빠른 통과 경로'
    for el in text:
        if el.text in template['idea']:
            el.text=room['role'] if el.text==template['idea'][0] else f'빠른 통과 {room["quick_activity_count"]}개 · 전체 탐사 {room["size"]}개 · 추가 {room["additional_full_activity_count"]}개'
    reserved={r['node']:r for r in room['reservations']}
    for group_el in root.iter(TAG('g')):
        key=group_el.attrib.get('data-node')
        if key is None:continue
        box=group_el.find(TAG('rect'));labels=group_el.findall(TAG('text'))
        if key.isdigit():box.set('fill','#ffffff')
        if key in reserved:
            r=reserved[key]
            labels[0].text=f'{int(key):02} · {r["short_label"]}'
            labels[1].text='선행 충족 시 조우' if r['condition'] else '고정 예약'
            box.set('fill','#fff0d4' if r['category']=='encounter' else '#e4f0ff')
            box.set('stroke','#a16b20' if r['category']=='encounter' else '#356492')
            for label in labels:label.set('fill','#a16b20' if r['category']=='encounter' else '#356492')
        elif key==room['omen_node']:
            labels[0].text=f'{int(key):02} · 전조 후보'
            labels[1].text='미배정 시 일반전'
            box.set('fill','#f0e8fa');box.set('stroke','#765993');box.set('stroke-dasharray','6 4')
            for label in labels:label.set('fill','#765993')
        elif key.isdigit():
            labels[1].text='정예 후보 가능' if key in room['elite_candidates'] else '일반 활동 배정'
        if key in room['quick_path'] and key.isdigit():
            box.set('stroke','#178660');box.set('stroke-width','5')
        if key.isdigit():
            group_el.set('data-quick',str(key in room['quick_path']).lower())
            group_el.set('data-reserved',str(key in reserved).lower())
    out=NODE/'svg'/directory/f'{room["id"]}.svg'
    tree.write(out,encoding='utf-8',xml_declaration=False)


def add_text(parent,x,y,text,size=23,color='#34475f',weight='400',anchor='middle'):
    el=ET.SubElement(parent,TAG('text'),dict(x=str(x),y=str(y),**{'text-anchor':anchor,'dominant-baseline':'central',
             'font-family':art.FONT,'font-size':str(size),'font-weight':weight,'fill':color}))
    el.text=text


def overview(source_map,rooms,summary):
    source=ET.parse(NODE/'svg'/'facility-map-review-v2.svg').getroot()
    result=ET.Element(TAG('svg'),dict(width='2800',height='1450',viewBox='0 0 2800 1450'))
    ids={r['id'] for r in rooms};by_id={r['id']:r for r in rooms}
    wing_sizes={w:sum(r['size'] for r in rooms if r['wing']==w) for w in WINGS}
    for original in source:
        el=deepcopy(original);kind=el.tag.split('}')[-1]
        cls=el.attrib.get('class')
        if kind in ('title','desc'):
            el.text='1~3층 방별 크기와 내부 동선 배정안' if kind=='title' else '기존 시설 연결에 33개 방의 새 크기와 템플릿을 배정한 검토안.'
        elif kind=='g' and cls=='rooms':
            for child in list(el):
                rid=child.attrib.get('data-room')
                if rid not in ids and rid!='ENTRY':el.remove(child);continue
                if rid=='ENTRY':continue
                room=by_id[rid]
                child.find(TAG('title')).text=f'{rid} · {room["name"]} · {room["template_id"]} · 빠른 통과 {room["quick_activity_count"]} / 전체 {room["size"]}'
                labels=child.findall(TAG('text'))
                labels[0].text=f'{rid} · {room["template_id"]}'
                res=room['reservations']
                labels[-1].text=f'{res[0]["short_label"]} #{int(res[0]["node"]):02} · {room["size"]}활동' if res else f'{room["size"]}활동 · 빠른 통과 {room["quick_activity_count"]}'
        elif kind=='g' and cls=='edges':
            include=False
            for child in list(el):
                if child.tag==TAG('polyline'):
                    ys=[float(pair.split(',')[1]) for pair in child.attrib['points'].split()]
                    include=max(ys)<1210
                if not include:el.remove(child)
        elif kind=='g' and cls=='encounter-dependencies':
            pass
        elif kind in ('rect','text'):
            y=float(el.attrib.get('y',0))
            if y>=1220:continue
            if kind=='rect' and y==0:el.set('height','1450')
            if kind=='text':
                value=el.text or ''
                if value=='시설 지도 · 재검토 시안 02':el.text='1~3층 방별 크기·동선 배정안'
                elif value.startswith('여러 원정에 걸쳐'):el.text='고정 조우·정비·저장 위치 배정  |  나머지 활동은 원정별 배정'
                elif value.startswith('탐사 방 84 +'):el.text=f'상층 33개 방 · 활동 {summary["activity_slots"]}자리'
                elif value.startswith('방 수 · 이름'):el.text='기존 258자리 → 배정안 300자리 · 검토안'
                elif value.startswith('탐사 방 ') and '일반 계열' in value:
                    wing='A' if float(el.attrib['x'])<1000 else 'B' if float(el.attrib['x'])<1900 else 'C'
                    el.text=f'상층 11개 방 · 활동 {wing_sizes[wing]}자리'
        else:
            continue
        result.append(el)
    add_text(result,1400,1260,'표기 예: A2-02 · 9B = 방 ID · 9개 활동의 B 동선',25,weight='700')
    add_text(result,1400,1310,'고정 예약 12자리 유지 · 같은 층 양방향 / 하강만 가능 · 두건 선행 없이 늑대 조우 발생 없음',23)
    add_text(result,1400,1360,'정비·저장 접근과 방 통과에 선택 조우 필수 없음 · 15·18개 방은 3층에만 배치',23)
    add_text(result,1400,1410,'활동 수는 방 전체 용량이며 한 원정의 방문 수가 아님 · 세부 위치와 경로는 방별 배정표에서 확인',21,color='#617086')
    ET.ElementTree(result).write(NODE/'svg'/f'{BASE}.svg',encoding='utf-8',xml_declaration=False)
    art.ROOT=NODE;art.BASE=BASE;art.WIDTH=2800;art.HEIGHT=1450;art.TEMPLATES=[];art.rasterize()


def path_label(path):
    return ' → '.join('입구' if n=='S' else 'Gate' if n=='G' else f'{int(n):02}' for n in path)


def markdown(data):
    rooms=data['rooms'];summary=data['summary'];access=data['facility_access']
    floors='\n'.join(f'| {d}층 | {s["room_count"]} | {s["old_slots"]} → {s["new_slots"]} | '+', '.join(f'{k}개 방 {v}곳' for k,v in s['sizes'].items())+' |' for d,s in summary['by_floor'].items())
    groups=[]
    for depth in (1,2,3):
        lines=[f'## {depth}층 방별 배정','', '| 방 ID와 이름 | 역할 | 크기와 동선 | 고정 위치 | 빠른 통과 / 전체 | 추가 탐사 |','| --- | --- | --- | --- | --- | --- |']
        for r in rooms:
            if r['depth']!=depth:continue
            reserved=' / '.join(f'{x["short_label"]} #{int(x["node"]):02}' for x in r['reservations']) or '없음'
            lines.append(f'| {r["id"]} {r["name"]} | {r["role"]} | {r["previous_size"]} → **{r["template_id"]}** | {reserved} | {r["quick_activity_count"]} / {r["size"]} | +{r["additional_full_activity_count"]} |')
        groups.append('\n'.join(lines))
    services='\n'.join(f'| {a["room"]} {a["label"]} | #{int(a["node"]):02} | {a["local_activities"]} | {a["entry_activities"]} | '+' → '.join(a['room_path'])+' |' for a in access['services'])
    encounters=[]
    for r in rooms:
        for p in r['reservations']:
            if p['category']=='encounter':encounters.append(f'| {r["id"]} {p["label"]} | #{int(p["node"]):02} | {path_label(p["access_path"])} | {p["access_activities"]} | {path_label(r["quick_path"])} |')
    omen_rows='\n'.join(f'| {r["id"]} | #{int(r["omen_node"]):02} | {path_label(r["omen_access_path"])} |' for r in rooms if r['omen_node'])
    normal=summary['normal_mean_before_omen_wolf_active']
    text=f'''# 공용 구간 방 크기와 내부 동선 배정안

기록일: 2026-10-09. 시설 지도 1~3층의 **33개 방에 크기·동선·예약 활동 위치를 배정한 초안**이다. 1층은 짧은 방 중심으로, 2층은 계약과 정비에 접근하기 쉽게, 3층은 넓은 선택 탐사와 짧은 하강 연결이 섞이도록 구성한다. 총 활동 용량은 기존 258자리에서 **{summary['activity_slots']}자리**로 늘고, 고정·조건부 예약 12자리는 유지한다.

[방별 배정표와 내부 도면](../node/html/{BASE}.html) · [상층 배정 지도 PNG](../node/{BASE}.png) · [CSV](../node/data/{BASE}.csv) · [경로와 위치 JSON](../node/data/{BASE}.json)

이 문서와 배정표가 새 상층 방 구성 초안이다. 기존 [시설 지도 원본](../node/html/facility-map-review-v2.html)과 [이전 상층 구성표](facility-upper-node-composition-v1.ko.md)의 6·9·12개 수량은 변경 전 비교 기준으로 남긴다. 방 사이 위치·연결·방 이름·조우가 속한 방은 유지한다. 4층 이후 배정과 실제 게임 데이터 반영은 포함하지 않는다.

## 층별 크기 구성

| 층 | 방 수 | 기존 활동 → 배정안 | 크기 분포 |
| --- | ---: | --- | --- |
{floors}

상층 전체는 6개 방 {summary['size_counts']['6']}곳, 9개 방 {summary['size_counts']['9']}곳, 12개 방 {summary['size_counts']['12']}곳, 15개 방 {summary['size_counts']['15']}곳, 18개 방 {summary['size_counts']['18']}곳이다. 큰 방을 모두 거쳐야 하강하도록 만들지 않으며, 3층 동간연결실은 6개로 줄여 짧게 통과할 수 있게 했다. 3층의 저장 방 세 곳은 9개로 늘려 저장·지원·전조가 겹쳐도 다른 활동을 넣을 여지를 확보한다.

## 수량과 경로를 읽는 기준

- `9B`는 활동 9개인 B형 동선이다. 기존 [동선 15종](../node/html/facility-room-route-variations-v5.html)의 연결을 그대로 사용한다.
- 빠른 통과는 고정 조우를 피하면서 입구에서 Gate에 도달하는 경로의 **서로 다른 활동 노드 수**다. 시작·Gate와 완료 노드의 재통과는 세지 않는다.
- 전체 탐사는 방의 모든 활동 자리를 방문할 때의 수다. 추가 탐사는 전체에서 빠른 통과를 뺀 값이다. 고정 조우가 조건을 충족해 활성화된 경우를 기준으로 하며 실제 전투 횟수나 억제력 소모량이 아니다.
- 도면의 초록 테두리는 빠른 통과 경로, 파란 채움은 고정 시설·보상, 황금색 채움은 고정 조우, 보라색 점선은 전조 호환 자리다. 나머지 종류는 원정별로 배정한다.
- 방 이름은 공간 예시다. 의료대기실에 고정 정비를 새로 보장하거나 보호계약실에 기사 계약을 다시 배치하지 않는다.

{(chr(10)*2).join(groups)}

## 초반 보상과 고정 조우 위치

B1-01 중앙접수홀은 9C를 쓰고 **#02에 단순 선택형 유물 보상**을 둔다. 입구에서 처음 만나는 활동이며 내부 첫 갈림길 전에 접근한다. 시설 입구는 기존대로 B1-01로만 이어지고, 보상 내용은 기존 원정별 후보 배정 규칙을 따른다.

| 고정 조우 방 | 위치 | 방 입구에서 접근 경로 | 접근 활동 수 | 조우를 피하는 통과 경로 |
| --- | --- | --- | ---: | --- |
{chr(10).join(encounters)}

두건은 선행 Event/Story, 기사는 보호 계약 시작, 여왕은 동행 시작이다. 이 세 자리를 최초 고정 보스전으로 바꾸지 않는다. 늑대 #09는 이번 원정의 A2-02 두건 선행을 완료했을 때만 발생하고 호출권 구매는 필수 조건이 아니다. 선행 미완료 시 그 자리를 일반전으로 유지하는 기존 검토안을 이어 쓰며, 이 처리까지 확정 게임 정책으로 간주하지 않는다. 어느 경우든 방의 입구·Gate와 나머지 활동은 조우를 우회해 접근할 수 있다.

## 정비와 저장 접근 거리

아래 수량은 목적지 활동 자체를 포함하고 시작·Gate를 제외한다. 시설 입구 기준은 다른 방을 빠른 통과 경로로 지나간 최단 구조 거리이며, 전투 승리 수·재화 보유·유료 기능 이용 가능성을 뜻하지 않는다. 모든 경로는 선택 조우를 방문하지 않고 도달할 수 있다.

| 방과 고정 활동 | 위치 | 방 내부 접근 활동 | 시설 입구부터 활동 | 시설 방 경로 |
| --- | --- | ---: | ---: | --- |
{services}

가장 이른 3층 Gate까지는 구조상 활동 **{access['first_3f_gate_activities']}개**를 거치는 경로가 있다. 이는 귀환 자격을 얻을 수 있는 빠른 하강 경로의 수량이며 충분한 성장이나 안전한 승리를 보장하지 않는다. 정비·저장의 최대 공백과 실제 플레이 시간은 4층 이후 배정을 연결한 뒤 검토한다.

## 전조와 원정별 활동 배정

3층의 각 방에 호환 자리 하나를 확보한다. 아래는 후보 위치이며, 실제로는 계열별·층별 첫 진입 방에서 적격 전조가 있을 때만 하나를 배정한다. **12개 후보가 전부 전조로 바뀌는 것이 아니다.** 이번 초안은 방별 후보를 한 곳으로 고정한 안이며, 후보 좌표를 여러 곳 중 무작위로 선택하도록 확장하지 않았다.

| 3층 방 | 전조 호환 자리 | 조우를 피하는 접근 경로 |
| --- | --- | --- |
{omen_rows}

전조가 없으면 호환 자리는 일반전으로 유지한다. 고정 조우·저장·정비·보상을 덮어쓰지 않는다. 원정별 배정은 [활동 배정 기준표](facility-room-activity-allocation-v1.ko.md)를 적용한다. 지원 총수는 예약을 포함하며 지원 종류는 중복 없는 기존 초안을 따른다.

엘리트는 빠른 통과와 예약 활동·전조 후보까지의 선택된 접근 경로 밖에 있는 일반 후보 자리에만 넣는 안이다. 이 후보 목록은 JSON과 방별 화면에 기록했다. 수량을 먼저 정한 뒤 예약과 전조 호환 자리를 보호하고, 엘리트 위치를 고른 다음 나머지 지원·이벤트·추가 보상을 배정한다. 고정 예약을 뺀 나머지 모든 활동을 영구 고정하는 방식은 아니다.

이 배정안의 상층 전체 일반 침식전 기대 용량은 늑대 활성·전조 전 **{normal:.2f}개**다. 실제 전조가 k개 붙은 경우에만 k를 빼고, 늑대 미발생 시 일반전으로 유지하는 초안을 적용하면 1개를 더한다. 상층 전조 k는 계열별 조건에 따라 0~3이다. 이 합계는 한 원정에서 방문할 전투 수나 필요한 고유 전장 도면 수가 아니다.

## 확인한 범위

33개 방의 활동 수와 템플릿 일치, 모든 활동 및 Gate 도달, 고정 조우 우회, 초반 보상 위치, 정비·저장 접근, 전조 자리 충돌을 확인했다. {summary['position_compositions_checked']:,}개 유효 개수 조합마다 예약을 보존하고 엘리트 위치 조건까지 만족하는 배치가 하나 이상 존재함을 확인했다. 최소 일반전 수와 지원 개수 분포도 활동 기준표와 일치한다.

시설의 실제 이동 연결을 사용해 모든 상층 방과 고정 거점으로의 경로를 검사했다. 이러한 구조 검증과 별개로 적 구성·재화 수지·부상·억제력 비용을 반영한 체감 난도 검증은 남아 있다. 다음 배정은 4층 이후 각 동의 독립 구역이며, 그 뒤 실제 경로별 전투량을 바탕으로 전장 풀을 산정한다.
'''
    (ROOT/'md'/f'{BASE}.ko.md').write_text(text,encoding='utf-8')


def main():
    source_map=json.loads((NODE/'data'/'facility-map-review-v2-data.json').read_text())
    source_rooms=sorted((r for r in source_map['rooms'] if 1<=r['depth']<=3),key=lambda r:(r['depth'],r['wing'],r['id']))
    templates={t['id']:t for t in json.loads((NODE/'data'/'facility-room-route-variations-v5-data.json').read_text())['variants']}
    assert set(PLAN)=={r['id'] for r in source_rooms}
    rooms=[assign(r,templates[PLAN[r['id']][0]]) for r in source_rooms]
    summary=dict(room_count=len(rooms),old_activity_slots=sum(r['previous_size'] for r in rooms),
                 activity_slots=sum(r['size'] for r in rooms),reserved_slots=sum(len(r['reservations']) for r in rooms),
                 size_counts={str(k):v for k,v in sorted(Counter(r['size'] for r in rooms).items())},
                 template_counts=dict(sorted(Counter(r['template_id'] for r in rooms).items())),
                 normal_mean_before_omen_wolf_active=sum(r['composition']['expected']['normal'] for r in rooms),
                 position_compositions_checked=sum(r['validated_position_compositions'] for r in rooms),
                 by_floor={str(d):dict(room_count=sum(r['depth']==d for r in rooms),old_slots=sum(r['previous_size'] for r in rooms if r['depth']==d),
                 new_slots=sum(r['size'] for r in rooms if r['depth']==d),sizes={str(k):v for k,v in sorted(Counter(r['size'] for r in rooms if r['depth']==d).items())}) for d in (1,2,3)})
    assert summary['room_count']==33 and summary['old_activity_slots']==258 and summary['activity_slots']==300 and summary['reserved_slots']==12
    data=dict(status='review_proposal_not_canonical',date='2026-10-09',scope='1–3F room assignment overlay; source facility connections preserved',
              source_map='facility-map-review-v2-data.json',source_templates='facility-room-route-variations-v5-data.json',
              allocation_basis='facility-room-activity-allocation-v1.json',summary=summary,rooms=rooms,facility_access=facility_access(source_map,rooms))
    (NODE/'svg'/'upper-room-assignment').mkdir(parents=True,exist_ok=True)
    for room in rooms:room_diagram(room,templates[room['template_id']])
    overview(source_map,rooms,summary)
    (NODE/'data'/f'{BASE}.json').write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    with (NODE/'data'/f'{BASE}.csv').open('w',encoding='utf-8-sig',newline='') as f:
        writer=csv.writer(f)
        writer.writerow(['방ID','동','층','방이름','역할','기존활동수','배정활동수','템플릿','고정예약위치','빠른통과활동','전체활동','추가탐사활동','통과경로','전조후보','정예후보','일반전평균_전조전','검증한위치구성수'])
        for r in rooms:
            writer.writerow([r['id'],r['wing'],r['depth'],r['name'],r['role'],r['previous_size'],r['size'],r['template_id'],
                             ' / '.join(p['label']+' #'+p['node'] for p in r['reservations']),r['quick_activity_count'],r['size'],r['additional_full_activity_count'],
                             path_label(r['quick_path']),r['omen_node'],','.join(r['elite_candidates']),r['composition']['expected']['normal'],r['validated_position_compositions']])
    markdown(data)
    html=Path(__file__).with_name('upper_room_assignment.template.html').read_text()
    html=html.replace('__DATA__',json.dumps(data,ensure_ascii=False).replace('</','<\\/'))
    # Embed all diagrams so direct file:// viewing needs no fetch or server.
    html=html.replace('__OVERVIEW__',(NODE/'svg'/f'{BASE}.svg').read_text())
    html=html.replace('__ROOM_DIAGRAMS__','\n'.join(
        f'<template id="room-svg-{r["id"]}">'+(NODE/'svg'/'upper-room-assignment'/f'{r["id"]}.svg').read_text()+'</template>' for r in rooms))
    route_data={tid:dict(nodes=t['nodes'],edges=t['edges']) for tid,t in templates.items()}
    html=html.replace('__ROUTES__',json.dumps(route_data,ensure_ascii=False).replace('</','<\\/'))
    (NODE/'html'/f'{BASE}.html').write_text(html,encoding='utf-8')
    print(json.dumps(dict(summary=summary,facility_access=data['facility_access']),ensure_ascii=False,indent=2))


if __name__=='__main__':main()
