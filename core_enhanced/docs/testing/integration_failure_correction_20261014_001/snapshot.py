from pathlib import Path
import sys,hashlib,difflib,json
O=Path(__file__).resolve().parent
R=O.parents[3]
label=sys.argv[1];patch=[];rows=[]
for p in sorted((O/'preimages').rglob('*')):
    if not p.is_file():continue
    rel=p.relative_to(O/'preimages');dest=R/rel
    before=p.read_bytes();after=dest.read_bytes() if dest.exists() else b''
    patch.extend(difflib.unified_diff(before.decode().splitlines(True),after.decode().splitlines(True),fromfile='a/'+str(rel),tofile='b/'+str(rel) if dest.exists() else '/dev/null'))
    rows.append({'path':str(rel),'before':hashlib.sha256(before).hexdigest(),'after':hashlib.sha256(after).hexdigest() if dest.exists() else None})
(O/(label+'.diff')).write_text(''.join(patch))
(O/(label+'.inputs.json')).write_text(json.dumps(rows,indent=2)+'\n')
print('snapshot',label,len(rows))
