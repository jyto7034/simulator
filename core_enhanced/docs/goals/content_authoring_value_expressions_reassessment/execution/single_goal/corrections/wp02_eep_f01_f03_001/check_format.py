"""Read-only changed-region rustfmt comparison against the actual-byte baseline."""
import difflib
import json
import subprocess
import tarfile
from pathlib import Path

root = Path('/mnt/f/work/simulator')
here = Path(__file__).resolve().parent
rows = []
with tarfile.open(here / 'baseline/entry_001/SOURCE_BEFORE.tar.gz') as archive:
    for relative in ['core_enhanced/src/game/battle/core/sim.rs',
                     'core_enhanced/src/game/battle/core/value_expr_runtime.rs']:
        before = archive.extractfile(relative).read().decode().splitlines(keepends=True)
        current = (root / relative).read_text().splitlines(keepends=True)
        output = subprocess.check_output(['rustfmt', '--edition', '2021', '--config',
                                         'skip_children=true', '--emit', 'stdout', str(root / relative)], text=True)
        output = output.removeprefix(str(root / relative) + ':\n\n')
        formatted = output.splitlines(keepends=True)
        changed = [(c, max(c + 1, d)) for tag, a, b, c, d in difflib.SequenceMatcher(
            a=before, b=current, autojunk=False).get_opcodes() if tag != 'equal']
        debt = [(a, max(a + 1, b)) for tag, a, b, c, d in difflib.SequenceMatcher(
            a=current, b=formatted, autojunk=False).get_opcodes() if tag != 'equal']
        overlap = [span for span in debt if any(span[0] < b and a < span[1] for a, b in changed)]
        rows.append(dict(path=relative, changed=changed, existing_format_debt=debt, overlapping=overlap))
print(json.dumps(rows, indent=2))
assert all(not row['overlapping'] for row in rows), 'changed-region formatting differs'
