"""Select only completed executions on the final U06 product source."""
import json
from run import ROOT,HERE,identity,sha
S=HERE/'execution/single_goal';C='execution/single_goal/compatibility/current_001/'
suites=['integrated_'+n+'008' for n in ['core_default','core_fixtures','core_observation','host_default','host_observation']]
checks=['integrated_'+n+'008' for n in ['fmt','check','event_log','clippy_core','clippy_host','heap']]
checks+=['compat_pc_prepare_003','compat_pc_001','compat_pc_gameplay_001','compat_pc_cost_002','compat_r05_summary_001']
checks+=['public_guards001','public_guards_allocator001']
public=['compat_'+n+'_001' for n in ['prepared_public','prepared_boundary','input_validation','default_parity','public_oracle','public_trace','public_cli','public_profile','public_boundary','public_api']]
public+=['public_starter_schema_002']+['public_'+n+'_001' for n in ['schema','independent_area','continuation','rewind']]
public+=['final_fault_001_'+n for n in ['build_on','none','detached','queue_full','on','websocket','build_off','off']]
summaries={
 'prepared_host':C+'public_probe_001/RESULT.json',
 'prepared_boundary':C+'boundary_001/RESULT.json',
 'input_validation':C+'input_validation_001/SUMMARY.json',
 'defaults':C+'evidence/DEFAULT_NUMBERS_001.json',
 'oracle':C+'evidence/public_after/SUMMARY.json',
 'trace':C+'evidence/public_after_trace/SUMMARY.json',
 'cli':C+'evidence/cli_current/CLI_FINAL_SUMMARY_002.json',
 'profile':C+'evidence/public_profile_001/SUMMARY.json',
 'boundary':C+'evidence/public_boundary_001/SUMMARY.json',
 'api':C+'evidence/public_api/RESULT.json',
 'starter':'evidence/public_starter_schema_002/SUMMARY.json',
 'schema':'evidence/public_schema_001/SUMMARY.json',
 'area':'evidence/public_independent_area_001/SUMMARY.json',
 'continuation':'evidence/public_continuation_001/SUMMARY.json',
 'rewind':'evidence/public_rewind_001/SUMMARY.json',
 'fault':'evidence/final_fault_001_SUMMARY.json',
 'fault_websocket':'evidence/public_fault_final_001/SUMMARY.json',
 'public_cost_guards':'evidence/public_guards001_raw.json',
 'public_allocator_guards':'evidence/public_guards_allocator001_raw.json',
}
current=identity()
for name in suites+checks+public:
 r=json.loads((HERE/'evidence'/(name+'.json')).read_text())
 assert r['exit_code']==0 and r['source_unchanged'] and r['source_before']==current==r['source_after'],name
 assert r['log_sha256']==sha(HERE/'evidence'/(name+'.log')),name
for key,path in summaries.items():
 d=json.loads((HERE/path).read_text());assert d.get('passed',True) and not d.get('failed',False),key
result={'source_sha256':current['sha256'],'suites':suites,'checks':checks,'public_receipts':public,'public_summaries':summaries,'pc_raw':'pc_raw_current_001.json','pc_summary':'PC_SUMMARY_CURRENT_001.json','pc_cost':'PC_IMMEDIATE_ACCOUNTED_U06.json','r05_summary':'R05_SUMMARY_CURRENT_003.json','r05_receipt':'compat_r05_002','acceptance_public_receipts':{
 'V04':['public_starter_schema_002','compat_input_validation_001'],
 'V05':['public_starter_schema_002','public_independent_area_001'],
 'V06':['public_starter_schema_002'],
 'V18':['public_continuation_001','public_rewind_001','final_fault_001_websocket'],
 'V19':checks[6:]+public,
},'supersession':'Earlier partial tests, failed attempts and DEFAULT_LIBRARY_REUSE_001 remain historical. The final implicit-Damage serializer correction is covered by all five fresh full suites; no earlier library reuse is claimed.'}
with (S/'EVIDENCE_SELECTION.json').open('x') as f:json.dump(result,f,ensure_ascii=False,indent=2);f.write('\n')
print(json.dumps({'suites':len(suites),'checks':len(checks),'public_receipts':len(public),'source':current['sha256']}))
