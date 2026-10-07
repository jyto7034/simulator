import pathlib,json,hashlib,subprocess,sys,datetime
R=pathlib.Path('/mnt/f/work/simulator'); D=R/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal'; T=D/'AUDIT_TARGET_006.json'; expected='a1f87aded22960fce13999d609c3d00f10ab4aa0c86423e63108313bc15b03af'
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
x=json.loads(T.read_text()); errors=[]; n=0; absent=0
def check(v,base=R):
 global n,absent
 p=base/v['path']; n+=1
 if v['sha256'] is None:
  absent+=1
  if p.exists() or p.is_symlink(): errors.append([str(p),'required absence'])
 elif not p.is_file():errors.append([str(p),'missing'])
 elif p.stat().st_size!=v['bytes'] or digest(p)!=v['sha256']:errors.append([str(p),'content mismatch'])
for v in x['manifest']:check(v)
external_count=0
for inv in x['external_actual']:
 base=R/inv['directory']; actual={str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}; wanted={v['path'] for v in inv['files']}
 if actual!=wanted:errors.append([str(base),'inventory difference',sorted(actual-wanted),sorted(wanted-actual)])
 for v in inv['files']:check(v,base);external_count+=1
for v in x['default_release_binaries']:check(v)
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip();branch=subprocess.check_output(['git','branch','--show-current'],cwd=R,text=True).strip()
tc={a:subprocess.check_output([a,'--version'],cwd=R,text=True).strip() for a in ['rustc','cargo']}
if digest(T)!=expected:errors.append(['self hash',digest(T)])
if head!=x['head'] or branch!=x['branch'] or tc!=x['toolchain']:errors.append(['environment',head,branch,tc])
out={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'target':str(T.relative_to(R)),'target_sha256':digest(T),'manifest_members':len(x['manifest']),'required_absences':absent,'exact_external_inventories':len(x['external_actual']),'external_files':external_count,'default_release_binaries':len(x['default_release_binaries']),'total_checks_including_overlap':n,'head':head,'branch':branch,'toolchain':tc,'errors':errors,'result':'current / unchanged' if not errors else 'stale'}
p=D/'audit-evidence/r04_correction_006'/sys.argv[1];p.write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
