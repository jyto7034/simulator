from pathlib import Path
import json,hashlib,tarfile,difflib,re,datetime
R=Path('/mnt/f/work/simulator');B=R/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal';E=B/'evidence';O=B/'audit-evidence/fresh_001'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def rows(p):return [json.loads(x) for x in p.read_text().splitlines()]
out={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'method':'Independent read-only source/hash/reconstructed delta and retained actual-output assertions; no owner command rerun'}
h=read(B.parent.parent/'P2G_60_HANDOFF_001.json');refs=h['source_locations']+h['external_context_locations']
for x in refs:assert sha(R/x['path'])==x['sha256']
out['exact_handoff_refs']=refs
baseline=read(B/'baseline/BASELINE_001.json')['files']
with tarfile.open(B/'baseline/RECOVERED_001.tar.gz') as a:
 original={x['path']:a.extractfile(x['path']).read() for x in baseline}
for x in baseline:assert hashlib.sha256(original[x['path']]).hexdigest()==x['sha256'] and len(original[x['path']])==x['bytes']
at=read(E/'ATTRIBUTION_030.json');patch=''
for x in at['files']:
 p=x['path'];before=original.get(p,b'');after=(R/p).read_bytes();assert hashlib.sha256(after).hexdigest()==x['current_sha256']
 if x['before_sha256']:assert hashlib.sha256(before).hexdigest()==x['before_sha256']
 patch+=''.join(difflib.unified_diff(before.decode().splitlines(True),after.decode().splitlines(True),fromfile='a/'+p if p in original else '/dev/null',tofile='b/'+p))
assert hashlib.sha256(patch.encode()).hexdigest()==at['patch_sha256']==sha(E/'R03_DELTA_030.patch')
official=[x['path'] for x in baseline if x['path'].startswith('game_resources/') and sha(R/x['path'])!=x['sha256']];assert official==[]
out['baseline']={'original_members':len(baseline),'archive_sha256':sha(B/'baseline/RECOVERED_001.tar.gz'),'attribution_rows':len(at['files']),'delta_sha256':sha(E/'R03_DELTA_030.patch'),'official_changes':official,'new_caveat':'Absent from bounded baseline does not establish new authorship'}
out['input_comparisons']={}
for n in [101,113,114,115,116,117,118]:
 a=read(E/f'INPUTS_{n:03}.json')['files'];changes=[x['path'] for x in a if sha(R/x['path'])!=x['sha256']];out['input_comparisons'][str(n)]={'count':len(a),'changes':changes}
 if n in [113,116,117,118]:assert not changes
 if n in [114,115]:assert changes==['core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs']
current=read(B/'AUDIT_TARGET_001.json');manifest={x['path']:x['sha256'] for x in current['manifest']};inputs={x['path'] for x in read(E/'INPUTS_118.json')['files']}
actual={str(p.relative_to(R)) for root in ['core_enhanced/src','core_enhanced/tests','game_resources/data'] for p in (R/root).rglob('*') if p.suffix in ['.rs','.ron']}
missing=sorted(actual-inputs);assert missing==['core_enhanced/tests/fixtures/content_authoring/literal_effects.ron'];literal=missing[0]
assert sha(R/literal)==manifest[literal]=='1bd51a81d6b528ea4a80895bf95348c8417fd7113c77a73c6a2543d7821cf8c0';assert 'include_str!("fixtures/content_authoring/literal_effects.ron")' in (R/'core_enhanced/tests/skill_runtime_contract.rs').read_text()
for n in ['001','003']:assert next(x['sha256'] for x in read(R/f'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/AUDIT_TARGET_{n}.json')['manifest'] if x['path']==literal)==sha(R/literal)
out['bounded_checkpoint_omission']={'path':literal,'direct_current_and_historical_target_hash':sha(R/literal),'test_include_str_verified':True}
f=read(E/'full_core_001.json');counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',f['output']);assert f['exit_code']==0 and sum(int(c[0]) for c in counts)==1327 and all(all(int(x)==0 for x in c[1:]) for c in counts)
tests=re.findall(r'^test (\S+) \.\.\. ok$',f['output'],re.M); selected=[x for x in tests if any(t in x for t in ['authoring','controlled_','owned_clone','clone_shape','clone_history','trial_','optional_trace','receipt_','content_generation','checkpoint_validation'])]
out['full_core']={'command':f['command'],'inputs':f['inputs'],'counts':counts,'reviewed_r03_and_predecessor_test_rows':selected}
# Independently compare actual source-bound external receipts, not summary pass flags.
runtime={}
for g in ['a','b']:
 p=E/'COMBAT_EXTERNAL_002';raw=(p/f'{g}_off/receipts.ndjson').read_bytes()
 for m in ['on','gap']:assert (p/f'{g}_{m}/receipts.ndjson').read_bytes()==raw
 for m in ['on','off']:assert (p/f'cli_{g}_{m}.stdout').read_bytes()==raw
 d=read(p/f'{g}_off/RESULT.json');assert d['accepted_steps']==13 and d['is_complete'] and d['failure'] is None;runtime[g]=d['first_sourced']
assert [runtime[g]['change']['after'] for g in ['a','b']]==[61,43]
for k in ['ordinal','time_ms','source_unit','target_unit','source']:assert runtime['a'][k]==runtime['b'][k]
out['combat']=runtime
p=E/'DELIVERY_EXTERNAL_001';raw=(p/'off/receipts.ndjson').read_bytes()
for m in ['on','gap','repeat']:assert (p/m/'receipts.ndjson').read_bytes()==raw
for m in ['on','off']:assert (p/f'cli_{m}.stdout').read_bytes()==raw
tr=rows(p/'on/terminal_trace.ndjson');counts={}
for a,e in [('delayed_repeat','repeat_hit'),('delayed_projectile','projectile_hit'),('periodic_area','area_hit')]:
 rs=[x for x in tr if x['source'] and x['source']['authored']['site']['action']['action_id']==a];assert all(x['change']['channel']=='hp' and x['change']['before']>x['change']['after'] and x['source']['authored']['site']['effect_id']==e for x in rs);assert read(p/f'explain_{a}.stdout')['source']==rs[0]['source']['authored'];counts[a]=len(rs)
assert list(counts.values())==[14,7,48];out['delivery_actual_hp']=counts
p=E/'PREVIEW_EXTERNAL_004';raw=(p/'api/off_receipts.ndjson').read_bytes()
for m in ['on','gap']:assert (p/f'api/{m}_receipts.ndjson').read_bytes()==raw
for m in ['on','off']:assert (E/f'FINAL_EXTERNAL_001/cli_{m}.log').read_bytes()==raw
assert (E/'FINAL_EXTERNAL_001/cli_trace.ndjson').read_bytes()==(p/'cli_trace.ndjson').read_bytes();out['preview_current_cli_full_receipts_trace_equal']=True
cost=read(E/'cost_summary_002.json');assert cost['plain_samples']==475 and cost['allocator_samples']==285
for mode,count in [('plain',5),('alloc',3)]:
 d=read(E/f'COST_EXTERNAL_002/{mode}.json');assert len(d['results'])==count and all(x['coexisting_independently_loaded_generations']==4 for x in d['results'])
out['cost']={k:cost[k] for k in ['source','plain_samples','allocator_samples','plain_repetitions','allocator_repetitions','cross_binary_all_receipts_usage_and_trace_equal']}
out['source_receipt_finding']=read(O/'source_receipt_probe.json')
(O/'evidence_review.json').write_text(json.dumps(out,indent=2)+'\n');print('Verified original284/delta127, checkpoints, full1327 with',len(selected),'relevant rows, actual combat/delivery/preview/cost evidence. Source-receipt finding retained.')
