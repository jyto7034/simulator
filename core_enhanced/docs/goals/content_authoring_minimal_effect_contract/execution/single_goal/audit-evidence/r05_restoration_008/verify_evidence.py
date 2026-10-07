import json, hashlib, re, tarfile
from pathlib import Path
R=Path('/mnt/f/work/simulator');E=R/'core_enhanced/docs/goals/content_authoring_resource_lifetimes/execution/single_goal/evidence';O=Path(__file__).parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
read=lambda p:json.loads(p.read_text())
target=read(R/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/AUDIT_TARGET_008.json');tm={r['path']:r for r in target['manifest']}
source=read(E/'IMPLEMENTATION_INPUT_001.json');sm={r['path']:r['sha256'] for r in source['files']};errors=[];results=[]
for path,h in sm.items():
 if sha(R/path)!=h:errors.append(['source',path])
for number in range(102,117):
 candidates=[p for p in E.glob('*_'+str(number)+'.json') if '_input' not in p.name]
 assert len(candidates)==1,(number,candidates)
 p=candidates[0];d=read(p);inp=read(E/d['input_manifest']);rows=inp['files'];mismatch=[r['path'] for r in rows if r['path'] in sm and r['sha256']!=sm[r['path']]];missing=sorted(set(sm)-{r['path'] for r in rows});log=p.with_suffix('.log');log_valid=sha(log)==d['log_sha256'];raw=log.read_text();counts=[tuple(map(int,m)) for m in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',raw)]
 if mismatch or missing or not log_valid or d['exit_code']!=0 or d['source_changed_during_command']:errors.append(['validation',p.name,mismatch,missing])
 results.append({'record':p.name,'record_sha256':sha(p),'input_rows':len(rows),'compared_source_rows':len(sm),'source_mismatches':mismatch,'missing_source_rows':missing,'log_sha256_valid':log_valid,'exit_code':d['exit_code'],'counts_from_raw_log':counts,'passed_from_raw_log':sum(x[0] for x in counts)})
# Independently check archive membership/content and delta ownership, without extraction.
a=read(E/'ATTRIBUTION_001.json');arc={}
with tarfile.open(E/'SOURCE_AFTER_001.tar.gz') as t:
 for m in t.getmembers():
  if m.isfile():arc[m.name]=hashlib.sha256(t.extractfile(m).read()).hexdigest()
archive_missing=[p for p in sm if p not in arc];archive_changed=[p for p in sm if p in arc and arc[p]!=sm[p]]
if archive_missing or archive_changed:errors.append(['archive',archive_missing,archive_changed])
old=read(R/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/AUDIT_TARGET_006.json');inventories=[]
for ext in old['external_actual']:
 directory=R/ext['directory'];want={r['path'] for r in ext['files']};actual={str(p.relative_to(directory)) for p in directory.rglob('*') if p.is_file()};missing=sorted(want-actual);extra=sorted(actual-want)
 not_flat=[str((directory/r['path']).relative_to(R)) for r in ext['files'] if str((directory/r['path']).relative_to(R)) not in tm]
 if missing or extra or not_flat:errors.append(['inventory',ext['directory'],missing,extra,not_flat])
 inventories.append({'directory':ext['directory'],'expected':len(want),'actual':len(actual),'missing':missing,'extra':extra,'not_flat':not_flat})
supp=read(E/'PREDECESSOR_CURRENTNESS_SUPPLEMENT_001.json');sealed=[]
for row in supp['sealed_audit_files']:
 p=R/row['path'];ok=sha(p)==row['sha256'] and row['path'] in tm
 if not ok:errors.append(['sealed',row['path']])
 sealed.append({'path':row['path'],'verified':ok})
# Current public results: check command logs and original mandatory observations.
c=E/'PREDECESSOR_CONSUMERS_001';cons=read(c/'RESULT.json');consumer_rows=[]
for r in cons['commands']:
 ok=r['exit_code']==0 and sha(c/(r['name']+'.log'))==r['log_sha256'];consumer_rows.append({'name':r['name'],'valid_command_log':ok})
 if not ok:errors.append(['consumer',r['name']])
combat=read(c/'combat/RESULT.json');assert combat['a_damage']==18 and combat['b_damage']==36 and combat['accepted_steps_per_run']==13 and combat['receipts_per_run']==15
for g,damage in [('a',18),('b',36)]:
 base=read(c/f'combat/{g}_off/RESULT.json');receipt=(c/f'combat/{g}_off/receipts.ndjson').read_bytes();assert base['is_complete'] and base['failure'] is None
 for mode in ['on','gap']:
  assert read(c/f'combat/{g}_{mode}/RESULT.json')==base
  assert (c/f'combat/{g}_{mode}/receipts.ndjson').read_bytes()==receipt
res=read(c/'resource/RESULT.json');assert res['amounts']=={'a':17,'b':31}
for g,n in [('a',17),('b',31)]:
 w=res['witnesses'][g]['first'];assert w['change']['after']-w['change']['before']==n;assert w['source']['authored']['site']['effect_id']=='resource_gain';assert w['source']['proc_origin']['activation_source']['Item']['item_instance_id']
 for mode in ['on','gap']:
  assert read(c/f'resource/{g}_{mode}/RESULT.json')==read(c/f'resource/{g}_off/RESULT.json')
  assert (c/f'resource/{g}_{mode}/receipts.ndjson').read_bytes()==(c/f'resource/{g}_off/receipts.ndjson').read_bytes()
# Diagnostic receipts intentionally include nonzero failures; expected success is refusal.
diag=read(E/'PREDECESSOR_DIAGNOSTIC_001/receipt.json');assert len(diag['commands'])==10
limits=read(E/'TYPED_RESOURCE_LIMITS_003.json');assert limits['passed'] and limits['normal_replay_matches'] and len(limits['cases'])==8
out={'source_rows':len(sm),'validation_records':results,'archive_members':len(arc),'archive_missing':archive_missing,'archive_changed':archive_changed,'modified_count':len(a['modified']),'added_count':len(a['added']),'old_external_inventory':inventories,'sealed_audit_files':sealed,'consumer_groups':consumer_rows,'recomputed_oracles':{'combat_damage':[18,36],'trace_on_off_gap_receipt_parity':True,'resonance_amounts':[17,31],'proc_copy_source_present':True,'diagnostic_commands':len(diag['commands']),'typed_boundary_cases':len(limits['cases'])},'errors':errors,'passed':not errors}
(O/'EVIDENCE_VERIFICATION.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({'passed':out['passed'],'source_rows':len(sm),'validation_records':len(results),'archive_members':len(arc),'external_inventories':len(inventories),'consumer_groups':len(consumer_rows),'errors':errors},indent=2))
