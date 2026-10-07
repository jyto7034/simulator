import hashlib,json
from pathlib import Path
import run
H=Path(__file__).resolve().parent;E=H/'evidence';current=run.identity();rows=[]
required=['integrated_all_003','integrated_all_003_environment_excluded','integrated_feature_003','integrated_feature_003_environment_excluded','integrated_check_003','integrated_event_check_003','integrated_release_003','integrated_clippy_003','integrated_fmt_003','integrated_public_export_003','pc_final_005','r05_final_005']
cli=json.loads((E/'CLI_FINAL_SUMMARY_002.json').read_text());required += [r['name'] for r in cli['commands']]
for name in required:
 path=E/f'{name}.json';d=json.loads(path.read_text());assert d['source_before']==d['source_after']==current,name
 assert hashlib.sha256((E/f'{name}.log').read_bytes()).hexdigest()==d['log_sha256'],name
 expected=101 if name in ['integrated_all_003','integrated_feature_003'] else None if name=='r05_final_005' else 0
 if expected is not None:assert d['exit_code']==expected,(name,d['exit_code'])
 rows.append({'name':name,'receipt_sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'log_sha256':d['log_sha256'],'exit_code':d['exit_code'],'source_matches':True,'test_passed':d.get('test_passed'),'test_failed':d.get('test_failed')})
pc=json.loads((E/'pc_isolate_final_005.json').read_text());assert pc['current_source']==pc['copied_source']==current
base=Path(pc['isolate']);built=pc['built_source']
for name,size,sha in built['files']:
 data=(base/name).read_bytes();assert len(data)==size and hashlib.sha256(data).hexdigest()==sha,name
for name,sha in pc['supplemental'].items():assert hashlib.sha256((base/name).read_bytes()).hexdigest()==sha
# Recheck the exact predecessor used to migrate only the log envelope version.
cap=json.loads((E/'r07_log_capture_identity_001.json').read_text())
golden=Path(cap['isolate']);old_manifest=H/'baseline/MANIFEST.json'
assert hashlib.sha256(old_manifest.read_bytes()).hexdigest()==cap['baseline_manifest_sha256']
for row in json.loads(old_manifest.read_text())['files']:
    data=(golden/row['path']).read_bytes()
    expected=cap['instrumented_sha256'] if row['path']==cap['instrumented_path'] else row['sha256']
    assert hashlib.sha256(data).hexdigest()==expected,row['path']
migration=json.loads((E/'LOG_VERSION_MIGRATION_001.json').read_text())
assert len(migration['captures'])==65
for row in migration['captures']:
    data=(H/row['path']).read_bytes()
    assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['original_sha256']
    prefix=b'{"version":31,';assert data.startswith(prefix)
    converted=b'{"version":32,'+data[len(prefix):]
    assert len(converted)==len(data) and hashlib.sha256(converted).hexdigest()==row['v32_sha256']
pack_migration=json.loads((E/'CLI_PREDECESSOR_MIGRATION_001.json').read_text())
assert len(pack_migration['packs'])==3
for pack in pack_migration['packs']:
    old=run.ROOT/pack['original'];new=H/pack['current_copy']
    expected=sorted(row['path'] for row in pack['files'])
    assert sorted(str(p.relative_to(old)) for p in old.rglob('*') if p.is_file())==expected
    assert sorted(str(p.relative_to(new)) for p in new.rglob('*') if p.is_file())==expected
    for row in pack['files']:
        a=(old/row['path']).read_bytes();b=(new/row['path']).read_bytes()
        assert hashlib.sha256(a).hexdigest()==row['before_sha256']
        assert hashlib.sha256(b).hexdigest()==row['after_sha256']
        if a != b:
            assert row['path']=='content/official/data/run/policy.ron'
            line=(pack_migration['inserted_exact_line']+'\n').encode()
            assert b.count(line)==1 and b.replace(line,b'',1)==a
with (E/'VALIDATION_CURRENTNESS.json').open('x') as f:json.dump({'source_digest':current['digest'],'source_file_count':current['file_count'],'all_selected_receipts_current':True,'PC36_unmodified_copy_matches_current':True,'PC36_built_instrumented_files_match_after_execution':True,'predecessor_capture_source_and_65_log_version_migrations_reverified':True,'three_predecessor_pack_copies_only_explicit_corrosion_policy_changed':True,'commands':rows},f,indent=2);f.write('\n')
print(json.dumps({'source_digest':current['digest'],'receipts':len(rows),'source_file_count':current['file_count']}))
