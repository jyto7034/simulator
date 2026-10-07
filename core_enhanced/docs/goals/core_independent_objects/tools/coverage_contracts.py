"""U07 exact policy evidence selectors; no changed acceptance or numeric oracle."""
CONTRACTS={
'V01':(['u07_object_actor_and_wall_adapters_follow_actual_side','u07_object_terminal_finalize','r08_'],[]),
'V02':(['u07_object_creation_batch_replay','native_u07_object_late_retained','native_u07_object_evolution_late_queue','native_u07_segmented_late_spawn'],[]),
'V03':(['u07_object_every_offset','u07_object_actor_path_is_checked','u07_object_owner_counter','u07_object_evolution_replay','u07_object_creation_batch_replay'],[]),
'V04':(['u07_object_actor_basic_attack','u07_object_actor_auto_cast','u07_object_path_actor_moves','u07_object_actor_and_wall_adapters','u07_public_external_ron_trial'],[]),
'V05':(['withdraw_preserves_ground_fixed_area_and_ends_follow_areas','resolved_area_tick_rebuilds_live_members','control_status_hard_cc_preserves_released_area_runtime','relocation_preserves_ground_fixed_and_reanchors_follow_areas','native_action_area_observes'],[]),
'V06':(['u07_official_small_hits_one','u07_official_base_second_cast','u07_object_trigger_without_damage_target','u07_object_trap_attacks_after_source_removal','u07_object_same_time_traps','u07_public_external_ron_trial'],[]),
'V07':(['u07_object_evolution_public_two_casts','u07_object_evolution_replay','u07_object_evolution_hp_forms','native_u07_object_evolution_late_queue','u07_official_base_second_cast'],[]),
'V08':(['u07_object_source_removal_policy','u07_object_cleanup_only_cascades','u07_object_public_tile_cast_rejects','u07_object_trap_attacks_after_source_removal','u07_object_origin_releases_parent'],[]),
'V09':(['u07_object_same_time_traps','u07_object_owner_counter','u07_object_terminal_finalize','u07_object_max_hp_zero','u07_object_creation_batch_replay'],[]),
'V10':(['u07_segmented_'],[]),
'V11':(['u07_segmented_public_fallback','u07_segmented_fallback_health_cost','native_u07_segmented_late_spawn','u07_official_meat_wall_uses_exact'],[]),
'V12':(['u07_object_wall_stops_actual_segment','u07_object_hp_zero_retires','u07_official_wall_preview','u07_public_external_ron_trial'],['u07_host_wall_cast']),
'V13':(['u07_object_max_hp_zero','u07_object_evolution_hp_forms','runtime_max_health_modifier_decision_table','runtime_max_health_modifier_without_a_clamp','sourced_lethal_max_health_modifier','r12_'],[]),
'V14':(['u07_segmented_public_fallback','u07_object_public_tile_cast_rejects','control_status_hard_cc_cancels_manual_focus','withdraw_cancels_queued_unreleased_skill_step'],['u07_host_wall_cast']),
'V15':(['u07_object_public_tile_cast_rejects','u07_segmented_public_producer','u07_official_wall_preview','u07_official_meat_wall_uses_exact','u07_public_external_ron_trial'],['u07_host_wall_cast']),
'V16':(['u07_object_same_time_traps','u07_object_terminal_finalize','u07_object_cleanup_only_cascades','owned_clone_shape_queue_all_variants'],[]),
'V17':(['u07_objects::','u07_public_external_ron_trial','u07_object_evolution_public_two_casts','u07_segmented_ordinary_shield'],['u07_host_wall_cast']),
'V18':(['u07_object_schema','u07_object_positive_limits','u07_object_evolution_rejects','u07_object_opcode_strict','u07_segmented_strict_schema','u07_object_zero_trap_target','u07_official_','u07_public_external_ron_trial'],[]),
'V19':(['native_','u07_object_compiled_clone','u07_object_owner_counter'],[]),
'V20':([],[]),
}
DIMENSIONS={
'success':['V01','V04','V05','V06','V07','V10','V11','V12','V15','V17','V18'],
'invalid_input':['V02','V03','V07','V10','V11','V14','V18'],
'failure_rollback':['V02','V07','V10','V11','V14','V17','V19'],
'retry_idempotency':['V02','V07','V09','V14'],
'ordering_concurrency':['V06','V09','V12','V13','V16'],
'boundary_empty_max':['V03','V06','V10','V11','V13','V19'],
'disconnect_restart_persistence':['V08','V09','V17'],
'live_data':['V01','V04','V05','V06','V12','V18'],
'producer_consumer':['V02','V10','V11','V15','V17','V18','V19'],
'legacy_fallback_absence':['V01','V04','V08','V10','V12','V13','V18','V19'],
}

# F01 adds current-source lifecycle/rollback/accounting evidence; original
# acceptance wording and all predecessor selectors remain unchanged.
for key, fragments in {
    'V02': ['u07_f01_evolution_clone_late_queue', 'u07_f01_actor_own_ground_contact'],
    'V07': ['u07_f01_actor_evolution', 'u07_f01_wall_evolution', 'u07_f01_actor_own_ground_contact'],
    'V09': ['u07_f01_actor_expiry', 'u07_f01_wall_expiry', 'u07_f01_actor_source_despawn', 'u07_f01_wall_source_despawn'],
    'V12': ['u07_f01_expired_own_wall', 'u07_f01_actor_evolution', 'u07_f01_wall_evolution'],
    'V16': ['u07_f01_'],
    'V19': ['u07_f01_refresh_'],
}.items():
    CONTRACTS[key][0].extend(fragments)
