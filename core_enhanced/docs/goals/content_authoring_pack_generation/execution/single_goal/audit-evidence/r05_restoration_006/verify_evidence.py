import json,hashlib,re,datetime,sys,difflib,tarfile
from pathlib import Path
R=Path('/mnt/f/work/simulator'); O=Path(__file__).parent
B=R/'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal'
P=R/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal'
E=R/'core_enhanced/docs/goals/content_authoring_resource_lifetimes/execution/single_goal/evidence'
read=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
t=read(B/'AUDIT_TARGET_008.json'); tm={x['path']:x for x in t['manifest']}; failures=[]
def check(ok,label):
 if not ok: failures.append(label)
def sealrow(p):return {'path':str(p.relative_to(R)),'sha256':sha(p),'bytes':p.stat().st_size}
reg=P/'audit-evidence/r05_restoration_008/AUDIT_OUTPUTS.json';d=read(reg)
check(sha(reg)=='0e550d13e6f8a5a5a7f90b19cea853adbd39ca4f4482fc8851fd114fc96cf8c6','R01 registry identity')
check(d['verdict']=='Complete / High' and d['currentness']=='current / unchanged','R01 verdict')
check(sha(P/'AUDIT_TARGET_008.json')=='773e2127cffdfce1a496beb01db8267b87c2d4e1ec79bd399d9328de69a3d277','R01 target identity')
check(read(P/'AUDIT_TARGET_008.json')['manifest']==t['manifest'],'R01 same complete target member vector')
outputs=[]
for row in d['outputs']:
 p=R/row['path'];ok=sha(p)==row['sha256'] and p.stat().st_size==row['bytes'];check(ok,row['path']);outputs.append(dict(row,verified=ok))
check(sha(P/'COMPLETION_AUDIT_007.md')=='c2d2b41530e4e7fefeca5715de4af7b9a2340d7a5f9a0244e150e4ebe6bcf8fe','R01 report identity')
for stage in ['ENTRY','CLOSURE']:
 q=read(P/f'audit-evidence/r05_restoration_008/CURRENTNESS_{stage}.json');check(q['status']=='current / unchanged' and not q['failures'] and q['manifest_rows']==12969,'R01 '+stage)
r01ev=read(P/'audit-evidence/r05_restoration_008/EVIDENCE_VERIFICATION.json')
check(r01ev['passed'] and not r01ev['errors'],'R01 reused recomputations')
old=read(B/'AUDIT_TARGET_006.json');old1=read(P/'AUDIT_TARGET_006.json')
check(old['external_actual']==old1['external_actual'],'identical external inventories to recomputed R01')
check(len(r01ev['old_external_inventory'])==len(old['external_actual'])==56,'all inventories represented')
# R01 verified all exact external file membership; current flat member hashes are independently verified here.
source=read(E/'IMPLEMENTATION_INPUT_001.json'); sm={x['path']:x['sha256'] for x in source['files']}
check(all(tm[p]['sha256']==h for p,h in sm.items()),'all final source rows in target')
validation=[]
for n in range(102,117):
 p=next(x for x in E.glob('*_%03d.json'%n) if '_input' not in x.name);rec=read(p);inp=read(E/rec['input_manifest']);im={x['path']:x['sha256'] for x in inp['files']};mm=[x for x,h in sm.items() if im.get(x)!=h];log=p.with_suffix('.log');raw=log.read_text();counts=[tuple(map(int,x)) for x in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',raw)]
 check(not mm and rec['exit_code']==0 and not rec['source_changed_during_command'] and sha(log)==rec['log_sha256'],'owner record '+str(n))
 validation.append({'number':n,'record':sealrow(p),'input_manifest':sealrow(E/rec['input_manifest']),'same_final_source_rows':len(sm)-len(mm),'source_mismatches':mm,'exit_code':rec['exit_code'],'log':sealrow(log),'passed_tests':sum(x[0] for x in counts),'failed_tests':sum(x[1] for x in counts),'ignored_tests':sum(x[2] for x in counts)})
log=(E/'resource_core_alltargets_103.log').read_text();testlines=[x for x in log.splitlines() if x.startswith('test ') and x.endswith(' ... ok') and any(y in x for y in ['game::data::content_pack::','game::world::tests::content_generation::','game::world::tests::checkpoint_validation::'])]
check(len(testlines)>50,'nonzero R02 durable source tests')
# Full source/log linkage is corroborated above. Only these selected oracles are indexed, never substitute for suite count.
(O/'REQUIRED_TEST_OBSERVATIONS.json').write_text(json.dumps({'source_record':'resource_core_alltargets_103.json','selected_passed_count':len(testlines),'actual_log_lines':testlines,'full_suite_passed':1410,'not_a_new_execution':True},indent=2)+'\n')
handoff=read(B.parent.parent/'P2G_60_HANDOFF_001.json');refs=[]
for row in handoff['source_locations']+handoff['external_context_locations']:
 p=R/row['path'];ok=sha(p)==row['sha256'];check(ok,row['path']);refs.append(dict(row,verified=ok))
# Original target correction adds documentary basis only.
initial=read(B/'AUDIT_TARGET_007.json');om={x['path']:x for x in initial['manifest']};added=sorted(set(tm)-set(om));removed=sorted(set(om)-set(tm));changed=[p for p in set(tm)&set(om) if tm[p]!=om[p]]
check(len(added)==18 and not removed and not changed,'evidence-only target correction')
# Historical release077 and producer097: preserve exact execution identities, derive changed source locators.
old_exec=[]
for n in [77,97]:
 p=next(x for x in E.glob('*_%03d.json'%n) if '_input' not in x.name);rec=read(p);im={x['path']:x['sha256'] for x in read(E/rec['input_manifest'])['files']};delta=[x for x,h in sm.items() if im.get(x)!=h];check(rec['exit_code']==0 and sha(p.with_suffix('.log'))==rec['log_sha256'] and not rec['source_changed_during_command'],'historical record '+str(n));old_exec.append({'number':n,'record':sealrow(p),'changed_final_source_paths':delta,'disposition':'original execution with explicitly bounded seam reuse; not a final104 binary execution'})
# Current command-level evidence.
c=E/'PREDECESSOR_CONSUMERS_001'; consumers=[]
for row in read(c/'RESULT.json')['commands']:
 ok=row['exit_code']==0 and sha(c/(row['name']+'.log'))==row['log_sha256'];check(ok,'consumer '+row['name']);consumers.append({'name':row['name'],'exit':row['exit_code'],'valid_log':ok})
cli=read(c/'cli/commands.json');check(cli[0]['name']=='package' and cli[0]['exit_code']==0,'final107 actual packager')
for row in cli:
 check(row['exit_code']==row['expected_exit'] and sha(Path(row['stdout']))==row['stdout_sha256'] and sha(Path(row['stderr']))==row['stderr_sha256'],'CLI '+row['name'])
pack=read(c/'cli/package.stdout');check(len(pack['source_receipt']['files'])==26 and len(pack['pack_receipt']['files'])==26,'current107 package receipts')
for row in pack['source_receipt']['files']:
 p=R/'game_resources'/row['path'];check(sha(p)==row['sha256'] and p.stat().st_size==row['byte_count'],'current107 official source '+row['path'])
for row in pack['pack_receipt']['files']:
 p=c/'cli/packs'/row['path'];check(sha(p)==row['sha256'],'current107 packaged '+row['path'])
diagdir=E/'PREDECESSOR_DIAGNOSTIC_001';diag=read(diagdir/'receipt.json');neg=0
for row in diag['commands']:
 out=diagdir/(row['name']+'.stdout');err=diagdir/(row['name']+'.stderr');check(sha(out)==row['stdout_sha256'] and sha(err)==row['stderr_sha256'],'diagnostic output '+row['name'])
 if row['exit_code']!=0:
  neg+=1;check(row['exit_code']==1 and out.stat().st_size==0 and err.stat().st_size>0,'diagnostic refusal '+row['name'])
check(neg==6 and len(diag['commands'])==10 and diag['contract_oracle_pass'],'10 diagnostic cases')
valid=read(diagdir/'valid_b.stdout');restored=read(diagdir/'restored_b.stdout');check(valid==restored and valid['run_snapshot']['resources']['enkephalin']==733,'current108 B startup and retry identity')
sr=read(c/'source_receipt/receipt.json');check(sr['passed'] and sr['same_semantic_different_bytes'] and sr['failed_replacements']==36 and sr['successful_replacements']==3 and sr['only_source_receipt_differs_across_equal_byte_length_inputs'],'source receipt seam')
for mode in ['off','on','gap']:
 a=read(c/f'source_receipt/{mode}/a_source.json');b=read(c/f'source_receipt/{mode}/b_source.json');check(a!=b,'source receipt different bytes '+mode)
 check((c/f'source_receipt/{mode}/a.ndjson').read_bytes()==(c/'source_receipt/off/a.ndjson').read_bytes(),'A trace parity '+mode)
 check((c/f'source_receipt/{mode}/b.ndjson').read_bytes()==(c/'source_receipt/off/b.ndjson').read_bytes(),'B trace parity '+mode)
invalid=read(c/'invalid/RESULT.json');check(invalid['passed'] and invalid['cases']==13 and invalid['api_invalid_attempts']==26 and invalid['cli_invalid_attempts']==26,'invalid retry counts')
# Inventory cardinality and policy inclusion checked against production closed enums/macro.
raw=(R/'core_enhanced/src/game/data/content_pack/raw.rs').read_text();catalogs=re.findall(r'^    (\w+),$',raw.split('pub enum CatalogKind {')[1].split('}')[0],re.M);definitions=re.findall(r'^    (\w+)\(',raw.split('definitions! {')[1].split('\n}')[0],re.M)
check(len(catalogs)==25 and len(definitions)==43,'25/43 typed inventory')
r={'checked_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'stage':sys.argv[1],'R01_registry':sealrow(reg),'R01_outputs':outputs,'R01_target_manifest_exactly_equal':True,'R01_reuse_scope':'349-source archive/49 modified+16 added attribution, 56 exact old external inventory and numerical common guard computations; only after registry/actual membership identity verification. R02 source judgment is separate.','R02_handoff_refs':refs,'final_source_rows':len(sm),'current_owner_validation':validation,'selected_R02_tests':len(testlines),'target_correction':{'added':added,'removed':removed,'changed':changed},'historical_execution_reuse':old_exec,'consumer_groups':consumers,'current107_package_source_and_pack_receipt_counts':[26,26],'current108_commands':10,'current108_empty_output_refusals':neg,'source_receipt_failed_replacements':36,'catalogs':catalogs,'definition_kinds':definitions,'errors':failures,'passed':not failures}
(O/('EVIDENCE_'+sys.argv[1]+'.json')).write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({'passed':not failures,'errors':failures,'R02_tests':len(testlines),'R01_outputs':len(outputs),'catalogs':len(catalogs),'definition_kinds':len(definitions)},indent=2))
