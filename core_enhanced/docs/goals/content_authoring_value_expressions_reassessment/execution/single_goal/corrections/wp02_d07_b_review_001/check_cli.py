"""Run real CLI option combinations using the frozen D07-B public fixture."""

import hashlib
import json
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path("/mnt/f/work/simulator")
HERE = Path(__file__).resolve().parent
BASE = HERE.parents[1]
FIXTURE = BASE / "segments/wp02_d07_runtime_amount_001/evidence/cli_fixture"
BINARY = ROOT / ".cache/d07-runtime-amount-product-target/debug/author_content"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    out = HERE / "evidence" / sys.argv[1]
    out.mkdir()
    explanation = BASE / "segments/wp02_d07_static_explain_001/evidence/cli_effect_request.json"
    request = out / "explain_request.json"
    request.write_bytes(explanation.read_bytes())
    prefix = [str(FIXTURE / "packs"), str(FIXTURE / "selection.json")]
    cases = [
        ("plain", ["simulate", *prefix, str(FIXTURE / "scenario.json")]),
        ("trace", ["simulate", *prefix, str(FIXTURE / "scenario.json"),
                   "--trace-output", str(out / "trace.ndjson")]),
        ("amount", ["simulate", *prefix, str(FIXTURE / "scenario.json"), "--amount-observation"]),
        ("amount_trace", ["simulate", *prefix, str(FIXTURE / "scenario.json"),
                          "--amount-observation", "--trace-output", str(out / "amount_trace.ndjson")]),
        ("trace_amount", ["simulate", *prefix, str(FIXTURE / "scenario.json"),
                          "--trace-output", str(out / "trace_amount.ndjson"), "--amount-observation"]),
        ("validate", ["validate", *prefix]),
        ("describe", ["describe", *prefix]),
        ("explain", ["explain", *prefix, str(request)]),
    ]
    rows, outputs = [], {}
    for name, args in cases:
        command = [str(BINARY), *args]
        start = time.monotonic()
        result = subprocess.run(command, cwd=ROOT, capture_output=True, timeout=120)
        (out / (name + ".stdout")).write_bytes(result.stdout)
        (out / (name + ".stderr")).write_bytes(result.stderr)
        row = {"name": name, "command": command, "exit_code": result.returncode,
               "elapsed_seconds": time.monotonic() - start, "stdout_sha256": sha(result.stdout),
               "stderr_sha256": sha(result.stderr)}
        rows.append(row)
        # Persist every completed attempt even when a later assertion fails.
        (out / "executions.json").write_text(json.dumps(rows, indent=2) + "\n")
        print(name, result.returncode, flush=True)
        assert result.returncode == 0, (name, result.stderr.decode())
        outputs[name] = result.stdout
        for line in result.stdout.splitlines():
            json.loads(line)
    assert outputs["plain"] == outputs["trace"]
    assert outputs["amount"] == outputs["amount_trace"] == outputs["trace_amount"]
    assert outputs["amount"] == (FIXTURE / "output_005_trace_off.ndjson").read_bytes()
    documents = {}
    for name in ("plain", "trace", "amount", "amount_trace", "trace_amount"):
        docs = [json.loads(line) for line in outputs[name].splitlines()]
        amounts = [doc for doc in docs if doc.get("kind") in ("amount_preview", "amount_explanation")]
        expected = 0 if name in ("plain", "trace") else 4
        assert len(amounts) == expected
        documents[name] = len(amounts)
    for name in ("trace", "amount_trace", "trace_amount"):
        assert (out / (name + ".ndjson")).stat().st_size > 0
    summary = {"result": "pass", "successful_commands": len(rows),
               "plain_trace_byte_equal": True, "amount_option_orders_and_trace_byte_equal": True,
               "amount_output_matches_predecessor": True, "amount_document_counts": documents,
               "binary_sha256": sha(BINARY.read_bytes()),
               "fixture_selection_sha256": sha((FIXTURE / "selection.json").read_bytes()),
               "fixture_scenario_sha256": sha((FIXTURE / "scenario.json").read_bytes())}
    (out / "SUMMARY.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary), flush=True)


if __name__ == "__main__":
    main()
