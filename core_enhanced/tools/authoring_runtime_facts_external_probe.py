#!/usr/bin/env python3
"""Check R04 mandatory facts via standalone default-release API and thin CLI."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    repo = args.repository.resolve(strict=True)
    source = args.fixture.resolve(strict=True)
    output = args.output_dir.absolute()
    output.mkdir()
    shutil.copytree(source / "packs", output / "packs")
    for name in ("selection.json", "scenario.json", "periodic_area_query.json"):
        shutil.copyfile(source / name, output / name)
    api = repo / "target/release/examples/combat"
    cli = repo / "target/release/author_content"
    commands = []

    def run(name, command):
        started = time.perf_counter()
        result = subprocess.run(list(map(str, command)), cwd=repo, capture_output=True, timeout=300)
        (output / (name + ".stdout")).write_bytes(result.stdout)
        (output / (name + ".stderr")).write_bytes(result.stderr)
        commands.append(dict(name=name, command=list(map(str, command)), exit_code=result.returncode,
                             elapsed_seconds=time.perf_counter() - started))
        (output / "commands.json").write_text(json.dumps(commands, indent=2) + "\n")
        assert result.returncode == 0, (name, result.stderr.decode(errors="replace"))
        print(json.dumps(dict(name=name, exit_verified=0)), flush=True)
        return result.stdout

    root, selection, scenario = output / "packs", output / "selection.json", output / "scenario.json"
    outputs = {}
    summaries = {}
    for name, trace in (("off", "off"), ("on", "on"), ("gap", "gap"), ("repeat", "off")):
        run(name, [api, root, selection, scenario, output / name, "replay", trace])
        report = json.loads((output / name / "RESULT.json").read_text())
        assert report["failure"] is None and report["is_complete"]
        terminal = report["first_terminal"]
        facts = terminal["runtime_facts"]
        summary = facts["summary"]
        assert summary["damage_mutations"] > 0 and summary["damage_hp_lost"] > 0
        assert summary["damage_strikes"] >= summary["damaging_strikes"] > 0
        assert summary["cost_mutations"] > 0 and summary["resource_spent"] > 0
        assert summary["deaths"] > 0
        assert facts["usage"]["accepted_actions"] > 0
        assert facts["usage"]["public_result_reservations"] == 0
        assert facts["usage"]["settled_actions"] > 0
        assert facts["latest_settled_action"]["effects_settled_at_ms"] is not None
        assert terminal == report["final_observation"]["last_terminal"]
        assert report["final_observation"]["active_runtime_facts"] is None
        assert isinstance(facts["actors"], list) and facts["actors"]
        assert isinstance(facts["providers"], list) and facts["providers"]
        copies = [p["provider"]["persistent_copy_uuid"] for p in facts["providers"]]
        assert any(copy is not None for copy in copies)
        outputs[name] = (output / name / "receipts.ndjson").read_bytes()
        summaries[name] = report
    for name in ("on", "gap", "repeat"):
        assert outputs[name] == outputs["off"], name
        assert summaries[name] == summaries["off"], name
    assert json.loads((output / "gap/terminal_trace.json").read_text())["gap"] is not None
    for mode in ("off", "on"):
        command = [cli, "simulate", root, selection, scenario]
        if mode == "on":
            command += ["--trace-output", output / "cli_trace.ndjson"]
        assert run("cli_" + mode, command) == outputs["off"]
    description = json.loads(run("describe", [cli, "describe", root, selection]))
    explanation = json.loads(run("explain", [cli, "explain", root, selection, output / "periodic_area_query.json"]))
    assert description["runtime_observation"] == explanation["runtime_observation"]
    assert description["runtime_observation"]["fact_kinds"] == ["damage", "heal", "cost", "resource_change", "death"]
    assert explanation["source"]["site"]["action"]["action_id"] == "periodic_area"
    result = dict(passed=True, api_runs=4, cli_simulations=2, cli_queries=2,
                  summary=summaries["off"]["first_terminal"]["runtime_facts"]["summary"],
                  source_fixture=str(source), binaries={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in (api, cli)},
                  files={str(p.relative_to(output)): hashlib.sha256(p.read_bytes()).hexdigest()
                         for p in output.rglob("*") if p.is_file()})
    (output / "RESULT.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(dict(passed=True, api_runs=4, cli_simulations=2, cli_queries=2)), flush=True)


if __name__ == "__main__":
    main()
