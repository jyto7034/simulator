"""Metadata comparison/restoration of the exact inherited test preimage only."""
from pathlib import Path
import json,tarfile,hashlib,datetime
ROOT=Path('/mnt/f/work/simulator');OUT=Path(__file__).resolve().parent
now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat()
def ref(p):
 b=p.read_bytes();return {'path':str(p.relative_to(ROOT)),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
before=json.loads((OUT/'BEFORE.json').read_text());base={r['path']:r for r in before['source']['files']};saved={};bindings={}
for archive in before['recovery_archives']:
 assert ref(ROOT/archive['path'])==archive
 with tarfile.open(ROOT/archive['path'],'r:gz') as t:
  for member in t.getmembers():
   if member.isfile():
    name=member.name.removeprefix('./');data=t.extractfile(member).read();saved[name]=data
    bindings[name]={'archive':archive,'member':member.name,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()}
fmt=(OUT/'execute_format.py').read_text();ns={'__file__':str(OUT/'execute_format.py'),'__name__':'metadata_only'}
# Equivalent regex-position scanning avoids quadratic copies of large strings.
prefix=fmt[:fmt.index('label=sys.argv')].replace('raw=re.match(', 'raw=re.compile(').replace('char=re.match(', 'char=re.compile(').replace(',s[i:])', ').match(s,i)')
exec(prefix,ns)
name='core_enhanced/src/game/world/application.rs';p=ROOT/name;old=saved[name].decode();current=p.read_text()
a=ns['items'](old);b=ns['items'](current);assert len(a)==len(b)
restoration=OUT/'CFG_TEST_RESTORATION_001.json';assert not restoration.exists()
started=now();observed=ref(p);edits=[]
for inherited,actual in reversed(list(zip(a,b))):
 if inherited[2]!=actual[2]:
  assert actual[2].replace('        | ActionKind::RequestRelicEffectivePlan\n','')==inherited[2]
  edits.append({'original_bytes':len(inherited[2].encode()),'observed_bytes':len(actual[2].encode()),'kind':'remove_own_new_enum_arm_only'})
  current=current[:actual[0]]+inherited[2]+current[actual[1]:]
assert len(edits)==1
p.write_text(current)
restoration.write_text(json.dumps({'actual_started_at_utc':started,'actual_closed_at_utc':now(),'before':observed,'after':ref(p),'preimage':bindings[name],'changes':edits,'observed_diff':ref(OUT/'CFG_TEST_COLLECTION_FAILURE_001.txt'),'check010_preserved':ref(OUT/'u09_product_bundle_b_check010.json'),'failed_collection_exact_start_end_utc':'not_captured; reported as selfcheck observation, not an execution interval claim','no_other_work_reverted':True},indent=2)+'\n')
rows=[];changes=[]
for name,row in base.items():
 raw=(ROOT/name).read_bytes();assert name in saved and hashlib.sha256(saved[name]).hexdigest()==row['sha256'],name
 changed=hashlib.sha256(raw).hexdigest()!=row['sha256']
 if name.endswith('.rs'):
  if changed:
   prior=ns['items'](saved[name].decode());actual=ns['items'](raw.decode());assert [r[2] for r in prior]==[r[2] for r in actual],name
   count=len(prior)
  else:
   # Exact whole-file bytes imply every test body/registration is unchanged.
   count=saved[name].count(b'#[cfg(test)]')
  rows.append({'path':name,'cfg_test_items':count,'exact_bytes_equal':True,'proof':'lexical_cfg_items' if changed else 'exact_whole_file','preimage':bindings[name]})
 if changed:changes.append(name)
result={'actual_closed_at_utc':now(),'kind':'source/test-registration-byte-comparison-only','inherited_source_sha256':before['source']['sha256'],'cfg_test_items_unchanged_at_final_boundary':True,'historical_correction':ref(restoration),'files':rows,'changed_existing_paths':changes,'new_paths':[p for p in ns['paths'] if p not in base]}
p=OUT/'CFG_TEST_PRESERVATION.json';assert not p.exists();p.write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'result':ref(p),'restoration':ref(restoration),'rust_files':len(rows),'cfg_test_items':sum(x['cfg_test_items'] for x in rows),'existing_product_delta':len(changes),'new_product_files':len(result['new_paths'])}))
