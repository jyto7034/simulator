//! Exact arithmetic checks of the actual product comparator, supplementary to
//! measured normal/expanded/pathological runs and a reviewed criterion.
use game_core::mod_validation::{CriterionAssessment,LabCriterion,LabMetrics};
use serde_json::{json,Value};
use std::{error::Error,fs::{self,OpenOptions},io::Write,path::Path};

fn save(path:&Path,value:&Value)->Result<(),Box<dyn Error>> {
    let mut bytes=serde_json::to_vec_pretty(value)?;bytes.push(b'\n');
    OpenOptions::new().write(true).create_new(true).open(path)?.write_all(&bytes)?;Ok(())
}
fn observe(criterion:&LabCriterion,metrics:&LabMetrics,out:&Path,label:&str)
    ->Result<CriterionAssessment,Box<dyn Error>> {
    let assessment=criterion.assess(metrics);
    save(&out.join(format!("{label}.json")),&json!({"synthetic_comparator_input":true,
        "authority_claimed":false,"criterion":criterion,"metrics":metrics,"assessment":assessment}))?;
    Ok(assessment)
}

/// Supply the criterion returned by the actual calibration/adoption boundary
/// and the successful real supervised metrics from that exact profile. This
/// function does not build a criterion from fixture constants or issue tokens.
pub fn verify_boundaries(criterion:&LabCriterion,measured:&LabMetrics,out:&Path)
    ->Result<(),Box<dyn Error>> {
    fs::create_dir(out)?;
    save(&out.join("measured_starting_point.json"),&json!({"synthetic_comparator_input":false,
        "criterion":criterion,"metrics":measured,"assessment":criterion.assess(measured)}))?;
    assert!(measured.is_complete(),"actual measurements must be complete before boundary arithmetic");
    assert_eq!(criterion.assess(measured),CriterionAssessment::Passed,
        "normal real run must pass the adopted criterion before supplementary synthetic checks");
    let serialized=serde_json::to_value(criterion)?;
    let threshold=|name:&str|->Result<u64,Box<dyn Error>>{
        serialized[name].as_u64().ok_or_else(||format!("reviewed criterion has no numeric {name}").into())
    };
    let elapsed=threshold("max_elapsed_ns")?;let heap=threshold("max_requested_heap_bytes")?;
    let rss=threshold("max_rss_bytes")?;let native=threshold("max_native_work")?;
    // A u64::MAX threshold has no representable one-over input; do not silently
    // skip the required boundary and report a successful comparator suite.
    for bound in [elapsed,heap,rss,native] {assert!(bound>0&&bound<u64::MAX,"finite one-over boundary required");}
    let decoded=serde_json::from_value::<LabCriterion>(serialized.clone());
    save(&out.join("public_criterion_roundtrip_denied.json"),&json!({
        "decoded":decoded.is_ok(),"error":decoded.err().map(|e|e.to_string()),"authority_claimed":false}))?;
    assert!(serde_json::from_value::<LabCriterion>(serialized).is_err(),"public criterion metadata is not an adoption capability");
    for field in ["revision","calibration_sha256","max_elapsed_ns","max_requested_heap_bytes","max_rss_bytes","max_native_work"] {
        let mut mutated=criterion.clone();
        match field {
            "revision"=>mutated.revision=if mutated.revision=="changed"{"other".into()}else{"changed".into()},
            "calibration_sha256"=>mutated.calibration_sha256=if mutated.calibration_sha256=="00".repeat(32){"11".repeat(32)}else{"00".repeat(32)},
            "max_elapsed_ns"=>mutated.max_elapsed_ns+=1,"max_requested_heap_bytes"=>mutated.max_requested_heap_bytes+=1,
            "max_rss_bytes"=>mutated.max_rss_bytes+=1,"max_native_work"=>mutated.max_native_work+=1,_=>unreachable!(),
        }
        assert!(matches!(observe(&mutated,measured,out,&format!("criterion_mutation_{field}"))?,CriterionAssessment::Incomplete{..}),
            "{field}: public field mutation must invalidate actual calibration seal");
    }
    let mut exact=measured.clone();
    for sample in exact.plain.iter_mut().chain(&mut exact.instrumented) {
        sample.elapsed_ns=elapsed;sample.native_work=native;
    }
    for sample in &mut exact.instrumented {
        sample.requested_heap_live_bytes=Some(heap);sample.requested_heap_peak_bytes=Some(heap);
        sample.rss_bytes=Some(rss);sample.rss_lifetime_peak_bytes=Some(rss);
        sample.native_accounting.as_mut().expect("actual complete native summary")["work"]=json!(native);
    }
    assert_eq!(observe(criterion,&exact,out,"all_exact")?,CriterionAssessment::Passed);
    for instrumented in [false,true] {
        for metric in ["elapsed_ns","native_work"] {
            let mut over=exact.clone();
            let sample=if instrumented {over.instrumented.last_mut().unwrap()}else{over.plain.last_mut().unwrap()};
            let limit=if metric=="elapsed_ns" {sample.elapsed_ns+=1;elapsed}else{sample.native_work+=1;native};
            if instrumented&&metric=="native_work" {
                sample.native_accounting.as_mut().unwrap()["work"]=json!(sample.native_work);
            }
            let label=format!("{}_{}_one_over",if instrumented{"instrumented"}else{"plain"},metric);
            assert_eq!(observe(criterion,&over,out,&label)?,CriterionAssessment::Exceeded{metric:metric.into(),actual:limit+1,limit});
        }
    }
    for (metric,limit) in [("requested_heap_bytes",heap),("rss_bytes",rss)] {
        let mut over=exact.clone();let sample=over.instrumented.last_mut().unwrap();
        if metric=="requested_heap_bytes" {sample.requested_heap_peak_bytes=Some(limit+1)}else{sample.rss_bytes=Some(limit+1)};
        assert_eq!(observe(criterion,&over,out,&format!("{metric}_one_over"))?,
            CriterionAssessment::Exceeded{metric:metric.into(),actual:limit+1,limit});
    }
    let mut over=exact.clone();over.instrumented.last_mut().unwrap().rss_lifetime_peak_bytes=Some(rss+1);
    assert_eq!(observe(criterion,&over,out,"rss_process_lifetime_high_water_one_over")?,
        CriterionAssessment::Exceeded{metric:"rss_bytes".into(),actual:rss+1,limit:rss});
    // A public boolean or an exceeded metric must not turn incomplete
    // measurements into a pass or a meaningful performance failure.
    let mut incomplete_cases=Vec::new();
    let mut value=exact.clone();value.complete=false;incomplete_cases.push(("complete_false",value));
    let mut value=exact.clone();value.pair=None;incomplete_cases.push(("missing_pair",value));
    let mut value=exact.clone();value.repeat_count=0;incomplete_cases.push(("zero_repeat",value));
    let mut value=exact.clone();value.instrumented.pop();incomplete_cases.push(("missing_drop_phase",value));
    let mut value=exact.clone();value.instrumented.last_mut().unwrap().phase="execute".into();incomplete_cases.push(("duplicate_phase",value));
    let mut value=exact.clone();value.instrumented.last_mut().unwrap().repeat_index+=1;incomplete_cases.push(("wrong_repeat",value));
    let mut value=exact.clone();value.instrumented.last_mut().unwrap().sample_count=0;incomplete_cases.push(("missing_samples",value));
    let mut value=exact.clone();value.instrumented.last_mut().unwrap().dropped_samples=1;incomplete_cases.push(("dropped_sample",value));
    let mut value=exact.clone();value.instrumented.last_mut().unwrap().missing_samples=1;incomplete_cases.push(("incomplete_sample_window",value));
    let mut value=exact.clone();value.instrumented.last_mut().unwrap().counter_overflowed=true;incomplete_cases.push(("counter_overflow",value));
    let mut value=exact.clone();value.instrumented.last_mut().unwrap().window_uncertain=true;incomplete_cases.push(("uncertain_window",value));
    for field in ["heap_live","heap_peak","rss","rss_lifetime_peak"] {
        let mut value=exact.clone();let sample=value.instrumented.last_mut().unwrap();
        match field {"heap_live"=>sample.requested_heap_live_bytes=None,"heap_peak"=>sample.requested_heap_peak_bytes=None,
            "rss"=>sample.rss_bytes=None,_=>sample.rss_lifetime_peak_bytes=None}
        incomplete_cases.push((field,value));
    }
    let mut value=exact.clone();value.instrumented.last_mut().unwrap().native_accounting=None;
    incomplete_cases.push(("missing_native_accounting",value));
    let mut value=exact.clone();value.instrumented.last_mut().unwrap().native_accounting=Some(Value::Null);
    incomplete_cases.push(("null_native_accounting",value));
    for field in ["work","clone_slots","admitted_bytes","sequence"] {
        let mut value=exact.clone();let sample=value.instrumented.last_mut().unwrap();
        sample.native_accounting.as_mut().unwrap().as_object_mut().unwrap().remove(field);
        incomplete_cases.push((field,value));
    }
    let mut value=exact.clone();value.instrumented.last_mut().unwrap().native_accounting.as_mut().unwrap()["work"]=json!(native+1);
    incomplete_cases.push(("native_work_mismatch",value));
    let mut value=exact.clone();value.instrumented.last_mut().unwrap().native_accounting.as_mut().unwrap()["sequence"]=json!(2);
    incomplete_cases.push(("native_window_sequence",value));
    for (label,mut value) in incomplete_cases {
        value.plain[0].elapsed_ns=elapsed+1;
        assert!(matches!(observe(criterion,&value,out,label)?,CriterionAssessment::Incomplete{..}),
            "{label}: incompleteness must take precedence over an exceeded threshold");
    }
    save(&out.join("RESULT.json"),&json!({"passed":true,"scope":"actual comparator arithmetic and completeness",
        "supplements_actual_measurements":true,"authority_claimed":false,
        "actual_exact_wall_time_claimed":false,"metrics":["elapsed_ns","native_work","requested_heap_bytes","rss_bytes"]}))?;
    Ok(())
}
