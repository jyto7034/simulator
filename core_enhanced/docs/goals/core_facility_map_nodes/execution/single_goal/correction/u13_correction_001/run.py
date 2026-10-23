from pathlib import Path
import subprocess,time,json,os,sys
E=Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_facility_map_nodes/execution/single_goal/correction/u13_correction_001')
label=sys.argv[1];cmd=sys.argv[2:]
env=os.environ.copy();env.update(PATH='/home/blast/.cargo/bin:'+env['PATH'],CARGO_HOME='/tmp/wp02-cargo-home')
t=time.monotonic()
with (E/(label+'.stdout.log')).open('x') as out,(E/(label+'.stderr.log')).open('x') as err:
 p=subprocess.Popen(cmd,env=env,stdout=out,stderr=err,start_new_session=True)
 print('started',p.pid,flush=True)
 while p.poll() is None:
  try:p.wait(timeout=60)
  except subprocess.TimeoutExpired:
   print('elapsed',round(time.monotonic()-t),'seconds',flush=True)
   subprocess.run(['ps','-o','pid,ppid,stat,etime,pcpu,comm','--ppid',str(p.pid)],check=False)
   print((E/(label+'.stderr.log')).read_text()[-1800:],flush=True)
r={'command':cmd,'exit_code':p.returncode,'seconds':time.monotonic()-t,'stdout':label+'.stdout.log','stderr':label+'.stderr.log'}
(E/(label+'.result.json')).write_text(json.dumps(r,indent=2));print(r,flush=True)
print((E/(label+'.stderr.log')).read_text()[-6000:])
