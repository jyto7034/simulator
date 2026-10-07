#!/usr/bin/env python3
"""Compare the default-release authoring CLI with a standalone public API consumer."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    repo = args.repository.resolve(strict=True)
    output = args.output_dir.absolute()
    output.mkdir()
    records = []

    def run(name, command, expected=0, sink=None):
        started = time.perf_counter()
        if sink is None:
            result = subprocess.run(command, cwd=repo, capture_output=True, timeout=300)
            stdout = result.stdout
        else:
            with sink.open("wb") as stream:
                result = subprocess.run(command, cwd=repo, stdout=stream, stderr=subprocess.PIPE, timeout=300)
            stdout = b""
        out = output / f"{name}.stdout"
        err = output / f"{name}.stderr"
        out.write_bytes(stdout)
        err.write_bytes(result.stderr)
        records.append({"name": name, "command": [str(x) for x in command], "expected_exit": expected,
                        "exit_code": result.returncode, "elapsed_seconds": time.perf_counter() - started,
                        "stdout": str(out), "stderr": str(err), "stdout_sink": str(sink) if sink else None,
                        "stdout_sha256": hashlib.sha256(stdout).hexdigest(),
                        "stderr_sha256": hashlib.sha256(result.stderr).hexdigest()})
        (output / "commands.json").write_text(json.dumps(records, indent=2) + "\n")
        assert result.returncode == expected, (name, result.returncode, result.stderr.decode(errors="replace"))
        print(json.dumps({"name": name, "exit_verified": result.returncode}), flush=True)
        return stdout, result.stderr

    packs = output / "packs"
    packs.mkdir()
    run("package", [repo / "target/release/package_official_content", repo / "game_resources", packs / "official"])
    api = output / "api"
    manifest = repo / "core_enhanced/tools/authoring_trial_api_probe/Cargo.toml"
    run("public_api", ["cargo", "run", "--release", "--offline", "--locked", "--manifest-path", manifest,
                       "--target-dir", repo / "target", "--", packs, "official", api])
    binary = repo / "target/release/author_content"

    def command(operation, request=None, extra=()):
        result = [binary, operation, packs, api / "selection.json"]
        if request is not None:
            result.append(api / request)
        return result + list(extra)

    for operation in ("validate", "describe", "explain"):
        stdout, stderr = run(operation, command(operation, "explain.json" if operation == "explain" else None))
        assert not stderr
        expected = "explain_result.json" if operation == "explain" else f"{operation}.json"
        assert json.loads(stdout) == json.loads((api / expected).read_bytes()), operation
    expected_receipts = (api / "receipts.ndjson").read_bytes()
    for name in ("simulate", "repeat"):
        stdout, stderr = run(name, command("simulate", "scenario.json"))
        assert stdout == expected_receipts and not stderr, name
    trace = output / "trace.ndjson"
    stdout, stderr = run("trace", command("simulate", "scenario.json", ["--trace-output", trace]))
    assert stdout == expected_receipts and not stderr
    trace_bytes = trace.read_bytes()
    windows = [json.loads(line) for line in trace_bytes.splitlines()]
    assert windows[0] == {"kind": "trace_windows", "schema_version": 1, "retention": "active_and_last_terminal"}
    headers = [row for row in windows[1:] if row.get("kind") == "trace_window"]
    assert headers and all(row["enabled"] for row in headers)
    stdout, stderr = run("trace_collision", command("simulate", "scenario.json", ["--trace-output", trace]), expected=3)
    assert stdout == expected_receipts and b"transport output failed" in stderr
    assert trace.read_bytes() == trace_bytes
    for name, request, prefix, diagnostic in [
        ("partial_failure", "partial.json", "partial_receipts.ndjson", "partial_failure.json"),
        ("preparation_failure", "unprepared.json", None, "preparation_failure.json"),
    ]:
        stdout, stderr = run(name, command("simulate", request), expected=1)
        assert stdout == ((api / prefix).read_bytes() if prefix else b"")
        assert json.loads(stderr) == json.loads((api / diagnostic).read_bytes())
    malformed = output / "malformed.json"
    malformed.write_text('{"packs":')
    oversized = output / "oversized.json"
    oversized.write_bytes(b" " * (64 * 1024 + 1))
    for name, selection in (("malformed_selection", malformed), ("oversized_selection", oversized)):
        stdout, stderr = run(name, [binary, "validate", packs, selection], expected=2)
        assert not stdout and json.loads(stderr)["diagnostic"]["code"] == "authoring_cli_input"
    stdout, stderr = run("invalid_args", [binary, "simulate"], expected=2)
    assert not stdout and json.loads(stderr)["diagnostic"]["code"] == "authoring_cli_input"
    full = Path("/dev/full")
    if full.exists():
        _, stderr = run("failed_stdout", command("simulate", "partial.json"), expected=3, sink=full)
        lines = stderr.splitlines()
        assert json.loads(lines[0]) == json.loads((api / "partial_failure.json").read_bytes())
        assert lines[-1] == b"author_content: transport output failed"
    else:
        raise AssertionError("required /dev/full transport oracle is unavailable")
    # Each separate crate must fail for its intended Rust boundary, not a missing
    # dependency/fixture/import. The generated inputs are included in the receipt.
    negative_cases = {
        "private_state": ("E0616", "state", "fn check(s: &TrialSession) { let _ = &s.state; }"),
        "clone": ("E0277", "Clone", "fn require<T: Clone>() {} fn check() { require::<TrialSession>(); }"),
        "serialize": ("E0277", "Serialize", "fn check(s: &TrialSession) { let _ = serde_json::to_string(s); }"),
        "deserialize": ("E0277", "Deserialize", 'fn check() { let _ = serde_json::from_str::<TrialSession>("{}"); }'),
        "deref": ("E0308", "CoreApplication", "fn check(s: &TrialSession) { let _: &game_core::application::CoreApplication = s; }"),
        "borrow_mutation": ("E0502", "borrow", "fn check(s: &mut TrialSession, c: &game_core::authoring::CancellationToken) { let r = s.ready_receipt(); let _ = s.advance(c); println!(\"{}\", r.document()); }"),
    }
    for name, (code, diagnostic, source) in negative_cases.items():
        project = output / f"privacy_{name}"
        (project / "src").mkdir(parents=True)
        (project / "src/main.rs").write_text("use game_core::authoring::TrialSession;\n" + source + "\nfn main() {}\n")
        configuration = manifest.read_text().replace('path = "../.."', 'path = ' + json.dumps(str(repo / "core_enhanced")))
        (project / "Cargo.toml").write_text(configuration)
        (project / "Cargo.lock").write_bytes(manifest.with_name("Cargo.lock").read_bytes())
        _, stderr = run(f"privacy_{name}", ["cargo", "check", "--release", "--offline", "--locked", "--manifest-path",
                       project / "Cargo.toml", "--target-dir", repo / "target"], expected=101)
        assert f"error[{code}]".encode() in stderr and diagnostic.encode() in stderr, (name, stderr)
        assert b"could not resolve" not in stderr and b"no matching package" not in stderr
    receipt = {"passed": True, "commands": len(records), "default_release_public_consumers": 1,
               "cli_cases": 13, "negative_compile_consumers": len(negative_cases),
               "receipt_documents": len(expected_receipts.splitlines()),
               "trace_window_headers": len(headers), "trace_sha256": hashlib.sha256(trace_bytes).hexdigest(),
               "scope": "external official pack; finite preparation and replay; final trace windows, not full history",
               "files": [{"path": str(path.relative_to(output)), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
                         for path in sorted(output.rglob("*")) if path.is_file()]}
    (output / "RESULT.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({key: value for key, value in receipt.items() if key != "files"}), flush=True)


if __name__ == "__main__":
    main()
