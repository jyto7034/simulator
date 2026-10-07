"""Sequential product validation with streamed, source-bound receipts."""
import subprocess
import sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
CARGO = "/home/blast/.cargo/bin/cargo"
checks = [
    ("check_final_002", [CARGO, "check", "-p", "game_core", "--all-targets", "--offline", "--locked"]),
    ("authoring_final_002", [CARGO, "test", "-p", "game_core", "--lib", "authoring", "--offline", "--locked"]),
    ("value_expr_final_002", [CARGO, "test", "-p", "game_core", "--lib", "value_expr", "--offline", "--locked"]),
    ("all_targets_final_002", [CARGO, "test", "-p", "game_core", "--all-targets", "--offline", "--locked"]),
    ("clippy_final_002", [CARGO, "clippy", "-p", "game_core", "--all-targets", "--offline", "--locked", "--", "-A", "clippy::never_loop", "-A", "clippy::eq_op"]),
    ("format_final_002", ["/home/blast/.cargo/bin/rustfmt", "--edition", "2021", "--check", "core_enhanced/src/game/battle/core/value_expr_runtime.rs", "core_enhanced/src/game/battle/core/value_expr_runtime/tests/aggregate_replay.rs"]),
]
for name, command in checks:
    subprocess.run([sys.executable, str(HERE / "run.py"), name, "--", *command], check=True)
