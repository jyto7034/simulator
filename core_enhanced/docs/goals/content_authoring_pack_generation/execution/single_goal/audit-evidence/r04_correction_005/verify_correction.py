import pathlib,json,hashlib,tarfile,difflib,re,datetime
R=pathlib.Path('/mnt/f/work/simulator');B=R/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal/evidence';D=R/'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal';O=D/'audit-evidence/r04_correction_005'
def j(p):return json.loads(p.read_text())
def sha(b):return hashlib.sha256(b).hexdigest()
errors=[];out={'at':datetime.datetime.now(datetime.timezone.utc).isoformat()}; before=j(B/'IMPLEMENTATION_INPUT_007.json')['files'];after=j(B/'IMPLEMENTATION_INPUT_008.json')['files'];changed=[]
for p,h in after.items():
 if sha((R/p).read_bytes())!=h:errors.append(['INPUT008 mismatch',p])
 if before.get(p)!=h:changed.append(p)
out['input008_count']=len(after);out['changed_inputs']=changed;out['unchanged_inputs']=len(after)-len(changed)
att=j(B/'CORRECTION_002_ATTRIBUTION.json')
with tarfile.open(B/'CORRECTION_002_BEFORE.tar.gz') as t:old={m.name:t.extractfile(m).read() for m in t if m.isfile()}
patch=[]
for row in att['changed_source_members']:
 p=row['path'];a=old[p];b=(R/p).read_bytes()
 if sha(a)!=row['before_sha256'] or sha(b)!=row['after_sha256']:errors.append(['attribution mismatch',p])
 patch.extend(difflib.unified_diff(a.decode().splitlines(keepends=True),b.decode().splitlines(keepends=True),fromfile='before/'+p,tofile='after/'+p))
rebuilt=''.join(patch).encode();out['correction_patch_sha256']=sha(rebuilt)
if rebuilt!=(B/att['patch']).read_bytes():errors.append(['patch reconstruction mismatch'])
archive_count=0
with tarfile.open(B/'SOURCE_AFTER_002.tar.gz') as t:
 for m in t:
  if m.isfile():
   archive_count+=1
   if t.extractfile(m).read()!=(R/m.name).read_bytes():errors.append(['source archive mismatch',m.name])
out['source_archive_current_members']=archive_count
prev=j(D/'AUDIT_TARGET_005.json');current=j(D/'AUDIT_TARGET_006.json');cm={x['path']:x for x in current['manifest']};old_changes=[]
for row in prev['manifest']:
 p=row['path'];actual=sha((R/p).read_bytes()) if (R/p).is_file() else None
 if actual!=row['sha256']:old_changes.append({'path':p,'old':row['sha256'],'current':actual})
out['previous_target_changed_members']=old_changes
# Bind complete current source/config/test/catalog identity to captured target, including bounded input omissions.
missing=[];surface=[]
for root in ['core_enhanced/src','core_enhanced/tests','core_enhanced/data']:
 for p in (R/root).rglob('*'):
  if p.is_file() and p.suffix in ['.rs','.ron']:
   rel=str(p.relative_to(R));surface.append(rel)
   if rel not in cm:missing.append(rel)
   elif sha(p.read_bytes())!=cm[rel]['sha256']:errors.append(['full surface mismatch',rel])
out['full_rust_ron_surface_count']=len(surface);out['surface_omitted_from_partial_input']=[p for p in surface if p not in after]
if missing:errors.append(['unmanifested source',missing])
log=(B/'r04_full209.log').read_text();tests=re.findall(r'^test (.+) \.\.\. ok$',log,re.M);groups=[list(map(int,x)) for x in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)]
out['full209_individual_passes']=len(tests);out['full209_groups']=groups
if len(tests)!=1363 or sum(g[0] for g in groups)!=1363 or any(any(g[1:]) for g in groups):errors.append(['full209 counts',groups])
for required in ['runtime_strike_admission_precedes_formula_limits_and_rolls_back','native_action_projectile_first_hit_is_at_300_after_accept_0_release_100','authored_literal_ron_executes_recomposed_definitions_with_exact_effect_sources','source_addresses_and_queries_distinguish_effect_and_definition_identity','scalar_edit_preserves_other_effects_and_rejects_shared_carrier_without_mutation','effect_authoring_failures_never_publish_or_change_an_existing_token']:
 if not any(s.endswith(required) for s in tests):errors.append(['required test absent',required])
out['selected_r01_passes']=[s for s in tests if any(k in s for k in ['authoring::tests','authored_literal','hit_gated','skill_data::tests','bundle_validation_returns_typed','range_preview','withdraw','checkpoint','strike_admission','first_hit_is_at_300','signed_heal','modify_damage','executed_proc_effect','publish','fragment','gate'])]
records=[]
for p in sorted(B.glob('r04_*.json')):
 if not p.stem[-3:].isdigit() or int(p.stem[-3:]) not in [206,208,*range(209,232)]:continue
 v=j(p);h=sha((B/v['raw_log']).read_bytes());records.append({'record':p.name,'command':v.get('command_args',v.get('command')),'exit_code':v['exit_code'],'raw_log_sha256':h,'input':v.get('implementation_input')})
 if h!=v['log_sha256'] or v['exit_code']!=0:errors.append(['execution mismatch',p.name])
 if v.get('implementation_input_sha256') and v['implementation_input_sha256']!=sha((B/'IMPLEMENTATION_INPUT_008.json').read_bytes()):errors.append(['run input mismatch',p.name])
out['current_execution_records']=records;out['errors']=errors;out['passed']=not errors
(O/'correction_review.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({k:v for k,v in out.items() if k not in ['selected_r01_passes','current_execution_records','surface_omitted_from_partial_input']},indent=2));print('selected tests',len(out['selected_r01_passes']),'execution receipts',len(records))
