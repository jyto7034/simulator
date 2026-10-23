from pathlib import Path
import shutil,sys,hashlib,json
O=Path(__file__).resolve().parent
R=O.parents[3]
D=Path(sys.argv[1]).resolve()
if D.exists(): raise SystemExit("choose a fresh destination")
manifest=json.loads((O/'INPUT_VERIFICATION.json').read_text())
for item in manifest['resources']:
    p=R/'game_resources'/item['path']
    assert hashlib.sha256(p.read_bytes()).hexdigest()==item['sha256'],p
shutil.copytree(O/'probe/core_enhanced',D/'core_enhanced')
shutil.copytree(R/'game_resources',D/'game_resources')
p=D/'core_enhanced/Cargo.toml'
p.write_text(p.read_text().replace('/mnt/f/work/simulator/core_enhanced',str(R/'core_enhanced')))
print(D/'core_enhanced/Cargo.toml')
