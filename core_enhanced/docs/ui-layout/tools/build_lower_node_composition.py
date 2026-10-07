"""Rebuild the review tables for floors 4–8; no runtime policies are changed."""
from pathlib import Path
from fractions import Fraction as F
from itertools import product
from collections import defaultdict
import csv
import json

ROOT = Path(__file__).resolve().parent.parent
BASE = 'facility-lower-node-composition-v1'
WINGS = {'A': '기록동', 'B': '중앙본부동', 'C': '추출동'}
EVENT = {6: {0: 55, 1: 45}, 9: {0: 30, 1: 60, 2: 10}, 12: {0: 20, 1: 55, 2: 25}}
REWARD = {6: {0: 70, 1: 30}, 9: {0: 45, 1: 50, 2: 5}, 12: {0: 30, 1: 55, 2: 15}}
ELITE_ONE = {4: {6: 30, 9: 45, 12: 60}, 5: {6: 35, 9: 50, 12: 65},
             6: {6: 40, 9: 55, 12: 70}, 7: {6: 45, 9: 60, 12: 75}}
SUPPORT = {0: 40, 1: 45, 2: 15}
FIXED_SUPPORT = {1: 85, 2: 15}
SUPPORT_TYPES = {'정비': 30, '상점': 45, '본사 연락': 25}
fmt = lambda x: f'{float(x):.2f}'
data = json.loads((ROOT / 'node' / 'data' / 'facility-map-review-v2-data.json').read_text())
upper = json.loads((ROOT / 'node' / 'data' / 'facility-upper-node-composition-v1.json').read_text())
source_rooms = sorted((r for r in data['rooms'] if r['depth'] >= 4), key=lambda r: (r['depth'], r['wing'], r['id']))
results, exact = [], {}


def support_types(fixed, total):
    out = defaultdict(F)
    def walk(chosen, p):
        if len(chosen) == total:
            for kind in chosen:
                out[kind] += p
            return
        eligible = {k: v for k, v in SUPPORT_TYPES.items() if k not in chosen}
        for kind, weight in eligible.items():
            walk(chosen + [kind], p * F(weight, sum(eligible.values())))
    walk(list(fixed), F(1))
    assert sum(out.values()) == total
    return out


for n in source_rooms:
    final = n['kind'] == 'final'
    cap, depth, rid = n['nodes'], n['depth'], n['id']
    encounter, save = int(bool(n['encounter'])), int(n['reserved'] == 'SavePoint')
    fixed_support = [n['reserved']] if n['reserved'] in SUPPORT_TYPES else []
    sd = {1: 100} if final else (FIXED_SUPPORT if fixed_support else SUPPORT)
    cases, expected, types = [], defaultdict(F), defaultdict(F)
    for s, sw in sd.items():
        if final:
            candidates = [(0, 1, 0, 3, F(1))]
        else:
            elite = ELITE_ONE[depth][cap]
            candidates = []
            for (e, ew), (r, rw), (l, lw) in product(EVENT[cap].items(), REWARD[cap].items(), {0: 100-elite, 1: elite}.items()):
                g = cap - encounter - save - s - e - r - l
                if g >= 3:  # One compatible omen slot; at least two ordinary combats remain.
                    candidates.append((e, r, l, g, F(ew * rw * lw)))
        total = sum(c[-1] for c in candidates)
        assert total > 0, (rid, s)
        for e, r, l, g, weight in candidates:
            p = F(sw, 100) * weight / total
            c = dict(support=s, event=e, reward=r, elite=l, normal=g, probability=p)
            assert encounter + save + s + e + r + l + g == cap
            cases.append(c)
            for key in ('support', 'event', 'reward', 'elite', 'normal'):
                expected[key] += c[key] * p
            for key in ('event', 'reward', 'elite'):
                expected[key + '_present'] += int(c[key] > 0) * p
        for kind, p in support_types(fixed_support, s).items():
            types[kind] += F(sw, 100) * p
    assert sum(c['probability'] for c in cases) == 1
    for s, sw in sd.items():
        assert sum(c['probability'] for c in cases if c['support'] == s) == F(sw, 100)
    assert sum(types.values()) == expected['support']
    reservation = ([n['encounter'] + ' 1'] if encounter else []) + ([n['reserved'] + ' 1'] if n['reserved'] else [])
    if final:
        reservation += ['준비 지원 1', '격파 후 일반전 3', '탐사 보상 1']
    role = ('숨겨진 최종 조우와 후속 탐사' if rid == 'A6-H' else '최종 조우와 후속 탐사') if final else (
        '호위·생존과 흉탄 선행' if rid == 'A6-05' else '비전투 장례 의식' if rid == 'B6-02' else
        '일반 보스 선택 갈래' if encounter else '저장 거점' if save else
        '성장 적용 거점' if fixed_support else '진입·연결' if rid.endswith('-01') or '연결' in n['name'] else '선택 탐사 갈래')
    fixed_count = 6 if final else encounter + save + len(fixed_support)
    g = {'min': min(c['normal'] for c in cases), 'max': max(c['normal'] for c in cases), 'mean': float(expected['normal'])}
    result = dict(id=rid, name=n['name'], wing=n['wing'], depth=depth, kind=n['kind'], role=role,
                  activity_nodes=cap, reservations=reservation, reserved_slots=fixed_count,
                  remaining_assignment_slots=cap-fixed_count, fixed_encounters=encounter, fixed_save=save,
                  fixed_support=fixed_support, omen_compatible_slots=int(not final),
                  normal_without_omen=g,
                  normal_with_omen={k: v-1 for k, v in g.items()} if not final else None,
                  expected={k: float(v) for k, v in expected.items()},
                  expected_support_types={k: float(types[k]) for k in SUPPORT_TYPES},
                  recruitment_allowed=depth == 4,
                  normal_availability='after_boss_victory' if final else 'ordinary_assignment',
                  candidate_count=len(cases), probability_cases=[{**c, 'probability': float(c['probability']), 'probability_exact': str(c['probability'])} for c in cases])
    results.append(result)
    exact[rid] = expected

explore = [r for r in results if r['kind'] == 'explore']
finals = [r for r in results if r['kind'] == 'final']
assert len(explore) == 51 and sum(r['activity_nodes'] for r in explore) == 453
assert len(finals) == 4 and sum(r['activity_nodes'] for r in finals) == 24
assert sum(r['reserved_slots'] for r in explore) == 20
assert sum(r['fixed_encounters'] for r in explore) == 8
assert sum(r['fixed_save'] for r in explore) == 6
assert sum(bool(r['fixed_support']) for r in explore) == 6


def aggregate(rs):
    ex = {k: sum(exact[r['id']][k] for r in rs) for k in ('normal', 'support', 'event', 'reward', 'elite')}
    enc, saves = sum(r['fixed_encounters'] for r in rs), sum(r['fixed_save'] for r in rs)
    slots = sum(r['activity_nodes'] for r in rs)
    assert sum(ex.values()) + enc + saves == slots
    return dict(rooms=len(rs), activity_slots=slots, reserved_slots=sum(r['reserved_slots'] for r in rs),
                expected={k: float(v) for k, v in ex.items()}, fixed_encounters=enc, fixed_save=saves)


agg = aggregate(explore)
by_floor = [dict(depth=d, **aggregate([r for r in explore if r['depth'] == d])) for d in range(4, 8)]
by_wing = [dict(wing=w, **aggregate([r for r in explore if r['wing'] == w])) for w in WINGS]
meta = dict(status='review_proposal_not_canonical', date='2026-10-07',
            scope='All-map capacity: 51 exploration rooms on floors 4–7 and 4 final rooms, not one-expedition visits.',
            source_map='facility-map-review-v2-data.json',
            proposal_parameters=dict(event_count_weights=EVENT, reward_count_weights=REWARD,
                elite_one_raw_weight_percent=ELITE_ONE, support_count_percent=SUPPORT,
                reserved_support_count_percent=FIXED_SUPPORT, support_type_weights=SUPPORT_TYPES,
                min_normal_before_omen=3, min_normal_after_omen=2,
                final_room_composition=dict(boss=1, support=1, normal_after_victory=3, exploration_reward=1)),
            omen_accounting=dict(compatible_exploration_rooms=51, per_wing_per_floor_max=1,
                whole_map_slot_upper_bound=12, single_wing_slot_upper_bound=4,
                note='Structural ceiling, not an expected spawn count. Eligibility, completed pools and visited rooms determine actual k.'),
            aggregate=agg, final_aggregate=aggregate(finals), by_floor=by_floor, by_wing=by_wing,
            combined=dict(exploration_rooms=84, exploration_slots=711, final_rooms=4, final_slots=24,
                normal_no_omens_wolf_active=upper['aggregate']['normal_mean_no_omen_wolf_active'] + agg['expected']['normal'],
                final_post_victory_normal_capacity=12), rooms=results)
(ROOT / 'node' / 'data' / f'{BASE}.json').write_text(json.dumps(meta, ensure_ascii=False, indent=2), encoding='utf-8')
with (ROOT / 'node' / 'data' / f'{BASE}.csv').open('w', encoding='utf-8-sig', newline='') as f:
    writer = csv.writer(f)
    writer.writerow(['방ID','동','층','구분','방이름','역할','전체활동','예약내역','예약수','남은배정자리','전조호환자리','일반전최소_전조없음','일반전최대_전조없음','일반전평균_전조없음','일반전평균_전조대상','이벤트평균','보상평균','엘리트평균','지원평균','정비평균','상점평균','본사연락평균','채용가능층','일반전접근','이벤트발현률','보상발현률','엘리트발현률'])
    for r in results:
        g, e = r['normal_without_omen'], r['expected']
        writer.writerow([r['id'],WINGS[r['wing']],r['depth'],r['kind'],r['name'],r['role'],r['activity_nodes'],' / '.join(r['reservations']),r['reserved_slots'],r['remaining_assignment_slots'],r['omen_compatible_slots'],g['min'],g['max'],fmt(g['mean']),fmt(r['normal_with_omen']['mean']) if r['normal_with_omen'] else '',*[fmt(e[k]) for k in ('event','reward','elite','support')],*[fmt(r['expected_support_types'][k]) for k in SUPPORT_TYPES],r['recruitment_allowed'],r['normal_availability'],*[fmt(100*e[k+'_present'])+'%' for k in ('event','reward','elite')]])

md = f'''# 심층과 최종방 노드 구성 검토안

기록일: 2026-10-07. [시설 지도 시안 02](../node/html/facility-map-review-v2.html)의 4~7층 탐사 방 51개와 최종방 4개를 상세화한다. 탐사 활동 453자리에 예약 20개를 반영하면 일반 침식전은 전조 치환 전 평균 **{fmt(agg['expected']['normal'])}개**다. 최종방은 각각 보스 1개·준비 지원 1개·격파 후 일반전 3개·탐사 보상 1개로 제안한다.

이 수량은 여러 원정에서 이용할 전체 지도 용량이다. 한 원정의 방문 수나 고유 전장 이미지 제작 수가 아니다. 방 이름·위치·6·9·12개 규모는 기존 지도 검토값을 유지하며, 아래 활동 비중과 최종방 내부 구성은 새 검토안이다.

[1~3층 구성](facility-upper-node-composition-v1.ko.md) · [필터 가능한 구성표](../node/html/{BASE}.html) · [방별 CSV](../node/data/{BASE}.csv) · [유효 구성과 확률 JSON](../node/data/{BASE}.json)

## 구간별 역할과 지원 배치

| 구간 | 탐사 역할 | 고정 거점과 조우 후보 | 진행에서 확인할 점 |
| --- | --- | --- | --- |
| 4층 | 공용 구간을 떠나 한 동의 탐사 방향 결정 | 각 동 정비 1개, 중앙본부동 B4-02 아무 것도 없는·추출동 C4-02 파고드는 천국 | 일반 채용의 마지막 층. 무작위 본사 연락이 생겨야 추가 채용 가능하며 고정 채용을 새로 보장하지 않음 |
| 5층 | 선택 보스와 추가 보상을 위한 탐사 | 각 동 SavePoint 1개, 탐욕의 왕·녹아내리는 사랑 | 채용 종료. 본사 연락은 응급지원·상점 등 기존 다른 기능 유지 |
| 6층 | 최종 준비에 필요한 성장 적용과 고유 조우 | 각 동 정비 1개, 마탄·죽은 나비들의 장례·음/양 | 마탄은 호위·생존 조우, 장례는 비전투 Story. 흉탄 접근은 별도 조건 검사 |
| 7층 | 남은 전조 조건과 편성을 점검하고 최종 진입 선택 | 각 동 SavePoint 1개, 중앙본부동 웃는 시체의 산 | 모든 7층 방은 같은 동의 8층 최종방과 연결. 조건 미달이면 귀환 또는 같은 층 탐사 |
| 최종방 | 보스 도전과 새 보상을 사용하는 후속 탐사 | 공개 8층 3개와 기록동 6층 흉탄 방 | 조건부 입장, 보스 격파 후 귀환·탐사 선택, 상승 금지 |

4층 이후에는 동 사이 횡단이 없다. 같은 동·같은 층의 기존 양방향 연결을 따라 거점에 접근하고, 하강 전에 필요한 정비와 저장을 선택한다. 5층 이후 본사 연락을 채용 불가라는 이유로 배정 풀에서 제거하지 않는다. [지도와 채용 규칙](../../game_rules/run-and-map.md)

## 일반 탐사 방의 배정

시작과 Gate는 활동 수에서 제외한다. 51개 탐사 방의 예약 20개는 일반 보스 계열 조우 8개·정비 6개·SavePoint 6개다. 예약 후 433자리에 나머지 활동을 배정한다. 정비·상점·본사 연락의 지원 합계는 방당 0~2개이며, 고정 정비도 그 상한에 포함한다. SavePoint와 보상은 지원 수와 별도로 센다.

1~3층 초안과 같은 이벤트·보상 가중치를 사용한다. 지원 수는 일반 방 0개 40%·1개 45%·2개 15%, 고정 정비 방은 예약 포함 1개 85%·2개 15%다. 지원 종류는 정비 30·상점 45·본사 연락 25의 가중치로 중복 없이 선택한다. 일반 지원 수 분포는 기존 초기값이고, 고정 지원 방의 비중과 종류 가중치는 앞선 검토안을 이어 쓴 값이다.

| 방 활동 수 | 이벤트 0개 / 1개 / 2개 원가중치 | 보상 0개 / 1개 / 2개 원가중치 |
| --- | --- | --- |
| 6 | 55 / 45 / 0 | 70 / 30 / 0 |
| 9 | 30 / 60 / 10 | 45 / 50 / 5 |
| 12 | 20 / 55 / 25 | 30 / 55 / 15 |

심층에서는 엘리트 후보 비중만 단계적으로 늘린다. 한 방의 엘리트 상한은 1개로 유지해 일반 보스·전조와 함께 과밀해지는 것을 제한한다. 아래 비중은 **자리 제약 적용 전 원가중치**이며 실제 발현률은 CSV의 값을 따른다.

| 층 | 6개 방 엘리트 0 / 1 | 9개 방 엘리트 0 / 1 | 12개 방 엘리트 0 / 1 |
| --- | --- | --- | --- |
'''
for d, values in ELITE_ONE.items():
    md += f'| {d}층 | ' + ' | '.join(f'{100-values[s]} / {values[s]}' for s in (6,9,12)) + ' |\n'
md += '''
고정 예약과 지원 수를 먼저 확보하고, 이벤트·보상·엘리트 조합 중 일반전이 3개 이상 남는 조합만 추첨한다. 각 지원 수 안에서 세 활동의 가중치 곱을 정규화하므로 지원 수 분포는 변하지 않는다. 실제 전조가 붙은 방에서 일반전 하나를 치환해도 최소 2개가 남는다. 예약 정비나 SavePoint를 덮어쓰지 않는다.

이 최소량은 작은 거점 방의 활동 다양성을 제한한다. 예를 들어 SavePoint가 있는 6개 방에 지원이 2개 나오면, 남은 3자리를 모두 일반전으로 두므로 이벤트·보상·엘리트는 배정하지 않는다. 자리 부족을 해결하기 위해 저장을 삭제하거나 지원 추첨 확률을 바꾸지 않는다. 일반전 난도는 기존 Easy 20%·Standard 50%·Hard 30%를 유지하고 정수 배분 규칙을 적용한다. [배정 기준](../../game_rules/run-and-map.md#노드-종류-배정)

## 전조 용량과 실제 보스 접근

51개 탐사 방에 각각 전조 호환 자리 하나를 준비하지만, 실제 배정은 계열별·층별 첫 진입 방 하나에만 이루어진다. 4~7층은 계열당 최대 4회, 전체 지도상 구조적 상한은 12자리다. **12는 예상 출현 수가 아니다.** 4층 이후 동을 바꿀 수 없고, 완료한 전조는 풀에서 제거되므로 실제 원정과 완료 상태에 따라 k가 줄어든다. 적격 전조가 없으면 호환 자리는 일반전으로 남는다.

| 계열 | 기존 개방 조건 | 4층 진입 시 진행이 0인 경우의 배치 여지 |
| --- | --- | --- |
| 기록동 종말새 | 서로 다른 새 전조 3개 결과 완료 | 4·5·6층에서 완료하고 7층을 미완료 전조 재제안 여지로 사용 가능 |
| 중앙본부동 오케스트라 | 서로 다른 심포니 4개 결과 완료 | 4~7층을 모두 사용해야 함. 하나를 미완료로 남기면 8층까지 완료 기회 부족 |
| 추출동 백야 | 현재 생존 소속 직원 전원 치료·표식 | 전조당 최대 3명. 4층 채용을 마친 후 미치료 직원 10명이면 최소 4회 치료 기회 필요 |

3층에서 같은 계열의 전조를 완료했다면 남은 요구량이 줄어든다. 전투 전조는 승리 또는 패배 확정이 완료이며, 방문·수동 후퇴는 완료가 아니다. 미완료 전조를 남기고 하강하면 이후 재제안될 수 있지만 후퇴 사용 횟수는 유지한다. 백야는 실제 치료만 인정하고, 새로 채용한 미치료 직원도 조건에 포함한다. 같은 방의 노드 선택 순서에 따라 4층 채용과 치료 순서도 달라질 수 있다. [전조 풀과 개방 조건](../../game_rules/growth-and-rewards.md#보스-징조와-반복-조우)

백야의 면류관은 입장 조건과 별개지만 격파를 위한 후속 진행에 필요하므로, 마지막 치료 후 교회 조사 기회를 하강 동선으로 잘라내지 않는다. 이 안에서는 별도 활동 수를 추가하지 않고 기존 전조 내부 상호작용을 보존한다. 보스 전조 완료를 최종방 순간이동이나 방 강제 전환으로 처리하지 않는다.

## 층별 탐사 용량

다음 표는 모든 방을 합친 배정 기대값이다. 실제 전조를 배정한 방에서만 일반전 1개를 뺀다. 최종방 4개는 이 표에서 제외한다.

| 층 | 탐사 방 | 전체 활동 | 예약 | 일반전 평균 전조 전 | 이벤트 평균 | 보상 평균 | 엘리트 평균 | 지원 평균 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
'''
for row in by_floor + [dict(depth='합계', **agg)]:
    e = row['expected']
    md += f'| {row["depth"]} | {row["rooms"]} | {row["activity_slots"]} | {row["reserved_slots"]} | ' + ' | '.join(fmt(e[k]) for k in ('normal','event','reward','elite','support')) + ' |\n'
md += '\n지원 평균에는 고정 정비를 포함한다. 위 다섯 활동 기대값에 조우 예약 8개와 SavePoint 6개를 더하면 453개가 된다.\n\n| 동 | 탐사 방 | 활동 자리 | 예약 | 일반전 평균 전조 전 |\n| --- | ---: | ---: | ---: | ---: |\n'
for row in by_wing:
    md += f'| {WINGS[row["wing"]]} | {row["rooms"]} | {row["activity_slots"]} | {row["reserved_slots"]} | {fmt(row["expected"]["normal"])} |\n'
md += '''
## 방별 구성표

범위는 각 방에서 가능한 최소·최대다. 전조 대상 평균 열은 그 계열·그 층의 실제 배정 방 하나에만 적용한다. 모든 행에 동시에 적용해 합산하지 않는다. 조우 갈래는 기존 방 연결과 방 내부 우회로를 유지한다.

| 방 ID와 이름 | 역할 | 활동 수 | 예약 | 일반전 범위 및 평균 전조 전 | 전조 대상 평균 |
| --- | --- | ---: | --- | --- | ---: |
'''
for r in explore:
    g = r['normal_without_omen']
    md += f'| {r["id"]} {r["name"]} | {r["role"]} | {r["activity_nodes"]} | {" / ".join(r["reservations"]) or "없음"} | {g["min"]}~{g["max"]} / {fmt(g["mean"])} | {fmt(r["normal_with_omen"]["mean"])} |\n'
md += '''
## 고정 조우를 세는 기준

아무 것도 없는은 중앙본부동 B4-02 비상회의실에 배치하여 B6-02 죽은 나비들의 장례와 2층 간격을 둔다. 기존 B5-05 폐쇄실험실은 12개 활동의 일반 탐사 방으로 사용한다.

파고드는 천국을 4층에 두어 추출동 독립 구간에서도 황금가지에 도전할 후보 위치를 확보한다. 같은 층 C4-03 정비 후 도전할 수 있고, C4-01과 C4-04 사이의 일반 탐사 경로로 조우 방을 우회할 수 있다. 황금가지의 실제 유무는 원정 시작에 선택하는 사전 작성 배치안이 결정하며, 천국에 매번 가지를 보장하는 변경은 아니다.

절망의 기사는 중앙본부동 B2-02, 파고드는 천국은 추출동 C4-02 배관점검실에 배치한다. 추출동 C3-02는 9개 활동의 일반 탐사 방이다. 천국이 떠난 기록동 A4-05는 12개 활동의 일반 탐사 방이다. 세 동의 일반 보스 계열 조우는 각각 4개이고, 심층 4~7층에 있는 조우는 합계 8개다.

파고드는 천국·탐욕의 왕·아무 것도 없는·녹아내리는 사랑·웃는 시체의 산은 각각 고정 조우 1자리다. 음/양은 한 조우로 세며 두 방으로 늘리지 않는다. 마탄은 이야기와 호위·생존을 묶은 1자리로 유지하고 일반 전투 보상·파편·알을 추가하지 않는다. 죽은 나비들의 장례는 실패 없는 비전투 의식 1자리이며, 알에서 부화하는 죽을 나비의 장례를 새로운 고정 방으로 추가하지 않는다. [조우 설계](../../skills/abnormality_design_notes.ko.md)

황금가지의 사전 작성 배치안은 즉시 조우 가능한 일반 보스 대상 여부와 실제 연결 조건을 별도로 검사한다. 위 조우 예약 8개를 모두 황금가지 보유 전투 8개로 간주하지 않는다. 전조 전투·엘리트·이벤트의 전투 분기·보상의 스트레스 전투도 일반 침식전 합계와 구분한다.

## 최종방의 여섯 활동

최종방 네 곳은 일반 방의 개수 추첨을 적용하지 않고 다음 여섯 활동의 역할을 고정하는 초안이다. 준비 지원 종류만 정비 30·상점 45·본사 연락 25 가중치로 하나를 선택한다. 네 곳 모두 5층 이후이므로 본사 연락에 채용 기능은 없다. SavePoint나 전조·엘리트를 추가로 배정하지 않는다.

| 활동 | 수 | 접근과 목적 |
| --- | ---: | --- |
| 준비 지원 | 1 | 방 진입 후 보스 전에 이용 가능. 선택 방문이고 지원 이용을 보스 입장 조건으로 만들지 않음 |
| 최종보스 | 1 | 기존 자격과 실제 연결로 방 입장 후 도전. 격파 보상은 기존 규칙으로 즉시 확정 |
| 후속 일반 침식전 | 3 | 보스 승리 후 선택 탐사. 새 보상을 준비 화면에서 장착한 뒤 이용 |
| 탐사 보상 | 1 | 후속 탐사 갈래의 끝. 보스 격파 보상을 중복 지급하는 자리가 아닌 별도 보상 노드 |

보스 격파 직후 안전한 분기에서 Gate 귀환과 후속 탐사를 선택한다. 후속 일반전은 `일반전 1 → 일반전 2 → 탐사 보상` 갈래와 `일반전 3` 갈래로 나누고 각 활동에서 귀환 가능한 연결을 둔다. 세 전투를 모두 해야 나갈 수 있는 직렬 경로로 만들지 않는다. 시작·Gate·빈 연결 분기는 여섯 활동에 추가하지 않는다. 준비 지원을 미이용한 채 격파했다면 같은 방 연결로 돌아갈 수 있고, 이용 완료한 지원은 재생성하지 않는다.

후속 탐사 보상은 기존 다섯 보상 유형 풀을 사용하되 유형별 비중은 이번에 확정하지 않는다. 스트레스 전투형이 뽑히면 그 전투는 일반전 3개와 별도다. 보스 격파 시 지급되는 전용 장비·파편과 후속 보상 노드의 보상은 서로 중복 집계하지 않는다. [보상 노드](../../game_rules/relic-reward-nodes.md)

| 방 | 위치 | 진입 조건 | 격파 후 이어지는 경로 |
| --- | --- | --- | --- |
| A8-F 종말새 | 기록동 8층 | 이번 원정 새 전조 3개 완료와 실제 7층 연결 | 현재 방의 후속 탐사와 Gate. 7층으로 상승 불가 |
| B8-F 고요한 오케스트라 | 중앙본부동 8층 | 이번 원정 심포니 4종 완료와 실제 7층 연결 | 현재 방의 후속 탐사와 Gate. 7층으로 상승 불가 |
| C8-F 백야 | 추출동 8층 | 현재 생존 소속 직원 전원 치료·표식과 실제 7층 연결 | 현재 방의 후속 탐사와 Gate. 면류관 관련 보스 규칙 유지 |
| A6-H 흉탄의 사수 | 기록동 6층 | 공개 최종보스 3종의 영구 격파 기록과 이번 원정 마탄 완료 | 현재 방 탐사 또는 직접 연결로 A6-05 복귀, 이후 기존 6층 연결·7층 하강 |

흉탄 방은 일반 전조 풀에 포함하지 않는다. 흉탄 본체 전투 패턴은 기존 보류 상태를 유지하며, 이 안은 접근과 활동 자리만 작성한다. 최종방의 후속 일반전 12개는 네 보스가 모두 격파 가능한 상태에서 이용할 전체 용량이다. 잠긴 방과 미격파 방의 후속 전투를 원정의 이용 가능 수에 더하지 않는다.

## 상층과 합친 전장 수요

'''
combined = meta['combined']['normal_no_omens_wolf_active']
md += f'''1~7층 탐사 방은 84개·711활동이며, 최종방 4개·24활동을 더하면 입구를 제외한 88개 활동 보유 방·735활동이다. 0층 입구까지 포함한 지도 방 수는 기존 89개다.

| 구간 | 전체 활동 | 일반 침식전 기대 용량 |
| --- | ---: | --- |
| 1~3층 탐사 | 258 | {fmt(upper['aggregate']['normal_mean_no_omen_wolf_active'])} − 실제 상층 전조 k상 + 늑대 미발생 시 1 |
| 4~7층 탐사 | 453 | {fmt(agg['expected']['normal'])} − 실제 심층 전조 k심 |
| 최종방 네 곳 | 24 | 격파 후 용량 합계 12개. 실제 이용 가능량은 격파한 방당 3개 |
| 탐사 방 84개 합계 | 711 | {fmt(combined)} − k상 − k심 + 늑대 미발생 시 1 |

k상은 최대 3, k심의 전체 지도상 구조적 상한은 12지만, 두 상한을 실제 원정의 예상값으로 사용하지 않는다. 전조 진행도·이동·채용과 치료 상태를 반영해야 한다. 이 합계 역시 한 원정에서 84개 방에 모두 갈 수 있다는 뜻이 아니다.

고유 전장 수를 정할 때는 방 내부의 빠른 통과 경로·추가 탐사 갈래와 원정별 방문량을 먼저 계산한다. 일반전 전장 풀의 재사용 간격을 그 방문량에 맞추고, 고유 보스·전조·엘리트·이벤트·보상 전투 수요를 별도로 더한다. 활동 자리 수만큼 새 이미지를 만들지는 않는다.
'''
(ROOT / 'md' / f'{BASE}.ko.md').write_text(md, encoding='utf-8')

# A local review table, integrated into the existing facility viewer.
brief = {k: v for k, v in meta.items() if k != 'rooms'}
brief['rooms'] = [{k: v for k, v in r.items() if k != 'probability_cases'} for r in results]
template = (Path(__file__).with_name('lower_node_composition.template.html')).read_text()
(ROOT / 'node' / 'html' / f'{BASE}.html').write_text(template.replace('__DATA__', json.dumps(brief, ensure_ascii=False).replace('</', '<\\/')), encoding='utf-8')
current = json.loads((ROOT / 'node' / 'data' / 'facility-map-review-v2-data.json').read_text())
by_id = {r['id']: r for r in current['rooms']}
for r in brief['rooms']:
    assert by_id[r['id']]['nodes'] == r['activity_nodes']
    by_id[r['id']]['lower_composition'] = r
viewer = ROOT / 'node' / 'html' / 'facility-map-review-v2.html'
view = viewer.read_text()
start = view.index('const data=') + len('const data=')
end = view.index(';const rooms=', start)
view = view[:start] + json.dumps(current, ensure_ascii=False).replace('</', '<\\/') + view[end:]
(ROOT / 'node' / 'data' / 'facility-map-review-v2-data.json').write_text(json.dumps(current, ensure_ascii=False, indent=2), encoding='utf-8')
viewer.write_text(view, encoding='utf-8')
print(json.dumps({k: meta[k] for k in ('aggregate','final_aggregate','by_floor','by_wing','combined')}, ensure_ascii=False, indent=2))
