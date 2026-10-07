#!/usr/bin/env python3
"""Verify fixed A/B combat, terminal source retention, replay and trace isolation."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--discovery", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    repo, fixture, discovery = (p.resolve(strict=True) for p in (args.repository, args.fixture, args.discovery))
    output = args.output_dir.absolute()
    output.mkdir()
    prototype = json.loads((discovery / "terminal_scenario.json").read_text())
    prototype["steps"] = [prototype["steps"][0],
        {"kind": "clock", "clock": {"mode": "elapsed", "delta_ms": 8000, "after_seq": 0}},
        {"kind": "clock", "clock": {"mode": "elapsed", "delta_ms": 12000, "after_seq": 0}},
        {"kind": "command", "command": {"type": "complete_combat_result"}}]
    scenario = output / "scenario.json"
    scenario.write_text(json.dumps(prototype, separators=(",", ":")))
    prior = json.loads((discovery / "RESULT.json").read_text())
    site = prior["first_sourced"]["source"]["authored"]["site"]
    query = output / "effect_query.json"
    query.write_text(json.dumps({"query": "effect", "site": site}, separators=(",", ":")))
    records = []

    def run(name, command):
        started = time.perf_counter()
        result = subprocess.run(command, cwd=repo, capture_output=True, timeout=300)
        stdout, stderr = output / f"{name}.stdout", output / f"{name}.stderr"
        stdout.write_bytes(result.stdout)
        stderr.write_bytes(result.stderr)
        records.append({"name": name, "command": [str(x) for x in command], "exit_code": result.returncode,
                        "elapsed_seconds": time.perf_counter() - started, "stdout": str(stdout), "stderr": str(stderr),
                        "stdout_sha256": hashlib.sha256(result.stdout).hexdigest(), "stderr_sha256": hashlib.sha256(result.stderr).hexdigest()})
        (output / "commands.json").write_text(json.dumps(records, indent=2) + "\n")
        assert result.returncode == 0, (name, result.returncode, result.stderr.decode(errors="replace"))
        print(json.dumps({"name": name, "exit_verified": 0}), flush=True)
        return result.stdout

    summaries = {}
    for generation in ("a", "b"):
        for mode in ("off", "on", "gap"):
            name = f"{generation}_{mode}"
            run(name, [repo / "target/release/examples/combat", fixture / generation, fixture / "selection.json",
                       scenario, output / name, "replay", mode])
            summary = json.loads((output / name / "RESULT.json").read_text())
            assert summary["is_complete"] and summary["accepted_steps"] == 13 and summary["failure"] is None
            assert summary["first_terminal"] == summary["final_observation"]["last_terminal"]
            assert summary["final_observation"]["active"] is None
            terminal = summary["first_terminal"]
            assert terminal["actual_time_ms"] > 8000 and terminal["verdict"] == "victory"
            witness = terminal["mandatory"]["latest_nonzero_sourced_hp"]
            assert witness["source"]["authored"]["site"] == site
            assert witness["change"]["before"] != witness["change"]["after"]
            rows = [json.loads(line) for line in (output / name / "receipts.ndjson").read_text().splitlines()]
            assert len(rows) == 15
            assert rows[-1]["payload"]["request"]["command"]["type"] == "complete_combat_result"
            assert rows[-1]["payload"]["public_result"]["disposition"] == "accepted"
            trace = json.loads((output / name / "terminal_trace.json").read_text())
            assert trace["key"] == terminal["key"]
            if mode == "off":
                assert not trace["enabled"] and trace["rows"] == 0 and trace["gap"] is None
            elif mode == "on":
                assert trace["enabled"] and trace["rows"] > 0 and trace["gap"] is None
                trace_rows = [json.loads(line) for line in (output / name / "terminal_trace.ndjson").read_text().splitlines()]
                assert len(trace_rows) == trace["rows"] == terminal["mandatory"]["applications"]
                assert any(row["source"] and row["source"]["authored"]["site"] == site for row in trace_rows)
            else:
                assert trace["enabled"] and trace["gap"] is not None and trace["rows"] == 0
            summaries[name] = summary
        expected = (output / f"{generation}_off/receipts.ndjson").read_bytes()
        for mode in ("on", "gap"):
            assert (output / f"{generation}_{mode}/receipts.ndjson").read_bytes() == expected
            assert summaries[f"{generation}_{mode}"] == summaries[f"{generation}_off"]
        for mode in ("off", "on"):
            command = [repo / "target/release/author_content", "simulate", fixture / generation,
                       fixture / "selection.json", scenario]
            if mode == "on":
                command += ["--trace-output", output / f"cli_{generation}_trace.ndjson"]
            assert run(f"cli_{generation}_{mode}", command) == expected
        explanation = json.loads(run(f"explain_{generation}", [repo / "target/release/author_content", "explain",
            fixture / generation, fixture / "selection.json", query]))
        assert explanation["source"]["site"] == site
        assert explanation["effect"]["op"]["Damage"]["amount"] == (18 if generation == "a" else 36)
        assert explanation["identity"] == summaries[f"{generation}_off"]["identity"]
    a, b = summaries["a_off"], summaries["b_off"]
    assert a["identity"] != b["identity"] and a["input_sha256"] == b["input_sha256"]
    aw, bw = a["first_sourced"], b["first_sourced"]
    for field in ("ordinal", "time_ms", "source_unit", "target_unit", "source"):
        assert aw[field] == bw[field], field
    assert aw["change"]["before"] == bw["change"]["before"] == 79
    assert aw["change"]["after"] == 61 and bw["change"]["after"] == 43
    result = {"passed": True, "commands": len(records), "api_replays": 6, "cli_simulations": 4, "cli_explanations": 2,
              "accepted_steps_per_run": 13, "receipts_per_run": 15, "a_damage": 18, "b_damage": 36,
              "first_application_time_ms": aw["time_ms"], "terminal_times_ms": [a["first_terminal"]["actual_time_ms"], b["first_terminal"]["actual_time_ms"]],
              "source_site": site, "input_sha256": a["input_sha256"],
              "files": [{"path": str(p.relative_to(output)), "sha256": hashlib.sha256(p.read_bytes()).hexdigest()}
                        for p in sorted(output.rglob("*")) if p.is_file()]}
    (output / "RESULT.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({k: v for k, v in result.items() if k != "files"}), flush=True)


if __name__ == "__main__":
    main()
