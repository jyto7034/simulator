"""Small safety checks for registration maintenance; never invokes Rust."""
import contextlib
import copy
import csv
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_inventory as inventory


class InventorySafetyTests(unittest.TestCase):
    def setUp(self):
        self.policy = {
            "lane_priority": {"parallel_candidate": 0, "resource_serial": 1,
                              "isolated_process": 2, "quiet_host": 3},
            "defaults": {name: name for name in (
                "parallel_candidate", "resource_serial", "isolated_process", "quiet_host")},
            "resources": {"checkout:files": {"scope": "checkout", "reason": "fixed paths"}},
            "rules": [
                {"id": "I", "paths": ["core_enhanced/src/*"], "lane": "isolated_process",
                 "reason": "process state", "resources": ["checkout:files"]},
                {"id": "Q", "paths": ["core_enhanced/src/*"], "lane": "quiet_host",
                 "reason": "timing"}],
        }

    def test_combined_constraints_survive_display_priority(self):
        row = {"lane": "quiet_host", "rule_ids": "I;Q", "parallel_validated": "false",
               "resource_groups": "checkout:files"}
        self.assertEqual(inventory.execution_constraints(row, self.policy), {
            "fresh_process": True, "quiet_host": True,
            "resources": ["checkout:files"], "parallel_validated": False})

    def test_added_declarations_not_line_movements_and_duplicates_counted(self):
        old = [{"source": "a.rs", "test_name": "a", "line": 1}]
        moved = [{"source": "a.rs", "test_name": "a", "line": 20}]
        self.assertEqual(inventory.declaration_changes(old, moved)["added_declarations"], [])
        self.assertEqual(inventory.declaration_changes(old, moved * 2)["added_declarations"],
                         [{"source": "a.rs", "test_name": "a", "count": 1}])

    def test_review_rejects_missing_extra_and_unreviewed_policy(self):
        for reviewed, changed_policy in [([], False), (["a", "extra"], False), (["a"], True)]:
            with self.subTest(reviewed=reviewed, policy=changed_policy), self.assertRaises(ValueError):
                inventory.require_review(["a"], reviewed, changed_policy, False)
        inventory.require_review(["a"], ["a"], True, True)

    def test_policy_rejects_unknown_resource_duplicate_and_empty_rule(self):
        for kind in ("resource", "duplicate", "paths"):
            policy = copy.deepcopy(self.policy)
            if kind == "resource":
                policy["rules"][0]["resources"] = ["unknown"]
            elif kind == "duplicate":
                policy["rules"][1]["id"] = "I"
            else:
                policy["rules"][0]["paths"] = []
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                inventory.validate_policy(policy)

    def test_update_lifecycle_and_pending_source_preservation(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            here = root / "registration"
            here.mkdir()
            source = root / "core_enhanced/src/lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text('#[test]\nfn first() {}\n// #[test] fn not_a_test() {}\n')
            (here / "policy.json").write_text(json.dumps(self.policy))
            with patch.object(inventory, "ROOT", root), patch.object(inventory, "HERE", here):
                rows, hashes = inventory.inventory(self.policy)
                snapshot = {"sources": hashes, "policy_sha256": hashlib.sha256(
                    (here / "policy.json").read_bytes()).hexdigest()}
                (here / "source_snapshot.json").write_text(json.dumps(snapshot))
                (here / "inventory.csv").write_text(inventory.encode_csv(rows))

                def run(*args):
                    output = io.StringIO()
                    with patch('sys.argv', ['check_inventory.py', *args]), contextlib.redirect_stdout(output):
                        code = inventory.main()
                    return code, json.loads(output.getvalue())

                self.assertEqual(run('--write')[0], 0)
                self.assertEqual(run()[0], 0)
                source.write_text(source.read_text() + '#[test]\nfn added() {}\n')
                code, report = run()
                self.assertEqual(code, 1)
                self.assertEqual(report['added_declarations'][0]['test_name'], 'added')
                saved = (here / 'source_snapshot.json').read_bytes()
                with self.assertRaises(ValueError):
                    run('--write')
                self.assertEqual(run('--render-table')[0], 0)
                self.assertEqual((here / 'source_snapshot.json').read_bytes(), saved)
                self.assertEqual(run()[0], 1)
                self.assertEqual(run('--write', '--reviewed-source',
                                     'core_enhanced/src/lib.rs')[0], 0)
                self.assertEqual(run()[0], 0)
                stored = list(csv.DictReader(io.StringIO((here / 'inventory.csv').read_text())))
                self.assertEqual(len(stored), 2)
                (here / 'CLASSIFICATION.ko.md').write_text('tampered')
                self.assertEqual(run()[0], 1)
                run('--render-table')
                (here / 'inventory.csv').write_text(inventory.encode_csv(stored[:1]))
                self.assertEqual(run()[0], 1)


if __name__ == '__main__':
    unittest.main()
