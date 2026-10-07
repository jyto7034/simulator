import json,hashlib,datetime,sys,re,statistics
from pathlib import Path
R=Path('/mnt/f/work/simulator'); O=Path(__file__).parent;B=O.parent.parent
G=R/'core_enhanced/docs/goals'; E=G/'content_authoring_resource_lifetimes/execution/single_goal/evidence'
read=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
t=read(B/'AUDIT_TARGET_006.json'); errors=[]
def ck(v,s):
 if not v:errors.append(s)
def ident(p):return {'path':str(p.relative_to(R)),'sha256':sha(p),'bytes':p.stat().st_size}
preds=[]
for label,directory,report,reg,th,rh,gh in [('R01','content_authoring_minimal_effect_contract','COMPLETION_AUDIT_007.md','r05_restoration_008','773e2127cffdfce1a496beb01db8267b87c2d4e1ec79bd399d9328de69a3d277','c2d2b41530e4e7fefeca5715de4af7b9a2340d7a5f9a0244e150e4ebe6bcf8fe','0e550d13e6f8a5a5a7f90b19cea853adbd39ca4f4482fc8851fd114fc96cf8c6'),('R02','content_authoring_pack_generation','COMPLETION_AUDIT_006.md','r05_restoration_006','fbf32708db3b45a2d6a71dd7bbf2257d766dfc02682c5a0c0c517f69d8f8f691','26192de088423250aaf7d12c10e3f86328593ea2c8779b14ad03bb8f8ef96252','05a1b51d67da50cea69f91b75bd4f9ccc7e05c7dcbe0f168f30e640a017bdff3')]:
 p=G/directory/'execution/single_goal';rt=p/'AUDIT_TARGET_008.json';rp=p/report;rr=p/'audit-evidence'/reg/'AUDIT_OUTPUTS.json';r=read(rr)
 ck(sha(rt)==th and sha(rp)==rh and sha(rr)==gh,label+' identities');ck(read(rt)['manifest']==t['manifest'],label+' equal whole manifest');ck(r['verdict']=='Complete / High',label+' verdict')
 for row in r['outputs']:
  f=R/row['path'];ck(sha(f)==row['sha256'] and f.stat().st_size==row['bytes'],label+' sealed '+row['path'])
 for stage in ['ENTRY','CLOSURE']:
  d=read(rr.parent/('CURRENTNESS_'+stage+'.json'));ck(d['status']=='current / unchanged' and not d['failures'],label+' currentness '+stage)
 preds.append({'label':label,'target':ident(rt),'report':ident(rp),'registry':ident(rr),'sealed_output_count':len(r['outputs']),'manifest_equal':True})
# Check applicability of existing independent archive and exact external inventory recomputations.
r1=G/'content_authoring_minimal_effect_contract/execution/single_goal'
ck(read(B/'AUDIT_TARGET_004.json')['external_actual']==read(r1/'AUDIT_TARGET_006.json')['external_actual'],'same56 external inventory')
for f in ['EVIDENCE_VERIFICATION.json','ATTRIBUTION_VERIFICATION.json','GUARD_VERIFICATION.json']:
 d=read(r1/'audit-evidence/r05_restoration_008'/f);ck(not d.get('errors',[]) and not d.get('failures',[]) and d.get('passed',True), 'reused '+f)
source=read(E/'IMPLEMENTATION_INPUT_001.json');sm={r['path']:r['sha256'] for r in source['files']};tm={r['path']:r['sha256'] for r in t['manifest']};ck(all(tm.get(p)==h for p,h in sm.items()),'all349source')
validation=[]
for n in range(102,117):
 p=next(x for x in E.glob('*_%03d.json'%n) if '_input' not in x.name);d=read(p);im={r['path']:r['sha256'] for r in read(E/d['input_manifest'])['files']};raw=p.with_suffix('.log');ck(d['exit_code']==0 and not d['source_changed_during_command'] and all(im.get(p)==h for p,h in sm.items()) and sha(raw)==d['log_sha256'],'final record'+str(n));counts=[tuple(map(int,x)) for x in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',raw.read_text())];validation.append({'number':n,'record':ident(p),'log':ident(raw),'source_matches':len(sm),'passed':sum(x[0] for x in counts),'failed':sum(x[1] for x in counts),'ignored':sum(x[2] for x in counts)})
# Every original source/criteria hash in exact handoff remains unchanged.
for row in read(B.parent.parent/'P2G_60_HANDOFF_001.json')['source_locations']+read(B.parent.parent/'P2G_60_HANDOFF_001.json')['external_context_locations']:ck(sha(R/row['path'])==row['sha256'],'handoff '+row['path'])
old={r['path']:r for r in read(B/'AUDIT_TARGET_005.json')['manifest']};new={r['path']:r for r in t['manifest']};added=sorted(new.keys()-old.keys());ck(len(added)==18 and not old.keys()-new.keys() and all(new[p]==r for p,r in old.items()),'18evidence-only additions')
c=E/'PREDECESSOR_CONSUMERS_001';groups=[]
for row in read(c/'RESULT.json')['commands']:
 ck(row['exit_code']==0 and sha(c/(row['name']+'.log'))==row['log_sha256'],'consumer '+row['name']);groups.append(row['name'])
for row in read(c/'RESULT.json')['files'].items():ck(sha(c/row[0])==row[1],'consumeroutput '+row[0])
# Required public privacy outcomes and real runtime counterexamples.
oracles={}
for g in ['api','combat','resource','buff','delivery','preview','replacement','source_receipt','invalid']:
 p=c/g;f=p/'RESULT.json' if (p/'RESULT.json').exists() else p/'receipt.json';d=read(f);ck(d['passed'],g+' result');oracles[g]={k:v for k,v in d.items() if isinstance(v,(bool,int,str))}
ck(oracles['api']['positive_runtime_consumers']==5 and oracles['api']['negative_compile_consumers']==6,'API privacy5/6');ck(oracles['combat']['a_damage']==18 and oracles['combat']['b_damage']==36,'A18B36');ck(oracles['replacement']['failed_attempts']==54 and oracles['replacement']['successful_replacements']==3,'replacement54/3');ck(oracles['source_receipt']['failed_replacements']==36,'source36')
for mode in ['off','on','gap']:
 for s in ['a','b']:ck((c/f'source_receipt/{mode}/{s}.ndjson').read_bytes()==(c/f'source_receipt/off/{s}.ndjson').read_bytes(),'mandatory onoffgap '+s+mode)
raw=(E/'resource_core_alltargets_103.log').read_text(); selected=[x for x in raw.splitlines() if x.startswith('test ') and x.endswith(' ... ok') and any(k in x for k in ['authoring::','authoring_','public_projection','native_clone','content_generation::','checkpoint_validation::'])];ck(len(selected)>70,'nonzero original required oracles')
(O/'REQUIRED_TEST_OBSERVATIONS.json').write_text(json.dumps({'record':'resource_core_alltargets_103.json','selected_count':len(selected),'full_passed':1410,'selected_actual_log_lines':selected,'fresh_execution':False},indent=2)+'\n')
out={'checked_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'stage':sys.argv[1],'predecessors':preds,'applicable_reuse':'R01 archive49modified+16added/349source,56exactexternal vectors and original numerical guards; R02 current pack/readers/source meaning. Own R03 source judgment separate.','source_rows':len(sm),'final_records':validation,'evidence_only_added':added,'consumer_groups':groups,'public_oracles':oracles,'errors':errors,'passed':not errors}
(O/('EVIDENCE_'+sys.argv[1]+'.json')).write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({'passed':not errors,'errors':errors,'selected_tests':len(selected),'groups':groups}))
