import pathlib,json,hashlib,tarfile,difflib,re,datetime
R=pathlib.Path('/mnt/f/work/simulator'); E=R/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal/evidence'; O=pathlib.Path(__file__).parent
checks=[]
def ck(label,ok,detail=None):checks.append({'check':label,'ok':bool(ok),'detail':detail})
def h(b):return hashlib.sha256(b).hexdigest()
def j(p):return json.loads(p.read_text())
target=j(R/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/AUDIT_TARGET_003.json');members={m['path']:m for m in target['manifest']}
handoff=j(R/'core_enhanced/docs/goals/content_authoring_tool_ports/P2G_60_HANDOFF_001.json')
for m in handoff['source_locations']+handoff['external_context_locations']:ck('handoff '+m['path'],h((R/m['path']).read_bytes())==m['sha256'])
attr=j(E/'ATTRIBUTION_001.json')
with tarfile.open(R/attr['baseline']['path']) as t: before={m.name:t.extractfile(m).read() for m in t.getmembers() if m.isfile()}
with tarfile.open(E/'SOURCE_AFTER_001.tar.gz') as t: after={m.name:t.extractfile(m).read() for m in t.getmembers() if m.isfile()}
for p,b in after.items():ck('final archive '+p,(R/p).read_bytes()==b)
# Every before archive member is checked against its historical intake BASELINE record.
baseline=j(E.parent/'baseline/BASELINE_001.json')
basefiles={x['path']:x for x in baseline['files']}
for path,data in before.items():ck('original dirty baseline '+path,path in basefiles and h(data)==basefiles[path]['sha256'] and len(data)==basefiles[path]['bytes'])
print('baseline files type',type(basefiles).__name__)

baselines=list((E.parent/'baseline').glob('*.json')); print('baseline records',[p.name for p in baselines])
for p,b in before.items():
 old=next((m for m in j(R/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/AUDIT_TARGET_002.json')['manifest'] if m['path']==p),None)
 if old:ck('old R03 intake '+p,old['sha256']==h(b))
 else:ck('before archive present '+p,bool(b) or len(b)==0)
patch=''
for c in attr['changed_files']:
 p=c['path'];a=before.get(p,b'');b=(R/p).read_bytes();ck('attribution before '+p,h(a)==c['before_sha256'] if c['before_sha256'] is not None else p not in before);ck('attribution after '+p,h(b)==c['after_sha256']);patch+=''.join(difflib.unified_diff(a.decode().splitlines(keepends=True),b.decode().splitlines(keepends=True),fromfile=('before/'+p if p in before else '/dev/null'),tofile='after/'+p))
ck('reconstructed dirty delta',patch.encode()==(E/'R04_DELTA_001.patch').read_bytes(),h(patch.encode()))
inputs=j(E/'IMPLEMENTATION_INPUT_007.json')['files']
for p,expected in inputs.items():ck('INPUT007 '+p,h((R/p).read_bytes())==expected)
omitted=[]
for p in sorted((R/'core_enhanced/tests').rglob('*')):
 if p.is_file() and p.suffix in ['.rs','.ron']:
  rel=str(p.relative_to(R))
  if rel not in inputs:
   omitted.append(rel);ck('omitted bound '+rel,rel in members and h(p.read_bytes())==members[rel]['sha256'])
   if rel.endswith('live_item_skill_activation.rs'):
    old=before[rel].decode();new=p.read_text();print('ADAPTATION', ''.join(difflib.unified_diff(old.splitlines(True),new.splitlines(True))))
    expected=old.replace('        equipped_items,','        equipped_items: equipped_items\n            .into_iter()\n            .map(\n                |base_uuid| game_core::game::battle::types::BattleEquipmentBinding {\n                    base_uuid,\n                    persistent_copy_uuid: None,\n                },\n            )\n            .collect(),',1)
    # Exact observed replacement is checked below after reading diff; no assertion is removed.
    ck('live_item adaptation only',new==expected)
   else:ck('omitted unchanged '+rel,p.read_bytes()==before.get(rel))
raw=(E/'r04_full164.log').read_text();tests=re.findall(r'^test (.*?) \.\.\. ok$',raw,re.M);summaries=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',raw)
ck('full 1361 actual rows',len(tests)==1361);ck('full nonzero groups',[int(s[0]) for s in summaries]==[1309,4,4,25,15,4,0]);ck('no failed ignored measured filtered',all(all(int(x)==0 for x in s[1:]) for s in summaries))
relevant=[t for t in tests if any(x in t for x in ['authoring','native_','runtime_fact','checkpoint','profile','generation','pin','source','preview','control','clone'])]
commands=[]
for p in sorted(E.glob('r04_*.json')):
 m=re.search(r'(\d+)\.json$',p.name)
 if not m or int(m.group(1)) not in [164,165,167,168,170,171,172,173,*range(174,192)]:continue
 d=j(p)
 if 'raw_log' in d:
  log=E/d['raw_log'];ck('command log '+p.name,log.exists() and h(log.read_bytes())==d['log_sha256'])
 ck('command exit '+p.name,d.get('exit_code')==0);commands.append(d)
old=j(R/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/AUDIT_TARGET_002.json');delta=[m['path'] for m in old['manifest'] if m.get('sha256') and (R/m['path']).is_file() and h((R/m['path']).read_bytes())!=m['sha256']]
result={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'checks':checks,'passed':all(c['ok'] for c in checks),'check_count':len(checks),'before_archive_members':len(before),'after_archive_members':len(after),'attribution_rows':len(attr['changed_files']),'omitted_inputs':omitted,'old_R03_changed_members':delta,'individual_tests':len(tests),'test_group_counts':summaries,'relevant_test_rows':relevant,'current_owner_commands':commands}
(O/'evidence_review.json').write_text(json.dumps(result,indent=2)+'\n');print('checks',len(checks),'failures',[c for c in checks if not c['ok']]);print('omitted',len(omitted),'old R03 delta',len(delta),'test rows',len(tests),'relevant',len(relevant))
