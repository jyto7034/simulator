#!/usr/bin/env python3
"""Build the local ARK wiki from authoritative Markdown and scene-guide data.

No network or third-party packages. Source documents are read, never rewritten.
"""
import argparse
from collections import Counter
import csv
import hashlib
import html
from html.parser import HTMLParser
import json
import os
from pathlib import Path
import re
import unicodedata
from urllib.parse import quote, unquote, urlencode, parse_qs

ROOT = Path(__file__).resolve().parents[3]
CATEGORIES = [
    ('abnormalities', '환상체', '조우·기믹·보상과 연결된 콘텐츠', '◇'),
    ('fragments', '스킬 파편', '직원이 사용하는 능력과 개화형', '✦'),
    ('equipment', '장비', '무기·방어구·액세서리', '⌑'),
    ('relics', '유물', '범용·전용 효과와 조합', '◈'),
    ('rules', '게임 규칙', '원정·전투·성장·저장', '≡'),
    ('principles', '설계 기준', '콘텐츠를 만들고 검토하는 기준', '⊞'),
    ('ui', '화면 상세', '입력·표시·전환과 데이터 계약', '▣'),
    ('history', '아카이브', '대체된 합의·미채택 검토안·이관 기록', '◷'),
]
STATUS = {'confirmed': '확정', 'default': '기본 적용', 'draft': '초안 채택', 'pending': '미확정', 'deferred': '보류'}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def slug(text):
    text = re.sub(r'<[^>]+>', '', text).lower()
    text = re.sub(r'!?\[([^]]+)\]\([^)]*\)', r'\1', text)
    return ''.join(c for c in text if c in '-_ ' or c.isspace() or unicodedata.category(c)[0] in 'LNM').replace(' ', '-')


def route(page, anchor=''):
    return '#' + urlencode({'page': page, **({'section': anchor} if anchor else {})}, quote_via=quote)


class Renderer:
    """The repository's Markdown subset, with exact source text retained separately."""
    def __init__(self, resolve):
        self.resolve = resolve
        self.headings = []
        self.counts = Counter()

    def inline(self, text):
        tokens = []
        def hold(s):
            tokens.append(s)
            return f'WIKITOKEN{len(tokens)-1}END'
        def restore(s):
            for _ in range(5):
                updated = re.sub(r'WIKITOKEN(\d+)END', lambda m: tokens[int(m[1])], s)
                if updated == s:
                    break
                s = updated
            return s
        text = re.sub(r'`([^`]+)`', lambda m: hold('<code>'+html.escape(m[1])+'</code>'), text)
        def link(m):
            image, label, url = m[1], m[2], m[3].strip('<>')
            target = html.escape(self.resolve(url), quote=True)
            label = restore(html.escape(label))
            if image:
                return hold(f'<img src="{target}" alt="{label}" loading="lazy">')
            return hold(f'<a href="{target}">{label}</a>')
        text = re.sub(r'(!?)\[([^\]\n]+)\]\((<[^>\n]+>|[^)\n]+)\)', link, text)
        text = re.sub(r'<br\s*/?>', lambda m: hold('<br>'), text)
        text = html.escape(text)
        text = re.sub(r'\*\*(.+?)\*\*', r'<strong>\1</strong>', text)
        text = re.sub(r'(?<!\*)\*([^*]+)\*(?!\*)', r'<em>\1</em>', text)
        text = re.sub(r'~~(.+?)~~', r'<del>\1</del>', text)
        return restore(text).replace('\\|', '|')

    def render(self, text):
        lines = re.sub(r'<!--.*?-->', '', text, flags=re.S).splitlines()
        return self.blocks(lines)

    def blocks(self, lines):
        out = []; i = 0
        def boundary(s):
            return re.match(r'^\s*(?:#{1,6} |\||>|[-*+] |\d+\. |<a |<details|</details|<summary|`{3,}|~{3,}|---+$)', s)
        while i < len(lines):
            line = lines[i]
            if not line.strip(): i += 1; continue
            a = re.fullmatch(r'\s*<a\s+id="([^"]+)"></a>\s*', line)
            if a:
                out.append('<span class="anchor" id="'+html.escape(a[1], quote=True)+'"></span>'); i += 1; continue
            if re.fullmatch(r'\s*</?details(?: open)?>\s*', line):
                out.append(line.strip()); i += 1; continue
            if re.fullmatch(r'\s*<summary>.*</summary>\s*', line):
                out.append(line.strip()); i += 1; continue
            fence = re.match(r'^\s*(`{3,}|~{3,})(.*)', line)
            if fence:
                code = []; i += 1
                while i < len(lines) and not lines[i].lstrip().startswith(fence[1]):
                    code.append(lines[i]); i += 1
                out.append('<pre><code>'+html.escape('\n'.join(code))+'</code></pre>'); i += 1; continue
            h = re.match(r'^(#{1,6})\s+(.+)', line)
            if h:
                base = slug(h[2]); n = self.counts[base]; self.counts[base] += 1
                anchor = base + (f'-{n}' if n else '')
                level = len(h[1])
                if level == 1:
                    out.append(f'<span class="anchor" id="{html.escape(anchor, quote=True)}"></span>')
                else:
                    self.headings.append({'title': h[2], 'anchor': anchor, 'level': level})
                    out.append(f'<h{level} id="{html.escape(anchor, quote=True)}">{self.inline(h[2])}</h{level}>')
                i += 1; continue
            if line.startswith('|'):
                rows = []
                while i < len(lines) and lines[i].startswith('|'):
                    # Pipes inside inline code are content, not cell separators.
                    raw = re.sub(r'`[^`]+`', lambda m: m[0].replace('|', '\\|'), lines[i].strip().strip('|'))
                    cells = [c.strip() for c in re.split(r'(?<!\\)\|', raw)]
                    if not all(re.fullmatch(r':?-+:?', c) for c in cells): rows.append(cells)
                    i += 1
                if rows:
                    out.append('<div class="table-wrap"><table><thead><tr>'+''.join('<th>'+self.inline(c)+'</th>' for c in rows[0])+'</tr></thead><tbody>')
                    out.extend('<tr>'+''.join('<td>'+self.inline(c)+'</td>' for c in row)+'</tr>' for row in rows[1:])
                    out.append('</tbody></table></div>')
                continue
            if line.startswith('>'):
                block = []
                while i < len(lines) and lines[i].startswith('>'):
                    block.append(re.sub(r'^> ?', '', lines[i])); i += 1
                out.append('<blockquote>'+self.blocks(block)+'</blockquote>'); continue
            item = re.match(r'^(\s*)([-*+]|\d+\.)\s+(.+)', line)
            if item:
                indent = len(item[1]); tag = 'ol' if item[2][0].isdigit() else 'ul'
                out.append('<'+tag+'>')
                while i < len(lines):
                    current = re.match(r'^(\s*)([-*+]|\d+\.)\s+(.+)', lines[i])
                    if not current or len(current[1]) != indent: break
                    block = [current[3]]; i += 1
                    while i < len(lines):
                        nxt = lines[i]
                        if nxt.strip() and len(nxt)-len(nxt.lstrip()) <= indent: break
                        if not nxt.strip():
                            if i+1 >= len(lines) or len(lines[i+1])-len(lines[i+1].lstrip()) <= indent: break
                        block.append(nxt[min(len(current[0])-len(current[3]), len(nxt)-len(nxt.lstrip())):]); i += 1
                    out.append('<li>'+self.blocks(block)+'</li>')
                    if i < len(lines) and not lines[i].strip(): break
                out.append('</'+tag+'>'); continue
            if re.fullmatch(r'\s*[-*_]{3,}\s*', line): out.append('<hr>'); i += 1; continue
            paragraph = [line]; i += 1
            while i < len(lines) and lines[i].strip() and not boundary(lines[i]):
                paragraph.append(lines[i]); i += 1
            out.append('<p>'+self.inline(' '.join(paragraph))+'</p>')
        return '\n'.join(out)


class Parsed(HTMLParser):
    def __init__(self, value):
        super().__init__(); self.ids = []; self.links = []; self.images = []; self.text = []
        self.feed(value)
    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if 'id' in a: self.ids.append(a['id'])
        if tag == 'a' and 'href' in a: self.links.append(a['href'])
        if tag == 'img' and 'src' in a: self.images.append(a['src'])
    def handle_data(self, data): self.text.append(data)


def build(root, ui, output, write_html=True):
    root = root.resolve(); ui = ui.resolve(); output = output.resolve()
    content = root/'core_enhanced/docs/content'; docs = root/'core_enhanced/docs'
    if not (ui/'wiki/scene-guide.json').is_file():
        raise SystemExit('UI source missing: pass --ui-root /path/to/ark/docs/ui-layout')
    paths = {}; specs = {}; source_hashes = {}
    def register(p, key, category, **extra):
        paths[p.resolve()] = key
        specs[key] = {'path': p.resolve(), 'category': category, **extra}
    for p in sorted(content.rglob('*.md')):
        rel = p.relative_to(content); parts = rel.parts; entity = ''
        category = {'relics':'relics','tags':'principles','tools':'abnormalities','principles':'principles',
                    'history':'history','review':'history','maintenance':'history','archive':'history','builds':'principles',
                    'skills':'fragments','equipment':'equipment'}.get(parts[0], 'principles')
        if parts[0] == 'abnormalities':
            category = {'README.md':'abnormalities','encounter.md':'abnormalities','fragment.md':'fragments',
                        'equipment.md':'equipment','synergies.md':'abnormalities','review.md':'history','reference.md':'history'}.get(p.name,'abnormalities')
            if len(parts)>2: entity = parts[1]
        # Explicit editorial status applies to every kind of document, not only relics.
        if '<!-- wiki-category: history -->' in p.read_text(encoding='utf-8'):
            category = 'history'
        register(p, 'content/'+rel.with_suffix('').as_posix(), category, entity=entity,
                 part=p.stem, index=(len(parts)<3 and p.name=='README.md'))
    for p in sorted((ui/'wiki').glob('*.md')):
        register(p, 'ui/'+p.stem, 'history' if p.stem in ('art-history','decision-log','reorganization','open-items') else 'ui')
    register(ui/'ui-layout.md', 'ui/index', 'ui', index=True)
    register(ui/'screen-inventory.md', 'ui/catalog-source', 'history')
    register(docs/'game_rulebook.md','rules/index','rules',index=True)
    for p in sorted((docs/'game_rules').glob('*.md')):
        register(p, 'rules/'+p.stem, 'history' if p.stem=='open-decisions' else 'rules')
    legacy = json.loads((content/'maintenance/migration-map.json').read_text())
    redirects = {}
    for r in legacy['redirects']:
        old = root/legacy['sources'][r['source']]['path']
        redirects[(str(old), r['old_anchor'])] = route(paths[(root/r['destination']).resolve()], r['new_anchor'])
    def local(path):
        return quote(os.path.relpath(path, output.parent).replace(os.sep, '/'), safe='/._-')
    def resolve(url, source):
        if re.match(r'^[a-zA-Z][\w+.-]*:',url) or url.startswith('//'): return url
        path, _, anchor = url.partition('#'); anchor = unquote(anchor)
        path = unquote(path)
        # Some source links were authored in WSL; resolve them on native Windows too.
        mount = re.match(r'^/mnt/([a-zA-Z])/(.*)$', path)
        if os.name == 'nt' and mount:
            path = mount[1] + ':/' + mount[2]
        target = (source.parent/path).resolve() if path else source.resolve()
        if target in paths: return route(paths[target], anchor)
        if (str(target), anchor) in redirects: return redirects[(str(target),anchor)]
        if target == ui/'ui-layout.html': return '#scenes'
        if target == output: return '#home'
        return local(target) + ('#'+quote(anchor) if anchor else '')
    pages = {}
    for key, spec in specs.items():
        p = spec['path']; raw = p.read_bytes(); text = raw.decode('utf-8-sig')
        source_hashes[str(p)] = sha(raw)
        title = next((s[2:] for s in text.splitlines() if s.startswith('# ')), p.stem)
        renderer = Renderer(lambda url: resolve(url, p)); rendered = renderer.render(text)
        parsed = Parsed(rendered)
        tier = re.search(r'\*\*분류:\*\* ([^\n·]+)',text)
        if spec.get('entity') and not tier:
            entity_home = content/'abnormalities'/spec['entity']/'README.md'
            tier = re.search(r'\*\*분류:\*\* ([^\n·]+)',entity_home.read_text())
        pages[key] = {k:v for k,v in spec.items() if k!='path'} | {
            'id':key,'title':title,'html':rendered,'markdown':text,'toc':renderer.headings,
            'text':' '.join(parsed.text), 'source':local(p),'sourceName':p.name,'sha256':sha(raw),
            'tier':tier[1].strip() if tier else ('도구형 환상체' if '/tools/' in key else ''),
        }
    guide_path = ui/'wiki/scene-guide.json'
    guide_raw = guide_path.read_bytes(); guide = json.loads(guide_raw)
    source_hashes[str(guide_path)] = sha(guide_raw)
    for s in guide['scenes']:
        for im in s['images']:
            image_path = ui/im['file']; assert image_path.is_file(), image_path
            im['url'] = local(image_path)
        for a in s['actions']:
            a['url'] = resolve(a['source'], ui/'wiki/scene-guide.json')
        for q in s['questions']: q['url'] = resolve(q['source'], ui/'wiki/scene-guide.json')
        s['sourceLinks'] = [resolve(s,ui/'wiki/scene-guide.json') for s in s['sources']]
    catalog_path = ui/'screen-state-inventory.csv'
    source_hashes[str(catalog_path)] = sha(catalog_path.read_bytes())
    with catalog_path.open(encoding='utf-8-sig',newline='') as f: rows = list(csv.DictReader(f))
    catalog = {}
    for r in rows:
        entry = catalog.setdefault(r['family_id'],{'id':r['family_id'],'group':r['group'],'title':r['screen'],
                                                   'presentation':r['presentation'],'states':[],'coverage':[]})
        entry['states'].append(r['state'])
        if r['art_coverage'] not in entry['coverage']: entry['coverage'].append(r['art_coverage'])
    data = {'title':'ARK 게임 위키','pages':pages,'scenes':guide['scenes'],'groups':guide['groups'],
            'statuses':STATUS,'catalog':list(catalog.values()),'catalogStates':len(rows),
            'categories':[{'id':i,'title':t,'description':d,'icon':ic} for i,t,d,ic in CATEGORIES],
            'uiUpdated':guide['updated'],'uiOriginal':local(ui/'ui-layout.html'),
            'sourceHashes':source_hashes,'catalogSource':local(catalog_path)}
    errors = []; external_missing = []; link_count = 0
    parsed = {k:Parsed(p['html']) for k,p in pages.items()}
    def check(url, origin):
        nonlocal link_count
        if url.startswith('#page='):
            params = parse_qs(url[1:]); target = params['page'][0]; anchor = params.get('section',[''])[0]
            if target not in pages: errors.append(f'{origin}: missing page {target}')
            elif anchor and anchor not in parsed[target].ids: errors.append(f'{origin}: missing section {target}#{anchor}')
        elif url.startswith('#') or re.match(r'^[a-zA-Z][\w+.-]*:',url): pass
        else:
            target = (output.parent/unquote(url.partition('#')[0])).resolve()
            if not target.exists(): external_missing.append(f'{origin}: {url}')
        link_count += 1
    for key,p in parsed.items():
        duplicate = [a for a,n in Counter(p.ids).items() if n>1]
        if duplicate: errors.append(f'{key}: duplicate anchors {duplicate}')
        for url in p.links+p.images: check(url,key)
    for s in guide['scenes']:
        for a in s['actions']: check(a['url'],s['id']+'/'+a['id'])
        for q in s['questions']: check(q['url'],s['id'])
        for url in s['sourceLinks']: check(url,s['id'])
    # Legacy source docs can contain pre-existing outside-scope broken references.
    # Internal generated page/section links are required to be valid.
    report = {'status':'PASS' if not errors else 'FAIL','pages':len(pages),'contentPages':sum(key.startswith('content/') for key in pages),'archivePages':sum(p['category']=='history' for p in pages.values()),
              'sceneGroups':len(guide['groups']),'views':len(guide['scenes']),
              'actions':sum(len(s['actions']) for s in guide['scenes']), 'catalogFamilies':len(catalog),'catalogStates':len(rows),
              'linksChecked':link_count,'errors':errors,'sourceExternalMissing':external_missing,
              'sourceHashes':source_hashes}
    output.parent.mkdir(parents=True,exist_ok=True)
    (output.parent/'build-report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    if errors:
        print(json.dumps({k:v for k,v in report.items() if k!='sourceHashes'},ensure_ascii=False,indent=2))
        raise SystemExit(1)
    if write_html:
        template = Path(__file__).with_name('template.html').read_text()
        assert template.count('__WIKI_DATA__')==1
        payload = json.dumps(data,ensure_ascii=False,separators=(',',':')).replace('<','\\u003c')
        output.write_text(template.replace('__WIKI_DATA__',payload))
    print(json.dumps({k:v for k,v in report.items() if k not in ('sourceHashes','sourceExternalMissing')},ensure_ascii=False))
    if external_missing: print(f'{len(external_missing)} pre-existing external references need review; see build-report.json')
    return data


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,default=ROOT)
    parser.add_argument('--ui-root',type=Path,default=ROOT.parents[1]/'unity projects/ark/docs/ui-layout')
    parser.add_argument('--output',type=Path,default=ROOT/'core_enhanced/docs/wiki/index.html')
    args=parser.parse_args(); build(args.root,args.ui_root,args.output)
