"""Check rustfmt only where this correction changed preserved source bytes."""

import difflib
import json
import subprocess
import tarfile
from pathlib import Path


ROOT = Path("/mnt/f/work/simulator")
HERE = Path(__file__).resolve().parent
BASE = HERE / "baseline/entry_001/SOURCE_BEFORE.tar.gz"

rows = []
with tarfile.open(BASE) as archive:
    for member in archive.getmembers():
        if not member.isfile() or not member.name.endswith(".rs"):
            continue
        source = archive.extractfile(member)
        assert source is not None
        before_bytes = source.read()
        path = ROOT / member.name
        current_bytes = path.read_bytes()
        if before_bytes == current_bytes:
            continue
        before = before_bytes.decode().splitlines(keepends=True)
        current = current_bytes.decode().splitlines(keepends=True)
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
        formatted = output.removeprefix(str(path) + ":\n\n").splitlines(keepends=True)
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
                a=current, b=formatted, autojunk=False
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
                "path": member.name,
                "changed_hunks": len(changed),
                "preexisting_format_hunks": len(debt),
                "overlap": overlap,
            }
        )

print(json.dumps(rows, indent=2))
assert all(not row["overlap"] for row in rows)
