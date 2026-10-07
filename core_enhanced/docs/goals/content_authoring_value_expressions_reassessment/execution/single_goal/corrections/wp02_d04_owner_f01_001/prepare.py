"""Preserve the exact D04 owner target as this correction's before baseline."""

import datetime
import hashlib
import json
import subprocess
import tarfile
from pathlib import Path


ROOT = Path("/mnt/f/work/simulator")
HERE = Path(__file__).resolve().parent
PREDECESSOR = ROOT / (
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/segments/wp02_d04_owner_relation_001/target/"
    "d04_owner_relation_001/MANIFEST.json"
)
REVIEW = ROOT / (
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/reviews/wp02_d04_owner_relation_001_review_001/REVIEW.md"
)
REQUEST = ROOT / (
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/handoffs/"
    "WP02_D04_OWNER_F01_CORRECTION_REQUEST_20260927.ko.md"
)
BASELINE = HERE / "baseline/entry_001"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def meta(path):
    data = path.read_bytes()
    return {
        "path": str(path.relative_to(ROOT)),
        "bytes": len(data),
        "sha256": sha(data),
    }


assert not BASELINE.exists(), BASELINE
assert sha(PREDECESSOR.read_bytes()) == "c2e703c2a08e1fc38141d3d070e052d8207cd7aaf9790f650c685429728a2a9f"
predecessor = json.loads(PREDECESSOR.read_text())
source_rows = predecessor["source"]["files"]
assert len(source_rows) == 337
for row in source_rows:
    current = meta(ROOT / row["path"])
    assert current == {"path": row["path"], **row["after"]}, row["path"]

rows = [
    (row["path"], row["after"]["bytes"], row["after"]["sha256"])
    for row in source_rows
]
digest = sha(json.dumps(rows, separators=(",", ":")).encode())
assert digest == predecessor["source"]["after_digest"]
assert digest == "eb9bd8fb3927f8a9f3c5b35908012300e4fad9c4e8d7a03a9e612e44b0dc804a"

BASELINE.mkdir(parents=True)
archive_path = BASELINE / "SOURCE_BEFORE.tar.gz"
with tarfile.open(archive_path, "x:gz") as archive:
    for row in source_rows:
        archive.add(ROOT / row["path"], arcname=row["path"], recursive=False)

status = subprocess.check_output(
    ["git", "status", "--short"], cwd=ROOT, text=True
)
(BASELINE / "GIT_STATUS_BEFORE.txt").write_text(status)
manifest = {
    "record": "R06_WP02_D04_OWNER_F01_ENTRY_001",
    "created_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "branch": subprocess.check_output(
        ["git", "branch", "--show-current"], cwd=ROOT, text=True
    ).strip(),
    "head": subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
    ).strip(),
    "predecessor": meta(PREDECESSOR),
    "review": meta(REVIEW),
    "request": meta(REQUEST),
    "source": {
        "file_count": len(source_rows),
        "product_file_count": predecessor["source"]["product_file_count"],
        "digest": digest,
        "product_digest": predecessor["source"]["product_after_digest"],
        "files": [
            {"path": row["path"], **row["after"]} for row in source_rows
        ],
    },
    "archive": meta(archive_path),
    "git_status": meta(BASELINE / "GIT_STATUS_BEFORE.txt"),
}
with (BASELINE / "MANIFEST.json").open("x") as stream:
    json.dump(manifest, stream, indent=2, ensure_ascii=False)
    stream.write("\n")
print(json.dumps({"baseline": meta(BASELINE / "MANIFEST.json"), "source": manifest["source"] | {"files": "omitted"}}, indent=2))
