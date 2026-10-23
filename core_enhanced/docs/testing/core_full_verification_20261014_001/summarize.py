from pathlib import Path
import json,re,collections,hashlib
O=Path(__file__).resolve().parent
runs=[]
for p in sorted(O.glob('*.result.json')):
    r=json.loads(p.read_text()); label=p.name.removesuffix('.result.json')
    stdout=(O/r['stdout']).read_text()
    matches=re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)',stdout,re.M)
    r.update(label=label,tests=[{'test':n,'status':s} for n,s in matches],counts=dict(collections.Counter(s for _,s in matches)),summaries=re.findall(r'^test result: (.+)$',stdout,re.M))
    runs.append(r)
(O/'EXECUTION_INDEX.json').write_text(json.dumps(runs,ensure_ascii=False,indent=2)+'\n')
for r in runs: print(r['label'],r['exit_code'],r['counts'],r['summaries'])
