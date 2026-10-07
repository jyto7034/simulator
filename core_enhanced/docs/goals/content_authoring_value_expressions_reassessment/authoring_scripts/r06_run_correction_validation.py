#!/usr/bin/env python3
"""Run a checkpoint-001 correction validation and preserve output plus receipt."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess
import sys
from datetime import datetime, timezone


ROOT = Path(__file__).resolve().parents[5]
OUTPUT = ROOT / (
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/corrections/checkpoint_001_f01_f04/evidence/validation"
)


def main() -> int:
    if len(sys.argv) < 4 or sys.argv[2] != "--":
        raise SystemExit("usage: r06_run_correction_validation.py LABEL -- COMMAND [ARG ...]")
    label = sys.argv[1]
    command = sys.argv[3:]
    OUTPUT.mkdir(parents=True, exist_ok=True)
    started = datetime.now(timezone.utc)
    completed = subprocess.run(
        command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT
    )
    finished = datetime.now(timezone.utc)
    log_path = OUTPUT / f"{label}.log"
    receipt_path = OUTPUT / f"{label}.json"
    log_path.write_bytes(completed.stdout)
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
            "sha256": hashlib.sha256(completed.stdout).hexdigest(),
            "bytes": len(completed.stdout),
        },
    }
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    sys.stdout.buffer.write(completed.stdout)
    return completed.returncode


if __name__ == "__main__":
    raise SystemExit(main())
