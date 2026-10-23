#!/usr/bin/env python3
"""Check content-wiki navigation and immutable source preservation."""
import argparse
import collections
import hashlib
import json
import os
from pathlib import Path
import re
import unicodedata
from urllib.parse import unquote

WIKI='core_enhanced/docs/content/'

def sha(value):
    return hashlib.sha256(value.encode() if isinstance(value,str) else value).hexdigest()

def semantic(text):
    text=re.sub(r'<a id="[^"]+"></a>\n?', '',text)
    text=re.sub(r'<!-- wiki-source:.*? -->\n?', '',text)
    text=re.sub(r'\[([^]\n]+)\]\(([^\s)]*)\)',r'\1',text)
    text=re.sub(r'^#{1,6}\s+', '',text,flags=re.M)
    return re.sub(r'\s+',' ',text).strip()

def anchors(text):
    ids=set(re.findall(r'<a\s+id="([^"]+)"',text))
    count=collections.Counter(); fence=False
    for line in text.splitlines():
        if line.lstrip().startswith(('```','~~~')):fence=not fence
        if fence:continue
        m=re.match(r'^#{1,6}\s+(.+?)\s*#*\s*$',line)
        if not m:continue
        s=re.sub(r'<[^>]+>','',m[1]).lower()
        s=re.sub(r'!?\[([^]]+)\]\([^)]*\)',r'\1',s)
        s=''.join(c for c in s if c in '-_ ' or c.isspace() or unicodedata.category(c)[0] in 'LNM')
        s=re.sub(r'\s','-',s)
        ids.add(s if not count[s] else s+'-'+str(count[s]))
        count[s]+=1
    return ids

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--root',type=Path,default=Path(__file__).resolve().parents[4])
    ap.add_argument('--fallback-root',type=Path)
    ap.add_argument('--migration-baseline',action='store_true')
    ap.add_argument('--report',type=Path)
    args=ap.parse_args()
    def locate(path):
        p=args.root/path
        if p.exists():return p
        if args.fallback_root:return args.fallback_root/path
        return p
    m=json.loads(locate(WIKI+'maintenance/migration-map.json').read_text())
    errors=[]; parsed={}; texts={}; check_count=0
    def read(path):
        if path not in texts:texts[path]=locate(path).read_text()
        return texts[path]
    def linkcheck(origin,target):
        nonlocal check_count
        if re.match(r'^[a-zA-Z][\w+.-]*:',target) or target.startswith('//'):return
        path,sep,anchor=target.partition('#')
        dest=os.path.normpath(os.path.join(os.path.dirname(origin),unquote(path))) if path else origin
        if not locate(dest).exists():
            errors.append(f'{origin}: missing {target}');return
        check_count+=1
        if sep and locate(dest).is_file() and dest.endswith('.md'):
            if dest not in parsed:parsed[dest]=anchors(read(dest))
            if unquote(anchor) not in parsed[dest]:errors.append(f'{origin}: missing anchor {target}')
    source_text={}
    for src,info in m['sources'].items():
        data=locate(info['snapshot']).read_bytes()
        if sha(data)!=info['sha256']:errors.append(f'changed original snapshot: {src}')
        source_text[src]=data.decode().splitlines(keepends=True)
        if len(source_text[src])!=info['line_count']:errors.append(f'line count: {src}')
        intervals=sorted((c for c in m['chunks'] if c['source']==src),key=lambda c:c['start'])
        cursor=1
        for c in intervals:
            if c['start']!=cursor:errors.append(f'coverage gap/overlap: {src} {cursor}')
            cursor=c['end']+1
            raw=''.join(source_text[src][c['start']-1:c['end']])
            if sha(raw)!=c['source_sha256']:errors.append(f'source slice mismatch: {c["id"]}')
            dest=WIKI+c['dest']
            body=read(dest)
            markers=re.findall(r'<!-- wiki-source:'+re.escape(c['id'])+r' begin -->\n(.*?)<!-- wiki-source:'+re.escape(c['id'])+r' end -->',body,re.S)
            if args.migration_baseline:
                if len(markers)!=1:
                    errors.append(f'missing/duplicate migrated block: {c["id"]}');continue
                rendered=markers[0].rstrip()+'\n'
                if sha(rendered)!=c['rendered_sha256']:errors.append(f'changed migration body: {c["id"]}')
                if semantic(raw)!=semantic(rendered):errors.append(f'non-format content changed: {c["id"]}')
                if sha(semantic(raw))!=c['semantic_sha256']:errors.append(f'semantic baseline mismatch: {c["id"]}')
        if cursor!=info['line_count']+1:errors.append(f'incomplete coverage: {src}')
    paths=sorted(set(m['pages']+[s['path'] for s in m['sources'].values()]) |
                 {p.relative_to(args.root).as_posix() for p in (args.root/WIKI).rglob('*.md')})
    preservation=locate(WIKI+'archive/preservation.json')
    archive_count=0
    if preservation.exists():
        for entry in json.loads(preservation.read_text())['entries']:
            archive_count+=1
            if sha(locate(WIKI+entry['original']).read_bytes())!=entry['sha256']:
                errors.append('changed archive original: '+entry['original'])
            if not locate(WIKI+entry['current']).is_file() or not locate(WIKI+entry['archive']).is_file():
                errors.append('missing archive/current pair: '+entry['current'])
    for path in paths:
        body=read(path);fence=False
        for line in body.splitlines():
            if line.lstrip().startswith(('```','~~~')):fence=not fence
            if fence:continue
            for target in re.findall(r'\[[^]\n]+\]\(([^\s)]*)\)',line):linkcheck(path,target)
    for r in m['redirects']:
        old=m['sources'][r['source']]['path']
        if r['old_anchor'] and r['old_anchor'] not in anchors(read(old)):
            errors.append('old anchor missing: '+old+'#'+r['old_anchor'])
        if not locate(r['destination']).exists():errors.append('redirect destination missing: '+r['destination'])
        elif r['new_anchor'] and r['new_anchor'] not in anchors(read(r['destination'])):
            errors.append('new anchor missing: '+r['destination']+'#'+r['new_anchor'])
    report={'status':'PASS' if not errors else 'FAIL','migration_baseline':args.migration_baseline,
            'sources':len(m['sources']),'source_lines':sum(s['line_count'] for s in m['sources'].values()),
            'chunks':len(m['chunks']),'pages':len(list((args.root/WIKI).rglob('*.md'))),'redirects':len(m['redirects']),
            'archive_originals':archive_count,'local_links_checked':check_count,'errors':errors}
    if args.report:
        args.report.parent.mkdir(parents=True,exist_ok=True)
        args.report.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(report,ensure_ascii=False,indent=2))
    raise SystemExit(bool(errors))

if __name__=='__main__':main()
