"""Audit-only snapshot/currentness; never rewrites historical targets."""
import hashlib, json, subprocess, sys, tarfile
from pathlib import Path
A = Path(__file__).resolve().parent
R = next(p for p in A.parents if (p/'game_server/Cargo.toml').exists())
def digest(p):
    if not p.is_file(): return None
    h=hashlib.sha256()
    with p.open('rb') as f:
        for b in iter(lambda:f.read(1024*1024),b''): h.update(b)
    return h.hexdigest()
def save(n,v): (A/n).write_text(json.dumps(v,indent=2,ensure_ascii=False)+'\n')
names=['core_support_node_activities','core_basic_relic_rewards','core_dealer_minigames','core_stress_combat']
if sys.argv[1]=='entry':
    groups={}; paths=set()
    for name in names:
        t=next((R/'core_enhanced/docs/goals'/name/'execution/single_goal/target').iterdir())
        groups[name]={k:json.loads((t/(k+'_MANIFEST.json')).read_text())['files'] for k in ['SOURCE','EVIDENCE']}
        paths.update(x['path'] for rows in groups[name].values() for x in rows)
        paths.update(str(p.relative_to(R)) for p in t.iterdir() if p.is_file() and p.suffix=='.json')
    roots=['core_enhanced/src','core_enhanced/tests','game_server/src','game_server/examples','game_resources/data','config']
    for root in roots: paths.update(str(p.relative_to(R)) for p in (R/root).rglob('*') if p.is_file())
    for name in names:
        g=R/'core_enhanced/docs/goals'/name
        paths.update(str(p.relative_to(R)) for p in g.glob('*.md'))
        paths.update(str(p.relative_to(R)) for p in g.glob('*.json'))
    for d in ['full_tests_20261015_001','test_correction_20261015_001']:
        p=A.parent/d
        paths.update(str(x.relative_to(R)) for x in p.iterdir() if x.is_file())
    paths.update(['core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md','core_enhanced/docs/game_rules/relic-reward-nodes.md','core_enhanced/docs/roadmaps/core_first/execution/CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md'])
    current={p:digest(R/p) for p in sorted(paths)}
    diffs={}
    for name,kinds in groups.items():
        diffs[name]={k:{'members':len(rows),'changed':[{'path':x['path'],'before':x['sha256'],'current':current[x['path']]} for x in rows if x['sha256']!=current[x['path']]]} for k,rows in kinds.items()}
    save('ENTRY.json',{'HEAD':subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip(),'files':current,'roots':roots})
    save('PREDECESSOR_DIFF.json',diffs)
    (A/'DIRTY_BEFORE.txt').write_bytes(subprocess.check_output(['git','status','--short','--untracked-files=normal'],cwd=R))
    # Archive current audit inputs, including evidence references; binaries and large
    # historical logs are identified by hashes and remain in their existing archives.
    sources=sorted({x['path'] for g in groups.values() for x in g['SOURCE']} | {p for p in paths if any(p.startswith(root+'/') for root in roots)})
    with tarfile.open(A/'CURRENT_SOURCE.tar.gz','w:gz') as tf:
        for p in sources:
            if (R/p).is_file(): tf.add(R/p,arcname=p,recursive=False)
    with tarfile.open(A/'CURRENT_SOURCE.tar.gz') as tf:
        verified={m.name:hashlib.sha256(tf.extractfile(m).read()).hexdigest() for m in tf if m.isfile()}
    assert all(current[p]==h for p,h in verified.items())
    save('ARCHIVE_CHECK.json',{'members':len(verified),'sha256':digest(A/'CURRENT_SOURCE.tar.gz'),'exact_bytes_match_entry':True})
    print(json.dumps({n:{k:{'members':v['members'],'changed':len(v['changed'])} for k,v in ks.items()} for n,ks in diffs.items()}))
else:
    entry=json.loads((A/'ENTRY.json').read_text()); changed=[p for p,h in entry['files'].items() if digest(R/p)!=h]
    additions=[str(p.relative_to(R)) for root in entry['roots'] for p in (R/root).rglob('*') if p.is_file() and str(p.relative_to(R)) not in entry['files']]
    save('EXIT.json',{'members':len(entry['files']),'changed':changed,'additions':additions,'unchanged':not changed and not additions})
    print(json.dumps({'changed':changed,'additions':additions})); assert not changed and not additions
