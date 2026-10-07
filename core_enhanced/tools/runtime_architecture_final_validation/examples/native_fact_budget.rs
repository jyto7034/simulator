//! Exact native-budget characterization on the unchanged kernel fixture.
//! This does not replace any assertion in final_performance.
#[allow(dead_code)]
mod kernel {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/final_performance.rs"));

    use game_core::game::{
        battle::runtime_control::{RuntimeFactLimits, RuntimeFaultCode},
        behavior::GameError,
    };

    fn sample(data: ValidatedGameData, limits: RuntimeFactLimits) -> serde_json::Value {
        let mut core = event_heavy_core(data, 0xA11C_E000)
            .with_runtime_fact_limits(limits).unwrap();
        let mut execution = None;
        let result = (|| {
            execution = Some(core.start_battle_execution()?);
            core.step_battle_execution_by(execution.as_mut().unwrap(), EVENT_HEAVY_DURATION_MS)?;
            Ok::<_, GameError>(())
        })();
        let fault = match result {
            Ok(()) => None,
            Err(GameError::BattleRuntimeFault(fault)) => Some(fault),
            Err(other) => panic!("unexpected native-budget result: {other:?}"),
        };
        serde_json::json!({
            "limits": limits,
            "passed": fault.is_none(),
            "fault": fault,
            "clock_ms": execution.as_ref().map(|state| state.last_event_time_ms()),
            "log_sha256": sha256(&serde_json::to_vec(&core.event_log).unwrap()),
            "facts": serde_json::to_value(core.runtime_fact_view()).unwrap(),
        })
    }

    pub fn characterize() -> serde_json::Value {
        let (data, _) = preservation_data();
        let baseline = sample(data.clone(), RuntimeFactLimits::default());
        assert_eq!(baseline["passed"], true);
        let mut cases = Vec::new();
        let mut boundaries = Vec::new();
        for (name, ceiling, code) in [
            ("clone_slots", RuntimeFactLimits::default().max_clone_slots as u64, RuntimeFaultCode::CloneLimit),
            ("work", RuntimeFactLimits::default().max_work, RuntimeFaultCode::WorkLimit),
            ("retained_bytes", RuntimeFactLimits::default().max_retained_bytes as u64, RuntimeFaultCode::RetainedByteLimit),
        ] {
            let limits_for = |value: u64| {
                let mut limits = RuntimeFactLimits::default();
                match name {
                    "clone_slots" => limits.max_clone_slots = value as usize,
                    "work" => limits.max_work = value,
                    "retained_bytes" => limits.max_retained_bytes = value as usize,
                    _ => unreachable!(),
                }
                limits
            };
            let mut low = 1;
            let mut high = ceiling;
            while low < high {
                let middle = low + (high - low) / 2;
                let result = sample(data.clone(), limits_for(middle));
                if result["passed"] == true {
                    assert_eq!(result["log_sha256"], baseline["log_sha256"]);
                    assert_eq!(result["facts"], baseline["facts"]);
                    high = middle;
                } else {
                    assert_eq!(result["fault"]["code"], serde_json::to_value(code).unwrap());
                    low = middle + 1;
                }
                cases.push(result);
            }
            assert!(low > 1);
            let exact = sample(data.clone(), limits_for(low));
            assert_eq!(exact["passed"], true);
            assert_eq!(exact["log_sha256"], baseline["log_sha256"]);
            assert_eq!(exact["facts"], baseline["facts"]);
            let one_below = sample(data.clone(), limits_for(low - 1));
            assert_eq!(one_below["passed"], false);
            assert_eq!(one_below["fault"]["code"], serde_json::to_value(code).unwrap());
            boundaries.push(serde_json::json!({"dimension": name, "minimum": low,
                "exact": exact, "one_below": one_below}));
        }
        serde_json::json!({
            "scope": "same event-heavy core, seed, data, five-second public step; separate native scopes for startup and step",
            "kernel_source_sha256": sha256(&fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/src/final_performance.rs")).unwrap()),
            "baseline": baseline, "boundaries": boundaries, "search_cases": cases,
            "passed": true,
        })
    }
}

fn main() {
    let output = std::env::args().nth(1).expect("native_fact_budget <output.json>");
    let receipt = kernel::characterize();
    std::fs::write(output, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
    println!("{}", serde_json::to_string(&receipt["boundaries"]).unwrap());
}
