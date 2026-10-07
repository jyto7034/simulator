"""Check rustfmt only where this segment changed the actual-byte baseline."""
import difflib
import json
import subprocess
import tarfile
from pathlib import Path

ROOT = Path('/mnt/f/work/simulator')
HERE = Path(__file__).resolve().parent
FILES = [
    'core_enhanced/src/game/battle/core/mod.rs',
    'core_enhanced/src/game/battle/core/sim.rs',
    'core_enhanced/src/game/battle/core/value_expr_runtime.rs',
    'core_enhanced/src/game/data/mod.rs',
]
rows = []
with tarfile.open(HERE / 'baseline/entry_001/SOURCE_BEFORE.tar.gz') as archive:
    for name in FILES:
        before = archive.extractfile(name).read().decode().splitlines(keepends=True)
        current = (ROOT / name).read_text().splitlines(keepends=True)
        formatted = subprocess.check_output([
            '/home/blast/.cargo/bin/rustfmt', '--edition', '2021',
            '--config', 'skip_children=true', '--emit', 'stdout', str(ROOT / name),
        ], text=True).removeprefix(str(ROOT / name) + ':\n\n').splitlines(keepends=True)
        changed = [(c, max(c + 1, d)) for tag, a, b, c, d in
                   difflib.SequenceMatcher(a=before, b=current, autojunk=False).get_opcodes()
                   if tag != 'equal']
        debt = [(a, max(a + 1, b)) for tag, a, b, c, d in
                difflib.SequenceMatcher(a=current, b=formatted, autojunk=False).get_opcodes()
                if tag != 'equal']
        overlap = [span for span in debt if any(span[0] < b and a < span[1]
                                                for a, b in changed)]
        rows.append({'path': name, 'changed_hunks': len(changed),
                     'preexisting_format_hunks': len(debt), 'overlap': overlap})
print(json.dumps(rows, indent=2))
assert all(not row['overlap'] for row in rows)
