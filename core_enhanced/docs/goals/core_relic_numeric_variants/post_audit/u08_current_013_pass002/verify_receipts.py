import json,hashlib,re,datetime,collections
from pathlib import Path
H=Path('core_enhanced/docs/goals/core_relic_numeric_variants');A=H/'post_audit/u08_current_013_pass002';C=json.loads((H/'evidence/finalizer_ready013/CONFIG_READY_013.json').read_text());bad=[];cache={};passed={};summary=[];refs=set();assertions=0;claims=0

def check(ok,what):
 if not ok:bad.append(what)
def ref(r):
 p=r['path']
 if p not in cache:cache[p]=Path(p).read_bytes()
 b=cache[p];check(hashlib.sha256(b).hexdigest()==r['sha256'],['hash',p]);check(len(b)==r['bytes'],['bytes',p]);refs.add(p);return b

def ptr(v,p):
 for k in p.lstrip('/').split('/') if p else []:
  k=k.replace('~1','/').replace('~0','~');v=v[int(k)] if isinstance(v,list) else v[k]
 return v
for r in C['receipts']:
 x=json.loads(ref(r['receipt']));raw=ref(r['raw_log']).decode();check(x['command']==r['command'],['command',r['id']]);check(x['source_before']==x['source_after'],['source_changed',r['id']]);check(x['source_before']==C['source'],['source_mismatch',r['id']]);check(x['exit_code']==(101 if r['kind']=='r05' else 0),['exit',r['id']]);check(x['log_sha256']==r['raw_log']['sha256'],['raw_binding',r['id']]);names=set(re.findall(r'^test ([^\s]+) \.\.\. ok$',raw,re.M));names.update(re.findall(r'^test ([^\s]+) \.\.\. (?!ok$)(?:(?!^test ).)*?^ok$',raw,re.M|re.S));passed[r['id']]=names
 check(set(r['required_selectors'])<=names,['missing_selector',r['id'],sorted(set(r['required_selectors'])-names)])
 total=sum(int(n) for n in re.findall(r'test result: ok\. (\d+) passed;',raw))
 if r['expected_passed_count'] is not None:check(total==r['expected_passed_count'],['pass_count',r['id'],total,r['expected_passed_count']])
 summary.append({'id':r['id'],'exit':x['exit_code'],'passed':total,'named_passes':len(names)})

def walk(v):
 global assertions,claims
 if isinstance(v,dict):
  if 'format' in v and 'pointer' in v:
   b=ref(v['artifact'])
   if v['format']=='json_document':x=json.loads(b)
   else:x=next(json.loads(line) for line in b.decode().splitlines() if line.startswith('{') and 'instrumented' in line)
   check(ptr(x,v['pointer'])==v['equals'],['assertion',v['artifact']['path'],v['pointer']]);assertions+=1
  if 'claim' in v and 'receipt_ids' in v and 'selectors' in v:
   claims+=1;names=set().union(*(passed[r] for r in v['receipt_ids']));check(set(v['selectors'])<=names,['claim_selector',v['claim'][:100],sorted(set(v['selectors'])-names)])
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
walk(C)
artifact_checks=0
for k,v in C['artifacts'].items():
 x=json.loads(ref(v['ref']))
 for t in v['checks']:check(ptr(x,t['pointer'])==t['equals'],['artifact',k,t['pointer']]);artifact_checks+=1
quiescence=[]
for q in ['pc','r05']:
 p=json.loads(ref(C['functional_quiescence'][q]));o=json.loads(ref(p['original_observation']));check(p['functional_receipts']==[x['receipt'] for x in o['functional_receipts']],['projection_refs',q]);check(p['recorded_at_utc']==o['recorded_at_utc'],['projection_time',q]);check(p['active_input_writers']==o['input_writers_active']==0,['writers',q]);check(p['active_functional_runners']==o['functional_runners_active']==0,['runners',q]);end=datetime.datetime.fromisoformat(o['recorded_at_utc']);check(set(x['id'] for x in o['functional_receipts'])==set(x['id'] for x in C['receipts'])-set(C['performance_receipt_ids']),['functional_roles',q])
 for x in o['functional_receipts']:
  r=json.loads(ref(x['receipt']));ref(x['raw']);finish=datetime.datetime.fromisoformat(r['started_at_utc'])+datetime.timedelta(seconds=r['duration_seconds']);check(finish<end,['functional_overlaps',q,x['id']]);check(r['exit_code']==0,['functional_exit',q,x['id']])
 measurement=json.loads(ref(next(r['receipt'] for r in C['receipts'] if r['id']==q)));check(end<datetime.datetime.fromisoformat(measurement['started_at_utc']),['measurement_precedes_quiescence',q]);quiescence.append({'phase':q,'functional_count':len(o['functional_receipts']),'recorded_at_utc':o['recorded_at_utc']})
out={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source':C['source']['sha256'],'receipts':summary,'claims_checked':claims,'assertions_checked':assertions,'artifact_checks':artifact_checks,'unique_refs':len(refs),'quiescence':quiescence,'failures':bad};(A/'RECEIPTS_INDEPENDENT.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({k:v for k,v in out.items() if k!='receipts'}))
