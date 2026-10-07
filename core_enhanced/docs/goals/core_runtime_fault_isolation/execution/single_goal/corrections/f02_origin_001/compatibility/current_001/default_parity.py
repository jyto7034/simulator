"""Compare every operational numeric default with the archived pre-S1 source."""
import ast
import hashlib
import json
import re
import tarfile
import sys
from pathlib import Path
sys.path.insert(0, '/mnt/f/work/simulator/core_enhanced/docs/goals/core_runtime_safety_observability/tools')
from initialize import ROOT, HERE

archive_path = HERE / "execution/s1/baseline/SOURCE_BEFORE.tar.gz"
manifest = json.loads((HERE / "execution/s1/baseline/SOURCE_BEFORE.json").read_text())
baseline = {row["path"]: row for row in manifest["files"]}
sources = {}


def source(archive, suffix):
    name = "core_enhanced/src/" + suffix
    raw = archive.extractfile(name).read()
    digest = hashlib.sha256(raw).hexdigest()
    assert digest == baseline[name]["sha256"]
    sources[name] = digest
    return raw.decode()


def integer(expression):
    def value(node):
        if isinstance(node, ast.Constant) and type(node.value) is int:
            return node.value
        if isinstance(node, ast.BinOp):
            left, right = value(node.left), value(node.right)
            if isinstance(node.op, ast.Mult):
                return left * right
            if isinstance(node.op, ast.LShift):
                return left << right
        raise AssertionError(ast.dump(node))
    return value(ast.parse(expression.strip(), mode="eval").body)


def fields(body):
    return {key: integer(value) for key, value in re.findall(r"\b(max_\w+)\s*:\s*([^,\n]+)", body)}


with tarfile.open(archive_path) as archive:
    before = {}
    for group, file, name in (
        ("native_runtime", "game/battle/runtime_control.rs", "RuntimeFactLimits"),
        ("authoring_execution", "authoring/control.rs", "ExecutionLimits"),
        ("expression.compile_and_evaluate", "game/value_expr/mod.rs", "ExprLimits"),
        ("authoring_trial", "authoring/trial.rs", "TrialLimits"),
    ):
        text = source(archive, file)
        body = text.split("impl Default for " + name + " {", 1)[1].split("\n}", 1)[0]
        before[group] = fields(body)
    text = source(archive, "game/value_expr/parse.rs")
    body = text.split("pub const HARD_MAX: Self = Self {", 1)[1].split("};", 1)[0]
    before["expression.parse"] = fields(body)
    text = source(archive, "game/battle/event_log.rs")
    before["battle_history"] = {
        field: integer(re.search(r"pub const " + constant + r": \w+ = ([^;]+);", text)[1])
        for field, constant in (
            ("max_events", "BATTLE_HISTORY_MAX_EVENTS"),
            ("max_encoded_entry_array_bytes", "BATTLE_HISTORY_MAX_ENCODED_BYTES"),
            ("max_span_ms", "BATTLE_HISTORY_MAX_SPAN_MS"),
        )
    }

# This evidence parser accepts only this repository's integer/struct RON shape;
# product parsing is independently exercised by the real RuntimeSafetySettings.
path = ROOT / "config/runtime_safety.ron"
tokens = re.findall(r"[A-Za-z_][A-Za-z_0-9]*|[0-9]+|[:(),]", re.sub(r"//[^\n]*", "", path.read_text()))
index = 0


def take(expected=None):
    global index
    result = tokens[index]
    index += 1
    assert expected is None or result == expected, (expected, result)
    return result


def struct_value():
    take("(")
    result = {}
    while tokens[index] != ")":
        key = take()
        take(":")
        assert key not in result
        result[key] = struct_value() if tokens[index] == "(" else int(take())
        if tokens[index] == ",":
            take(",")
    take(")")
    return result


configured = struct_value()
assert index == len(tokens) and configured["schema_version"] == 1
rows = []
for group, expected in before.items():
    current = configured
    for part in group.split("."):
        current = current[part]
    assert current == expected, (group, expected, current)
    rows.extend(dict(field=group + "." + key, before=value, configured=current[key]) for key, value in expected.items())
assert len(rows) == 38
result = dict(passed=True, compared_defaults=len(rows), rows=rows, archived_sources=sources,
              archive_sha256=hashlib.sha256(archive_path.read_bytes()).hexdigest(),
              configured_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
              scope="Numeric parity only. Execution/error parity uses current full suites and public predecessor oracles.")
with (Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_runtime_fault_isolation/execution/single_goal/corrections/f02_origin_001/compatibility/current_001') / "evidence/DEFAULT_NUMBERS_001.json").open("x") as stream:
    json.dump(result, stream, indent=2)
    stream.write("\n")
print(json.dumps(dict(passed=True, compared_defaults=len(rows))))
