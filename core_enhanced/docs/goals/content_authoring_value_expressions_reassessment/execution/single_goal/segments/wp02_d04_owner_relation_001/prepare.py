"""Preserve the exact D04-F01 source before owner-relation implementation edits."""
import hashlib
import json
import subprocess
import tarfile
from pathlib import Path

ROOT = Path('/mnt/f/work/simulator')
HERE = Path(__file__).resolve().parent
PRE = ROOT / ('core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/'
              'execution/single_goal/corrections/wp02_d04_f01_001/'
              'target/d04_f01_correction_001/MANIFEST.json')
prior = json.loads(PRE.read_text())
files = [item['path'] for item in prior['source']['files']]
assert len(files) == 333
assert all(hashlib.sha256((ROOT / item['path']).read_bytes()).hexdigest()
           == item['after']['sha256'] for item in prior['source']['files'])
BASE = HERE / 'baseline/entry_001'
BASE.mkdir(parents=True, exist_ok=False)
status = subprocess.check_output(['git', 'status', '--short', '--branch',
                                  '--untracked-files=all'], cwd=ROOT)
(BASE / 'GIT_STATUS_BEFORE.txt').write_bytes(status)
archive = BASE / 'SOURCE_BEFORE.tar.gz'
with tarfile.open(archive, 'w:gz') as output:
    for name in files:
        output.add(ROOT / name, arcname=name, recursive=False)


def meta(path):
    data = path.read_bytes()
    return {'path': str(path.relative_to(ROOT)), 'bytes': len(data),
            'sha256': hashlib.sha256(data).hexdigest()}


rows = [(name, len(data := (ROOT / name).read_bytes()), hashlib.sha256(data).hexdigest())
        for name in files]
digest = hashlib.sha256(json.dumps(rows, separators=(',', ':')).encode()).hexdigest()
assert digest == prior['source']['after_digest']
manifest = {'record': 'R06_WP02_D04_OWNER_RELATION_ENTRY_001',
            'branch': subprocess.check_output(['git', 'branch', '--show-current'],
                                              cwd=ROOT, text=True).strip(),
            'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'],
                                            cwd=ROOT, text=True).strip(),
            'predecessor': meta(PRE),
            'source': {'file_count': len(files), 'digest': digest,
                       'files': [dict(path=name, bytes=size, sha256=sha)
                                 for name, size, sha in rows]},
            'archive': meta(archive), 'status': meta(BASE / 'GIT_STATUS_BEFORE.txt')}
with (BASE / 'MANIFEST.json').open('x') as stream:
    json.dump(manifest, stream, indent=2)
    stream.write('\n')
print(json.dumps({'digest': digest, 'archive': manifest['archive'],
                  'predecessor': manifest['predecessor']}, indent=2))
