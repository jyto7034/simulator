from pathlib import Path
import json, os, subprocess, time, statistics, hashlib
R=Path('/mnt/f/work/simulator');E=R/'core_enhanced/docs/goals/core_expedition_deterrence_rewards/execution/single_goal'
out=Path('/tmp/u14-correction-measurements001');out.mkdir()
binary=E/'correction/u14_correction_001/artifacts/u14_correction_probe'
env=os.environ.copy();env['U14_PROBE_INPUTS']=str(E/'test_parts/public_inputs001');env.pop('U14_COMMAND_TRACE',None)
samples=[]
for case in ['official','expanded']:
 for index in range(3):
  path=out/f'{case}_{index+1}.json';cmd=[str(binary),'measure',case,str(path)];start=time.monotonic()
  p=subprocess.run(cmd,env=env,capture_output=True)
  (out/f'{case}_{index+1}.stdout').write_bytes(p.stdout);(out/f'{case}_{index+1}.stderr').write_bytes(p.stderr)
  record={'argv':cmd,'exit_code':p.returncode,'seconds':time.monotonic()-start}
  (out/f'{case}_{index+1}.execution.json').write_text(json.dumps(record,indent=2));assert p.returncode==0,record
  value=json.loads(path.read_text());assert value['passed'] and value['exact_requested_bytes_recovered']
  assert all(s['ok'] for s in value['stages']);samples.append(value)
summary={case:{s['name']:{'median_seconds':statistics.median(v['seconds'] for x in samples if x['case']==case for v in x['stages'] if v['name']==s['name']),
 'max_seconds':max(v['seconds'] for x in samples if x['case']==case for v in x['stages'] if v['name']==s['name'])} for s in next(x for x in samples if x['case']==case)['stages']} for case in ['official','expanded']}
result={'passed':True,'samples':len(samples),'all_exact_requested_bytes_recovered':True,'summary':summary,
 'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'input':'test_parts/public_inputs001',
 'scope':'full authored static graph, initial room manifested; six fresh processes after compilation and host execution finished; not all rooms visited or final performance SLO',
 'native_counters':'Raw load stage is measured. Unavailable operation counters remain unavailable; do not interpret zero raw counters without coverage as zero cost. Actual movement-budget regression is in correction core002; G11 default and runtime-observation cases are rerun in correction.',
 'source_currentness':'Correction consumer compiled after room-context fix; runtime inputs fixed through this run. Exact successor source and build receipt retained.'}
(out/'RESULT.json').write_text(json.dumps(result,indent=2));print(json.dumps(result))
