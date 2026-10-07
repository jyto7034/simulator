#!/usr/bin/env python3
"""Validate this local research distribution, not the game or P2G/GEX artifacts.

Uses only the Python standard library. Writes two result files under evidence/.
No repository connector, network, subprocess, production edits, or game execution.
"""
from __future__ import annotations
import hashlib
import json
import re
import sys
from collections import Counter, defaultdict, deque
from pathlib import Path
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[1]
ROAD = ROOT / 'roadmap_v2.2-review'
RECEIPT = ROOT / 'evidence/DOCUMENT_VALIDATION.json'
READABLE = ROOT / 'evidence/DOCUMENT_VALIDATION.ko.md'


def load(path: Path):
    return json.loads(path.read_text(encoding='utf-8'))


def require(condition: bool, detail: str) -> None:
    if not condition:
        raise AssertionError(detail)


def unique_ids(rows, expected, label):
    got = [x['id'] for x in rows]
    require(len(got) == len(set(got)), f'{label}: duplicate IDs')
    require(set(got) == set(expected), f'{label}: ID mismatch: {set(got)^set(expected)}')
    return {x['id']: x for x in rows}


def main() -> int:
    m = load(ROAD / 'GOALS.json')
    ledger = load(ROOT / 'evidence/source_ledger.json')
    U = {x['id']: x for x in m['units']}
    S = {x['id']: x for x in ledger['sources']}
    oldnums = [[], [0], [1], [2], [3], [4], [4,5], [5,6], [7], [6,8],
               [5,8], [4,10], [9,10,11], [6,12], [4,7,9,10], [13,14],
               [7,8,9,14], [10,11,12,16], [6,9,10,13,16], [15,17,18],
               [15,16,19], [20], [21], [22]]
    oldexpected = {f'CA{i:02}': [f'CA{j:02}' for j in d] for i,d in enumerate(oldnums)}
    checks: list[dict] = []

    def check(name, fn):
        try:
            detail = fn()
            checks.append({'name': name, 'status': 'passed', 'detail': str(detail or '일치')})
        except Exception as exc:
            checks.append({'name': name, 'status': 'failed', 'detail': str(exc)})

    def model_identity():
        require(m['schema']=='content-authoring-roadmap-review/1', 'wrong schema')
        require(m['version']=='2.2-review.1', 'wrong review version')
        require(m['goal_verdict']=='A' and m['goal_activation_authorized'] is False, 'authority or verdict drift')
        require(m['current_formal_basis']['goal_set']=='GS-CAMEC-P2G50-001', 'formal ID drift')
        require(m['current_formal_basis']['phase_plan']=='PP-CAMEC-P2G40-002', 'formal Plan drift')
        require(m['current_formal_basis']['status']=='Authored / Not Activated / Implementation Authorized: No', 'status drift')
        return '연구용 객체 스키마·버전·A 권고·원래 정식 ID·미승인 상태 일치'
    check('모델 정체성과 정식 경계', model_identity)

    def unit_ids_order():
        unique_ids(m['units'], [f'R{i:02}' for i in range(25)], 'units')
        require(m['recommended_order']==[f'R{i:02}' for i in range(25)], 'order mismatch')
        for i, uid in enumerate(m['recommended_order']):
            require(U[uid]['order_index']==i, f'{uid}: index mismatch')
        return 'R00–R24 25개, 기본 순서와 order_index 일치'
    check('단위 ID와 하나의 기본 순서', unit_ids_order)

    def status_guards():
        require(U['R00']['execution_state']=='inherited_investigation_and_formal_authoring', 'R00 status')
        require(U['R01']['execution_state']=='formal_authored_not_activated', 'R01 status')
        for uid,u in U.items():
            require(u['implementation_authorized'] is False, f'{uid}: authorized')
            require(u['actual_runtime_receipt'] is None, f'{uid}: invented runtime receipt')
            require(u['parallel_default'] is False, f'{uid}: unsupported default parallel')
            if uid not in ('R00','R01'):
                require(u['execution_state']=='proposed_not_implemented', f'{uid}: status')
        for field in ('game_tests_executed','independent_audit_performed','repo_changes_applied','next_0009_executed'):
            require(m['validation_contract'][field] is False, field)
        return '구현·게임 실행·독립 감사·저장소 적용·NEXT-0009 실행 완료 플래그 없음'
    check('실행·승인 상태 보호', status_guards)

    def original_edges():
        require(m['old_dependencies']==oldexpected, 'original adjacency mismatch')
        expected={(p,t) for t,parents in oldexpected.items() for p in parents}
        rows=m['old_dependency_edges']; actual={(e['from'],e['to']) for e in rows}
        require(len(rows)==49 and len(actual)==49 and actual==expected, '49 original edges missing/extra/duplicate')
        require(len({e['id'] for e in rows})==49, 'duplicate edge IDs')
        for e in rows:
            for f in ('disposition','relationship_types','producer_output_to_consumer','reason','broken_if_ignored','already_satisfied_or_reusable','remaining_condition','sources','evidence_kind'):
                require(bool(e.get(f)), f"{e['id']}: missing {f}")
        return '원래 adjacency 49쌍 전부 정확히 한 번 disposition, 근거·파손 조건·재사용·제한 포함'
    check('기존 관계 49개 전량 대응', original_edges)

    def ca_maps():
        cm=unique_ids(m['ca_mapping'],[f'CA{i:02}' for i in range(24)], 'CA')
        for cid,row in cm.items():
            expected={u['id'] for u in U.values() if cid in u['old_ca']}
            require(set(row['units'])==expected, f'{cid}: direct owner mismatch: {set(row["units"])} / {expected}')
            require(set(row['supporting_owners'])<=set(U), f'{cid}: unknown supporting owner')
            for key in ('disposition','current_status','review_limits','sources'):
                require(bool(row[key]), f'{cid}: missing {key}')
        return 'CA00–CA23 직접 owner가 카드의 old_ca와 양방향 일치, 보조 owner·한계 존재'
    check('전체 CA 대응·역방향 매핑', ca_maps)

    def milestone_maps():
        mm=unique_ids(m['milestone_mapping'],[f'M{i}' for i in range(9)],'M')
        groups=[[0,1],[2,3],[4,5,6,7],[8,9,10],[11,12,13],[14,15,16],[17,18,19],[20,21],[22,23]]
        cm={x['id']:x for x in m['ca_mapping']}
        for i,g in enumerate(groups):
            row=mm[f'M{i}']; ids=[f'CA{v:02}' for v in g]
            require(row['old_ca']==ids, f'M{i}: original CA mismatch')
            expected={u for c in ids for u in cm[c]['units']}
            require(set(row['units'])==expected, f'M{i}: unit mapping mismatch')
        return 'M0–M8 9개와 원래 소속 CA·새 R 매핑 일치'
    check('전체 마일스톤 대응', milestone_maps)

    def sources():
        unique_ids(ledger['sources'],[f'S{i:03}' for i in range(1,98)],'sources')
        require(len({x['path'] for x in S.values()})==97, 'duplicate paths counted')
        for sid,s in S.items():
            require(s['ranges'], f'{sid}: no ranges')
            for a,b in s['ranges']:
                require(isinstance(a,int) and isinstance(b,int) and 1<=a<=b, f'{sid}: bad range')
            for a in s['anchors']:
                require(hashlib.sha256(a['text'].encode()).hexdigest()==a['sha256_utf8'], f'{sid}: anchor digest')
            require(s['file_sha256'] is None, f'{sid}: unverified full-file digest')
        blobids=[s for s in S.values() if s.get('git_worktree_blob_abbrev')]
        require(len(blobids)==3, 'expected 3 partial git identifiers')
        for s in blobids:
            require(bool(re.fullmatch('[0-9a-f]{7,40}',s['git_worktree_blob_abbrev'])), 'bad git object id')
        return '97개 고유 경로·구간·앵커 해시 검증; 전체 source SHA256은 null, 일부 Git ID는 3개'
    check('근거 색인 구조와 제한', sources)

    def source_references():
        n=0
        def walk(x):
            nonlocal n
            if isinstance(x,dict):
                if 'sources' in x and isinstance(x['sources'],list) and all(isinstance(v,str) for v in x['sources']):
                    require(set(x['sources'])<=set(S), f'unknown source: {x["sources"]}'); n+=len(x['sources'])
                for v in x.values(): walk(v)
            elif isinstance(x,list):
                for v in x: walk(v)
        walk(m)
        return f'모델 내 source 참조 {n}개가 모두 S001–S097 안에 있음'
    check('모든 구조 데이터의 source 참조', source_references)

    def relations():
        unique_ids(m['new_relationships'],[f'REL-{i:03}' for i in range(1,77)],'relations')
        allowed={'기능상필수','검증선행','진행상권장','조건부'}
        for e in m['new_relationships']:
            require(e['from'] in U and e['to'] in U and e['from']!=e['to'], f'{e["id"]}: endpoints')
            require(e['type'] in allowed,f'{e["id"]}: type')
            for k in ('producer_output','consumer','reason','sources','evidence_kind','broken_if_ignored','already_satisfied_or_reusable','evidence_classification'):
                require(bool(e.get(k)),f'{e["id"]}: {k}')
            require(set(e['evidence_classification'])=={'observation','judgment','execution','assumption'}, 'classification shape')
        return str(dict(Counter(e['type'] for e in m['new_relationships'])))
    check('새 관계 76개의 종류·소비 지점·근거', relations)

    def typed_deps():
        rows=m['new_relationships']
        for uid,u in U.items():
            for field,typ in [('implementation_dependencies','기능상필수'),('verification_dependencies','검증선행'),('recommended_after','진행상권장')]:
                expect={e['from'] for e in rows if e['to']==uid and e['type']==typ and not e.get('conditional_revalidation')}
                require(set(u[field])==expect, f'{uid}/{field}: {set(u[field])} != {expect}')
            require(set(u['dependencies'])==set(u['implementation_dependencies']+u['verification_dependencies']),f'{uid}: combined dependencies')
        return '카드의 기능/검증/권장 선행이 typed 관계와 정확히 일치'
    check('기능·검증·권장 선행의 분리와 일치', typed_deps)

    def graph():
        rows=[e for e in m['new_relationships'] if e['type'] in ('기능상필수','검증선행') and not e.get('conditional_revalidation')]
        adj=defaultdict(list); indeg={u:0 for u in U}; pos={u:i for i,u in enumerate(m['recommended_order'])}
        for e in rows:
            adj[e['from']].append(e['to']);indeg[e['to']]+=1
            require(pos[e['from']]<pos[e['to']], f'{e["id"]}: order violation')
        q=deque(u for u,d in indeg.items() if d==0); seen=[]
        while q:
            u=q.popleft();seen.append(u)
            for v in adj[u]:
                indeg[v]-=1
                if indeg[v]==0:q.append(v)
        require(len(seen)==len(U),'hard DAG cycle')
        exceptional=[e for e in m['new_relationships'] if e.get('conditional_revalidation')]
        require(len(exceptional)==1 and exceptional[0]['id']=='REL-076' and exceptional[0]['condition'],'unexpected refresh edge')
        for e in m['new_relationships']:
            if not e.get('conditional_revalidation'):require(pos[e['from']]<pos[e['to']],f'{e["id"]}: baseline-order violation')
        return f'기능/검증 {len(rows)}개 edge DAG; 나머지 권장/조건 관계도 기본 순서와 일치. REL-076은 변경 시 재검증이며 구현 선행에서 제외'
    check('후보 DAG와 기본 순서', graph)

    def atomics():
        atoms=unique_ids(m['atomic_groups'],[f'AT{i:02}' for i in range(1,17)],'atomic groups')
        for aid,a in atoms.items():
            require(a['type']=='동시전환' and a['owner'] in U,aid)
            require(aid in U[a['owner']]['atomic_groups'],f'{aid}: owner backref')
            for k in ('components','reason','paths_and_symbols','sources','commit_policy'): require(bool(a[k]),f'{aid}:{k}')
        for uid,u in U.items():
            require(set(u['atomic_groups'])=={a['id'] for a in atoms.values() if a['owner']==uid},f'{uid}: atomics')
        return '16개 동시 전환·카드 owner 양방향 일치; runtime transaction 또는 commit 개수와 구별'
    check('동시 전환 16개와 owner', atomics)

    def contracts_gates():
        unique_ids(m['contracts'],[f'K{i:02}' for i in range(1,11)],'contracts')
        unique_ids(m['policy_gates'],[f'PG{i:02}' for i in range(10)],'gates')
        for c in m['contracts']:require(set([c['initial_owner']]+c['extension_owners'])<=set(U),f'{c["id"]}: owners')
        for p in m['policy_gates']:require(set(p['owners'])<=set(U) and p['open_condition'],f'{p["id"]}: owners/condition')
        for u in U.values():require(set(u['policy_gates'])<={p['id'] for p in m['policy_gates']},f'{u["id"]}: gates')
        return 'K01–K10, PG00–PG09와 owner·조건 참조 유효'
    check('계약과 정책 gate 보존', contracts_gates)

    def acceptance():
        a=unique_ids(m['acceptance'],[f'T{i:02}' for i in range(30)],'acceptance')
        for tid,t in a.items():
            require(t['owners'] and set(t['owners']+t['supporting_owners'])<=set(U),f'{tid}: owners')
            require(t['input'] and t['expected'] and '미실행' in t['actual_status'],f'{tid}: recipe/status')
        for uid,u in U.items():
            for tid in u['tests']:
                require(tid in a and uid in a[tid]['owners']+a[tid]['supporting_owners'],f'{uid}/{tid}: unowned test reference')
            require(u['verification_owner']==uid and u['verification_input'] and u['expected_result'],f'{uid}: verification')
        return 'T00–T29 30개; 각 카드의 모든 T 참조에 주 또는 보조 검증 owner 존재'
    check('검증 기준·입력·기대 결과·주/보조 owner', acceptance)

    def cases_findings_limits():
        unique_ids(m['representative_cases'],[f'C{i:02}' for i in range(1,8)],'cases')
        unique_ids(m['findings'],[f'F{i:02}' for i in range(1,22)],'findings')
        unique_ids(m['limitations'],[f'L{i:02}' for i in range(1,10)],'limits')
        for c in m['representative_cases']:
            require(set(c['owners'])<=set(U) and c['final_owner']=='R21','case owners')
            require('미수행' in c['actual_status'],'case falsely executed')
        for f in m['findings']:require(f['observation'] and f['design_judgment'] and f['limits'] and set(f['owners'])<=set(U),'finding fields')
        for l in m['limitations']:require(l['affected'] and l['basis'] and l['resume'],'limit fields')
        return 'C01–C07 / F01–F21 / L01–L09 모두 존재; 관찰·설계·한계·재개 구분'
    check('대표 사례·finding·미완료 범위', cases_findings_limits)

    def cards():
        files=list((ROAD/'goals').glob('*.md'))
        require({p.name for p in files}=={f'R{i:02}.ko.md' for i in range(25)},'card set mismatch')
        for uid,u in U.items():
            p=ROAD/u['card'];text=p.read_text(encoding='utf-8')
            require(text.startswith(f'# {uid} — {u["name"]}'),f'{uid}: card heading')
            require(f'기본 순서 번호: {u["order_index"]}' in text,f'{uid}: order')
            for field,heading in [('implementation_dependencies','기능상 선행'),('verification_dependencies','검증 선행'),('recommended_after','진행상 권장')]:
                line=next(x for x in text.splitlines() if x.startswith(heading+':'))
                require(set(re.findall(r'\[(R\d{2})\]',line))==set(u[field]),f'{uid}:{heading}')
            for sid in u['sources']: require(f'#s{int(sid[1:]):03}' in text,f'{uid}:source')
            require('직후 유효 상태' in text and '미승인' in text, f'{uid}: boundaries')
        return '25개 실파일의 제목·순서·3종 선행·source·중간 상태가 모델과 일치'
    check('실제 개정 작업 카드와 JSON', cards)

    def rendered_inventories():
        dep=(ROAD/'DEPENDENCIES.ko.md').read_text(encoding='utf-8')
        for field in ('old_dependency_edges','new_relationships','atomic_groups'):
            for row in m[field]:require(row['id'] in dep,f'{row["id"]}: missing rendered relation')
        road=(ROAD/'ROADMAP.ko.md').read_text(encoding='utf-8')
        for u in U:require(f'goals/{u}.ko.md' in road,f'{u}: missing ROADMAP')
        for field,filename in [('contracts','CONTRACTS.ko.md'),('policy_gates','POLICY_GATES.ko.md'),('acceptance','ACCEPTANCE.ko.md'),('representative_cases','COVERAGE.ko.md')]:
            text=(ROAD/filename).read_text(encoding='utf-8')
            for row in m[field]:require(row['id'] in text,f'{filename}: {row["id"]}')
        require(len(list((ROAD/'architecture').glob('*.md')))==8,'architecture count')
        return '관계·단위·K/PG/T/C 색인과 8개 아키텍처 실파일 존재'
    check('본문 표와 구조 데이터 색인', rendered_inventories)

    def all_json():
        paths=[p for p in ROOT.rglob('*.json') if 'input_package' not in p.parts and p!=RECEIPT and p.name!='SHA256SUMS.json']
        for p in paths:load(p)
        return f'생성 JSON {len(paths)}개 구문 해석 성공 (현재 receipt/manifest는 후속 출력)'
    check('생성 JSON 구문', all_json)

    def guards_and_guidance():
        for name in ('README.ko.md','APPLY.ko.md','CHANGELOG.ko.md','ROADMAP_STATUS_ADDENDUM.ko.md','roadmap_v2.2-review/00_README.md'):
            require((ROOT/name).is_file(),f'{name}: missing')
        st=(ROOT/'ROADMAP_STATUS_ADDENDUM.ko.md').read_text(encoding='utf-8')
        for literal in ('미적용','기록 번호 미배정','NEXT-0009','0010','Authored / Not Activated / Implementation Authorized: No'):
            require(literal in st,f'addendum: {literal}')
        close=load(ROOT/'evidence/workspace_closing_observation.json')
        require(close['record']=='0010' and close['next_card']=='NEXT-0009', 'closing selection')
        require(close['repository_changes_made_by_this_review'] is False and close['game_tests_executed'] is False,'closing authority')
        return '새 파일 배치 안내와 미적용·미배정 현황 추가 기록안 존재; 기존 선택 보존'
    check('적용 안내·현황 제안·종료 관찰', guards_and_guidance)

    def markdown_links():
        count=0; explicit_anchors=0; pending=[]
        pattern=re.compile(r'(?<!!)\[[^\]\n]+\]\(([^\n]+?)\)')
        documents=[p for p in ROOT.rglob('*.md') if 'input_package' not in p.parts and p!=READABLE]
        reserved={RECEIPT.resolve(),READABLE.resolve()}
        for p in documents:
            for match in pattern.finditer(p.read_text(encoding='utf-8')):
                target=match.group(1).split(' "',1)[0].strip('<>')
                if re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:', target):continue
                pathpart,_,fragment=target.partition('#')
                dest=(p.parent/unquote(pathpart)).resolve() if pathpart else p.resolve()
                require(dest.is_relative_to(ROOT),f'{p.relative_to(ROOT)}: outside bundle link {target}')
                if dest in reserved and not dest.exists():pending.append(str(dest.relative_to(ROOT)));continue
                require(dest.exists(),f'{p.relative_to(ROOT)}: missing link {target}')
                if fragment and re.fullmatch(r's\d{3}',fragment):
                    text=dest.read_text(encoding='utf-8')
                    require(f'id="{fragment}"' in text or f'name="{fragment}"' in text,f'{p}: missing source anchor {fragment}')
                    explicit_anchors+=1
                count+=1
        return f'{count}개 상대 링크 대상 존재, {explicit_anchors}개 source 앵커 존재. 자체 출력 예약: {sorted(set(pending)) or "없음"}. 보존한 input_package의 과거 링크는 검사/수정 대상 아님'
    check('문서 상대 링크와 source 앵커', markdown_links)

    failures=[c for c in checks if c['status']=='failed']
    result={
        'kind':'local_research_document_consistency_validation_not_P2G_GEX_or_game_test',
        'review_version':m['version'], 'review_date':m['date'],
        'command':'python tools/validate_package.py', 'exit_code':1 if failures else 0,
        'status':'failed' if failures else 'passed',
        'checks':checks,'check_count':len(checks),'passed_count':len(checks)-len(failures),'failed_count':len(failures),
        'counts':{'units':len(U),'old_edges':len(m['old_dependency_edges']),'new_relationships':len(m['new_relationships']),
                  'atomic_groups':len(m['atomic_groups']),'sources':len(S),'CA':len(m['ca_mapping']),'M':len(m['milestone_mapping']),
                  'acceptance':len(m['acceptance']),'cases':len(m['representative_cases'])},
        'input_sha256':{'GOALS.json':hashlib.sha256((ROAD/'GOALS.json').read_bytes()).hexdigest(),
                        'source_ledger.json':hashlib.sha256((ROOT/'evidence/source_ledger.json').read_bytes()).hexdigest()},
        'game_tests_executed':False,'independent_reviewer':False,'formal_validator_executed':False,
        'repository_mutation':False,'goal_activation':False,
        'limits':['로컬 문서 구조·참조·후보 그래프 검사만 수행; 관계의 의미가 실제 게임에서 옳다는 실행 증명 아님.',
                  '현재 source 전체/dirty snapshot 해시 검증 아님. 기존 formal 검증의 재실행 아님.',
                  '354 COV/127 BOSS-COV의 fresh 행별 전수 검증, geometry 및 성능, 외부 consumer 검증은 미수행.',
                  '배포 ZIP/파일 해시/create-only patch 일치 검사는 별도 packager가 수행한다.']}
    # Only these two local output files are written by this validator.
    RECEIPT.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    rows='\n'.join(f"| {i+1} | {x['name']} | {x['status']} | {x['detail'].replace('|',' / ')} |" for i,x in enumerate(checks))
    text=f'''# 실제 로컬 문서·그래프 정합성 검사\n\n**{m['version']} · 결과: {result['status']} · exit {result['exit_code']} · {result['passed_count']}/{len(checks)}개 검사 통과**\n\n실제 명령: `python tools/validate_package.py`. 아래는 이 배포본의 문서·JSON·참조·후보 그래프 검사다. 게임 테스트, 정식 P2G/GEX 검증, 독립 감사 또는 구현 자격 판정이 아니다.\n\n| 번호 | 검사 | 결과 | 실제 확인 내용 |\n| --- | --- | --- | --- |\n{rows}\n\n## 범위와 한계\n\n게임 실행 **없음**, 독립 검토자 **없음**, 정식 검증기 실행 **없음**, 연결된 저장소 변경 **없음**, Goal activation **없음**. 모든 판정의 실행 의미는 각 구현 owner의 후속 실제 시험에서 증명해야 한다. 이 스크립트는 source에서 해석한 설계 판단을 독립적으로 재심사하지 않는다.\n\n자체 검사 결과 파일은 이 명령의 출력이다. 입력 해시는 JSON receipt에 보존했고 ZIP의 해시는 별도 배포 manifest가 소유한다. 새 파일만 추가하는 패치/ZIP 무결성 검사는 포장 스크립트가 담당한다.\n\n[기계 판독 결과](DOCUMENT_VALIDATION.json) · [조사 한계](../report/LIMITATIONS.ko.md) · [핵심 보고서](../report/REVIEW.ko.md)\n'''
    READABLE.write_text(text,encoding='utf-8')
    print(json.dumps({'status':result['status'],'checks':len(checks),'passed':result['passed_count'],'failures':failures},ensure_ascii=False,indent=2))
    return result['exit_code']


if __name__=='__main__':
    sys.exit(main())
