"""Rebuild and bind the public fault consumers to final integrated source."""
import hashlib,json,subprocess,sys
from pathlib import Path
from initialize import ROOT,HERE
from run import identity
label=sys.argv[1] if len(sys.argv)>1 else '001'
prefix='final_fault_'+label+'_'
E=HERE/'evidence'
P=Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_resonance_lifecycle/execution/single_goal/compatibility/current_001/inputs')/'core_enhanced/docs/goals/core_runtime_safety_observability/evidence/public_profile_002'
B=ROOT/'.cache/d07-runtime-amount-product-target/release/examples/fault_runtime'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
before=identity()
def run(name,args):
    r=subprocess.run([sys.executable,str(HERE/'tools/run.py'),prefix+name,'--',*map(str,args)])
    assert r.returncode==0,(name,r.returncode)
run('build_on',['cargo','build','--release','-p','game_server','--features','runtime-observation','--example','fault_runtime','--offline','--locked'])
observed_sha=sha(B)
rows={}
for mode in ['none','detached','queue_full','on']:
    run(mode,[B,P/'content',P/'selection.json',E/'observation_fault_001.ron',mode,'1000'])
    rows[mode]=[json.loads(line) for line in (E/(prefix+mode+'.log')).read_text().splitlines() if line.startswith('{')]
run('websocket',['python3',HERE/'tools/public_fault_probe.py','final_'+label,'1000'])
run('build_off',['cargo','build','--release','-p','game_server','--example','fault_runtime','--offline','--locked'])
off_sha=sha(B)
run('off',[B,P/'content',P/'selection.json',E/'observation_fault_001.ron','off','1000'])
rows['off']=[json.loads(line) for line in (E/(prefix+'off.log')).read_text().splitlines() if line.startswith('{')]
results={mode:next(x['result'] for x in items if 'result' in x) for mode,items in rows.items()}
assert all(x==results['off'] for x in results.values())
assert next(x['queue_dropped'] for x in rows['queue_full'] if 'queue_dropped' in x)>0
before_obs=next(x['before_observation'] for x in rows['on'] if 'before_observation' in x)
after_obs=next(x['after_observation'] for x in rows['on'] if 'after_observation' in x)
assert before_obs['selected_session'] is not None and after_obs['selected_session'] is None
assert before_obs['sessions'][0]['commits']==after_obs['sessions'][0]['commits']
assert after_obs['sessions'][0]['current']['battle'] is None
assert before==identity()
result={'passed':True,'source':before,'mode_result_exact_equality':list(results),'on_binary_sha256':observed_sha,'off_binary_sha256':off_sha,'external_input_files':{str(p.relative_to(ROOT)):sha(p) for p in P.rglob('*') if p.is_file() and (p.is_relative_to(P/'content') or p.name=='selection.json')},'websocket_summary':'public_fault_final_'+label+'/SUMMARY.json','observation_commits':after_obs['sessions'][0]['commits'],'selected_ended':True,'receipt_prefix':prefix}
with (E/(prefix+'SUMMARY.json')).open('x') as f:json.dump(result,f,indent=2)
print('PASS final native fault exact parity across five observation modes and stock websocket recovery')
