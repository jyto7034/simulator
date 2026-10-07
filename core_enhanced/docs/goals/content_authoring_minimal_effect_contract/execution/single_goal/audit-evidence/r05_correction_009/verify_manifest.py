import json,hashlib,sys,datetime,concurrent.futures,os
from pathlib import Path
T=Path("core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/AUDIT_TARGET_009.json")
D=Path(__file__).parent
def sha(p):
 h=hashlib.sha256()
 with open(p,"rb") as f:
  for b in iter(lambda:f.read(1024*1024),b""): h.update(b)
 return h.hexdigest()
def check(r):
 p=Path(r["path"]); exists=os.path.lexists(p)
 if r["sha256"] is None: return {"path":str(p),"ok":not exists,"required_absence":True}
 if not exists:return {"path":str(p),"ok":False,"error":"missing"}
 s=p.stat().st_size; h=sha(p)
 return {"path":str(p),"ok":s==r["bytes"] and h==r["sha256"],"bytes":s,"sha256":h}
t=json.loads(T.read_text());m=t["manifest"]
with concurrent.futures.ThreadPoolExecutor(max_workers=8) as ex: rows=list(ex.map(check,m))
canonical=hashlib.sha256(json.dumps(m,ensure_ascii=False,sort_keys=True,separators=(",",":")).encode()).hexdigest()
o={"stage":sys.argv[1],"captured_at":datetime.datetime.now(datetime.timezone.utc).isoformat(),"target_path":str(T),"target_sha256":sha(T),"target_bytes":T.stat().st_size,"target_self_match":sha(T)=="ef6a0a2c66603bc2876c88b43106a4c1006ca639f1c07ea582b7eee7e2a17e45","manifest_count":len(m),"manifest_total_bytes":sum(r["bytes"] for r in m),"canonical_manifest_sha256":canonical,"canonical_match":canonical==t["canonical_manifest_sha256"],"unique_paths":len({r["path"] for r in m})==len(m),"required_absences":sum(r["sha256"] is None for r in m),"failures":[r for r in rows if not r["ok"]],"rows":rows}
(D/(sys.argv[1]+"_MANIFEST.json")).write_text(json.dumps(o,ensure_ascii=False,indent=2)+"\n")
print(json.dumps({k:v for k,v in o.items() if k!="rows"}))
