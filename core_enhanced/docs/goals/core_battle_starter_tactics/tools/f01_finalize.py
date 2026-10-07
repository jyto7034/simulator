"""Assemble fresh F01 validation and explicit, non-relabelled evidence reuse."""
import copy,datetime,json,re,tarfile
from run import ROOT,HERE,identity,sha
from coverage_contracts import CONTRACTS
S=HERE/'execution/single_goal';F=S/'correction/f01_001';T=S/'target/u06_001';E=HERE/'evidence'
A=HERE/'post_audit/u06_20261005_001';current=identity()
reuse_path=F/'EVIDENCE_REUSE.json';reuse=json.loads(reuse_path.read_text())
assert reuse['source_after']==current
old_product=reuse['source_before']
old_manifest=json.loads((T/'MANIFEST.json').read_text())
product_paths={row['path'] for row in old_product['files']}
roadmap='core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md'
for row in old_manifest['source_files']:
 if row['path'] not in product_paths and row['path']!=roadmap:
  assert sha(ROOT/row['path'])==row['sha256'],row['path']
with tarfile.open(T/'SOURCE_AFTER.tar.gz') as archive:
 old_roadmap=archive.extractfile(roadmap).read().decode()
section='\n## 0. 입력·권위·증거 수준\n'
assert section in old_roadmap and old_roadmap.split(section,1)[1]==(ROOT/roadmap).read_text().split(section,1)[1]
frozen_records={row['path']:row for row in json.loads((T/'RECORDS.json').read_text())}
def unchanged_record(path):
 row=frozen_records[str(path.relative_to(ROOT))]
 assert sha(path)==row['sha256'] and path.stat().st_size==row['bytes'],str(path)
for row in current['files']:
 assert sha(ROOT/row['path'])==row['sha256']
with tarfile.open(T/'RECORDS.tar.gz') as archive:
 def original(name):
  return json.load(archive.extractfile(str((S/name).relative_to(ROOT))))
 old_selection=original('EVIDENCE_SELECTION.json')
 old_impact=original('PREDECESSOR_IMPACT.json')
 old_coverage=original('COVERAGE.json')
 old_service=original('SERVICE_REUSE.json')
 old_gex=original('GEX_10_RESULT.json')
selection=copy.deepcopy(old_selection)
suite_names=['core_default','core_fixtures','core_observation','host_default','host_observation']
static_names=['fmt','check','event_log','clippy_core','clippy_host','heap']
selection['suites']=['integrated_'+n+'009' for n in suite_names]
selection['checks']=['integrated_'+n+'009' for n in static_names]+old_selection['checks'][6:]+[
 'f01_public_consumer_001','f01_reproduce_001','f01_public_release_build_001']
selection['public_receipts']=[n if n!='public_starter_schema_002' else 'public_starter_schema_003' for n in old_selection['public_receipts']]
selection['public_summaries']['starter']='evidence/public_starter_schema_003/SUMMARY.json'
selection['public_summaries']['f01_reproduction']='evidence/f01_reproduce_001/SUMMARY.json'
selection['source_sha256']=current['sha256']
selection['evidence_reuse']={'path':'correction/f01_001/EVIDENCE_REUSE.json','sha256':sha(reuse_path),'receipts':sorted(reuse['reused_receipts'])}
selection['supersession']='Five final009 suites and static/heap checks are fresh on corrected source. Fresh public CoreApplication/strict-authoring and exact auditor-probe evidence closes the implementation F01 boundary. Thirty previous executions are explicitly reused only through the bounded source/input impact assessment; their source hashes/raws remain original. Independent closure is pending.'
for key,names in selection['acceptance_public_receipts'].items():
 selection['acceptance_public_receipts'][key]=[n.replace('public_starter_schema_002','public_starter_schema_003') for n in names]
for key in ['V05','V15','V19','V20']:
 selection['acceptance_public_receipts'].setdefault(key,[])
 selection['acceptance_public_receipts'][key]=sorted(set(selection['acceptance_public_receipts'][key]+['f01_public_consumer_001','f01_reproduce_001','public_starter_schema_003']))
selection['acceptance_public_receipts']['V19']=selection['checks'][6:]+selection['public_receipts']
bindings={};logs={};counts={}
all_names=selection['suites']+selection['checks']+selection['public_receipts']+[selection['r05_receipt']]
for name in all_names:
 p=E/(name+'.json');r=json.loads(p.read_text());log=E/(name+'.log')
 assert r['exit_code'] in ((0,101) if name==selection['r05_receipt'] else (0,)),name
 assert r['source_unchanged'] and r['source_before']==r['source_after'] and r['log_sha256']==sha(log),name
 if name in reuse['reused_receipts']:
  unchanged_record(p);unchanged_record(log)
  old=reuse['reused_receipts'][name]
  assert r['source_before']==old_product and sha(p)==old['receipt_sha256'] and sha(log)==old['log_sha256'],name
  mode='reused / proven non-impacting F01 delta'
 else:
  assert r['source_before']==current,name
  mode='fresh current-source execution'
 bindings[name]={'receipt_sha256':sha(p),'log_sha256':sha(log),'source_sha256':r['source_before']['sha256'],'applicable_current_source_sha256':current['sha256'],'raw_exit_code':r['exit_code'],'evidence_mode':mode}
 if name in selection['suites']:
  text=log.read_text();logs[name]=set(re.findall(r'^test (\S+) \.\.\. ok$',text,re.M))
  totals=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',text)
  counts[name]={k:sum(int(row[i]) for row in totals) for i,k in enumerate(['passed','failed','ignored'])}
  assert counts[name]['passed']>0 and counts[name]['failed']==0
assert set(reuse['reused_receipts'])<=set(all_names)
for key,path in selection['public_summaries'].items():
 if key not in ('starter','f01_reproduction'):unchanged_record(HERE/path)
 summary=json.loads((HERE/path).read_text());assert summary.get('passed',True) and not summary.get('failed',False),key
pool=set.union(*logs.values());core=set.union(*(v for n,v in logs.items() if 'core_' in n));host=set.union(*(v for n,v in logs.items() if 'host_' in n))
def bound(selectors):
 assert set(selectors)<=pool,sorted(set(selectors)-pool)
 return {s:[n for n,v in logs.items() if s in v] for s in selectors}
def select(fragments,pool):
 result=set()
 for fragment in fragments:
  found={s for s in pool if fragment in s};assert found,fragment;result.update(found)
 return sorted(result)
def replace_receipt(n):
 if n.startswith('integrated_') and n.endswith('008'):return n[:-3]+'009'
 return n.replace('public_starter_schema_002','public_starter_schema_003')
impact=copy.deepcopy(old_impact)
impact['source_sha256']=current['sha256']
impact['f01_delta']={'record':selection['evidence_reuse'],'prior_independent_judgments':str((A/'CONDITION_JUDGMENTS.json').relative_to(ROOT)),'prior_independent_judgments_sha256':sha(A/'CONDITION_JUDGMENTS.json'),'meaning':'All original inherited meanings remain. F01 changes only ineffective basic-Heal admission; fresh009 native selectors revalidate affected consumers, unchanged external executions retain explicit impact-checked reuse.'}
for row in impact['conditions']:
 row['selector_receipts']=bound(row['current_selectors'])
 if 'direct_receipts' in row:row['direct_receipts']=[replace_receipt(n) for n in row['direct_receipts']]
 if row['id'] in ('X09','X10','PE-CRSO-12'):row['direct_receipts']=all_names
 if 'final_artifact_refs' in row:row['final_artifact_refs']=[n.replace('target/u06_001/','target/u06_002/') for n in row['final_artifact_refs']]
 row['correction_validation']='Fresh final009 selector executions; external direct receipt applicability is bound by COVERAGE.evidence_reuse. First audit judgments are prior evidence, not a new positive verdict.'
assert len(impact['conditions'])==141 and len(impact['r06_acceptances'])==31
for row in impact['original_requirement_sources']:assert sha(ROOT/row['path'])==row['sha256']
coverage=copy.deepcopy(old_coverage)
coverage.update(source_sha256=current['sha256'],suite_counts=counts,receipt_bindings=bindings,public_summaries=selection['public_summaries'],evidence_reuse=selection['evidence_reuse'])
coverage['validations']=[]
for i,(key,(cf,hf)) in enumerate(CONTRACTS.items()):
 cf=list(cf)
 if key in ('V15','V20'):cf+=['u06_basic_heal_public_query_and_clock_agree_on_current_recovery']
 cs,hs=select(cf,core),select(hf,host)
 coverage['validations'].append({'id':key,'requirement':coverage['acceptances'][i]['Observable Policy Acceptance'],'core_selectors':cs,'host_selectors':hs,'selector_receipts':bound(cs+hs),'direct_receipts':selection['acceptance_public_receipts'].get(key,[]),'status':'F01-corrected self-verification; fresh independent completion judgment pending'})
coverage['performance_environment']['evidence_mode']='reused original observation; no new timing measurement'
coverage['performance_environment']['original_source_sha256']=old_product['sha256']
for key in ['pc_comparison','r05_summary']:
 coverage[key].update(actual_execution_source_sha256=old_product['sha256'],evidence_mode='reused / proven non-impacting F01 delta',applicability_record=selection['evidence_reuse'])
coverage['compatibility_adaptation_mode']='Original source-bound adaptation record retained; all input/harness bytes rechecked by F01 EVIDENCE_REUSE, with no old execution relabelled current.'
coverage['limitations'].append('F01 reuses30 old executions through exact three-file impact analysis. The old measured source digest is retained; PC/R05 Damage-only workloads and all layout/accounting inputs are unchanged. Original independent audit is Not Complete for F01, not a positive completion basis.')
C=S/'compatibility/current_001';CE=C/'evidence'
for path in [CE/selection['pc_summary'],CE/selection['pc_cost'],CE/selection['pc_raw'],CE/selection['r05_summary'],C/'ADAPTATION_FINAL.json']:
 unchanged_record(path)
pc=json.loads((CE/selection['pc_summary']).read_text());accounted=json.loads((CE/selection['pc_cost']).read_text())
assert pc['same_gameplay_oracle_and_typed_refusal'] and pc['rows_each']==576
assert (pc['distinct_conditions'],pc['successful_conditions'],pc['typed_refusal_conditions'])==(36,31,5)
assert pc['current']['sha256']==sha(CE/selection['pc_raw'])==accounted['after']['sha256']
assert accounted['passed'] and accounted['rows']==576 and len(accounted['per_row_cost_changes'])==576
for path,digest in accounted['source_basis'].items():assert sha(ROOT/path)==digest
r05=json.loads((CE/selection['r05_summary']).read_text())
assert r05['source_digest']==old_product['sha256'] and not r05['other_hard_guard_failures']
assert r05['classification']=='warning_concurrent_peak_only'
adaptation=json.loads((C/'ADAPTATION_FINAL.json').read_text())
assert adaptation['source_sha256']==old_product['sha256']
for row in adaptation['files']:assert sha(ROOT/row['copy'])==row['copy_sha256']
assert coverage['compatibility_adaptation_sha256']==sha(C/'ADAPTATION_FINAL.json')
service=copy.deepcopy(old_service)
for row in service['unchanged_service_inputs']:assert sha(ROOT/row['path'])==row['sha256']
for key in ['reused_audit','reused_service_evidence','reused_master_source_review']:assert sha(ROOT/service[key])==service[key+'_sha256']
service.update(source_sha256=current['sha256'],current_validation=selection['suites']+selection['checks']+selection['public_receipts'],f01_evidence_reuse=selection['evidence_reuse'])
service['changed_producer']+=' F01 only guards ineffective basic Heal through one pure amount calculation; new positive/zero producers and public query are directly verified, and final native/host/heap suites rerun. Old fault/service external records retain exact non-impacting applicability rather than being relabelled current executions.'
fault=json.loads((HERE/selection['public_summaries']['fault']).read_text());assert fault['passed'] and fault['source']['sha256']==old_product['sha256']
gex=copy.deepcopy(old_gex)
gex.update(source_sha256=current['sha256'],recorded_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),suite_counts=counts,public_summaries=selection['public_summaries'],evidence_reuse=selection['evidence_reuse'])
gex['status']='F01 correction implementation and self-verification complete; fresh independent GEX-20 pending'
gex['r05_execution_source_sha256']=old_product['sha256']
gex['finding_correction']={'id':'U06-F01','prior_audit':str((A/'AUDIT_CLOSURE.json').relative_to(ROOT)),'prior_audit_sha256':sha(A/'AUDIT_CLOSURE.json'),'scope':'single/area current positive Heal amount at automatic target selection, public query and release; coefficient schema preserved','direct_current_receipts':['f01_public_consumer_001','f01_reproduce_001','public_starter_schema_003'],'native_current_suites':selection['suites'],'independent_finding_closed':False}
gex['next']='Freeze target/u06_002, independent implementation-nonparticipant GPT-6 Astra / High fresh GEX-20; U07 remains ineligible until Complete / High.'
def encoded(value):return json.dumps(value,ensure_ascii=False,indent=2)+'\n'
import hashlib
coverage['predecessor_impact_sha256']=hashlib.sha256(encoded(impact).encode()).hexdigest()
artifacts={'EVIDENCE_SELECTION.json':selection,'PREDECESSOR_IMPACT.json':impact,'COVERAGE.json':coverage,'SERVICE_REUSE.json':service,'GEX_10_RESULT.json':gex}
assert identity()==current
backup=F/'BEFORE_ASSEMBLY';backup.mkdir(exist_ok=False)
for name in artifacts:
 (backup/name).write_bytes((S/name).read_bytes())
for name,value in artifacts.items():(S/name).write_text(encoded(value))
result={'source_sha256':current['sha256'],'fresh_receipts':[n for n in all_names if n not in reuse['reused_receipts']],'reused_receipts':list(reuse['reused_receipts']),'counts':counts,'evidence_reuse_sha256':sha(reuse_path),'artifacts':{name:sha(S/name) for name in artifacts},'preserved_pre_f01_artifacts':{name:sha(backup/name) for name in artifacts},'canonical_docs':'All bound non-product source documents unchanged except ROADMAP progress preamble; ROADMAP from section0 onward byte-identical. Current status-only records are attributed separately.','independent_verdict_pending':True}
with (F/'ASSEMBLY.json').open('x') as stream:stream.write(encoded(result))
print(json.dumps({'fresh':len(result['fresh_receipts']),'reused':len(result['reused_receipts']),'source':current['sha256'],'counts':counts}))
