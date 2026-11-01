use super::*;
use crate::game::ability::sustained::{PreparedBasicProfile, PreparedBasicReward};
impl<'a> Lower<'a> {
    pub(super) fn prepared_basic_action(
        &mut self,
        request: &str,
        plan: &'a str,
        admission: ResonanceAdmission,
    ) -> Result<GraphOperation, Diagnostic> {
        self.self_selection(request)?;
        let (numerator, denominator) = self.exact_ratio(plan, "multiplier")?;
        let kill_reward = if self.doc.nodes[plan].inputs.contains_key("reward") {
            let state = self.input(plan, "reward", "skill-target-state")?;
            let (duration_ms, profile) = self.state_profile(state)?;
            Some(PreparedBasicReward {
                duration_ms,
                profile,
            })
        } else {
            None
        };
        let profile = PreparedBasicProfile {
            definition: format!("{}:{plan}", self.doc.document),
            numerator,
            denominator,
            kill_reward,
        };
        profile.validate().map_err(|e| self.error(plan, None, &e))?;
        Ok(GraphOperation::ResonancePreparedBasic { admission, profile })
    }
}
