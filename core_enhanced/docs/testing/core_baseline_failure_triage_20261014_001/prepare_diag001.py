"""Diagnostic-only fixture updates in a verified /tmp source copy."""
from pathlib import Path
import difflib, json, re

O = Path(__file__).resolve().parent
D = Path('/tmp/core-failure-triage-20261014-001')
before = {}
edited = {}
def edit(path, change):
    p = D / path
    s = edited.get(path, p.read_text())
    before.setdefault(path, s)
    t = change(s)
    assert s != t, path
    edited[path] = t

def selected_function(s, name, transform):
    m = re.search(r'^(\s*)fn ' + re.escape(name) + r'\(', s, re.M)
    assert m, name
    start = m.start()
    # Next function at the same indentation is a safe boundary for these tests.
    indent = m.group(1).split('\n')[-1]
    n = re.search(r'^' + re.escape(indent) + r'fn \w+\(', s[m.end():], re.M)
    end = m.end() + n.start() if n else len(s)
    body = s[start:end]
    replacement = transform(body)
    assert replacement != body, name
    return s[:start] + replacement + s[end:]

def current_starters(s):
    a = s.index('        let fixture = Self::official();', s.index('pub(crate) fn official_with_legacy_starters'))
    b = s.index('\n    pub(crate) fn write', a)
    return s[:a] + '        Self::official()\n    }\n' + s[b:]
edit('core_enhanced/src/game/data/content_pack/tests.rs', current_starters)

def manifested_slot(s):
    a = s.index('    let target_slot_id = template', s.index('fn force_first_available_node('))
    b = s.index('    let start_slot_id', a)
    old = s[a:b]
    replacement = old.replace('.find(|slot| slot.allowed_categories.contains(&category))', '''.filter(|slot| slot.allowed_categories.contains(&category))
        .min_by_key(|slot| {
            let node = run.map.nodes.iter().find(|node| node.slot_id == slot.id).unwrap();
            // Preserve an existing authored difficulty when preparing a battle.
            if matches!(category, MapNodeCategory::Combat | MapNodeCategory::Boss) {
                (node.category != category, node.combat_difficulty.is_none())
            } else { (false, false) }
        })''')
    assert replacement != old
    return s[:a] + replacement + s[b:]
edit('core_enhanced/src/game/world/tests/mod.rs', manifested_slot)

# The command now requires an explicit loan list; resolver semantics are unchanged.
edit('core_enhanced/src/authoring/resolve.rs', lambda s: s.replace(
    '"type":"select_starter_employees", "employee_definition_ids"',
    '"type":"select_starter_employees", "loan_copy_ids":[], "employee_definition_ids"'))

intake = json.loads((O / 'INTAKE.json').read_text())
for row in intake['tests']:
    name = row['test'].split('::')[-1]
    path = row['source_candidates'][0]['path']
    failure = row['baseline_failure']
    if 'Number(14)' in failure:
        def update_version(body):
            return re.sub(r'(\["version"\]\s*,\s*)(json!\(12\)|12)',
                          lambda m: m.group(1) + ('json!(15)' if m.group(2).startswith('json') else '15'), body)
        edit(path, lambda s, name=name: selected_function(s, name, update_version))
    elif 'left: 80' in failure and 'right: 79' in failure:
        edit(path, lambda s, name=name: selected_function(s, name, lambda b: re.sub(r'(\.len\(\),\s*)79', r'\g<1>80', b)))
    elif 'left: 46' in failure and 'right: 45' in failure:
        edit(path, lambda s, name=name: selected_function(s, name, lambda b: b.replace('.len(), 45', '.len(), 46')))
    elif 'effective_skill_definition_copy' in failure:
        edit(path, lambda s, name=name: selected_function(s, name, lambda b: b.replace('error.site, "skill_definition_copy"', 'error.site, "effective_skill_definition_copy"')))

patch = []
for path, old in before.items():
    (D / path).write_text(edited[path])
    patch += difflib.unified_diff(old.splitlines(True), (D / path).read_text().splitlines(True), fromfile='a/' + path, tofile='b/' + path)
(O / 'DIAGNOSTIC_001.diff').write_text(''.join(patch))
(O / 'DIAGNOSTIC_001.json').write_text(json.dumps({'root': str(D), 'changed_files': list(before), 'purpose': 'Expose downstream assertions behind known invalid legacy fixture/count/version gates; original workspace unchanged', 'not_a_product_fix': True}, indent=2) + '\n')
print('diagnostic files', len(before))
