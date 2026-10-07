#!/usr/bin/env python3
"""Bounded R02 probe of the real default-release packager/application binaries."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import time


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree_receipt(root):
    return {str(path.relative_to(root)): digest(path)
            for path in sorted(root.rglob("*")) if path.is_file()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    repo = args.repository.resolve(strict=True)
    output = args.output_dir.absolute()
    # An explicit absent fixture directory only. No cleanup or overwrite option.
    output.mkdir()
    source = output / "source"
    shutil.copytree(repo / "game_resources/data", source / "data")
    live = output / "live_data"
    live.mkdir()
    custom = live / "b"
    custom.mkdir()
    policy = (source / "data/run/policy.ron").read_text()
    assert policy.count("starter_enkephalin: 500") == 1
    changed_policy = policy.replace("starter_enkephalin: 500", "starter_enkephalin: 733")
    (custom / "policy.ron").write_text(changed_policy)
    manifest = '''(
    pack_id: "b", pack_version: "1", namespace: Global,
    contract: (pack_format: 1, catalog_schema: 3, authoring: 1, execution: 2, fingerprint: 1),
    required_capabilities: [], dependencies: ["official"],
    files: [(kind: RunPolicy, path: "policy.ron")],
    operations: [(target: (kind: RunPolicy, id: "run_policy"), operation: Replace, after: ["official"])],
)'''
    (custom / "pack.ron").write_text(manifest)
    (custom / "user-notes.bin").write_bytes(b"isolated user-owned sentinel\x00\xff")
    source_before = tree_receipt(source)
    custom_before = tree_receipt(custom)
    packager = repo / "target/release/package_official_content"
    consumer = repo / "target/release/consume_content_pack"
    binary_before = {str(p): digest(p) for p in [packager, consumer]}
    commands = []

    def run(name, executable, arguments, success=True, diagnostic=None):
        command = [str(executable), *map(str, arguments)]
        started = time.perf_counter()
        result = subprocess.run(command, cwd=repo, capture_output=True, text=True, timeout=90)
        stdout = output / f"{name}.stdout"
        stderr = output / f"{name}.stderr"
        stdout.write_text(result.stdout)
        stderr.write_text(result.stderr)
        record = {"name": name, "command": command, "exit_code": result.returncode,
                  "elapsed_seconds": time.perf_counter() - started,
                  "stdout": str(stdout), "stdout_sha256": digest(stdout),
                  "stderr": str(stderr), "stderr_sha256": digest(stderr)}
        commands.append(record)
        (output / "commands.json").write_text(json.dumps(commands, indent=2) + "\n")
        assert (result.returncode == 0) == success, (name, result.returncode, result.stderr)
        if diagnostic:
            assert diagnostic in result.stderr, (name, result.stderr)
        return json.loads(result.stdout) if success else None

    packed = run("package_a", packager, [source, live / "official"])
    assert len(packed["source_receipt"]["files"]) == 26
    assert len(packed["pack_receipt"]["files"]) == 26
    assert tree_receipt(source) == source_before
    assert tree_receipt(custom) == custom_before
    for relative, sha256 in source_before.items():
        if relative != "data/bundle.ron":
            assert digest(live / "official" / relative) == sha256, relative
    official_before = tree_receipt(live / "official")
    a1 = run("consume_a1", consumer, [live, "official"])
    b = run("consume_b", consumer, [live, "official", "b"])
    a2 = run("consume_a2", consumer, [live, "official"])
    assert [r["run_snapshot"]["resources"]["enkephalin"] for r in [a1, b, a2]] == [500, 733, 500]
    assert a1 == a2, "unchanged A must reproduce full public snapshot and receipt"
    assert a1["content_identity"] != b["content_identity"]
    assert a1["content_identity"] == packed["content_identity"]
    assert a1["run_seed"] == b["run_seed"]
    assert a1["briefing_scripts"] and b["briefing_scripts"]
    assert b["run_snapshot"]["game_state"] == "viewing_map"
    assert any(r["path"] == "b/policy.ron" for r in b["source_receipt"]["files"])
    run("existing_output", packager, [source, live / "official"], False, "exclusively create")
    assert tree_receipt(live / "official") == official_before

    # Negative edits concern only this newly created disposable B fixture.
    (custom / "policy.ron").unlink()
    run("missing_b_catalog", consumer, [live, "official", "b"], False, "b/policy.ron")
    (custom / "policy.ron").write_text("(unsupported: true)")
    run("invalid_b_catalog", consumer, [live, "official", "b"], False, "b/policy.ron")
    (custom / "policy.ron").write_text(changed_policy)
    (custom / "pack.ron").write_text(manifest.replace("pack_format: 1", "pack_format: 0"))
    run("old_b_contract", consumer, [live, "official", "b"], False, "b/pack.ron")
    (custom / "pack.ron").write_text(manifest.replace("required_capabilities: []", 'required_capabilities: ["unsupported_execution"]'))
    run("unsupported_b_capability", consumer, [live, "official", "b"], False, "b/pack.ron")
    (custom / "pack.ron").write_text(manifest)
    run("missing_source", packager, [output / "absent", live / "never-created"], False)
    assert not (live / "never-created").exists()
    assert tree_receipt(source) == source_before
    assert tree_receipt(custom) == custom_before
    assert tree_receipt(live / "official") == official_before
    assert {str(p): digest(p) for p in [packager, consumer]} == binary_before
    receipt = {"passed": True, "commands": commands, "binary_sha256": binary_before,
               "source_before_after": source_before, "custom_before_after": custom_before,
               "official_before_after": official_before,
               "actual_starting_currency": [500, 733, 500],
               "a_identity": a1["content_identity"], "b_identity": b["content_identity"],
               "scope": "same default-release binary, actual briefing/starter/resource assignment; no combat claim"}
    target = output / "receipt.json"
    target.write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({"passed": True, "commands": len(commands), "receipt": str(target),
                      "actual_starting_currency": receipt["actual_starting_currency"]}))


if __name__ == "__main__":
    main()
