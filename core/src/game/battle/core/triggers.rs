use uuid::Uuid;

use crate::game::stats::TriggerType;

use super::{BattleCore, RuntimeArtifact, RuntimeItem, TriggerSource};
use crate::game::battle::cooldown::{CooldownSource, SourcedEffect};

impl BattleCore {
    pub(super) fn collect_triggers(
        &self,
        source: TriggerSource,
        trigger: TriggerType,
    ) -> Vec<SourcedEffect> {
        let mut effects: Vec<SourcedEffect> = Vec::new();

        match source {
            TriggerSource::Artifact { side } => {
                let mut artifacts: Vec<&RuntimeArtifact> = self
                    .artifacts
                    .values()
                    .filter(|a| a.owner == side)
                    .collect();
                artifacts.sort_by(|a, b| a.instance_id.as_bytes().cmp(b.instance_id.as_bytes()));

                for artifact in artifacts {
                    if let Some(metadata) = self
                        .game_data
                        .artifact_data
                        .get_by_uuid(&artifact.base_uuid)
                    {
                        if let Some(triggered) = metadata.triggered_effects.get(&trigger) {
                            effects.extend(triggered.iter().cloned().map(|effect| SourcedEffect {
                                source: CooldownSource::Artifact {
                                    artifact_instance_id: artifact.instance_id,
                                },
                                effect,
                            }));
                        }
                    }
                }
            }
            TriggerSource::Item { unit_instance_id } => {
                let mut items: Vec<&RuntimeItem> = self
                    .items
                    .values()
                    .filter(|i| i.owner_unit_instance == unit_instance_id)
                    .collect();
                items.sort_by(|a, b| a.instance_id.as_bytes().cmp(b.instance_id.as_bytes()));

                for item in items {
                    if let Some(metadata) =
                        self.game_data.equipment_data.get_by_uuid(&item.base_uuid)
                    {
                        if let Some(triggered) = metadata.triggered_effects.get(&trigger) {
                            effects.extend(triggered.iter().cloned().map(|effect| SourcedEffect {
                                source: CooldownSource::Item {
                                    item_instance_id: item.instance_id,
                                },
                                effect,
                            }));
                        }
                    }
                }
            }
        }

        effects
    }

    pub(super) fn collect_all_triggers(
        &self,
        unit_instance_id: Uuid,
        trigger: TriggerType,
    ) -> Vec<SourcedEffect> {
        let Some(unit) = self.units.get(&unit_instance_id) else {
            return Vec::new();
        };

        let mut effects =
            self.collect_triggers(TriggerSource::Artifact { side: unit.owner }, trigger);
        effects.extend(self.collect_triggers(TriggerSource::Item { unit_instance_id }, trigger));

        effects
    }
}
