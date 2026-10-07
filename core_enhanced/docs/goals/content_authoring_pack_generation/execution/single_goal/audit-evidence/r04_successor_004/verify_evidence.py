import json,hashlib,tarfile,difflib,re,datetime
from pathlib import Path
R=Path('/mnt/f/work/simulator'); B=R/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal'; O=R/'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal/audit-evidence/r04_successor_004'
def read(p):return json.loads(p.read_text())
def sha(b):return hashlib.sha256(b).hexdigest()
checks=[]
def check(name,ok,detail=None):checks.append({'name':name,'pass':bool(ok),'detail':detail})
target=read(O.parent.parent/'AUDIT_TARGET_005.json'); tm={x['path']:x for x in target['manifest']};a=read(B/'evidence/ATTRIBUTION_001.json');baseline=read(B/'baseline/BASELINE_001.json');inp=read(B/'evidence/IMPLEMENTATION_INPUT_007.json')['files']
with tarfile.open(R/baseline['archive']['path']) as ar: originals={m.name:ar.extractfile(m).read() for m in ar if m.isfile()}
for e in baseline['files']:check('baseline:'+e['path'],e['path'] in originals and sha(originals[e['path']])==e['sha256'] and len(originals[e['path']])==e['bytes'])
for p,h in inp.items():check('INPUT007:'+p,sha((R/p).read_bytes())==h)
patch=[]
for e in a['changed_files']:
 p=e['path'];old=originals.get(p,b'');new=(R/p).read_bytes() if (R/p).exists() else b''
 check('attribution:'+p,(sha(old) if old else None)==e['before_sha256'] and (sha(new) if new else None)==e['after_sha256'])
 patch.extend(difflib.unified_diff(old.decode().splitlines(keepends=True),new.decode().splitlines(keepends=True),fromfile='before/'+p if old else '/dev/null',tofile='after/'+p if new else '/dev/null'))
rebuilt=sha(''.join(patch).encode());check('exact scoped patch',rebuilt==a['patch_sha256'],rebuilt)
omitted=[]
for d in ['core_enhanced/src','core_enhanced/tests','core_enhanced/data']:
 for p in (R/d).rglob('*'):
  if not p.is_file() or p.suffix not in ['.rs','.ron']:continue
  rel=str(p.relative_to(R));h=sha(p.read_bytes());check('formal source binding:'+rel,rel in tm and tm[rel]['sha256']==h)
  if rel in inp:continue
  same=rel in originals and originals[rel]==p.read_bytes(); adaptation=False
  if rel=='core_enhanced/tests/live_item_skill_activation.rs':
   replacement='''        equipped_items: equipped_items
            .into_iter()
            .map(
                |base_uuid| game_core::game::battle::types::BattleEquipmentBinding {
                    base_uuid,
                    persistent_copy_uuid: None,
                },
            )
            .collect(),'''
   adaptation=originals[rel].decode().replace('        equipped_items,',replacement,1)==p.read_text()
  omitted.append({'path':rel,'unchanged_from_archive':same,'exact_binding_only_adaptation':adaptation,'sha256':h});check('omitted input applicability:'+rel,same or adaptation)
old_target=read(O.parent.parent/'AUDIT_TARGET_004.json');old_changes=[]
for m in old_target['manifest']:
 p=R/m['path'];h=sha(p.read_bytes()) if p.is_file() else None
 if h!=m['sha256']:old_changes.append({'path':m['path'],'old':m['sha256'],'current':h,'R04_attributed':m['path'] in {e['path'] for e in a['changed_files']}})
# Original handoff has six source/context entries, checked independently of target.
handoff=read(O.parent.parent.parent.parent/'P2G_60_HANDOFF_001.json')
for k in ['source_locations','external_context_locations']:
 for e in handoff[k]:check('handoff:'+e['path'],sha((R/e['path']).read_bytes())==e['sha256'])
records=[]
for p in sorted((B/'evidence').glob('r04_*.json')):
 if not p.stem[-3:].isdigit() or int(p.stem[-3:]) not in [164,165,168,170,171,172,173,174,175,176,177,178,185,186,187,188,189,190,191]:continue
 x=read(p);check('owner command:'+p.name,x['exit_code']==0 and sha((B/'evidence'/x['raw_log']).read_bytes())==x['log_sha256']);records.append(x)
log=(B/'evidence/r04_full164.log').read_text();tests=re.findall(r'^test (.+) \.\.\. ok$',log,re.M);summaries=[list(map(int,m)) for m in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)];check('nonzero full164 actual tests',len(tests)==1361 and sum(x[0] for x in summaries)==1361 and all(not any(x[1:]) for x in summaries),summaries)
relevant=[t for t in tests if any(x in t for x in ['game::data::content_pack::','game::world::tests::content_generation::','game::world::tests::checkpoint_validation::'])]
check('original R02 durable seam executions',len(relevant)>=72,{'count':len(relevant),'tests':relevant})
result={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'command':'python3 '+str(Path(__file__).relative_to(R)),'baseline_members':len(originals),'INPUT007_members':len(inp),'attribution_members':len(a['changed_files']),'patch_sha256':rebuilt,'old_target_changed_members':old_changes,'omitted_source_inputs':omitted,'full164_summaries':summaries,'full164_individual_passes':len(tests),'R02_test_rows':relevant,'owner_execution_records':records,'checks':checks,'pass':all(x['pass'] for x in checks)}
(O/'evidence_review.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['checks','owner_execution_records','R02_test_rows','omitted_source_inputs','old_target_changed_members']},indent=2));print('changed',len(old_changes),'omitted',len(omitted),'R02 tests',len(relevant),'checks',len(checks),'failures',[x for x in checks if not x['pass']])
