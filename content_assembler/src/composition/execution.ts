import { invoke, isTauri } from "@tauri-apps/api/core";
import { digest } from "./client";
import type { Definition } from "./session";

export type TrialConfig = {
  root: string;
  selection: { packs: string[]; winners: unknown[] };
  scenario: unknown;
  lab?: unknown;
};
export type SourceTrace = {
  definition: string;
  calls: string[];
  inputs: string[];
};
export const sustainedEndReasons = [
  "Expired",
  "Stopped",
  "Interrupted",
  "HardCrowdControl",
  "SourceUnavailable",
  "TargetUnavailable",
  "Terminal",
  "AttackReleased",
  "Depleted",
  "Triggered",
  "Reinput",
  "Replaced",
] as const;
type AreaMembership = { members: number; contributions: number };
export type ObservedArea = {
  ordinal: number;
  time_ms: number;
  area_id: string;
  skill_id: string;
  step_id: string;
  caster: string;
  cast_sequence: number;
  started_at_ms: number;
  expires_at_ms: number;
  change:
    | { kind: "started" | "reconciled" }
    | { kind: "ended"; reason: (typeof sustainedEndReasons)[number] };
  before: AreaMembership | null;
  after: AreaMembership | null;
  battle_terminal: boolean;
  source_file: string | null;
  field_path: string | null;
  origin: SourceTrace | null;
};
export type RunResponse = {
  epoch: number;
  revision: number;
  request_id: number;
  source_digest: string;
  artifact_digest: string;
  status: "completed" | "rejected" | "failed";
  accepted_steps: number;
  identity: null | {
    run_id: string;
    source_digest: string;
    artifact_digest: string;
    artifact_path: string;
    content: { semantic_sha256: string };
    scenario_digest: string;
  };
  latest_effect: null | {
    time_ms: number;
    skill_id: string | null;
    definition_id: string;
    equipment: null | {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string | null;
    };
    source_file: string;
    field_path: string;
    target_unit: string;
    hp_delta: number;
    hp_reason?:
      | null
      | "Corrosion"
      | "HealthCost"
      | "BasicAttack"
      | "Command"
      | "Rest"
      | "FieldWork";
    origin: null | SourceTrace;
  };
  latest_resonance?: null | {
    time_ms: number;
    battle_terminal: boolean;
    skill_id: string | null;
    definition_id: string;
    equipment: NonNullable<RunResponse["latest_effect"]>["equipment"];
    source_file: string;
    field_path: string;
    target_unit: string;
    before: number;
    after: number;
    max: number;
    origin: null | SourceTrace;
  };
  latest_hit_counter?: null | {
    equipment: NonNullable<RunResponse["latest_effect"]>["equipment"];
    area_source: NonNullable<RunResponse["latest_sustained"]>["area_source"];
    time_ms: number;
    battle_terminal: boolean;
    definition_id: string;
    skill_id: string | null;
    cast_sequence: number | null;
    source_file: string;
    field_path: string;
    target_unit: string;
    before: number;
    added: number;
    after: number;
    threshold: number;
    control: "Stun" | "Root";
    triggered: boolean;
    control_applied: boolean;
    counter_expires_at_ms: number | null;
    control_expires_at_ms: number | null;
    origin: SourceTrace | null;
  };
  latest_sustained: null | {
    equipment?: NonNullable<RunResponse["latest_effect"]>["equipment"];
    area_source?: null | {
      area_id: string;
      skill_id: string;
      step_id: string;
      cast_sequence: number;
      status_index: number;
      definition: string;
      modifier_source: null | {
        copy_uuid: string;
        definition: string;
        rule: string;
      };
    };
    time_ms: number;
    battle_terminal: boolean;
    definition_id: string;
    skill_id: string | null;
    cast_sequence: number | null;
    source_file: string;
    field_path: string;
    target_unit: string;
    activation: number;
    ended: null | (typeof sustainedEndReasons)[number];
    expires_at_ms: number;
    basic_speed_reduction_percent?: number | null;
    damage_taken_increase_percent?: number | null;
    target_state?: null | {
      damage_invulnerable: boolean;
      basic_attack_locked: boolean;
      active_skill_locked: boolean;
      received_heal_percent: number;
    };
    before: null | ObservedStats;
    after: null | ObservedStats;
    origin: null | SourceTrace;
  };
  latest_area: ObservedArea | null;
  latest_ready_wait?: null | {
    time_ms: number;
    actor: string;
    skill_id: string;
    percent: number;
    battle_terminal: boolean;
    contributors: {
      equipment: {
        definition_uuid: string;
        runtime_item_instance_id: string;
        persistent_copy_uuid: string;
      };
      waited_ms: number;
      per_second_percent: number;
      max_percent: number;
      percent: number;
      until_ms: number;
      definition_id: string;
      source_file: string;
      field_path: string;
    }[];
    origins: (SourceTrace | null)[];
  };
  latest_nearby_defense?: null | {
    time_ms: number;
    actor: string;
    bonus: number;
    actual_defense: number;
    battle_terminal: boolean;
    contributors: {
      equipment: {
        definition_uuid: string;
        runtime_item_instance_id: string;
        persistent_copy_uuid: string;
      };
      count: number;
      per_enemy: number;
      max_bonus: number;
      bonus: number;
      definition_id: string;
      source_file: string;
      field_path: string;
    }[];
    origins: (SourceTrace | null)[];
  };
  latest_area_damage?: null | {
    time_ms: number;
    actor: string;
    action: number | null;
    target_count: number;
    percent: number;
    battle_terminal: boolean;
    contributors: {
      equipment: {
        definition_uuid: string;
        runtime_item_instance_id: string;
        persistent_copy_uuid: string;
      };
      percent: number;
      definition_id: string;
      source_file: string;
      field_path: string;
    }[];
    origins: (SourceTrace | null)[];
  };
  latest_actual_heal_reaction?: null | {
    time_ms: number;
    actor: string;
    target: string;
    actual_healed: number;
    healing_source: { actor: string | null; action_ordinal?: number | null };
    battle_terminal: boolean;
    contributors: {
      equipment: {
        definition_uuid: string;
        runtime_item_instance_id: string;
        persistent_copy_uuid: string | null;
      };
      definition_id: string;
      source_file: string;
      field_path: string;
      self_shield: null | {
        actual_heal_percent: number;
        max_hp_cap_percent: number;
        before: number;
        after: number;
        cap: number;
        remainder_before: number;
        remainder_after: number;
        expires_at_ms: number;
      };
      target_haste: null | {
        attack_speed_points: number;
        expires_at_ms: number;
        refreshed: boolean;
      };
    }[];
    origins: (SourceTrace | null)[];
  };
  latest_resonance_gain?: null | {
    time_ms: number;
    actor: string;
    channel: "automatic" | "authored";
    base_amount: number;
    increased_amount: number;
    actual_gain: number;
    before: number;
    after: number;
    max: number;
    percent: number;
    remainder_before: number;
    remainder_after: number;
    battle_terminal: boolean;
    contributors: {
      equipment: {
        definition_uuid: string;
        runtime_item_instance_id: string;
        persistent_copy_uuid: string | null;
      };
      percent: number;
      definition_id: string;
      source_file: string;
      field_path: string;
    }[];
    origins: (SourceTrace | null)[];
  };
  latest_nearby_ally_damage?: null | {
    time_ms: number;
    actor: string;
    wearer: string;
    target: string;
    percent: number;
    battle_terminal: boolean;
    contributors: {
      equipment: {
        definition_uuid: string;
        runtime_item_instance_id: string;
        persistent_copy_uuid: string;
      };
      count: number;
      percent: number;
      definition_id: string;
      source_file: string;
      field_path: string;
    }[];
    origins: (SourceTrace | null)[];
  };
  latest_conditional_damage?: null | {
    time_ms: number;
    actor: string;
    wearer: string;
    target: string;
    wearer_hp: number;
    wearer_max_hp: number;
    target_hp: number;
    target_max_hp: number;
    target_movement_impaired: boolean | null;
    distance_squared?: number | null;
    percent: number;
    battle_terminal: boolean;
    contributors: {
      equipment: {
        definition_uuid: string;
        runtime_item_instance_id: string;
        persistent_copy_uuid: string;
      };
      matched: boolean;
      percent: number;
      definition_id: string;
      source_file: string;
      field_path: string;
    }[];
    origins: (SourceTrace | null)[];
  };
  latest_conditional_reduction?:
    | null
    | (NonNullable<RunResponse["latest_conditional_damage"]> & {
        equipment: {
          definition_uuid: string;
          runtime_item_instance_id: string;
          persistent_copy_uuid: string | null;
        };
        incoming: { actor: string; action_ordinal: number | null };
      });
  latest_primary_damage?: null | {
    time_ms: number;
    actor: string;
    target: string;
    action: number;
    target_current_hp: number;
    target_max_hp: number;
    percent: number;
    battle_terminal: boolean;
    contributors: {
      equipment: {
        definition_uuid: string;
        runtime_item_instance_id: string;
        persistent_copy_uuid: string;
      };
      percent: number;
      definition_id: string;
      source_file: string;
      field_path: string;
    }[];
    origins: (SourceTrace | null)[];
  };
  latest_equipment_speed?: null | {
    time_ms: number;
    actor: string;
    current_hp: number;
    max_hp: number;
    base_interval_ms: number;
    final_interval_ms: number;
    equipment_points: number;
    battle_terminal: boolean;
    contributors: {
      equipment: {
        definition_uuid: string;
        runtime_item_instance_id: string;
        persistent_copy_uuid: string | null;
      };
      points: number;
      definition_id: string;
      source_file: string;
      field_path: string;
    }[];
    origins: (SourceTrace | null)[];
  };
  latest_equipment_support?: null | {
    time_ms: number;
    actor: string;
    target: string;
    channel: "heal" | "shield";
    base_amount: number;
    increased_amount: number;
    percent: number;
    battle_terminal: boolean;
    contributors: {
      equipment: {
        definition_uuid: string;
        runtime_item_instance_id: string;
        persistent_copy_uuid: string | null;
      };
      percent: number;
      definition_id: string;
      source_file: string;
      field_path: string;
    }[];
    origins: (SourceTrace | null)[];
  };
  latest_overheal_reserve?: null | {
    time_ms: number;
    target: string;
    equipment: {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string | null;
    };
    definition_id: string;
    source_file: string;
    field_path: string;
    operation: "Store" | "Release";
    capacity: number;
    stored_before: number;
    stored_after: number;
    incoming_overheal: number;
    actual_healed: number;
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_shared_cohort?: null | {
    time_ms: number;
    employee: string;
    definition: string;
    unit: string;
    next_spawn_ms: number;
    provider_count: number;
    fragment_recipient: string | null;
    equipment_copy: string | null;
    initial_hp: number;
    source_file: string | null;
    field_path: string | null;
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_weapon_stage?: null | {
    actor: string;
    target: string;
    equipment: {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string | null;
    };
    stage: "Base" | "Upgraded";
    time_ms: number;
    interval_ms: number;
    windup_ms: number;
    push_distance_units: number;
    actual_hp_loss: number;
    from_data_units: [number, number];
    to_data_units: [number, number];
    definition_id: string;
    source_file: string;
    field_path: string;
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_employee_transformation?: null | {
    actor: string;
    employee_uuid: string;
    successor: string;
    equipment: {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string;
    };
    time_ms: number;
    position: { x: number; y: number };
    form: string;
    definition_id: string;
    source_file: string;
    field_path: string;
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_last_will?: null | {
    actor: string;
    equipment: {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string;
    };
    time_ms: number;
    center: { x: number; y: number };
    facing: "up" | "right" | "down" | "left";
    hp_before: number;
    hp_protected: number;
    hp_after: number;
    attack: number;
    base_damage: number;
    targets: number;
    incapacitated: boolean;
    definition_id: string;
    source_file: string;
    field_path: string;
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_equipment_death_burst?: null | {
    time_ms: number;
    id: number;
    actor: string;
    equipment_copy: string;
    victim: string;
    center: { x: number; y: number };
    due_ms: number;
    attack: number;
    base_damage: number;
    operation: "Reserved" | "Detonated" | "Cancelled";
    targets: number;
    pending: number;
    definition_id: string;
    source_file: string;
    field_path: string;
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_equipment_form?: null | {
    time_ms: number;
    actor: string;
    equipment_copy: string;
    operation: "Earned" | "Confirmed" | "Discarded" | "Equipped";
    pending: number;
    confirmed: number;
    form: {
      index: number;
      progress: number;
      wins_required: number;
      name: string;
      description: string;
      appearance: string;
      effect:
        | { MaxHealth: { amount: number } }
        | { Attack: { amount: number } }
        | { IncomingDamage: { percent: number } };
    };
    definition_id: string;
    source_file: string;
    field_path: string;
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_bounty?: null | {
    time_ms: number;
    actor: string | null;
    equipment_copy: string;
    operation: "Earned" | "Confirmed" | "Discarded" | "Equipped";
    pending: number;
    confirmed: number;
    tier_attack: number;
    next_threshold: number | null;
    actual_attack: number | null;
    definition_id: string;
    source_file: string;
    field_path: string;
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_magazine?: null | {
    time_ms: number;
    actor: string;
    equipment: {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string | null;
    };
    definition_id: string;
    source_file: string;
    field_path: string;
    operation: "Fired" | "Reloaded" | "Reward";
    enhanced: boolean;
    remaining: number;
    capacity: number;
    reload_at_ms: number | null;
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_deferred_damage?: null | {
    time_ms: number;
    target: string;
    equipment: {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string | null;
    };
    definition_id: string;
    source_file: string;
    field_path: string;
    operation: "Store" | "Repay" | "ClearOnKill";
    amount: number;
    actual_hp_loss: number;
    remaining: number;
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_basic_heal_echo?: null | {
    time_ms: number;
    actor: string;
    equipment: {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string | null;
    };
    definition_id: string;
    source_file: string;
    field_path: string;
    actual_other_healed: number;
    actual_heal_percent: number;
    max_hp_cap_percent: number;
    max_hp: number;
    requested_self_heal: number;
    actual_self_healed: number;
    healing_source: { actor: string; action_ordinal: number };
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_heal_barrier?: null | {
    time_ms: number;
    actor: string;
    target: string;
    equipment: {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string | null;
    };
    definition_id: string;
    source_file: string;
    field_path: string;
    hp_before: number;
    max_hp: number;
    actual_healed: number;
    base_capacity: number;
    increased_capacity: number;
    remaining: number;
    expires_at_ms: number;
    ready_at_ms: number;
    kept_existing: boolean;
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_adaptive_shield?: null | {
    actor: string;
    primary: string;
    equipment: {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string | null;
    };
    damage_type: "Physical" | "Magic";
    max_hp: number;
    capacity: number;
    activation: number;
    time_ms: number;
    expires_at_ms: number;
    ready_at_ms: number;
    definition_id: string;
    source_file: string;
    field_path: string;
    attack_source: { actor: string; action_ordinal: number };
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_wearer_reduction?: null | {
    target: string;
    equipment: {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string | null;
    };
    time_ms: number;
    strike: number;
    current_hp: number;
    max_hp: number;
    percent: number;
    definition_id: string;
    source_file: string;
    field_path: string;
    incoming: { actor: string | null; action_ordinal: number | null };
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_incoming_flat?: null | {
    target: string;
    equipment: {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string | null;
    };
    time_ms: number;
    strike: number;
    defense: number;
    allowance: number;
    before: number;
    used: number;
    remaining: number;
    definition_id: string;
    source_file: string;
    field_path: string;
    incoming: { actor: string; action_ordinal: number | null };
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_shield_beneficiary_damage?: null | {
    actor: string;
    beneficiary: string;
    target: string;
    provider: string;
    equipment: {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string | null;
    };
    shield_activation: number;
    shield_remaining: number;
    percent: number;
    time_ms: number;
    definition_id: string;
    source_file: string;
    field_path: string;
    shield_source: { actor: string | null; action_ordinal: number | null };
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_received_heal?: null | {
    target: string;
    equipment: {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string | null;
    };
    time_ms: number;
    definition_id: string;
    source_file: string;
    field_path: string;
    incoming: { actor: string | null; action_ordinal: number | null };
    battle_terminal: boolean;
    origin: SourceTrace | null;
    actor: string | null;
    blocked_hostiles: number;
    percent: number;
    other_percent: number;
    before: number;
    after: number;
  };
  latest_control_duration?: null | {
    target: string;
    equipment: {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string | null;
    };
    time_ms: number;
    definition_id: string;
    source_file: string;
    field_path: string;
    incoming: { actor: string | null; action_ordinal: number | null };
    battle_terminal: boolean;
    origin: SourceTrace | null;
    caster: string;
    kind:
      "Stun" | "Freeze" | "Root" | "Slow" | "Silence" | "Taunt" | "BasicLock";
    reduction_percent: number;
    original_ms: number;
    effective_ms: number;
  };
  latest_channel_opening?: null | {
    target: string;
    equipment: {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string | null;
    };
    time_ms: number;
    started_at_ms: number;
    window_end_ms: number;
    reduction_percent: number;
    definition_id: string;
    source_file: string;
    field_path: string;
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_skill_barrier?: null | {
    target: string;
    equipment: {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string | null;
    };
    time_ms: number;
    ready_at_ms: number;
    definition_id: string;
    source_file: string;
    field_path: string;
    incoming: {
      cast: { skill_id: string; caster_instance_id: string; cast_seq: number };
    };
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  latest_basic_evasion?: null | {
    target: string;
    equipment: {
      definition_uuid: string;
      runtime_item_instance_id: string;
      persistent_copy_uuid: string | null;
    };
    time_ms: number;
    strike: number;
    chance_percent: number;
    roll_percent: number;
    evaded: boolean;
    definition_id: string;
    source_file: string;
    field_path: string;
    incoming: { actor: string };
    battle_terminal: boolean;
    origin: SourceTrace | null;
  };
  failure: null | { stage: string; diagnostic: unknown };
};
type ObservedStats = {
  attack: number;
  defense: number;
  move_speed_units_per_ms?: number;
};
const record = (value: unknown): value is Record<string, unknown> =>
  !!value && typeof value === "object" && !Array.isArray(value);
const strings = (value: unknown): value is string[] =>
  Array.isArray(value) && value.every((item) => typeof item === "string");
const natural = (value: unknown) =>
  Number.isSafeInteger(value) && (value as number) >= 0;
const sourceTrace = (value: unknown) =>
  value === null ||
  (record(value) &&
    typeof value.definition === "string" &&
    strings(value.calls) &&
    strings(value.inputs));
const observedStats = (value: unknown) =>
  value === null ||
  (record(value) &&
    natural(value.attack) &&
    Number.isSafeInteger(value.defense) &&
    (value.move_speed_units_per_ms === undefined ||
      natural(value.move_speed_units_per_ms)));
const areaMembership = (value: unknown) =>
  value === null ||
  (record(value) && natural(value.members) && natural(value.contributions));
export function parseTrialConfig(text: string): TrialConfig {
  if (new TextEncoder().encode(text).length > 8 * 1024 * 1024)
    throw new Error("실행 설정 파일은 8MB 이하여야 합니다.");
  const value: unknown = JSON.parse(text);
  if (
    !record(value) ||
    typeof value.root !== "string" ||
    !value.root.trim() ||
    !record(value.selection) ||
    !strings(value.selection.packs) ||
    !Array.isArray(value.selection.winners) ||
    !record(value.scenario)
  )
    throw new Error(
      "콘텐츠 폴더(root), 선택(selection), 시나리오(scenario)가 있는 실행 설정을 선택하세요.",
    );
  return value as TrialConfig;
}
export async function runDefinition(
  definition: Definition,
  config: TrialConfig,
  requestId: number,
): Promise<RunResponse> {
  const scenario = JSON.stringify(config.scenario);
  const scenarioDigest = await digest(scenario);
  const input = {
    source: definition.ticket.source,
    epoch: definition.ticket.epoch,
    revision: definition.ticket.revision,
    request_id: requestId,
    artifact_digest: definition.artifactDigest,
    root: config.root,
    selection: config.selection,
    scenario,
    lab: config.lab ?? null,
  };
  let result: unknown;
  if (isTauri()) result = await invoke("run_composition", { input });
  else {
    const response = await fetch("/__content/run-composition", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Content-Assembler": "compile",
      },
      body: JSON.stringify(input),
    });
    if (
      !response.ok ||
      !response.headers.get("content-type")?.includes("application/json")
    )
      throw new Error("Core 실행기에 연결하지 못했습니다.");
    result = await response.json();
  }
  if (record(result) && typeof result.error === "string")
    throw new Error(result.error);
  if (
    !record(result) ||
    result.epoch !== input.epoch ||
    result.revision !== input.revision ||
    result.request_id !== requestId ||
    result.source_digest !== definition.sourceDigest ||
    result.artifact_digest !== definition.artifactDigest ||
    !["completed", "rejected", "failed"].includes(String(result.status)) ||
    !Number.isSafeInteger(result.accepted_steps) ||
    (result.accepted_steps as number) < 0
  )
    throw new Error("실행 결과가 요청한 원본과 정의에 일치하지 않습니다.");
  if (
    result.identity !== null &&
    (!record(result.identity) ||
      typeof result.identity.run_id !== "string" ||
      result.identity.source_digest !== definition.sourceDigest ||
      result.identity.artifact_digest !== definition.artifactDigest ||
      typeof result.identity.artifact_path !== "string" ||
      result.identity.scenario_digest !== scenarioDigest ||
      !record(result.identity.content) ||
      typeof result.identity.content.semantic_sha256 !== "string")
  )
    throw new Error("실행 인스턴스의 버전 정보가 올바르지 않습니다.");
  if (
    (result.status === "completed" &&
      (result.identity === null || result.failure !== null)) ||
    (result.status !== "completed" &&
      (!record(result.failure) || typeof result.failure.stage !== "string"))
  )
    throw new Error("실행 결과와 오류 상태가 일치하지 않습니다.");
  const effect = result.latest_effect;
  if (
    effect !== null &&
    (!record(effect) ||
      !Number.isSafeInteger(effect.time_ms) ||
      !Number.isSafeInteger(effect.hp_delta) ||
      (effect.skill_id !== null && typeof effect.skill_id !== "string") ||
      !["definition_id", "source_file", "field_path", "target_unit"].every(
        (key) => typeof effect[key] === "string",
      ) ||
      (effect.equipment !== null &&
        (!record(effect.equipment) ||
          typeof effect.equipment.definition_uuid !== "string" ||
          typeof effect.equipment.runtime_item_instance_id !== "string" ||
          (effect.equipment.persistent_copy_uuid !== null &&
            typeof effect.equipment.persistent_copy_uuid !== "string"))) ||
      (effect.hp_reason != null &&
        (typeof effect.hp_reason !== "string" ||
          ![
            "Corrosion",
            "HealthCost",
            "BasicAttack",
            "Command",
            "Rest",
            "FieldWork",
          ].includes(effect.hp_reason))) ||
      (effect.origin !== null &&
        (!record(effect.origin) ||
          typeof effect.origin.definition !== "string" ||
          !strings(effect.origin.calls) ||
          !strings(effect.origin.inputs))))
  )
    throw new Error("실행 관측의 형식이 올바르지 않습니다.");
  const resonance = result.latest_resonance;
  if (
    resonance != null &&
    (!record(resonance) ||
      ![
        resonance.time_ms,
        resonance.before,
        resonance.after,
        resonance.max,
      ].every(natural) ||
      (resonance.max as number) < 1 ||
      (resonance.before as number) > (resonance.max as number) ||
      (resonance.after as number) > (resonance.max as number) ||
      typeof resonance.battle_terminal !== "boolean" ||
      (resonance.skill_id !== null && typeof resonance.skill_id !== "string") ||
      !["definition_id", "source_file", "field_path", "target_unit"].every(
        (key) => typeof resonance[key] === "string",
      ) ||
      !sourceTrace(resonance.origin) ||
      (resonance.equipment !== null &&
        (!record(resonance.equipment) ||
          typeof resonance.equipment.definition_uuid !== "string" ||
          typeof resonance.equipment.runtime_item_instance_id !== "string" ||
          (resonance.equipment.persistent_copy_uuid !== null &&
            typeof resonance.equipment.persistent_copy_uuid !== "string"))))
  )
    throw new Error("공명 관측의 형식이 올바르지 않습니다.");
  const hitCounter = result.latest_hit_counter;
  if (hitCounter != null) {
    if (
      !record(hitCounter) ||
      !["definition_id", "source_file", "field_path", "target_unit"].every(
        (key) => typeof hitCounter[key] === "string",
      ) ||
      ![
        hitCounter.time_ms,
        hitCounter.before,
        hitCounter.added,
        hitCounter.after,
        hitCounter.threshold,
      ].every(natural) ||
      Number(hitCounter.threshold) < 1 ||
      Number(hitCounter.threshold) > 1000 ||
      Number(hitCounter.before) >= Number(hitCounter.threshold) ||
      Number(hitCounter.after) >= Number(hitCounter.threshold) ||
      Number(hitCounter.added) < 1 ||
      Number(hitCounter.added) > Number(hitCounter.threshold) ||
      (hitCounter.control !== "Stun" && hitCounter.control !== "Root") ||
      typeof hitCounter.triggered !== "boolean" ||
      typeof hitCounter.control_applied !== "boolean" ||
      typeof hitCounter.battle_terminal !== "boolean" ||
      (hitCounter.skill_id !== null &&
        typeof hitCounter.skill_id !== "string") ||
      (hitCounter.cast_sequence !== null &&
        !natural(hitCounter.cast_sequence)) ||
      !sourceTrace(hitCounter.origin) ||
      (hitCounter.equipment !== null &&
        (!record(hitCounter.equipment) ||
          typeof hitCounter.equipment.definition_uuid !== "string" ||
          typeof hitCounter.equipment.runtime_item_instance_id !== "string" ||
          (hitCounter.equipment.persistent_copy_uuid !== null &&
            typeof hitCounter.equipment.persistent_copy_uuid !== "string"))) ||
      (Number(hitCounter.after) === 0
        ? hitCounter.counter_expires_at_ms !== null
        : !natural(hitCounter.counter_expires_at_ms) ||
          Number(hitCounter.counter_expires_at_ms) <=
            Number(hitCounter.time_ms)) ||
      (hitCounter.control_applied
        ? !hitCounter.triggered ||
          !natural(hitCounter.control_expires_at_ms) ||
          Number(hitCounter.control_expires_at_ms) <= Number(hitCounter.time_ms)
        : hitCounter.control_expires_at_ms !== null) ||
      (hitCounter.area_source != null &&
        (!record(hitCounter.area_source) ||
          !["area_id", "skill_id", "step_id", "definition"].every(
            (key) =>
              typeof (hitCounter.area_source as Record<string, unknown>)[
                key
              ] === "string",
          ) ||
          !natural(hitCounter.area_source.cast_sequence) ||
          !natural(hitCounter.area_source.status_index) ||
          (hitCounter.area_source.modifier_source !== null &&
            (!record(hitCounter.area_source.modifier_source) ||
              !["copy_uuid", "definition", "rule"].every(
                (key) =>
                  typeof (
                    (hitCounter.area_source as Record<string, unknown>)
                      .modifier_source as Record<string, unknown>
                  )[key] === "string",
              )))))
    )
      throw new Error("적중 누적 관측의 형식이 올바르지 않습니다.");
  }
  const sustained = result.latest_sustained;
  if (
    sustained !== null &&
    (!record(sustained) ||
      (sustained.equipment != null &&
        (!record(sustained.equipment) ||
          typeof sustained.equipment.definition_uuid !== "string" ||
          typeof sustained.equipment.runtime_item_instance_id !== "string" ||
          (sustained.equipment.persistent_copy_uuid !== null &&
            typeof sustained.equipment.persistent_copy_uuid !== "string"))) ||
      typeof sustained.battle_terminal !== "boolean" ||
      ![sustained.time_ms, sustained.expires_at_ms, sustained.activation].every(
        natural,
      ) ||
      (sustained.cast_sequence !== null && !natural(sustained.cast_sequence)) ||
      (sustained.skill_id !== null && typeof sustained.skill_id !== "string") ||
      !["definition_id", "source_file", "field_path", "target_unit"].every(
        (key) => typeof sustained[key] === "string",
      ) ||
      (sustained.ended !== null &&
        !sustainedEndReasons.some((reason) => reason === sustained.ended)) ||
      (sustained.damage_taken_increase_percent != null &&
        (!natural(sustained.damage_taken_increase_percent) ||
          (sustained.damage_taken_increase_percent as number) < 1 ||
          (sustained.damage_taken_increase_percent as number) >
            2_147_483_647)) ||
      (sustained.basic_speed_reduction_percent != null &&
        (!natural(sustained.basic_speed_reduction_percent) ||
          (sustained.basic_speed_reduction_percent as number) < 1 ||
          (sustained.basic_speed_reduction_percent as number) > 99)) ||
      (sustained.target_state != null &&
        (!record(sustained.target_state) ||
          ![
            "damage_invulnerable",
            "basic_attack_locked",
            "active_skill_locked",
          ].every(
            (key) =>
              typeof (sustained.target_state as Record<string, unknown>)[
                key
              ] === "boolean",
          ) ||
          !natural(sustained.target_state.received_heal_percent) ||
          (sustained.target_state.received_heal_percent as number) > 10000)) ||
      !observedStats(sustained.before) ||
      !observedStats(sustained.after) ||
      !sourceTrace(sustained.origin) ||
      (sustained.area_source != null &&
        (!record(sustained.area_source) ||
          !["area_id", "skill_id", "step_id", "definition"].every(
            (key) =>
              typeof (sustained.area_source as Record<string, unknown>)[key] ===
              "string",
          ) ||
          !natural(sustained.area_source.cast_sequence) ||
          !natural(sustained.area_source.status_index) ||
          (sustained.area_source.modifier_source !== null &&
            (!record(sustained.area_source.modifier_source) ||
              !["copy_uuid", "definition", "rule"].every(
                (key) =>
                  typeof (
                    sustained.area_source as {
                      modifier_source: Record<string, unknown>;
                    }
                  ).modifier_source[key] === "string",
              ))))))
  )
    throw new Error("지속 효과 관측의 형식이 올바르지 않습니다.");
  const area = result.latest_area;
  if (
    area !== null &&
    (!record(area) ||
      typeof area.battle_terminal !== "boolean" ||
      ![
        area.ordinal,
        area.time_ms,
        area.cast_sequence,
        area.started_at_ms,
        area.expires_at_ms,
      ].every(natural) ||
      !["area_id", "skill_id", "step_id", "caster"].every(
        (key) => typeof area[key] === "string",
      ) ||
      !areaMembership(area.before) ||
      !areaMembership(area.after) ||
      !record(area.change) ||
      !["started", "reconciled", "ended"].includes(String(area.change.kind)) ||
      (area.change.kind === "ended" &&
        (area.after !== null ||
          !sustainedEndReasons.some(
            (reason) =>
              reason === (area.change as Record<string, unknown>).reason,
          ))) ||
      (area.change.kind === "started" && area.before !== null) ||
      (area.change.kind === "reconciled" &&
        (area.before === null || area.after === null)) ||
      !sourceTrace(area.origin) ||
      (area.origin === null
        ? area.source_file !== null || area.field_path !== null
        : typeof area.source_file !== "string" ||
          typeof area.field_path !== "string"))
  )
    throw new Error("장판 관측의 형식이 올바르지 않습니다.");
  const waiting = result.latest_ready_wait;
  if (waiting != null) {
    const text = (value: unknown) =>
      typeof value === "string" && value.length > 0;
    const rate = (value: unknown) =>
      natural(value) && Number(value) >= 1 && Number(value) <= 10000;
    if (
      !record(waiting) ||
      !text(waiting.actor) ||
      !text(waiting.skill_id) ||
      !natural(waiting.time_ms) ||
      !natural(waiting.percent) ||
      Number(waiting.percent) > 2147483647 ||
      typeof waiting.battle_terminal !== "boolean" ||
      !Array.isArray(waiting.contributors) ||
      waiting.contributors.length === 0 ||
      !Array.isArray(waiting.origins) ||
      waiting.origins.length !== waiting.contributors.length ||
      !waiting.origins.every(sourceTrace) ||
      !waiting.contributors.every(
        (item: unknown) =>
          record(item) &&
          natural(item.waited_ms) &&
          Number(item.waited_ms) <= Number(waiting.time_ms) &&
          rate(item.per_second_percent) &&
          rate(item.max_percent) &&
          natural(item.percent) &&
          natural(item.until_ms) &&
          Number(item.until_ms) > Number(waiting.time_ms) &&
          Number(item.until_ms) - Number(waiting.time_ms) <= 4294967295 &&
          [item.definition_id, item.source_file, item.field_path].every(text) &&
          record(item.equipment) &&
          [
            item.equipment.definition_uuid,
            item.equipment.runtime_item_instance_id,
            item.equipment.persistent_copy_uuid,
          ].every(text),
      )
    )
      throw new Error("시전 대기 피해 증가 관측의 형식이 올바르지 않습니다.");
    const items = waiting.contributors as {
      waited_ms: number;
      per_second_percent: number;
      max_percent: number;
      percent: number;
      source_file: string;
    }[];
    if (
      items.some((item) => {
        const linear =
          (BigInt(item.waited_ms) * BigInt(item.per_second_percent)) / 1000n;
        return (
          BigInt(item.percent) !==
          (linear < BigInt(item.max_percent)
            ? linear
            : BigInt(item.max_percent))
        );
      }) ||
      items.reduce((sum, item) => sum + item.percent, 0) !== waiting.percent
    )
      throw new Error("시전 대기 피해 증가 관측의 기여량이 일치하지 않습니다.");
    if (
      items.some(
        (item, index) =>
          (waiting.origins as unknown[])[index] !== null &&
          (!record(result.identity) ||
            item.source_file !== result.identity.artifact_path),
      )
    )
      throw new Error(
        "시전 대기 피해 증가 관측의 원본이 실행 세대에 일치하지 않습니다.",
      );
  }
  const nearby = result.latest_nearby_defense;
  if (nearby != null) {
    const text = (value: unknown) =>
      typeof value === "string" && value.length > 0;
    const positiveI32 = (value: unknown) =>
      natural(value) && Number(value) > 0 && Number(value) <= 2147483647;
    if (
      !record(nearby) ||
      !text(nearby.actor) ||
      !natural(nearby.time_ms) ||
      !natural(nearby.bonus) ||
      Number(nearby.bonus) > 2147483647 ||
      !Number.isInteger(nearby.actual_defense) ||
      Number(nearby.actual_defense) < -2147483648 ||
      Number(nearby.actual_defense) > 2147483647 ||
      typeof nearby.battle_terminal !== "boolean" ||
      !Array.isArray(nearby.contributors) ||
      nearby.contributors.length === 0 ||
      !Array.isArray(nearby.origins) ||
      nearby.origins.length !== nearby.contributors.length ||
      !nearby.origins.every(sourceTrace) ||
      !nearby.contributors.every(
        (item: unknown) =>
          record(item) &&
          natural(item.count) &&
          Number(item.count) <= 4294967295 &&
          positiveI32(item.per_enemy) &&
          positiveI32(item.max_bonus) &&
          natural(item.bonus) &&
          [item.definition_id, item.source_file, item.field_path].every(text) &&
          record(item.equipment) &&
          [
            item.equipment.definition_uuid,
            item.equipment.runtime_item_instance_id,
            item.equipment.persistent_copy_uuid,
          ].every(text),
      )
    )
      throw new Error("주변 적 방어력 관측의 형식이 올바르지 않습니다.");
    const contributors = nearby.contributors as {
      count: number;
      per_enemy: number;
      max_bonus: number;
      bonus: number;
      source_file: string;
    }[];
    if (
      contributors.some(
        (item) =>
          BigInt(item.bonus) !==
          (BigInt(item.count) * BigInt(item.per_enemy) < BigInt(item.max_bonus)
            ? BigInt(item.count) * BigInt(item.per_enemy)
            : BigInt(item.max_bonus)),
      ) ||
      contributors.reduce((sum, item) => sum + item.bonus, 0) !== nearby.bonus
    )
      throw new Error("주변 적 방어력 관측의 기여량이 일치하지 않습니다.");
    if (
      contributors.some(
        (item, index) =>
          (nearby.origins as unknown[])[index] !== null &&
          (!record(result.identity) ||
            item.source_file !== result.identity.artifact_path),
      )
    )
      throw new Error(
        "주변 적 방어력 관측의 원본이 실행 세대에 일치하지 않습니다.",
      );
  }
  const areaDamage = result.latest_area_damage;
  if (areaDamage != null) {
    const text = (value: unknown) =>
      typeof value === "string" && value.length > 0;
    if (
      !record(areaDamage) ||
      !text(areaDamage.actor) ||
      typeof areaDamage.battle_terminal !== "boolean" ||
      ![areaDamage.time_ms, areaDamage.target_count, areaDamage.percent].every(
        natural,
      ) ||
      (areaDamage.action !== null && !natural(areaDamage.action)) ||
      !Array.isArray(areaDamage.contributors) ||
      areaDamage.contributors.length === 0 ||
      !Array.isArray(areaDamage.origins) ||
      areaDamage.origins.length !== areaDamage.contributors.length ||
      !areaDamage.origins.every(sourceTrace) ||
      !areaDamage.contributors.every(
        (item: unknown) =>
          record(item) &&
          natural(item.percent) &&
          Number(item.percent) <= 10000 &&
          [item.definition_id, item.source_file, item.field_path].every(text) &&
          record(item.equipment) &&
          [
            item.equipment.definition_uuid,
            item.equipment.runtime_item_instance_id,
            item.equipment.persistent_copy_uuid,
          ].every(text),
      )
    )
      throw new Error("범위 피해 관측의 형식이 올바르지 않습니다.");
    const contributors = areaDamage.contributors as {
      percent: number;
      source_file: string;
    }[];
    if (
      contributors.reduce((sum, item) => sum + item.percent, 0) !==
      areaDamage.percent
    )
      throw new Error("범위 피해 관측의 합계가 일치하지 않습니다.");
    if (
      contributors.some(
        (item, index) =>
          (areaDamage.origins as unknown[])[index] !== null &&
          (!record(result.identity) ||
            item.source_file !== result.identity.artifact_path),
      )
    )
      throw new Error("범위 피해 관측의 원본이 실행 세대에 일치하지 않습니다.");
  }
  const healReaction = result.latest_actual_heal_reaction;
  if (healReaction != null) {
    const text = (v: unknown) => typeof v === "string" && v.length > 0;
    const unsigned = (v: unknown) => natural(v) && Number(v) <= 0xffffffff;
    const positive = (v: unknown, max: number) =>
      natural(v) && Number(v) > 0 && Number(v) <= max;
    if (
      !record(healReaction) ||
      !natural(healReaction.time_ms) ||
      !text(healReaction.actor) ||
      !text(healReaction.target) ||
      healReaction.actor === healReaction.target ||
      !positive(healReaction.actual_healed, 0xffffffff) ||
      typeof healReaction.battle_terminal !== "boolean" ||
      !record(healReaction.healing_source) ||
      !(
        healReaction.healing_source.actor === null ||
        text(healReaction.healing_source.actor)
      ) ||
      !(
        healReaction.healing_source.action_ordinal == null ||
        natural(healReaction.healing_source.action_ordinal)
      ) ||
      !Array.isArray(healReaction.contributors) ||
      healReaction.contributors.length === 0 ||
      !Array.isArray(healReaction.origins) ||
      healReaction.origins.length !== healReaction.contributors.length ||
      !healReaction.origins.every(sourceTrace)
    )
      throw new Error("실제 치유 반응 관측의 형식이 올바르지 않습니다.");
    const copies = new Set<string>();
    for (const [index, item] of healReaction.contributors.entries()) {
      if (
        !record(item) ||
        ![item.definition_id, item.source_file, item.field_path].every(text) ||
        !record(item.equipment) ||
        ![
          item.equipment.definition_uuid,
          item.equipment.runtime_item_instance_id,
        ].every(text) ||
        !(
          item.equipment.persistent_copy_uuid === null ||
          text(item.equipment.persistent_copy_uuid)
        ) ||
        (item.self_shield == null && item.target_haste == null)
      )
        throw new Error("실제 치유 반응의 장비 기여가 올바르지 않습니다.");
      const copy = String(item.equipment.runtime_item_instance_id);
      if (copies.has(copy))
        throw new Error("실제 치유 반응에 같은 장비 사본이 중복되었습니다.");
      copies.add(copy);
      if (item.self_shield !== null) {
        const shield = item.self_shield;
        if (
          !record(shield) ||
          !positive(shield.actual_heal_percent, 10000) ||
          !positive(shield.max_hp_cap_percent, 100) ||
          ![shield.before, shield.after, shield.cap].every(unsigned) ||
          Number(shield.before) > Number(shield.after) ||
          Number(shield.after) > Number(shield.cap) ||
          ![shield.remainder_before, shield.remainder_after].every(
            (v) => natural(v) && Number(v) < 100,
          ) ||
          (shield.after === shield.cap && shield.remainder_after !== 0) ||
          !natural(shield.expires_at_ms) ||
          Number(shield.expires_at_ms) <= Number(healReaction.time_ms)
        )
          throw new Error("실제 치유 반응의 보호막 수치가 올바르지 않습니다.");
      }
      if (item.target_haste !== null) {
        const haste = item.target_haste;
        if (
          !record(haste) ||
          !positive(haste.attack_speed_points, 10000) ||
          typeof haste.refreshed !== "boolean" ||
          !natural(haste.expires_at_ms) ||
          Number(haste.expires_at_ms) <= Number(healReaction.time_ms)
        )
          throw new Error(
            "실제 치유 반응의 공격 속도 수치가 올바르지 않습니다.",
          );
      }
      if (
        healReaction.origins[index] !== null &&
        (!record(result.identity) ||
          item.source_file !== result.identity.artifact_path)
      )
        throw new Error(
          "실제 치유 반응의 원본이 실행 세대에 일치하지 않습니다.",
        );
    }
  }
  const resonanceGain = result.latest_resonance_gain;
  if (resonanceGain != null) {
    const text = (value: unknown) =>
      typeof value === "string" && value.length > 0;
    const unsigned = (value: unknown) =>
      natural(value) && Number(value) <= 0xffffffff;
    if (
      !record(resonanceGain) ||
      !text(resonanceGain.actor) ||
      !["automatic", "authored"].includes(String(resonanceGain.channel)) ||
      typeof resonanceGain.battle_terminal !== "boolean" ||
      ![resonanceGain.time_ms, resonanceGain.percent].every(natural) ||
      ![
        resonanceGain.base_amount,
        resonanceGain.increased_amount,
        resonanceGain.actual_gain,
        resonanceGain.before,
        resonanceGain.after,
        resonanceGain.max,
      ].every(unsigned) ||
      ![resonanceGain.remainder_before, resonanceGain.remainder_after].every(
        (v) => natural(v) && Number(v) < 100,
      ) ||
      !Array.isArray(resonanceGain.contributors) ||
      resonanceGain.contributors.length === 0 ||
      !Array.isArray(resonanceGain.origins) ||
      resonanceGain.origins.length !== resonanceGain.contributors.length ||
      !resonanceGain.origins.every(sourceTrace) ||
      !resonanceGain.contributors.every(
        (item: unknown) =>
          record(item) &&
          natural(item.percent) &&
          Number(item.percent) > 0 &&
          Number(item.percent) <= 10000 &&
          [item.definition_id, item.source_file, item.field_path].every(text) &&
          record(item.equipment) &&
          [
            item.equipment.definition_uuid,
            item.equipment.runtime_item_instance_id,
          ].every(text) &&
          (item.equipment.persistent_copy_uuid === null ||
            text(item.equipment.persistent_copy_uuid)),
      )
    )
      throw new Error("공명 획득 관측의 형식이 올바르지 않습니다.");
    const contributors = resonanceGain.contributors as {
      percent: number;
      source_file: string;
    }[];
    const before = Number(resonanceGain.before),
      after = Number(resonanceGain.after),
      max = Number(resonanceGain.max);
    if (
      Number(resonanceGain.base_amount) === 0 ||
      Number(resonanceGain.increased_amount) <
        Number(resonanceGain.base_amount) ||
      before >= max ||
      after <= before ||
      after !==
        Math.min(max, before + Number(resonanceGain.increased_amount)) ||
      Number(resonanceGain.actual_gain) !== after - before ||
      (after === max && resonanceGain.remainder_after !== 0) ||
      contributors.reduce((sum, item) => sum + item.percent, 0) !==
        resonanceGain.percent
    )
      throw new Error("공명 획득 관측의 수치가 일치하지 않습니다.");
    if (
      contributors.some(
        (item, index) =>
          (resonanceGain.origins as unknown[])[index] !== null &&
          (!record(result.identity) ||
            item.source_file !== result.identity.artifact_path),
      )
    )
      throw new Error("공명 획득 관측의 원본이 실행 세대에 일치하지 않습니다.");
  }
  const nearbyAllyDamage = result.latest_nearby_ally_damage;
  if (nearbyAllyDamage != null) {
    const text = (value: unknown) =>
      typeof value === "string" && value.length > 0;
    if (
      !record(nearbyAllyDamage) ||
      !text(nearbyAllyDamage.actor) ||
      !text(nearbyAllyDamage.wearer) ||
      !text(nearbyAllyDamage.target) ||
      nearbyAllyDamage.wearer === nearbyAllyDamage.target ||
      typeof nearbyAllyDamage.battle_terminal !== "boolean" ||
      ![nearbyAllyDamage.time_ms, nearbyAllyDamage.percent].every(natural) ||
      !Array.isArray(nearbyAllyDamage.contributors) ||
      nearbyAllyDamage.contributors.length === 0 ||
      !Array.isArray(nearbyAllyDamage.origins) ||
      nearbyAllyDamage.origins.length !==
        nearbyAllyDamage.contributors.length ||
      !nearbyAllyDamage.origins.every(sourceTrace) ||
      !nearbyAllyDamage.contributors.every(
        (item: unknown) =>
          record(item) &&
          natural(item.count) &&
          natural(item.percent) &&
          Number(item.percent) <= 10000 &&
          [item.definition_id, item.source_file, item.field_path].every(text) &&
          record(item.equipment) &&
          [
            item.equipment.definition_uuid,
            item.equipment.runtime_item_instance_id,
            item.equipment.persistent_copy_uuid,
          ].every(text),
      )
    )
      throw new Error("주변 아군 피해 관측의 형식이 올바르지 않습니다.");
    const contributors = nearbyAllyDamage.contributors as {
      percent: number;
      source_file: string;
    }[];
    if (
      contributors.reduce((sum, item) => sum + item.percent, 0) !==
      nearbyAllyDamage.percent
    )
      throw new Error("주변 아군 피해 관측의 합계가 일치하지 않습니다.");
    if (
      contributors.some(
        (item, index) =>
          (nearbyAllyDamage.origins as unknown[])[index] !== null &&
          (!record(result.identity) ||
            item.source_file !== result.identity.artifact_path),
      )
    )
      throw new Error(
        "주변 아군 피해 관측의 원본이 실행 세대에 일치하지 않습니다.",
      );
  }
  for (const [kind, conditionalDamage] of [
    ["increase", result.latest_conditional_damage],
    ["reduction", result.latest_conditional_reduction],
  ] as const) {
    if (conditionalDamage != null) {
      const text = (value: unknown) =>
        typeof value === "string" && value.length > 0;
      if (
        !record(conditionalDamage) ||
        !text(conditionalDamage.actor) ||
        !text(conditionalDamage.wearer) ||
        !text(conditionalDamage.target) ||
        conditionalDamage.wearer === conditionalDamage.target ||
        typeof conditionalDamage.battle_terminal !== "boolean" ||
        ![
          conditionalDamage.time_ms,
          conditionalDamage.percent,
          conditionalDamage.wearer_hp,
          conditionalDamage.wearer_max_hp,
          conditionalDamage.target_hp,
          conditionalDamage.target_max_hp,
        ].every(natural) ||
        Number(conditionalDamage.wearer_hp) <= 0 ||
        Number(conditionalDamage.target_hp) <= 0 ||
        Number(conditionalDamage.wearer_hp) >
          Number(conditionalDamage.wearer_max_hp) ||
        Number(conditionalDamage.target_hp) >
          Number(conditionalDamage.target_max_hp) ||
        !(
          conditionalDamage.target_movement_impaired === null ||
          typeof conditionalDamage.target_movement_impaired === "boolean"
        ) ||
        !(
          conditionalDamage.distance_squared == null ||
          (typeof conditionalDamage.distance_squared === "number" &&
            Number.isFinite(conditionalDamage.distance_squared) &&
            conditionalDamage.distance_squared >= 0)
        ) ||
        !Array.isArray(conditionalDamage.contributors) ||
        conditionalDamage.contributors.length === 0 ||
        !Array.isArray(conditionalDamage.origins) ||
        conditionalDamage.origins.length !==
          conditionalDamage.contributors.length ||
        !conditionalDamage.origins.every(sourceTrace) ||
        !conditionalDamage.contributors.every(
          (item: unknown) =>
            record(item) &&
            typeof item.matched === "boolean" &&
            (item.matched || item.percent === 0) &&
            natural(item.percent) &&
            Number(item.percent) <= (kind === "reduction" ? 99 : 10000) &&
            [item.definition_id, item.source_file, item.field_path].every(
              text,
            ) &&
            record(item.equipment) &&
            [
              item.equipment.definition_uuid,
              item.equipment.runtime_item_instance_id,
              item.equipment.persistent_copy_uuid,
            ].every(text),
        )
      )
        throw new Error("조건부 피해 관측의 형식이 올바르지 않습니다.");
      const contributors = conditionalDamage.contributors as {
        percent: number;
        source_file: string;
      }[];
      if (
        (kind === "reduction"
          ? Math.max(...contributors.map((item) => item.percent))
          : contributors.reduce((sum, item) => sum + item.percent, 0)) !==
        conditionalDamage.percent
      )
        throw new Error("조건부 피해 관측의 합계가 일치하지 않습니다.");
      if (kind === "reduction") {
        const chosen = conditionalDamage.equipment;
        const incoming = conditionalDamage.incoming;
        if (
          !record(chosen) ||
          !record(incoming) ||
          incoming.actor !== conditionalDamage.actor ||
          !(
            incoming.action_ordinal === null || natural(incoming.action_ordinal)
          ) ||
          !conditionalDamage.contributors.some(
            (entry: unknown) =>
              record(entry) &&
              record(entry.equipment) &&
              entry.percent === conditionalDamage.percent &&
              entry.equipment.definition_uuid === chosen.definition_uuid &&
              entry.equipment.runtime_item_instance_id ===
                chosen.runtime_item_instance_id &&
              entry.equipment.persistent_copy_uuid ===
                chosen.persistent_copy_uuid,
          )
        ) {
          throw new Error(
            "조건부 감면의 선택 사본 또는 공격 출처가 일치하지 않습니다.",
          );
        }
      }
      if (
        contributors.some(
          (item, index) =>
            (conditionalDamage.origins as unknown[])[index] !== null &&
            (!record(result.identity) ||
              item.source_file !== result.identity.artifact_path),
        )
      )
        throw new Error(
          "조건부 피해 관측의 원본이 실행 세대에 일치하지 않습니다.",
        );
    }
  }
  const primaryDamage = result.latest_primary_damage;
  if (primaryDamage != null) {
    const text = (value: unknown) =>
      typeof value === "string" && value.length > 0;
    if (
      !record(primaryDamage) ||
      !text(primaryDamage.actor) ||
      !text(primaryDamage.target) ||
      primaryDamage.actor === primaryDamage.target ||
      typeof primaryDamage.battle_terminal !== "boolean" ||
      ![
        primaryDamage.time_ms,
        primaryDamage.action,
        primaryDamage.target_current_hp,
        primaryDamage.target_max_hp,
        primaryDamage.percent,
      ].every(natural) ||
      Number(primaryDamage.target_current_hp) === 0 ||
      Number(primaryDamage.target_max_hp) <
        Number(primaryDamage.target_current_hp) ||
      !Array.isArray(primaryDamage.contributors) ||
      primaryDamage.contributors.length === 0 ||
      !Array.isArray(primaryDamage.origins) ||
      primaryDamage.origins.length !== primaryDamage.contributors.length ||
      !primaryDamage.origins.every(sourceTrace) ||
      !primaryDamage.contributors.every(
        (item: unknown) =>
          record(item) &&
          natural(item.percent) &&
          Number(item.percent) <= 10000 &&
          [item.definition_id, item.source_file, item.field_path].every(text) &&
          record(item.equipment) &&
          [
            item.equipment.definition_uuid,
            item.equipment.runtime_item_instance_id,
            item.equipment.persistent_copy_uuid,
          ].every(text),
      )
    )
      throw new Error("주 대상 피해 관측의 형식이 올바르지 않습니다.");
    const contributors = primaryDamage.contributors as {
      percent: number;
      source_file: string;
    }[];
    if (
      contributors.reduce((sum, item) => sum + item.percent, 0) !==
      primaryDamage.percent
    )
      throw new Error("주 대상 피해 관측의 합계가 일치하지 않습니다.");
    if (
      contributors.some(
        (item, index) =>
          (primaryDamage.origins as unknown[])[index] !== null &&
          (!record(result.identity) ||
            item.source_file !== result.identity.artifact_path),
      )
    )
      throw new Error(
        "주 대상 피해 관측의 원본이 실행 세대에 일치하지 않습니다.",
      );
  }
  const speed = result.latest_equipment_speed;
  if (speed != null) {
    if (
      !record(speed) ||
      typeof speed.actor !== "string" ||
      typeof speed.battle_terminal !== "boolean" ||
      ![
        speed.time_ms,
        speed.current_hp,
        speed.max_hp,
        speed.base_interval_ms,
        speed.final_interval_ms,
        speed.equipment_points,
      ].every(natural) ||
      Number(speed.max_hp) === 0 ||
      Number(speed.current_hp) > Number(speed.max_hp) ||
      Number(speed.base_interval_ms) === 0 ||
      Number(speed.final_interval_ms) === 0 ||
      !Array.isArray(speed.contributors) ||
      speed.contributors.length === 0 ||
      !Array.isArray(speed.origins) ||
      speed.origins.length !== speed.contributors.length ||
      !speed.origins.every(sourceTrace) ||
      !speed.contributors.every(
        (item: unknown) =>
          record(item) &&
          natural(item.points) &&
          ["definition_id", "source_file", "field_path"].every(
            (key) => typeof item[key] === "string",
          ) &&
          record(item.equipment) &&
          typeof item.equipment.definition_uuid === "string" &&
          typeof item.equipment.runtime_item_instance_id === "string" &&
          (item.equipment.persistent_copy_uuid === null ||
            typeof item.equipment.persistent_copy_uuid === "string"),
      )
    )
      throw new Error("장비 공속 관측의 형식이 올바르지 않습니다.");
    const contributors = speed.contributors as {
      points: number;
      source_file: string;
    }[];
    if (
      contributors.reduce((sum, item) => sum + item.points, 0) !==
      speed.equipment_points
    )
      throw new Error("장비 공속 관측의 합계가 일치하지 않습니다.");
    if (
      contributors.some(
        (item, index) =>
          (speed.origins as unknown[])[index] !== null &&
          (!record(result.identity) ||
            item.source_file !== result.identity.artifact_path),
      )
    )
      throw new Error("장비 공속 관측의 원본이 실행 세대에 일치하지 않습니다.");
  }
  const support = result.latest_equipment_support;
  if (support != null) {
    if (
      !record(support) ||
      typeof support.actor !== "string" ||
      typeof support.target !== "string" ||
      !["heal", "shield"].includes(String(support.channel)) ||
      typeof support.battle_terminal !== "boolean" ||
      ![
        support.time_ms,
        support.base_amount,
        support.increased_amount,
        support.percent,
      ].every(natural) ||
      !Array.isArray(support.contributors) ||
      support.contributors.length === 0 ||
      !Array.isArray(support.origins) ||
      support.origins.length !== support.contributors.length ||
      !support.origins.every(sourceTrace) ||
      !support.contributors.every(
        (item: unknown) =>
          record(item) &&
          natural(item.percent) &&
          Number(item.percent) <= 10000 &&
          ["definition_id", "source_file", "field_path"].every(
            (key) => typeof item[key] === "string",
          ) &&
          record(item.equipment) &&
          typeof item.equipment.definition_uuid === "string" &&
          typeof item.equipment.runtime_item_instance_id === "string" &&
          (item.equipment.persistent_copy_uuid === null ||
            typeof item.equipment.persistent_copy_uuid === "string"),
      )
    )
      throw new Error("장비 지원 관측의 형식이 올바르지 않습니다.");
    const contributors = support.contributors as {
      percent: number;
      source_file: string;
    }[];
    if (
      contributors.reduce((sum, item) => sum + item.percent, 0) !==
      support.percent
    )
      throw new Error("장비 지원 관측의 합계가 일치하지 않습니다.");
    if (
      contributors.some(
        (item, index) =>
          (support.origins as unknown[])[index] !== null &&
          (!record(result.identity) ||
            item.source_file !== result.identity.artifact_path),
      )
    )
      throw new Error("장비 지원 관측의 원본이 실행 세대에 일치하지 않습니다.");
  }
  const reserve = result.latest_overheal_reserve;
  if (reserve != null) {
    if (
      !record(reserve) ||
      !["target", "definition_id", "source_file", "field_path"].every(
        (key) => typeof reserve[key] === "string",
      ) ||
      ![
        "time_ms",
        "capacity",
        "stored_before",
        "stored_after",
        "incoming_overheal",
        "actual_healed",
      ].every((key) => natural(reserve[key])) ||
      typeof reserve.battle_terminal !== "boolean" ||
      !record(reserve.equipment) ||
      typeof reserve.equipment.definition_uuid !== "string" ||
      typeof reserve.equipment.runtime_item_instance_id !== "string" ||
      !(
        reserve.equipment.persistent_copy_uuid === null ||
        typeof reserve.equipment.persistent_copy_uuid === "string"
      ) ||
      (reserve.operation !== "Store" && reserve.operation !== "Release") ||
      !sourceTrace(reserve.origin)
    ) {
      throw new Error("초과 회복 저장 관측의 형식이 올바르지 않습니다.");
    }
    if (
      reserve.origin !== null &&
      (!record(result.identity) ||
        reserve.source_file !== result.identity.artifact_path)
    ) {
      throw new Error(
        "초과 회복 저장 관측의 원본이 실행 세대에 일치하지 않습니다.",
      );
    }
  }
  const cohort = result.latest_shared_cohort;
  if (cohort != null) {
    if (
      !record(cohort) ||
      !["employee", "definition", "unit"].every(
        (key) => typeof cohort[key] === "string",
      ) ||
      !["time_ms", "next_spawn_ms", "provider_count", "initial_hp"].every(
        (key) => natural(cohort[key]),
      ) ||
      ![
        "fragment_recipient",
        "equipment_copy",
        "source_file",
        "field_path",
      ].every(
        (key) => cohort[key] === null || typeof cohort[key] === "string",
      ) ||
      typeof cohort.battle_terminal !== "boolean" ||
      !sourceTrace(cohort.origin)
    ) {
      throw new Error("공유 사냥감 관측의 형식이 올바르지 않습니다.");
    }
    if (
      cohort.origin !== null &&
      (!record(result.identity) ||
        cohort.source_file !== result.identity.artifact_path ||
        typeof cohort.field_path !== "string")
    ) {
      throw new Error(
        "공유 사냥감 관측의 원본이 실행 세대에 일치하지 않습니다.",
      );
    }
  }
  const weaponStage = result.latest_weapon_stage;
  if (weaponStage != null) {
    const equipment = record(weaponStage) ? weaponStage.equipment : null;
    if (
      !record(weaponStage) ||
      !["actor", "target", "definition_id", "source_file", "field_path"].every(
        (key) => typeof weaponStage[key] === "string",
      ) ||
      ![
        "time_ms",
        "interval_ms",
        "windup_ms",
        "push_distance_units",
        "actual_hp_loss",
      ].every((key) => natural(weaponStage[key])) ||
      Number(weaponStage.interval_ms) === 0 ||
      Number(weaponStage.push_distance_units) > 32_000_000 ||
      !["Base", "Upgraded"].includes(String(weaponStage.stage)) ||
      !["from_data_units", "to_data_units"].every((key) => {
        const point = weaponStage[key];
        return (
          Array.isArray(point) &&
          point.length === 2 &&
          point.every(Number.isSafeInteger)
        );
      }) ||
      !record(equipment) ||
      !["definition_uuid", "runtime_item_instance_id"].every(
        (key) => typeof equipment[key] === "string",
      ) ||
      !(
        equipment.persistent_copy_uuid === null ||
        typeof equipment.persistent_copy_uuid === "string"
      ) ||
      typeof weaponStage.battle_terminal !== "boolean" ||
      !sourceTrace(weaponStage.origin)
    ) {
      throw new Error("무기 단계 관측의 형식이 올바르지 않습니다.");
    }
    if (
      weaponStage.origin !== null &&
      (!record(result.identity) ||
        weaponStage.source_file !== result.identity.artifact_path)
    ) {
      throw new Error("무기 단계 관측의 원본이 실행 세대에 일치하지 않습니다.");
    }
  }
  const transformation = result.latest_employee_transformation;
  if (transformation != null) {
    const position = record(transformation) ? transformation.position : null;
    const equipment = record(transformation) ? transformation.equipment : null;
    if (
      !record(transformation) ||
      ![
        "actor",
        "employee_uuid",
        "successor",
        "form",
        "definition_id",
        "source_file",
        "field_path",
      ].every(
        (key) =>
          typeof transformation[key] === "string" && transformation[key] !== "",
      ) ||
      transformation.actor === transformation.successor ||
      !natural(transformation.time_ms) ||
      !record(position) ||
      !["x", "y"].every((key) => Number.isSafeInteger(position[key])) ||
      !record(equipment) ||
      ![
        "definition_uuid",
        "runtime_item_instance_id",
        "persistent_copy_uuid",
      ].every((key) => typeof equipment[key] === "string") ||
      typeof transformation.battle_terminal !== "boolean" ||
      !sourceTrace(transformation.origin)
    ) {
      throw new Error("직원 변신 관측의 형식이 올바르지 않습니다.");
    }
    if (
      transformation.origin !== null &&
      (!record(result.identity) ||
        transformation.source_file !== result.identity.artifact_path)
    ) {
      throw new Error("직원 변신 관측의 원본이 실행 세대에 일치하지 않습니다.");
    }
  }
  const lastWill = result.latest_last_will;
  if (lastWill != null) {
    const center = record(lastWill) ? lastWill.center : null;
    const equipment = record(lastWill) ? lastWill.equipment : null;
    if (
      !record(lastWill) ||
      !["actor", "definition_id", "source_file", "field_path"].every(
        (key) => typeof lastWill[key] === "string",
      ) ||
      ![
        "time_ms",
        "hp_before",
        "hp_protected",
        "hp_after",
        "attack",
        "base_damage",
        "targets",
      ].every((key) => natural(lastWill[key])) ||
      Number(lastWill.hp_before) === 0 ||
      lastWill.hp_protected !== 1 ||
      lastWill.hp_after !== 0 ||
      lastWill.incapacitated !== true ||
      !record(center) ||
      !["x", "y"].every((key) => Number.isSafeInteger(center[key])) ||
      !["up", "right", "down", "left"].includes(String(lastWill.facing)) ||
      !record(equipment) ||
      ![
        "definition_uuid",
        "runtime_item_instance_id",
        "persistent_copy_uuid",
      ].every((key) => typeof equipment[key] === "string") ||
      typeof lastWill.battle_terminal !== "boolean" ||
      !sourceTrace(lastWill.origin)
    ) {
      throw new Error("마지막 유언 관측의 형식이 올바르지 않습니다.");
    }
    if (
      lastWill.origin !== null &&
      (!record(result.identity) ||
        lastWill.source_file !== result.identity.artifact_path)
    ) {
      throw new Error(
        "마지막 유언 관측의 원본이 실행 세대에 일치하지 않습니다.",
      );
    }
  }
  const deathBurst = result.latest_equipment_death_burst;
  if (deathBurst != null) {
    const center = record(deathBurst) ? deathBurst.center : null;
    if (
      !record(deathBurst) ||
      ![
        "actor",
        "equipment_copy",
        "victim",
        "definition_id",
        "source_file",
        "field_path",
      ].every((key) => typeof deathBurst[key] === "string") ||
      ![
        "time_ms",
        "id",
        "due_ms",
        "attack",
        "base_damage",
        "targets",
        "pending",
      ].every((key) => natural(deathBurst[key])) ||
      deathBurst.id === 0 ||
      !record(center) ||
      !["x", "y"].every((key) => Number.isSafeInteger(center[key])) ||
      typeof deathBurst.operation !== "string" ||
      !["Reserved", "Detonated", "Cancelled"].includes(deathBurst.operation) ||
      typeof deathBurst.battle_terminal !== "boolean" ||
      !sourceTrace(deathBurst.origin)
    ) {
      throw new Error("처치 연쇄 폭발 관측의 형식이 올바르지 않습니다.");
    }
    if (
      deathBurst.origin !== null &&
      (!record(result.identity) ||
        deathBurst.source_file !== result.identity.artifact_path)
    ) {
      throw new Error(
        "처치 연쇄 폭발 관측의 원본이 실행 세대에 일치하지 않습니다.",
      );
    }
  }
  const equipmentForm = result.latest_equipment_form;
  if (equipmentForm != null) {
    const form = record(equipmentForm) ? equipmentForm.form : null;
    const effect = record(form) ? form.effect : null;
    const effectKeys = record(effect) ? Object.keys(effect) : [];
    const effectKey = effectKeys[0];
    const amount =
      record(effect) && record(effect[effectKey]) ? effect[effectKey] : null;
    const magnitude =
      amount?.[effectKey === "IncomingDamage" ? "percent" : "amount"];
    if (
      !record(equipmentForm) ||
      ![
        "actor",
        "equipment_copy",
        "definition_id",
        "source_file",
        "field_path",
      ].every((key) => typeof equipmentForm[key] === "string") ||
      !["time_ms", "pending", "confirmed"].every((key) =>
        natural(equipmentForm[key]),
      ) ||
      (equipmentForm.pending as number) > 1 ||
      typeof equipmentForm.operation !== "string" ||
      !["Earned", "Confirmed", "Discarded", "Equipped"].includes(
        equipmentForm.operation,
      ) ||
      !record(form) ||
      !["index", "progress", "wins_required"].every((key) =>
        natural(form[key]),
      ) ||
      (form.index as number) >= 16 ||
      (form.wins_required === 0
        ? form.progress !== 0
        : (form.progress as number) >= (form.wins_required as number)) ||
      !["name", "description", "appearance"].every(
        (key) => typeof form[key] === "string" && form[key].length > 0,
      ) ||
      effectKeys.length !== 1 ||
      !["MaxHealth", "Attack", "IncomingDamage"].includes(effectKey) ||
      !natural(magnitude) ||
      magnitude === 0 ||
      (magnitude as number) > 2147483647 ||
      typeof equipmentForm.battle_terminal !== "boolean" ||
      !sourceTrace(equipmentForm.origin)
    )
      throw new Error("장비 형태 관측의 형식이 올바르지 않습니다.");
    if (
      equipmentForm.origin !== null &&
      (!record(result.identity) ||
        equipmentForm.source_file !== result.identity.artifact_path)
    ) {
      throw new Error("장비 형태 관측의 원본이 실행 세대에 일치하지 않습니다.");
    }
  }
  const bounty = result.latest_bounty;
  if (bounty != null) {
    if (
      !record(bounty) ||
      !["equipment_copy", "definition_id", "source_file", "field_path"].every(
        (key) => typeof bounty[key] === "string",
      ) ||
      !["time_ms", "pending", "confirmed", "tier_attack"].every((key) =>
        natural(bounty[key]),
      ) ||
      !["next_threshold", "actual_attack"].every(
        (key) => bounty[key] === null || natural(bounty[key]),
      ) ||
      !(bounty.actor === null || typeof bounty.actor === "string") ||
      typeof bounty.operation !== "string" ||
      !["Earned", "Confirmed", "Discarded", "Equipped"].includes(
        bounty.operation,
      ) ||
      typeof bounty.battle_terminal !== "boolean" ||
      !sourceTrace(bounty.origin)
    ) {
      throw new Error("수금 관측의 형식이 올바르지 않습니다.");
    }
    if (
      bounty.origin !== null &&
      (!record(result.identity) ||
        bounty.source_file !== result.identity.artifact_path)
    ) {
      throw new Error("수금 관측의 원본이 실행 세대에 일치하지 않습니다.");
    }
  }
  const magazine = result.latest_magazine;
  if (magazine != null) {
    if (
      !record(magazine) ||
      !["actor", "definition_id", "source_file", "field_path"].every(
        (key) => typeof magazine[key] === "string",
      ) ||
      !["time_ms", "remaining", "capacity"].every((key) =>
        natural(magazine[key]),
      ) ||
      !(magazine.reload_at_ms === null || natural(magazine.reload_at_ms)) ||
      typeof magazine.enhanced !== "boolean" ||
      typeof magazine.battle_terminal !== "boolean" ||
      !record(magazine.equipment) ||
      typeof magazine.equipment.definition_uuid !== "string" ||
      typeof magazine.equipment.runtime_item_instance_id !== "string" ||
      !(
        magazine.equipment.persistent_copy_uuid === null ||
        typeof magazine.equipment.persistent_copy_uuid === "string"
      ) ||
      typeof magazine.operation !== "string" ||
      !["Fired", "Reloaded", "Reward"].includes(magazine.operation) ||
      !sourceTrace(magazine.origin)
    ) {
      throw new Error("탄창 관측의 형식이 올바르지 않습니다.");
    }
    if (
      magazine.origin !== null &&
      (!record(result.identity) ||
        magazine.source_file !== result.identity.artifact_path)
    ) {
      throw new Error("탄창 관측의 원본이 실행 세대에 일치하지 않습니다.");
    }
  }
  const deferred = result.latest_deferred_damage;
  if (deferred != null) {
    if (
      !record(deferred) ||
      !["target", "definition_id", "source_file", "field_path"].every(
        (key) => typeof deferred[key] === "string",
      ) ||
      !["time_ms", "amount", "actual_hp_loss", "remaining"].every((key) =>
        natural(deferred[key]),
      ) ||
      typeof deferred.battle_terminal !== "boolean" ||
      !record(deferred.equipment) ||
      typeof deferred.equipment.definition_uuid !== "string" ||
      typeof deferred.equipment.runtime_item_instance_id !== "string" ||
      !(
        deferred.equipment.persistent_copy_uuid === null ||
        typeof deferred.equipment.persistent_copy_uuid === "string"
      ) ||
      (deferred.operation !== "Store" &&
        deferred.operation !== "Repay" &&
        deferred.operation !== "ClearOnKill") ||
      !sourceTrace(deferred.origin)
    ) {
      throw new Error("지연 피해 관측의 형식이 올바르지 않습니다.");
    }
    if (
      deferred.origin !== null &&
      (!record(result.identity) ||
        deferred.source_file !== result.identity.artifact_path)
    ) {
      throw new Error("지연 피해 관측의 원본이 실행 세대에 일치하지 않습니다.");
    }
  }
  const echo = result.latest_basic_heal_echo;
  if (echo != null) {
    if (
      !record(echo) ||
      !["actor", "definition_id", "source_file", "field_path"].every(
        (key) => typeof echo[key] === "string",
      ) ||
      ![
        "time_ms",
        "actual_other_healed",
        "actual_heal_percent",
        "max_hp_cap_percent",
        "max_hp",
        "requested_self_heal",
        "actual_self_healed",
      ].every((key) => natural(echo[key])) ||
      typeof echo.battle_terminal !== "boolean" ||
      !record(echo.equipment) ||
      typeof echo.equipment.definition_uuid !== "string" ||
      typeof echo.equipment.runtime_item_instance_id !== "string" ||
      !(
        echo.equipment.persistent_copy_uuid === null ||
        typeof echo.equipment.persistent_copy_uuid === "string"
      ) ||
      !record(echo.healing_source) ||
      typeof echo.healing_source.actor !== "string" ||
      !natural(echo.healing_source.action_ordinal) ||
      !sourceTrace(echo.origin)
    ) {
      throw new Error("기본 치료 자기 회복 관측의 형식이 올바르지 않습니다.");
    }
    if (
      echo.origin !== null &&
      (!record(result.identity) ||
        echo.source_file !== result.identity.artifact_path)
    ) {
      throw new Error(
        "기본 치료 자기 회복 관측의 원본이 실행 세대에 일치하지 않습니다.",
      );
    }
  }
  const healBarrier = result.latest_heal_barrier;
  if (healBarrier != null) {
    if (
      !record(healBarrier) ||
      !["actor", "target", "definition_id", "source_file", "field_path"].every(
        (key) => typeof healBarrier[key] === "string",
      ) ||
      ![
        "time_ms",
        "hp_before",
        "max_hp",
        "actual_healed",
        "base_capacity",
        "increased_capacity",
        "remaining",
        "expires_at_ms",
        "ready_at_ms",
      ].every((key) => natural(healBarrier[key])) ||
      typeof healBarrier.kept_existing !== "boolean" ||
      typeof healBarrier.battle_terminal !== "boolean" ||
      !record(healBarrier.equipment) ||
      typeof healBarrier.equipment.definition_uuid !== "string" ||
      typeof healBarrier.equipment.runtime_item_instance_id !== "string" ||
      !(
        healBarrier.equipment.persistent_copy_uuid === null ||
        typeof healBarrier.equipment.persistent_copy_uuid === "string"
      ) ||
      !sourceTrace(healBarrier.origin)
    ) {
      throw new Error("회복 결계 관측의 형식이 올바르지 않습니다.");
    }
    if (
      healBarrier.origin !== null &&
      (!record(result.identity) ||
        healBarrier.source_file !== result.identity.artifact_path)
    ) {
      throw new Error("회복 결계 관측의 원본이 실행 세대에 일치하지 않습니다.");
    }
  }
  const shieldBeneficiary = result.latest_shield_beneficiary_damage;
  if (shieldBeneficiary != null) {
    if (
      !record(shieldBeneficiary) ||
      ![
        "actor",
        "beneficiary",
        "target",
        "provider",
        "definition_id",
        "source_file",
        "field_path",
      ].every((k) => typeof shieldBeneficiary[k] === "string") ||
      !["shield_activation", "shield_remaining", "percent", "time_ms"].every(
        (k) => natural(shieldBeneficiary[k]),
      ) ||
      Number(shieldBeneficiary.shield_activation) === 0 ||
      Number(shieldBeneficiary.shield_remaining) === 0 ||
      Number(shieldBeneficiary.shield_remaining) > 4294967295 ||
      Number(shieldBeneficiary.percent) === 0 ||
      Number(shieldBeneficiary.percent) > 10000 ||
      typeof shieldBeneficiary.battle_terminal !== "boolean" ||
      !sourceTrace(shieldBeneficiary.origin) ||
      !record(shieldBeneficiary.equipment) ||
      typeof shieldBeneficiary.equipment.definition_uuid !== "string" ||
      typeof shieldBeneficiary.equipment.runtime_item_instance_id !==
        "string" ||
      !(
        shieldBeneficiary.equipment.persistent_copy_uuid === null ||
        typeof shieldBeneficiary.equipment.persistent_copy_uuid === "string"
      ) ||
      !record(shieldBeneficiary.shield_source) ||
      !(
        shieldBeneficiary.shield_source.actor === null ||
        shieldBeneficiary.shield_source.actor === shieldBeneficiary.provider
      ) ||
      !(
        shieldBeneficiary.shield_source.action_ordinal === null ||
        natural(shieldBeneficiary.shield_source.action_ordinal)
      ) ||
      (shieldBeneficiary.origin != null &&
        shieldBeneficiary.source_file !== result.identity?.artifact_path)
    ) {
      throw new Error(
        "보호막 수혜 피해 증가 관측의 출처 또는 값이 올바르지 않습니다.",
      );
    }
  }
  const adaptive = result.latest_adaptive_shield;
  if (adaptive != null) {
    if (
      !record(adaptive) ||
      !["actor", "primary", "definition_id", "source_file", "field_path"].every(
        (key) => typeof adaptive[key] === "string",
      ) ||
      ![
        "max_hp",
        "capacity",
        "activation",
        "time_ms",
        "expires_at_ms",
        "ready_at_ms",
      ].every((key) => natural(adaptive[key])) ||
      Number(adaptive.capacity) === 0 ||
      Number(adaptive.activation) === 0 ||
      Number(adaptive.expires_at_ms) <= Number(adaptive.time_ms) ||
      Number(adaptive.ready_at_ms) <= Number(adaptive.time_ms) ||
      !["Physical", "Magic"].includes(String(adaptive.damage_type)) ||
      typeof adaptive.battle_terminal !== "boolean" ||
      !record(adaptive.equipment) ||
      typeof adaptive.equipment.definition_uuid !== "string" ||
      typeof adaptive.equipment.runtime_item_instance_id !== "string" ||
      !(
        adaptive.equipment.persistent_copy_uuid === null ||
        typeof adaptive.equipment.persistent_copy_uuid === "string"
      ) ||
      !record(adaptive.attack_source) ||
      adaptive.attack_source.actor !== adaptive.actor ||
      !natural(adaptive.attack_source.action_ordinal) ||
      !sourceTrace(adaptive.origin)
    ) {
      throw new Error("적응형 보호막 관측의 형식이 올바르지 않습니다.");
    }
    if (
      adaptive.origin !== null &&
      (!record(result.identity) ||
        adaptive.source_file !== result.identity.artifact_path)
    ) {
      throw new Error(
        "적응형 보호막 관측의 원본이 실행 세대에 일치하지 않습니다.",
      );
    }
  }
  for (const [kind, witness] of [
    ["heal", result.latest_received_heal],
    ["control", result.latest_control_duration],
  ] as const) {
    if (witness == null) continue;
    if (
      !record(witness) ||
      !["target", "definition_id", "source_file", "field_path"].every(
        (k) => typeof witness[k] === "string",
      ) ||
      !natural(witness.time_ms) ||
      typeof witness.battle_terminal !== "boolean" ||
      !sourceTrace(witness.origin) ||
      !record(witness.equipment) ||
      typeof witness.equipment.definition_uuid !== "string" ||
      typeof witness.equipment.runtime_item_instance_id !== "string" ||
      !(
        witness.equipment.persistent_copy_uuid === null ||
        typeof witness.equipment.persistent_copy_uuid === "string"
      ) ||
      !record(witness.incoming) ||
      !(
        witness.incoming.actor === null ||
        typeof witness.incoming.actor === "string"
      ) ||
      !(
        witness.incoming.action_ordinal === null ||
        natural(witness.incoming.action_ordinal)
      ) ||
      (witness.origin != null &&
        witness.source_file !== result.identity?.artifact_path)
    )
      throw new Error(
        "받는 회복·제어 시간 관측의 실행 원본이 올바르지 않습니다.",
      );
    if (kind === "heal") {
      if (
        ![
          "blocked_hostiles",
          "percent",
          "other_percent",
          "before",
          "after",
        ].every((k) => natural(witness[k])) ||
        !(witness.actor === null || typeof witness.actor === "string") ||
        Number(witness.percent) > 10000 ||
        Number(witness.other_percent) > 4294967295 ||
        Number(witness.blocked_hostiles) > 4294967295 ||
        Number(witness.before) > 2147483647 ||
        Number(witness.after) > 2147483647 ||
        BigInt(Number(witness.after)) !==
          ((BigInt(Number(witness.before)) *
            (100n +
              BigInt(Number(witness.percent)) +
              BigInt(Number(witness.other_percent)))) /
            100n >
          2147483647n
            ? 2147483647n
            : (BigInt(Number(witness.before)) *
                (100n +
                  BigInt(Number(witness.percent)) +
                  BigInt(Number(witness.other_percent)))) /
              100n)
      )
        throw new Error("받는 회복 관측의 수치가 올바르지 않습니다.");
    } else {
      if (
        typeof witness.caster !== "string" ||
        ![
          "Stun",
          "Freeze",
          "Root",
          "Slow",
          "Silence",
          "Taunt",
          "BasicLock",
        ].includes(String(witness.kind)) ||
        !["reduction_percent", "original_ms", "effective_ms"].every((k) =>
          natural(witness[k]),
        ) ||
        Number(witness.reduction_percent) < 1 ||
        Number(witness.reduction_percent) > 99 ||
        Number(witness.original_ms) < 1 ||
        Number(witness.effective_ms) < 1 ||
        BigInt(Number(witness.effective_ms)) !==
          (BigInt(Number(witness.original_ms)) *
            (100n - BigInt(Number(witness.reduction_percent))) +
            99n) /
            100n
      )
        throw new Error("제어 시간 감소 관측의 수치가 올바르지 않습니다.");
    }
  }
  const wearerReduction = result.latest_wearer_reduction;
  if (wearerReduction != null) {
    const w = wearerReduction;
    if (
      !record(w) ||
      !["target", "definition_id", "source_file", "field_path"].every(
        (k) => typeof w[k] === "string",
      ) ||
      !["time_ms", "strike", "current_hp", "max_hp", "percent"].every((k) =>
        natural(w[k]),
      ) ||
      Number(w.current_hp) < 1 ||
      Number(w.max_hp) < Number(w.current_hp) ||
      Number(w.max_hp) > 4294967295 ||
      Number(w.percent) > 99 ||
      typeof w.battle_terminal !== "boolean" ||
      !sourceTrace(w.origin) ||
      !record(w.equipment) ||
      typeof w.equipment.definition_uuid !== "string" ||
      typeof w.equipment.runtime_item_instance_id !== "string" ||
      !(
        w.equipment.persistent_copy_uuid === null ||
        typeof w.equipment.persistent_copy_uuid === "string"
      ) ||
      !record(w.incoming) ||
      !(w.incoming.actor === null || typeof w.incoming.actor === "string") ||
      !(
        w.incoming.action_ordinal === null || natural(w.incoming.action_ordinal)
      ) ||
      (w.origin != null && w.source_file !== result.identity?.artifact_path)
    ) {
      throw new Error(
        "현재 HP 기반 감면 관측의 수치 또는 실행 원본이 올바르지 않습니다.",
      );
    }
  }
  const flat = result.latest_incoming_flat;
  if (flat != null) {
    if (
      !record(flat) ||
      !["target", "definition_id", "source_file", "field_path"].every(
        (key) => typeof flat[key] === "string",
      ) ||
      !["time_ms", "strike", "allowance", "before", "used", "remaining"].every(
        (key) => natural(flat[key]),
      ) ||
      !Number.isSafeInteger(flat.defense) ||
      Number(flat.defense) < -2147483648 ||
      Number(flat.defense) > 2147483647 ||
      Number(flat.allowance) > 2147483647 ||
      Number(flat.before) > Number(flat.allowance) ||
      Number(flat.used) + Number(flat.remaining) !== Number(flat.before) ||
      typeof flat.battle_terminal !== "boolean" ||
      !sourceTrace(flat.origin) ||
      !record(flat.equipment) ||
      typeof flat.equipment.definition_uuid !== "string" ||
      typeof flat.equipment.runtime_item_instance_id !== "string" ||
      !(
        flat.equipment.persistent_copy_uuid === null ||
        typeof flat.equipment.persistent_copy_uuid === "string"
      ) ||
      !record(flat.incoming) ||
      typeof flat.incoming.actor !== "string" ||
      !(
        flat.incoming.action_ordinal === null ||
        natural(flat.incoming.action_ordinal)
      ) ||
      (flat.origin != null &&
        flat.source_file !== result.identity?.artifact_path)
    ) {
      throw new Error(
        "물리 기본 타격 차감 관측의 수치 또는 실행 원본이 올바르지 않습니다.",
      );
    }
  }
  const opening = result.latest_channel_opening;
  if (opening != null) {
    if (
      !record(opening) ||
      !["target", "definition_id", "source_file", "field_path"].every(
        (key) => typeof opening[key] === "string",
      ) ||
      !natural(opening.time_ms) ||
      !natural(opening.started_at_ms) ||
      !natural(opening.window_end_ms) ||
      Number(opening.time_ms) < Number(opening.started_at_ms) ||
      Number(opening.time_ms) >= Number(opening.window_end_ms) ||
      !natural(opening.reduction_percent) ||
      Number(opening.reduction_percent) < 1 ||
      Number(opening.reduction_percent) > 100 ||
      typeof opening.battle_terminal !== "boolean" ||
      !sourceTrace(opening.origin) ||
      !record(opening.equipment) ||
      typeof opening.equipment.definition_uuid !== "string" ||
      typeof opening.equipment.runtime_item_instance_id !== "string" ||
      !(
        opening.equipment.persistent_copy_uuid === null ||
        typeof opening.equipment.persistent_copy_uuid === "string"
      ) ||
      (opening.origin != null &&
        opening.source_file !== result.identity?.artifact_path)
    ) {
      throw new Error(
        "집중 초반 보호 관측의 형식 또는 실행 원본이 올바르지 않습니다.",
      );
    }
  }
  const barrier = result.latest_skill_barrier;
  if (barrier != null) {
    if (
      !record(barrier) ||
      !["target", "definition_id", "source_file", "field_path"].every(
        (key) => typeof barrier[key] === "string",
      ) ||
      !natural(barrier.time_ms) ||
      !natural(barrier.ready_at_ms) ||
      Number(barrier.ready_at_ms) <= Number(barrier.time_ms) ||
      typeof barrier.battle_terminal !== "boolean" ||
      !sourceTrace(barrier.origin) ||
      !record(barrier.equipment) ||
      typeof barrier.equipment.definition_uuid !== "string" ||
      typeof barrier.equipment.runtime_item_instance_id !== "string" ||
      !(
        barrier.equipment.persistent_copy_uuid === null ||
        typeof barrier.equipment.persistent_copy_uuid === "string"
      ) ||
      !record(barrier.incoming) ||
      !record(barrier.incoming.cast) ||
      typeof barrier.incoming.cast.skill_id !== "string" ||
      typeof barrier.incoming.cast.caster_instance_id !== "string" ||
      !natural(barrier.incoming.cast.cast_seq)
    ) {
      throw new Error("스킬 차단 관측의 형식이 올바르지 않습니다.");
    }
  }
  const evasion = result.latest_basic_evasion;
  if (evasion != null) {
    if (
      !record(evasion) ||
      !["target", "definition_id", "source_file", "field_path"].every(
        (key) => typeof evasion[key] === "string",
      ) ||
      !natural(evasion.time_ms) ||
      !natural(evasion.strike) ||
      !natural(evasion.chance_percent) ||
      Number(evasion.chance_percent) > 100 ||
      !natural(evasion.roll_percent) ||
      Number(evasion.roll_percent) >= 100 ||
      typeof evasion.evaded !== "boolean" ||
      evasion.evaded !==
        Number(evasion.roll_percent) < Number(evasion.chance_percent) ||
      typeof evasion.battle_terminal !== "boolean" ||
      !sourceTrace(evasion.origin) ||
      !record(evasion.equipment) ||
      typeof evasion.equipment.definition_uuid !== "string" ||
      typeof evasion.equipment.runtime_item_instance_id !== "string" ||
      !(
        evasion.equipment.persistent_copy_uuid === null ||
        typeof evasion.equipment.persistent_copy_uuid === "string"
      ) ||
      !record(evasion.incoming) ||
      typeof evasion.incoming.actor !== "string"
    ) {
      throw new Error("기본 공격 회피 관측의 형식이 올바르지 않습니다.");
    }
  }
  for (const observation of [
    effect,
    resonance,
    hitCounter,
    sustained,
    area,
    barrier,
    evasion,
  ]) {
    if (
      record(observation) &&
      observation.origin !== null &&
      (!record(result.identity) ||
        observation.source_file !== result.identity.artifact_path)
    )
      throw new Error("실행 관측의 원본이 실행 세대에 일치하지 않습니다.");
  }
  return result as RunResponse;
}
