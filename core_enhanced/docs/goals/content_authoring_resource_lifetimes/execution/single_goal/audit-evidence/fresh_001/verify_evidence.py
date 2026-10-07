import datetime, hashlib, json, pathlib, re, sys
from verify_currentness import ROOT, BASE, OUT, TARGET, digest

read=lambda p:json.loads(p.read_text())
errors=[]
def ck(condition,label):
    if not condition:errors.append(label)
def ident(p):return {'path':str(p.relative_to(ROOT)),'bytes':p.stat().st_size,'sha256':digest(p)}

target=read(TARGET); goals=ROOT/'core_enhanced/docs/goals'; evidence=BASE/'evidence'
specs=[
('R01','content_authoring_minimal_effect_contract','008','007','r05_restoration_008','773e2127cffdfce1a496beb01db8267b87c2d4e1ec79bd399d9328de69a3d277','c2d2b41530e4e7fefeca5715de4af7b9a2340d7a5f9a0244e150e4ebe6bcf8fe','0e550d13e6f8a5a5a7f90b19cea853adbd39ca4f4482fc8851fd114fc96cf8c6'),
('R02','content_authoring_pack_generation','008','006','r05_restoration_006','fbf32708db3b45a2d6a71dd7bbf2257d766dfc02682c5a0c0c517f69d8f8f691','26192de088423250aaf7d12c10e3f86328593ea2c8779b14ad03bb8f8ef96252','05a1b51d67da50cea69f91b75bd4f9ccc7e05c7dcbe0f168f30e640a017bdff3'),
('R03','content_authoring_tool_ports','006','005','r05_restoration_005','9333a90ce31a07448ea1975fd2c1534ee9cc961d90dc13846a913ac504042ed9','36d76874de7767a0184418b96abd85a3dbe484390b9d2996631e704e75fd466b','a04960bf9ba8711c80ddb07f00864a88ec64fa76824c2c5444c3a914926d9391'),
('R04','content_authoring_runtime_facts','005','003','r05_restoration_003','9d4be8c4371fb2967f1fa39facff9a457cfc6c6c2a69f4daefa25decd4a0f3c9','eb648d01f9a7509653bfcdc1990cd91ce4bb55df4bcf6c68e62fdf024fd3217d','b0db9e1cbf19633397674aa6fb20eaf28ce2c833dbcaee51abbdacfdf6e1aafd')]
predecessors=[]
for label,topic,tn,rn,folder,th,rh,gh in specs:
    b=goals/topic/'execution/single_goal';tp=b/f'AUDIT_TARGET_{tn}.json';rp=b/f'COMPLETION_AUDIT_{rn}.md';gp=b/'audit-evidence'/folder/'AUDIT_OUTPUTS.json'
    registry=read(gp)
    ck(digest(tp)==th and digest(rp)==rh and digest(gp)==gh,label+' exact identities')
    ck(read(tp)['manifest']==target['manifest'],label+' complete identical manifest')
    ck(registry['verdict']=='Complete / High',label+' actual verdict')
    checked=[]
    for row in registry['outputs']:
        p=ROOT/row['path'];ck(digest(p)==row['sha256'] and p.stat().st_size==row['bytes'],label+' output '+row['path']);checked.append(ident(p))
    current=[]
    for phase in ['ENTRY','CLOSURE']:
        p=gp.parent/f'CURRENTNESS_{phase}.json';d=read(p)
        ck(d.get('status')=='current / unchanged' and not d['failures'],label+' '+phase);current.append(ident(p))
    listed={str((ROOT/r['path']).resolve()) for r in registry['outputs'] if str(ROOT/r['path']).startswith(str(gp.parent))}
    actual={str(p.resolve()) for p in gp.parent.rglob('*') if p.is_file() and p!=gp}
    ck(listed==actual,label+' exact audit output membership')
    predecessors.append({'label':label,'target':ident(tp),'report':ident(rp),'registry':ident(gp),'outputs':checked,'currentness':current})

# Reuse independent calculations only after exact input and result identity checks.
r1=goals/'content_authoring_minimal_effect_contract/execution/single_goal'
r3=goals/'content_authoring_tool_ports/execution/single_goal'
reuse=[]
for p in [r1/'audit-evidence/r05_restoration_008/ATTRIBUTION_VERIFICATION.json',r1/'audit-evidence/r05_restoration_008/EVIDENCE_VERIFICATION.json',r1/'audit-evidence/r05_restoration_008/GUARD_VERIFICATION.json',r3/'audit-evidence/r05_restoration_005/COST_VERIFICATION.json',r3/'audit-evidence/r05_restoration_005/RESOURCE_OUTPUT_VERIFICATION.json']:
    d=read(p);ck(not d.get('errors',[]) and not d.get('failures',[]) and d.get('passed',True),'reused '+p.name);reuse.append(ident(p))
old_r04=goals/'content_authoring_runtime_facts/execution/single_goal/AUDIT_TARGET_003.json'
ck(read(old_r04)['external_actual']==read(r1/'AUDIT_TARGET_006.json')['external_actual'],'56 historical external inventory applicability')

sources=read(evidence/'IMPLEMENTATION_INPUT_001.json')['files'];source_map={r['path']:r['sha256'] for r in sources};manifest={r['path']:r['sha256'] for r in target['manifest']}
ck(len(sources)==349 and all(manifest.get(p)==h for p,h in source_map.items()),'349 current source rows')
executions=[]
for n in range(102,117):
    p=next(evidence.glob(f'*_{n:03d}.json'));d=read(p);im={r['path']:r['sha256'] for r in read(evidence/d['input_manifest'])['files']};raw=p.with_suffix('.log')
    ck(d['exit_code']==0 and not d['source_changed_during_command'],'execution '+str(n))
    ck(all(im.get(p)==h for p,h in source_map.items()),'all349 input applicability '+str(n));ck(digest(raw)==d['log_sha256'],'raw log '+str(n))
    counts=[tuple(map(int,x)) for x in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',raw.read_text())]
    executions.append({'number':n,'record':ident(p),'raw_log':ident(raw),'source_matches':349,'command':d['command'],'counts':counts,'exit_code':d['exit_code']})
raw=(evidence/'resource_core_alltargets_103.log').read_text()
selected=[x for x in raw.splitlines() if x.startswith('test ') and x.endswith(' ... ok') and ('game::world::tests::combat::resource_lifetimes::' in x or 'game::resource::' in x)]
ck(sum(x[0] for x in executions[1]['counts'])==1410,'1410 raw count');ck(sum('world::tests::combat::resource_lifetimes' in x for x in selected)==17,'world17');ck(sum('resource::store::tests' in x for x in selected)==24,'store24')
(OUT/'REQUIRED_TEST_OBSERVATIONS.json').write_text(json.dumps({'source':ident(evidence/'resource_core_alltargets_103.log'),'selected_actual_lines':selected,'count':len(selected),'fresh_execution':False},indent=2)+'\n')
old={r['path']:r for r in read(BASE/'AUDIT_TARGET_001.json')['manifest']};new={r['path']:r for r in target['manifest']};added=sorted(new.keys()-old.keys())
ck(len(added)==18 and not old.keys()-new.keys() and all(new[p]==r for p,r in old.items()),'18 documentary additions only')
result={'stage':sys.argv[1],'checked_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'predecessors':predecessors,'reused_independent_calculations':reuse,'executions':executions,'documentary_target_additions':added,'errors':errors,'passed':not errors}
(OUT/f'EVIDENCE_{sys.argv[1].upper()}.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'passed':not errors,'errors':errors,'predecessor_outputs':sum(len(x['outputs']) for x in predecessors),'selected_test_count':len(selected),'final_source_matches':349,'final_executions':len(executions)},indent=2))
