import pathlib,json,hashlib,tarfile,difflib,re,datetime
R=pathlib.Path('/mnt/f/work/simulator'); B=R/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal'; O=R/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/audit-evidence/r04_successor_005'
def sha(b): return hashlib.sha256(b).hexdigest()
def read(p):return json.loads(p.read_text())
errors=[]; result={'at':datetime.datetime.now(datetime.timezone.utc).isoformat()}
a=read(B/'evidence/ATTRIBUTION_001.json');baseline=read(B/'baseline/BASELINE_001.json');inputs=read(B/'evidence/IMPLEMENTATION_INPUT_007.json')['files']
with tarfile.open(R/baseline['archive']['path']) as t:
 originals={m.name:t.extractfile(m).read() for m in t if m.isfile()}
for m in baseline['files']:
 data=originals.get(m['path'])
 if data is None or sha(data)!=m['sha256'] or len(data)!=m['bytes']:errors.append({'baseline':m['path']})
result['baseline_archive_count']=len(originals)
for p,h in inputs.items():
 if sha((R/p).read_bytes())!=h:errors.append({'input007':p})
result['current_input007_count']=len(inputs)
patch=[];changed=[]
for m in a['changed_files']:
 p=m['path'];before=originals.get(p,b'');after=(R/p).read_bytes() if (R/p).exists() else b''
 if (sha(before) if before else None)!=m['before_sha256'] or (sha(after) if after else None)!=m['after_sha256']:errors.append({'attribution':p,'before':sha(before),'after':sha(after)})
 patch.extend(difflib.unified_diff(before.decode().splitlines(keepends=True),after.decode().splitlines(keepends=True),fromfile='before/'+p if before else '/dev/null',tofile='after/'+p if after else '/dev/null'))
 changed.append(m)
rebuilt=''.join(patch).encode();result['reconstructed_patch_sha256']=sha(rebuilt);result['declared_patch_sha256']=a['patch_sha256'];result['attributed_changed_count']=len(changed)
if sha(rebuilt)!=a['patch_sha256']:errors.append({'patch_reconstruction':'mismatch','actual':sha(rebuilt)})
old=read(R/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/AUDIT_TARGET_004.json')
old_changes=[]
for m in old['manifest']:
 p=R/m['path'];actual=sha(p.read_bytes()) if p.is_file() else None
 if actual!=m['sha256']:old_changes.append({'path':m['path'],'before':m['sha256'],'after':actual,'r04_attribution':any(x['path']==m['path'] for x in changed)})
result['previous_target_changed_members']=old_changes
# Source inputs outside the deliberately bounded INPUT007: bind to original dirty baseline and exact audit target.
current_target=read(R/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/AUDIT_TARGET_005.json');tm={m['path']:m for m in current_target['manifest']};om={m['path']:m for m in baseline['files']}
omitted=[]
for directory in ['core_enhanced/src','core_enhanced/tests','core_enhanced/data']:
 for p in (R/directory).rglob('*'):
  if not p.is_file() or p.suffix not in ['.rs','.ron']:continue
  rel=str(p.relative_to(R));actual=sha(p.read_bytes())
  if rel not in tm:errors.append({'not_in_target':rel})
  if rel not in inputs:
   row={'path':rel,'sha256':actual,'baseline_unchanged':rel in om and om[rel]['sha256']==actual};omitted.append(row)
   if not row['baseline_unchanged']:
    # This integration fixture is absent from the bounded production INPUT007.
    # Establish the entire delta exactly: same UUID vector mapped to the required
    # binding type with explicit missing persistence; no other fixture/assertion edit.
    if rel == 'core_enhanced/tests/live_item_skill_activation.rs':
     old=originals[rel].decode(); new=p.read_text()
     replacement="""        equipped_items: equipped_items
            .into_iter()
            .map(
                |base_uuid| game_core::game::battle::types::BattleEquipmentBinding {
                    base_uuid,
                    persistent_copy_uuid: None,
                },
            )
            .collect(),"""
     row['exact_mechanical_binding_adaptation'] = old.replace('        equipped_items,',replacement,1)==new
     row['input_reuse_basis']='Only required BattleUnitDraft equipped_items type adaptation; same source UUIDs/order, explicit None, unchanged assertions. Four named live_item integration tests execute in full164 on the matching current production shape.'
     if not row['exact_mechanical_binding_adaptation']:errors.append({'source_input_unbound':row})
    else:errors.append({'source_input_unbound':row})
for rel in ['Cargo.toml','Cargo.lock','core_enhanced/Cargo.toml']:
 actual=sha((R/rel).read_bytes())
 if rel not in inputs and (rel not in om or om[rel]['sha256']!=actual):errors.append({'config_unbound':rel})
result['bounded_input_omissions_checked']=omitted
log=(B/'evidence/r04_full164.log').read_text();tests=re.findall(r'^test (.+) \.\.\. ok$',log,re.M);summaries=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)
result['full_test_results']=[list(map(int,x)) for x in summaries];result['full_individual_pass_count']=len(tests)
if len(tests)!=1361 or sum(int(x[0]) for x in summaries)!=1361 or any(any(int(y) for y in x[1:]) for x in summaries):errors.append({'full_test_counts':summaries})
relevant=[s for s in tests if any(k in s for k in ['ability::authoring::tests','authored_literal','hit_gated','skill_data::tests','bundle_validation_returns_typed_skill','range_preview','withdraw','checkpoint','native_action_failed_timestamp','skill_projectile','signed_heal','modify_damage','executed_proc_effect_retains','partial_publish','publish','fragment'])]
result['selected_r01_test_passes']=relevant
records=[]
for p in sorted((B/'evidence').glob('r04_*.json')):
 if not p.stem[-3:].isdigit() or int(p.stem[-3:]) not in [164,165,167,168,170,171,172,173,174,175,176,177,178,179,180,181,185,186,187,188,189,190,191]:continue
 d=read(p);lp=B/'evidence'/d['raw_log']; actual=sha(lp.read_bytes());records.append({'record':p.name,'command':d['command'],'exit':d['exit_code'],'log':d['raw_log'],'log_sha256':actual,'input':d.get('input',d.get('scope'))})
 if actual!=d['log_sha256'] or d['exit_code']!=0:errors.append({'execution_record':p.name})
result['execution_records']=records
result['errors']=errors;result['passed']=not errors
(O/'evidence_review.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['selected_r01_test_passes','bounded_input_omissions_checked','previous_target_changed_members','execution_records']},indent=2));print('old changes',len(old_changes),'omitted unchanged',len(omitted),'selected tests',len(relevant),'records',len(records))
