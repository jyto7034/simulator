import pathlib,json,hashlib,tarfile,difflib,re
R=pathlib.Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_battle_starter_tactics';S=H/'execution/single_goal';O=pathlib.Path(__file__).parent
j=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();sha_bytes=lambda b:hashlib.sha256(b).hexdigest()
old=j(S/'target/u06_001/MANIFEST.json');new=j(S/'target/u06_002/MANIFEST.json');reuse=j(S/'correction/f01_001/EVIDENCE_REUSE.json');errors=[]
def check(ok,why):
 if not ok:errors.append(why)
oldp={x['path']:x for x in old['product_source']['files']};newp={x['path']:x for x in new['product_source']['files']}
check(oldp.keys()==newp.keys(),'product membership changed');changed=sorted(k for k in oldp if oldp[k]!=newp[k]);check(changed==reuse['changed_product_files'],'recorded delta membership')
oldsource={x['path']:x for x in old['source_files']};newsource={x['path']:x for x in new['source_files']};source_changed=sorted(k for k in oldsource if oldsource[k]!=newsource[k]);canonical_changed=sorted(set(source_changed)-set(changed));roadmap='core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md';check(canonical_changed==[roadmap],'unexpected canonical delta')
with tarfile.open(S/'target/u06_001/SOURCE_AFTER.tar.gz') as a:
 patch=[]
 for path in changed:
  b=a.extractfile(path).read();check(sha_bytes(b)==oldp[path]['sha256'],'old source bytes '+path);patch+=list(difflib.unified_diff(b.decode().splitlines(keepends=True),(R/path).read_text().splitlines(keepends=True),fromfile='a/'+path,tofile='b/'+path))
 query='core_enhanced/src/game/battle/core/targeting.rs';q=a.extractfile(query).read().decode();check('#[cfg(test)]' in q and q.split('#[cfg(test)]')[0]==(R/query).read_text().split('#[cfg(test)]')[0],'query production prefix changed')
 section='\n## 0. 입력·권위·증거 수준\n';prior=a.extractfile(roadmap).read().decode();check(prior.split(section,1)[1]==(R/roadmap).read_text().split(section,1)[1],'roadmap policy changed')
 check(''.join(patch)==(S/'correction/f01_001/PRODUCT_DELTA.patch').read_text(),'correction patch differs from independent diff')
check('#[cfg(test)]\nmod tests;' in (R/'core_enhanced/src/game/world.rs').read_text(),'world test cfg')
priorrecords={x['path']:x for x in j(S/'target/u06_001/RECORDS.json')};frozen_matches=[]
def oldrecord(path):
 key=str(path.relative_to(R));check(key in priorrecords and sha(path)==priorrecords[key]['sha256'] and path.stat().st_size==priorrecords[key]['bytes'],'changed old evidence '+key);frozen_matches.append(key)
with tarfile.open(S/'target/u06_001/RECORDS.tar.gz') as a:
 def orig(n):return json.load(a.extractfile(str((S/n).relative_to(R))))
 oc=orig('COVERAGE.json');op=orig('PREDECESSOR_IMPACT.json');os=orig('EVIDENCE_SELECTION.json')
c=j(S/'COVERAGE.json');p=j(S/'PREDECESSOR_IMPACT.json');sel=j(S/'EVIDENCE_SELECTION.json')
check(oc['acceptances']==c['acceptances'],'U06 frozen acceptances changed');check(oc['required_dimensions']==c['required_dimensions'],'required dimensions changed')
check([(x['id'],x['requirement']) for x in op['conditions']]==[(x['id'],x['requirement']) for x in p['conditions']],'inherited condition meaning changed');check(op['r06_acceptances']==p['r06_acceptances'],'R06 acceptance meaning changed')
originalr06=j(R/'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/continuations/r06_completion_001/COVERAGE.json')['acceptances'];strip=lambda rows:[{k:x[k] for k in ['acceptance','expected','iva']} for x in rows];check(strip(originalr06)==strip(p['r06_acceptances']),'original R06 chain changed')
selector_additions={}
for ov,nv in zip(oc['validations'],c['validations']):
 a=set(ov['core_selectors']+ov['host_selectors']);b=set(nv['core_selectors']+nv['host_selectors']);check(a<=b,'removed selectors '+nv['id']);selector_additions[nv['id']]=sorted(b-a)
for n,x in reuse['reused_receipts'].items():
 for ext in ['json','log']:oldrecord(H/'evidence'/(n+'.'+ext))
 rr=j(H/'evidence'/(n+'.json'));check(rr['source_before']==rr['source_after']==old['product_source'],'old execution relabelled '+n);check(x['actual_execution_source_sha256']==old['product_source']['sha256'] and x['applicable_current_source_sha256']==new['product_source']['sha256'],'source applicability '+n)
for k,path in sel['public_summaries'].items():
 if k not in ['starter','f01_reproduction']:oldrecord(H/path)
C=S/'compatibility/current_001';CE=C/'evidence'
for n in ['ADAPTATION_FINAL.json']:oldrecord(C/n)
for n in ['PC_SUMMARY_CURRENT_001.json','PC_IMMEDIATE_ACCOUNTED_U06.json','pc_raw_current_001.json','R05_SUMMARY_CURRENT_003.json','r05_raw_current_002.json']:oldrecord(CE/n)
adapt=j(C/'ADAPTATION_FINAL.json');inputs=[];harness=[]
for row in adapt['files']:
 path=R/row['copy'];check(sha(path)==row['copy_sha256'],'adapted file changed '+row['copy'])
 (inputs if row['semantic_change'].startswith('none;') else harness).append(row['copy'])
 # General skill Heal is unrelated to BasicAttackAction::Heal.
 if row['semantic_change'].startswith('none;'):check(not re.search(rb'action\s*:\s*Heal\s*\(',path.read_bytes()),'inherited basic Heal input '+row['copy'])
layout=[]
for path,digest in reuse['native_accounting_source_basis'].items():check(sha(R/path)==digest==oldp[path]['sha256'],'layout/accounting changed '+path);layout.append(path)
heal=[]
for root in ['game_resources/data','game_resources/live_data']:
 for path in (R/root).rglob('*.ron'):
  for m in re.finditer(r'action\s*:\s*Heal\s*\(flat:\s*(\d+),\s*spell_power_percent:\s*(\d+)',path.read_text()):
   check(int(m[1])>0,'nonpositive official Heal');heal.append({'path':str(path.relative_to(R)),'flat':int(m[1]),'percent':int(m[2])})
check(len(heal)==2,'official basic Heal enumeration')
probe=H/'evidence/f01_reproduce_001';summary=j(probe/'SUMMARY.json');priorprobe=H/'post_audit/u06_20261005_001/probe'
for path,digest in summary['exact_auditor_probe_input'].items():check(sha(priorprobe/path)==sha(probe/'probe'/path)==digest,'auditor probe input changed '+path)
raw=j(probe/'RAW.json') if (probe/'RAW.json').exists() else None
for x in ['zero','coefficient_rounds_to_zero']:check(summary['cases'][x]['attack_starts']==0 and summary['cases'][x]['positive_hp_events']==0 and summary['cases'][x]['final_hp']==50,'ineffective reproduction '+x)
check(summary['cases']['positive_control']['attack_starts']==4 and summary['cases']['positive_control']['positive_hp_events']==4 and summary['cases']['positive_control']['final_hp']==90,'positive reproduction')
result={'errors':errors,'old_product':old['product_source']['sha256'],'current_product':new['product_source']['sha256'],'changed_product_paths':changed,'canonical_changed':canonical_changed,'roadmap_body_unchanged':True,'query_production_prefix_unchanged':True,'independent_patch_sha256':sha_bytes(''.join(patch).encode()),'exact_original_acceptances':{'u06':len(c['acceptances']),'inherited':len(p['conditions']),'r06':len(p['r06_acceptances']),'dimensions':len(c['required_dimensions'])},'selector_additions':selector_additions,'reused_receipts':len(reuse['reused_receipts']),'unchanged_old_record_paths':frozen_matches,'unchanged_compatibility_inputs':len(inputs),'unchanged_compatibility_harness':len(harness),'unchanged_layout_accounting_paths':layout,'official_basic_heal_definitions':heal,'exact_prior_auditor_probe_inputs':summary['exact_auditor_probe_input'],'probe_summary':summary['cases']}
(O/'DELTA_REUSE_VERIFICATION.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['unchanged_old_record_paths','selector_additions','unchanged_layout_accounting_paths','exact_prior_auditor_probe_inputs','probe_summary']},ensure_ascii=False))
