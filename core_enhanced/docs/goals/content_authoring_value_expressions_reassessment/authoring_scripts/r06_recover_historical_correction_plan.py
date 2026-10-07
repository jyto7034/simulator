#!/usr/bin/env python3
"""Recover the historical correction PLAN bytes from their apply-patch record."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from datetime import datetime, timezone


ROOT = Path(__file__).resolve().parents[5]
SESSION = Path(
    "/home/acr0tic/.codex/sessions/2026/09/13/"
    "rollout-2026-09-13T15-28-44-01a09974-34e9-7cd1-ae6e-9274421f9c45.jsonl"
)
SOURCE_LINE = 1075
HISTORICAL_PLAN = ROOT / (
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/corrections/checkpoint_001_f01_f04/PLAN.md"
)
OLD_BASELINE = ROOT / (
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/corrections/checkpoint_001_f01_f04/baseline/pre_fix_001/"
    "MANIFEST.json"
)
OUTPUT = ROOT / (
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/corrections/correction_001_r01_r03/evidence/r03"
)
RECOVERED = OUTPUT / "HISTORICAL_PLAN_PRE_FIX_001.md"
PROVENANCE = OUTPUT / "PROVENANCE_APPLY_PATCH_EVENT.jsonl"
RECORD = OUTPUT / "R03_HISTORICAL_PLAN_RECOVERY.json"
EXPECTED_BYTES = 4460
EXPECTED_SHA256 = "661dd456d923e323cfd5704dd520e322251ba9a603303a4b73c295e6c13412c7"


def digest_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def row(path: Path) -> dict[str, object]:
    return {
        "path": path.relative_to(ROOT).as_posix(),
        "sha256": digest_bytes(path.read_bytes()),
        "bytes": path.stat().st_size,
    }


def main() -> None:
    if OUTPUT.exists():
        raise SystemExit("R03 recovery evidence already exists; refusing to overwrite it")
    raw_line = None
    with SESSION.open("rb") as handle:
        for number, line in enumerate(handle, 1):
            if number == SOURCE_LINE:
                raw_line = line
                break
    if raw_line is None:
        raise SystemExit("provenance JSONL line is missing")
    event = json.loads(raw_line)
    plan_key = str(HISTORICAL_PLAN)
    content = event["payload"]["item"]["changes"][plan_key]["content"]
    recovered = content.encode("utf-8")
    if len(recovered) != EXPECTED_BYTES or digest_bytes(recovered) != EXPECTED_SHA256:
        raise SystemExit("embedded historical PLAN does not match the frozen baseline reference")

    old_baseline = json.loads(OLD_BASELINE.read_text(encoding="utf-8"))
    reference = old_baseline["correction_plan"]
    if reference["bytes"] != EXPECTED_BYTES or reference["sha256"] != EXPECTED_SHA256:
        raise SystemExit("predecessor baseline historical PLAN reference changed")

    OUTPUT.mkdir(parents=True, exist_ok=False)
    RECOVERED.write_bytes(recovered)
    PROVENANCE.write_bytes(raw_line)
    record = {
        "record": "R06_CORRECTION_001_R03_HISTORICAL_PLAN_RECOVERY_001",
        "captured_at": datetime.now(timezone.utc).isoformat(),
        "finding": "R03",
        "status": "historical_bytes_recovered_by_successor",
        "predecessor_baseline_unchanged": True,
        "predecessor_reference": {
            "manifest": OLD_BASELINE.relative_to(ROOT).as_posix(),
            "path": reference["path"],
            "sha256": reference["sha256"],
            "bytes": reference["bytes"],
        },
        "recovered_copy": row(RECOVERED),
        "provenance": {
            "kind": "Codex session apply_patch result containing the exact file content",
            "source_session_path": str(SESSION),
            "source_jsonl_line": SOURCE_LINE,
            "json_pointer": f"/payload/item/changes/{plan_key}/content",
            "raw_event": row(PROVENANCE),
            "raw_event_original_sha256": digest_bytes(raw_line),
            "raw_event_original_bytes": len(raw_line),
        },
        "search_history": {
            "initial_exact_file_search": "no match",
            "repository_tar_search": "no match",
            "git_all_objects_search": "no match among 4460-byte blob candidates",
            "session_record_search": "exact match found",
        },
        "interpretation": (
            "This successor record supplies the missing actual bytes without rewriting the "
            "historical baseline or representing the later live PLAN as the old snapshot."
        ),
    }
    RECORD.write_text(
        json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(json.dumps(record, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
