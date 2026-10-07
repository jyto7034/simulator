"""Rerun the exact closed independent F01 probe; preserve its input and actual output."""
import json,os,shutil,subprocess,sys
from run import HERE,ROOT,sha
label=sys.argv[1];out=HERE/('evidence/f01_reproduce_'+label);out.mkdir()
old=HERE/'post_audit/u06_20261005_001/probe';probe=out/'probe'
shutil.copytree(old,probe)
assert all(sha(p)==sha(old/p.relative_to(probe)) for p in probe.rglob('*') if p.is_file())
command=['cargo','run','--offline','--locked','--manifest-path',str(probe/'Cargo.toml')]
r=subprocess.run(command,cwd=ROOT,capture_output=True)
(out/'stdout.json').write_bytes(r.stdout);(out/'stderr.log').write_bytes(r.stderr)
assert r.returncode==0, r.stderr.decode(errors='replace')[-3000:]
x=json.loads(r.stdout)
for key in ('zero','coefficient_rounds_to_zero'):
 row=x[key];assert row['strict_decode_accepted'] and row['attack_starts']==0 and row['positive_hp_events']==0 and row['final_hp']==50,(key,row)
row=x['positive_control'];assert row['attack_starts']==4 and row['positive_hp_events']==4 and row['final_hp']==90,row
result={'passed':True,'command':command,'exit_code':r.returncode,'raw_sha256':sha(out/'stdout.json'),'build_log_sha256':sha(out/'stderr.log'),'exact_auditor_probe_input':{str(p.relative_to(probe)):sha(p) for p in probe.rglob('*') if p.is_file()},'cases':{k:{n:v for n,v in row.items() if n!='events'} for k,row in x.items()}}
(out/'SUMMARY.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
