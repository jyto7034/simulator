import json,re,hashlib
from pathlib import Path
R=Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_battle_starter_tactics';S=H/'execution/single_goal';O=Path(__file__).parent
j=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
c=j(S/'COVERAGE.json');p=j(S/'PREDECESSOR_IMPACT.json');errors=[];logs={};receipts={};counts={}
for name,b in c['receipt_bindings'].items():
 r=H/'evidence'/(name+'.json'); l=r.with_suffix('.log'); d=j(r);receipts[name]=d
 if sha(r)!=b['receipt_sha256'] or sha(l)!=b['log_sha256'] or sha(l)!=d['log_sha256']:errors.append(['receipt_bytes',name])
 if d['source_before']!=d['source_after'] or d['source_before']['sha256']!=c['source_sha256']:errors.append(['receipt_source',name])
 if d['exit_code']!=b['raw_exit_code']:errors.append(['receipt_exit',name])
 for q in d.get('command_inputs',[]):
  cp=Path(q['path']); cp=cp if cp.is_absolute() else R/cp
  if sha(H/q['archive'])!=q['sha256']:errors.append(['input_archive',name,q['path']])
  if cp.exists() and sha(cp)!=q['sha256']:errors.append(['input_current',name,q['path']])
 logs[name]=l.read_text(); counts[name]=[list(map(int,x)) for x in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',logs[name])]
rows=[]
for group,items,key in [('u06',c['validations'],'id'),('inherited',p['conditions'],'id')]:
 for v in items:
  misses=[]
  for selector,names in v.get('selector_receipts',{}).items():
   for n in names:
    if not re.search(r'^test '+re.escape(selector)+r' \.\.\. ok$',logs.get(n,''),re.M):misses.append([selector,n])
  if misses:errors.append(['selectors',group,v[key],misses])
  rows.append({'group':group,'id':v[key],'bound_selectors':len(v.get('selector_receipts',{})),'missing':misses})
r={'errors':errors,'receipt_count':len(receipts),'rows':rows,'raw_suite_results':counts,'commands':{n:d['command'] for n,d in receipts.items()}}
(O/'EVIDENCE_VERIFICATION.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({'errors':errors,'receipt_count':len(receipts),'condition_rows':len(rows),'suite_results':{k:v for k,v in counts.items() if k.startswith('integrated_')}}))
