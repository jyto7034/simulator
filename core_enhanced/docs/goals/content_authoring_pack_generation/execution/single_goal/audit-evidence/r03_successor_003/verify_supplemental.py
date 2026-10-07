import pathlib,json,hashlib,datetime,math
R=pathlib.Path('/mnt/f/work/simulator');E=R/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/evidence';O=R/'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal/audit-evidence/r03_successor_003'
def read(p):return json.loads(p.read_text())
def dig(b):return hashlib.sha256(b).hexdigest()
def sha(p):return dig(p.read_bytes())
result={'at':datetime.datetime.now(datetime.timezone.utc).isoformat()}
reuse=read(E/'release_source_reuse_002.json');inp={x['path']:x['sha256'] for x in read(E/'INPUTS_101.json')['files']};restored=[]
for x in reuse['exact_nonimpact_changes']:
 p=R/x['path'];raw=p.read_bytes()
 if 'removed_range_bytes' in x:
  a,b=x['removed_range_bytes'];removed=raw[a:b];prior=raw[:a]+raw[b:]
  assert b'#[test]' in removed
 else:
  removed=b'\n[features]\nallocation-observer = []\n';assert removed in raw;prior=raw.replace(removed,b'')
 assert dig(removed)==x['removed_sha256'];assert dig(prior)==x['restored_sha256']==inp[x['path']];restored.append(x['path'])
for p,d in inp.items():
 if p not in restored:assert sha(R/p)==d
result['input101_default_production_reconstruction']={'unchanged':len(inp)-len(restored),'restored':restored,'scope':'Two test-only additions and one standalone optional allocation feature declaration; does not assert old and current binary byte identity.'}
api=read(E/'FINAL_EXTERNAL_001/api/commands.json');positives=negatives=0
for c in api:
 p=pathlib.Path(c['log']);assert sha(p)==c['log_sha256'];s=p.read_text()
 if c['kind']=='valid':assert c['exit_code']==0;positives+=1
 else:
  assert c['exit_code']==101 and all(word in s for word in c['required_diagnostics']);negatives+=1
assert positives==5 and negatives==6
result['public_api']={'positive_runtime_consumers':positives,'expected_privacy_rejections':negatives,'all_named_compiler_diagnostics_observed':True}
checks={}
for n in ['all_targets_001','release_check_001','release_build_041']:
 d=read(E/(n+'.json'));assert d['exit_code']==0;checks[n]={'command':d['command'],'input':d.get('inputs'),'exit_code':0}
result['compile_only_checks']=checks
for name in ['runtime_guards_001','kernel_guards_002','cost_external_002']:
 d=read(E/(name+'.json'))
 for c in d['commands']:
  if 'log' in c:assert sha(R/c['log'])==c['log_sha256']
  if name!='runtime_guards_001':assert c['exit_code']==0
r=read(E/'runtime_guards_001.json');assert not r['passed'];assert [(x['name'],x['exit_code']) for x in r['commands'] if x['exit_code']] == [('kernel_plain',101),('kernel_alloc',101)]
result['preserved_failures']={'runtime_guards_001':False,'kernel_plain':101,'kernel_alloc':101}
def bools(d):
 if isinstance(d,dict):
  for k,v in d.items():
   if isinstance(v,bool) and (k.endswith('_pass') or k in ['exact_repeatable','repeatable','all_caps_respected','rejection_rollback_exact']):assert v,k
   else:bools(v)
 elif isinstance(d,list):
  for v in d:bools(v)
summary={}
for directory in ['RUNTIME_GUARDS_001','KERNEL_GUARDS_002']:
 for mode in ['plain','alloc']:
  d=read(E/directory/(mode+'.json'));bools(d);assert d['allocation_observer_enabled']==(mode=='alloc')
  assert d['latency_gate_applicable']==(mode=='plain') and d['allocation_gate_applicable']==(mode=='alloc')
  if directory=='RUNTIME_GUARDS_001':
   for k in ['run_snapshot','checkpoint_export']:
    v=d[k];assert len(v['raw_elapsed_ns'])==v['sample_count']==101;assert sorted(v['raw_elapsed_ns'])[math.ceil(.95*101)-1]==v['nearest_rank_p95_ns'];assert v['nearest_rank_p95_ns']<=v['absolute_cap_ns'];assert v['encoded_bytes']<=v['encoded_byte_cap']
   summary[mode]={'snapshot_p95_ns':d['run_snapshot']['nearest_rank_p95_ns'],'export_p95_ns':d['checkpoint_export']['nearest_rank_p95_ns'],'snapshot_bytes':d['run_snapshot']['encoded_bytes'],'export_bytes':d['checkpoint_export']['encoded_bytes']}
  else:assert len(d['fixture_conversion_elapsed_ns'])==5 and all(x>0 for x in d['fixture_conversion_elapsed_ns'])
result['public_guard_observations']=summary;result['adapted_kernel']='Current token-input plain/allocation guards pass original checks; five separate positive conversion timings retained; old raw fixture failures not overwritten.'
invalid=read(E/'invalid_external_002.json');assert invalid['exit_code']==0 and invalid['content_cases']==13
assert sha(E/invalid['result'])==invalid['result_sha256'];result['additional_invalid_evidence']={'content_cases':13,'api_rejections':invalid['api_content_rejections'],'cli_rejections':invalid['cli_content_rejections'],'applicability':'Same production source reconstructed above; no new audit execution.'}
(O/'supplemental_review.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
