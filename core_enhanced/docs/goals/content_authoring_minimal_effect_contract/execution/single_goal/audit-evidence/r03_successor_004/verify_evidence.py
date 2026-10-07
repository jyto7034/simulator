import pathlib,json,hashlib,tarfile,difflib,re,datetime
R=pathlib.Path('/mnt/f/work/simulator')
B=R/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal'
E=B/'evidence'
O=R/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/audit-evidence/r03_successor_004'
def sha(b):return hashlib.sha256(b).hexdigest()
def read(p):return json.loads(p.read_text())
errors=[]; result={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
base=read(B/'baseline/BASELINE_001.json'); original={}
with tarfile.open(B/'baseline/RECOVERED_001.tar.gz') as archive:
 for x in base['files']:
  data=archive.extractfile(x['path']).read();original[x['path']]=data
  if sha(data)!=x['sha256']:errors.append('baseline '+x['path'])
result['original284_verified']=len(original)
inputs=read(E/'INPUTS_118.json')['files']; current={x['path']:x['sha256'] for x in inputs}
for p,s in current.items():
 if sha((R/p).read_bytes())!=s:errors.append('current '+p)
result['current_inputs_verified']=len(inputs)
a=read(E/'ATTRIBUTION_030.json');patch=''
for x in a['files']:
 p=x['path'];prior=original.get(p,b'');now=(R/p).read_bytes()
 if x.get('before_sha256') and sha(prior)!=x['before_sha256']:errors.append('attribution-before '+p)
 if sha(now)!=x['current_sha256']:errors.append('attribution-after '+p)
 patch+=''.join(difflib.unified_diff(prior.decode().splitlines(keepends=True),now.decode().splitlines(keepends=True),fromfile='a/'+p if p in original else '/dev/null',tofile='b/'+p))
result['reconstructed_patch_sha256']=sha(patch.encode())
result['attributed_files']=len(a['files'])
if sha(patch.encode())!=a['patch_sha256']:errors.append('patch reconstruction mismatch')
old=read(R/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/AUDIT_TARGET_003.json')
changed=[]
for x in old['manifest']:
 if x.get('sha256') and sha((R/x['path']).read_bytes())!=x['sha256']:changed.append(x['path'])
result['previous_target_changed']=changed
new=read(R/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/AUDIT_TARGET_004.json')
if set(changed)!={x['path'] for x in new['previous_target_changed_members']}:errors.append('previous-target delta list mismatch')
result['official_ron_changes']=[p for p,v in original.items() if p.startswith('game_resources/data/') and sha(v)!=sha((R/p).read_bytes())]
result['input_currentness']={}
for n in [101,104,113,114,115,116,117,118]:
 rows=read(E/f'INPUTS_{n:03}.json')['files'];changes=[x['path'] for x in rows if sha((R/x['path']).read_bytes())!=x['sha256']]
 result['input_currentness'][str(n)]={'files':len(rows),'changed':changes}
reuse=read(E/'release_source_reuse_002.json'); recon=[]
for x in reuse['exact_nonimpact_changes']:
 data=(R/x['path']).read_bytes()
 if 'removed_range_bytes' in x:
  l,h=x['removed_range_bytes'];removed=data[l:h];restored=data[:l]+data[h:]
 else:
  removed=b'\n[features]\nallocation-observer = []\n';restored=data.replace(removed,b'')
 ok=sha(removed)==x['removed_sha256'] and sha(restored)==x['restored_sha256']
 recon.append({'path':x['path'],'restored_sha256':sha(restored),'passed':ok})
 if not ok:errors.append('release reconstruction '+x['path'])
result['release_reconstruction']=recon
full=read(E/'full_core_001.json'); lines=full['output'].splitlines()
counts=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',full['output'])
result['full_core_actual_counts']=counts
if sum(int(x[1]) for x in counts)!=1327 or any(x[0]!='ok' or any(int(v) for v in x[2:]) for x in counts):errors.append('full core counts')
selectors=['source_addresses_and_queries_distinguish_effect_and_definition_identity','scalar_edit_preserves_other_effects_and_rejects_shared_carrier_without_mutation','invalid_authoring_has_source_object_action_effect_and_field','reorder_and_failed_retry_preserve_stable_addresses_and_supported_operations','authored_literal_ron_executes_recomposed_definitions_with_exact_effect_sources','effect_authoring_failures_never_publish_or_change_an_existing_token','executed_proc_effect_retains_item_cast_and_authored_origin','withdraw_preserves_released_skill_projectile_with_launch_snapshot','withdraw_preserves_ground_fixed_area_and_ends_follow_areas','withdraw_cancels_queued_unreleased_skill_step','hit_gated_self_heal_and_buff_stack_repeat_attack_work_together','current_v6_checkpoint_continue_then_v6_capture_is_semantically_identical']
result['r01_direct_test_results']={s:[x for x in lines if s in x and x.startswith('test ')] for s in selectors}
for s,hits in result['r01_direct_test_results'].items():
 if len(hits)!=1 or not hits[0].endswith(' ... ok'):errors.append('test missing '+s)
result['predecessor_commands']=[]
for folder in ['PREDECESSOR_AB_001','PREDECESSOR_DIAGNOSTIC_001']:
 d=read(E/folder/'receipt.json');rows=d['commands']
 for x in rows:
  for stream in ['stdout','stderr']:
   p=pathlib.Path(x.get(stream,str(E/folder/(x['name']+'.'+stream))))
   if sha(p.read_bytes())!=x[stream+'_sha256']:errors.append('output '+str(p))
 result['predecessor_commands'].append({'folder':folder,'commands':len(rows),'success':sum(x['exit_code']==0 for x in rows),'expected_rejections':sum(x['exit_code']!=0 for x in rows)})
ab=E/'PREDECESSOR_AB_001';values=[read(ab/(s+'.stdout')) for s in ['consume_a1','consume_b','consume_a2']]
result['actual_currency']=[v['run_snapshot']['resources']['enkephalin'] for v in values]
result['aba_equal']=values[0]==values[2]
if result['actual_currency']!=[500,733,500] or not result['aba_equal']:errors.append('ABA oracle')
dg=E/'PREDECESSOR_DIAGNOSTIC_001'
for prefix in ['invalid_b','boss_invalid_zone','boss_invalid_route']:
 first=(dg/(prefix+'_1.stderr')).read_bytes();second=(dg/(prefix+'_2.stderr')).read_bytes()
 if first!=second or not first or (dg/(prefix+'_1.stdout')).read_bytes():errors.append('diagnostic oracle '+prefix)
for a,b in [('valid_b','restored_b'),('boss_valid','boss_restored')]:
 if (dg/(a+'.stdout')).read_bytes()!=(dg/(b+'.stdout')).read_bytes():errors.append('restored diagnostic '+a)
fixture='core_enhanced/tests/fixtures/content_authoring/literal_effects.ron'
fixture_sha=sha((R/fixture).read_bytes())
result['literal_fixture_continuity']={'path':fixture,'sha256':fixture_sha,'bytes':(R/fixture).stat().st_size,'basis':[]}
for n in [1,3,4]:
 t=read(R/f'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/AUDIT_TARGET_{n:03}.json')
 row=next(x for x in t['manifest'] if x['path']==fixture)
 if row['sha256']!=fixture_sha:errors.append('literal fixture continuity')
 result['literal_fixture_continuity']['basis'].append(t.get('record_id',f'target{n:03}'))
result['independent_input_enumeration']={}
for directory in ['core_enhanced/src','core_enhanced/tests','game_resources/data']:
 omitted=[str(p.relative_to(R)) for p in (R/directory).rglob('*') if p.is_file() and p.suffix in ['.rs','.ron'] and str(p.relative_to(R)) not in current]
 result['independent_input_enumeration'][directory]=omitted
 if omitted not in [[],[fixture]]:errors.append('unbound source inputs '+directory)
for prefix,required in [('invalid_b',['b/policy.ron','generated_combat_preview','combat_difficulty.profiles[1].ordinary_bands[0].tuning.extra_waves[0].spawn_zone_ids[0]']),('boss_invalid_zone',['custom/pattern.ron','patterns[0]','missing summon spawn zone']),('boss_invalid_route',['custom/pattern.ron','patterns[0]','missing summon route'])]:
 text=(dg/(prefix+'_1.stderr')).read_text()
 for token in required:
  if token not in text:errors.append('missing exact diagnostic '+token)
result['external_runtime_oracles']={}
for folder in ['COMBAT_EXTERNAL_002','DELIVERY_EXTERNAL_001','PREVIEW_EXTERNAL_004']:
 d=read(E/folder/'RESULT.json')
 result['external_runtime_oracles'][folder]={k:d[k] for k in ['passed','commands','api_replays','a_damage','b_damage','first_application_time_ms','terminal_times_ms','action_counts','positive_previews'] if k in d}
 if not d['passed']:errors.append('external runtime '+folder)
result['evidence_reuse']='No gameplay/test/build executions in this audit; source/evidence verification only.'
result['errors']=errors;result['result']='pass' if not errors else 'fail'
(O/'evidence_review.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({k:v for k,v in result.items() if k not in ['previous_target_changed','r01_direct_test_results']},indent=2))
raise SystemExit(bool(errors))
