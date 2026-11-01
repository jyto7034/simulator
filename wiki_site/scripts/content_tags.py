"""Curated navigation tags; evidence is checked against source text on every export.

These describe document contents, never runtime compatibility or a union of effects.
"""
import re


def build_tags(pages):
    result = {}

    def add(key, labels, evidence, note, relation='effect', source=None):
        if pages[key]['category'] == 'history':
            return
        source = source or key
        assert evidence in pages[source]['markdown'], ('Tag evidence changed', key, evidence)
        rows = result.setdefault(key, [])
        for label in labels.split('·'):
            rows.append(dict(label=label, relation=relation, note=note, source=source))

    def fragment(name, labels, evidence, note, relation='effect'):
        add('content/abnormalities/'+name+'/fragment', labels, evidence, note, relation)

    def relic(name, labels, evidence, note, relation='effect'):
        add('content/relics/'+name, labels, evidence, note, relation)

    fragment('apocalypse-bird','피해·제어·범위','지연 판결','긴 팔의 단일 판결과 작은 부리의 범위 반격 등, 각 권능의 효과를 구분합니다.')
    fragment('big-bad-wolf','파편 없음','직원용 스킬 파편을 제공하지 않는다','2026-10-19 파편·개화형·전용 결합 제외. 2026-10-25 무기·방어구도 제외하고 액세서리 양의 탈만 유지합니다.','topic')
    fragment('burrowing-heaven','시전·진행도·집중유지·지속영역·지속시간·범위·소환물·제어·받는피해증가','집중형 액티브','집중 장판의 체류 둔화→기절과 받는 피해 20% 증가. 기본 제어는 느리게 두고 유물로 강화합니다.')
    fragment('clouded-monk','피해·범위·회복','적중 시 1회 자기 회복','범위 물어뜯기와 적중 후 자기 회복은 별도 효과이며 흡혈이 아닙니다.')
    fragment('der-fluchschutze','피해·관통·투사체·스택','탄환은 지정한 사선을 따라','충전 스택과 기본형 마탄의 관통 탄환. 개화형 탄환의 충돌 규칙은 별도입니다.')
    fragment('dream-of-black-swan','피해감소·반사·패링·피해차단','패링에 성공','상시 감면·확률 반사와 패링 성공 후 0.5초 피해 면역.')
    fragment('funeral-of-dead-butterflies','파편 없음','일반 전투 보상을 지급하지 않는다','이 항목은 알만 지급합니다. 실제 파편은 죽을 나비의 장례에 있습니다.','topic')
    fragment('funeral-of-dying-butterflies','소환물·지연발동·스택·피해·범위·폭발','지연된 대규모 피해','기본형 경로의 나비와 개화형 타일 스택·폭발. 세부는 형태별로 구분합니다.')
    add('content/abnormalities/funeral-of-dying-butterflies/fragment','전용 전환','전용 전환이라는 표현은 탐색 분류','개화형 전용 유물: 기준 스택 이상이면 기존 폭발을 중앙 구체로 전환합니다.','topic','content/abnormalities/funeral-of-dying-butterflies/synergies')
    fragment('king-of-greed','시전·피해·범위·투사체·다중타격·집중유지·스택·스택획득·지속시간','초기 재고 10·자동 대여 한도 20','차징·순차 범위 사출·피해 표식 회수와 재고 대여. 기본형과 전용 기절·무한 사출은 별도 효과입니다.')
    fragment('knight-of-despair','스택·피해·범위·제어·보호막','긴급 보호막','검·상처, 검술의 피해·기절, 가호의 긴급 보호막은 별도 효과입니다.')
    fragment('laetitia','소환물·피해·폭발·범위','비저지 함정형 소환물','일반형 작은 친구 2마리·개화형 큰 친구와 작은 친구. 같은 타일 2마리분 큰 친구·3마리분 이상 특별한 친구. 선물은 전용 유물 효과입니다.')
    fragment('little-red','피해·투사체·관통','총알은 지정 타일을 출발점으로','기본형: 일반·강화 탄환의 직접 관통 피해. 두 조준 모드 모두 전투 재개 시 전체 동시 발사합니다.')
    fragment('little-red','제어·지속시간','적중 대상의 2초 속박','강화탄의 2초 속박. 직접 피해와 별도 효과이며 슬러그탄의 기절과 구분합니다.')
    fragment('little-red','스택','일반 창과 강화 창은 종류를 구별해','시간으로 얻는 일반 창과 늑대 처치로 얻는 강화 창의 보유량. 획득 증폭의 합의 대상은 강화 창뿐입니다.')
    fragment('melting-love','소환물·방해물','고기벽','검토 보류인 탱커 파편의 기존 고기벽 분류. 공격형 설계 종료와 구분합니다.')
    fragment('melting-love','피해·지속피해·주기·전파·범위·폭발·자원·성장·자연감소','감염','공격형 구조 설계 종료: 감염의 지속 피해·전파·위력 성장과 숙주 일제 기폭.')
    fragment('mountain-of-smiling-bodies','제어·범위·능력치변경·지속시간','광역 도발과 접근 강제','성장형 무기를 지원하는 광역 도발. 기존 저지를 풀고 적의 빠른 접근과 1타일 공격 제한을 적용합니다.')
    fragment('nameless-fetus','집중유지·지속영역·기본공격강화·능력치변경','집중','울음을 집중 유지하는 동안 범위 안 아군의 기본 공격 피해·공격 속도를 지원합니다.')
    fragment('nothing-there','기본공격변형·피해·투사체·관통·범위','Hello','Hello의 짧은 사거리 관통 투사체 기본 공격과 Goodbye의 범위 피해는 별도 효과입니다.')
    add('content/abnormalities/nothing-there/fragment','투사체·관통·피해','실제 투사체가 적을 관통할 때마다','꿰뚫는 송곳: Hello 탄환이 실제로 관통할 때마다 그 탄환의 이후 직접 피해가 최초 피해량의 5%씩 증가합니다. 탄환마다 초기화하며 Goodbye에는 적용하지 않습니다.','synergy','content/relics/piercing-damage-increase')
    fragment('porccubus','보호막·범위·지속시간','독립 방어막','시전 순간 범위 내 아군별 개인 방어막을 부여합니다. 지속영역이 아닙니다.')
    fragment('queen-bee','회복','일벌 하나를 소비','벌집의 횟수를 소비하는 회복. 전달 연출인 일벌을 소환물로 분류하지 않습니다.')
    fragment('queen-of-hatred','시전·피해·회복·범위·집중유지·스택','2초 동안','집중 광선과 하트. 히스테릭에서는 아군 회복 대신 피해가 적용됩니다.')
    fragment('rudolta','다중타격·피해·범위·폭발','선물 폭탄','폭격 순서는 다중타격, 개별 선물은 범위 폭발 피해입니다.')
    fragment('scarecrow-searching-for-wisdom','기본공격강화·능력치변경','헤드샷 준비','다음 기본 공격 강화와 강화 공격 처치 후 공격력 증가.')
    fragment('schadenfreude','생존보장·능력치변경·지속시간','HP가 `1`','일정 시간 HP 1을 보장하고 공격력·공격 속도·방어력을 높입니다.')
    fragment('silent-orchestra','피해·범위·스택','합창','개화형: 최초 공격·비명 파동과 공연 중 합창 누적.')
    fragment('snow-queen','피해·범위·제어·지속시간·소환물·폭발·지연발동','기본 시전으로 생성한 빙결 잔영','T자 직접 피해, 소유 피해로 냉기 30%·3초, 액티브의 혹한 2초 전환과 추격 폭발 잔영. 냉기·혹한은 상태이며 스택·진행도가 아닙니다.')
    fragment('warm-hearted-woodsman','피해·범위·다중타격·제어·지속시간·시전','끌어모으기 → 기절 → 쪼개기','범위 3연타: 1타 최대 2칸 끌어모으기, 2타 기절, 3타 잃은 체력에 따른 피해 증가. 회복은 별도 장비 효과입니다.')
    fragment('white-night','사도화·원정 누적','효과 태그를 부여하지 않으며','창→낫→봉→호위 반복의 자율 사도. 직원과 남은 시간을 원정에 보존하며 파편·사도에 효과 태그와 유물 연결을 두지 않습니다.','topic')
    fragment('yin-yang','소환물·범위·지속영역·지속시간·회복','두 구역의 아군을 회복','기본형: 두 회복 장판. 용의 피해는 별도 전용 유물 효과입니다.')
    add('content/skills/company-skills','검토 제안','태그 후보','회사 지급 스킬 묶음의 조립 검토와 태그 후보. 채택 여부는 본문을 따릅니다.','topic')
    add('content/skills/company-skills','능력치변경·회복·흡혈·피해감소·피해·범위·보호막','아드레날린','서로 다른 회사 스킬의 효과 후보를 모아 찾기 위한 분류입니다. 하나의 스킬이 모두 갖는 효과가 아닙니다.')

    # A relic declares target tags separately from the tags of effects it creates.
    # Read the current source contracts, never infer tags from an item name or operation label.
    tag_dictionary = pages['content/tags/README']['markdown']
    known_tags = set(re.findall(r'^\| `([^`]+)` \|', tag_dictionary, re.M))

    for key, page in pages.items():
        if page['category'] != 'relics' or page.get('index'):
            continue
        section = page['markdown'].split('## 유물 정의\n', 1)[1].split('\n## ', 1)[0]
        fields = {}
        for line in section.splitlines():
            cells = [cell.strip() for cell in line.strip('|').split('|')]
            if line.startswith('|') and len(cells) == 2:
                fields[cells[0]] = cells[1]
        for field, relation in [('대상 태그', 'require'), ('생성 효과 태그', 'effect')]:
            labels = re.findall(r'`([^`]+)`', fields[field])
            assert set(labels) <= known_tags, ('Unknown tag in relic', key, labels)
            if labels:
                note = fields['유물명'] + ': ' + fields['변경 규칙']
                if relation == 'require': note += ' 조건: ' + fields['적용 조건']
                add(key, '·'.join(labels), fields[field], note, relation)

    # Per-fragment agreed relic tables preserve application scope and unresolved conditions.
    for name in ('knight-of-despair','queen-of-hatred','yin-yang','little-red','funeral-of-dying-butterflies','melting-love','burrowing-heaven','mountain-of-smiling-bodies','king-of-greed','warm-hearted-woodsman','laetitia','snow-queen'):
        key = 'content/abnormalities/' + name + '/fragment'
        source = 'content/abnormalities/' + name + '/synergies'
        for line in pages[source]['markdown'].splitlines():
            match = re.match(r'^\| \[([^]]+)\]\(../../relics/([^)/]+)\.md\) \|', line)
            if not match: continue
            cells = [cell.strip() for cell in line.strip('|').split('|')]
            if len(cells) != 5: continue
            labels = re.findall(r'`([^`]+)`', cells[1])
            assert labels and set(labels) <= known_tags, ('Invalid connection tags', source, line)
            target = 'content/relics/' + match[2]
            assert target in pages and pages[target]['title'] == match[1], ('Relic link/name mismatch', source, line)
            note = match[1] + ': ' + re.sub(r'[`*]', '', ' / '.join(cells[2:]))
            add(key, '·'.join(labels), line, note, 'synergy', source)

    expected={key for key,p in pages.items() if p['category'] in ('relics','fragments') and not p.get('index')}
    assert set(result)==expected, ('Tag coverage mismatch', expected-set(result),set(result)-expected)
    for key, rows in result.items():
        for row in rows:
            if row['relation'] != 'topic':
                assert row['label'] in known_tags, ('Operation/item name used as a tag', key, row['label'])
    return result
