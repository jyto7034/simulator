#!/usr/bin/env python3
"""Export the existing verified wiki as portable, separately loadable site data."""
from pathlib import Path
import hashlib
import html
import importlib.util
import json
import os
import re
import shutil
from urllib.parse import quote, unquote, parse_qs
from content_tags import build_tags

SITE=Path(__file__).resolve().parents[1]
ROOT=SITE.parent
GENERATED=SITE/'.generated'
PUBLIC=SITE/'public/generated'
UI=Path(os.environ.get('ARK_UI_ROOT',str(ROOT.parents[1]/'unity projects/ark/docs/ui-layout')))
CONFIG=json.loads((SITE/'site.config.json').read_text())
BASE=CONFIG['basePath'].rstrip('/')
assert not BASE or (BASE.startswith('/') and '..' not in BASE and not BASE.startswith('//'))

def dump(path,value):
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_text(json.dumps(value,ensure_ascii=False,separators=(',',':'))+'\n')

def page_url(key):
    return '/wiki/'+quote(key,safe='/')+'/'

def relic_preview(markdown, kind, module):
    """Extract display excerpts from the current source, without separate design data."""
    if '## 유물 정의\n' in markdown:
        section=markdown.split('## 유물 정의\n',1)[1].split('\n## ',1)[0]
        fields={}
        for line in section.splitlines():
            cells=[cell.strip() for cell in line.strip('|').split('|')]
            if line.startswith('|') and len(cells)==2: fields[cells[0]]=cells[1]
        assert kind.startswith(fields['분류']), ('Relic index classification mismatch', fields['유물명'], kind, fields['분류'])
        return {'kind':kind,'summary':fields['변경 규칙'],'status':fields['설계 상태'],
                'facts':[{'label':label,'value':fields[label].replace('`','')} for label in ('대상 태그','적용 조건')]}
    body=re.split(r'^## ',markdown,maxsplit=1,flags=re.M)[1]
    body=re.sub(r'<!--.*?-->|<a\b[^>]*>\s*</a>','',body,flags=re.S)
    body=body.split('<details>',1)[0]
    paragraphs=[p.strip() for p in re.split(r'\n\s*\n',body) if p.strip()]
    def plain(value):
        rendered=module.Renderer(lambda url:url).render(value)
        return re.sub(r'\s+',' ',' '.join(module.Parsed(rendered).text)).strip()
    effect=re.search(r'^- 효과:\s*(.+(?:\n[ \t]+[^\n]+)*)',body,re.M)
    summary=effect[1] if effect else next((p for p in paragraphs[1:] if not p.startswith(('>','#','-','|','`','출발 콘텐츠:'))),None)
    if not summary:raise ValueError('유물 효과 요약을 추출할 수 없습니다: '+markdown.splitlines()[0])
    status=next((p for p in paragraphs if p.startswith('> 상태:')),None)
    if status:
        status=re.sub(r'^>\s*','',status,flags=re.M).removeprefix('상태:').strip()
    else:
        status=next((line[2:] for line in body.splitlines() if line.startswith('- ') and ('미정:' in line or '제안 상태' in line)), '세부 합의·제안·미정 항목은 상세 문서를 참고하세요.')
    facts=[]
    for line in body.splitlines():
        if not line.startswith('|'):continue
        cells=[c.strip() for c in line.strip('|').split('|')]
        if len(cells)==2 and cells[0] in ('희귀도','호환','호환 방향','요구 태그','적용 태그','발동 조건','지속 시간'):
            facts.append({'label':plain(cells[0]),'value':plain(cells[1])})
    return {'kind':kind,'summary':plain(summary),'status':plain(status),'facts':facts[:3]}

def document_preview(source, module):
    text=re.sub(r'<!--.*?-->|<a\b[^>]*>\s*</a>','',source['markdown'],flags=re.S).split('<details>',1)[0]
    # Fragment/equipment introductions mostly contain navigation and policy links.
    if source.get('part') in ('fragment','equipment') and re.search(r'^## ',text,re.M):
        text=re.split(r'^## ',text,maxsplit=1,flags=re.M)[1]
        text=text.partition('\n')[2]
    def plain(value):
        return re.sub(r'\s+',' ',' '.join(module.Parsed(module.Renderer(lambda u:u).render(value)).text)).strip()
    paragraphs=[p.strip() for p in re.split(r'\n\s*\n',text) if p.strip()]
    candidates=[]
    for p in paragraphs:
        if p.startswith(('#','>','|','```','<','**분류:','**현재 진행:')):continue
        without_links=re.sub(r'\[[^\]]+\]\([^)]+\)','',p).strip(' ·\n-')
        if not without_links:continue
        if p.startswith(('본문의 공명·침식','각 문서의 상태','이관 대응표','관련 유물:','출발 콘텐츠:','현재 파편:')):continue
        value=plain(p)
        if len(value)>25:candidates.append(value)
    summary=candidates[0] if candidates else '문서에서 다루는 내용: '+', '.join(h['title'] for h in source['toc'][:4])
    if len(summary)>700:summary=summary[:700].rsplit(' ',1)[0]+'…'
    status=next((p for p in paragraphs if p.startswith('> 상태:')),'')
    status=plain(re.sub(r'^>\s*','',status,flags=re.M).removeprefix('상태:'))
    facts=[]
    for line in text.splitlines():
        if not line.startswith('|'):continue
        cells=[c.strip() for c in line.strip('|').split('|')]
        if len(cells)==2 and plain(cells[0]) in ('분류','희귀도','rarity','compatibility','equip_limit','역할','주된 역할','호환','발동 조건','지속 시간'):
            facts.append({'label':plain(cells[0]),'value':plain(cells[1])})
            if len(facts)==3:break
    if not facts:
        facts=[{'label':'문서 주제','value':' · '.join(h['title'] for h in source['toc'][:3])}] if source['toc'] else []
    return {'kind':source.get('tier',''),'summary':summary,'status':status,'facts':facts}

def main():
    GENERATED.mkdir(parents=True,exist_ok=True)
    spec=importlib.util.spec_from_file_location('ark_wiki_source',ROOT/'core_enhanced/tools/wiki/build.py')
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    original=module.build(ROOT,UI,GENERATED/'source/index.html',write_html=False)
    archived={key for key,p in original['pages'].items() if p['category']=='history'}
    def published_url(key):
        return '/wiki/archive/'+quote(key,safe='/')+'/' if key in archived else page_url(key)
    tags=build_tags(original['pages'])
    if PUBLIC.exists():
        assert not PUBLIC.is_symlink()
        shutil.rmtree(PUBLIC)
    PUBLIC.mkdir(parents=True)
    packed={};missing=[]
    def pack(path):
        path=path.resolve()
        if path in packed:return packed[path]
        if path.is_dir():
            digest=hashlib.sha256(path.relative_to(UI).as_posix().encode()).hexdigest()[:16]
            packed[path]='/generated/directories/'+digest+'/index.html'
            entries=[(p.relative_to(path).as_posix(),pack(p)) for p in sorted(path.rglob('*')) if p.is_file()]
            target=PUBLIC/'directories'/digest/'index.html';target.parent.mkdir(parents=True,exist_ok=True)
            target.write_text('<!doctype html><html lang="ko"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>이전 자료 목록</title><body><h1>이전 자료 목록</h1><ul>'+''.join('<li><a href="'+html.escape(BASE+url)+'">'+html.escape(name)+'</a></li>' for name,url in entries)+'</ul></body></html>')
            return packed[path]
        if not path.is_file():
            missing.append(str(path));return '/about/'
        # Only explicit links from the existing source documents are exported.
        raw=path.read_bytes();digest=hashlib.sha256(raw).hexdigest()[:16]
        group='images' if path.suffix.lower() in ('.png','.jpg','.jpeg','.webp','.gif','.svg') else 'sources'
        rel=Path(group)/digest/path.name;dst=PUBLIC/rel
        dst.parent.mkdir(parents=True,exist_ok=True);dst.write_bytes(raw)
        packed[path]='/generated/'+quote(rel.as_posix(),safe='/')
        return packed[path]
    def convert(url):
        if url.startswith('#page='):
            q=parse_qs(url[1:]);result=published_url(q['page'][0])
            if q.get('section'):result+='#'+quote(q['section'][0])
            return result
        if url.startswith('#'):
            p=parse_qs(url[1:])
            if p.get('scene'):return '/scenes/'+p['scene'][0]+'/'
            return {'#home':'/','#scenes':'/scenes/','#catalog':'/screens/','#about':'/about/'}.get(url,url)
        if re.match(r'^[a-zA-Z][\w+.-]*:',url) or url.startswith('//'):return url
        name,sep,anchor=url.partition('#')
        target=(GENERATED/'source'/unquote(name)).resolve()
        if target==ROOT/'core_enhanced/docs/wiki/index.html':return '/'
        if target==UI/'ui-layout.html':return '/scenes/'
        return pack(target)+('#'+anchor if sep else '')
    def href(url):
        result=convert(url)
        return BASE+result if result.startswith('/') and not result.startswith('//') else result
    relic_index=original['pages']['content/relics/README']['markdown']
    relic_kinds={Path(name).stem:kind.strip() for name,kind in re.findall(r'^\| \[[^\]]+\]\(([^)]+\.md)\) \| ([^|\n]+) \|[ \t]*$',relic_index,re.M)}
    pages={};metadata=[];search=[]
    for key,source in original['pages'].items():
        p=dict(source)
        if key in tags:
            p['tags']=[tag for tag in tags[key] if tag['relation']!='topic']
            p['classifications']=[tag['label'] for tag in tags[key] if tag['relation']=='topic']
        p['html']=re.sub(r'\b(href|src)="([^"]+)"',lambda m:m[1]+'="'+html.escape(href(html.unescape(m[2])),quote=True)+'"',p['html'])
        p['source']=href(p['source']);p['path']=published_url(key)
        p['archived']=key in archived
        if p['archived']:
            p['legacyPath']=page_url(key)
        origin=re.search(r'<!-- wiki-archive-of: ([^\s]+) -->',source['markdown'])
        if origin:
            p['currentPath']=published_url(origin[1])
        if p['category'] == 'relics' and not p.get('index'):
            p['relic']=relic_preview(source['markdown'],relic_kinds[key.rsplit('/',1)[-1]],module)
        else:
            p['preview']=document_preview(source,module)
        # Keep the exact Markdown as a downloadable source, not in the JS payload.
        assert hashlib.sha256(source['markdown'].encode()).hexdigest()==source['sha256'],key
        p.pop('markdown');fulltext=p.pop('text')
        p['description']=re.sub(r'\s+',' ',fulltext).strip()[:170]
        pages[key]=p
        metadata.append({k:v for k,v in p.items() if k not in ('html','toc')})
        search.append({'id':key,'title':p['title'],'category':p['category'],'path':p['path'],'text':fulltext})
        dump(PUBLIC/'pages'/Path(key+'.json'),p)
    scenes=original['scenes']
    for s in scenes:
        for im in s['images']:im['url']=href(im['url'])
        for a in s['actions']:a['url']=href(a['url'])
        for q in s['questions']:q['url']=href(q['url'])
        s['sourceLinks']=[href(u) for u in s['sourceLinks']]
    meta={k:original[k] for k in ('title','groups','statuses','categories','catalog','catalogStates','uiUpdated')}
    meta.update({'pages':metadata,'scenes':scenes,'catalogSource':href(original['catalogSource']),'config':CONFIG})
    if missing:raise RuntimeError('Unpackaged local references: '+str(missing))
    parsed={key:module.Parsed(p['html']) for key,p in pages.items()}
    links=[u for p in parsed.values() for u in p.links]
    links += [a['url'] for s in scenes for a in s['actions']]
    links += [q['url'] for s in scenes for q in s['questions']]
    links += [u for s in scenes for u in s['sourceLinks']]
    by_path={p['path']:key for key,p in pages.items()}
    for url in links:
        if not url.startswith(BASE+'/wiki/'):continue
        target,_,anchor=url[len(BASE):].partition('#')
        key=by_path.get(unquote(target))
        assert key in parsed,('missing exported page',url)
        assert not anchor or unquote(anchor) in parsed[key].ids,('missing exported anchor',url)
    # Ensure the published data has no browser dependencies on the original machine.
    for p in pages.values():
        for u in re.findall(r'(?:href|src)="([^"]+)"',p['html']):
            assert not u.startswith(('file:','../','/mnt/')),u
    dump(PUBLIC/'meta.json',meta)
    dump(PUBLIC/'search.json',[p for p in search if p['id'] not in archived])
    dump(PUBLIC/'archive-search.json',[p for p in search if p['id'] in archived])
    dump(GENERATED/'pages.json',pages)
    report={'status':'PASS','pages':len(pages),'actions':sum(len(s['actions']) for s in scenes),
            'taggedPages':len(tags),'tagAssignments':sum(map(len,tags.values())),
            'images':sum(len(s['images']) for s in scenes),'catalogFamilies':len(meta['catalog']),
            'sourceHashes':{p['id']:p['sha256'] for p in metadata},'packedFiles':len(packed),
            'packedBytes':sum(p.stat().st_size for p in packed if p.is_file()),
            'legacySourceHashes':original['sourceHashes']}
    dump(GENERATED/'content-report.json',report)
    print(f'Exported {len(pages)} pages, {report["actions"]} actions and {len(packed)} portable assets.')

if __name__=='__main__':main()
