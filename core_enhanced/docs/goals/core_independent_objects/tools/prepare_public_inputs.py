"""Explicit U05 successor copies; never rewrite predecessor packs or receipts."""
import hashlib, json, re, shutil
from pathlib import Path
from initialize import ROOT, HERE

OUT = HERE / 'execution/single_goal/compatibility/current_001'
INPUTS = OUT / 'inputs'
OLD = ROOT / 'core_enhanced/docs/goals/core_result_persistence'
PRIOR = OLD / 'execution/single_goal/compatibility/current_003'
R08 = Path('core_enhanced/docs/goals/content_authoring_r08_r12_runtime')
E = R08 / 'execution/single_goal/evidence'
REVIEW = R08 / 'reviews/r08_r12_20261002_001/evidence'
PACKS = [E / 'cli_predecessor_current_001' / x for x in ('r06', 'r07_snapshot', 'r07_live')]
PACKS += [E / 'r10_public_cli_002' / x for x in ('snapshot', 'live')]
PACKS += [E / 'r12_public_cli_003' / x for x in ('hello', 'protection', 'released', 'effectssettled')]
PACKS += [REVIEW / folder / case for folder, cases in (
    ('timeout_marker_probe_002', ('early_cancel', 'cancel', 'detach')),
    ('timeout_marker_probe_003', ('inflight_cancel', 'state_exit'))) for case in cases]
PACKS += [Path('core_enhanced/docs/goals/core_runtime_safety_observability/evidence/public_profile_002')]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()

def spans(text, token):
    """Balanced named RON nodes, ignoring quoted strings and line comments."""
    for match in re.finditer(r'\b' + re.escape(token) + r'\s*\(', text):
        start = match.end() - 1
        depth, quoted, escape, comment = 0, False, False, False
        for i in range(start, len(text)):
            c = text[i]
            if comment:
                if c == '\n': comment = False
                continue
            if quoted:
                if escape: escape = False
                elif c == '\\': escape = True
                elif c == '"': quoted = False
                continue
            if text[i:i+2] == '//': comment = True; continue
            if c == '"': quoted = True
            elif c == '(': depth += 1
            elif c == ')':
                depth -= 1
                if depth == 0:
                    yield match.start(), i+1
                    break
        else: raise AssertionError(('unclosed node', token))

def transform_skills(text):
    changes = []
    for start, end in reversed(list(spans(text, 'SkillDef'))):
        block = text[start:end]
        skill = re.search(r'\bid:\s*"([^"]+)"', block)[1]
        refs = []
        for a, b in spans(block, 'SkillStepDef'):
            step = block[a:b]
            sid = re.search(r'\bid:\s*"([^"]+)"', step)[1]
            # Every supported effect is a named (id, op) tuple. Inspect only
            # finite beneficial operations; Control and periodic buffs are independent.
            for effect in re.finditer(r'\(\s*id:\s*"([^"]+)"\s*,\s*op:\s*(ShieldExpr|Sustained)\s*\(', step):
                op = step[effect.end()-len(effect[2])-1:]
                tail = step[effect.end():]
                finite = effect[2] == 'ShieldExpr'
                if effect[2] == 'Sustained':
                    kind = re.match(r'\s*definition:\s*(\w+)', tail)[1]
                    finite = kind != 'Control'
                    if kind == 'Modifiers':
                        node_end = next(spans(step[effect.start():], 'Sustained'))[1]
                        node = step[effect.start():][:node_end]
                        mods = re.findall(r'stat:\s*(\w+).*?value:\s*(-?\d+)', node, re.S)
                        assert mods, (skill, sid)
                        finite = not any(int(v)>0 if s=='AttackIntervalMs' else int(v)<0 for s,v in mods)
                if finite: refs.append((sid, effect[1]))
        descriptor = 'Action' if not refs else 'ActionAndEffects(effects: [' + ', '.join(
            f'(step: "{s}", effect: "{e}")' for s,e in refs) + '])'
        assert 'use_lifetime:' not in block
        block = block.replace('SkillDef(', 'SkillDef(use_lifetime: ' + descriptor + ', ', 1)
        # Manual fragment forms must reuse the accepted anchor. Keep autonomous
        # Funeral skill unchanged; append a separately named manual clone below.
        if skill == 'fragment_funeral_butterfly_eulogy':
            block = block.replace('RetargetOnStep', 'ReuseCastTarget')
        text = text[:start] + block + text[end:]
        changes.append({'skill':skill, 'descriptor':descriptor})
    funeral = [(a,b) for a,b in spans(text,'SkillDef') if re.search(r'\bid:\s*"funeral_butterfly_eulogy_volley"',text[a:b])]
    assert len(funeral) == 1
    a,b = funeral[0]
    clone = text[a:b].replace('id: "funeral_butterfly_eulogy_volley"', 'id: "fragment_funeral_awakened_eulogy"', 1).replace('RetargetOnStep','ReuseCastTarget')
    text = text[:b] + ',\n' + clone + text[b:]
    return text, changes

def main():
    OUT.mkdir(parents=True, exist_ok=False)
    rows, descriptors = [], {}
    for relative in PACKS:
        src, dst = ROOT / relative, INPUTS / relative
        for p in src.rglob('*'):
            if not p.is_file() or not (p.is_relative_to(src/'content') or p.name in ('selection.json','scenario.json')): continue
            q = dst / p.relative_to(src)
            q.parent.mkdir(parents=True, exist_ok=True)
            text = p.read_text()
            reasons = []
            if p.name == 'pack.ron':
                text, count = re.subn(r'catalog_schema:\s*4\b','catalog_schema:5',text)
                assert count == 1
                reasons.append('catalog schema 4 -> 5')
            elif p.parent.name == 'skills':
                text, details = transform_skills(text)
                descriptors[str(q.relative_to(HERE))] = details
                reasons += ['explicit action/effect lifetime', 'manual Funeral anchor clone']
            elif p.parent.name == 'skill_fragments':
                text, count = re.subn(r'(skill_id:\s*"[^"]+")',r'\1, resonance_max: 100',text)
                assert count > 0
                text = text.replace('skill_id: "funeral_butterfly_eulogy_volley"','skill_id: "fragment_funeral_awakened_eulogy"')
                reasons += ['explicit per-form maximum 100', 'manual Funeral anchor clone reference']
            elif p.parent.name == 'equipments':
                text, count = re.subn(r'\s*resonance_recovery_per_tick:\s*\d+\s*,','',text)
                if count: reasons.append('remove obsolete Rest resonance authoring')
            elif p.parent.name == 'employees':
                assert 'resonance:' not in text, 'employee override needs explicit semantic migration'
            q.write_text(text)
            rows.append({'source':str(p.relative_to(ROOT)), 'source_sha256':sha(p), 'copy':str(q.relative_to(ROOT)), 'copy_sha256':sha(q), 'changes':reasons})
    # Current harness code only: historical outputs stay in their original goal.
    for p in PRIOR.iterdir():
        if p.suffix not in ('.py','.rs'): continue
        text = p.read_text().replace(str(PRIOR),str(OUT)).replace(str(OLD),str(HERE))
        if p.name in ('public_cli.py','public_profile_probe.py','host_peak_failure_probe.rs'):
            text = text.replace(str(ROOT / R08),str(INPUTS / R08))
            text = text.replace("E=H.parents[1]/'evidence'", "E=H.parents[1]/'evidence'")
            if p.name == 'public_profile_probe.py':
                text = text.replace('fixture = ROOT / "core_enhanced/', 'fixture = Path(' + repr(str(INPUTS)) + ') / "core_enhanced/')
        if p.name == 'public_probe.py':
            text = text.replace("run.PRIOR = run.ROOT /", 'run.PRIOR = Path(' + repr(str(INPUTS)) + ') /')
        if p.name == 'probe.py':
            text = text.replace("fixture = R / old['fixture']", "fixture = Path(" + repr(str(INPUTS)) + ") / old['fixture']")
        if p.name in ('regressions.py','summarize_r05_current.py'):
            text = text.replace('_005', '_001')
        if p.name == 'regressions.py': text = text.replace("C / 'summarize_r05.py'", "C / 'summarize_r05_current.py'")
        (OUT/p.name).write_text(text)
        rows.append({'source':str(p.relative_to(ROOT)), 'source_sha256':sha(p), 'copy':str((OUT/p.name).relative_to(ROOT)), 'copy_sha256':sha(OUT/p.name), 'changes':['U05 destinations and explicit migrated input paths']})
    for name in ('public_continuation_probe.py','public_rewind_probe.py','public_fault_probe.py','final_fault_probes.py'):
        p = OLD/'tools'/name
        text = p.read_text().replace('core["version"] == 10','core["version"] == 11')
        text = text.replace("P=ROOT/'core_enhanced/", 'P=Path(' + repr(str(INPUTS)) + ")/'core_enhanced/")
        text = text.replace('original_packs=ROOT/"core_enhanced/', 'original_packs=Path(' + repr(str(INPUTS)) + ')/"core_enhanced/')
        q = HERE/'tools'/name
        assert not q.exists()
        q.write_text(text)
        rows.append({'source':str(p.relative_to(ROOT)), 'source_sha256':sha(p), 'copy':str(q.relative_to(ROOT)), 'copy_sha256':sha(q), 'changes':['U05 destination, continuation 11, migrated fault pack']})
    shutil.copyfile(OLD/'evidence/observation_fault_001.ron',HERE/'evidence/observation_fault_001.ron')
    (OUT/'ADAPTATION_001.json').write_text(json.dumps({'status':'prepared_not_validated','files':rows,'descriptors':descriptors,'unchanged_oracles':True},indent=2)+'\n')
    print(json.dumps({'copied_files':len(rows),'input_packs':len(PACKS),'status':'prepared_not_validated'}))

if __name__ == '__main__': main()
