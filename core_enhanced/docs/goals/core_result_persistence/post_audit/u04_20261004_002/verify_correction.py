import pathlib,json,hashlib,tarfile,datetime
R=pathlib.Path('/mnt/f/work/simulator'); H=R/'core_enhanced/docs/goals/core_result_persistence';S=H/'execution/single_goal';A=pathlib.Path(__file__).parent;P=A.with_name('u04_20261004_001');T=S/'target/u04_003';O=S/'target/u04_001'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text())
old=read(O/'MANIFEST.json');new=read(T/'MANIFEST.json');m=read(P/'AUDIT_ARTIFACTS.json')
assert sha(P/'AUDIT_ARTIFACTS.json')=='a7dd51b1920b5d79b0211648520de01c040220bf0614449a2c20d5407520bcab'
assert sha(P/'COMPLETION_AUDIT.md')=='214cfbdddb57a3c99ae8040530847f0ebc1bc2ffc6651a63c73f661cca532b95'
for r in m['files']:assert sha(P/r['path'])==r['sha256'] and (P/r['path']).stat().st_size==r['bytes'],r['path']
assert old['product_source']==new['product_source']
assert new['product_source']['sha256']=='cb033469ed897c2bb38cdd5cd2bc4561b8b820acd0b680d45c5381abf4b3c629'
oldrows={r['path']:r for r in old['source_files']};newrows={r['path']:r for r in new['source_files']}
changed=[{'path':p,'before':oldrows.get(p),'after':newrows.get(p)} for p in sorted(set(oldrows)|set(newrows)) if oldrows.get(p)!=newrows.get(p)]
assert set(oldrows)==set(newrows)
path='game_server/docs/unity_bridge_protocol.md';expected='ebfb9534fc901afe8a40885c23c6504e6524f0b07420f83aa0f45a60898429c4';after='22392eede07dcf66a8110ea2a07f94429597657df5dcfb780faa970d10562c63'
rows=[]
for n in ['before_files','source_files']:
 r=next(x for x in old[n] if x['path']==path);assert r['sha256']==expected;rows.append({'locator':'target/u04_001/MANIFEST.json#'+n,'member':r})
r=next(x for x in read(O/'MEMBERSHIP.json') if x['path']==path);assert r['sha256']==expected and r['groups']==['source'];rows.append({'locator':'target/u04_001/MEMBERSHIP.json','member':r})
archive_proofs=[]
for archive,want in [(O/'SOURCE_BEFORE.tar.gz',expected),(O/'SOURCE_AFTER.tar.gz',expected),(S/'baseline/SOURCE_BEFORE.tar.gz',expected),(S/'baseline/HOST_PROTOCOL_BEFORE.tar.gz',expected),(T/'SOURCE_BEFORE.tar.gz',expected),(T/'SOURCE_AFTER.tar.gz',after)]:
 with tarfile.open(archive) as t:
  raw=t.extractfile(path).read();assert hashlib.sha256(raw).hexdigest()==want
 archive_proofs.append({'path':str(archive.relative_to(R)),'member':path,'member_sha256':want,'bytes':len(raw)})
assert sha(P/'HOST_PROTOCOL_AT_AUDIT.md')==expected
assert sha(R/path)==after
assert read(P/'OMITTED_SURFACE.json')['in_target_manifest']==False
# The prior omission assertion is explicitly rejected; no old audit file is changed.
assert old['before_files']==new['before_files']
for n in ['PREDECESSOR_IMPACT.json','STATE_INVENTORY.md','SERVICE_REUSE.json']:
 row=next(r for r in read(O/'RECORDS.json') if r['path']==str((S/n).relative_to(R)));assert sha(S/n)==row['sha256']
result={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'old_audit_all_artifacts_unchanged':True,'old_audit_artifact_manifest_sha256':sha(P/'AUDIT_ARTIFACTS.json'),'old_target_sha256':sha(O/'MANIFEST.json'),'new_target_sha256':sha(T/'MANIFEST.json'),'product_exact_unchanged':True,'product_source_sha256':new['product_source']['sha256'],'before_source_union_exact_unchanged':True,'source_membership_exact_unchanged':True,'source_delta_from_audit001':changed,'original_host_document_rows':rows,'actual_archive_member_proofs':archive_proofs,'erratum_judgment':{'old_stale_content_finding':'valid; assigned canonical protocol was materially obsolete','old_omission_claim':'factually false; rejected in this pass','old_OMITTED_SURFACE_in_target_manifest_false':'factually false; rejected in this pass','baseline_supplement':'redundant exact copy of originally archived member; no added member','audit001_history':'preserved immutable Not Complete / High; its original valid content finding still required correction'},'unchanged_predecessor_state_service_records':True}
(A/'CORRECTION_INTEGRITY_AND_ERRATUM.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'product_exact':True,'source_delta':[r['path'] for r in changed],'original_membership_proven':True,'old_audit_preserved':True}))
