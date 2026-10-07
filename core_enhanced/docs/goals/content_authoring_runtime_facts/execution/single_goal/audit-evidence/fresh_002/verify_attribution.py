import pathlib,json,hashlib,tarfile,difflib,datetime
ROOT=pathlib.Path('/mnt/f/work/simulator'); P=ROOT/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal'; OUT=pathlib.Path(__file__).parent
checks=[]
def ck(name,value):
 checks.append({'check':name,'pass':bool(value)})
 if not value:raise AssertionError(name)
def sha(b):return hashlib.sha256(b).hexdigest()
def archive(p):
 with tarfile.open(p) as t:return {m.name:t.extractfile(m).read() for m in t.getmembers() if m.isfile()}
before=archive(P/'baseline/SOURCE_BEFORE_001.tar.gz');after=archive(P/'evidence/SOURCE_AFTER_002.tar.gz');narrow=archive(P/'evidence/CORRECTION_002_BEFORE.tar.gz')
b=json.loads((P/'baseline/BASELINE_001.json').read_text()); a=json.loads((P/'evidence/ATTRIBUTION_002.json').read_text()); inp=json.loads((P/'evidence/IMPLEMENTATION_INPUT_008.json').read_text())['files'];oldinp=json.loads((P/'evidence/IMPLEMENTATION_INPUT_007.json').read_text())['files']
ck('original353',len(before)==353)
for row in b['files']:ck('baseline archive '+row['path'],sha(before[row['path']])==row['sha256'] and len(before[row['path']])==row['bytes'])
for path,data in after.items():ck('final archive '+path,(ROOT/path).read_bytes()==data)
ck('final329',len(after)==329)
for path,h in inp.items():ck('input '+path,sha((ROOT/path).read_bytes())==h)
changed=[p for p in inp if inp[p]!=oldinp[p]];ck('narrow3',changed==['core_enhanced/src/game/battle/core/damage_runtime.rs','core_enhanced/src/game/battle/core/mod.rs','core_enhanced/src/game/battle/core/runtime_facts.rs'])
entry=json.loads((P/'evidence/CORRECTION_002_ENTRY.json').read_text())
for row in entry['before_members']:ck('correction archive '+row['path'],sha(narrow[row['path']])==row['sha256'] and len(narrow[row['path']])==row['bytes'])
reg=json.loads((ROOT/entry['registry']['path']).read_text());
for row in reg['files']:ck('historical initial output '+row['path'],sha((ROOT/row['path']).read_bytes())==row['sha256'] and (ROOT/row['path']).stat().st_size==row['bytes'])
supplement=json.loads((ROOT/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/baseline/BASELINE_001.json').read_text())['files']+json.loads((ROOT/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/AUDIT_TARGET_001.json').read_text())['manifest']
preexisting={}
for row in supplement:
 path=row['path']
 if path in after and path not in before and row['sha256']==sha(after[path]):preexisting[path]=row['sha256']
ck('four archive extension files independently preexisting',len(preexisting)==4)
allchanged=sorted(p for p in set(before)|set(after) if p not in preexisting and (ROOT/p).is_file() and before.get(p,b'')!=(ROOT/p).read_bytes())
listed=sorted(x['path'] for x in a['changed_files']);ck('full attribution63 exact',allchanged==listed)
for row in a['changed_files']:
 path=row['path'];data=(ROOT/path).read_bytes();ck('attribution current '+path,sha(data)==row['after_sha256']);ck('attribution prior '+path,(sha(before[path]) if path in before else None)==row['before_sha256'])
def patch(paths,prior):
 return ''.join(''.join(difflib.unified_diff(prior.get(p,b'').decode().splitlines(keepends=True),(ROOT/p).read_text().splitlines(keepends=True),fromfile=('before/'+p if p in prior else '/dev/null'),tofile='after/'+p)) for p in paths).encode()
fullpatch=patch([x['path'] for x in a['changed_files']],before);narrowpatch=patch(changed,narrow)
ck('fullpatch reconstructed',fullpatch==(P/'evidence/R04_DELTA_002.patch').read_bytes());ck('narrowpatch reconstructed',narrowpatch==(P/'evidence/CORRECTION_002_SOURCE.patch').read_bytes())
target=json.loads((P/'AUDIT_TARGET_003.json').read_text());members={r['path'] for r in target['manifest']};members.update(str(pathlib.Path(d['directory'])/r['path']) for d in target['external_actual'] for r in d['files'])
walk=sorted(str(p.relative_to(ROOT)) for root in ['core_enhanced/src','core_enhanced/tests','game_resources/data'] for p in (ROOT/root).rglob('*') if p.is_file() and p.suffix in ['.rs','.ron'])
ck('independent source/test/live data target coverage',not(set(walk)-members))
ck('finalarchive covers all current Rust/RON',not(set(walk)-set(after)))
old=json.loads((P/'AUDIT_TARGET_002.json').read_text());oldchanged=[r['path'] for r in old['manifest'] if r['sha256'] is not None and sha((ROOT/r['path']).read_bytes())!=r['sha256']];ck('old target six changed members',len(oldchanged)==6)
result={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'checks':checks,'passed':True,'preexisting_archive_extensions':preexisting,'baseline_members':len(before),'after_members':len(after),'full_changes':a['changed_files'],'narrow_changed':changed,'unchanged_input':len(inp)-len(changed),'old_target_changed':oldchanged,'independent_source_test_data_files':len(walk),'partial_input_omissions':sorted(set(walk)-set(inp)),'scope':'Compare actual dirty archive, full current source walk and exact target; no clean HEAD substitution; changes include separate P2G Working Policy owner.'}
(OUT/'attribution_review.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['checks','full_changes']},indent=2))
