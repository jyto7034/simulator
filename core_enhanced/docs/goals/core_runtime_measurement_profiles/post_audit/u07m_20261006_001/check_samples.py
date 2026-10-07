import json,pathlib,hashlib,datetime
R=pathlib.Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_runtime_measurement_profiles';E=H/'evidence';A=pathlib.Path(__file__).parent
load=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
out={};raws={}
def window(w):
 b=w['baseline']['live_requested_bytes'];e=w['end']['live_requested_bytes'];assert w['baseline']['instrumented'] and w['end']['instrumented'];assert w['live_delta_bytes']==e-b;assert w['peak_requested_bytes']>=max(b,e);assert not w['boundary_updates_unavailable'] and w['retiring_hooks_at_end']==0
for family,n in [('playback_samples_003',26),('lifecycle_samples_003',10)]:
 d=load(E/family/'SUMMARY.json');assert len(d['rows'])==n; rows={};raws[family]={}
 for row in d['rows']:
  name=row['name'];p=pathlib.Path(row.get('sample_path',row['command'][2]));m=pathlib.Path(row['command'][1]);assert sha(p)==row['sample_sha256'];assert sha(m)==row['manifest_sha256'];r=[json.loads(x) for x in p.read_text().splitlines()];header,footer=r[0],r[-1];assert header['record']=='header' and footer['record']=='summary';assert footer==row['summary'];assert footer['driver_errors']==0 and not footer['export_io_failed'];assert header['binary_sha256']==d['binary_sha256'];assert header['manifest_sha256']==row['manifest_sha256'];assert header['source_sha256_supplied']==d['source_sha256'];window(footer['process_window']);assert footer['process_rss']['status']=='available'
  finals=[x for x in r if x['record']=='final_state'];steps=[x['outcome'] for x in r if x['record']=='step'];native=[x['summary'] for x in r if x['record']=='native_operation'];assert finals==row['final_states'];assert steps==row.get('steps',row.get('step_results'));assert row['exit_code']==0
  report=load(p.with_suffix('.stdout'));assert report['summary']==footer and report['summary_written'] and not report['summary_missing']
  if row.get('stdout_sha256'):assert sha(p.with_suffix('.stdout'))==row['stdout_sha256'];assert sha(p.with_suffix('.stderr'))==row['stderr_sha256']
  prior=row.get('prior_receipt',row.get('prior_execution_receipt'))
  if prior:
   pr=load(pathlib.Path(prior));assert pr['source_unchanged'] and pr['source_before']['sha256']==d['source_sha256'];assert pr['log_sha256']==sha(pathlib.Path(prior).with_suffix('.log'))
  rows[name]={'accepted':footer['accepted_steps'],'rejected':footer['rejected_steps'],'native_seen':footer['raw_operations_seen'],'final_states':len(finals),'sample_sha256':sha(p),'profile_caps':[x['selected_hard'] for x in header['sessions']],'window':footer['process_window']};raws[family][name]=(header,steps,finals,native,footer,r)
 out[family]=rows
p=raws['playback_samples_003']
for profile in ['default','dev16','dev32']:
 for mode in ['normal','detached','full']:assert p[f'normal_{profile}_{mode}'][1:3]==p[f'normal_{profile}_none'][1:3]
for mode in ['normal','detached','full']:assert p[f'queue_{mode}'][2]==p['queue_none'][2]
assert p['queue_full'][4]['queue_counters']['dropped']==76 and p['queue_full'][4]['raw_operations_seen']==1024
for case in ['actor','small','ice']:
 for profile in ['dev16','dev32']:
  a=p[f'objects_{case}_{profile}'];b=p[f'objects_{case}_default'];assert a[1]==b[1];assert [x['public_snapshots_sha256'] for x in a[2]]==[x['public_snapshots_sha256'] for x in b[2]]
iso=p['profile_isolation'];assert [s['selected_hard'] for s in iso[0]['sessions']]==[16777216,33554432]
for rec in iso[5]:
 if rec['record']=='native_operation':
  v=rec['summary'];i=(v['session']-1)%16;s=iso[0]['sessions'][i]
  for identity in [v['started_identity']]+[v[k]['identity'] for k in ['prepared','committed'] if v[k]]:
   assert bytes(identity['effective_profile_sha256']).hex()==s['effective_profile'];assert identity['selected_profile_hard_bytes']==s['selected_hard']
  if v['retained']:assert v['retained']['owner_hard_bytes']==s['selected_hard']
mis=load(E/'playback_samples_003/SUMMARY.json')['mismatch'];assert mis['exit_code']!=0 and not pathlib.Path(mis['command'][2]).exists();assert b'profile mismatch' in (E/'playback_samples_003/mismatch.stderr').read_bytes();assert sha(E/'playback_samples_003/mismatch.stderr')==mis['stderr_sha256']
l=raws['lifecycle_samples_003']
for family in ['area','ttl']:
 for mode in ['none','normal','detached','full']:
  assert l[family+'_'+mode][1:3]==l[family+'_none'][1:3];assert l[family+'_'+mode][4]['final_states_seen']==3 and l[family+'_'+mode][4]['rejected_steps']==0
assert l['low_strict'][4]['rejected_steps']==1 and l['low_dev16'][4]['rejected_steps']==0
hard=[r['retained'] for r in l['low_strict'][3] if r['retained'] and r['retained']['first_hard_exceed']];soft=[r['retained'] for r in l['low_dev16'][3] if r['retained'] and r['retained']['first_soft_exceed']];assert hard and soft
assert all(r['owner_hard_bytes']==262144 and r['first_hard_exceed']['requested_bytes']>262144 for r in hard);assert all(r['owner_hard_bytes']==16777216 and r['profile_warning_bytes']==262144 and r['first_hard_exceed'] is None for r in soft)
h=load(E/'host_samples_002/SUMMARY.json');assert len(h['rows'])==4;host=[]
for row in h['rows']:
 folder=E/'host_samples_002'/row['case'];assert sha(folder/'config/development.toml')==row['config_sha256'];assert sha(folder/'observation.ron')==row['observation_sha256']
 for phase in row['phases']:
  name=phase['phase'];assert sha(folder/(name+'.log'))==phase['log_sha256'];assert sha(folder/(name+'.messages.jsonl'))==phase['messages_sha256'];assert load(folder/(name+'.details.json'))==phase['details'];assert phase['exit_code']==0;window(phase['process_window'])
  messages=[json.loads(x) for x in (folder/(name+'.messages.jsonl')).read_text().splitlines()];sent=[x['message'] for x in messages if x['direction']=='send'];received=[x['message'] for x in messages if x['direction']=='receive']
  cmd='start_new_game' if name=='initial' else 'continue_run';assert any(x.get('behavior',{}).get('type')==cmd for x in sent)
  assert any(x.get('type')=='command_result' and x.get('ok') for x in received)
  if name=='restarted':
   assert any(x.get('type')=='error' and x.get('code')=='missing_battle_state_authority' for x in received);assert (folder/'reconnected.messages.jsonl').is_file()
   if row['observation']:
    active,retired=phase['details'];assert active['sessions'] and not retired['sessions'];assert retired['raw_operations_received']>=active['raw_operations_received'];assert 0<len(retired['raw_operations'])<=300 and len(retired['ring'])<=300
    i=active['sessions'][0]['current']['identity'];assert bytes(i['effective_profile_sha256']).hex()==row['effective_profile_sha256'];assert i['selected_profile_hard_bytes']==row['selected_hard_bytes'];assert any(x['operation']=='ContinueCheckpoint' and x['outcome']=='Accepted' and x['committed'] for x in active['raw_operations']);assert any(x['retained'] and x['retained']['accepted_total_bytes']>0 for x in active['raw_operations'])
  host.append({'case':row['case'],'phase':name,'exit':phase['exit_code'],'shutdown_signal':phase['shutdown_signal'],'messages':len(messages),'profile':row['effective_profile_sha256'],'window':phase['process_window']})
out.update(host_phases=host,low_profile={'hard':hard,'soft':soft},at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),errors=[],scope='Read-only independently decoded actual CLI JSONL and host messages/details. Low262144 warning/hard sample is explicitly separate from deployment8MiB; exact8/16/32MiB boundaries use native tests. Shared process allocator/RSS not attributed to individual session.')
(A/'SAMPLE_VERIFICATION.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({'playback':26,'lifecycle':10,'host_phases':len(host),'errors':[]}))
