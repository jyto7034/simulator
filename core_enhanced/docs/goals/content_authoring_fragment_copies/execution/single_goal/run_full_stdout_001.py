"""Record one R07 validation command against the exact current source membership."""

import hashlib
import json
import os
import re
import selectors
import signal
import subprocess
import sys
import time
import tarfile
from pathlib import Path


ROOT = Path("/mnt/f/work/simulator")
HERE = Path(__file__).resolve().parent
BASE = json.loads((HERE / "baseline/MANIFEST.json").read_text())
def source_names():
    paths = {r["path"] for r in BASE["files"] if (ROOT / r["path"]).is_file()}
    for folder in ("core_enhanced/src", "core_enhanced/tests", "game_resources/data", "core_enhanced/tools/runtime_architecture_final_validation", "core_enhanced/tools/runtime_architecture_preservation"):
        paths.update(str(p.relative_to(ROOT)) for p in (ROOT / folder).rglob("*") if p.is_file() and not any(x in p.parts for x in ("target", "__pycache__")))
    paths.add("core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json")
    return sorted(paths)


def identity(source_root=ROOT):
    rows = []
    for name in source_names():
        if not (source_root / name).is_file():
            rows.append((name, None, None))
            continue
        data = (source_root / name).read_bytes()
        rows.append((name, len(data), hashlib.sha256(data).hexdigest()))
    digest = hashlib.sha256(json.dumps(rows, separators=(",", ":")).encode()).hexdigest()
    return {"file_count": len(rows), "digest": digest, "files": rows}


if __name__ == "__main__":
    args = sys.argv[1:]
    name = args.pop(0)
    cwd = ROOT
    if args[:1] == ["--cwd"]:
        candidate = Path(args[1])
        cwd = candidate if candidate.is_absolute() else ROOT / candidate
        args = args[2:]
    assert args.pop(0) == "--"
    command = args
    evidence = HERE / "evidence"
    evidence.mkdir(exist_ok=True)
    receipt_path = evidence / f"{name}.json"
    log_path = evidence / f"{name}.log"
    interrupted_path = evidence / f"{name}.interrupted.json"
    assert not receipt_path.exists(), name
    assert not log_path.exists(), name
    assert not interrupted_path.exists(), name
    env = os.environ.copy()
    env["PATH"] = "/home/blast/.cargo/bin:" + env["PATH"]
    env.setdefault("CARGO_HOME", "/tmp/wp02-cargo-home")
    env.setdefault(
        "CARGO_TARGET_DIR",
        "/mnt/f/work/simulator/.cache/d07-runtime-amount-product-target",
    )
    before = identity()
    with tarfile.open(evidence / f"{name}.source.tar.gz", "w:gz") as archive:
        for source_name, _, _ in before["files"]:
            archive.add(ROOT / source_name, arcname=source_name, recursive=False)
    start = time.time()
    process = subprocess.Popen(
        command,
        cwd=cwd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        start_new_session=True,
    )
    assert process.stdout is not None

    interrupted_signal = [None]

    def interrupt(signum, _frame):
        interrupted_signal[0] = signum
        raise KeyboardInterrupt

    previous_handlers = {
        signum: signal.signal(signum, interrupt)
        for signum in (signal.SIGINT, signal.SIGTERM)
    }
    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ)
    last_status = time.monotonic()
    try:
        with log_path.open("xb", buffering=0) as stream:
            while True:
                events = selector.select(timeout=5)
                if events:
                    chunk = os.read(process.stdout.fileno(), 64 * 1024)
                    if not chunk:
                        break
                    stream.write(chunk)
                    sys.stdout.buffer.write(chunk)
                    sys.stdout.buffer.flush()
                now = time.monotonic()
                if now - last_status >= 20:
                    print(
                        f"[run.py] {name}: still running after {time.time() - start:.1f}s",
                        flush=True,
                    )
                    last_status = now
        return_code = process.wait()
    except BaseException as error:
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait()
        after = identity()
        elapsed = time.time() - start
        partial = log_path.read_bytes() if log_path.exists() else b""
        status = {
            "command": command,
            "cwd": str(cwd),
            "source_before": before,
            "source_after": after,
            "elapsed_seconds": elapsed,
            "status": "interrupted",
            "signal": interrupted_signal[0],
            "reason": type(error).__name__,
            "child_exit_code": process.returncode,
            "log_sha256": hashlib.sha256(partial).hexdigest(),
        }
        with interrupted_path.open("x") as stream:
            json.dump(status, stream, indent=2)
            stream.write("\n")
        print(json.dumps({"name": name, **status}, indent=2), flush=True)
        raise
    finally:
        selector.close()
        for signum, handler in previous_handlers.items():
            signal.signal(signum, handler)

    elapsed = time.time() - start
    output = log_path.read_bytes()
    after = identity()
    groups = [
        (int(passed), int(failed))
        for passed, failed in re.findall(
            rb"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed;", output
        )
    ]
    receipt = {
        "command": command,
        "cwd": str(cwd),
        "env": {
            "PATH": env["PATH"],
            "RUST_TEST_THREADS": env.get("RUST_TEST_THREADS"),
            "CARGO_HOME": env["CARGO_HOME"],
            "CARGO_TARGET_DIR": env["CARGO_TARGET_DIR"],
        },
        "toolchain": {
            "cargo": subprocess.check_output(
                ["/home/blast/.cargo/bin/cargo", "--version"], text=True
            ).strip(),
            "rustc": subprocess.check_output(
                ["/home/blast/.cargo/bin/rustc", "--version"], text=True
            ).strip(),
        },
        "source_before": before,
        "source_after": after,
        "exit_code": return_code,
        "elapsed_seconds": elapsed,
        "test_groups": groups,
        "test_passed": sum(passed for passed, _ in groups),
        "test_failed": sum(failed for _, failed in groups),
        "log_sha256": hashlib.sha256(output).hexdigest(),
    }
    with receipt_path.open("x") as stream:
        json.dump(receipt, stream, indent=2)
        stream.write("\n")
    print(
        json.dumps(
            {
                "name": name,
                "exit_code": return_code,
                "test_passed": receipt["test_passed"],
                "test_failed": receipt["test_failed"],
                "elapsed_seconds": round(elapsed, 1),
                "source_digest": after["digest"],
                "log_tail": output[-1800:].decode(errors="replace"),
            },
            indent=2,
        )
    )
    sys.exit(return_code)
