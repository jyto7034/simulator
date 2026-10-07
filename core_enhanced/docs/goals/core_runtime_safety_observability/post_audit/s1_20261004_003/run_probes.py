import hashlib,json,subprocess,tempfile
from pathlib import Path
R=Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_runtime_safety_observability';C=H/'execution/s1/corrections/f02_persistence_entry_001';A=Path(__file__).parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
libs=json.loads((A/'CURRENT_LIBRARIES.json').read_text())
for row in libs.values():assert sha(Path(row['path']))==row['sha256']
host=(C/'host_peak_failure_probe.rs').read_text().replace('s1f02-host-', 's1f02-audit-host-')
host=host.replace('std::fs::create_dir(temp.join("active_run.tmp")).unwrap();','let bytes_before=std::fs::read(temp.join("active_run.json")).unwrap();\n   std::fs::create_dir(temp.join("active_run.tmp")).unwrap();')
host=host.replace('   let result=player_actor.send(', '   for attempt in 0..2 {\n   let result=player_actor.send(',1)
host=host.replace('   std::fs::remove_dir(temp.join("active_run.tmp")).unwrap();','   assert_eq!(std::fs::read(temp.join("active_run.json")).unwrap(),bytes_before);\n   println!("PASS failed attempt {}: exact root and on-disk bytes retained",attempt);\n   }\n   std::fs::remove_dir(temp.join("active_run.tmp")).unwrap();')
host=host.replace('assert!(checkpoint.run_peak_certified_power>before.run_peak_certified_power());','assert_eq!(before.run_peak_certified_power(),204);\n   assert_eq!(checkpoint.run_peak_certified_power,213);')
(A/'host_peak_failure_probe.rs').write_text(host)
rollback=(H/'post_audit/s1_20261004_002/rollback_battle_probe.rs').read_text();(A/'rollback_battle_probe.rs').write_text(rollback)
rows=[]
with tempfile.TemporaryDirectory(prefix='s1f02-audit-probes-') as tmp:
 for name in ['host_peak_failure_probe','rollback_battle_probe']:
  binary=Path(tmp)/name;source=A/(name+'.rs');command=['/home/blast/.cargo/bin/rustc','--edition=2021',str(source)]
  for lib,row in libs.items():command+=['--extern',lib+'='+row['path']]
  command+=['-L','dependency='+str(Path(libs['game_core']['path']).parent),'-o',str(binary)]
  compiled=subprocess.run(command,capture_output=True,cwd=R);(A/(name+'_compile.log')).write_bytes(compiled.stdout+compiled.stderr);assert compiled.returncode==0,compiled.stderr.decode()
  run=subprocess.run([str(binary)],capture_output=True,cwd=R);log=A/(name+'_run.log');log.write_bytes(run.stdout+run.stderr)
  rows.append(dict(name=name,command=command,compile_exit=compiled.returncode,run_exit=run.returncode,source_sha256=sha(source),binary_sha256=sha(binary),log_sha256=sha(log)))
  assert run.returncode==0,run.stdout.decode()+run.stderr.decode()
# Independently rerun the public prepared-boundary compiler cases against the verified artifacts.
boundary=json.loads((C/'boundary_001/RESULT.json').read_text());bd=A/'boundary';bd.mkdir(exist_ok=False)
for row in boundary['cases']:
 name=row['name'];source=C/'boundary_001'/(name+'.rs');assert sha(source)==row['source_sha256'];dest=bd/source.name;dest.write_bytes(source.read_bytes());command=['/home/blast/.cargo/bin/rustc','--edition=2021',str(dest),'-o',str(bd/name)]
 for lib in ['game_core','serde_json','uuid']:command+=['--extern',lib+'='+libs[lib]['path']]
 command+=['-L','dependency='+str(Path(libs['game_core']['path']).parent)]
 result=subprocess.run(command,capture_output=True,cwd=R);log=bd/(name+'.log');log.write_bytes(result.stdout+result.stderr)
 assert result.returncode==row['exit_code'];error=row['expected_error'];assert not error or ('error['+error+']').encode() in result.stderr
 rows.append(dict(name='boundary/'+name,command=command,exit_code=result.returncode,expected_error=error,source_sha256=sha(dest),log_sha256=sha(log)))
(A/'PROBES.json').write_text(json.dumps(dict(libraries=libs,rows=rows,passed=True,fixture_binding=str((C/'public_probe_001/RESULT.json').relative_to(R)),fixture_binding_sha256=sha(C/'public_probe_001/RESULT.json')),indent=2)+'\n')
print(json.dumps(dict(passed=True,runtime_probes=2,compile_boundary_cases=8)))
