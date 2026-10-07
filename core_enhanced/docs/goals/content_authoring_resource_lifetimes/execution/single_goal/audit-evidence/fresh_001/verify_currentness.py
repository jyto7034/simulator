import concurrent.futures, datetime, hashlib, json, pathlib, subprocess, sys

ROOT = pathlib.Path('/mnt/f/work/simulator')
BASE = ROOT / 'core_enhanced/docs/goals/content_authoring_resource_lifetimes/execution/single_goal'
OUT = BASE / 'audit-evidence/fresh_001'
TARGET = BASE / 'AUDIT_TARGET_002.json'
EXPECTED = 'c6f995769f7a1293f1787f856541462fa7826370f74c7ff87f4d8a7b93e8f778'

def digest(p):
    h=hashlib.sha256()
    with p.open('rb') as f:
        for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
    return h.hexdigest()

def check(x):
    p=ROOT/x['path']
    if x.get('sha256') is None:
        return {'path':x['path'],'expected_absence':True,'ok':not p.exists()}
    try:
        n=p.stat().st_size; h=digest(p)
        return {'path':x['path'],'bytes':n,'sha256':h,'ok':n==x['bytes'] and h==x['sha256']}
    except OSError as e:return {'path':x['path'],'ok':False,'error':str(e)}

def main():
    phase=sys.argv[1]
    target=json.loads(TARGET.read_text())
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool: rows=list(pool.map(check,target['manifest']))
    canonical=hashlib.sha256(json.dumps(target['manifest'],sort_keys=True,separators=(',',':')).encode()).hexdigest()
    # Preserve the exact original serialization convention separately; content membership is independently checked above.
    r={'phase':phase,'time_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'target':str(TARGET.relative_to(ROOT)),
       'target_sha256':digest(TARGET),'expected_target_sha256':EXPECTED,'target_ok':digest(TARGET)==EXPECTED,
       'manifest_count':len(rows),'manifest_bytes':sum(x.get('bytes',0) for x in rows),'duplicate_paths':len(rows)-len({x['path'] for x in rows}),
       'declared_canonical_sha256':target['canonical_manifest_sha256'],'sorted_compact_json_manifest_sha256':canonical,
       'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
       'failures':[x for x in rows if not x['ok']],'members':rows}
    r['pass']=r['target_ok'] and not r['failures'] and r['duplicate_paths']==0 and r['head']==target['head'] and canonical==target['canonical_manifest_sha256']
    (OUT/(phase.upper()+'_CURRENTNESS.json')).write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({k:v for k,v in r.items() if k!='members'},ensure_ascii=False,indent=2))

if __name__=='__main__':main()
