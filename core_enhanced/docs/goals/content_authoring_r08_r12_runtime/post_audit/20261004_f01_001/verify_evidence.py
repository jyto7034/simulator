"""Independent read-only validation; writes only this audit's new evidence."""
from pathlib import Path
import hashlib,json,os,re,tarfile,tempfile,subprocess,sys
R=Path('/mnt/f/work/simulator')
G=R/'core_enhanced/docs/goals/content_authoring_r08_r12_runtime'
B=G/'execution/single_goal/corrections/f01_action_flow_20261004_001'
T=B/'target/f01_action_flow_001'
A=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
load=lambda p:json.loads(Path(p).read_text())
def write(n,d):
 with (A/n).open('x') as f:json.dump(d,f,ensure_ascii=False,indent=2);f.write('\n')
def rows(names,root=R):return [[n,(root/n).stat().st_size,sha(root/n)] for n in sorted(names)]
def digest(r):return hashlib.sha256(json.dumps(r,separators=(',',':')).encode()).hexdigest()
m=load(T/'MANIFEST.json')
assert sha(T/'MANIFEST.json')=='3a7bd83b904d1f8367590b7c2d3f4616a9cce8ca6a8a9cb0c7e0cc767d12b50a'
expected=m['source_files'];names={r[0] for r in expected}
base=load(G/'execution/single_goal/baseline/MANIFEST.json')
actual={r['path'] for r in base['files'] if (R/r['path']).is_file()}
for folder in ['core_enhanced/src','core_enhanced/tests','game_resources/data','core_enhanced/tools/runtime_architecture_final_validation','core_enhanced/tools/runtime_architecture_preservation']:
 for parent,dirs,files in os.walk(R/folder):
  dirs[:]=[d for d in dirs if d not in ('target','__pycache__')]
  actual.update(str((Path(parent)/f).relative_to(R)) for f in files if (Path(parent)/f).is_file())
actual.add('core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json')
assert actual==names,(actual-names,names-actual)
assert rows(actual)==expected
assert digest(expected)==m['source_digest']
checks={}
for group in ['RECORDS','REFERENCES']:
 rs=load(T/(group+'.json'))
 assert all((R/r['path']).stat().st_size==r['bytes'] and sha(R/r['path'])==r['sha256'] for r in rs)
 with tarfile.open(T/(group+'.tar.gz')) as ar:
  members={x.name:x for x in ar.getmembers() if x.isfile()}
  assert len(members)==len(rs) and set(members)=={r['path'] for r in rs}
  for r in rs:assert members[r['path']].size==r['bytes'] and hashlib.sha256(ar.extractfile(members[r['path']]).read()).hexdigest()==r['sha256']
 checks[group]={'files':len(rs),'live_hashes_and_archive_members_bytes_match':True,'manifest_sha256':sha(T/(group+'.json'))}
for r in m['archives_and_patches']:assert (T/r['path']).stat().st_size==r['bytes'] and sha(T/r['path'])==r['sha256']
for p,rs in [(T/'SOURCE_AFTER.tar.gz',expected),(B/'baseline/SOURCE_BEFORE.tar.gz',m['before_files'])]:
 with tarfile.open(p) as ar:
  members={x.name:x for x in ar.getmembers() if x.isfile()};assert set(members)=={r[0] for r in rs}
  for n,s,h in rs:assert members[n].size==s and hashlib.sha256(ar.extractfile(members[n]).read()).hexdigest()==h
 checks[p.name]={'files':len(rs),'membership_and_bytes_match':True}
with tempfile.TemporaryDirectory(prefix='f01-independent-restore-') as d:
 root=Path(d)
 with tarfile.open(B/'baseline/SOURCE_BEFORE.tar.gz') as ar:ar.extractall(root,filter='data')
 for rev,rs in [(False,expected),(True,m['before_files'])]:
  command=['patch',*(['-R'] if rev else []),'-p1','--fuzz=0','--batch','--input',str(T/'FORWARD.patch')]
  p=subprocess.run(command,cwd=root,capture_output=True);assert p.returncode==0,p.stderr
  (A/('restore_reverse.log' if rev else 'restore_forward.log')).write_bytes(p.stdout+p.stderr)
  found={str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()}
  assert found=={r[0] for r in rs};assert rows(found,root)==rs
checks['restoration']={'forward':True,'reverse':True,'fuzz':0,'membership_and_bytes':True}
old=load(G/'reviews/r08_r12_20261002_001/AUDIT_TARGET.json')['current']['files']
oldmap={r[0]:r for r in old};newmap={r[0]:r for r in expected}
delta=[n for n in sorted(set(oldmap)|set(newmap)) if oldmap.get(n)!=newmap.get(n)]
receipts=[]
for n in ['all_default_002','all_feature_001','check_001','event_log_check_001','clippy_001','fmt_001','release_001','public_oracle_after_001','public_cli_001','public_trace_001','pc_current_001','pc_compare_001','r05_current_002','r05_compare_002']:
 p=B/'evidence'/(n+'.json');d=load(p);log=B/'evidence'/(n+'.log')
 assert d['source_before']['files']==d['source_after']['files']==expected
 assert d['source_before']['digest']==d['source_after']['digest']==m['source_digest']
 assert d['runner_sha256']==sha(B/'run.py') and d['log_sha256']==sha(log)
 assert d['exit_code']==(101 if n=='r05_current_002' else 0)
 result={k:v for k,v in d.items() if k not in ['source_before','source_after']};result['receipt']=str(p.relative_to(R));result['receipt_sha256']=sha(p)
 if n.startswith('all_'):
  parsed=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',log.read_text())
  assert sum(int(x[1]) for x in parsed)==1684 and all(x[0]=='ok' and all(int(y)==0 for y in x[2:]) for x in parsed)
 receipts.append(result)
cov=load(B/'COVERAGE_CURRENT.json');selector_rows=[]
for row in cov['conditions']:
 for n in ['all_default_002','all_feature_001']:
  log=(B/'evidence'/(n+'.log')).read_text()
  for selector in row.get('selectors',[]):assert 'test '+selector+' ... ok' in log,(row['id'],selector,n)
 selector_rows.append({'id':row['id'],'selectors':row.get('selectors',[]),'passed_in':['all_default_002','all_feature_001']})
for s in cov['correction_selectors']:
 for n in ['all_default_002','all_feature_001']:assert 'test '+s+' ... ok' in (B/'evidence'/(n+'.log')).read_text()
# Independently parse public raw events and enforce full expected timing/counts.
def events(p):
 found={}
 def visit(v):
  if isinstance(v,dict):
   if 'time_ms' in v and 'event' in v:found[v['seq']]=v
   for x in v.values():visit(x)
  elif isinstance(v,list):
   for x in v:visit(x)
 for l in p.read_text().splitlines():visit(json.loads(l))
 return found
pub=load(B/'evidence/public_after/SUMMARY.json');before=load(B/'evidence/public_before/SUMMARY.json')
assert before['input_files']==pub['input_files'];assert all(sha(R/n)==h for n,h in pub['input_files'].items())
assert sha(Path(pub['rows'][0]['command'][0]))==pub['binary_sha256']
assert sha(B/'public_probe.py')==pub['probe_sha256']
public=[]
for r in pub['rows']:
 case=r['case'];p=B/'evidence/public_after'/(case+'.jsonl');assert sha(p)==r['log_sha256']
 ev=events(p);steps=[(v['event']['step_id'],v['time_ms']) for v in ev.values() if v['event']['type']=='AbilityStepTriggered' and v['event'].get('skill_id') in ('r08_strike','r12_boss_marker')]
 expected_steps={'early_cancel':[('marker',5051)],'cancel':[('projectile',5300),('marker',6501)],'detach':[('projectile',5300),('marker',6501),('area',6885)],'inflight_cancel':[('projectile',5300),('marker',5501)],'state_exit':[('projectile',5300),('marker',6501)]}[case]
 assert steps==expected_steps,(case,steps)
 hp=[v for v in ev.values() if v['event']['type']=='HpChanged' and v['event'].get('raw_damage')==1]
 expected_times=[] if case=='early_cancel' else [5885,6885,6885] if case=='detach' else [5885]
 assert [v['time_ms'] for v in hp]==expected_times
 assert all(v['event']['delta']==-1 and v['event']['hp_before']-v['event']['hp_after']==1 and v['event']['source_instance_id']=='a1fbb727-5065-44b3-b248-2cd9e88a484a' for v in hp)
 trace=events(B/'evidence/public_after_trace'/(case+'.jsonl'));assert trace==ev
 public.append({'case':case,'raw_sha256':sha(p),'steps':steps,'hp_times':expected_times,'event_count':len(ev),'trace_parity':True,'pass':True})
assert rows(actual)==expected
write('CURRENTNESS.json',{'manifest_sha256':sha(T/'MANIFEST.json'),'target':str(T.relative_to(R)),'source_digest':m['source_digest'],'file_count':len(expected),'currentness':'current / unchanged','independent_membership_check':True,'checks':checks,'cumulative_delta_from_prior_audit':delta,'source_unchanged_during_checks':True})
write('EVIDENCE_REUSE.json',{'source_digest':m['source_digest'],'receipts':receipts,'selectors':selector_rows,'correction_selectors':cov['correction_selectors'],'public_inputs':{'count':len(pub['input_files']),'before_after_identical':True,'all_live_sha256_match':True},'public_oracles':public,'note':'Fresh audit validations of existing raw command evidence, not new execution of the suites or public CLI.'})
print(json.dumps({'currentness':'current / unchanged','source_files':len(expected),'checks':checks,'cumulative_delta':delta,'receipts':len(receipts),'condition_selector_rows':len(selector_rows),'public_oracles':len(public)},indent=2))
