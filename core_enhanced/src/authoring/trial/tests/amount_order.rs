//! C07 uses the public loader/session/selector; no retained record is fabricated.
use super::*;
use crate::authoring::{AmountBattle, AmountQuery};

#[test]
fn d07_f03_c07_public_replacement_rejects_actual_old_exact_and_retains_zero() {
    let (_fixture, token, mut document) = public_amount_trial_fixture(|fixture| {
        let path = "official/data/skills/base.ron";
        let text = fixture.read(path);
        let begin = text
            .find("id: \"fragment_freischutz_black_round\"")
            .unwrap();
        let end = text[begin..]
            .find("id: \"fragment_funeral_butterfly_eulogy\"")
            .unwrap()
            + begin;
        let mut skill = text[begin..end]
            .replace(
                "delivery: Projectile(speed_units_per_ms: 7000)",
                "delivery: Instant",
            )
            .replace(
                "Damage(amount: 42, damage_type: Magic)",
                "Damage(amount: 1, damage_type: True)",
            );
        let start = skill.find("op: ModifyStats(").unwrap();
        let finish = skill[start..].find(")),],").unwrap() + start + 1;
        skill.replace_range(start..finish, "op: Heal(amount: 0)");
        fixture.write(path, format!("{}{}{}", &text[..begin], skill, &text[end..]));
    });
    let steps = document["steps"].as_array_mut().unwrap();
    // Same original activation; expose its first and second real effect times.
    assert_eq!(steps.pop().unwrap()["clock"]["delta_ms"], 700);
    steps.extend([
        json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":320,"after_seq":0}}),
        json!({"kind":"clock","clock":{"mode":"elapsed","delta_ms":100,"after_seq":0}}),
    ]);
    let cancel = CancellationToken::default();
    let mut session = TrialSession::prepare(
        fixture_content(&token),
        checked(&document),
        TrialLimits {
            amount_observation: true,
            ..Default::default()
        },
        &cancel,
    )
    .unwrap();
    while session.next_step_index() < 27 {
        session.advance(&cancel).unwrap();
    }
    let latest = AmountQuery::Latest {
        battle: AmountBattle::Active,
    };
    let query_pair = |session: &TrialSession, query| {
        let receipts = session
            .receipt_documents()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let before = (session.accepted_work(), session.next_step_index());
        let limits = ExecutionLimits::default();
        let p = session.preview_amount(query, &limits, &cancel).unwrap();
        let e = session.explain_amount(query, &limits, &cancel).unwrap();
        assert_eq!(
            p.document(),
            session
                .preview_amount(query, &limits, &cancel)
                .unwrap()
                .document()
        );
        assert_eq!(
            e.document(),
            session
                .explain_amount(query, &limits, &cancel)
                .unwrap()
                .document()
        );
        assert_eq!(before, (session.accepted_work(), session.next_step_index()));
        assert_eq!(receipts, session.receipt_documents().collect::<Vec<_>>());
        let p: Value = serde_json::from_str(p.document()).unwrap();
        let e: Value = serde_json::from_str(e.document()).unwrap();
        assert_eq!(p["status"], e["status"]);
        assert_eq!(p["identity"], e["identity"]);
        assert_eq!(p["trace"], e["trace"]);
        assert_eq!(p["narrowed_base_amount"], e["narrowed_base_amount"]);
        eprintln!("C07_QUERY {}", json!({"preview":p,"explain":e}));
        (p, e)
    };
    let before = query_pair(&session, latest).0;
    assert_eq!(before["reason"], "not_yet_evaluated");
    session.advance(&cancel).unwrap();
    let a = query_pair(&session, latest).0;
    assert_eq!(a["status"], "evaluated", "{a}");
    assert_eq!(a["identity"]["evaluation_ordinal"], 1);
    assert_eq!(a["site"]["action"]["action_id"], "fragment_black_round");
    assert_eq!(a["typed_base_amount"]["value"], 1);
    let key = session
        .observation()
        .unwrap()
        .active()
        .unwrap()
        .key()
        .unwrap();
    let exact_a = AmountQuery::Exact {
        battle_key: key,
        evaluation_ordinal: 1,
    };
    assert_eq!(query_pair(&session, exact_a).0["identity"], a["identity"]);
    session.advance(&cancel).unwrap();
    assert!(session.is_complete());
    let b = query_pair(&session, latest).0;
    assert_eq!(b["status"], "evaluated", "{b}");
    assert_eq!(b["identity"]["evaluation_ordinal"], 2);
    assert_eq!(b["identity"]["battle_key"], a["identity"]["battle_key"]);
    assert_eq!(b["site"]["action"]["action_id"], "fragment_smoke_focus");
    assert_eq!(b["typed_base_amount"]["value"], 0);
    assert_eq!(
        b["consume_time_ms"].as_u64().unwrap(),
        a["consume_time_ms"].as_u64().unwrap() + 100
    );
    assert_eq!(b["amount_kind"], "heal");
    let old = query_pair(&session, exact_a);
    for doc in [old.0, old.1] {
        assert_eq!(doc["status"], "unavailable");
        assert_eq!(doc["reason"], "not_retained");
        assert!(doc.get("identity").is_none());
        assert!(doc.get("narrowed_base_amount").is_none());
        assert_eq!(doc["selector"]["evaluation_ordinal"], 1);
    }
    let exact_b = AmountQuery::Exact {
        battle_key: key,
        evaluation_ordinal: 2,
    };
    assert_eq!(query_pair(&session, exact_b).0["identity"], b["identity"]);
    assert_eq!(query_pair(&session, latest).0, b);
}
