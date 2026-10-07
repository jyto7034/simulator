from pathlib import Path
import subprocess,os,sys,json,datetime,hashlib
here=Path(__file__).resolve().parent
cfg=json.loads((here/'isolate.json').read_text())
tag,*command=sys.argv[1:]
sha=lambda b:hashlib.sha256(b).hexdigest()
def identities():
 return {str(p.relative_to(cfg['isolate'])):sha(p.read_bytes()) for p in [Path(cfg['isolate'])/f['path'] for f in cfg['source']]}
def live_identities():
 return {f['path']:sha((Path(cfg['root'])/f['path']).read_bytes()) for f in cfg['source']}
receipt=dict(command=command,cwd=cfg['isolate'],start=datetime.datetime.now(datetime.timezone.utc).isoformat(),source_before=identities(),workspace_before=live_identities())
env={**os.environ,'CARGO_BUILD_JOBS':'4','CARGO_TARGET_DIR':cfg['target_dir']}
receipt['env']={k:env[k] for k in ['CARGO_BUILD_JOBS','CARGO_TARGET_DIR']}
with (here/(tag+'.log')).open('xb') as log:
 try:
  p=subprocess.run(command,cwd=cfg['isolate'],env=env,stdout=log,stderr=subprocess.STDOUT)
  code=p.returncode
 except Exception as e:
  log.write(repr(e).encode());code=127
receipt.update(end=datetime.datetime.now(datetime.timezone.utc).isoformat(),exit_code=code,source_after=identities(),workspace_after=live_identities())
with (here/(tag+'.json')).open('x') as f:json.dump(receipt,f,indent=2);f.write('\n')
print(json.dumps(dict(tag=tag,exit_code=code,log=str(here/(tag+'.log')),source_unchanged=receipt['source_before']==receipt['source_after'],workspace_unchanged=receipt['workspace_before']==receipt['workspace_after'])))
sys.exit(code)
