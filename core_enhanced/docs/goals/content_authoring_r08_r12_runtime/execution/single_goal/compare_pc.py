"""Compare final PC36 output with the immediate frozen predecessor."""

import collections
import hashlib
import json
import sys
from pathlib import Path


ROOT = Path("/mnt/f/work/simulator")
HERE = Path(__file__).resolve().parent
PRIOR = (
    ROOT / "core_enhanced/docs/goals/content_authoring_fragment_copies/execution/single_goal/evidence/pc_raw_integrated_003.json"
)
CURRENT = (ROOT / sys.argv[1]).resolve()
OUTPUT = (ROOT / sys.argv[2]).resolve()


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


before = json.loads(PRIOR.read_text())
after = json.loads(CURRENT.read_text())
assert before["schema"] == after["schema"] == 1
assert before["seed"] == after["seed"] == 17
assert len(before["rows"]) == len(after["rows"]) == 576
omit = {
    "measurement",
    "setup",
    "native_usage",
    "retained_extent_at_end",
    "last_expr_usage",
    "total_expr_work",
    "error",
}
differences = collections.Counter()
status_differences = 0
error_code_differences = 0
error_detail_differences = 0
for old, new in zip(before["rows"], after["rows"]):
    for key in old.keys() | new.keys():
        if key not in omit and old.get(key) != new.get(key):
            differences[key] += 1
    status_differences += (old["error"] is None) != (new["error"] is None)
    error_code_differences += (old["error"] or "").split("site:")[0] != (
        new["error"] or ""
    ).split("site:")[0]
    error_detail_differences += old["error"] != new["error"]
warmup = [
    row
    for row in after["rows"]
    if row["warmup"] and not row["measurement"]["allocation_observer"]
]
counts = collections.Counter(row["error"] is None for row in warmup)
summary = {
    "predecessor": {
        "path": str(PRIOR.relative_to(ROOT)),
        "sha256": sha(PRIOR),
    },
    "current": {"path": str(CURRENT.relative_to(ROOT)), "sha256": sha(CURRENT)},
    "rows_each": 576,
    "distinct_conditions": len(warmup),
    "successful_conditions": counts[True],
    "typed_refusal_conditions": counts[False],
    "oracle_differences_excluding_measurement_and_cost": dict(differences),
    "completion_status_differences": status_differences,
    "error_code_differences": error_code_differences,
    "error_requested_detail_differences": error_detail_differences,
    "same_gameplay_oracle_and_typed_refusal": not (
        differences or status_differences or error_code_differences
    ),
}
with OUTPUT.open("x") as stream:
    json.dump(summary, stream, indent=2)
    stream.write("\n")
print(json.dumps(summary, indent=2))

assert summary["same_gameplay_oracle_and_typed_refusal"], "PC36 oracle differences require correction"
