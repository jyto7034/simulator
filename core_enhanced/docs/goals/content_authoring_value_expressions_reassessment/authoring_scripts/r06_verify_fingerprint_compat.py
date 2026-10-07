#!/usr/bin/env python3
"""Compare corrected streaming fingerprints with frozen pre-fix source bytes."""

from __future__ import annotations

import hashlib
from pathlib import Path
import subprocess
import tarfile
import tempfile


ROOT = Path(__file__).resolve().parents[5]
CORRECTION = ROOT / (
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/corrections/checkpoint_001_f01_f04"
)
ARCHIVE = CORRECTION / "baseline/pre_fix_001/SOURCE_BEFORE_FIX_001.tar.gz"
PROBE = CORRECTION / "probes/fingerprint_probe.rs"
def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def run(command: list[str], cwd: Path) -> bytes:
    completed = subprocess.run(
        command,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if completed.returncode != 0:
        raise RuntimeError(
            f"command failed ({completed.returncode}): {' '.join(command)}\n"
            + completed.stdout.decode(errors="replace")
        )
    return completed.stdout


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="r06-fingerprint-compat-") as raw_temp:
        temporary = Path(raw_temp)
        frozen = temporary / "frozen"
        old_target = temporary / "target"
        frozen.mkdir()
        with tarfile.open(ARCHIVE, "r:gz") as archive:
            archive.extractall(frozen, filter="data")
        workspace = frozen / "Cargo.toml"
        workspace.write_text(
            "\n".join(
                line
                for line in workspace.read_text(encoding="utf-8").splitlines()
                if '"game_server"' not in line
            )
            + "\n",
            encoding="utf-8",
        )

        run(
            [
                "cargo",
                "build",
                "-p",
                "game_core",
                "--lib",
                "--offline",
                "--target-dir",
                str(old_target),
            ],
            frozen,
        )
        run(
            ["cargo", "build", "-p", "game_core", "--lib", "--offline", "--locked"],
            ROOT,
        )

        old_binary = temporary / "fingerprint_pre_fix"
        corrected_binary = temporary / "fingerprint_corrected"
        run(
            [
                "rustc",
                "--edition=2021",
                str(PROBE),
                "--extern",
                f"game_core={old_target / 'debug/libgame_core.rlib'}",
                "-L",
                f"dependency={old_target / 'debug/deps'}",
                "-o",
                str(old_binary),
            ],
            ROOT,
        )
        run(
            [
                "rustc",
                "--edition=2021",
                str(PROBE),
                "--extern",
                f"game_core={ROOT / 'target/debug/libgame_core.rlib'}",
                "-L",
                f"dependency={ROOT / 'target/debug/deps'}",
                "-o",
                str(corrected_binary),
            ],
            ROOT,
        )
        before = run([str(old_binary)], ROOT)
        after = run([str(corrected_binary)], ROOT)
        if before != after:
            raise SystemExit(
                "fingerprint mismatch\n"
                f"pre-fix:\n{before.decode(errors='replace')}"
                f"corrected:\n{after.decode(errors='replace')}"
            )

    print(f"baseline_archive_sha256={digest(ARCHIVE)}")
    print(f"probe_sha256={digest(PROBE)}")
    print(before.decode(), end="")
    print("fingerprint_compatibility=PASS")


if __name__ == "__main__":
    main()
