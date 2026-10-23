"""Build the full facility review, retaining the accepted upper-room assignments.

The 51 deeper exploration rooms reuse the existing count distributions. Four
final rooms use an explicit victory-gated six-activity layout instead.
"""
from collections import Counter
from copy import deepcopy
from pathlib import Path
import csv
import json
import xml.etree.ElementTree as ET

import build_upper_room_assignment as upper
import node_diagram_art as art

ROOT=Path(__file__).resolve().parent.parent
NODE=ROOT/'node'
BASE='facility-room-assignment-v1'
DIRECTORY='room-assignment'
TAG=upper.TAG
LOWER_PLAN={
 'A4-01':('9A',None,'8','독립 구간의 짧은 진입'),
 'A4-02':('12B',None,'6','두 열람 구역의 선택 탐사'),
 'A4-03':('6A','2','5','하강 전에 들르는 짧은 정비 거점'),
 'A4-04':('9C',None,'3','분류 통로와 선택 곁방'),
 'A4-05':('18B',None,'8','막다른 관측 구역의 넓은 선택 탐사'),
 'B4-01':('9B',None,'5','독립 구간의 순환 진입'),
 'B4-02':('12A','2','11','아무 것도 없는을 선택하는 곁방'),
 'B4-03':('6C','2','5','순환 통로의 정비 거점'),
 'B4-04':('9A',None,'8','관측 갈래와 하강 연결'),
 'C4-01':('9C',None,'3','추출동 독립 구간의 진입'),
 'C4-02':('12A','10','3','정비 후 천국에 도전하는 선택 곁방'),
 'C4-03':('6A','2','5','짧은 정비와 전조 접근'),
 'C4-04':('12C',None,'7','시료 회랑의 선택 탐사'),
 'A5-01':('9B',None,'5','회수 구역의 순환 진입'),
 'A5-02':('15B','9','14','탐욕의 왕과 넓은 보관 구역'),
 'A5-03':('9A','4','8','저장과 선택 활동을 함께 배치'),
 'A5-04':('6A',None,'5','다음 층으로 이어지는 짧은 연결'),
 'B5-01':('9A',None,'8','관측 구역의 진입과 곁방'),
 'B5-02':('12B',None,'6','직원 대기 구역의 선택 탐사'),
 'B5-03':('9C','5','3','하강 전 저장과 선택 탐사'),
 'B5-04':('6C',None,'5','격리 구역의 짧은 연결'),
 'B5-05':('18C',None,'8','폐쇄 실험 구역의 넓은 선택 탐사'),
 'C5-01':('9B',None,'5','추출 구역의 순환 진입'),
 'C5-02':('15B','9','14','녹아내리는 사랑과 격리 구역 탐사'),
 'C5-03':('9B','4','5','저장과 순환 통로의 전조 접근'),
 'C5-04':('6A',None,'5','정제 구역의 짧은 하강 연결'),
 'A6-01':('12B',None,'6','최종 준비를 위한 두 기록 구역'),
 'A6-02':('15C',None,'8','봉인 문서 구역의 넓은 선택 탐사'),
 'A6-03':('9C','5','3','최하층 정비와 하강 선택'),
 'A6-04':('6C',None,'5','반출 통로의 짧은 연결'),
 'A6-05':('9B','5','8','마탄의 호위·생존과 흉탄 접근'),
 'B6-01':('12C',None,'7','통제 회랑의 최종 준비 탐사'),
 'B6-02':('9A','2','8','실패 없는 비전투 장례 의식'),
 'B6-03':('9B','4','5','심층 정비와 선택 곁방'),
 'B6-04':('6A',None,'5','신호 통로의 짧은 하강 연결'),
 'C6-01':('12B',None,'6','최종 준비를 위한 두 추출 구역'),
 'C6-02':('18B',None,'8','고압 설비와 부속 구역 탐사'),
 'C6-03':('9C','5','3','하강 전 심층 정비 거점'),
 'C6-04':('9A',None,'8','폐액 회수 구역의 선택 탐사'),
 'C6-05':('15B','9','14','음과 양을 한 조우로 선택'),
 'A7-01':('9B',None,'5','전조 확인과 최종 진입 선택'),
 'A7-02':('15A',None,'4','마지막 추가 탐사를 위한 보관 구역'),
 'A7-03':('9A','4','8','최종 진입 전 저장 거점'),
 'A7-04':('6A',None,'5','전조 확인과 짧은 최종 연결'),
 'B7-01':('9C',None,'3','마지막 심포니 확인과 최종 진입'),
 'B7-02':('15B','9','14','웃는 시체의 산을 선택하는 곁방'),
 'B7-03':('9B','4','5','최종 진입 전 저장과 전조 확인'),
 'C7-01':('9A',None,'8','마지막 치료 확인과 최종 진입'),
 'C7-02':('15C',None,'8','최종 준비를 위한 원액 구역 탐사'),
 'C7-03':('9C','5','3','최종 진입 전 저장과 교회 접근'),
 'C7-04':('6C',None,'5','치료 확인과 짧은 최종 연결'),
}
FINAL_CONDITIONS={
 'A8-F':'이번 원정 서로 다른 새 전조 3개 결과 완료 + 실제 기록동 7층 연결',
 'B8-F':'이번 원정 서로 다른 심포니 4종 결과 완료 + 실제 중앙본부동 7층 연결',
 'C8-F':'현재 생존 소속 직원 전원 치료·표식 + 실제 추출동 7층 연결',
 'A6-H':'공개 최종보스 3종의 영구 격파 기록 + 이번 원정 마탄 완료 + A6-05 직접 연결',
}
ENCOUNTER_DETAILS={
 '아무 것도 없는':'선택 가능한 일반 보스 조우입니다. 방문하지 않고 방을 통과하거나 다른 활동에 접근할 수 있습니다.',
 '파고드는 천국':'선택 가능한 일반 보스 조우입니다. 황금가지 보유 여부는 원정 시작의 사전 작성 배치안에 따르며 매번 보장하지 않습니다.',
 '탐욕의 왕':'선택 가능한 일반 보스 조우입니다. 넓은 보관 구역의 다른 활동과 Gate는 조우를 우회해 접근할 수 있습니다.',
 '녹아내리는 사랑':'선택 가능한 일반 보스 조우입니다. 격리 구역의 다른 활동과 Gate는 조우를 우회해 접근할 수 있습니다.',
 '마탄의 사수':'이야기와 호위·생존을 묶은 한 조우입니다. 일반 전투 보상·파편·알을 추가하지 않습니다. 이번 원정 완료는 흉탄 접근 조건의 일부입니다.',
 '죽은 나비들의 장례':'실패 없는 비전투 장례 의식입니다. 일반 보스 전투로 바꾸지 않으며 알에서 부화하는 죽을 나비의 장례와 구분합니다.',
 '음 / 양':'음과 양을 묶은 한 조우입니다. 별도 두 자리나 두 방으로 나누지 않습니다.',
 '웃는 시체의 산':'선택 가능한 일반 보스 조우입니다. 마지막 저장과 최종방 진입을 위해 이 조우를 반드시 거칠 필요는 없습니다.',
}
ENCOUNTER_SHORT={'아무 것도 없는':'아무 것도 없는','파고드는 천국':'천국 조우',
    '탐욕의 왕':'탐욕 조우','녹아내리는 사랑':'사랑 조우','마탄의 사수':'마탄 조우',
    '죽은 나비들의 장례':'장례 의식','음 / 양':'음·양','웃는 시체의 산':'시체 산'}


def final_template():
    nodes=[dict(id=i,name=name,x=x,y=y) for i,name,x,y in [
        ('S','입구',850,230),('1','준비 지원',350,430),('2','최종보스',850,570),
        ('J','격파 후 선택',850,800),('3','후속 일반전 1',380,1030),
        ('4','후속 일반전 2',380,1250),('5','후속 일반전 3',1330,1030),
        ('6','탐사 보상',380,1480),('G','Gate',1330,1500)]]
    edges=[]
    def edge(a,b,one_way=False,condition=None,via=None):
        e=dict(a=a,b=b,one_way=one_way)
        if condition:e['condition']=condition
        if via:e['via']=via
        edges.append(e)
    edge('S','1',via=[[350,230]])
    edge('S','2')
    edge('1','2',via=[[350,570]])
    edge('2','J',condition='boss_victory')
    edge('J','3',via=[[380,800]])
    edge('J','5',via=[[1330,800]])
    edge('3','4');edge('4','6')
    edge('J','G',True,via=[[1030,800],[1030,1500]])
    edge('3','G',True,via=[[630,1030],[630,1440],[1330,1440]])
    edge('4','G',True,via=[[740,1250],[740,1460],[1330,1460]])
    edge('6','G',True,via=[[380,1600],[1330,1600]])
    edge('5','G',True)
    return dict(id='F6',activity_count=6,title='최종 조우와 격파 후 선택 탐사',nodes=nodes,edges=edges)


def phase_graph(template,after):
    g={n['id']:[] for n in template['nodes']}
    for e in template['edges']:
        if e.get('condition')=='boss_victory' and not after:continue
        g[e['a']].append(e['b'])
        if not e['one_way']:g[e['b']].append(e['a'])
    return g


def final_room(source,t):
    reservations=[dict(node=i,label=label,short_label=label,category=kind,condition='boss_victory' if i in ('3','4','5','6') else None)
        for i,label,kind in [('1','준비 지원','support'),('2',source['encounter'],'final_boss'),('3','후속 일반전 1','normal'),('4','후속 일반전 2','normal'),('5','후속 일반전 3','normal'),('6','탐사 보상','reward')]]
    before=phase_graph(t,False);after=phase_graph(t,True)
    reachable_before=[n for n in before if upper.shortest(before,goal=n)]
    assert set(reachable_before)=={'S','1','2'}
    assert all(upper.shortest(after,goal=n) for n in after)
    # Support is optional; no post-victory fight is required for the first exit.
    assert upper.shortest(after,goal='J',blocked=['1'])==['S','2','J']
    assert upper.shortest(after,start='J',goal='G')==['J','G']
    for n in ('3','4','5','6'):
        assert upper.shortest(after,start=n,goal='G',blocked=[v for v in ('3','4','5','6') if v!=n])==[n,'G']
    assert upper.shortest(after,goal='6',blocked=['5'])
    assert upper.shortest(after,goal='5',blocked=['3','4','6'])
    assert upper.shortest(after,start='J',goal='1',blocked=['3','4','5','6'])
    for p in reservations:
        path=upper.shortest(after,goal=p['node']);p.update(access_path=path,access_activities=sum(n.isdigit() for n in path))
    return dict(id=source['id'],name=source['name'],wing=source['wing'],depth=source['depth'],kind='final',role='준비 지원은 선택 · 격파 후 귀환 또는 후속 탐사',
        previous_size=6,size=6,template_id='F6',template_title=t['title'],reservations=reservations,omen_node=None,omen_access_path=None,elite_candidates=[],
        safe_access_nodes=[],quick_path=['S','2','J','G'],quick_activity_count=1,full_activity_count=6,additional_full_activity_count=5,
        composition=dict(expected=dict(normal=3,event=0,reward_extra=1,elite=0,support=1),composition_count=1),
        diagram_svg=f'../svg/{DIRECTORY}/{source["id"]}.svg',access_condition=FINAL_CONDITIONS[source['id']],
        return_policy='같은 6층 직접 연결로 A6-05 복귀 가능. 그 뒤 기존 6층 탐사·7층 하강. 상승 금지.' if source['id']=='A6-H' else '현재 방의 후속 탐사 또는 Gate 귀환. 7층 상승 및 9층 생성 없음.',
        phase_reachable=dict(before=reachable_before,after=list(after)),validated_position_compositions=1,
        final_composition=dict(support=1,boss=1,normal_after_victory=3,reward_after_victory=1),
        notes='준비 지원은 정비 30·상점 45·본사 연락 25 중 1종. 채용 없음. 별도 전조·정예·저장 없음. 격파 보상은 보스에서 확정하며 탐사 보상과 중복 집계하지 않음.')


def final_diagram(room,t):
    art.parts=[]
    art.parts.append('<svg xmlns="http://www.w3.org/2000/svg" width="1800" height="1780" viewBox="0 0 1800 1780">')
    art.rect(0,0,1800,1780,'#f7f8fc',radius=0)
    art.text(60,60,room['id']+' · '+room['name'],35,weight=700,anchor='start')
    art.text(60,120,'보스 1 · 준비 지원 1 · 격파 후 일반전 3 · 탐사 보상 1',24,anchor='start')
    art.rect(155,915,1400,750,'#edf5ee',radius=24)
    art.text(850,950,'격파 후 선택 탐사 · 각 활동에서 Gate로 귀환 가능',23,fill='#397554')
    by_id={n['id']:n for n in t['nodes']}
    for e in t['edges']:
        a,b=by_id[e['a']],by_id[e['b']]
        # Route endpoints to node borders, not through the labels.
        mids=e.get('via',[]);points=[(a['x'],a['y']),*mids,(b['x'],b['y'])]
        for idx,other in ((0,1),(-1,-2)):
            x,y=points[idx];ox,oy=points[other];dx,dy=ox-x,oy-y
            scale=min(120/abs(dx) if dx else 1e9,44/abs(dy) if dy else 1e9)
            points[idx]=(x+dx*scale,y+dy*scale)
        art.parts.append('<g data-victory-edge="'+str(e.get('condition')=='boss_victory').lower()+'">')
        art.line(points,color='#a46d28' if e.get('condition') else '#91a0b3',both=not e['one_way'],dashed=bool(e.get('condition')))
        art.parts.append('</g>')
    reserved={p['node']:p for p in room['reservations']}
    for n in t['nodes']:
        key=n['id'];p=reserved.get(key)
        fill='#34475f' if key in ('S','G') else '#fff0d4' if key=='2' else '#e4f0ff' if key=='1' else '#f0e8fa' if key=='J' else '#ffffff'
        color='#ffffff' if key in ('S','G') else '#34475f'
        art.parts.append(f'<g data-node="{key}" data-after-victory="{str(key in ("J","3","4","5","6","G")).lower()}">')
        art.rect(n['x']-120,n['y']-44,240,88,fill,'#8d9eb3',12)
        label=(key.zfill(2)+' · '+('최종보스' if key=='2' else p['short_label'])) if p else n['name']
        art.text(n['x'],n['y']-12,label,22,fill=color,weight=700)
        sub='지원 종류 1종 배정' if key=='1' else '승리 후 아래 연결 개방' if key=='2' else '활동 수 제외' if key in ('S','J','G') else '보스 격파 후 선택'
        art.text(n['x'],n['y']+21,sub,17,fill=color)
        art.parts.append('</g>')
    art.text(900,1720,'빈 분기·입구·Gate는 활동 수 제외 · 완료한 보스·지원 재방문으로 활동을 재생성하지 않음',23)
    art.parts.append('</svg>')
    (NODE/'svg'/DIRECTORY/f'{room["id"]}.svg').write_text('\n'.join(art.parts),encoding='utf-8')


def overview(rooms,summary):
    tree=ET.parse(NODE/'svg'/'facility-map-review-v2.svg');root=tree.getroot();by_id={r['id']:r for r in rooms}
    root.find(TAG('title')).text='시설 전체 방 크기와 내부 동선 배정안'
    for g in root.findall('.//'+TAG('g')):
        rid=g.get('data-room')
        if rid not in by_id:continue
        r=by_id[rid];labels=g.findall(TAG('text'))
        labels[0].text=f'{rid} · {r["template_id"]}'
        res=r['reservations'];label='최종보스 · 격파 후 선택 탐사' if r['kind']=='final' else f'{res[0]["short_label"]} #{int(res[0]["node"]):02}' if res else f'활동 {r["size"]}개 · 통과 {r["quick_activity_count"]}'
        # Long encounter names already appear as the main room label.
        if res and r['kind']!='final' and res[0]['category']=='encounter':label=f'조우 #{int(res[0]["node"]):02} · 활동 {r["size"]}개'
        labels[-1].text=label
        g.find(TAG('title')).text=f'{rid} · {r["name"]} · 활동 {r["size"]}개 · 클릭하여 내부 노드 보기'
    for el in root.findall(TAG('text')):
        v=el.text or ''
        if v=='시설 지도 · 재검토 시안 02':el.text='시설 전체 방 크기·내부 동선 배정안'
        elif v.startswith('여러 원정에 걸쳐'):el.text='방 클릭 → 내부 노드 → 조건·경로 확인  |  고정 지도 · 원정별 활동 배정'
        elif v.startswith('방 수 · 이름'):el.text=f'탐사 {summary["exploration_slots"]} + 최종방 24 = 전체 {summary["activity_slots"]}활동'
        elif v.startswith('상자 = 방'):el.text='상자 클릭 = 내부 노드 확인  |  6~18활동 동선 배정 · 최종방 F6 별도 구성'
        elif v.startswith('점선 최종방'):el.text='설계 검토용 전체 정보 공개 · 실제 원정의 숨김·전조·진입 조건을 해제하는 화면 아님'
    tree.write(NODE/'svg'/f'{BASE}.svg',encoding='utf-8',xml_declaration=False)
    art.ROOT=NODE;art.BASE=BASE;art.WIDTH=2800;art.HEIGHT=3010;art.TEMPLATES=[];art.rasterize()


def validate_facility(source,rooms):
    by_id={r['id']:r for r in rooms};edges=source['connections']
    for e in edges:
        if e['a']=='ENTRY':continue
        a,b=by_id[e['a']],by_id[e['b']]
        if e['kind'] in ('down','final_down'):assert a['depth']<b['depth']
        if a['depth']>=4 and b['depth']>=4:assert a['wing']==b['wing']
    seventh=[r for r in rooms if r['depth']==7]
    for r in seventh:assert any(e['a']==r['id'] and e['b']==r['wing']+'8-F' and e['kind']=='final_down' for e in edges)
    hidden=[e for e in edges if 'A6-H' in (e['a'],e['b'])]
    assert len(hidden)==1 and hidden[0]['a']=='A6-05' and hidden[0]['kind']=='conditional'
    # Every deeper room can reach that floor's service through same-floor links,
    # without entering another fixed encounter's node or a conditional final room.
    services=[]
    for depth in range(4,8):
        for wing in ('A','B','C'):
            subset={r['id']:r for r in rooms if r['depth']==depth and r['wing']==wing and r['kind']=='explore'}
            g={rid:[] for rid in subset}
            for e in edges:
                if e['kind']=='lateral' and e['a'] in g and e['b'] in g:g[e['a']].append(e['b']);g[e['b']].append(e['a'])
            hubs=[r for r in subset.values() if any(p['category'] in ('support','save') for p in r['reservations'])]
            assert len(hubs)==1
            hub=hubs[0]
            for rid in g:assert upper.shortest(g,start=rid,goal=hub['id'])
            services.append(dict(depth=depth,wing=wing,room=hub['id'],local_activities=hub['reservations'][0]['access_activities'],rooms_with_access=len(g)))
    return dict(all_7f_final_connections=len(seventh),no_deep_cross_wing=True,no_upward_connections=True,
                hidden_entry_only='A6-05',same_floor_service_access=services)


def write_markdown(data):
    rooms=data['rooms'];lower=[r for r in rooms if 4<=r['depth']<=7 and r['kind']=='explore'];s=data['summary']
    lines=['# 시설 전체 방 크기와 내부 동선 배정안','',f'기록일: 2026-10-09. 상층 33개 방에 이어 **4~7층 탐사 방 51개와 최종방 4개**를 배정했다. 탐사 방 84개와 최종방 4개를 하나의 지도에서 선택하고 내부 노드의 조건과 경로를 확인할 수 있다. 전체 활동 용량은 **{s["activity_slots"]}자리**이며 여러 원정에서 이용할 시설 전체 규모다. 한 원정의 방문 수·전투 횟수·고유 전장 이미지 수가 아니다.','',f'[전체 인터랙티브 지도](../node/html/{BASE}.html) · [지도 PNG](../node/{BASE}.png) · [방별 CSV](../node/data/{BASE}.csv) · [상세 JSON](../node/data/{BASE}.json)','',
        '기존 1~3층 300활동 배정은 유지한다. [이전 심층 구성](facility-lower-node-composition-v1.ko.md)의 453활동은 비교 기준이며, 새 크기와 내부 동선은 아래 배정안을 따른다. 시설 방 좌표·방 사이 연결·조우가 속한 방·입장 규칙은 그대로다. 6·9·12·15·18활동의 [기존 배정 비중](facility-room-activity-allocation-v1.ko.md)을 적용하고 실제 게임 데이터 변경은 포함하지 않는다.','',
        '## 심층의 크기와 역할','',
        '| 층 | 탐사 방 | 기존 → 새 활동 | 6 / 9 / 12 / 15 / 18활동 방 수 |','| --- | ---: | ---: | --- |']
    for depth in range(4,8):
        rs=[r for r in lower if r['depth']==depth];counts=Counter(r['size'] for r in rs)
        lines.append(f'| {depth}층 | {len(rs)} | {sum(r["previous_size"] for r in rs)} → {sum(r["size"] for r in rs)} | '+ ' / '.join(str(counts[n]) for n in (6,9,12,15,18))+' |')
    lines+=['','4층은 동별 독립 탐사를 시작하고, 5층은 저장과 선택 보스, 6층은 정비와 고유 조우, 7층은 마지막 전조·저장과 최종 진입 선택을 중심으로 구성한다. 저장 방은 모두 9활동으로 두고, 일부 연결 방은 6활동으로 짧게 유지한다. 15·18활동 방은 보스 곁방이나 넓은 선택 탐사 구역에 배치한다.','',
        '## 방별 배정','', '| 방 | 역할 | 이전 → 동선 | 예약 | 전조 후보 | 빠른 통과 / 전체 |','| --- | --- | --- | --- | --- | --- |']
    for r in lower:
        res=' / '.join(p['short_label']+' #'+p['node'].zfill(2) for p in r['reservations']) or '없음'
        lines.append(f'| {r["id"]} {r["name"]} | {r["role"]} | {r["previous_size"]} → {r["template_id"]} | {res} | #{r["omen_node"].zfill(2)} | {r["quick_activity_count"]} / {r["size"]} |')
    lines+=['','## 전조와 고정 조우','',
        '51개 심층 탐사 방마다 일반전을 전조로 치환할 수 있는 자리 하나를 확보한다. 실제 배정은 3~7층의 계열별·층별 첫 진입 방에서 적격 후보가 있을 때만 일어나며, 후보 전체가 전조로 변하지 않는다. 고정 예약과 전조 위치는 겹치지 않는다. 정예는 빠른 통과·예약·전조 접근 경로 밖에만 배정하고 방당 최대 1개다. 전조가 배정되어도 일반전은 최소 2개 남는다.','',
        '4층은 일반 채용의 마지막 층이다. 5층 이후에도 본사 연락을 지원 후보로 유지하되 채용 기능은 없다. 백야 계열은 현재 생존 직원 전원의 실제 치료·표식을 검사하며 마지막 치료 후 교회 조사 상호작용을 보존한다. 면류관은 입장 조건과 구분한다.','']
    for label,desc in ENCOUNTER_DETAILS.items():lines.append(f'- **{label}**: {desc}')
    lines+=['','## 최종방 여섯 활동과 격파 후 동선','',
        '최종방은 일반 방의 무작위 개수 추첨을 적용하지 않는 별도 F6 동선이다. #01 준비 지원, #02 최종보스, #03·#04·#05 후속 일반전, #06 탐사 보상을 고정한다. 입구·Gate·빈 선택 분기는 활동 수에서 제외한다. 준비 지원은 정비 30·상점 45·본사 연락 25 중 1종이며 이용은 선택이다. 전조·정예·SavePoint는 추가하지 않는다.','',
        '입구에서 준비 지원을 거치지 않고 보스에 접근할 수 있다. 보스 승리 후 빈 선택 분기에서 즉시 Gate로 귀환하거나 `일반전 1 → 일반전 2 → 탐사 보상`과 `일반전 3` 갈래를 고른다. 각 후속 활동에서 다른 미해결 활동을 거치지 않고 Gate로 갈 수 있다. 완료한 보스 노드를 통해 미이용 지원에도 돌아갈 수 있으며 완료 활동과 최초 입장 비용을 재생성하지 않는다.','',
        '격파 보상은 보스 승리 시 확정한다. 후속 탐사 보상은 별도 보상 노드이며 기존 다섯 유형 풀을 사용하고 유형별 비중은 확정하지 않는다. 스트레스 전투형의 전투는 일반전 3개와 별도다. 흉탄의 본체 전투 패턴은 기존 보류 상태를 유지한다.','',
        '| 최종방 | 입장 조건 | 격파 후 방 밖 연결 |','| --- | --- | --- |']
    for r in rooms:
        if r['kind']=='final':lines.append(f'| {r["id"]} {r["name"]} | {r["access_condition"]} | {r["return_policy"]} |')
    lines+=['','## 거점 접근과 검증','',
        '4층 이후 동간 횡단을 추가하지 않았다. 모든 심층 탐사 방에서 같은 층·같은 동의 정비 또는 저장 거점에 접근할 수 있다. 고정 조우 노드를 막아도 각 탐사 방의 다른 활동과 Gate는 도달 가능하다. 7층 11개 방은 기존대로 같은 동 8층 최종방과 연결되고 상승 연결은 없다. 흉탄 방의 유일한 외부 연결은 같은 6층 A6-05이며 선행 조건은 별도로 검사한다.','',
        '| 층·동 | 거점 | 방 내부 접근 활동 | 같은 층 접근 가능한 탐사 방 |','| --- | --- | ---: | ---: |']
    for row in data['validation']['same_floor_service_access']:lines.append(f'| {row["depth"]}층 {upper.WINGS[row["wing"]]} | {row["room"]} | {row["local_activities"]} | {row["rooms_with_access"]} |')
    lines+=['',f'84개 탐사 방의 유효 활동 개수 조합 **{s["position_compositions_checked"]:,}개**에 대해 예약과 전조 후보를 보존하는 배치가 존재함을 확인했다. 최종방 4개는 격파 전후 도달 범위, 준비 지원 생략, 격파 직후 Gate 접근, 후속 갈래 독립 접근, 모든 후속 활동의 직접 귀환, 지원 재접근을 검사했다.','',
        f'상층 300 + 심층 {s["lower_exploration_slots"]} + 최종방 24 = 전체 {s["activity_slots"]}활동이다. 탐사 방 일반전 기대 용량은 전조 치환 전·늑대 활성 기준 {s["normal_mean_before_omen_wolf_active"]:.2f}개이며 실제 전조 k만 빼고 늑대 미발생 시 1개를 더한다. 최종방의 후속 일반전은 격파한 방당 3개, 전체 용량은 12개로 별도 집계한다.','',
        '지도는 배정 검토용으로 모든 방과 후보 정보를 표시한다. 실제 원정의 미탐사 숨김·최종방 공개 조건·이동 잠금을 해제하는 화면이 아니다. 거리와 활동 수 검증은 적 난도·부상·재화·억제력 비용을 반영한 플레이 검증과 구분한다.']
    (ROOT/'md'/f'{BASE}.ko.md').write_text('\n'.join(lines)+'\n',encoding='utf-8')


def main():
    source=json.loads((NODE/'data'/'facility-map-review-v2-data.json').read_text())
    templates={t['id']:t for t in json.loads((NODE/'data'/'facility-room-route-variations-v5-data.json').read_text())['variants']}
    accepted=json.loads((NODE/'data'/'facility-upper-room-assignment-v1.json').read_text())
    rooms=deepcopy(accepted['rooms'])
    for r in rooms:r['kind']='explore'
    lower_sources=sorted((r for r in source['rooms'] if r['depth']>=4 and r['kind']=='explore'),key=lambda r:(r['depth'],r['wing'],r['id']))
    assert set(LOWER_PLAN)=={r['id'] for r in lower_sources}
    (NODE/'svg'/DIRECTORY).mkdir(parents=True,exist_ok=True)
    for sr in lower_sources:
        t=templates[LOWER_PLAN[sr['id']][0]];r=upper.assign(sr,t,LOWER_PLAN);r['kind']='explore';r['diagram_svg']=f'../svg/{DIRECTORY}/{r["id"]}.svg'
        for p in r['reservations']:
            if p['category']=='encounter':
                p['description']=ENCOUNTER_DETAILS[p['label']]
                p['short_label']=ENCOUNTER_SHORT[p['label']]
        upper.room_diagram(r,t,DIRECTORY);rooms.append(r)
    explore=list(rooms);access=upper.facility_access(source,explore)
    ft=final_template();templates['F6']=ft
    for sr in source['rooms']:
        if sr['kind']=='final':
            r=final_room(sr,ft);final_diagram(r,ft);rooms.append(r)
    rooms.sort(key=lambda r:(r['depth'],r['wing'],r['id']))
    for r in rooms:
        r['facility_connections']=[]
        for e in source['connections']:
            target=e['b'] if e['a']==r['id'] else e['a'] if e['b']==r['id'] and e['kind'] in ('lateral','cross','conditional') else None
            if target:r['facility_connections'].append(dict(room=target,kind=e['kind'],condition=e['condition']))
    lower=[r for r in explore if r['depth']>=4]
    summary=dict(room_count=88,exploration_rooms=84,final_rooms=4,upper_exploration_slots=300,
        lower_exploration_slots=sum(r['size'] for r in lower),exploration_slots=sum(r['size'] for r in explore),activity_slots=sum(r['size'] for r in rooms),
        old_activity_slots=735,reserved_slots=sum(len(r['reservations']) for r in explore)+24,
        exploration_reserved_slots=sum(len(r['reservations']) for r in explore),
        size_counts={str(n):v for n,v in sorted(Counter(r['size'] for r in rooms).items())},
        normal_mean_before_omen_wolf_active=sum(r['composition']['expected']['normal'] for r in explore),
        position_compositions_checked=sum(r['validated_position_compositions'] for r in explore))
    assert len(rooms)==88 and len(explore)==84 and len(lower)==51 and summary['exploration_reserved_slots']==32
    validation=validate_facility(source,rooms)
    data=dict(status='review_proposal_not_canonical',date='2026-10-09',scope='1–8F plus hidden 6F final room; all-facility capacity, not one expedition',
        source_map='facility-map-review-v2-data.json',upper_assignment='facility-upper-room-assignment-v1.json',allocation_basis='facility-room-activity-allocation-v1.json',
        summary=summary,rooms=rooms,facility_access=access,connections=deepcopy(source['connections']),validation=validation)
    overview(rooms,summary)
    (NODE/'data'/f'{BASE}.json').write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    with (NODE/'data'/f'{BASE}.csv').open('w',encoding='utf-8-sig',newline='') as f:
        writer=csv.writer(f);writer.writerow(['방ID','층','동','방이름','구분','이전활동수','배정활동수','동선','예약위치','전조후보','정예후보','빠른통과활동_최종방은격파후','전체활동','입장조건'])
        for r in rooms:writer.writerow([r['id'],r['depth'],r['wing'],r['name'],r['kind'],r['previous_size'],r['size'],r['template_id'],' / '.join(p['short_label']+' #'+p['node'] for p in r['reservations']),r['omen_node'],','.join(r['elite_candidates']),r['quick_activity_count'],r['size'],r.get('access_condition','')])
    write_markdown(data)
    html=Path(__file__).with_name('facility_room_assignment.template.html').read_text()
    html=html.replace('__DATA__',json.dumps(data,ensure_ascii=False).replace('</','<\\/')).replace('__ROUTES__',json.dumps({tid:dict(nodes=t['nodes'],edges=t['edges']) for tid,t in templates.items()},ensure_ascii=False).replace('</','<\\/'))
    html=html.replace('__OVERVIEW__',(NODE/'svg'/f'{BASE}.svg').read_text())
    html=html.replace('__ROOM_DIAGRAMS__','\n'.join(f'<template id="room-svg-{r["id"]}">'+(NODE/'html'/r['diagram_svg']).read_text()+'</template>' for r in rooms))
    (NODE/'html'/f'{BASE}.html').write_text(html,encoding='utf-8')
    print(json.dumps(dict(summary=summary,validation=validation),ensure_ascii=False,indent=2))


if __name__=='__main__':main()
