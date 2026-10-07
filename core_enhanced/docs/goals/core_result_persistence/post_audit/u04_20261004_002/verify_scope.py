import pathlib,json,hashlib,datetime
R=pathlib.Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_result_persistence';S=H/'execution/single_goal';A=pathlib.Path(__file__).parent;T=S/'target/u04_003'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text())
M=read(T/'MANIFEST.json');paths=set()
for folder in ('core_enhanced/src','core_enhanced/tests','game_server/src','game_server/tests','game_server/config','game_server/examples','config','game_resources/data','game_resources/live_data'):
 if (R/folder).exists():paths.update(p for p in (R/folder).rglob('*') if p.is_file())
for name in ('Cargo.toml','Cargo.lock','core_enhanced/Cargo.toml','game_server/Cargo.toml','core_enhanced/docs/audit/live_skill_catalog_manifest.ron','core_enhanced/docs/skills/live_content_index.md','core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json'):paths.add(R/name)
rows=[{'path':str(p.relative_to(R)),'sha256':sha(p),'bytes':p.stat().st_size} for p in sorted(paths)]
assert rows==M['product_source']['files'];assert hashlib.sha256(json.dumps(rows,sort_keys=True).encode()).hexdigest()==M['product_source']['sha256']
records=sorted(str(p.relative_to(R)) for p in H.rglob('*') if p.is_file() and not {'target','post_audit','__pycache__'}&set(p.relative_to(H).parts));assert records==sorted(r['path'] for r in read(T/'RECORDS.json'))
for key,n in [('records_manifest_sha256','RECORDS.json'),('references_manifest_sha256','REFERENCES.json'),('membership_manifest_sha256','MEMBERSHIP.json')]:assert M[key]==sha(T/n)
for r in M['archives_and_patches']:assert sha(T/r['path'])==r['sha256']
upstream=[]
for n in ['P2G_30_FREEZE_001.json','P2G_40_PHASE_PLAN_001.json','P2G_60_HANDOFF_001.json']:
 d=read(H/n)
 for a in d['artifacts']:
  raw=json.dumps(a['payload'],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode();assert 'sha256:'+hashlib.sha256(raw).hexdigest()==a['ref']['Content Digest']
 upstream.append({'path':str((H/n).relative_to(R)),'sha256':sha(H/n),'refs':[a['ref'] for a in d['artifacts']]})
upstream.append({'path':str((H/'P2G_50_GOAL_SET.ko.md').relative_to(R)),'sha256':sha(H/'P2G_50_GOAL_SET.ko.md'),'canonical_host_obligation_line':82})
c=read(S/'COVERAGE_003.json');f=read(H/'P2G_30_FREEZE_001.json')['artifacts'][0]['payload']['Observable Policy Acceptance Set'];old=read(A.with_name('u04_20261004_001')/'CONDITION_REVIEW.json')
assert len(f)==len(c['acceptances'])==len(old['acceptances'])==15
for freeze,coverage,audit in zip(f,c['acceptances'],old['acceptances']):
 assert freeze['Policy Element Revision Ref']==coverage['Policy Element Revision Ref']==audit['policy_ref'];assert freeze['Observable Policy Acceptance']==coverage['Observable Policy Acceptance']==audit['observable_acceptance']
assert len(c['validations'])==15 and len(c['required_dimensions'])==10
p=read(S/'PREDECESSOR_IMPACT.json');pa=read(A.with_name('u04_20261004_001')/'PREDECESSOR_REVIEW.json');assert len(p['conditions'])==len(pa['conditions'])==112;assert len(p['r06_acceptances'])==len(pa['r06_acceptances'])==31;assert [r['id'] for r in p['conditions']]==[r['id'] for r in pa['conditions']]
result={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'product_roots_fresh_enumeration_matches':True,'product_member_count':len(rows),'record_roots_fresh_enumeration_matches':True,'record_count':len(records),'manifest_index_and_archive_hashes_match':True,'exact_upstream_artifact_digests_verified':upstream,'all_frozen_acceptance_meanings_match_current_coverage_and_prior_audit':True,'counts':{'acceptances':15,'validation_seams':15,'dimensions':10,'inherited':112,'separate_r06':31}}
(A/'SCOPE_AND_UPSTREAM.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='exact_upstream_artifact_digests_verified'}))
