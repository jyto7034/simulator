"""Rebuild the exact D04 predecessor source with unchanged R05 tooling."""

import hashlib
import json
import shutil
import tarfile
import tempfile
from pathlib import Path

ROOT = Path('/mnt/f/work/simulator')
HERE = Path(__file__).resolve().parent
BASE = json.loads((HERE / 'baseline/entry_001/MANIFEST.json').read_text())
FILES = [entry['path'] for entry in BASE['source']['files']]
isolate = Path(tempfile.mkdtemp(prefix='wp02-d04-f01-r05-control-', dir='/tmp'))
with tarfile.open(ROOT / BASE['archive']['path']) as archive:
    for name in FILES:
        destination = isolate / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(archive.extractfile(name).read())
tool_path = Path('core_enhanced/tools/runtime_architecture_final_validation')
shutil.copytree(ROOT / tool_path, isolate / tool_path,
                ignore=shutil.ignore_patterns('target', '.git'))
baseline_input = Path('core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json')
(isolate / baseline_input).parent.mkdir(parents=True, exist_ok=True)
shutil.copyfile(ROOT / baseline_input, isolate / baseline_input)
receipt = {
    'isolate': str(isolate),
    'source_manifest': BASE['archive'],
    'source_files': len(FILES),
    'tool_files': [
        {'path': str(path.relative_to(isolate)),
         'bytes': len(data := path.read_bytes()),
         'sha256': hashlib.sha256(data).hexdigest()}
        for path in sorted((isolate / tool_path).rglob('*')) if path.is_file()
    ],
    'input': {'path': str(baseline_input),
              'sha256': hashlib.sha256((isolate / baseline_input).read_bytes()).hexdigest()},
}
with (HERE / 'evidence/r05_control_isolate_001.json').open('x') as stream:
    json.dump(receipt, stream, indent=2)
    stream.write('\n')
print(isolate)
