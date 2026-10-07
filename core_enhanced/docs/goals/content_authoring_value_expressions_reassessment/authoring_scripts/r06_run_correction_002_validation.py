#!/usr/bin/env python3
"""Run correction-002 validation and preserve command, output, and source identity."""

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
    "execution/single_goal/corrections/correction_001_r01_r03/evidence/validation"
)
SOURCE_FILES = [
    Path("core_enhanced/src/game/value_expr/compile.rs"),
    Path("core_enhanced/src/game/value_expr/eval.rs"),
    Path("core_enhanced/src/game/value_expr/mod.rs"),
    Path("core_enhanced/src/game/value_expr/tests.rs"),
]


def digest_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def source_rows() -> list[dict[str, object]]:
    result = []
    for relative in SOURCE_FILES:
        data = (ROOT / relative).read_bytes()
        result.append(
            {"path": relative.as_posix(), "sha256": digest_bytes(data), "bytes": len(data)}
        )
    return result


def main() -> int:
    if len(sys.argv) < 4 or sys.argv[2] != "--":
        raise SystemExit("usage: r06_run_correction_002_validation.py LABEL -- COMMAND [ARG ...]")
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
        "source_identity": source_rows(),
        "output": {
            "path": log_path.relative_to(ROOT).as_posix(),
            "sha256": digest_bytes(completed.stdout),
            "bytes": len(completed.stdout),
        },
    }
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    sys.stdout.buffer.write(completed.stdout)
    return completed.returncode


if __name__ == "__main__":
    raise SystemExit(main())
