//! Fixed-size observation of native retained admission. Never a safety authority.
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetainedRequestKind {
    Seed,
    Owner,
    CumulativeGrowth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RetainedExceed {
    pub kind: RetainedRequestKind,
    pub site: &'static str,
    /// None means checked arithmetic could not produce a finite request.
    pub requested_bytes: Option<u64>,
    pub limit_bytes: u64,
    pub overflowed: bool,
}

/// The first seed belongs to the first native scope in the public operation.
/// Accepted counters aggregate scopes; nested native entries reuse one scope.
/// Legacy admitted_bytes remains separate and can include a refused seed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct RetainedAccounting {
    pub native_scopes: u64,
    /// Unknown for a raw native entry without immutable settings metadata.
    pub profile_warning_bytes: Option<u64>,
    pub owner_hard_bytes: u64,
    pub seed_attempted_bytes: Option<u64>,
    pub seed_accepted_bytes: Option<u64>,
    pub accepted_growth_bytes: u64,
    pub accepted_total_bytes: u64,
    pub max_cumulative_requested_bytes: Option<u64>,
    pub max_owner_requested_bytes: Option<u64>,
    pub cumulative_request_overflowed: bool,
    pub owner_request_overflowed: bool,
    pub first_soft_exceed: Option<RetainedExceed>,
    pub first_hard_exceed: Option<RetainedExceed>,
    pub multiple_policies: bool,
    /// Aggregation saturates only with this explicit unavailable-total flag.
    pub aggregation_overflowed: bool,
}

impl RetainedAccounting {
    pub(crate) fn new(hard: usize) -> Self {
        Self {
            native_scopes: 1,
            owner_hard_bytes: hard as u64,
            ..Self::default()
        }
    }

    pub(crate) fn warning(&mut self, nominal: usize) {
        // Nested entries cannot replace the enclosing operation's profile.
        self.profile_warning_bytes.get_or_insert(nominal as u64);
    }

    fn exceed(
        kind: RetainedRequestKind,
        site: &'static str,
        requested: Option<usize>,
        limit: u64,
    ) -> RetainedExceed {
        RetainedExceed {
            kind,
            site,
            requested_bytes: requested.map(|v| v as u64),
            limit_bytes: limit,
            overflowed: requested.is_none(),
        }
    }

    fn soft(&mut self, kind: RetainedRequestKind, site: &'static str, requested: Option<usize>) {
        if let Some(warning) = self.profile_warning_bytes {
            if requested.is_none_or(|v| v as u64 > warning) {
                self.first_soft_exceed
                    .get_or_insert_with(|| Self::exceed(kind, site, requested, warning));
            }
        }
    }

    fn hard(
        &mut self,
        kind: RetainedRequestKind,
        site: &'static str,
        requested: Option<usize>,
        limit: usize,
    ) {
        if requested.is_none_or(|v| v > limit) {
            self.first_hard_exceed
                .get_or_insert_with(|| Self::exceed(kind, site, requested, limit as u64));
        }
    }

    fn maximum(slot: &mut Option<u64>, requested: Option<usize>, overflow: &mut bool) {
        if let Some(value) = requested {
            *slot = Some(slot.unwrap_or(0).max(value as u64));
        } else {
            *overflow = true;
        }
    }

    pub(crate) fn seed_attempt(&mut self, bytes: usize) {
        self.seed_attempted_bytes = Some(bytes as u64);
        Self::maximum(
            &mut self.max_cumulative_requested_bytes,
            Some(bytes),
            &mut self.cumulative_request_overflowed,
        );
        self.soft(
            RetainedRequestKind::Seed,
            "retained_operation_start",
            Some(bytes),
        );
    }

    pub(crate) fn seed_accept(&mut self, bytes: usize) {
        self.seed_accepted_bytes = Some(bytes as u64);
        self.accepted_total_bytes = bytes as u64;
    }

    pub(crate) fn owner_request(
        &mut self,
        site: &'static str,
        requested: Option<usize>,
        limit: usize,
    ) {
        Self::maximum(
            &mut self.max_owner_requested_bytes,
            requested,
            &mut self.owner_request_overflowed,
        );
        // Seed already records the same soft attempt with the more precise kind.
        let kind = if site == "retained_operation_start" {
            RetainedRequestKind::Seed
        } else {
            RetainedRequestKind::Owner
        };
        self.soft(kind, site, requested);
        self.hard(kind, site, requested, limit);
    }

    pub(crate) fn cumulative_attempt(&mut self, site: &'static str, requested: Option<usize>) {
        Self::maximum(
            &mut self.max_cumulative_requested_bytes,
            requested,
            &mut self.cumulative_request_overflowed,
        );
        self.soft(RetainedRequestKind::CumulativeGrowth, site, requested);
    }

    pub(crate) fn cumulative_refused(
        &mut self,
        site: &'static str,
        requested: Option<usize>,
        limit: usize,
    ) {
        self.hard(
            RetainedRequestKind::CumulativeGrowth,
            site,
            requested,
            limit,
        );
    }

    pub(crate) fn growth_accept(&mut self, growth: usize) {
        // The real native admission checked the finite hard cap first.
        self.accepted_growth_bytes += growth as u64;
        self.accepted_total_bytes += growth as u64;
    }

    pub(crate) fn include(&mut self, other: Self) {
        if self.native_scopes == 0 {
            *self = other;
            return;
        }
        self.multiple_policies |= other.multiple_policies
            || self.profile_warning_bytes != other.profile_warning_bytes
            || self.owner_hard_bytes != other.owner_hard_bytes;
        for (a, b) in [
            (&mut self.native_scopes, other.native_scopes),
            (&mut self.accepted_growth_bytes, other.accepted_growth_bytes),
            (&mut self.accepted_total_bytes, other.accepted_total_bytes),
        ] {
            if let Some(value) = a.checked_add(b) {
                *a = value;
            } else {
                *a = u64::MAX;
                self.aggregation_overflowed = true;
            }
        }
        for (a, b) in [
            (
                &mut self.max_cumulative_requested_bytes,
                other.max_cumulative_requested_bytes,
            ),
            (
                &mut self.max_owner_requested_bytes,
                other.max_owner_requested_bytes,
            ),
        ] {
            if let Some(value) = b {
                *a = Some(a.unwrap_or(0).max(value));
            }
        }
        self.cumulative_request_overflowed |= other.cumulative_request_overflowed;
        self.owner_request_overflowed |= other.owner_request_overflowed;
        self.aggregation_overflowed |= other.aggregation_overflowed;
        self.first_soft_exceed = self.first_soft_exceed.or(other.first_soft_exceed);
        self.first_hard_exceed = self.first_hard_exceed.or(other.first_hard_exceed);
    }
}
