"""Preserve a exact current product checkpoint without nesting audit archives."""
import json
import sys
import tarfile
from pathlib import Path
from run import ROOT, HERE, identity, sha

label = sys.argv[1]
assert label.isdigit()
output = HERE / 'evidence' / ('checkpoint_' + label)
output.mkdir()
before = identity()
archive = output / 'PRODUCT.tar.gz'
with tarfile.open(archive, 'w:gz') as stream:
    for row in before['files']:
        path = ROOT / row['path']
        assert sha(path) == row['sha256']
        stream.add(path, arcname=row['path'], recursive=False)
with tarfile.open(archive) as stream:
    assert sorted(stream.getnames()) == sorted(row['path'] for row in before['files'])
    for row in before['files']:
        import hashlib
        assert hashlib.sha256(stream.extractfile(row['path']).read()).hexdigest() == row['sha256']
assert before == identity()
with (output / 'CHECKPOINT.json').open('x') as stream:
    json.dump({'product': before, 'archive_sha256': sha(archive), 'tool_sha256': sha(Path(__file__)), 'scope': 'product paths only; no target or historical evidence embedding'}, stream, indent=2)
print(json.dumps({'checkpoint': str(output), 'product_sha256': before['sha256'], 'files': len(before['files']), 'archive_sha256': sha(archive)}))
