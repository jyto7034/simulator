from pathlib import Path
import json,tarfile,tomllib,hashlib
R=Path('/mnt/f/work/simulator');D=R/'core_enhanced/docs/goals/core_runtime_safety_observability';S=D/'execution/s2';A=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();load=lambda p:json.loads(p.read_text());extras=load(A/'EXTRA_INPUTS.json')
with tarfile.open(S/'target/s2_001/SOURCE_BEFORE.tar.gz') as tf:before=tomllib.loads(tf.extractfile('Cargo.lock').read().decode())
after=tomllib.loads((R/'Cargo.lock').read_text());old={(x['name'],x['version']) for x in before['package']};new={(x['name'],x['version']) for x in after['package']};assert not old-new
review=load(S/'evidence/dependencies/product002/REVIEW.json');assert sorted(new-old)==[tuple(x) for x in review['added_versions']]
metadata=load(S/'evidence/dependencies/product002/metadata.json');packages={p['id']:p for p in metadata['packages']};epoch=next(p for p in metadata['packages'] if p['name']=='crossbeam-epoch');reverse=[]
for node in metadata['resolve']['nodes']:
 if epoch['id'] in node['dependencies']:reverse.append(packages[node['id']]['name'])
assert sorted(reverse)==['game_server','metrics-util']
files=[]
for name in ['metrics','metrics-util','metrics-exporter-prometheus','crossbeam-epoch']:
 package=next(p for p in metadata['packages'] if p['name']==name);folder=Path(package['manifest_path']).parent;lock=next(p for p in after['package'] if p['name']==name)
 archive=next(Path('/tmp/wp02-cargo-home/registry/cache').glob(f'*/{name}-{package["version"]}.crate'));assert sha(archive)==lock['checksum']
 with tarfile.open(archive) as tf:
  for member in tf.getmembers():
   if not member.isfile():continue
   path=Path(member.name).relative_to(f'{name}-{package["version"]}');data=tf.extractfile(member).read();expected=hashlib.sha256(data).hexdigest();p=folder/path;assert sha(p)==expected
   x={'path':str(p),'bytes':p.stat().st_size,'sha256':expected};files.append(x);extras.append(x)
node=next(n for n in metadata['resolve']['nodes'] if packages[n['id']]['name']=='metrics-exporter-prometheus');assert 'http-listener' in node['features'];assert not any(x in node['features'] for x in ['push-gateway','protobuf','tls'])
external=[]
for p in (S/'evidence/services').glob('*download001.json'):
 r=load(p);actual=Path(r['path']);assert actual.stat().st_size==r['bytes'];assert sha(actual)==r['actual_sha256']==r['expected_sha256'];external.append({'name':r['name'],'url':r['url'],'sha256':r['actual_sha256'],'archive_current':True});extras.append({'path':str(actual),'bytes':actual.stat().st_size,'sha256':r['actual_sha256']})
(A/'DEPENDENCY_VERIFICATION.json').write_text(json.dumps({'old_locked_pairs':len(old),'new_locked_pairs':len(new),'added_versions':sorted(new-old),'original_versions_preserved':True,'epoch_reverse_dependencies':reverse,'exporter_features':node['features'],'verified_dependency_files':len(files),'external_archives':external,'storage_review':'AtomicBucket.clear_with synchronously detaches/drains; fixed classic distributions; crossbeam defer+flush FIFO queue and one in-flight callback per participant; runtime has one collector and at most one blocking HTTP renderer. Fixture allocator has four test buckets/no HTTP, separate production ten/thirteen buckets.'},indent=2)+'\n')
unique={x['path']:x for x in extras};(A/'EXTRA_INPUTS.json').write_text(json.dumps(list(unique.values()),indent=2)+'\n');print('dependency checked',len(files),'source files; versions',len(old),len(new),'and official archives')
