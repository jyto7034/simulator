"""Finalize U06 evidence only after current-source execution and every public/performance gate."""
import copy,json,re
from run import ROOT,HERE,identity,sha
from coverage_contracts import CONTRACTS,DIMENSIONS
S=HERE/'execution/single_goal';E=HERE/'evidence';C=S/'compatibility/current_001'
selection=json.loads((S/'EVIDENCE_SELECTION.json').read_text());current=identity();bindings={}
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
# Earlier partial/default reuse is historical; the serializer correction invalidates it.
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
assert all(sha(ROOT/p)==h for p,h in accounted['source_basis'].items())
assert len(accounted['per_row_cost_changes'])==576
r05_path=C/'evidence'/selection['r05_summary'];r05=json.loads(r05_path.read_text())
receipt(selection['r05_receipt'],(0,101));assert r05['source_digest']==current['sha256'] and not r05['other_hard_guard_failures']
adaptation=json.loads((C/'ADAPTATION_FINAL.json').read_text());assert adaptation['source_sha256']==current['sha256']
for row in adaptation['files']:assert sha(ROOT/row['copy'])==row['copy_sha256']
impact=json.loads((S/'PREDECESSOR_IMPACT.json').read_text());assert impact['source_sha256']==current['sha256'] and len(impact['conditions'])==141 and len(impact['r06_acceptances'])==31
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
result={'goal':'GOAL-CBST-01','goal_set':'GS-CBST-001','source_sha256':current['sha256'],'frozen_policy_ref':freeze['ref'],'status':'GEX-10 self-verification; GEX-20 independent audit pending','acceptances':acceptances,'validations':validations,'required_dimensions':DIMENSIONS,'suite_counts':counts,'receipt_bindings':bindings,'state_inventory':'STATE_INVENTORY.md','predecessor_impact_sha256':sha(S/'PREDECESSOR_IMPACT.json'),'public_summaries':selection['public_summaries'],'pc_comparison':{'gameplay_sha256':sha(pc_path),'cost_sha256':sha(accounted_path),'raw_sha256':sha(pc_raw),'claim':accounted['claim']},'r05_summary':{'path':str(r05_path.relative_to(S)),'sha256':sha(r05_path)},'compatibility_adaptation_sha256':sha(C/'ADAPTATION_FINAL.json'),'limitations':['Local Linux CoreApplication, real host actor, stock WebSocket and process/file boundaries. No Unity or deployed service claim.','Only inherited R05 concurrent-peak warning and20minute stress exclusion apply; no numeric cap changed.','Direct starter tests use normal grants/equipment/copies with explicitly recorded readiness/injury/opponent position fixtures.','Shared action/CC/work gates retain generic regression evidence; selector links do not claim every Cartesian combination was separately run.','V20 Complete/High and unresolved-findings gate is reserved for independent audit; no dependent U07 work before it.']}
environment_path=C/'evidence/PERFORMANCE_ENVIRONMENT_001.json';environment=json.loads(environment_path.read_text())
assert environment['source_sha256']==current['sha256']
result['performance_environment']={'path':str(environment_path.relative_to(S)),'sha256':sha(environment_path),'other_training_present':environment['other_training_present'],'timing_claim':environment['timing_claim']}
result['limitations'].append('PC elapsed times are shared-host observations; the original R05 allocation-instrumented run has latency_gate_applicable=false. Raw latency results remain visible, and no numeric allocation ceiling or exception was changed.')
write(S/'COVERAGE.json',result)
print(json.dumps({'source':current['sha256'],'acceptances':20,'inherited':141,'r06':31,'counts':counts}))
