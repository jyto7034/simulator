from pathlib import Path
import json,hashlib,tarfile,difflib,re
R=Path('/mnt/f/work/simulator');P=R/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal';E=P/'evidence';O=P/'audit-evidence/fresh_002'
def h(b):return hashlib.sha256(b).hexdigest()
def load(p):return json.loads(p.read_text())
out={}
archives={}
for record,archive in [('BASELINE_001.json','RECOVERED_001.tar.gz'),('CORRECTION_001_BEFORE.json','CORRECTION_001_BEFORE.tar.gz')]:
 rows=load(P/'baseline'/record)['files'];before={}
 with tarfile.open(P/'baseline'/archive) as t:
  for row in rows:
   b=t.extractfile(row['path']).read();assert h(b)==row['sha256'];assert len(b)==row['bytes'];before[row['path']]=b
 out[record]={'members_verified':len(before),'archive_sha256':h((P/'baseline'/archive).read_bytes())};archives[record]=before
baseline=archives['BASELINE_001.json'];correction=archives['CORRECTION_001_BEFORE.json'];attrs=load(E/'ATTRIBUTION_031.json');patch=[]
for row in attrs['files']:
 old=baseline.get(row['path'],b'');new=(R/row['path']).read_bytes();assert (h(old) if row['path'] in baseline else None)==row['before_sha256'];assert h(new)==row['current_sha256'];patch.extend(difflib.unified_diff(old.decode().splitlines(True),new.decode().splitlines(True),fromfile=('a/'+row['path'] if row['path'] in baseline else '/dev/null'),tofile='b/'+row['path']))
patch=''.join(patch).encode();assert h(patch)==attrs['patch_sha256'];assert patch==(E/'R03_DELTA_031.patch').read_bytes();out['attribution']={'rows':len(attrs['files']),'rust_rows':sum(x['path'].endswith('.rs') for x in attrs['files']),'patch_sha256':h(patch),'official_ron_changes':[p for p,b in baseline.items() if p.endswith('.ron') and (R/p).read_bytes()!=b]}
inputs=load(E/'INPUTS_119.json')['files'];changed=[]
for row in inputs:
 b=(R/row['path']).read_bytes();assert h(b)==row['sha256']
 if row['path'] not in correction or b!=correction[row['path']]:changed.append(row['path'])
trial=R/'core_enhanced/src/authoring/trial.rs';raw=trial.read_bytes();removed=[l for l in raw.splitlines(True) if b'source_receipt:' in l];assert len(removed)==6;restored=b''.join(l for l in raw.splitlines(True) if b'source_receipt:' not in l);assert restored==correction['core_enhanced/src/authoring/trial.rs'];out['correction']={'changed_source':changed,'removed_lines':[x.decode().rstrip() for x in removed],'restored_sha256':h(restored)}
canonical='core_enhanced/docs/core_runtime_contract.md';d=''.join(difflib.unified_diff(correction[canonical].decode().splitlines(True),(R/canonical).read_text().splitlines(True),fromfile='before/'+canonical,tofile='current/'+canonical));(O/'canonical_correction_diff.patch').write_text(d)
reuse=load(E/'release_source_reuse_002.json');proof=[]
for row in reuse['exact_nonimpact_changes']:
 b=(R/row['path']).read_bytes()
 if 'removed_range_bytes' in row:
  start,end=row['removed_range_bytes'];removed=b[start:end];rb=b[:start]+b[end:];assert h(removed)==row['removed_sha256']
 else:
  removed=b'\n[features]\nallocation-observer = []\n';assert h(removed)==row['removed_sha256'];assert b.count(removed)==1;rb=b.replace(removed,b'',1)
 assert h(rb)==row['restored_sha256'];proof.append({'path':row['path'],'restored_sha256':h(rb)})
out['prior_source_reconstruction']=proof
out['input_comparisons']={}
for n in [101,103,106,109,113,114,115,116,117,118,119]:
 ip=E/f'INPUTS_{n:03d}.json'
 if not ip.exists():continue
 rows=load(ip)['files'];differences=[r['path'] for r in rows if h((R/r['path']).read_bytes())!=r['sha256']];out['input_comparisons'][ip.name]={'count':len(rows),'differences':differences}
full=load(E/'correction001_full_core_001.json');assert full['exit_code']==0 and not full['source_changed_during_command'];summaries=re.findall(r'test result:.*',full['output']);counts=[int(re.search(r'(\d+) passed',x)[1]) for x in summaries];assert sum(counts)==1327;assert all('0 failed; 0 ignored;' in x for x in summaries);out['full_core']={'command':full['command'],'summaries':summaries,'passed_total':sum(counts),'relevant_tests':[l for l in full['output'].splitlines() if l.startswith('test ') and any(s in l for s in ['authoring','owned_clone','checkpoint','profile','generation','effect_source','authored_literal','same_time','pin'])]}
for n in ['correction001_authoring_001','correction001_all_targets_001','correction001_release_check_001','correction001_release_build_001','correction001_probe_build_001','correction001_combat_build_001']:
 a=load(E/(n+'.json'));assert a['exit_code']==0;assert not a.get('source_changed_during_command');out[n]={'command':a['command'],'summaries':re.findall(r'test result:.*',a.get('output','')),'exit_code':0}
handoff=load(P.parent.parent/'P2G_60_HANDOFF_001.json');out['handoff']=[]
for row in handoff['source_locations']+handoff['external_context_locations']:
 assert h((R/row['path']).read_bytes())==row['sha256'];out['handoff'].append({'path':row['path'],'sha256':row['sha256']})
# No whole old receipt/work/cost values are asserted current by this reconstruction.
(O/'evidence_check.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({k:v for k,v in out.items() if k not in ['full_core','handoff']},indent=2));print('full core',out['full_core']['summaries'])
