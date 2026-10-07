"""Build fifteen review-only room graphs, SVG/PNG comparisons and a local gallery.

Run from any directory. Activity kinds are deliberately not assigned here.
No facility room counts, canonical rules or earlier illustrations are changed.
"""
from pathlib import Path
from collections import deque, Counter
from itertools import combinations
from html import escape
import json
import node_diagram_art as art

OUT = Path(__file__).resolve().parent.parent / 'node'
W, H, HW, HH = 1800, 1510, 100, 43
PALETTE = {
    'blue': ('#356492', '#eaf1f8'), 'green': ('#317964', '#eaf4ee'),
    'purple': ('#765993', '#f0edf7'), 'amber': ('#a16b20', '#fff3de'),
}


def model(count, letter, title, idea, names, positions, groups, chains,
          starts, exits, via=None, source=(900, 235), gate=(900, 1190)):
    nodes = [dict(id='S', x=source[0], y=source[1], name='입구', zone=None),
             dict(id='G', x=gate[0], y=gate[1], name='Gate', zone=None)]
    for i, ((x, y), name) in enumerate(zip(positions, names), 1):
        zone = next(j for j, g in enumerate(groups) if i in g['ids'])
        nodes.append(dict(id=str(i), x=x, y=y, name=name, zone=zone))
    edges = []
    for chain in chains:
        for a, b in zip(chain, chain[1:]):
            edges.append(dict(a=str(a), b=str(b), one_way=False))
    for n in starts:
        x, y = positions[n-1]
        edges.append(dict(a='S', b=str(n), one_way=False,
                          via=[] if x == source[0] else [(x, source[1])]))
    for n in exits:
        x, y = positions[n-1]
        edges.append(dict(a=str(n), b='G', one_way=True,
                          via=[] if x == gate[0] else [(x, gate[1])]))
    for e in edges:
        authored = (via or {}).get((int(e['a']), int(e['b']))) if e['a'].isdigit() and e['b'].isdigit() else None
        if authored:
            e['via'] = authored
    return dict(id=f'{count}{letter}', base=f'facility-room-route-{count}-{letter.lower()}-v5',
                activity_count=count, title=title, idea=idea, nodes=nodes,
                groups=groups, edges=edges, status='review_proposal',
                start_and_gate_excluded=True, activity_types='Unassigned; existing expedition assignment rules apply.')


def group(label, color, ids, rect):
    return dict(label=label, color=color, ids=ids, rect=rect)


def grid(xs, ys=(445, 715, 985)):
    return [(x, y) for y in ys for x in xs]


def models():
    result = []
    p6 = [(550,445),(550,715),(550,985),(1250,445),(1250,715),(1250,985)]
    names6 = ['접수 전실','기록 복도','기록 반출','작업 전실','설비 복도','출입 관리']
    g6 = [group('서측 통로 · 3자리','blue',[1,2,3],[350,340,400,735]),
          group('동측 통로 · 3자리','green',[4,5,6],[1050,340,400,735])]
    result.append(model(6,'A','짧은 두 경로와 연결문',
        ['각 세 자리인 두 경로를 가운데에서 연결해 진행 방향을 바꿀 수 있는 구조.',
         '짧은 연결용 방의 기본형. 어느 쪽으로 들어가도 반대편 통로로 건너갈 수 있음.'],
        names6,p6,g6,[[1,2,3],[4,5,6],[2,5]],[1,4],[3,6]))
    result.append(model(6,'B','주경로와 안쪽 곁방',
        ['주경로 네 자리에 두 자리짜리 곁방을 덧붙이고, 출구는 한 곳으로 모음.',
         '곁방 끝까지 추가 탐사하거나 주경로만 통과. 지원 거점·작은 조우 방의 후보.'],
        ['접수 전실','연결 홀','안내 복도','출구 전실','부속실','안쪽 보관실'],
        [(650,420),(650,610),(650,800),(650,990),(1250,610),(1250,900)],
        [group('주경로 · 4자리','blue',[1,2,3,4],[450,315,400,780]),
         group('곁방 · 2자리','amber',[5,6],[1050,490,400,500])],
        [[1,2,3,4],[2,5,6]],[1],[4],source=(650,235),gate=(650,1190)))
    result.append(model(6,'C','작은 순환 회랑',
        ['여섯 자리를 하나의 고리로 연결해, 출구 앞에서도 반대쪽 탐사를 이어갈 수 있음.',
         '막다른 곁방 없이 한 바퀴 돌아오는 작은 탐사 방. 가운데 횡단 연결은 없음.'],
        names6,p6,[group('작은 순환 회랑 · 6자리','purple',[1,2,3,4,5,6],[350,340,1100,735])],
        [[1,2,3,6,5,4,1]],[1,4],[3,6]))
    p9 = grid([400, 900, 1400])
    names9 = ['전실', '보관실', '작업 접수', '서측 복도', '연결 홀', '설비 복도', '반출 전실', '안쪽 창고', '출입 관리']
    g9 = [group('서측 탐사', 'blue', [1,4,7], [220,340,360,735]),
          group('중앙 연결과 곁방', 'purple', [2,5,8], [720,340,360,735]),
          group('동측 탐사', 'green', [3,6,9], [1220,340,360,735])]
    result.append(model(9,'A','두 복도와 작은 곁방',
        ['서측·동측 복도를 중앙에서 연결하고, 짧은 곁방 두 곳을 덧붙인 구조.',
         '빠른 통과와 곁방 탐사를 선택하는 작은 방의 기본형.'], names9,p9,g9,
        [[1,4,7],[3,6,9],[4,5,6],[3,2],[5,8]], [1,3],[7,9]))
    result.append(model(9,'B','외곽 고리와 안쪽 방',
        ['외곽 여덟 자리가 하나의 고리로 이어지고 중앙 방 하나가 안쪽으로 분기.',
         '한 바퀴 돌거나 반대 복도로 돌아올 수 있는 순환 중심 변형.'], names9,p9,
        [group('외곽 순환 복도 · 8자리','blue',[1,2,3,4,6,7,8,9],[220,340,1360,735]),
         group('안쪽 방 · 1자리','amber',[5],[740,620,320,180])],
        [[1,2,3,6,9,8,7,4,1],[4,5]], [1,3],[7,9]))
    result.append(model(9,'C','중앙 복귀와 단일 출구',
        ['중앙 통로에서 양쪽을 탐사하고, 아래의 출구 전실 한 곳으로 복귀.',
         '드물게 섞는 단일 출구형. 우측 상단에는 짧은 막다른 곁방을 배치.'],
        ['접수 구역','중앙 전실','예비 창고','기록 복도','중앙 연결','설비 분기','기록 반출','출구 전실','설비 반출'],p9,g9,
        [[2,5,8],[2,1,4,7,8],[5,6,9,8],[6,3]], [2],[8]))

    p12 = grid([300,700,1100,1500])
    names12 = ['접수 전실','자료 보관','시료 보관','작업 전실','기록 복도','서측 연결',
               '동측 연결','설비 복도','기록 반출','서고 안쪽','예비 창고','작업 반출']
    g12 = [group('서측 · 기록 구역 6자리','blue',[1,2,5,6,9,10],[120,340,760,735]),
           group('동측 · 작업 구역 6자리','green',[3,4,7,8,11,12],[920,340,760,735])]
    result.append(model(12,'A','횡단 복도와 네 곁방',
        ['두 주 복도를 긴 연결 복도로 잇고, 위아래에 곁방을 나누어 배치.',
         '활동 8자리를 잇는 통로와 막다른 곁방 4자리로 탐사 범위를 선택.'],
        names12,p12,g12, [[1,5,9],[4,8,12],[5,6,7,8],[1,2],[4,3],[9,10],[12,11]], [1,4],[9,12]))
    result.append(model(12,'B','두 순환 구역과 연결문',
        ['각 여섯 자리 구역이 독립된 순환 복도를 가지며, 중간 연결문으로 왕래.',
         '한 구역만 탐사하거나 다른 구역까지 확장하는 선택이 뚜렷한 구조.'],
        names12,p12,g12, [[1,2,6,10,9,5,1],[3,4,8,12,11,7,3],[6,7]], [1,4],[9,12]))
    result.append(model(12,'C','외곽 회랑과 내부 횡단로',
        ['외곽 고리를 따라 돌거나, 안쪽 두 자리를 통과해 반대편으로 이동.',
         '아래 중앙의 두 출구 접근점으로 모이는 회랑형. 막다른 곁방은 없음.'],
        names12,p12,[group('외곽 회랑 · 10자리','blue',[1,2,3,4,5,8,9,10,11,12],[120,340,1560,735]),
                      group('내부 횡단로 · 2자리','purple',[6,7],[540,615,720,200])],
        [[1,2,3,4,8,12,11,10,9,5,1],[5,6,7,8]], [1,4],[10,11]))

    p15 = grid([230,565,900,1235,1570])
    names15 = ['기록 접수','열람 구역','중앙 전실','검사 접수','작업 전실',
               '분류 구역','기록 연결','중앙 홀','설비 연결','작업 복도',
               '기록 반출','보존 구역','아래 연결','설비 보관','작업 반출']
    result.append(model(15,'A','중앙 홀과 두 순환 구역',
        ['중앙 세 자리에서 기록·작업 구역으로 나뉘고, 아래 연결점으로 복귀.',
         '중앙 3자리 + 양쪽 6자리씩. 방향을 잡기 쉽고 양쪽 탐사를 이어가기 좋음.'],
        names15,p15,[group('기록 구역 · 6자리','blue',[1,2,6,7,11,12],[90,340,620,735]),
                    group('중앙 · 3자리','purple',[3,8,13],[740,340,320,735]),
                    group('작업 구역 · 6자리','green',[4,5,9,10,14,15],[1090,340,620,735])],
        [[3,8,13],[1,2,7,12,11,6,1],[4,5,10,15,14,9,4],[7,8,9],[12,13,14]], [3],[11,15]))
    result.append(model(15,'B','외곽 고리와 깊은 곁방',
        ['외곽 열두 자리는 순환하고, 안쪽 세 자리는 끝까지 들어갔다가 복귀.',
         '곁방 하나를 깊게 탐사하는 변형. 안쪽 갈래를 건너뛰어도 출구 접근 가능.'],
        names15,p15,[group('외곽 순환 · 12자리','blue',[1,2,3,4,5,6,10,11,12,13,14,15],[90,340,1620,735]),
                    group('안쪽 곁방 · 3자리','amber',[7,8,9],[425,615,950,200])],
        [[1,2,3,4,5,10,15,14,13,12,11,6,1],[6,7,8,9]], [1,5],[11,15]))
    result.append(model(15,'C','엇갈린 세 구역',
        ['접수·처리·반출 구역을 각각 다섯 자리로 나누고 연결 위치를 엇갈리게 배치.',
         '출구로 가며 세 구역을 거치되, 각 구역에서 어느 쪽으로 횡단할지 선택.'],
        names15,p15,[group('접수 구역 · 5자리','blue',[1,2,3,4,5],[90,340,1620,195]),
                    group('처리 구역 · 5자리','purple',[6,7,8,9,10],[90,610,1620,195]),
                    group('반출 구역 · 5자리','green',[11,12,13,14,15],[90,880,1620,195])],
        [[1,2,3,4,5],[6,7,8,9,10],[11,12,13,14,15],[1,6],[4,9],[7,12],[10,15]], [1,5],[11,15]))

    p18 = grid([200,480,760,1040,1320,1600])
    names18 = ['기록 접수','열람 전실','관제 접수','검사 전실','작업 접수','공정 관리',
               '분류 구역','기록 연결','관제 복도','검수 구역','설비 복도','작업 구역',
               '기록 반출','보존 구역','관제 반출','출입 관리','설비 반출','작업 반출']
    g18 = [group('기록 구역 · 6자리','blue',[1,2,7,8,13,14],[65,340,550,735]),
           group('관제 구역 · 6자리','purple',[3,4,9,10,15,16],[625,340,550,735]),
           group('작업 구역 · 6자리','green',[5,6,11,12,17,18],[1185,340,550,735])]
    loops18 = [[1,2,8,14,13,7,1],[3,4,10,16,15,9,3],[5,6,12,18,17,11,5]]
    result.append(model(18,'A','세 구역을 차례로 횡단',
        ['기록 → 관제 → 작업 구역으로 진행하며 각 구역의 순환로와 연결문을 선택.',
         '구역마다 6자리. 세 구역을 거치는 동선이며 모든 활동을 거칠 필요는 없음.'],
        names18,p18,g18,loops18+[[2,3],[8,9],[4,5],[16,17]], [1,2],[17,18],
        source=(340,235),gate=(1460,1190)))
    result.append(model(18,'B','중앙 통로와 양쪽 탐사동',
        ['중앙 여섯 자리에서 출구 방향을 잡고, 양쪽 탐사동을 선택적으로 순환.',
         '양쪽 구역은 위에서 들어가 아래로 복귀. 중앙만 통과하는 짧은 경로도 유지.'],
        names18,p18,g18,loops18+[[9,10],[2,3],[14,15],[4,5],[16,17]], [3,4],[15,16]))
    p18c = [(200,445),(200,715),(200,990),(480,445),(480,715),(480,990),
            (850,420),(850,610),(850,800),(850,990),(1120,420),(1120,610),(1120,800),(1120,990),
            (1560,420),(1560,610),(1560,800),(1560,990)]
    names18c = ['기록 접수','분류 구역','기록 반출','열람 구역','기록 연결','보존 구역',
                '관제 접수','제어 복도','관측 구역','관제 반출','검사 전실','검수 구역','설비 연결','출입 관리',
                '설비 전실','동력 구역','냉각 구역','설비 반출']
    result.append(model(18,'C','큰 관제실과 좁은 설비동',
        ['기록 6자리 · 관제 8자리 · 설비 4자리로 구역 크기와 모양을 다르게 구성.',
         '넓은 중앙 순환 구역에서 가느다란 설비동으로 확장하는 비대칭 건물형.'],
        names18c,p18c,[group('기록 · 6자리','blue',list(range(1,7)),[65,340,550,735]),
                      group('관제 · 8자리','purple',list(range(7,15)),[710,315,550,765]),
                      group('설비 · 4자리','green',list(range(15,19)),[1420,315,280,765])],
        [[1,2,3,6,5,4,1],[7,8,9,10,14,13,12,11,7],[8,12],[15,16,17,18],[5,8],[6,10],[11,15],[13,17]],
        [1,4],[14,18],via={(5,8):[(660,715),(660,610)]},source=(340,235),gate=(1400,1190)))
    return result


def route(m, e):
    ns = {n['id']:n for n in m['nodes']}
    a, b = ns[e['a']], ns[e['b']]
    points = [(a['x'],a['y']), *e.get('via',[]), (b['x'],b['y'])]
    def clip(p, q):
        dx, dy = q[0]-p[0], q[1]-p[1]
        f = 1/max(abs(dx)/HW, abs(dy)/HH)
        assert f <= 1, (m['id'], e, 'bend inside node')
        return p[0]+f*dx, p[1]+f*dy
    return [clip(points[0],points[1]), *points[1:-1], clip(points[-1],points[-2])]


def validate(m):
    ns = {n['id']:n for n in m['nodes']}
    expected = {str(i) for i in range(1,m['activity_count']+1)}
    assert set(ns) == expected | {'S','G'}
    assert Counter(i for g in m['groups'] for i in g['ids']) == Counter(range(1,m['activity_count']+1))
    graph = {n:[] for n in ns}
    seen = set()
    segments = []
    for e in m['edges']:
        key = frozenset([e['a'], e['b']])
        assert len(key) == 2 and key not in seen and key != {'S','G'}
        seen.add(key)
        graph[e['a']].append(e['b'])
        if not e['one_way']:
            graph[e['b']].append(e['a'])
        pts = route(m,e)
        for p,q in zip(pts,pts[1:]):
            assert p[0]==q[0] or p[1]==q[1], (m['id'],e,'non-orthogonal')
            segments.append((e,p,q))
            for nid,n in ns.items():
                if nid in (e['a'], e['b']):
                    continue
                # Exact axis-aligned segment / open node-box intersection.
                overlap = (max(min(p[0],q[0]),n['x']-HW+1) <= min(max(p[0],q[0]),n['x']+HW-1)
                           and max(min(p[1],q[1]),n['y']-HH+1) <= min(max(p[1],q[1]),n['y']+HH-1))
                assert not overlap, (m['id'],e,nid,'node collision')
    # A crossing must never look like an unlabelled junction.
    for (e,p,q),(f,r,s) in combinations(segments,2):
        if e is f or {e['a'],e['b']} & {f['a'],f['b']}:
            continue
        overlap = (max(min(p[0],q[0]),min(r[0],s[0])) <= min(max(p[0],q[0]),max(r[0],s[0]))
                   and max(min(p[1],q[1]),min(r[1],s[1])) <= min(max(p[1],q[1]),max(r[1],s[1])))
        assert not overlap, (m['id'],e,f,'connector crossing')
    def reach(start, blocked=None):
        d={start:0}; queue=deque([start])
        while queue:
            n=queue.popleft()
            for nb in graph[n]:
                if nb not in d and nb!=blocked:
                    d[nb]=d[n]+1; queue.append(nb)
        return d
    distances=reach('S')
    assert set(distances)==set(ns), (m['id'],'unreachable')
    assert all('G' in reach(k) for k in expected)
    for a,b in combinations(ns.values(),2):
        assert abs(a['x']-b['x']) >= HW*2+8 or abs(a['y']-b['y']) >= HH*2+8
    leaves=sorted([n for n in expected if len(graph[n])==1],key=int)
    mandatory=sorted([n for n in expected if 'G' not in reach('S',n)],key=int)
    return dict(activity_count=len(expected),exit_approaches=[e['a'] for e in m['edges'] if e['b']=='G'],
                shortest_activity_path=distances['G']-1,dead_end_slots=leaves,mandatory_activity_slots=mandatory,
                all_nodes_reachable=True,every_activity_can_reach_exit=True,
                no_direct_entry_exit=True,no_node_or_connector_collisions=True,
                note='Activity slots, not battle counts. Connectivity only; no difficulty rating or unlock rules.')


def raster(base,width,height):
    art.ROOT,art.BASE,art.WIDTH,art.HEIGHT,art.TEMPLATES=OUT,base,width,height,[]
    art.rasterize()


def draw(m):
    v=m['validation']; ns={n['id']:n for n in m['nodes']}
    art.parts.clear()
    art.parts.append(f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}"><title>{m["id"]} · {escape(m["title"])}</title>')
    art.rect(0,0,W,H,'#f7f8fc',radius=0)
    art.text(55,60,f'{m["id"]} · {m["title"]}',39,weight=700,anchor='start')
    art.text(55,118,'방 내부 동선 검토안 · 공간명은 예시 · 활동 내용은 원정별 배정',23,fill='#65758a',anchor='start')
    art.rect(1260,35,490,105,'#e8edf5',radius=16)
    art.text(1505,69,f'활동 {m["activity_count"]}개 · 출구 접근 {len(v["exit_approaches"])}곳',25,weight=700)
    art.text(1505,111,f'최단 활동 {v["shortest_activity_path"]}개 · 시작 / Gate 제외',21,fill='#65758a')
    for g in m['groups']:
        x,y,w,h=g['rect']; ink,fill=PALETTE[g['color']]
        art.rect(x,y,w,h,fill,radius=22)
    side=set(v['dead_end_slots'])
    for e in m['edges']:
        a,b=ns[e['a']],ns[e['b']]
        color='#b18439' if e['a'] in side or e['b'] in side else '#16747f' if a['zone'] is not None and b['zone'] is not None and a['zone']!=b['zone'] else '#71869f'
        art.line(route(m,e),color,both=not e['one_way'])
    # Place zone captions above incoming corridors for unobstructed text.
    for g in m['groups']:
        x,y,w,h=g['rect']; ink,fill=PALETTE[g['color']]
        label_w=min(w-20,len(g['label'])*21+20)
        art.rect(x+(w-label_w)/2,y+10,label_w,36,fill,radius=8)
        art.text(x+w/2,y+28,g['label'],21,ink,700)
    for n in m['nodes']:
        key,x,y=n['id'],n['x'],n['y']
        ink=PALETTE[m['groups'][n['zone']]['color']][0] if n['zone'] is not None else '#34475f'
        fill='#ffffff'
        if key in side: ink,fill=PALETTE['amber']
        if key=='S': fill='#eef2f7'
        if key=='G': fill,ink='#34475f','#ffffff'
        art.parts.append(f'<g data-node="{key}">')
        art.rect(x-HW,y-HH,HW*2,HH*2,fill,ink,10)
        label=n['name'] if key in ('S','G') else f'{int(key):02} · {n["name"]}'
        art.text(x,y-10,label,23,ink,700)
        role='시작 위치' if key=='S' else '출구' if key=='G' else '곁방 끝' if key in side else '활동 자리'
        art.text(x,y+22,role,17,ink)
        art.parts.append('</g>')
    for x,color,label in [(80,'#71869f','구역 내부 연결'),(560,'#16747f','구역 사이 연결'),(1040,'#b18439','곁방 끝으로 연결')]:
        art.line([(x,1290),(x+80,1290)],color,both=True)
        art.text(x+100,1290,label,21,anchor='start')
    art.text(1740,1290,'Gate 방향은 단방향',19,fill='#65758a',anchor='end')
    for i,line in enumerate(m['idea']): art.text(900,1360+i*44,line,23,weight=700 if i==0 else 400)
    art.text(900,1470,'입구→출구 직행 없음 · 실제 이동은 노드 해결 상태를 따름 · 세로 위치는 층수가 아님',20,fill='#65758a')
    art.parts.append('</svg>')
    svg='\n'.join(art.parts)
    (OUT/'svg'/f'{m["base"]}.svg').write_text(svg,encoding='utf-8')
    (OUT/'data'/f'{m["base"]}-data.json').write_text(json.dumps(m,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    raster(m['base'],W,H)
    return svg


def comparison(ms,svgs,count):
    base=f'facility-room-route-{count}-comparison-v5'
    width,height=4200,1320
    parts=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}"><title>활동 {count}개 · 세 가지 변형</title>',
           '<rect width="100%" height="100%" fill="#f7f8fc"/>',
           f'<text x="50" y="62" font-family="{art.FONT}" font-size="39" fill="#25354b" font-weight="700">활동 {count}개 · 세 가지 내부 동선</text>']
    for i,m in enumerate(ms):
        svg=svgs[m['id']]; inner=svg[svg.index('>')+1:svg.rfind('</svg>')]
        parts.append(f'<svg x="{i*1400+15}" y="110" width="1370" height="1150" viewBox="0 0 {W} {H}">{inner}</svg>')
    parts.append(f'<text x="2100" y="1290" text-anchor="middle" font-family="{art.FONT}" font-size="25" fill="#65758a">시작·Gate 제외 · 활동 종류 미배정 · 원본 PNG와 SVG는 갤러리에서 개별 열기</text></svg>')
    (OUT/'svg'/f'{base}.svg').write_text('\n'.join(parts),encoding='utf-8'); raster(base,width,height)


def gallery(ms):
    cards=[]
    for m in ms:
        v=m['validation']; base=m['base']
        cards.append(f'''<article data-count="{m['activity_count']}"><h2>{m['id']} · {escape(m['title'])}</h2>
<p>{escape(m['idea'][0])}</p><a href="../svg/{base}.svg" target="_blank"><img src="../{base}.png" alt="{m['id']} 내부 동선: {escape(m['title'])}" loading="lazy"></a>
<p>{escape(m['idea'][1])}</p><p class="stats">활동 {m['activity_count']}개 · 출구 접근 {len(v['exit_approaches'])}곳 · 최단 활동 {v['shortest_activity_path']}개</p>
<nav><a href="../{base}.png" target="_blank">PNG 원본</a><a href="../svg/{base}.svg" target="_blank">SVG 확대</a><a href="../data/{base}-data.json" download>연결 데이터</a></nav></article>''')
    counts=sorted({m['activity_count'] for m in ms})
    buttons=f'<button data-filter="all" aria-pressed="true">전체 {len(ms)}개</button>'+''.join(f'<button data-filter="{n}" aria-pressed="false">{n}개 노드</button>' for n in counts)
    compare=' · '.join(f'<a href="../facility-room-route-{n}-comparison-v5.png" target="_blank">{n}개 비교 이미지</a>' for n in counts)
    html='''<!doctype html><html lang="ko"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>방 내부 동선 · VARIANT_COUNT가지 변형</title><style>
*{box-sizing:border-box}body{margin:0;background:#f3f5fa;color:#25354b;font:16px/1.65 'Malgun Gothic',sans-serif}main{max-width:1800px;margin:auto;padding:30px}h1{margin:0;font-size:30px}header p{max-width:1000px}button{font:inherit;cursor:pointer;padding:9px 19px;border:1px solid #a6b4c6;border-radius:8px;background:white;color:#34475f}button[aria-pressed=true]{background:#34475f;color:white}#filters{display:flex;gap:10px;flex-wrap:wrap;position:sticky;top:0;padding:15px 0;background:#f3f5fa;z-index:2}.grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:22px}article{background:white;border:1px solid #dce3ed;border-radius:14px;padding:20px}article[hidden]{display:none}h2{font-size:21px;margin:0}article p{margin:10px 0}img{display:block;width:100%;height:auto;border:1px solid #e6ebf3}nav{display:flex;gap:18px;flex-wrap:wrap}a{color:#356492}.stats{font-size:14px;color:#65758a}footer{padding:30px 0;color:#65758a}@media(max-width:1150px){.grid{grid-template-columns:repeat(2,minmax(0,1fr))}}@media(max-width:700px){main{padding:16px}.grid{grid-template-columns:1fr}}
</style><main><header><h1>방 내부 동선 · VARIANT_COUNT가지 변형</h1><p>6·9·12·15·18개마다 3가지 시안. 작은 방은 곁방과 순환로, 큰 방은 구역의 크기와 연결 방식을 바꾸었습니다. 이미지를 누르면 SVG 원본을 크게 볼 수 있습니다.</p><p>시작·Gate는 활동 수에서 제외합니다. 입구에서 출구로 바로 통과하는 연결은 없습니다. 6B·9C는 출구 접근이 한 곳이며, 나머지는 두 곳입니다. 활동 수는 전투 횟수가 아니며, 최단 활동 수는 연결 구조만으로 계산한 값입니다.</p><p>COMPARE</p></header><div id="filters" role="group" aria-label="활동 노드 수">BUTTONS</div><p id="shown" aria-live="polite">VARIANT_COUNT개 시안 표시</p><section class="grid">CARDS</section><footer>모두 검토안 · 공간명은 내부 구역 예시 · 시설 전체 지도와 실제 방별 노드 수는 변경하지 않음</footer></main>
<script>const buttons=[...document.querySelectorAll('[data-filter]')],cards=[...document.querySelectorAll('article[data-count]')];for(const button of buttons){button.addEventListener('click',()=>{const filter=button.dataset.filter;for(const b of buttons)b.setAttribute('aria-pressed',String(b===button));let count=0;for(const card of cards){card.hidden=filter!=='all'&&card.dataset.count!==filter;if(!card.hidden)count++;}document.getElementById('shown').textContent=`${count}개 시안 표시`;});}</script></html>'''
    html=html.replace('VARIANT_COUNT',str(len(ms))).replace('COMPARE',compare).replace('BUTTONS',buttons).replace('CARDS','\n'.join(cards))
    (OUT/'html'/'facility-room-route-variations-v5.html').write_text(html,encoding='utf-8')


def main():
    for folder in ('html','svg','data'): (OUT/folder).mkdir(parents=True,exist_ok=True)
    ms=models(); assert Counter(m['activity_count'] for m in ms)=={6:3,9:3,12:3,15:3,18:3}
    # Validate all graphs before writing any output.
    for m in ms: m['validation']=validate(m)
    svgs={m['id']:draw(m) for m in ms}
    for count in (6,9,12,15,18): comparison([m for m in ms if m['activity_count']==count],svgs,count)
    gallery(ms)
    index=dict(status='review_proposal',version=5,variants=ms,
               constraints=['No direct entry to Gate','Start and Gate excluded','6B and 9C have one exit approach; all others have two'],
               reference='Original graph variations of the user-approved room diagrams; no external game map copied.')
    (OUT/'data'/'facility-room-route-variations-v5-data.json').write_text(json.dumps(index,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    for m in ms: print(m['id'],m['title'],json.dumps(m['validation'],ensure_ascii=False))


if __name__=='__main__':main()
