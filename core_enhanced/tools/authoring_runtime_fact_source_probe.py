#!/usr/bin/env python3
"""Verify current native witnesses against real authored source and optional trace."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--runtime-output", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    repo = args.repository.resolve(strict=True)
    source = args.runtime_output.resolve(strict=True)
    output = args.output_dir.absolute()
    output.mkdir()
    previous = json.loads((source / "RESULT.json").read_text())
    cli = repo / "target/release/author_content"
    assert hashlib.sha256(cli.read_bytes()).hexdigest() == previous["binaries"][str(cli)]
    witnesses = {}

    def visit(value):
        if isinstance(value, dict):
            origin = value.get("source")
            if "hp_before" in value and "hp_after" in value and isinstance(origin, dict) and origin.get("authored"):
                witnesses[origin["authored"]["site"]["action"]["action_id"]] = value
            for child in value.values():
                visit(child)
        elif isinstance(value, list):
            for child in value:
                visit(child)

    for line in (source / "off/receipts.ndjson").read_text().splitlines():
        visit(json.loads(line))
    assert {"periodic_area", "delayed_projectile"}.issubset(witnesses)
    trace = [json.loads(line) for line in (source / "on/terminal_trace.ndjson").read_text().splitlines()]
    terminal = json.loads((source / "off/RESULT.json").read_text())["first_terminal"]["runtime_facts"]
    providers = [row["provider"] for row in terminal["providers"]]
    comparisons = []
    for action, witness in sorted(witnesses.items()):
        origin = witness["source"]
        assert witness["hp_before"] > witness["hp_after"]
        assert origin["actor"] == origin["cast"]["caster_instance_id"]
        assert origin["equipment"] in providers
        assert origin["equipment"]["persistent_copy_uuid"] is not None
        assert origin["action_ordinal"] is not None
        assert "ability_event_seq" not in origin["cast"]
        query = output / (action + ".query.json")
        query.write_text(json.dumps(dict(query="effect", site=origin["authored"]["site"])))
        command = [str(cli), "explain", str(source / "packs"), str(source / "selection.json"), str(query)]
        result = subprocess.run(command, cwd=repo, capture_output=True, timeout=60)
        (output / (action + ".stdout")).write_bytes(result.stdout)
        (output / (action + ".stderr")).write_bytes(result.stderr)
        assert result.returncode == 0, result.stderr
        explanation = json.loads(result.stdout)
        assert explanation["source"] == origin["authored"]
        matched = [row for row in trace if row["source"] and row["source"]["authored"] == origin["authored"]
                   and row["time_ms"] == witness["time_ms"] and row["target_unit"] == witness["target"]
                   and row["change"]["before"] == witness["hp_before"] and row["change"]["after"] == witness["hp_after"]]
        assert matched, action
        comparisons.append(dict(action=action, command=command, exit_code=result.returncode,
                                native_witness=witness, trace_witness=matched[0]))
    result = dict(passed=True, comparisons=comparisons, input_result_sha256=hashlib.sha256((source / "RESULT.json").read_bytes()).hexdigest(),
                  observation_authority="native trace-off facts; trace is only a comparison oracle",
                  files={str(p.relative_to(output)): hashlib.sha256(p.read_bytes()).hexdigest() for p in output.rglob("*") if p.is_file()})
    (output / "RESULT.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(dict(passed=True, source_comparisons=len(comparisons))))


if __name__ == "__main__":
    main()
