import sys
from finalization_paths import paths
_, final_names = paths(sys.argv[1] if len(sys.argv) > 1 else '001')
"""Finalize U07 evidence only after current-source execution and every public/performance gate."""
import copy,json,re
from run import ROOT,HERE,identity,sha
from coverage_contracts import CONTRACTS,DIMENSIONS
S=HERE/'execution/single_goal';E=HERE/'evidence';C=S/'compatibility/current_001'
selection=json.loads((S/final_names['selection']).read_text());current=identity();bindings={}
def write(path,value):
 with path.open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
def receipt(name,allowed=(0,)):
 path=E/(name+'.json');r=json.loads(path.read_text());log=E/(name+'.log')
 assert r['exit_code'] in allowed and r['source_unchanged'],name
 assert r['source_before']==current==r['source_after'],name
 assert r['log_sha256']==sha(log),name
 bindings[name]={'receipt_sha256':sha(path),'log_sha256':sha(log),'source_sha256':current['sha256'],'raw_exit_code':r['exit_code']}
 return r
passed={};counts={}
for name in selection['suites']:
 receipt(name);text=(E/(name+'.log')).read_text()
 passed[name]=set(re.findall(r'^test (\S+) \.\.\. ok$',text,re.M))
 totals=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',text)
 counts[name]={k:sum(int(row[i]) for row in totals) for i,k in enumerate(['passed','failed','ignored'])}
 assert counts[name]['passed']>0 and counts[name]['failed']==0,name
core=set.union(*(v for n,v in passed.items() if 'core_' in n));host=set.union(*(v for n,v in passed.items() if 'host_' in n))
for name in selection['checks']+selection['public_receipts']:receipt(name)
# Final integrated runs must match the current source exactly.
assert any('core_default' in name for name in selection['suites'])
for key,path in selection['public_summaries'].items():
 summary=json.loads((HERE/path).read_text());assert summary.get('passed',True),key
# Numeric guards retain raw and immediate predecessor cost changes.
pc_raw=C/'evidence'/selection['pc_raw'];pc_path=C/'evidence'/selection['pc_summary'];pc=json.loads(pc_path.read_text())
assert pc['same_gameplay_oracle_and_typed_refusal'] and pc['rows_each']==576
assert (pc['distinct_conditions'],pc['successful_conditions'],pc['typed_refusal_conditions'])==(36,31,5)
assert pc['current']['sha256']==sha(pc_raw)
accounted_path=C/'evidence'/selection['pc_cost'];accounted=json.loads(accounted_path.read_text())
assert accounted['passed'] and accounted['rows']==576 and accounted['after']['sha256']==sha(pc_raw)
assert accounted['before']['sha256']==sha(ROOT/accounted['before']['path'])
assert accounted['source_sha256']==current['sha256'] and accounted['expected_owner_extent_delta']==200
assert accounted['pc_receipt']['sha256']==sha(ROOT/accounted['pc_receipt']['path'])
assert json.loads((ROOT/accounted['pc_receipt']['path']).read_text())['source_after']==current
assert all(sha(ROOT/p)==row['sha256'] for p,row in accounted['source_owner_proof'].items())
assert len(accounted['controls'])>=18 and accounted['controls'][0]['passed']
assert all(row['rejected'] for row in accounted['controls'][1:])
assert len(accounted['per_row_cost_changes'])==576
r05_path=C/'evidence'/selection['r05_summary'];r05=json.loads(r05_path.read_text())
receipt(selection['r05_receipt'],(0,101));assert r05['source_digest']==current['sha256'] and not r05['other_hard_guard_failures']
import sys
sys.path.insert(0,str(C))
from r05_harness import verify
r05_proof_path,r05_proof=verify(selection['labels']['r05'],current['sha256'])
assert r05['harness_proof']['sha256']==sha(r05_proof_path)
assert C/'evidence'/selection['r05_harness']==r05_proof_path
if selection.get('r05_execution'):
 execution_path=C/'evidence'/selection['r05_execution'];execution=json.loads(execution_path.read_text())
 assert execution['passed'] and execution['source_sha256']==current['sha256'] and execution['target_was_absent']
 assert execution['harness_proof_sha256']==sha(r05_proof_path) and execution['depinfo_sha256']==sha(execution_path.parent/'FRESH_TARGET_DEPINFO.d')
 assert execution['receipt_sha256']==sha(ROOT/execution['receipt']) and execution['summary_sha256']==sha(r05_path)
 assert execution['environment_sha256']==sha(ROOT/execution['environment'])
assert r05['raw_sha256']==sha(C/'evidence'/f"r05_raw_current_{selection['labels']['r05']}.json")
adaptation_path=S/selection['adaptation'];adaptation=json.loads(adaptation_path.read_text());assert adaptation['source_sha256']==current['sha256']
for row in adaptation['files']:assert sha(ROOT/row['copy'])==row.get('current_copy_sha256',row['copy_sha256'])
impact=json.loads((S/final_names['impact']).read_text());assert impact['source_sha256']==current['sha256'] and len(impact['conditions'])==161 and len(impact['r06_acceptances'])==31
freeze=json.loads((HERE/'P2G_30_FREEZE_001.json').read_text())['artifacts'][0];acceptances=copy.deepcopy(freeze['payload']['Observable Policy Acceptance Set']);assert len(acceptances)==20
def select(fragments,pool):
 result=set()
 for fragment in fragments:
  found={s for s in pool if fragment in s};assert found,fragment;result.update(found)
 return sorted(result)
validations=[]
for i,(key,(c,h)) in enumerate(CONTRACTS.items()):
 cs,hs=select(c,core),select(h,host);selectors=cs+hs
 validations.append({'id':key,'requirement':acceptances[i]['Observable Policy Acceptance'],'core_selectors':cs,'host_selectors':hs,'selector_receipts':{s:[n for n,v in passed.items() if s in v] for s in selectors},'direct_receipts':selection['acceptance_public_receipts'].get(key,[]),'status':'implementation self-verification; independent completion verdict pending'})
 acceptances[i].update(validation_refs=[key],status='self-verification bound; independent audit pending')
result={'goal':'GOAL-CIOB-01','goal_set':'GS-CIOB-001','source_sha256':current['sha256'],'frozen_policy_ref':freeze['ref'],'status':'GEX-10 self-verification; GEX-20 independent audit pending','acceptances':acceptances,'validations':validations,'required_dimensions':DIMENSIONS,'suite_counts':counts,'receipt_bindings':bindings,'state_inventory':'STATE_INVENTORY.md','predecessor_impact_sha256':sha(S/final_names['impact']),'public_summaries':selection['public_summaries'],'pc_comparison':{'gameplay_sha256':sha(pc_path),'cost_sha256':sha(accounted_path),'raw_sha256':sha(pc_raw),'claim':accounted['claim']},'r05_summary':{'path':str(r05_path.relative_to(S)),'sha256':sha(r05_path)},'compatibility_adaptation_sha256':sha(adaptation_path),'limitations':['Local Linux CoreApplication, real host actor, stock WebSocket and process/file boundaries. No Unity or deployed service claim.','Only inherited R05 concurrent-peak warning and20minute stress exclusion apply; no numeric cap changed.','Object tests use normal grants/equipment/copies, with explicitly recorded readiness, reward, unlocked-form and standalone wall fixtures. No new recipe/reward or infection producer claim.','Shared action/CC/work gates retain generic regression evidence; selector links do not claim every Cartesian combination was separately run.','V20 Complete/High and unresolved-findings gate is reserved for independent audit; no dependent U08 work before it.']}
environments=[]
for relative in selection['performance_environment']:
 environment_path=S/relative;environment=json.loads(environment_path.read_text())
 assert environment['source_sha256']==current['sha256']
 environments.append({'path':relative,'sha256':sha(environment_path),'phase':environment['phase'],'other_training_present':environment['other_training_present'],'timing_claim':environment['timing_claim']})
assert {row['phase'] for row in environments}=={'pc','r05'}
result['performance_environments']=environments
if 'audit_correction' in selection:
 correction=selection['audit_correction'];proof_path=ROOT/correction['path'];proof=json.loads(proof_path.read_text())
 assert sha(proof_path)==correction['sha256'] and proof['source_after']==current
 assert sha(ROOT/proof['closed_audit'])==proof['closed_audit_sha256']
 for name,digest in proof['proofs'].items():assert sha(proof_path.parent/name)==digest,name
 result['audit_correction']=correction
if 'withdrawn_investigation' in selection:
 investigation=selection['withdrawn_investigation'];proof_path=ROOT/investigation['path'];proof=json.loads(proof_path.read_text())
 assert sha(proof_path)==investigation['sha256'] and proof['source_after']==current and proof['production_change'] is False
 for name,digest in proof['proofs'].items():assert sha(proof_path.parent/name)==digest,name
 result['withdrawn_investigation']=investigation
result['compatibility_adaptation']={'path':selection['adaptation'],'sha256':sha(adaptation_path)}
result['limitations'].append('PC elapsed times are shared-host observations; the original R05 allocation-instrumented run has latency_gate_applicable=false. Raw latency results remain visible, and no numeric allocation ceiling or exception was changed.')
write(S/final_names['coverage'],result)
print(json.dumps({'source':current['sha256'],'acceptances':20,'inherited':161,'r06':31,'counts':counts}))
