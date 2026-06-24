use super::*;

fn core_with_starter_employee_ids() -> (GameCore, Vec<Uuid>) {
    let mut core = GameCore::new(empty_game_data(), 123);
    start_new_game_with_default_starters(&mut core, Uuid::from_u128(1));
    let employee_ids = core.roster().unwrap().available_employee_ids();
    (core, employee_ids)
}

#[test]
fn move_roster_unit_reorders_and_swaps_roster_slots() {
    let (mut core, employee_ids) = core_with_starter_employee_ids();
    let left_uuid = employee_ids[0];
    let right_uuid = employee_ids[1];

    let result = core
        .execute(
            Uuid::from_u128(1),
            PlayerBehavior::MoveRosterUnit {
                target_unit_uuid: left_uuid,
                dest_slot: 1,
                swap_with_unit_uuid: Some(right_uuid),
            },
        )
        .unwrap();

    let BehaviorResult::MoveRosterUnit { roster_slots } = result else {
        panic!("expected roster order move result");
    };
    assert_eq!(
        roster_slots
            .iter()
            .find(|slot| slot.slot == 1)
            .and_then(|slot| slot.unit_uuid),
        Some(left_uuid)
    );
    let roster_order = core.roster_order().unwrap();
    assert_eq!(roster_order.slot_of(left_uuid), Some(1));
    assert_eq!(roster_order.slot_of(right_uuid), Some(0));
}
