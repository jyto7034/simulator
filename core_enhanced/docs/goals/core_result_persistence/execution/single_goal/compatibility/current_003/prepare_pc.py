import sys
from pathlib import Path
sys.path.insert(0, '/mnt/f/work/simulator/core_enhanced/docs/goals/core_result_persistence/tools')
import run
run.HERE = Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_result_persistence/execution/single_goal/compatibility/current_003')
"""Copy the final product source and attach the frozen PC36 test probe."""

import hashlib
import json
import shutil
import sys
import tarfile
import tempfile
from pathlib import Path


ROOT = Path("/mnt/f/work/simulator")
HERE = Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_result_persistence/execution/single_goal/compatibility/current_003')
SUFFIX = sys.argv[1] if len(sys.argv) > 1 else "final"
PRIOR = ROOT / (
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/corrections/wp02_eep_f01_f03_001/evidence/pc_probe.rs"
)
# Copy the complete current baseline membership, including measurement inputs.
import run as runner
FILES = [row['path'] for row in runner.identity()['files']]
ADAPTER = "core_enhanced/src/game/battle/core/value_expr_runtime.rs"
NATIVE = "core_enhanced/src/game/battle/runtime_control.rs"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def identity(root):
    rows = [
        (name, len(data := (root / name).read_bytes()), digest(data))
        for name in FILES
    ]
    return {
        "file_count": len(rows),
        "digest": digest(json.dumps(rows, separators=(",", ":")).encode()),
        "files": rows,
    }


isolate = Path(tempfile.mkdtemp(prefix="r06-completion-pc-", dir="/tmp"))
for name in FILES:
    destination = isolate / name
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(ROOT / name, destination)
unmodified = identity(isolate)

review = (
    "use super::*;\nuse std::cell::Cell;\n"
    "thread_local! { static TRACK: Cell<(bool,usize,usize)> = const { Cell::new((false,0,0)) }; }\n"
    "pub(super) fn requested(n:usize) { let _=TRACK.try_with(|s|{let(a,v,m)=s.get();if a{s.set((a,v+n,m.max(n)))}}); }\n"
)
(isolate / "review_probes.rs").write_text(review)
shutil.copyfile(PRIOR, isolate / "pc_probe.rs")
probe = (isolate / "pc_probe.rs").read_text()
original = "value:TypedInteger::new(10,ValueUnit::Magnitude)"
assert probe.count(original) == 1
(isolate / "pc_probe.rs").write_text(
    probe.replace(original, "value:Some(TypedInteger::new(10,ValueUnit::Magnitude))")
)
# Diagnostic log format v32 adds R09/R12 event variants. Preserve raw hashes
# and compare the unchanged entry bytes using the predecessor v31 envelope.
probe_path = isolate / "pc_probe.rs"
probe = probe_path.read_text()
probe = probe.replace("fn battle(n: usize) -> BattleCore {", r'''fn digest_v31(core: &BattleCore) -> String {
    use sha2::{Digest, Sha256};
    let raw = serde_json::to_vec(&core.event_log).unwrap();
    let suffix = raw.strip_prefix(b"{\"version\":32,").expect("exact v32 envelope");
    let mut hash = Sha256::new();
    hash.update(b"{\"version\":31,"); hash.update(suffix);
    format!("{:x}", hash.finalize())
}
fn battle(n: usize) -> BattleCore {''')
probe = probe.replace("let before=digest(&core);", "let before=digest(&core); let before_v31=digest_v31(&core);")
probe = probe.replace("let after=digest(&core);", "let after=digest(&core); let after_v31=digest_v31(&core);")
probe = probe.replace('"before_log_sha256":before,"after_log_sha256":after}', '"before_log_sha256":before,"after_log_sha256":after,"before_log_v31_sha256":before_v31,"after_log_v31_sha256":after_v31}')
probe_path.write_text(probe)
adapter = (isolate / ADAPTER).read_text().rstrip()
assert adapter.endswith("}")
(isolate / ADAPTER).write_text(
    adapter[:-1]
    + f'\n#[path = "{isolate}/review_probes.rs"] mod review_probes;\n'
    + f'#[path = "{isolate}/pc_probe.rs"] mod load_baseline;\n}}\n'
)
with (isolate / NATIVE).open("a") as stream:
    stream.write(
        "\n#[cfg(test)]\npub(crate) fn baseline_usage() -> Option<(u64, usize, Option<usize>)> {\n"
        " ACTIVE.with(|slot| slot.borrow().as_ref().map(|b| (b.work, b.clone_slots, b.retained_admission)))\n}\n"
    )
instrumented = identity(isolate)
hooks = HERE / f"evidence/pc_isolate_hooks_{SUFFIX}.tar.gz"
with tarfile.open(hooks, "x:gz") as archive:
    for name in [ADAPTER, NATIVE, "review_probes.rs", "pc_probe.rs"]:
        archive.add(isolate / name, arcname=name, recursive=False)
receipt = {
    "isolate": str(isolate),
    "current_source": identity(ROOT),
    "copied_source": unmodified,
    "built_source": instrumented,
    "instrumented_paths": [ADAPTER, NATIVE],
    "supplemental": {
        name: digest((isolate / name).read_bytes())
        for name in ["review_probes.rs", "pc_probe.rs"]
    },
    "hooks_archive": {"bytes": hooks.stat().st_size, "sha256": digest(hooks.read_bytes())},
}
with (HERE / f"evidence/pc_isolate_{SUFFIX}.json").open("x") as stream:
    json.dump(receipt, stream, indent=2)
    stream.write("\n")
print(
    json.dumps(
        {
            "isolate": str(isolate),
            "built_digest": instrumented["digest"],
            "product_matches_current": unmodified == receipt["current_source"],
        }
    )
)
