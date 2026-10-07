#!/usr/bin/env python3
"""Run one WP01 validation command and preserve exact output plus receipt."""

from __future__ import annotations

from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[5]
OUTPUT = ROOT / (
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/segments/wp01_content_input_compile_001/evidence/validation"
)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> int:
    if len(sys.argv) < 4 or sys.argv[2] != "--":
        raise SystemExit("usage: r06_wp01_run_validation.py LABEL -- COMMAND [ARG ...]")
    label = sys.argv[1]
    command = sys.argv[3:]
    OUTPUT.mkdir(parents=True, exist_ok=True)
    log_path = OUTPUT / f"{label}.log"
    receipt_path = OUTPUT / f"{label}.json"
    if log_path.exists() or receipt_path.exists():
        raise SystemExit(f"validation label already frozen: {label}")

    started = datetime.now(timezone.utc)
    completed = subprocess.run(
        command,
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    finished = datetime.now(timezone.utc)
    output = completed.stdout
    log_path.write_bytes(output)
    receipt = {
        "label": label,
        "command": command,
        "cwd": str(ROOT),
        "started_at": started.isoformat(),
        "finished_at": finished.isoformat(),
        "duration_seconds": (finished - started).total_seconds(),
        "exit_code": completed.returncode,
        "output": {
            "path": log_path.relative_to(ROOT).as_posix(),
            "sha256": digest(output),
            "bytes": len(output),
        },
    }
    receipt_path.write_text(
        json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    sys.stdout.buffer.write(output)
    return completed.returncode


if __name__ == "__main__":
    raise SystemExit(main())
