import pathlib,json,hashlib,re,datetime
R=pathlib.Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_runtime_measurement_profiles';S=H/'execution/single_goal';A=pathlib.Path(__file__).parent;P=R/'core_enhanced/docs/goals/core_independent_objects';load=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
p=load(S/'PREDECESSOR_IMPACT.json');old=load(P/'execution/single_goal/PREDECESSOR_IMPACT_002.json');cov=load(P/'execution/single_goal/COVERAGE_002.json');prior=load(P/'post_audit/u07_20261005_002/CONDITION_JUDGMENTS.json')
assert len(p['conditions'])==181 and len(old['conditions'])==161
for x,y in zip(p['conditions'][:161],old['conditions']):
 assert x['id']==y['id'] and x['requirement']==y['requirement'] and x.get('original_meaning_basis')==y.get('original_meaning_basis');assert sha(R/x['predecessor_row']['path'])==x['predecessor_row']['sha256']
for x,y in zip(p['conditions'][161:],cov['validations']):assert x['requirement']==y['requirement'];assert sha(R/x['predecessor_row']['path'])==x['predecessor_row']['sha256']
for x,y in zip(p['r06_acceptances'],old['r06_acceptances']):assert all(x[k]==y[k] for k in ['acceptance','expected','iva'])
assert len(p['r06_acceptances'])==31
for row in p['r06_acceptances']:
 for iva in row['iva']:assert any(x['id']==iva and x['selector_receipts'] for x in p['conditions'])
assert sha(P/'post_audit/u07_20261005_002/AUDIT_CLOSURE.json')=='d371f43c1122826347261ed68f0817e66432bbc97e85fdb6759deaf686a59cb9'
arts={};checked=[]
for f in H.glob('P2G*.json'):
 for a in load(f).get('artifacts',[]):
  digest='sha256:'+hashlib.sha256(json.dumps(a['payload'],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest();assert digest==a['ref']['Content Digest'],f;key=a['ref']['Artifact ID'];assert key not in arts or arts[key]==a;arts[key]=a;checked.append({'path':str(f.relative_to(R)),'ref':a['ref']})
freeze=load(H/'P2G_30_FREEZE_001.json')['artifacts'][0];plan=load(H/'P2G_40_PHASE_PLAN_001.json')['artifacts'][0];handoff=load(H/'P2G_60_HANDOFF_001.json')['artifacts'][0];gex=load(S/'GEX10_RESULT.json');gs={'Artifact Type':'Goal Set','Artifact ID':'GS-CMRP-001','Content Digest':'sha256:'+sha(H/'P2G_50_GOAL_SET.ko.md')}
assert handoff['payload']['Final Policy Freeze Ref']==plan['payload']['Bound Final Policy Freeze Ref']==freeze['ref'];assert handoff['payload']['Phase Plan Ref']==plan['ref'];assert handoff['payload']['Goal Set Ref']==gs;assert gex['handoff_ref']==handoff['ref'];assert len(plan['payload']['work_packages'])==5 and len(plan['payload']['integration'])==10 and len(plan['payload']['phases'])==1
assert [x['id'] for x in plan['payload']['work_packages']]==[x['id'] for x in gex['execution_nodes']]
for key in ['selection','coverage','predecessor_impact','compatibility_adaptation']:
 row=gex[key];assert sha(R/row['path'])==row['sha256']
m=load(S/'target/u07m_001/MANIFEST.json');oldfiles={x['path']:x for x in m['before_files']};changed=[{'path':x['path'],'before_sha256':oldfiles.get(x['path'],{}).get('sha256'),'after_sha256':x['sha256'],'new':x['path'] not in oldfiles} for x in m['source_files'] if x['path'] not in oldfiles or oldfiles[x['path']]['sha256']!=x['sha256']]
(A/'SOURCE_DELTA.json').write_text(json.dumps(changed,indent=2)+'\n')
out={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exact_predecessor_meanings':181,'exact_r06_meanings':31,'r06_iva_current_nonzero':True,'planning_artifacts_checked':checked,'freeze_phase_goal_handoff_gex_exact':True,'nodes':5,'integration_seams':10,'source_delta':{'modified':sum(not x['new'] for x in changed),'new':sum(x['new'] for x in changed)},'prior_complete_audit_sha256':sha(P/'post_audit/u07_20261005_002/AUDIT_CLOSURE.json'),'errors':[]}
(A/'MEANING_VERIFICATION.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in out.items() if k!='planning_artifacts_checked'}))
# Independently recalculate exported-sample population and distributions.
count=0
for family in ['playback_samples_003','lifecycle_samples_003']:
 for row in load(H/'evidence'/family/'SUMMARY.json')['rows']:
  p=pathlib.Path(row.get('sample_path',row['command'][2]));lines=p.read_text().splitlines(True);r=[json.loads(x) for x in lines];s=r[-1];assert len(r)-2==s['samples_retained'];assert s['records_seen']==s['samples_retained']+s['samples_dropped'];assert len(''.join(lines[:-1]).encode())==s['export_bytes_before_summary']
  for key,values in [('step_wall_ns',[x['wall_ns'] for x in r if x['record']=='step']),('accepted_native_bytes',[x['summary']['retained']['accepted_total_bytes'] for x in r if x['record']=='native_operation' and x['summary']['retained'] and not x['summary']['retained']['aggregation_overflowed']])]:
   v=sorted(values);n=len(v);expected={'retained_count':n,'mean':sum(v)/n if n else None,'max':v[-1] if n else None,'p95':v[(n*95+99)//100-1] if n else None};assert s[key]==expected,(row['name'],key);count+=1
(A/'STATISTICS_VERIFICATION.json').write_text(json.dumps({'distributions_recomputed':count,'population_and_export_byte_counts':36,'errors':[]},indent=2)+'\n')
