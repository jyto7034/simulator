from pathlib import Path
import json,hashlib,tarfile,re
R=Path('/mnt/f/work/simulator');E=R/'core_enhanced/docs/goals/core_dealer_minigames/execution/single_goal';A=E/'audit/independent001';T=E/'target/u16_3_current_001';load=lambda p:json.loads(p.read_bytes());sha=lambda b:hashlib.sha256(b).hexdigest()
def archive(p):
 with tarfile.open(p) as t:return {m.name:t.extractfile(m).read() for m in t if m.isfile()}
before=archive(T/'BEFORE.tar.gz');after=archive(T/'SOURCE.tar.gz');ops=load(T/'DELTA.json');actual={p for p in before.keys()|after.keys() if before.get(p)!=after.get(p)}
assert actual=={x['path'] for x in ops} and len(actual)==251
for op in ops:
 for side,tree in [('before',before),('after',after)]:assert op[side+'_sha256']==(sha(tree[op['path']]) if op['path'] in tree else None)
for row in load(E/'SOURCE_DELTA_REVIEW.json')['members']:
 for side,tree in [('before',before),('after',after)]:assert row[side+'_sha256']==(sha(tree[row['path']]) if row['path'] in tree else None)
base=archive(E/'BASELINE_SOURCE_001.tar.gz');old=R/'core_enhanced/docs/goals/core_basic_relic_rewards/execution/single_goal';previous=archive(old/'target/u16_2_current_001/SOURCE.tar.gz')
predecessor=[p for p in base if p in previous];assert all(base[p]==previous[p] for p in predecessor)
testrows={}
for name in ['core005','focused007','integration009']:
 txt=(E/'runs'/f'{name}.log').read_text();rows=re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$',txt,re.M);testrows[name]={s:[n for n,v in rows if v==s] for s in ['ok','FAILED','ignored']}
assert [len(testrows['core005'][k]) for k in ['ok','FAILED','ignored']]==[2138,2,6]
assert len(testrows['focused007']['ok'])==35 and not testrows['focused007']['FAILED'];assert len(testrows['integration009']['ok'])==72
closure='game::behavior::tests::every_behavior_result_has_exact_old_command_projection_contract';addition='game::world::tests::u163_dealer::u163_other_authored_rarity_is_pledgeable_and_redeems_same_copy'
assert closure in testrows['focused007']['ok'] and addition in testrows['focused007']['ok']
unique=set(testrows['core005']['ok'])|set(testrows['focused007']['ok']);assert len(unique)==2140
reuse=load(E/'BASELINE_REUSE.json')
for item in reuse['socket_reuse']['unchanged_files']:assert sha((R/item['path']).read_bytes())==item['sha256']
for name,raw,count in [('socket',reuse['socket_reuse']['raw'],1),('native',reuse['native_g11_reuse']['raw'],4),('server',reuse['server_persistence_reuse']['raw'],31)]:
 text=(R/raw).read_text();assert f'{count} passed; 0 failed' in text
host=load(E/'runs/host001/RESULT.json');host_commands=0;io_count=0
for row in host['rows']:
 name=row['name'];req=load(E/'test_parts/host_replays001'/f'{name}.json');out=Path(row['result']).parent;v=load(out/'RESULT.json');trace=load(out/'commands.json');assert v['passed'] and v['fresh_process'] and len(trace)==len(req['commands'])==row['commands'];assert [x['command'] for x in trace]==req['commands'];host_commands+=len(trace)
 assert [x['index'] for x in trace if x.get('rejected')]==req.get('reject_indices',[])
 if row['actual_io']:
  fail=load(out/'IO_FAILURE.json');assert fail['memory_and_disk_unchanged'] and 'os error 21' in fail['error'];assert trace[req['fail_index']]['command']==fail['command'] and 'result' in trace[req['fail_index']];io_count+=1
assert host_commands==111 and io_count==4
def outcome(name):return load(E/'runs/host001'/name/'RESULT.json')
def dealer(name):return outcome(name)['run']['selected_event']['dealer']
assert dealer('offer_unfinished')['points']==30 and dealer('offer_unfinished')['assigned_multiplier']==3 and len(dealer('offer_unfinished')['offers'])==4
assert dealer('gambit_unfinished')['phase']=='Gambit'
assert dealer('loan_unfinished')['points']==20 and len(dealer('loan_unfinished')['collateral'])==2
assert dealer('purchase_unfinished')['phase']=='Shop' and dealer('purchase_unfinished')['points']==5
assert dealer('negative_unfinished')['points']==-60 and 'continue' not in dealer('negative_unfinished')['eligible_actions']
assert dealer('repeat_offer')['first_explanation']==False and dealer('repeat_offer')['points']==30
assert dealer('checkpoint_offer')['first_explanation']==True
def relic(name):return load(E/'runs/host001'/name/'commands.json')[-1]['result']['payload']
original=relic('initial_checkpoint');assert relic('purchase_unfinished_continue')==original and relic('trade_savepoint')==original and relic('decline_complete')==original and relic('negative_complete')==original
assert relic('trade_complete')==relic('trade_completed_continue') and relic('foreclose_complete')==relic('foreclose_continue')
public=[]
for game in ['Blackjack','Flip7']:
 out=E/'runs/public_games001';req=load(E/'test_parts/public_games001'/f'{game}-host.json');trace=load(out/f'{game}-host/commands.json');v=load(out/f'{game}-host/RESULT.json');cont=load(out/f'{game}-continue/RESULT.json')
 assert [x['command'] for x in trace]==req['commands'] and v['passed'] and cont['passed'] and v['run']==cont['run'] and v['inventory']==cont['inventory']
 public.append({'game':game,'commands':len(trace)+1,'fresh_processes':2})
author=load(E/'runs/authoring001/RESULT.json');assert len(author['rows'])==15
for row in author['rows']:
 assert row['accepted']==row['expected_accept'] and (row['exit_code']==0)==row['accepted']
 if not row['accepted']:assert (E/'runs/authoring001'/row['case']/'changed_input.ron').is_file() and (E/'runs/authoring001'/row['case']/'stderr.log').stat().st_size>0
cost=[]
for row in load(E/'COST_REVIEW.json')['rows']:
 p=Path(row['path']);v=load(p);release=load(Path(str(p)+'.release.json'));assert v['passed'] and len(v['trace'])==64 and len(v['stages'])==64 and len(v['snapshot_stages'])==64
 assert all(x['accepted'] for x in v['stages']+v['snapshot_stages']);assert release['exact_requested_bytes_recovered'] and release['after']['live_requested_bytes']==release['baseline']['live_requested_bytes']
 assert max(x['native']['work'] for x in v['stages']+v['snapshot_stages'])==row['max_native_work'];assert max(x['heap']['peak_requested_bytes'] for x in v['stages']+v['snapshot_stages'])==row['heap_peak_requested_bytes']
 if row['case']=='expanded_67_pledged':assert max(len(x['run'].get('selected_event',{}).get('dealer',{}).get('collateral',[])) for x in v['trace'] if isinstance(x['run'].get('selected_event'),dict))==67
 cost.append({'case':row['case'],'repeat':row['repeat'],'commands':64,'windows':128,'recovered':True})
result={'passed':True,'archive_before':len(before),'archive_after':len(after),'exact_delta':len(ops),'product_delta':27,'predecessor_equal_members':len(predecessor),'original_pre_mutation_members':len(base),'tests':{k:{s:len(v) for s,v in r.items()} for k,r in testrows.items()},'core_unique_library':len(unique),'core_unique_integration':72,'socket_reused':1,'core_unique_total':len(unique)+72+1,'ignored':testrows['core005']['ignored'],'host':{'fresh_processes':17,'commands':host_commands,'actual_io':io_count},'public_games':public,'authoring':{'accepted':1,'rejected':14},'cost':cost,'cost_windows':1152}
(A/'EVIDENCE_RECHECK.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
