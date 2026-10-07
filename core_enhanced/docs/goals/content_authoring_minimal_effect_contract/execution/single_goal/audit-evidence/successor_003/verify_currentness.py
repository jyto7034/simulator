from pathlib import Path
import json,hashlib,subprocess,os,datetime,sys
base=Path("core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal"); out=base/"audit-evidence/successor_003";target=base/"AUDIT_TARGET_003.json"
x=json.loads(target.read_text()); digest=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
issues=[]; count=0; absent=0
for m in x["manifest"]:
 p=Path(m["path"])
 if m["sha256"] is None:
  absent+=1
  if p.exists():issues.append({"path":str(p),"issue":"expected absent"})
 else:
  count+=1
  if not p.is_file() or digest(p)!=m["sha256"]:issues.append({"path":str(p),"issue":"missing/hash mismatch"})
external=[]
for e in x["external_actual"]:
 d=Path(e["directory"]); expected={f["path"]:f["sha256"] for f in e["files"]}; actual={str(p.relative_to(d)):digest(p) for p in d.rglob("*") if p.is_file()}
 bad=[p for p in expected.keys()|actual.keys() if expected.get(p)!=actual.get(p)]
 external.append({"directory":str(d),"files":len(actual),"mismatches":bad})
 if bad:issues.append({"external":str(d),"mismatches":bad})
for m in x["default_release_binaries"]:
 p=Path(m["path"])
 if not p.is_file() or digest(p)!=m["sha256"]:issues.append({"binary":str(p),"issue":"missing/hash mismatch"})
head=subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip();branch=subprocess.check_output(["git","branch","--show-current"],text=True).strip()
if head!=x["head"] or branch!=x["branch"]:issues.append({"head":head,"branch":branch})
env={k:os.environ.get(k) for k in x["toolchain"]["selected_environment"]}
versions={k:subprocess.check_output([k,"--version"],text=True).strip() for k in ("rustc","cargo")}
if env!=x["toolchain"]["selected_environment"]:issues.append({"environment":env})
if any(versions[k]!=x["toolchain"][k] for k in versions):issues.append({"toolchain":versions})
sha=digest(target)
if sha!="7800068bb87b0e3dd87d9ec99920600867714d35eef9dabb4f26a0bdf137e0e6":issues.append({"target_hash":sha})
r={"reviewer":"/root/r01_correction_fresh_audit","captured_at":datetime.datetime.now(datetime.timezone.utc).isoformat(),"target":str(target),"target_sha256":sha,"head":head,"branch":branch,"manifest_members":len(x["manifest"]),"hashed_present":count,"required_absent":absent,"external":external,"binary_count":len(x["default_release_binaries"]),"toolchain":versions,"environment":env,"issues":issues,"currentness":"current / unchanged" if not issues else "stale"}
(out/(sys.argv[1]+".json")).write_text(json.dumps(r,indent=2)+"\n")
print(json.dumps(r,indent=2))
