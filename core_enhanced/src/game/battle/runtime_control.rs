//! Synchronous native operation fuel. This is separate from authoring trials
//! and lives outside gameplay candidates, so rollback cannot replenish fuel.
use std::cell::{Cell, RefCell};

use serde::Serialize;

use crate::game::behavior::GameError;
pub use crate::game::resource::store::ResourceFaultCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeFaultCode {
    Resource(ResourceFaultCode),
    InvalidLimits,
    WorkLimit,
    CloneLimit,
    ActiveActionLimit,
    RetainedResultLimit,
    DeliveryLimit,
    ActorLimit,
    ProviderLimit,
    ControlStepLimit,
    StrikeSegmentLimit,
    PredeathBuffLimit,
    RetainedByteLimit,
    CounterOverflow,
    Invariant,
    HealthCostUnavailable,
    ShieldUnavailable,
    InvalidControlTarget,
}

impl RuntimeFaultCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Resource(code) => code.as_str(),
            Self::InvalidLimits => "battle_runtime_invalid_limits",
            Self::WorkLimit => "battle_runtime_work_limit",
            Self::CloneLimit => "battle_runtime_clone_limit",
            Self::ActiveActionLimit => "battle_runtime_active_action_limit",
            Self::RetainedResultLimit => "battle_runtime_retained_result_limit",
            Self::DeliveryLimit => "battle_runtime_delivery_limit",
            Self::ActorLimit => "battle_runtime_actor_limit",
            Self::ProviderLimit => "battle_runtime_provider_limit",
            Self::ControlStepLimit => "battle_runtime_control_step_limit",
            Self::StrikeSegmentLimit => "battle_runtime_strike_segment_limit",
            Self::PredeathBuffLimit => "battle_runtime_predeath_buff_limit",
            Self::RetainedByteLimit => "battle_runtime_retained_byte_limit",
            Self::CounterOverflow => "battle_runtime_counter_overflow",
            Self::Invariant => "battle_runtime_invariant",
            Self::HealthCostUnavailable => "battle_health_cost_unavailable",
            Self::ShieldUnavailable => "battle_shield_unavailable",
            Self::InvalidControlTarget => "battle_invalid_control_target",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BattleRuntimeFault {
    pub code: RuntimeFaultCode,
    pub site: &'static str,
    pub limit: Option<u64>,
    pub requested: Option<u64>,
}

/// Finite host limits. RON cannot override these engineering bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RuntimeFactLimits {
    pub max_work: u64,
    pub max_clone_slots: usize,
    pub max_active_actions: usize,
    pub max_retained_results: usize,
    pub max_deliveries: usize,
    pub max_actors: usize,
    pub max_providers: usize,
    pub max_control_steps: usize,
    pub max_strike_segments: usize,
    pub max_predeath_buffs: usize,
    pub max_retained_bytes: usize,
}

impl Default for RuntimeFactLimits {
    fn default() -> Self {
        crate::runtime_safety::RuntimeSafetySettings::embedded().native_runtime()
    }
}

impl RuntimeFactLimits {
    pub const ENGINE_MAX: Self = Self {
        max_work: 10_000_000,
        max_clone_slots: 1_048_576,
        max_active_actions: 4096,
        max_retained_results: 4096,
        max_deliveries: 8192,
        max_actors: 4096,
        max_providers: 16384,
        max_control_steps: 4096,
        max_strike_segments: 8192,
        max_predeath_buffs: 4096,
        max_retained_bytes: 8 * 1024 * 1024,
    };
    pub(crate) fn validate(&self) -> Result<(), GameError> {
        let ceiling = Self::ENGINE_MAX;
        for (site, value, cap) in [
            ("max_work", self.max_work, ceiling.max_work),
            (
                "max_clone_slots",
                self.max_clone_slots as u64,
                ceiling.max_clone_slots as u64,
            ),
            (
                "max_active_actions",
                self.max_active_actions as u64,
                ceiling.max_active_actions as u64,
            ),
            (
                "max_retained_results",
                self.max_retained_results as u64,
                ceiling.max_retained_results as u64,
            ),
            (
                "max_deliveries",
                self.max_deliveries as u64,
                ceiling.max_deliveries as u64,
            ),
            (
                "max_actors",
                self.max_actors as u64,
                ceiling.max_actors as u64,
            ),
            (
                "max_providers",
                self.max_providers as u64,
                ceiling.max_providers as u64,
            ),
            (
                "max_control_steps",
                self.max_control_steps as u64,
                ceiling.max_control_steps as u64,
            ),
            (
                "max_strike_segments",
                self.max_strike_segments as u64,
                ceiling.max_strike_segments as u64,
            ),
            (
                "max_predeath_buffs",
                self.max_predeath_buffs as u64,
                ceiling.max_predeath_buffs as u64,
            ),
            (
                "max_retained_bytes",
                self.max_retained_bytes as u64,
                ceiling.max_retained_bytes as u64,
            ),
        ] {
            if value == 0 || value > cap {
                return Err(GameError::BattleRuntimeFault(BattleRuntimeFault {
                    code: RuntimeFaultCode::InvalidLimits,
                    site,
                    limit: Some(cap),
                    requested: Some(value),
                }));
            }
        }
        Ok(())
    }
}

crate::authoring::clone_shape::copy_shape!(RuntimeFactLimits, BattleRuntimeFault, RuntimeFaultCode);

struct Budget {
    limits: RuntimeFactLimits,
    work: u64,
    clone_slots: usize,
    retained_admission: Option<usize>,
    #[cfg(feature = "runtime-observation")]
    retained_observation: crate::runtime_observation::RetainedAccounting,
    fault: Option<BattleRuntimeFault>,
}

#[cfg(test)]
thread_local! {
    static RETAINED_ADMISSIONS: RefCell<Option<Vec<(&'static str, usize, usize)>>> = const { RefCell::new(None) };
}

/// Observe successful cumulative admissions without changing native limits.
#[cfg(test)]
pub(crate) fn observe_retained_admissions<T>(
    operation: impl FnOnce() -> T,
) -> (T, Vec<(&'static str, usize, usize)>) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            RETAINED_ADMISSIONS.with(|slot| *slot.borrow_mut() = None);
        }
    }
    RETAINED_ADMISSIONS.with(|slot| {
        assert!(slot.borrow().is_none());
        *slot.borrow_mut() = Some(Vec::new());
    });
    let _reset = Reset;
    let result = operation();
    let admissions = RETAINED_ADMISSIONS.with(|slot| slot.borrow_mut().take().unwrap());
    (result, admissions)
}

/// Stack-local allowance for an ownership walk that cannot invoke gameplay.
/// It admits every aggregate increment without one TLS access per scalar. Fuel
/// is charged once before the actual copy; clone and scan each cost one slot.
#[derive(Clone, Copy)]
pub(crate) struct CloneScanAllowance {
    clone_used: usize,
    clone_limit: usize,
    work_used: u64,
    work_limit: u64,
}
impl CloneScanAllowance {
    pub(crate) fn admit(&self, slots: usize, string_bytes: usize) -> Result<(), GameError> {
        let requested = slots
            .checked_add(string_bytes)
            .and_then(|n| self.clone_used.checked_add(n));
        if requested.is_none_or(|n| n > self.clone_limit) {
            return Err(reject(BattleRuntimeFault {
                code: RuntimeFaultCode::CloneLimit,
                site: "fact_clone_slots",
                limit: Some(self.clone_limit as u64),
                requested: Some(requested.map_or(u64::MAX, |n| n as u64)),
            }));
        }
        let work = (slots as u64)
            .checked_mul(2)
            .and_then(|n| n.checked_add(string_bytes as u64))
            .and_then(|n| self.work_used.checked_add(n));
        if work.is_none_or(|n| n > self.work_limit) {
            return Err(reject(BattleRuntimeFault {
                code: RuntimeFaultCode::WorkLimit,
                site: "fact_clone_scan_and_copy",
                limit: Some(self.work_limit),
                requested: Some(work.unwrap_or(u64::MAX)),
            }));
        }
        Ok(())
    }
}
pub(crate) fn clone_scan_allowance() -> Result<CloneScanAllowance, GameError> {
    poll()?;
    ACTIVE.with(|slot| {
        let slot = slot.borrow();
        let budget = slot
            .as_ref()
            .expect("native clone preflight requires an operation");
        Ok(CloneScanAllowance {
            clone_used: budget.clone_slots,
            clone_limit: budget.limits.max_clone_slots,
            work_used: budget.work,
            work_limit: budget.limits.max_work,
        })
    })
}

thread_local! {
    static ACTIVE: RefCell<Option<Budget>> = const { RefCell::new(None) };
    static DEFERRED_OWNER: Cell<bool> = const { Cell::new(false) };
}

struct Scope;
impl Drop for Scope {
    fn drop(&mut self) {
        ACTIVE.with(|slot| {
            let removed = slot.borrow_mut().take();
            #[cfg(feature = "runtime-observation")]
            if let Some(budget) = &removed {
                crate::runtime_observation::budget(
                    budget.work,
                    budget.clone_slots,
                    budget.retained_admission,
                    budget.retained_observation,
                );
            }
            drop(removed);
        });
    }
}

pub(crate) fn is_active() -> bool {
    ACTIVE.with(|slot| slot.borrow().is_some())
}

pub(crate) fn has_operation_scope() -> bool {
    is_active() || DEFERRED_OWNER.with(Cell::get)
}
struct DeferredScope;
impl Drop for DeferredScope {
    fn drop(&mut self) {
        DEFERRED_OWNER.with(|owner| owner.set(false));
        ACTIVE.with(|slot| {
            let removed = slot.borrow_mut().take();
            #[cfg(feature = "runtime-observation")]
            if let Some(budget) = &removed {
                crate::runtime_observation::budget(
                    budget.work,
                    budget.clone_slots,
                    budget.retained_admission,
                    budget.retained_observation,
                );
            }
            drop(removed);
        });
    }
}

/// World operations that may create a battle own the lifetime of its budget,
/// while unrelated operations remain inactive. The first battle entry activates
/// the scope; subsequent reconciliation/projection shares that same allowance.
pub(crate) fn run_deferred<T>(
    operation: impl FnOnce() -> Result<T, GameError>,
) -> Result<T, GameError> {
    if has_operation_scope() {
        return operation();
    }
    DEFERRED_OWNER.with(|owner| owner.set(true));
    let _scope = DeferredScope;
    let value = operation();
    poll()?;
    value
}

/// Nested candidate/local rollback work shares the enclosing operation's fuel.
pub(crate) fn run<T>(
    limits: RuntimeFactLimits,
    operation: impl FnOnce() -> Result<T, GameError>,
) -> Result<T, GameError> {
    if is_active() {
        poll()?;
        let value = operation();
        poll()?;
        return value;
    }
    limits.validate()?;
    run_initialized(limits, operation)
}

/// Only immutable, validated host settings authorize a larger retained window.
/// Owner limits are retained verbatim, including explicit smaller overrides.
pub(crate) fn run_with_profile<T>(
    limits: RuntimeFactLimits,
    settings: &crate::runtime_safety::RuntimeSafetySettings,
    operation: impl FnOnce() -> Result<T, GameError>,
) -> Result<T, GameError> {
    if is_active() {
        return run(limits, operation);
    }
    if settings.development_measurement().is_none() {
        return run(limits, || {
            #[cfg(feature = "runtime-observation")]
            observe_profile_warning(settings.nominal_native_runtime().max_retained_bytes);
            operation()
        });
    }
    let mut strict = limits;
    strict.max_retained_bytes = strict
        .max_retained_bytes
        .min(RuntimeFactLimits::ENGINE_MAX.max_retained_bytes);
    strict.validate()?;
    let ceiling = settings.native_runtime().max_retained_bytes;
    if limits.max_retained_bytes > ceiling {
        return Err(GameError::BattleRuntimeFault(BattleRuntimeFault {
            code: RuntimeFaultCode::InvalidLimits,
            site: "max_retained_bytes",
            limit: Some(ceiling as u64),
            requested: Some(limits.max_retained_bytes as u64),
        }));
    }
    run_initialized(limits, || {
        #[cfg(feature = "runtime-observation")]
        observe_profile_warning(settings.nominal_native_runtime().max_retained_bytes);
        operation()
    })
}

/// Observation metadata cannot change an active allowance. Only the first
/// settings-bearing entry supplies the nominal profile warning.
#[cfg(feature = "runtime-observation")]
pub(crate) fn observe_profile_warning(nominal: usize) {
    ACTIVE.with(|slot| {
        if let Some(budget) = slot.borrow_mut().as_mut() {
            budget.retained_observation.warning(nominal);
        }
    });
}

fn run_initialized<T>(
    limits: RuntimeFactLimits,
    operation: impl FnOnce() -> Result<T, GameError>,
) -> Result<T, GameError> {
    ACTIVE.with(|slot| {
        *slot.borrow_mut() = Some(Budget {
            limits,
            work: 0,
            clone_slots: 0,
            retained_admission: None,
            #[cfg(feature = "runtime-observation")]
            retained_observation: crate::runtime_observation::RetainedAccounting::new(
                limits.max_retained_bytes,
            ),
            fault: None,
        });
    });
    let _scope = (!DEFERRED_OWNER.with(Cell::get)).then(|| Scope);
    let value = operation();
    poll()?;
    value
}

pub(crate) fn poll() -> Result<(), GameError> {
    ACTIVE.with(
        |slot| match slot.borrow().as_ref().and_then(|budget| budget.fault) {
            Some(fault) => Err(GameError::BattleRuntimeFault(fault)),
            None => Ok(()),
        },
    )
}

pub(crate) fn reject(fault: BattleRuntimeFault) -> GameError {
    let fault = ACTIVE.with(|slot| {
        let mut slot = slot.borrow_mut();
        match slot.as_mut() {
            Some(budget) => *budget.fault.get_or_insert(fault),
            None => fault,
        }
    });
    GameError::BattleRuntimeFault(fault)
}

/// Trusted producer failure, distinct from a caller's ordinary invalid input.
pub(crate) fn invariant(site: &'static str) -> GameError {
    reject(BattleRuntimeFault {
        code: RuntimeFaultCode::Invariant,
        site,
        limit: None,
        requested: None,
    })
}

/// Only use after ordinary target/input refusals have been handled, at an
/// internally owned dispatch boundary. Other typed causes retain their identity.
pub(crate) fn trusted_failure(error: GameError, site: &'static str) -> GameError {
    match error {
        GameError::InvalidAction | GameError::UnitNotFound => invariant(site),
        other => other,
    }
}

pub(crate) fn limit(
    code: RuntimeFaultCode,
    site: &'static str,
    limit: usize,
    requested: Option<usize>,
) -> Result<(), GameError> {
    poll()?;
    #[cfg(feature = "runtime-observation")]
    if code == RuntimeFaultCode::RetainedByteLimit {
        ACTIVE.with(|slot| {
            if let Some(budget) = slot.borrow_mut().as_mut() {
                budget
                    .retained_observation
                    .owner_request(site, requested, limit);
            }
        });
    }
    match requested {
        Some(value) if value <= limit => Ok(()),
        value => Err(reject(BattleRuntimeFault {
            code,
            site,
            limit: Some(limit as u64),
            requested: Some(value.map_or(u64::MAX, |n| n as u64)),
        })),
    }
}

pub(crate) fn charge(units: u64, site: &'static str) -> Result<(), GameError> {
    let failure = ACTIVE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let budget = slot.as_mut()?;
        if let Some(fault) = budget.fault {
            return Some(fault);
        }
        let requested = budget.work.checked_add(units);
        match requested {
            Some(value) if value <= budget.limits.max_work => {
                budget.work = value;
                None
            }
            _ => Some(BattleRuntimeFault {
                code: RuntimeFaultCode::WorkLimit,
                site,
                limit: Some(budget.limits.max_work),
                requested: Some(requested.unwrap_or(u64::MAX)),
            }),
        }
    });
    match failure {
        Some(fault) => Err(reject(fault)),
        None => Ok(()),
    }
}

pub(crate) fn charge_clone(slots: usize, site: &'static str) -> Result<(), GameError> {
    let failure = ACTIVE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let budget = slot.as_mut()?;
        if let Some(fault) = budget.fault {
            return Some(fault);
        }
        let requested = budget.clone_slots.checked_add(slots);
        match requested {
            Some(value) if value <= budget.limits.max_clone_slots => {
                budget.clone_slots = value;
                None
            }
            _ => Some(BattleRuntimeFault {
                code: RuntimeFaultCode::CloneLimit,
                site,
                limit: Some(budget.limits.max_clone_slots as u64),
                requested: Some(requested.map_or(u64::MAX, |n| n as u64)),
            }),
        }
    });
    if let Some(fault) = failure {
        return Err(reject(fault));
    }
    charge(slots as u64, site)
}

/// Seed the operation from its first live owner. Candidate/local clones join
/// this scope and cannot reset the admitted-allocation count after rollback.
pub(crate) fn seed_retained_extent(bytes: usize) -> Result<(), GameError> {
    let requested = ACTIVE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let budget = slot.as_mut()?;
        if budget.retained_admission.is_some() {
            return None;
        }
        budget.retained_admission = Some(bytes);
        #[cfg(feature = "runtime-observation")]
        budget.retained_observation.seed_attempt(bytes);
        Some(budget.limits.max_retained_bytes)
    });
    if let Some(limit_bytes) = requested {
        limit(
            RuntimeFaultCode::RetainedByteLimit,
            "retained_operation_start",
            limit_bytes,
            Some(bytes),
        )?;
        #[cfg(feature = "runtime-observation")]
        ACTIVE.with(|slot| {
            if let Some(budget) = slot.borrow_mut().as_mut() {
                budget.retained_observation.seed_accept(bytes);
            }
        });
    }
    Ok(())
}

/// Conservative cumulative growth admission, in addition to the live owner's
/// current extent ledger. Releases do not replenish an operation's allowance;
/// a new public operation starts from the then-current retained extent.
pub(crate) fn admit_retained_growth(
    old: usize,
    new: Option<usize>,
    site: &'static str,
) -> Result<(), GameError> {
    admit_retained_growth_with_inactive_cap(
        old,
        new,
        site,
        RuntimeFactLimits::default().max_retained_bytes,
    )
}

/// Outside an operation only a validated owner's finite local allocation cap
/// applies. An active budget always takes precedence and remains cumulative.
pub(crate) fn admit_retained_growth_with_owner_limits(
    old: usize,
    new: Option<usize>,
    site: &'static str,
    owner_limits: RuntimeFactLimits,
) -> Result<(), GameError> {
    admit_retained_growth_with_inactive_cap(old, new, site, owner_limits.max_retained_bytes)
}

fn admit_retained_growth_with_inactive_cap(
    old: usize,
    new: Option<usize>,
    site: &'static str,
    inactive_cap: usize,
) -> Result<(), GameError> {
    poll()?;
    let ceiling = ACTIVE.with(|slot| {
        slot.borrow()
            .as_ref()
            .map_or(inactive_cap, |budget| budget.limits.max_retained_bytes)
    });
    limit(RuntimeFaultCode::RetainedByteLimit, site, ceiling, new)?;
    let growth = new.expect("admitted extent").saturating_sub(old);
    #[cfg(feature = "runtime-observation")]
    ACTIVE.with(|slot| {
        if let Some(budget) = slot.borrow_mut().as_mut() {
            let requested = budget.retained_admission.unwrap_or(0).checked_add(growth);
            budget
                .retained_observation
                .cumulative_attempt(site, requested);
        }
    });
    crate::authoring::control::charge((growth as u64).div_ceil(8), site)?;
    let failure = ACTIVE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let budget = slot.as_mut()?;
        let requested = budget.retained_admission.unwrap_or(0).checked_add(growth);
        match requested {
            Some(value) if value <= budget.limits.max_retained_bytes => {
                #[cfg(test)]
                let previous = budget.retained_admission.unwrap_or(0);
                budget.retained_admission = Some(value);
                #[cfg(feature = "runtime-observation")]
                budget.retained_observation.growth_accept(growth);
                #[cfg(test)]
                RETAINED_ADMISSIONS.with(|slot| {
                    if let Some(admissions) = slot.borrow_mut().as_mut() {
                        admissions.push((site, previous, value));
                    }
                });
                None
            }
            _ => {
                #[cfg(feature = "runtime-observation")]
                budget.retained_observation.cumulative_refused(
                    site,
                    requested,
                    budget.limits.max_retained_bytes,
                );
                Some(BattleRuntimeFault {
                    code: RuntimeFaultCode::RetainedByteLimit,
                    site,
                    limit: Some(budget.limits.max_retained_bytes as u64),
                    requested: Some(requested.map_or(u64::MAX, |value| value as u64)),
                })
            }
        }
    });
    if let Some(failure) = failure {
        return Err(reject(failure));
    }
    Ok(())
}

/// Choose one bounded buffer replacement without latching a fault for an
/// unaffordable preferred size. The actual full new allocation is admitted
/// exactly once; if even the minimum does not fit, that admission reports it.
pub(crate) fn admitted_buffer_capacity(
    len: usize,
    capacity: usize,
    max_elements: usize,
    element_size: usize,
    overhead: usize,
    site: &'static str,
) -> Result<usize, GameError> {
    poll()?;
    let minimum = len.checked_add(1).ok_or_else(|| {
        reject(BattleRuntimeFault {
            code: RuntimeFaultCode::CounterOverflow,
            site,
            limit: None,
            requested: None,
        })
    })?;
    if minimum > max_elements || element_size == 0 {
        return Err(reject(BattleRuntimeFault {
            code: RuntimeFaultCode::Invariant,
            site,
            limit: None,
            requested: None,
        }));
    }
    let preferred = capacity
        .checked_add((capacity / 2).max(1))
        .unwrap_or(max_elements)
        .max(2)
        .max(minimum)
        .min(max_elements);
    let (retained_left, native_work_left) = ACTIVE.with(|slot| {
        let slot = slot.borrow();
        slot.as_ref().map_or(
            (RuntimeFactLimits::default().max_retained_bytes, u64::MAX),
            |budget| {
                (
                    budget
                        .limits
                        .max_retained_bytes
                        .saturating_sub(budget.retained_admission.unwrap_or(0)),
                    budget.limits.max_work.saturating_sub(budget.work),
                )
            },
        )
    });
    let authoring_work_left =
        crate::authoring::control::growth_work_allowance().unwrap_or(u64::MAX);
    let work_bytes = usize::try_from(native_work_left.min(authoring_work_left))
        .unwrap_or(usize::MAX)
        .saturating_mul(8);
    let affordable = retained_left.min(work_bytes).saturating_sub(overhead) / element_size;
    let selected = preferred.min(affordable.max(minimum));
    let bytes = selected
        .checked_mul(element_size)
        .and_then(|bytes| bytes.checked_add(overhead));
    admit_retained_growth(0, bytes, site)?;
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmrp_settings(cap: usize) -> crate::runtime_safety::RuntimeSafetySettings {
        let mut source =
            String::from_utf8(crate::runtime_safety::EMBEDDED_RUNTIME_SAFETY_RON.to_vec()).unwrap();
        let end = source.rfind(')').unwrap();
        source.insert_str(
            end,
            &format!(
                "development_measurement: Some((revision: 1, emergency_retained_bytes: {cap})),\n"
            ),
        );
        crate::runtime_safety::RuntimeSafetySettings::from_ron(
            "trusted native boundary fixture",
            source.as_bytes(),
        )
        .unwrap()
    }

    #[test]
    fn cmrp_native_actual_cap_seed_growth_exact_and_one_over() {
        for cap in [8 * 1024 * 1024, 16 * 1024 * 1024, 32 * 1024 * 1024] {
            let settings = if cap == 8 * 1024 * 1024 {
                crate::runtime_safety::RuntimeSafetySettings::embedded().clone()
            } else {
                cmrp_settings(cap)
            };
            let limits = settings.native_runtime();
            run_with_profile(limits, &settings, || seed_retained_extent(cap)).unwrap();
            let error =
                run_with_profile(limits, &settings, || seed_retained_extent(cap + 1)).unwrap_err();
            assert!(
                matches!(error, GameError::BattleRuntimeFault(BattleRuntimeFault { code: RuntimeFaultCode::RetainedByteLimit, site: "retained_operation_start", limit: Some(n), requested: Some(r) }) if n == cap as u64 && r == cap as u64 + 1)
            );
            run_with_profile(limits, &settings, || {
                seed_retained_extent(cap / 4)?;
                admit_retained_growth(0, Some(cap - cap / 4), "cmrp_exact_growth")
            })
            .unwrap();
            let error = run_with_profile(limits, &settings, || {
                seed_retained_extent(cap / 4)?;
                admit_retained_growth(0, Some(cap - cap / 4 + 1), "cmrp_one_over_growth")
            })
            .unwrap_err();
            assert!(
                matches!(error, GameError::BattleRuntimeFault(BattleRuntimeFault { code: RuntimeFaultCode::RetainedByteLimit, site: "cmrp_one_over_growth", limit: Some(n), requested: Some(r) }) if n == cap as u64 && r == cap as u64 + 1)
            );
        }
    }

    #[test]
    fn cmrp_small_owner_override_and_nested_deferred_do_not_raise_or_refund() {
        let settings = cmrp_settings(32 * 1024 * 1024);
        let limits = RuntimeFactLimits {
            max_retained_bytes: 255,
            ..Default::default()
        };
        let error = run_deferred(|| {
            run_with_profile(limits, &settings, || {
                seed_retained_extent(128)?;
                run_with_profile(settings.native_runtime(), &settings, || {
                    admit_retained_growth(0, Some(127), "cmrp_nested")
                })?;
                seed_retained_extent(0)?;
                admit_retained_growth(0, Some(1), "cmrp_after_discard")
            })
        })
        .unwrap_err();
        assert!(matches!(
            error,
            GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: RuntimeFaultCode::RetainedByteLimit,
                site: "cmrp_after_discard",
                limit: Some(255),
                requested: Some(256)
            })
        ));
        assert!(!has_operation_scope());
        run_with_profile(limits, &settings, || {
            seed_retained_extent(128)?;
            admit_retained_growth(0, Some(127), "cmrp_retry")
        })
        .unwrap();
        assert!(run(settings.native_runtime(), || Ok(())).is_err());
    }

    #[test]
    fn cmrp_other_limits_and_overflow_keep_typed_failures() {
        let settings = cmrp_settings(32 * 1024 * 1024);
        let limits = settings.native_runtime();
        let invalid = RuntimeFactLimits {
            max_work: RuntimeFactLimits::ENGINE_MAX.max_work + 1,
            ..limits
        };
        assert!(matches!(
            run_with_profile(invalid, &settings, || Ok(())).unwrap_err(),
            GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: RuntimeFaultCode::InvalidLimits,
                site: "max_work",
                ..
            })
        ));
        let work = RuntimeFactLimits {
            max_work: 2,
            ..limits
        };
        assert!(matches!(
            run_with_profile(work, &settings, || charge(3, "cmrp_work")).unwrap_err(),
            GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: RuntimeFaultCode::WorkLimit,
                site: "cmrp_work",
                ..
            })
        ));
        let clones = RuntimeFactLimits {
            max_clone_slots: 2,
            ..limits
        };
        assert!(matches!(
            run_with_profile(clones, &settings, || charge_clone(3, "cmrp_clone")).unwrap_err(),
            GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: RuntimeFaultCode::CloneLimit,
                site: "cmrp_clone",
                ..
            })
        ));
        assert!(
            run_with_profile(limits, &settings, || admit_retained_growth(
                0,
                None,
                "cmrp_overflow"
            ))
            .is_err()
        );
    }

    #[test]
    fn cmrp_local_projection_owner_cap_applies_only_without_active_budget() {
        let settings = cmrp_settings(16 * 1024 * 1024);
        let owner = settings.native_runtime();
        let extent = 9 * 1024 * 1024;
        assert!(!is_active());
        assert!(admit_retained_growth(0, Some(extent), "cmrp_original_fallback").is_err());
        admit_retained_growth_with_owner_limits(0, Some(extent), "cmrp_local_projection", owner)
            .unwrap();
        let error = run(
            RuntimeFactLimits {
                max_retained_bytes: 255,
                ..Default::default()
            },
            || {
                admit_retained_growth_with_owner_limits(
                    0,
                    Some(256),
                    "cmrp_outer_projection",
                    owner,
                )
            },
        )
        .unwrap_err();
        assert!(matches!(
            error,
            GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: RuntimeFaultCode::RetainedByteLimit,
                site: "cmrp_outer_projection",
                limit: Some(255),
                requested: Some(256)
            })
        ));
        assert!(!is_active());
    }

    #[test]
    fn bounded_buffer_growth_uses_minimum_when_spare_reservation_exceeds_allowance() {
        let bytes_for_four = 4 * 32 + 64;
        let limits = RuntimeFactLimits {
            max_retained_bytes: bytes_for_four,
            max_work: (bytes_for_four / 8) as u64,
            ..Default::default()
        };
        run(limits, || {
            seed_retained_extent(0)?;
            assert_eq!(
                admitted_buffer_capacity(3, 3, 100, 32, 64, "buffer_growth")?,
                4
            );
            Ok(())
        })
        .unwrap();
        run(RuntimeFactLimits::default(), || {
            assert_eq!(
                admitted_buffer_capacity(0, 0, 100, 32, 64, "buffer_growth")?,
                2
            );
            assert_eq!(
                admitted_buffer_capacity(3, 3, 4, 32, 64, "buffer_growth")?,
                4
            );
            Ok(())
        })
        .unwrap();
        let error = run(
            RuntimeFactLimits {
                max_retained_bytes: bytes_for_four - 1,
                ..Default::default()
            },
            || {
                seed_retained_extent(0)?;
                admitted_buffer_capacity(3, 3, 100, 32, 64, "buffer_growth")?;
                Ok(())
            },
        )
        .unwrap_err();
        assert!(matches!(
            error,
            GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: RuntimeFaultCode::RetainedByteLimit,
                site: "buffer_growth",
                ..
            })
        ));
        let error = run(RuntimeFactLimits::default(), || {
            admitted_buffer_capacity(usize::MAX, usize::MAX, usize::MAX, 32, 64, "overflow")?;
            Ok(())
        })
        .unwrap_err();
        assert!(matches!(
            error,
            GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: RuntimeFaultCode::CounterOverflow,
                site: "overflow",
                ..
            })
        ));
        use crate::authoring::{
            control as authoring_control, CancellationToken, ExecutionLimits, ExecutionPhase,
        };
        let work_limits = ExecutionLimits {
            max_work: 28,
            ..Default::default()
        };
        let (selected, _) = authoring_control::run(
            &work_limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || admitted_buffer_capacity(4, 4, 100, 32, 64, "buffer_work"),
        )
        .unwrap();
        assert_eq!(selected.unwrap(), 5);
        let error = authoring_control::run(
            &ExecutionLimits {
                max_work: 27,
                ..Default::default()
            },
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || admitted_buffer_capacity(4, 4, 100, 32, 64, "buffer_work"),
        )
        .unwrap_err();
        assert_eq!(error.site, "buffer_work");
    }
    #[test]
    fn native_clone_limit_is_cumulative_inclusive_and_recovers_next_operation() {
        let limits = RuntimeFactLimits {
            max_clone_slots: 2,
            ..Default::default()
        };
        run(limits, || charge_clone(2, "exact_copy")).unwrap();
        let error = run(limits, || {
            charge_clone(1, "first_copy")?;
            run(limits, || charge_clone(2, "nested_copy"))
        })
        .unwrap_err();
        assert!(matches!(
            error,
            GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: RuntimeFaultCode::CloneLimit,
                limit: Some(2),
                requested: Some(3),
                ..
            })
        ));
        run(limits, || charge_clone(2, "retry_copy")).unwrap();
    }

    #[test]
    fn native_deferred_world_budget_activates_once_and_keeps_first_fault() {
        let limits = RuntimeFactLimits {
            max_work: 2,
            ..Default::default()
        };
        let result = run_deferred(|| {
            assert!(!is_active());
            charge(1000, "unrelated_world_work")?;
            run(limits, || charge(2, "battle_start"))?;
            assert!(is_active());
            let error = run(limits, || charge(1, "later_reconcile")).unwrap_err();
            assert!(matches!(
                error,
                GameError::BattleRuntimeFault(BattleRuntimeFault {
                    site: "later_reconcile",
                    code: RuntimeFaultCode::WorkLimit,
                    ..
                })
            ));
            // An infallible inner path cannot hide the fault from the owner.
            Ok(())
        });
        assert!(result.is_err());
        assert!(!has_operation_scope());
        run(limits, || charge(2, "new_public_operation")).unwrap();
    }
    #[test]
    fn native_retained_admission_does_not_refund_on_candidate_discard() {
        let limits = RuntimeFactLimits {
            max_retained_bytes: 512,
            ..Default::default()
        };
        let error = run(limits, || {
            seed_retained_extent(256)?;
            admit_retained_growth(0, Some(128), "candidate_source")?;
            // Simulated local discard neither reseeds nor replenishes allowance.
            seed_retained_extent(256)?;
            admit_retained_growth(0, Some(129), "second_candidate_source")
        })
        .unwrap_err();
        assert!(matches!(
            error,
            GameError::BattleRuntimeFault(BattleRuntimeFault {
                code: RuntimeFaultCode::RetainedByteLimit,
                requested: Some(513),
                ..
            })
        ));
        run(limits, || {
            seed_retained_extent(256)?;
            admit_retained_growth(0, Some(256), "inclusive_retry")
        })
        .unwrap();
    }
}
