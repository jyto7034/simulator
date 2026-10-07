import json,hashlib
from pathlib import Path
R=Path('/mnt/f/work/simulator');C=R/'core_enhanced/docs/goals/core_runtime_safety_observability/execution/s1/corrections/f02_persistence_entry_001';A=Path(__file__).parent;E=C/'evidence';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
profile=json.loads((E/'public_profile_001/SUMMARY.json').read_text());boundary=json.loads((E/'public_boundary_001/SUMMARY.json').read_text());api=json.loads((E/'public_api/RESULT.json').read_text())
assert profile['passed'] and len(profile['commands'])==24 and len(profile['hosts'])==5
for data in [profile,boundary]:
 for n,digest in data['input_files'].items():assert sha(R/n)==digest
for row in profile['commands']:
 for suffix in ['stdout','stderr']:assert sha(E/'public_profile_001'/(row['name']+'.'+suffix))==row[suffix+'_sha256']
for row in profile['hosts']:
 h=E/'public_profile_001'/('host_'+row['name']);assert sha(h/'config/development.toml')==row['config_sha256'];assert sha(h/'process.log')==row['process_log_sha256']
 if row['name']=='invalid':assert row['exit_code']==1 and row['message_count']==0 and 'battle_history.max_events' in (h/'process.log').read_text()
 else:assert row['exit_code']==0 and row['message_count']==5 and row['expected_profile'] in (h/'process.log').read_text()
assert len(boundary['cases'])==13 and boundary['passed']
for row in boundary['cases']:
 log=E/'public_boundary_001'/(row['name']+'.log');assert sha(log)==row['log_sha256'];assert row['exit_code']==row['expected']
 if row['code']:assert row['code'] in log.read_text() and row['diagnostic'] in log.read_text()
assert api['passed'] and api['commands']==21
for row in api['files']:assert sha(E/'public_api'/row['path'])==row['sha256']
(A/'PORTS_VERIFICATION.json').write_text(json.dumps(dict(profile_cli_commands=24,local_hosts=5,profile_inputs_outputs_verified=True,old_boundary_cases=13,public_api_cli_commands=21,all_hashes_verified=True),indent=2)+'\n');print('PASS current public profile/host/boundary/API artifacts')
