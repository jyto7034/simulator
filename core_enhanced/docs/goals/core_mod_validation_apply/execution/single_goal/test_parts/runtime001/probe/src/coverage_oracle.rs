//! Assertions over actual producer reports. A declaration, total HP delta, or
//! final latest-only snapshot cannot stand in for the requested site witness.
use game_core::mod_validation::{CoverageReport, CoverageStage};
use std::{collections::BTreeSet, error::Error, io};

pub struct ExpectedWitness<'a> {
    pub kind: &'a str,
    pub definition: &'a str,
    pub action: &'a str,
    pub effect: &'a str,
    pub family: &'a str,
    pub stage: CoverageStage,
    pub minimum: u64,
    pub maximum: Option<u64>,
}
fn invalid(cause: impl Into<String>) -> Box<dyn Error> {
    io::Error::new(io::ErrorKind::InvalidData,cause.into()).into()
}
fn index(stage: CoverageStage)->usize {
    // Explicit public wire ordering, kept separate from enum discriminant casts.
    match stage {
        CoverageStage::CatalogValidated=>0,
        CoverageStage::Triggered=>1,
        CoverageStage::Ineligible=>2,
        CoverageStage::LegalNoop=>3,
        CoverageStage::ConsumedOrEffect=>4,
        CoverageStage::TerminatedOrReclaimed=>5,
    }
}

/// Call after preserving the full raw report. `expected` must be authored from
/// fixture semantics, never derived by accepting whatever keys the report has.
pub fn require_witnesses(report:&CoverageReport,generation:&str,expected:&[ExpectedWitness<'_>])
    ->Result<(),Box<dyn Error>> {
    if report.overflow { return Err(invalid("producer report overflowed")); }
    if !report.unsupported.is_empty() {return Err(invalid("producer report contains unsupported required families"));}
    if report.rows.len()>65_536 {return Err(invalid("independent oracle finite row bound exceeded"));}
    let mut unique=BTreeSet::new();
    for row in &report.rows {
        let key=&row.requirement.key;
        if key.generation!=generation {return Err(invalid("producer report belongs to another generation"));}
        if !unique.insert((key.clone(),index(row.requirement.required_stage))) {
            return Err(invalid("duplicate required key/stage in producer report"));
        }
        if row.counts[index(row.requirement.required_stage)]==0 {
            return Err(invalid(format!("unfulfilled required site {}:{}:{}/{}",key.definition_kind,
                key.definition_id,key.action_id,key.effect_id)));
        }
    }
    for witness in expected {
        // A zero assertion is useful only as an explicit exact absence check,
        // paired by the caller with a positive witness at the same source site.
        if (witness.minimum==0 && witness.maximum!=Some(0))
            || witness.maximum.is_some_and(|max|max<witness.minimum) {
            return Err(invalid("independent witness has invalid expected count bounds"));
        }
        // More than one requirement stage may refer to the same producer key.
        // They must repeat the same counters rather than inflate an event sum.
        let mut counts=None;
        for row in &report.rows {
            let key=&row.requirement.key;
            if key.definition_kind==witness.kind && key.definition_id==witness.definition
                && key.action_id==witness.action && key.effect_id==witness.effect
                && key.producer_family==witness.family {
                if counts.is_some_and(|prior|prior!=row.counts) {
                    return Err(invalid("one producer key has inconsistent stage counters"));
                }
                counts=Some(row.counts);
            }
        }
        let counts=counts.ok_or_else(||invalid(format!("missing independent site {}:{}:{}/{} [{}]",
            witness.kind,witness.definition,witness.action,witness.effect,witness.family)))?;
        let actual=counts[index(witness.stage)];
        if actual<witness.minimum || witness.maximum.is_some_and(|max|actual>max) {
            return Err(invalid(format!("site witness count out of bounds {}:{}:{}/{}: observed {}",
                witness.kind,witness.definition,witness.action,witness.effect,actual)));
        }
    }
    Ok(())
}
