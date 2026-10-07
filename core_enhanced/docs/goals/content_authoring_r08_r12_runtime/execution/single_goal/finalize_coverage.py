"""Bind acceptance claims to successful selectors on the actual final source."""
import copy,hashlib,json,re,sys
from pathlib import Path
import run
H=Path(__file__).resolve().parent;E=H/'evidence';ROOT=run.ROOT
current=run.identity();digest=current['digest']
RECEIPTS=['integrated_all_003_environment_excluded','integrated_feature_003_environment_excluded']
logs=[]
for name in RECEIPTS:
 d=json.loads((E/f'{name}.json').read_text());assert d['exit_code']==0
 assert d['source_before']['digest']==d['source_after']['digest']==digest
 logs.append(set(re.findall(r'^test (\S+) \.\.\. ok$',(E/f'{name}.log').read_text(),re.M)))
names=logs[0]&logs[1]
def resolve(suffix):
 found=[n for n in names if n==suffix or n.endswith('::'+suffix)]
 assert len(found)==1,(suffix,found)
 return found[0]
M={
'A10-01':['r10_public_area_three_targets_then_one_self_heal','r10_public_branch_uses_real_hp_loss_and_never_executes_unselected_leaf','r12_shield_zero_actual_hp_result_expression_cannot_reuse_absorbed_damage'],
'A10-02':['r10_public_repeat_result_branch_and_interval_are_finite','r10_public_wait_parallel_join_and_released_delivery_cancellation','r10_public_flow_amount_generation_query_trace_and_cli_export'],
'A10-03':['r10_public_repeat_result_branch_and_interval_are_finite','r10_public_repeat_spends_empty_opportunity_without_catchup'],
'A10-04':['r10_public_branch_uses_real_hp_loss_and_never_executes_unselected_leaf','r10_public_wait_parallel_join_and_released_delivery_cancellation','r10_public_flow_faults_restore_timestamp_and_retry_exactly'],
'A10-05':['r12_public_boss_flow_projectile_area_milestones_and_cli_export','r10_public_area_three_targets_then_one_self_heal','instant_tile_area_origin_anchor_projects_range_from_cast_target_tile','relocation_preserves_ground_fixed_and_reanchors_follow_areas','manual_player_unit_invalidation_noops_without_retarget_or_refund'],
'A10-06':['r10_public_wait_parallel_join_and_released_delivery_cancellation','r08_public_state_exit_preserves_only_released_projectile','r10_structured_flow_preserves_corrosion_start_freeze_results_and_save'],
'A10-07':['r10_public_flow_rejects_bad_slots_units_bounds_and_overflow','r11_overlay_public_contract_rejects_unimplemented_or_unbounded_delivery','r12_parry_attachment_and_counter_publication_rejects_ambiguous_bindings'],
'A10-08':['r10_large_repeat_and_control_admission_faults_preserve_state','r10_public_flow_faults_restore_timestamp_and_retry_exactly'],
'A10-09':['r10_public_flow_amount_generation_query_trace_and_cli_export','r12_public_boss_flow_projectile_area_milestones_and_cli_export','d07_f03_c07_d04_f01_empty_dynamic_impact_keeps_record_and_runs_following_resource','d07_f03_c07_piercing_hits_replace_zero_record_but_empty_expiry_preserves_it'],
'A11-01':['r11_public_modifiers_compose_expire_and_preserve_permanent_base','r11_public_modifier_repeated_lifetimes_do_not_drift','r11_public_modifier_faults_restore_source_stats_queue_and_retry'],
'A11-02':['r11_aura_expiry_keeps_already_released_projectile_damage_snapshot','r11_public_channel_aura_membership_damage_identity_and_reinput_stop','r11_aura_overlap_uses_priority_then_creation_and_removes_only_own_source','r11_aura_live_deploy_withdraw_redeploy_and_nonemployee_exclusion','r11_aura_scenario_summon_and_defense_object_never_become_members','r11_aura_normal_tactical_exit_reentry_and_channel_commands_preserve_state','r11_overlay_directional_release_survives_expiry_and_pierces_once_per_enemy'],
'A11-03':['r11_channel_blocks_real_periodic_attacks_and_stop_resumes_them','r11_aura_normal_tactical_exit_reentry_and_channel_commands_preserve_state','r11_channel_excludes_actual_field_work_and_starts_after_stop','r11_channel_silence_damage_preserve_hard_cc_interrupt_end_only_own_lease'],
'A11-04':['r11_channel_silence_damage_preserve_hard_cc_interrupt_end_only_own_lease','r11_channel_actual_enemy_lethal_hit_clears_only_dead_source','r11_channel_rejects_empty_start_and_withdraw_reclaims_membership','r11_world_channel_pause_public_query_terminal_and_save','r11_world_retreat_discards_channel_and_overlay_and_restores_after_outer_failure'],
'A11-05':['r11_hello_reinput_captures_remaining_time_once_without_resonance_or_basic_source','r11_hello_expiry_same_time_stale_activation_and_child_fault_are_atomic','r11_world_exact_fragment_hello_reinput_goodbye_result_and_save'],
'A11-06':['r11_hello_expiry_same_time_stale_activation_and_child_fault_are_atomic','r11_hello_old_activation_cannot_consume_a_new_activation','r11_hello_actual_child_expression_error_restores_live_overlay_and_all_facts'],
'A11-07':['r11_overlay_directional_release_survives_expiry_and_pierces_once_per_enemy','r11_overlay_public_area_once_uses_weapon_and_restores_without_cycle_reset','r11_overlay_compatible_weapons_preserve_type_identity_lifesteal_and_cycle','r12_basic_lifesteal_sums_actual_area_hp_before_rounding_and_one_cap','r12_directional_basic_lifesteal_uses_release_provider_after_overlay_expiry','r12_goodbye_is_not_basic_lifesteal_and_settlement_fault_rolls_back_the_attack'],
'A11-08':['r11_overlay_public_area_once_uses_weapon_and_restores_without_cycle_reset','r11_overlay_expiry_during_windup_uses_restored_weapon_without_consumption','r11_overlay_directional_release_survives_expiry_and_pierces_once_per_enemy'],
'A11-09':['r11_public_modifier_faults_restore_source_stats_queue_and_retry','r11_aura_member_admission_failure_rolls_back_live_deployment_and_retries','r11_world_retreat_discards_channel_and_overlay_and_restores_after_outer_failure','r12_public_sustained_protection_amount_trace_and_cli_export'],
'A12-01':['r12_public_shared_pool600_absorbs400_then200_and_applies_only150_hp_loss','r12_shield_initial_scale_once_membership_recast_and_old_expiry'],
'A12-02':['r12_personal_then_shared_priority_preserves_single_mitigation_and_target_hp','r12_same_timestamp_area_shield_consumption_is_stable_under_spawn_order','r12_world_two_employees_shared_personal_parry_cost_result_and_fresh_restore'],
'A12-03':['r12_shield_initial_scale_once_membership_recast_and_old_expiry','r12_public_shared_pool600_absorbs400_then200_and_applies_only150_hp_loss'],
'A12-04':['r12_heal_and_health_cost_leave_shield_unchanged_only_recharge_changes_balance','r12_health_cost_is_separate_from_damage_heal_shield_and_hp_floor','r12_passive_guard_health_cost_bypasses_protection_and_self_damage_request_rejects'],
'A12-05':['r12_floor_precedes_one_use_death_prevention_then_ordinary_death_is_recorded','r12_health_cost_is_separate_from_damage_heal_shield_and_hp_floor','r12_corrosion_bypasses_enemy_shield_floor_parry_with_live_boss_flow_and_channel','r12_world_corrosion_during_boss_flow_channel_pool_freezes_then_commits_result_and_restore'],
'A12-06':['r12_parry_suppresses_bound_attachment_counters_once_and_preserves_unrelated_effect','r12_parry_basic_on_hit_commands_are_suppressed_before_dispatch','r12_passive_guard_success_and_invulnerability_never_reflect'],
'A12-07':['r12_parry_miss_disables_only_own_reduction_then_recovers_without_rearming','r12_passive_guard_miss_half_open_window_removes_only_own_reduction','r12_passive_guard_at_zero_charge_reduces_reflects_once_per_strike_and_keeps_copy','r12_passive_guard_original_attacker_gone_never_retargets_counter_or_reflection','r12_passive_guard_miss_and_recovery_equal_timestamp_boundaries','r12_world_passive_guard_actual_copy_withdraw_redeploy_and_retreat','r12_passive_guard_health_cost_bypasses_protection_and_self_damage_request_rejects'],
'A12-08':['r12_slow_and_root_change_real_enemy_movement_without_stat_drift','r12_root_keeps_basic_attack_capability_and_taunt_changes_actual_attack_target','r12_pull_uses_valid_unoccupied_cells_and_fault_restores_body_queue_and_ledger','control_status_hard_cc_releases_block_and_silence_preserves_engagement','control_status_overlapping_silence_clears_only_after_last_expiry','control_status_hard_cc_cancels_basic_windup_while_silence_preserves_it'],
'A12-09':['r12_shield_zero_actual_hp_result_expression_cannot_reuse_absorbed_damage','r12_shielded_basic_attack_grants_release_charge_but_no_damage_charge_or_lifesteal','r12_goodbye_is_not_basic_lifesteal_and_settlement_fault_rolls_back_the_attack','r12_world_corrosion_during_boss_flow_channel_pool_freezes_then_commits_result_and_restore'],
'A12-10':['r12_shield_consumption_faults_restore_pool_hp_facts_queue_and_retry','r12_native_protection_limits_are_hard_and_retry_restores_same_result','r12_parry_and_atomic_counter_faults_restore_hp_activation_queue_and_observation','r12_passive_guard_reflection_failure_rolls_back_damage_and_exact_replay','r12_pull_uses_valid_unoccupied_cells_and_fault_restores_body_queue_and_ledger'],
'X01':['r12_public_boss_flow_projectile_area_milestones_and_cli_export','r08_public_rejection_and_target_loss_follow_distinct_routes','r09_corrosion_captures_current_enemies_and_preserves_required_future_wave'],
'X02':['r12_public_sustained_protection_amount_trace_and_cli_export','r11_world_exact_fragment_hello_reinput_goodbye_result_and_save','r11_world_channel_pause_public_query_terminal_and_save','r11_overlay_compatible_weapons_preserve_type_identity_lifesteal_and_cycle'],
'X03':['r12_world_two_employees_shared_personal_parry_cost_result_and_fresh_restore'],
'X04':['r12_world_corrosion_during_boss_flow_channel_pool_freezes_then_commits_result_and_restore','r12_corrosion_bypasses_enemy_shield_floor_parry_with_live_boss_flow_and_channel','r09_public_equipment_death_reactions_are_suppressed_only_for_corrosion','r09_corrosion_captures_current_enemies_and_preserves_required_future_wave'],
'X05':['r07_f07_f08_f09_public_ron_recipe_forms_sources_resources_and_fresh_process','r12_world_two_employees_shared_personal_parry_cost_result_and_fresh_restore','r12_world_corrosion_during_boss_flow_channel_pool_freezes_then_commits_result_and_restore','r09_counter_checkpoint_fresh_process_preserves_remaining_nodes','r09_savepoint_recovery_precedes_counter_expiry'],
'X06':['r11_world_retreat_discards_channel_and_overlay_and_restores_after_outer_failure','r12_world_passive_guard_actual_copy_withdraw_redeploy_and_retreat','r09_corrosion_faults_capture_cost_death_presentation_roll_back_and_retry'],
'X07':['r12_public_sustained_protection_amount_trace_and_cli_export','r10_public_flow_amount_generation_query_trace_and_cli_export','r06_public_dynamic_query_limits_identity_lifecycle_and_trace_independence'],
'X08':['r10_public_flow_rejects_bad_slots_units_bounds_and_overflow','r10_large_repeat_and_control_admission_faults_preserve_state','r11_public_modifier_faults_restore_source_stats_queue_and_retry','r12_native_protection_limits_are_hard_and_retry_restores_same_result','r12_same_timestamp_area_shield_consumption_is_stable_under_spawn_order','r12_parry_and_atomic_counter_faults_restore_hp_activation_queue_and_observation','r11_hello_old_activation_cannot_consume_a_new_activation'],
'X09':[], 'X10':[],
}
d=json.loads((H/'COVERAGE.json').read_text())
for x in d['conditions']:
 if x['id'] in M:x['actual_test_selectors']=[resolve(n) for n in M[x['id']]]
 else:
  for n in x['actual_test_selectors']: assert n in names,n
 if x['id']=='A09-04':x['actual_test_selectors'].append(resolve('r12_corrosion_bypasses_enemy_shield_floor_parry_with_live_boss_flow_and_channel'))
 x['historical_receipts']=x.get('actual_receipts',[])
 x['actual_receipts']=[f'evidence/{n}.json' for n in RECEIPTS]
 x['status']='self_verified';x['reason']='Both final default/test-fixtures executions contain these successful actual selectors; independent R08-R12 audit not performed.'
 if x['id']=='X09':
  x['status']='environment_limited';x['reason']='Both required unfiltered all-target commands reject one Unix socket fixture at OS EPERM. All other tests and mandatory check/release/CLI/PC36/R05 pass with only the separately authorized concurrent peak warning. This is not an unqualified full-suite PASS.'
  x['actual_receipts'] += [f'evidence/{n}.json' for n in ['integrated_all_003','integrated_feature_003','integrated_check_003','integrated_event_check_003','integrated_release_003','integrated_clippy_003','integrated_fmt_003','CLI_FINAL_SUMMARY_002','PC_SUMMARY_005','R05_SUMMARY_005']]
 if x['id']=='X10':
  proof=H/'target/r08_r12_integrated_001/RESTORE_VERIFICATION.json';p=json.loads(proof.read_text());assert p['after_digest']==digest and p['forward_membership_and_bytes_match'] and p['reverse_membership_and_bytes_match']
  x['actual_receipts']=['target/r08_r12_integrated_001/RESTORE_VERIFICATION.json','evidence/VALIDATION_CURRENTNESS.json']
  x['reason']='Actual added/deleted source membership and bytes restored forward/reverse with fuzz0 in an isolated directory; final currentness matched.'
 for receipt in x['actual_receipts']:assert (H/receipt).is_file(),receipt
 for n in x['actual_test_selectors']:assert n in names
assert len(d['conditions'])==55
oldsource=d['source_sha256'];d.update(status='implementation_complete_self_validation_with_environment_limitation_independent_audit_pending',source_sha256=digest,implemented_count=55,verified_count=54,environment_limited_count=1,previous_intermediate_source_sha256=oldsource,independent_audit=False)
(H/'COVERAGE.json').write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n')
# Preserve the predecessor mapping and explicitly rebind V8-named selectors to V9.
old=ROOT/'core_enhanced/docs/goals/content_authoring_fragment_copies/execution/single_goal'
r06=json.loads((old/'R06_IMPACT_COVERAGE.json').read_text());r07=json.loads((old/'COVERAGE.json').read_text())
impact={'source_digest':digest,'prior_audit_reuse':'R06/R07 historical Complete/High applies only to its frozen source. Current shared runtime/schema changes require a new independent impact audit.','R06_acceptances':r06['acceptances'],'R06_integration':copy.deepcopy(r06['integration']),'R07_F01_F12':copy.deepcopy(r07['F01_F12']),'independent_audit':False}
for row in impact['R06_acceptances']:row['status']='same acceptance; current integration selectors rerun; new independent impact audit pending'
for row in impact['R06_integration']+impact['R07_F01_F12']:
 prior=row.get('actual_selectors',[]);row['predecessor_selectors']=prior
 row['actual_selectors']=[n.replace('::current_v8_','::current_v9_') for n in prior]
 for n in row['actual_selectors']:assert n in names,n
 row.update(receipt='evidence/integrated_all_003_environment_excluded.json',feature_receipt='evidence/integrated_feature_003_environment_excluded.json',raw='evidence/integrated_all_003_environment_excluded.log',source_digest=digest,result='selected_oracles_passed_on_current_source_independent_impact_audit_pending',reuse='actual final source execution; explicit V7/V8 to V9 conversion and original-byte preservation retained')
with (H/'R06_R07_IMPACT_COVERAGE.json').open('x') as f:json.dump(impact,f,ensure_ascii=False,indent=2);f.write('\n')
print(json.dumps({'source_digest':digest,'self_verified':54,'environment_limited':1,'R06_integration':len(impact['R06_integration']),'R07_F01_F12':len(impact['R07_F01_F12'])}))
