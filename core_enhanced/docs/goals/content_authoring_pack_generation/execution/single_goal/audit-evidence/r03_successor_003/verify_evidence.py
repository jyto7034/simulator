import pathlib,json,hashlib,tarfile,difflib,re,datetime
R=pathlib.Path('/mnt/f/work/simulator');B=R/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal';E=B/'evidence';A=R/'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal';O=A/'audit-evidence/r03_successor_003'
def digest(x):return hashlib.sha256(x).hexdigest()
def sha(p):return digest(p.read_bytes())
def read(p):return json.loads(p.read_text())
res={'at':datetime.datetime.now(datetime.timezone.utc).isoformat()}
h=read(A.parent.parent/'P2G_60_HANDOFF_001.json')
for x in h['source_locations']+h['external_context_locations']:assert sha(R/x['path'])==x['sha256']
res['handoff_bindings_verified']=len(h['source_locations']+h['external_context_locations'])
base=read(B/'baseline/BASELINE_001.json')['files'];archive=tarfile.open(B/'baseline/RECOVERED_001.tar.gz');original={x['path']:archive.extractfile(x['path']).read() for x in base}
for x in base:assert digest(original[x['path']])==x['sha256'] and len(original[x['path']])==x['bytes']
attr=read(E/'ATTRIBUTION_030.json');patch=''
for x in attr['files']:
 path=x['path'];before=original.get(path,b'');after=(R/path).read_bytes()
 assert sha(R/path)==x['current_sha256']
 if x['before_sha256'] is not None:assert digest(before)==x['before_sha256']
 patch+=''.join(difflib.unified_diff(before.decode().splitlines(True),after.decode().splitlines(True),fromfile=('a/'+path if path in original else '/dev/null'),tofile='b/'+path))
assert digest(patch.encode())==attr['patch_sha256']==sha(E/'R03_DELTA_030.patch')
res['baseline_and_delta']={'original_verified':len(base),'archive_sha256':sha(B/'baseline/RECOVERED_001.tar.gz'),'attribution_rows':len(attr['files']),'patch_sha256':digest(patch.encode()),'official_ron_changed':[x['path'] for x in base if x['path'].startswith('game_resources/') and sha(R/x['path'])!=x['sha256']]}
assert not res['baseline_and_delta']['official_ron_changed']
old=read(A/'AUDIT_TARGET_003.json');new=read(A/'AUDIT_TARGET_004.json');newmap={x['path']:x['sha256'] for x in new['manifest']};changes=[{'path':x['path'],'before_sha256':x['sha256'],'current_sha256':newmap.get(x['path'])} for x in old['manifest'] if x['sha256']!=newmap.get(x['path'])]
assert changes==new['previous_target_changed_members'];res['old_target_changed_members']=changes
res['input_comparisons']={}
for n in [101,113,114,115,116,117,118]:
 inp=read(E/f'INPUTS_{n:03}.json')['files']; changes=[x['path'] for x in inp if sha(R/x['path'])!=x['sha256']]
 res['input_comparisons'][str(n)]={'count':len(inp),'changed':changes}
 if n in [113,116,117,118]:assert not changes
 if n in [114,115]:assert changes==['core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs']
inputs={x['path'] for x in read(E/'INPUTS_118.json')['files']}
actual={str(p.relative_to(R)) for root in ['core_enhanced/src','core_enhanced/tests','game_resources/data'] for p in (R/root).rglob('*') if p.suffix in ['.rs','.ron']}
missing=sorted(actual-inputs);assert missing==['core_enhanced/tests/fixtures/content_authoring/literal_effects.ron'];res['bounded_input_omission']=missing
literal=missing[0];assert newmap[literal]==sha(R/literal)=='1bd51a81d6b528ea4a80895bf95348c8417fd7113c77a73c6a2543d7821cf8c0';assert 'include_str!("fixtures/content_authoring/literal_effects.ron")' in (R/'core_enhanced/tests/skill_runtime_contract.rs').read_text()
res['literal_current_identity']=sha(R/literal)
full=read(E/'full_core_001.json');assert full['exit_code']==0;out=full['output'];counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',out)
assert len(counts)==6 and sum(int(v[0]) for v in counts)==1327 and all(all(int(q)==0 for q in v[1:]) for v in counts)
need=['game::data::content_pack::','game::world::tests::content_generation::','game::world::tests::checkpoint_validation::']
passed=[]
for prefix in need:passed.extend(re.findall(r'^test ('+re.escape(prefix)+r'\S+) \.\.\. ok$',out,re.M))
assert len(passed)>50;res['full_core']={'command':full['command'],'input':full['inputs'],'counts':counts,'r02_tests':passed}
pre=read(E/'predecessor_external_001.json');rows=[]
for c in pre['commands']:
 receipt=E/c['receipt'];assert sha(receipt)==c['receipt_sha256'];d=read(receipt)
 for row in d['commands']:
  for stream in ['stdout','stderr']:
   p=pathlib.Path(row.get(stream,receipt.parent/(row['name']+'.'+stream)))
   assert sha(p)==row[stream+'_sha256']
  rows.append({'name':row['name'],'exit':row['exit_code']})
 assert len(d['commands'])==10
ab=read(E/'PREDECESSOR_AB_001/receipt.json');ad=E/'PREDECESSOR_AB_001';a1=read(ad/'consume_a1.stdout');a2=read(ad/'consume_a2.stdout');b=read(ad/'consume_b.stdout')
assert a1==a2 and a1['content_identity']!=b['content_identity'];currency=[x['run_snapshot']['resources']['enkephalin'] for x in [a1,b,a2]];assert currency==[500,733,500]
for field,loc in [('source_before_after','source'),('custom_before_after','live_data/b'),('official_before_after','live_data/official')]:
 for path,dig in ab[field].items():assert sha(ad/loc/path)==dig
for path,dig in ab['binary_sha256'].items():assert sha(pathlib.Path(path))==dig
neg={'missing_b_catalog':'b/policy.ron','invalid_b_catalog':'b/policy.ron','old_b_contract':'b/pack.ron','unsupported_b_capability':'b/pack.ron','existing_output':'exclusively create'}
for name,term in neg.items():assert term in (ad/(name+'.stderr')).read_text() and (ad/(name+'.stdout')).read_bytes()==b''
dd=E/'PREDECESSOR_DIAGNOSTIC_001';dr=read(dd/'receipt.json')
for prefix,terms in [('invalid_b',['b/policy.ron','combat_difficulty.profiles[1].ordinary_bands[0].tuning.extra_waves[0].spawn_zone_ids[0]','official/data/pve/encounters.ron','official/data/map/battlefield_templates.ron']),('boss_invalid_zone',['custom/pattern.ron','patterns[0]','actor_behaviors[1]','missing summon spawn zone']),('boss_invalid_route',['custom/pattern.ron','patterns[0]','actor_behaviors[1]','missing summon route'])]:
 s=(dd/(prefix+'_1.stderr')).read_text();assert s==(dd/(prefix+'_2.stderr')).read_text();assert all(t in s for t in terms)
 for i in [1,2]:assert (dd/(prefix+f'_{i}.stdout')).read_bytes()==b''
for x,y in [('valid_b','restored_b'),('boss_valid','boss_restored')]:assert (dd/(x+'.stdout')).read_bytes()==(dd/(y+'.stdout')).read_bytes()
for p,dig in dr['boss_fixture_original'].items():assert sha(E/'PREDECESSOR_DIAGNOSTIC_FIXTURE_001/live_data'/p)==dig
assert sha(R/'target/release/consume_content_pack')==dr['binary_sha256'];res['predecessor_external']={'rows':rows,'successes':sum(x['exit']==0 for x in rows),'expected_rejections':sum(x['exit']==1 for x in rows),'currency':currency,'repeat_restore_source_oracles':True}
assert res['predecessor_external']['successes']==8 and res['predecessor_external']['expected_rejections']==12
(O/'evidence_review.json').write_text(json.dumps(res,indent=2)+'\n');print(json.dumps({k:v for k,v in res.items() if k not in ['old_target_changed_members','full_core']},indent=2));print('R02 passing test rows:',len(passed))
