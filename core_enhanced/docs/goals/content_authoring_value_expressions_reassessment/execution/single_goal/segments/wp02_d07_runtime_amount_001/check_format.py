"""Check rustfmt for every D07-B changed region and the complete added module."""

import difflib
import json
import subprocess
import tarfile
from pathlib import Path


ROOT = Path("/mnt/f/work/simulator")
HERE = Path(__file__).resolve().parent
BASE = HERE / "baseline/entry_001/SOURCE_BEFORE.tar.gz"
ADDED = {
    "core_enhanced/src/authoring/amount_query.rs",
    "core_enhanced/src/game/battle/core/amount_observation.rs",
}


def formatted(path):
    output = subprocess.check_output(
        [
            "/home/blast/.cargo/bin/rustfmt",
            "--edition",
            "2021",
            "--config",
            "skip_children=true",
            "--emit",
            "stdout",
            str(path),
        ],
        text=True,
    )
    return output.removeprefix(str(path) + ":\n\n").splitlines(keepends=True)


rows = []
with tarfile.open(BASE) as archive:
    before_by_name = {}
    for member in archive.getmembers():
        if member.isfile() and member.name.endswith(".rs"):
            source = archive.extractfile(member)
            assert source is not None
            before_by_name[member.name] = source.read()

for name, before_bytes in before_by_name.items():
    path = ROOT / name
    current_bytes = path.read_bytes()
    if before_bytes == current_bytes:
        continue
    before = before_bytes.decode().splitlines(keepends=True)
    current = current_bytes.decode().splitlines(keepends=True)
    changed = [
        (start, max(start + 1, end))
        for tag, _a, _b, start, end in difflib.SequenceMatcher(
            a=before, b=current, autojunk=False
        ).get_opcodes()
        if tag != "equal"
    ]
    debt = [
        (start, max(start + 1, end))
        for tag, start, end, _a, _b in difflib.SequenceMatcher(
            a=current, b=formatted(path), autojunk=False
        ).get_opcodes()
        if tag != "equal"
    ]
    overlap = [
        span
        for span in debt
        if any(
            span[0] < changed_end and changed_start < span[1]
            for changed_start, changed_end in changed
        )
    ]
    rows.append(
        {
            "path": name,
            "changed_hunks": len(changed),
            "preexisting_format_hunks": len(debt),
            "overlap": overlap,
        }
    )

for name in ADDED:
    path = ROOT / name
    current = path.read_text().splitlines(keepends=True)
    debt = current != formatted(path)
    rows.append(
        {
            "path": name,
            "changed_hunks": 1,
            "preexisting_format_hunks": int(debt),
            "overlap": [[0, len(current)]] if debt else [],
        }
    )

print(json.dumps(rows, indent=2))
assert all(not row["overlap"] for row in rows)
