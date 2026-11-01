fn canonical_fixture(
    equipped: &[usize],
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    use crate::authoring::composition::canonical_greed_tests as canonical;
    let (mut c, s, a, e, b) = crate::game::battle::core::boss_runtime::tests::compiled_action_fixture::fixture_content_with_weapon(
        &canonical::RELICS, equipped, canonical::fixture(), |_| {},
    );
    c.units.get_mut(&b).unwrap().fragment_source = None;
    c.units.get_mut(&b).unwrap().skill_id = None;
    Arc::make_mut(&mut c.scenario).terminal_outcomes =
        crate::game::battle::scenario::BattleScenario::fixture_survival_outcomes(120000);
    for id in [a, e, b] {
        let u = c.units.get_mut(&id).unwrap();
        u.stats.max_health = 10000;
        u.stats.current_health = 10000;
        u.stats.defense = 0;
    }
    c.runtime_transaction(|c| {
        c.clear_prepared_volleys(0)?;
        c.sustained.volleys = Default::default();
        c.reconcile_prepared_volleys(0)
    })
    .unwrap();
    (c, s, a, e, b)
}

#[test]
fn canonical_greed_finite_source_serial_impact_and_observation_parity() {
    let (mut c, mut s, a, e, _) = canonical_fixture(&[]);
    let copy = primary_fragment_input(&c, a);
    let mut off = controlled_clone(&c).unwrap();
    let mut off_s = s.clone();
    let (_trace, from) = enable_complete_trace(&mut c);
    cast(&mut c, a, 0);
    step(&mut c, &mut s, 1750);
    assert_eq!(c.sustained.volleys.owners[&copy].balance, 8);
    release(&mut c, a, 1750);
    assert_eq!(c.sustained.volleys.flights.len(), 1);
    step(&mut c, &mut s, 3000);
    let v = c.prepared_volleys_snapshot().unwrap();
    assert_eq!(v.settled.len(), 2);
    assert_eq!(v.settled[1].released_at_ms, v.settled[0].ended_at_ms);
    assert_eq!(c.units[&e].stats.current_health, 9800);
    assert!(c.sustained.volleys.owners[&copy].active.is_none());
    let rows = hp_trace_rows(&c, from);
    assert_eq!(rows.len(), 2);
    for row in &rows {
        assert_fragment_trace_origin(row, a, Some(("r11_aura", 0)));
    }
    cast(&mut off, a, 0);
    step(&mut off, &mut off_s, 1750);
    release(&mut off, a, 1750);
    step(&mut off, &mut off_s, 3000);
    assert_eq!(
        serde_json::to_value(c.runtime_fact_snapshot().unwrap()).unwrap(),
        serde_json::to_value(off.runtime_fact_snapshot().unwrap()).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&c.event_log.entries).unwrap(),
        serde_json::to_value(&off.event_log.entries).unwrap()
    );
    assert_eq!(
        c.prepared_volleys_snapshot().unwrap(),
        off.prepared_volleys_snapshot().unwrap()
    );
}

#[test]
fn canonical_greed_continuous_preserves_debt_and_cost_stops_on_manual_end() {
    let (mut c, mut s, a, _, _) = canonical_fixture(&[2, 6]);
    let copy = primary_fragment_input(&c, a);
    let mut ledgers = CreditLedgers::default();
    ledgers.entries.insert(
        copy,
        CreditCarry {
            definition: "6fcdba2d-8201-4735-901f-83ec7e231cba".into(),
            unpaid: 20,
            borrowed: true,
        },
    );
    c.mount_credit_ledgers(&ledgers).unwrap();
    cast(&mut c, a, 0);
    step(&mut c, &mut s, 999);
    assert_eq!(c.units[&a].stats.current_health, 10000);
    step(&mut c, &mut s, 1000);
    assert_eq!(c.units[&a].stats.current_health, 9500);
    step(&mut c, &mut s, 2000);
    assert_eq!(c.units[&a].stats.current_health, 9000);
    let owner = &c.sustained.volleys.owners[&copy];
    assert_eq!(owner.balance, -10);
    assert!(!owner.borrowed && !owner.can_borrow);
    assert!(owner.active.is_some());
    assert!(c.sustained.volleys.flights.len() <= 1);
    cast(&mut c, a, 2000);
    assert!(c.sustained.volleys.owners[&copy].active.is_none());
    step(&mut c, &mut s, 4000);
    assert_eq!(c.units[&a].stats.current_health, 9000);
    assert_eq!(c.sustained.volleys.owners[&copy].balance, -10);
    assert!(c.sustained.volleys.flights.is_empty());
    assert!(!c.prepared_volleys_snapshot().unwrap().settled.is_empty());
}
