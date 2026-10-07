"""Select executed current U07-M evidence, preserving every earlier result."""
import argparse
from evidence_common import ROOT, HERE, S, C, read, write, ref, receipt, identity

p = argparse.ArgumentParser()
for name, default in [('integrated','001'), ('public','001'), ('starter','001'), ('release','001'),
                      ('pc','002'), ('prepare','002'), ('r05','002'), ('host','002'),
                      ('playback','003'), ('lifecycle','003')]:
    p.add_argument('--' + name, default=default)
a = p.parse_args()
assert all(value.isdigit() for value in vars(a).values())
current = identity()
suites = ['integrated_' + name + a.integrated for name in
          ['core_default','core_fixtures','core_observation','host_default','host_observation','host_measurement']]
checks = ['integrated_' + name + a.integrated for name in ['check','event_log','clippy_core','clippy_host','heap']]
checks += ['fmt_current001', 'host_measurement_heap001', 'measurement_support006',
           'pc_prepare' + a.prepare, 'compat_pc_' + a.pc,
           'compat_pc_gameplay_' + a.pc, 'compat_pc_cost_' + a.pc,
           'compat_r05_prepare_' + a.r05, 'compat_r05_summary_' + a.r05, 'compat_r05_dependencies_' + a.r05,
           'public_guards' + a.public, 'public_guards_allocator' + a.public,
           'release_build_' + a.release, 'host_actual' + a.host, 'lifecycle_actual' + a.lifecycle]
public = [name + '_' + a.public for name in
          ['compat_prepared_public','compat_prepared_boundary','compat_input_validation','compat_default_parity',
           'compat_public_oracle','compat_public_trace','compat_public_cli','compat_public_profile',
           'compat_public_boundary','compat_public_api','public_schema','public_independent_area',
           'public_continuation','public_rewind','public_object_schema']]
public += ['public_starter_schema_' + a.starter, 'public_objects_' + a.release, 'public_object_combat_' + a.release]
public += ['final_fault_' + a.public + '_' + mode for mode in
           ['build_on','none','detached','queue_full','on','websocket','build_off','off']]
r05 = 'compat_r05_' + a.r05
bindings = [receipt(name, current) for name in suites + checks + public]
bindings += [receipt(r05, current, r05=True), receipt('playback_actual' + a.playback, current, cli=True)]
summaries = {
    'prepared_host': C / ('public_probe_' + a.public) / 'RESULT.json',
    'prepared_boundary': C / ('boundary_' + a.public) / 'RESULT.json',
    'input_validation': C / ('input_validation_' + a.public) / 'SUMMARY.json',
    'defaults': C / 'evidence' / ('DEFAULT_NUMBERS_' + a.public + '.json'),
    'oracle': C / 'evidence' / ('public_after_' + a.public) / 'SUMMARY.json',
    'trace': C / 'evidence' / ('public_after_' + a.public + '_trace') / 'SUMMARY.json',
    'cli': C / 'evidence' / ('cli_current' + ('_' + a.public if a.public != '001' else '')) / 'CLI_FINAL_SUMMARY_002.json',
    'profile': C / 'evidence' / ('public_profile_' + a.public) / 'SUMMARY.json',
    'boundary': C / 'evidence' / ('public_boundary_' + a.public) / 'SUMMARY.json',
    'api': C / 'evidence' / ('public_api_' + a.public) / 'RESULT.json',
    'fault': HERE / 'evidence' / ('final_fault_' + a.public + '_SUMMARY.json'),
    'host_actual': HERE / 'evidence' / ('host_samples_' + a.host) / 'SUMMARY.json',
    'playback_actual': HERE / 'evidence' / ('playback_samples_' + a.playback) / 'SUMMARY.json',
    'lifecycle_actual': HERE / 'evidence' / ('lifecycle_samples_' + a.lifecycle) / 'SUMMARY.json',
}
for key, label in [('starter_schema',a.starter),('schema',a.public),('independent_area',a.public),
                   ('continuation',a.public),('rewind',a.public),('object_schema',a.public),
                   ('objects',a.release),('object_combat',a.release)]:
    summaries[key] = HERE / 'evidence' / ('public_' + key + '_' + label) / 'SUMMARY.json'
for key, path in summaries.items():
    assert read(path).get('passed', True), key
result = {'goal':'GOAL-CMRP-01','source_sha256':current['sha256'],'labels':vars(a),'target':'u07m_001',
          'suites':suites,'checks':checks,'public_receipts':public,'r05_receipt':r05,
          'receipt_bindings':bindings,'public_summaries':{key:ref(path) for key,path in summaries.items()},
          'pc_raw':ref(C/'evidence'/('pc_raw_current_'+a.pc+'.json')),
          'pc_summary':ref(C/'evidence'/('PC_SUMMARY_CURRENT_'+a.pc+'.json')),
          'pc_cost':ref(C/'evidence'/('PC_IMMEDIATE_ACCOUNTED_U07_'+a.pc+'.json')),
          'r05_summary':ref(C/'evidence'/('R05_SUMMARY_CURRENT_'+a.r05+'.json')),
          'performance_environments':[ref(C/'evidence'/('PERFORMANCE_ENVIRONMENT_'+label+'_'+phase+'.json'))
                                      for label,phase in [(a.pc,'pc'),(a.r05,'r05')]],
          'reuse_rule':'Each receipt retains its actual executed source and output. Only exact server-main CLI non-impact and README-only whole-product deltas are accepted through explicit proofs. Earlier failures and raw outputs remain original; no new independent verdict.'}
write(S/'EVIDENCE_SELECTION.json',result)
print('PASS current evidence selection with explicit scoped reuse')
