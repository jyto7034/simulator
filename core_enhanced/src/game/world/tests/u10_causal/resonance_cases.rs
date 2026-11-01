use super::*;

#[test]
fn u10_e_queued_and_periodic_positive_resonance_use_the_existing_gate_without_banking() {
    let skills = skill_input(&[(0, &[1]), (125, &[1])]);
    let needle = r#"id:"damage_0_0",tags:["u10_direct","u10_multi_hit"]"#;
    assert!(skills.contains(needle));
    let skills = skills.replacen(needle, r#"id:"damage_0_0",tags:["u10_first"]"#, 1);
    let rules = r#"
        (id:"derived_resonance",apply:Subscribe((event:DamageHpLost(perspective:Dealt),
            source:(all_tags:["u10_first"]),payload:Program((steps:[
                (id:"delayed_gain_and_cost",delay_ms:30,target:Owner,effects:[
                    (id:"gain",op:ModifyResonance(amount:50)),
                    (id:"cost",op:ModifyResonance(amount:-10))
                ])
            ]))))),
        (id:"periodic_resonance",apply:Periodic(interval_ms:50,payload:Program((steps:[
            (id:"gain",target:Owner,effects:[(id:"gain",op:ModifyResonance(amount:50))])
        ]))))
    "#;
    let mut fixture = CausalWorld::prepare(&skills, &relic_input(rules), &["u10_relic"], true);
    fixture.cast();
    fixture.advance(1);
    // Explicit native preparation makes a negative change observable during
    // an already established real use lock. It does not perform the tested
    // gain/cost or create the lock; both payloads run through normal consumers.
    fixture
        .core
        .state
        .active_battle
        .as_mut()
        .unwrap()
        .battle
        .units
        .get_mut(&fixture.actor)
        .unwrap()
        .resonance_current = 30;
    let resonance = |fixture: &CausalWorld| {
        fixture
            .core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .units[&fixture.actor]
            .resonance_current
    };
    fixture.advance(30);
    assert_eq!(
        resonance(&fixture),
        20,
        "queued positive 50 is blocked; negative 10 remains allowed"
    );
    fixture.advance(20);
    assert_eq!(
        resonance(&fixture),
        20,
        "first independent periodic positive is also blocked"
    );
    fixture.advance(50);
    assert_eq!(
        resonance(&fixture),
        20,
        "both periodic occurrences at50 and100 are inside the original use"
    );
    fixture.advance(25);
    assert_eq!(
        resonance(&fixture),
        20,
        "ending the original use must not bank earlier blocked gains"
    );
    fixture.advance(25);
    assert_eq!(
        resonance(&fixture),
        70,
        "the next unlocked positive is accepted normally"
    );
    fixture.advance(50);
    assert_eq!(
        resonance(&fixture),
        100,
        "the existing cap discards only excess from accepted gain"
    );
}
