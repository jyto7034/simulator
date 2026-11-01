import { test, expect } from "@playwright/test";
import { digest } from "../../src/composition/client";
import {
  parseTrialConfig,
  runDefinition,
} from "../../src/composition/execution";
import type { Definition } from "../../src/composition/session";

test("execution accepts an authored equipment source without inventing a skill cast", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const effect = {
    time_ms: 31,
    skill_id: null,
    definition_id: "equipment:weapon",
    equipment: {
      definition_uuid: "definition",
      runtime_item_instance_id: "item",
      persistent_copy_uuid: "copy",
    },
    source_file: "artifact",
    field_path: "equipment_attacks[0].program.reactions[0].body[1]",
    target_unit: "employee",
    hp_delta: 5,
    origin: {
      definition: "values[0].value.AttackProgram",
      calls: [],
      inputs: [],
    },
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 35,
    failure: null,
    latest_effect: effect,
    latest_sustained: null,
    latest_area: null,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    globalThis.fetch = async () =>
      new Response(JSON.stringify(response), {
        headers: { "Content-Type": "application/json" },
      });
    expect((await runDefinition(definition, config, 7)).latest_effect).toEqual(
      effect,
    );
    Object.assign(effect, { hp_reason: "HealthCost", hp_delta: -18 });
    expect(
      (await runDefinition(definition, config, 7)).latest_effect?.hp_reason,
    ).toBe("HealthCost");
    for (const hp_reason of ["unknown", ["HealthCost"]]) {
      Object.assign(effect, { hp_reason });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "실행 관측",
      );
    }
    Object.assign(effect, { hp_reason: null, hp_delta: 5 });
    const opening = {
      target: "employee",
      equipment: effect.equipment,
      time_ms: 31,
      started_at_ms: 10,
      window_end_ms: 2010,
      reduction_percent: 50,
      definition_id: "equipment:armor",
      source_file: "artifact",
      field_path: "equipment_defenses[0].program.ChannelOpeningGuard",
      battle_terminal: false,
      origin: effect.origin,
    };
    Object.assign(response, { latest_channel_opening: opening });
    expect(
      (await runDefinition(definition, config, 7)).latest_channel_opening,
    ).toEqual(opening);
    for (const patch of [
      { time_ms: 9 },
      { time_ms: 2010 },
      { started_at_ms: -1 },
      { reduction_percent: 0 },
      { reduction_percent: 101 },
      { reduction_percent: 1.5 },
      { equipment: {} },
      { origin: { definition: "x", calls: 2, inputs: [] } },
      { source_file: "stale-artifact" },
    ]) {
      Object.assign(response, {
        latest_channel_opening: { ...opening, ...patch },
      });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "집중 초반 보호 관측",
      );
    }
    Object.assign(response, { latest_channel_opening: null });
    const resonance = {
      time_ms: 31,
      battle_terminal: false,
      skill_id: null,
      definition_id: "equipment:accessory",
      equipment: effect.equipment,
      source_file: "artifact",
      field_path: "equipment_states[0].program.handlers[0].body[0]",
      target_unit: "employee",
      before: 8,
      after: 15,
      max: 25,
      origin: effect.origin,
    };
    Object.assign(response, { latest_resonance: resonance });
    expect(
      (await runDefinition(definition, config, 7)).latest_resonance,
    ).toEqual(resonance);
    for (const patch of [
      { before: -1 },
      { after: 26 },
      { max: 0 },
      { after: 1.5 },
      { equipment: {} },
      { origin: { definition: "x", calls: 2, inputs: [] } },
    ]) {
      Object.assign(response, { latest_resonance: { ...resonance, ...patch } });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "공명 관측",
      );
    }
    Object.assign(response, {
      latest_resonance: { ...resonance, source_file: "old-generation" },
    });
    await expect(runDefinition(definition, config, 7)).rejects.toThrow(
      "실행 세대",
    );
    Object.assign(response, { latest_resonance: null });
    const sustained = {
      time_ms: 1501,
      battle_terminal: false,
      expires_at_ms: 1501,
      activation: 2,
      ended: "Expired",
      skill_id: "skill",
      cast_sequence: 3,
      definition_id: "skill",
      source_file: "artifact",
      field_path: "skills.skill.steps.grant.effects.state",
      target_unit: "employee",
      before: { attack: 18, defense: -4 },
      after: { attack: 12, defense: -6 },
      origin: {
        definition: "values[0].value.Effects[0].op",
        calls: ["skills[0]"],
        inputs: [],
      },
    };
    Object.assign(response, { latest_sustained: sustained });
    expect(
      (await runDefinition(definition, config, 7)).latest_sustained,
    ).toEqual(sustained);
    Object.assign(sustained, { equipment: effect.equipment });
    expect(
      (await runDefinition(definition, config, 7)).latest_sustained?.equipment,
    ).toEqual(effect.equipment);
    for (const equipment of [
      {},
      { ...effect.equipment, persistent_copy_uuid: 3 },
    ]) {
      Object.assign(sustained, { equipment });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "지속 효과 관측",
      );
    }
    Object.assign(sustained, { equipment: null });
    sustained.ended = "unknown reason";
    await expect(runDefinition(definition, config, 7)).rejects.toThrow(
      "지속 효과 관측",
    );
    sustained.ended = "Expired";
    sustained.source_file = "another-generation";
    await expect(runDefinition(definition, config, 7)).rejects.toThrow(
      "실행 세대",
    );
    sustained.source_file = "artifact";
    Object.assign(sustained.before, { attack: -1 });
    await expect(runDefinition(definition, config, 7)).rejects.toThrow(
      "지속 효과 관측",
    );
    sustained.before.attack = 18;
    Object.assign(sustained.before, { move_speed_units_per_ms: 2000 });
    Object.assign(sustained.after, { move_speed_units_per_ms: 1000 });
    expect(
      (await runDefinition(definition, config, 7)).latest_sustained,
    ).toEqual(sustained);
    for (const invalid of [
      -1,
      0.5,
      "1000",
      null,
      Number.MAX_SAFE_INTEGER + 1,
    ]) {
      Object.assign(sustained.before, { move_speed_units_per_ms: invalid });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "지속 효과 관측",
      );
    }
    Object.assign(sustained.before, { move_speed_units_per_ms: 0 });
    expect(
      (await runDefinition(definition, config, 7)).latest_sustained,
    ).toEqual(sustained);
    const producer = {
      area_id: "area",
      skill_id: "skill",
      step_id: "maintain",
      cast_sequence: 3,
      status_index: 1,
      definition: "generated-weakness",
      modifier_source: {
        copy_uuid: "copy",
        definition: "broken-balance",
        rule: "on-stun",
      },
    };
    Object.assign(sustained, { area_source: producer });
    expect(
      (await runDefinition(definition, config, 7)).latest_sustained
        ?.area_source,
    ).toEqual(producer);
    for (const invalid of [
      { status_index: -1 },
      { cast_sequence: 0.5 },
      { definition: null },
      { modifier_source: {} },
      { modifier_source: { copy_uuid: 1, definition: "relic", rule: "rule" } },
    ]) {
      Object.assign(sustained, { area_source: { ...producer, ...invalid } });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "지속 효과 관측",
      );
    }
    Object.assign(sustained, { area_source: producer });
    const area = {
      ordinal: 83,
      time_ms: 2503,
      area_id: "area",
      skill_id: "skill",
      step_id: "maintain",
      caster: "caster",
      cast_sequence: 1,
      started_at_ms: 1,
      expires_at_ms: 2503,
      change: { kind: "ended", reason: "Expired" },
      before: { members: 1, contributions: 3 },
      after: null,
      battle_terminal: false,
      source_file: "artifact",
      field_path: "skills[0].steps[0].delivery",
      origin: {
        definition: "definitions[0].body.Action.delivery",
        calls: ["skills[0].body.Call"],
        inputs: ["values[0].value"],
      },
    };
    Object.assign(response, { latest_area: area });
    expect((await runDefinition(definition, config, 7)).latest_area).toEqual(
      area,
    );
    for (const patch of [
      { change: { kind: "ended", reason: "guessed" } },
      { after: { members: 1, contributions: 1 } },
      { before: { members: -1, contributions: 1 } },
      { before: { members: 0.5, contributions: 1 } },
      { cast_sequence: Number.MAX_SAFE_INTEGER + 1 },
      { change: { kind: "reconciled" } },
      { origin: null },
    ]) {
      Object.assign(response, { latest_area: { ...area, ...patch } });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "장판 관측",
      );
    }
    Object.assign(response, {
      latest_area: { ...area, source_file: "another-generation" },
    });
    await expect(runDefinition(definition, config, 7)).rejects.toThrow(
      "실행 세대",
    );
    const unlinked = {
      ...area,
      origin: null,
      source_file: null,
      field_path: null,
    };
    Object.assign(response, { latest_area: unlinked });
    expect((await runDefinition(definition, config, 7)).latest_area).toEqual(
      unlinked,
    );
    Object.assign(response, { latest_area: null });
    const speed = {
      time_ms: 2000,
      actor: "employee",
      current_hp: 75,
      max_hp: 100,
      base_interval_ms: 1000,
      final_interval_ms: 666,
      equipment_points: 50,
      battle_terminal: true,
      contributors: [
        {
          equipment: { ...effect.equipment },
          points: 50,
          definition_id: "equipment:receiver",
          source_file: "artifact",
          field_path: "equipment_attacks[0].program.basic_speed_points",
        },
      ],
      origins: [effect.origin],
    };
    Object.assign(response, { latest_equipment_speed: speed });
    expect(
      (await runDefinition(definition, config, 7)).latest_equipment_speed,
    ).toEqual(speed);
    for (const patch of [
      { max_hp: 0 },
      { current_hp: 101 },
      { final_interval_ms: 0 },
      { equipment_points: 51 },
      { origins: [] },
      { contributors: [] },
      { origins: [{ definition: 42, calls: [], inputs: [] }] },
    ]) {
      Object.assign(response, {
        latest_equipment_speed: { ...speed, ...patch },
      });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "장비 공속 관측",
      );
    }
    Object.assign(response, {
      latest_equipment_speed: {
        ...speed,
        contributors: [
          { ...speed.contributors[0], source_file: "old-generation" },
        ],
      },
    });
    await expect(runDefinition(definition, config, 7)).rejects.toThrow(
      "실행 세대",
    );
    Object.assign(response, { latest_equipment_speed: null });
    const barrier = {
      target: "employee",
      equipment: { ...effect.equipment },
      time_ms: 1,
      ready_at_ms: 20001,
      definition_id: "shroud",
      source_file: "artifact",
      field_path:
        "equipment_defenses[0].program.RechargingSkillBarrier.recharge_ms",
      incoming: {
        cast: {
          skill_id: "hostile-skill",
          caster_instance_id: "enemy",
          cast_seq: 1,
        },
      },
      battle_terminal: true,
      origin: effect.origin,
    };
    Object.assign(response, { latest_skill_barrier: barrier });
    expect(
      (await runDefinition(definition, config, 7)).latest_skill_barrier,
    ).toEqual(barrier);
    for (const patch of [
      { ready_at_ms: 1 },
      { time_ms: -1 },
      { incoming: [] },
      { incoming: { cast: { ...barrier.incoming.cast, cast_seq: "1" } } },
      { equipment: { ...barrier.equipment, persistent_copy_uuid: [] } },
    ]) {
      Object.assign(response, {
        latest_skill_barrier: { ...barrier, ...patch },
      });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "스킬 차단 관측",
      );
    }
    Object.assign(response, {
      latest_skill_barrier: { ...barrier, source_file: "old-generation" },
    });
    await expect(runDefinition(definition, config, 7)).rejects.toThrow(
      "실행 세대",
    );
    Object.assign(response, { latest_skill_barrier: null });
    const evasion = {
      target: "target",
      equipment: barrier.equipment,
      time_ms: 10,
      strike: 1,
      chance_percent: 30,
      roll_percent: 20,
      evaded: true,
      definition_id: "evasion",
      source_file: barrier.source_file,
      field_path:
        "equipment_defenses[0].program.BasicAttackEvasion.chance_percent",
      incoming: { actor: "enemy" },
      battle_terminal: true,
      origin: effect.origin,
    };
    Object.assign(response, { latest_basic_evasion: evasion });
    expect(
      (await runDefinition(definition, config, 7)).latest_basic_evasion,
    ).toEqual(evasion);
    for (const patch of [
      { chance_percent: 101 },
      { roll_percent: 100 },
      { roll_percent: -1 },
      { evaded: false },
      { strike: -1 },
      { incoming: { actor: null } },
    ]) {
      Object.assign(response, {
        latest_basic_evasion: { ...evasion, ...patch },
      });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "기본 공격 회피 관측",
      );
    }
    Object.assign(response, {
      latest_basic_evasion: { ...evasion, source_file: "old-generation" },
    });
    await expect(runDefinition(definition, config, 7)).rejects.toThrow(
      "실행 세대",
    );
    Object.assign(response, { latest_basic_evasion: null });
    const support = {
      actor: "healer",
      target: "ally",
      channel: "heal",
      time_ms: 20,
      base_amount: 19,
      increased_amount: 23,
      percent: 25,
      battle_terminal: true,
      contributors: [
        {
          equipment: barrier.equipment,
          percent: 25,
          definition_id: "brooch",
          source_file: "artifact",
          field_path: "equipment_supports[0].program.heal_percent",
        },
      ],
      origins: [effect.origin],
    };
    Object.assign(response, { latest_equipment_support: support });
    expect(
      (await runDefinition(definition, config, 7)).latest_equipment_support,
    ).toEqual(support);
    for (const patch of [
      { channel: "damage" },
      { percent: 30 },
      { origins: [] },
      { contributors: [{ ...support.contributors[0], percent: 10001 }] },
      {
        contributors: [
          { ...support.contributors[0], source_file: "old-generation" },
        ],
      },
    ]) {
      Object.assign(response, {
        latest_equipment_support: { ...support, ...patch },
      });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "장비 지원 관측",
      );
    }
    Object.assign(response, { latest_equipment_support: null });
    const reserve = {
      time_ms: 1,
      target: "wearer",
      equipment: barrier.equipment,
      definition_id: "armor",
      source_file: barrier.source_file,
      field_path: "equipment_defenses[0].program.OverhealReserve",
      operation: "Release",
      capacity: 500,
      stored_before: 500,
      stored_after: 0,
      incoming_overheal: 0,
      actual_healed: 500,
      battle_terminal: false,
      origin: barrier.origin,
    };
    Object.assign(response, { latest_overheal_reserve: reserve });
    expect(
      (await runDefinition(definition, config, 7)).latest_overheal_reserve,
    ).toEqual(reserve);
    for (const patch of [
      { stored_after: -1 },
      { actual_healed: 0.5 },
      { operation: ["Store"] },
      { operation: "other" },
      { equipment: null },
      { source_file: "old-generation" },
    ]) {
      Object.assign(response, {
        latest_overheal_reserve: { ...reserve, ...patch },
      });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "초과 회복 저장 관측",
      );
    }
    Object.assign(response, { latest_overheal_reserve: null });
    const deferred = {
      time_ms: 1000,
      target: "wearer",
      equipment: barrier.equipment,
      definition_id: "scar-armor",
      source_file: barrier.source_file,
      field_path: "equipment_defenses[0].program.DeferredDamage",
      operation: "Repay",
      amount: 6,
      actual_hp_loss: 6,
      remaining: 24,
      battle_terminal: false,
      origin: barrier.origin,
    };
    Object.assign(response, { latest_deferred_damage: deferred });
    expect(
      (await runDefinition(definition, config, 7)).latest_deferred_damage,
    ).toEqual(deferred);
    for (const patch of [
      { amount: -1 },
      { actual_hp_loss: 0.5 },
      { remaining: -1 },
      { operation: ["Repay"] },
      { operation: "other" },
      { equipment: null },
      { source_file: "old-generation" },
    ]) {
      Object.assign(response, {
        latest_deferred_damage: { ...deferred, ...patch },
      });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "지연 피해 관측",
      );
    }
    Object.assign(response, { latest_deferred_damage: null });
    const magazine = {
      time_ms: 200,
      actor: "shooter",
      equipment: barrier.equipment,
      definition_id: "equipment:handcannon",
      source_file: barrier.source_file,
      field_path: "equipment_attacks[0].program.magazine",
      operation: "Fired",
      enhanced: false,
      remaining: 5,
      capacity: 6,
      reload_at_ms: null,
      battle_terminal: false,
      origin: barrier.origin,
    };
    Object.assign(response, { latest_magazine: magazine });
    expect(
      (await runDefinition(definition, config, 7)).latest_magazine,
    ).toEqual(magazine);
    for (const patch of [
      { remaining: -1 },
      { capacity: 0.5 },
      { enhanced: "yes" },
      { reload_at_ms: -1 },
      { operation: ["Fired"] },
      { operation: "other" },
      { equipment: null },
      { source_file: "old-generation" },
    ]) {
      Object.assign(response, { latest_magazine: { ...magazine, ...patch } });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "탄창 관측",
      );
    }
    Object.assign(response, { latest_magazine: null });
    const bounty = {
      time_ms: 6000,
      actor: "wearer",
      equipment_copy: "actual-copy",
      operation: "Earned",
      pending: 1,
      confirmed: 2,
      tier_attack: 10,
      next_threshold: 4,
      actual_attack: 110,
      definition_id: "bounty",
      source_file: barrier.source_file,
      field_path: "equipment_attacks[0].program.cohort",
      battle_terminal: false,
      origin: barrier.origin,
    };
    Object.assign(response, { latest_bounty: bounty });
    expect((await runDefinition(definition, config, 7)).latest_bounty).toEqual(
      bounty,
    );
    for (const patch of [
      { pending: -1 },
      { confirmed: 0.5 },
      { tier_attack: "10" },
      { next_threshold: -1 },
      { actual_attack: -1 },
      { equipment_copy: null },
      { actor: 17 },
      { operation: ["Earned"] },
      { operation: "other" },
      { source_file: "old-generation" },
    ]) {
      Object.assign(response, { latest_bounty: { ...bounty, ...patch } });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "수금 관측",
      );
    }
    const terminalBounty = {
      ...bounty,
      actor: null,
      actual_attack: null,
      next_threshold: null,
      operation: "Confirmed",
      pending: 0,
      confirmed: 4,
      tier_attack: 25,
      battle_terminal: true,
    };
    Object.assign(response, { latest_bounty: terminalBounty });
    expect((await runDefinition(definition, config, 7)).latest_bounty).toEqual(
      terminalBounty,
    );
    Object.assign(response, { latest_bounty: null });
    const cohort = {
      time_ms: 5000,
      employee: "employee",
      definition: "prey",
      unit: "actual-hostile",
      next_spawn_ms: 10000,
      provider_count: 1,
      fragment_recipient: null,
      equipment_copy: "actual-accessory-copy",
      initial_hp: 300,
      source_file: barrier.source_file,
      field_path: "equipment_attacks[0].program.cohort",
      battle_terminal: false,
      origin: barrier.origin,
    };
    Object.assign(response, { latest_shared_cohort: cohort });
    expect(
      (await runDefinition(definition, config, 7)).latest_shared_cohort,
    ).toEqual(cohort);
    for (const patch of [
      { initial_hp: -1 },
      { provider_count: 0.5 },
      { unit: null },
      { fragment_recipient: 17 },
      { next_spawn_ms: -1 },
      { source_file: "old-generation" },
      { field_path: null },
    ]) {
      Object.assign(response, {
        latest_shared_cohort: { ...cohort, ...patch },
      });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "공유 사냥감 관측",
      );
    }
    const fragmentCohort = {
      ...cohort,
      fragment_recipient: "fragment-copy",
      equipment_copy: null,
      source_file: null,
      field_path: null,
      origin: null,
    };
    Object.assign(response, { latest_shared_cohort: fragmentCohort });
    expect(
      (await runDefinition(definition, config, 7)).latest_shared_cohort,
    ).toEqual(fragmentCohort);
    Object.assign(response, { latest_shared_cohort: null });

    const echo = {
      time_ms: 1,
      actor: "healer",
      equipment: barrier.equipment,
      definition_id: "pendant",
      source_file: barrier.source_file,
      field_path: "equipment_supports[0].program.basic_heal_echo",
      actual_other_healed: 135,
      actual_heal_percent: 25,
      max_hp_cap_percent: 10,
      max_hp: 1000,
      requested_self_heal: 33,
      actual_self_healed: 20,
      healing_source: { actor: "healer", action_ordinal: 1 },
      battle_terminal: false,
      origin: barrier.origin,
    };
    Object.assign(response, { latest_basic_heal_echo: echo });
    expect(
      (await runDefinition(definition, config, 7)).latest_basic_heal_echo,
    ).toEqual(echo);
    for (const patch of [
      { actual_other_healed: -1 },
      { requested_self_heal: 1.5 },
      { equipment: null },
      { healing_source: { actor: "healer" } },
      { source_file: "old-generation" },
    ]) {
      Object.assign(response, {
        latest_basic_heal_echo: { ...echo, ...patch },
      });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "기본 치료 자기 회복 관측",
      );
    }
    Object.assign(response, { latest_basic_heal_echo: null });
    const healBarrier = {
      time_ms: 1,
      actor: "healer",
      target: "ally",
      equipment: barrier.equipment,
      definition_id: "armor",
      source_file: barrier.source_file,
      field_path: "equipment_supports[0].program.direct_heal_barrier",
      hp_before: 500,
      max_hp: 1000,
      actual_healed: 25,
      base_capacity: 6,
      increased_capacity: 7,
      remaining: 7,
      expires_at_ms: 5001,
      ready_at_ms: 8001,
      kept_existing: false,
      battle_terminal: false,
      origin: barrier.origin,
    };
    Object.assign(response, { latest_heal_barrier: healBarrier });
    expect(
      (await runDefinition(definition, config, 7)).latest_heal_barrier,
    ).toEqual(healBarrier);
    for (const patch of [
      { actual_healed: -1 },
      { kept_existing: 1 },
      { equipment: null },
      { source_file: "old-generation" },
    ]) {
      Object.assign(response, {
        latest_heal_barrier: { ...healBarrier, ...patch },
      });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        /회복 결계 관측/,
      );
    }
    Object.assign(response, { latest_heal_barrier: null });

    const adaptive = {
      actor: "employee",
      primary: "enemy",
      equipment: barrier.equipment,
      damage_type: "Magic",
      max_hp: 1000,
      capacity: 200,
      activation: 1,
      time_ms: 1,
      expires_at_ms: 5001,
      ready_at_ms: 15001,
      definition_id: "armor",
      source_file: barrier.source_file,
      field_path: "equipment_defenses[0].program.AdaptiveBasicShield",
      attack_source: { actor: "employee", action_ordinal: 1 },
      battle_terminal: false,
      origin: barrier.origin,
    };
    Object.assign(response, { latest_adaptive_shield: adaptive });
    expect(
      (await runDefinition(definition, config, 7)).latest_adaptive_shield,
    ).toEqual(adaptive);
    for (const patch of [
      { damage_type: "True" },
      { capacity: 0 },
      { expires_at_ms: 1 },
      { ready_at_ms: 0 },
      { activation: -1 },
      { source_file: "old-generation" },
      { attack_source: { actor: "enemy", action_ordinal: 1 } },
      { equipment: null },
    ]) {
      Object.assign(response, {
        latest_adaptive_shield: { ...adaptive, ...patch },
      });
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "적응형 보호막 관측",
      );
    }
    Object.assign(response, { latest_adaptive_shield: null });

    // A malformed provider must not silently become an unknown source.
    Object.assign(effect.equipment, { persistent_copy_uuid: 42 });
    await expect(runDefinition(definition, config, 7)).rejects.toThrow(
      "관측의 형식",
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("execution rejects mismatched versions and never treats rejected preparation as a completed run", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const originalFetch = globalThis.fetch;
  let response: Record<string, unknown> = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "rejected",
    identity: null,
    accepted_steps: 0,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    failure: { stage: "prepare", diagnostic: "missing content" },
  };
  try {
    globalThis.fetch = async (_url, options) => {
      const input = JSON.parse(options!.body as string) as Record<
        string,
        unknown
      >;
      expect(input.source).toBe(source);
      expect(input.artifact_digest).toBe(definition.artifactDigest);
      return new Response(JSON.stringify(response), {
        headers: { "Content-Type": "application/json" },
      });
    };
    expect((await runDefinition(definition, config, 7)).status).toBe(
      "rejected",
    );
    response = { ...response, source_digest: "stale" };
    await expect(runDefinition(definition, config, 7)).rejects.toThrow(
      "일치하지",
    );
    response = {
      ...response,
      source_digest: definition.sourceDigest,
      status: "completed",
      failure: null,
    };
    await expect(runDefinition(definition, config, 7)).rejects.toThrow(
      "오류 상태",
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("equipment form observations reject malformed phases and mismatched source generations", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const witness = {
    time_ms: 0,
    actor: "employee",
    equipment_copy: "copy",
    operation: "Earned",
    pending: 1,
    confirmed: 2,
    form: {
      index: 1,
      progress: 1,
      wins_required: 2,
      name: "흐려진 미소",
      description: "공격력 증가",
      appearance: "mask-clouded",
      effect: { Attack: { amount: 10 } },
    },
    definition_id: "equipment:mask",
    source_file: "artifact",
    field_path: "equipment_attacks[0].program.victory_forms",
    battle_terminal: false,
    origin: {
      definition: "values[0].value.AttackProgram",
      calls: [],
      inputs: [],
    },
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_equipment_form: witness,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7)).latest_equipment_form,
    ).toEqual(witness);
    for (const invalid of [
      { ...witness, pending: 2 },
      { ...witness, form: { ...witness.form, progress: 2 } },
      { ...witness, form: { ...witness.form, wins_required: 0 } },
      {
        ...witness,
        form: { ...witness.form, effect: { Attack: { amount: -1 } } },
      },
      {
        ...witness,
        form: {
          ...witness.form,
          effect: { Attack: { amount: 10 }, MaxHealth: { amount: 100 } },
        },
      },
      { ...witness, source_file: "another-artifact" },
    ]) {
      body = { ...response, latest_equipment_form: invalid };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "장비 형태 관측",
      );
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("death burst observations reject invalid coordinates, counters and mismatched source generations", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const witness = {
    time_ms: 500,
    id: 1,
    actor: "employee",
    equipment_copy: "copy",
    victim: "enemy",
    center: { x: 4, y: 3 },
    due_ms: 500,
    attack: 100,
    base_damage: 100,
    operation: "Detonated",
    targets: 2,
    pending: 1,
    definition_id: "equipment:echo",
    source_file: "artifact",
    field_path: "equipment_attacks[0].program.death_burst",
    battle_terminal: false,
    origin: {
      definition: "values[0].value.AttackProgram",
      calls: [],
      inputs: [],
    },
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_equipment_death_burst: witness,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7)).latest_equipment_death_burst,
    ).toEqual(witness);
    for (const invalid of [
      { ...witness, id: 0 },
      { ...witness, operation: "Unknown" },
      { ...witness, pending: -1 },
      { ...witness, base_damage: 1.5 },
      { ...witness, center: { x: 0.5, y: 3 } },
      { ...witness, center: null },
      { ...witness, source_file: "another-artifact" },
    ]) {
      body = { ...response, latest_equipment_death_burst: invalid };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "처치 연쇄 폭발 관측",
      );
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("last will observations require protected HP, exit and the actual armor source", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const witness = {
    time_ms: 500,
    actor: "employee",
    equipment: {
      definition_uuid: "armor",
      runtime_item_instance_id: "runtime-copy",
      persistent_copy_uuid: "copy",
    },
    center: { x: 4, y: 3 },
    facing: "right",
    hp_before: 100,
    hp_protected: 1,
    hp_after: 0,
    attack: 100,
    base_damage: 300,
    targets: 1,
    incapacitated: true,
    definition_id: "armor",
    source_file: "artifact",
    field_path: "equipment_defenses[0].program.LastWill",
    battle_terminal: false,
    origin: {
      definition: "values[0].value.DefenseProgram",
      calls: [],
      inputs: [],
    },
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_last_will: witness,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7)).latest_last_will,
    ).toEqual(witness);
    for (const invalid of [
      { ...witness, hp_before: 0 },
      { ...witness, hp_protected: 2 },
      { ...witness, hp_after: 1 },
      { ...witness, incapacitated: false },
      {
        ...witness,
        equipment: { ...witness.equipment, persistent_copy_uuid: null },
      },
      { ...witness, facing: "forward" },
      { ...witness, targets: -1 },
      { ...witness, base_damage: 1.5 },
      { ...witness, center: { x: 0.5, y: 3 } },
      { ...witness, center: null },
      { ...witness, source_file: "another-artifact" },
    ]) {
      body = { ...response, latest_last_will: invalid };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "마지막 유언 관측",
      );
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("weapon stage observations retain actual displacement and reject stale or malformed sources", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const witness = {
    actor: "employee",
    target: "enemy",
    stage: "Upgraded",
    time_ms: 200,
    equipment: {
      definition_uuid: "weapon",
      runtime_item_instance_id: "runtime",
      persistent_copy_uuid: "copy",
    },
    interval_ms: 250,
    windup_ms: 50,
    push_distance_units: 100000,
    actual_hp_loss: 100,
    from_data_units: [4500000, 1500000],
    to_data_units: [4600000, 1500000],
    definition_id: "equipment:weapon",
    source_file: "artifact",
    field_path: "equipment_attacks[0].program.weapon_stages.upgraded",
    battle_terminal: false,
    origin: {
      definition: "values[0].value.AttackProgram",
      calls: [],
      inputs: [],
    },
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_weapon_stage: witness,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7)).latest_weapon_stage,
    ).toEqual(witness);
    for (const invalid of [
      { ...witness, stage: "Unknown" },
      { ...witness, interval_ms: 0 },
      { ...witness, from_data_units: [1.5, 0] },
      { ...witness, to_data_units: [1] },
      { ...witness, equipment: null },
      { ...witness, source_file: "stale-artifact" },
    ]) {
      body = { ...response, latest_weapon_stage: invalid };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        /무기 단계/,
      );
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("employee transformation observations distinguish the successor and reject stale or malformed evidence", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const witness = {
    actor: "employee-runtime",
    employee_uuid: "employee-owned",
    successor: "new-enemy",
    form: "authored-form",
    time_ms: 200,
    position: { x: 3, y: 4 },
    equipment: {
      definition_uuid: "accessory",
      runtime_item_instance_id: "runtime-copy",
      persistent_copy_uuid: "owned-copy",
    },
    definition_id: "equipment:accessory",
    source_file: "artifact",
    field_path: "equipment_defenses[0].program.EmployeeRevival",
    battle_terminal: false,
    origin: {
      definition: "values[0].value.DefenseProgram",
      calls: [],
      inputs: [],
    },
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_employee_transformation: witness,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7))
        .latest_employee_transformation,
    ).toEqual(witness);
    for (const invalid of [
      { ...witness, successor: witness.actor },
      { ...witness, position: { x: 1.5, y: 4 } },
      { ...witness, form: "" },
      { ...witness, time_ms: -1 },
      {
        ...witness,
        equipment: { ...witness.equipment, persistent_copy_uuid: null },
      },
      { ...witness, origin: { definition: 5, calls: [], inputs: [] } },
      { ...witness, source_file: "stale-artifact" },
    ]) {
      body = { ...response, latest_employee_transformation: invalid };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        /직원 변신/,
      );
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("primary damage evidence validates HP, contributor totals and source generation", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const contribution = {
    equipment: {
      definition_uuid: "accessory",
      runtime_item_instance_id: "runtime-copy",
      persistent_copy_uuid: "owned-copy",
    },
    percent: 25,
    definition_id: "equipment:accessory",
    source_file: "artifact",
    field_path: "equipment_attacks[0].program.primary_damage_percent",
  };
  const witness = {
    time_ms: 11,
    actor: "wearer",
    target: "enemy",
    action: 1,
    target_current_hp: 750,
    target_max_hp: 1000,
    percent: 25,
    battle_terminal: false,
    contributors: [contribution],
    origins: [
      { definition: "values[0].value.AttackProgram", calls: [], inputs: [] },
    ],
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_primary_damage: witness,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7)).latest_primary_damage,
    ).toEqual(witness);
    for (const invalid of [
      { ...witness, target: witness.actor },
      { ...witness, target_current_hp: 0 },
      { ...witness, target_max_hp: 749 },
      { ...witness, action: -1 },
      { ...witness, percent: 26 },
      { ...witness, origins: [] },
      { ...witness, contributors: [{ ...contribution, percent: 10001 }] },
      {
        ...witness,
        contributors: [
          {
            ...contribution,
            equipment: {
              ...contribution.equipment,
              persistent_copy_uuid: null,
            },
          },
        ],
      },
      {
        ...witness,
        contributors: [{ ...contribution, source_file: "stale-artifact" }],
      },
    ]) {
      body = { ...response, latest_primary_damage: invalid };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        /주 대상 피해/,
      );
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("nearby defense evidence validates counts, caps, signed stats and source generation", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const contribution = {
    equipment: {
      definition_uuid: "accessory",
      runtime_item_instance_id: "runtime-copy",
      persistent_copy_uuid: "owned-copy",
    },
    count: 3,
    per_enemy: 10,
    max_bonus: 25,
    bonus: 25,
    definition_id: "equipment:accessory",
    source_file: "artifact",
    field_path: "equipment_defenses[0].program.NearbyHostileDefense",
  };
  const witness = {
    time_ms: 11,
    actor: "wearer",
    bonus: 25,
    actual_defense: -5,
    battle_terminal: false,
    contributors: [contribution],
    origins: [
      { definition: "values[0].value.DefenseProgram", calls: [], inputs: [] },
    ],
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_nearby_defense: witness,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7)).latest_nearby_defense,
    ).toEqual(witness);
    for (const invalid of [
      { ...witness, actual_defense: 2147483648 },
      { ...witness, actual_defense: -2147483649 },
      { ...witness, actual_defense: 0.5 },
      { ...witness, bonus: 26 },
      { ...witness, origins: [] },
      { ...witness, contributors: [{ ...contribution, count: -1 }] },
      { ...witness, contributors: [{ ...contribution, count: 0.5 }] },
      { ...witness, contributors: [{ ...contribution, per_enemy: 0 }] },
      { ...witness, contributors: [{ ...contribution, bonus: 30 }] },
      {
        ...witness,
        contributors: [
          {
            ...contribution,
            equipment: {
              ...contribution.equipment,
              persistent_copy_uuid: null,
            },
          },
        ],
      },
      {
        ...witness,
        contributors: [{ ...contribution, source_file: "stale-artifact" }],
      },
    ]) {
      body = { ...response, latest_nearby_defense: invalid };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        /주변 적 방어력/,
      );
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("ready-wait evidence validates consumed time, cap, deadline and source generation", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const contribution = {
    equipment: {
      definition_uuid: "definition",
      runtime_item_instance_id: "runtime-copy",
      persistent_copy_uuid: "saved-copy",
    },
    waited_ms: 2999,
    per_second_percent: 10,
    max_percent: 50,
    percent: 29,
    until_ms: 19000,
    definition_id: "armor",
    source_file: "artifact",
    field_path: "equipment_defenses[0].program.ReadyWaitDamage",
  };
  const witness = {
    time_ms: 4000,
    actor: "wearer",
    skill_id: "skill",
    percent: 29,
    battle_terminal: false,
    contributors: [contribution],
    origins: [
      { definition: "values[0].value.DefenseProgram", calls: [], inputs: [] },
    ],
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_ready_wait: witness,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7)).latest_ready_wait,
    ).toEqual(witness);
    for (const invalid of [
      { ...witness, percent: 30 },
      { ...witness, skill_id: "" },
      { ...witness, origins: [] },
      ...[
        { waited_ms: 4001 },
        { waited_ms: -1 },
        { waited_ms: 0.5 },
        { per_second_percent: 0 },
        { max_percent: 10001 },
        { percent: 30 },
        { until_ms: 4000 },
        { source_file: "stale-artifact" },
        {
          equipment: { ...contribution.equipment, persistent_copy_uuid: null },
        },
      ].map((fields) => ({
        ...witness,
        contributors: [{ ...contribution, ...fields }],
      })),
    ]) {
      body = { ...response, latest_ready_wait: invalid };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        /시전 대기 피해 증가/,
      );
    }
    body = {
      ...response,
      latest_ready_wait: {
        ...witness,
        percent: 50,
        contributors: [
          {
            ...contribution,
            waited_ms: 4000,
            per_second_percent: 10000,
            percent: 50,
          },
        ],
      },
    };
    expect(
      (await runDefinition(definition, config, 7)).latest_ready_wait?.percent,
    ).toBe(50);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("area damage evidence validates unique count, contributor totals and source generation", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const contribution = {
    equipment: {
      definition_uuid: "accessory",
      runtime_item_instance_id: "runtime-copy",
      persistent_copy_uuid: "owned-copy",
    },
    percent: 25,
    definition_id: "equipment:accessory",
    source_file: "artifact",
    field_path: "equipment_attacks[0].program.area_damage_percent",
  };
  const witness = {
    time_ms: 11,
    actor: "wearer",
    action: 1,
    target_count: 3,
    percent: 25,
    battle_terminal: false,
    contributors: [contribution],
    origins: [
      { definition: "values[0].value.AttackProgram", calls: [], inputs: [] },
    ],
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_area_damage: witness,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7)).latest_area_damage,
    ).toEqual(witness);
    for (const invalid of [
      { ...witness, target_count: -1 },
      { ...witness, target_count: 1.5 },
      { ...witness, action: -1 },
      { ...witness, percent: 26 },
      { ...witness, origins: [] },
      { ...witness, contributors: [{ ...contribution, percent: 10001 }] },
      {
        ...witness,
        contributors: [
          {
            ...contribution,
            equipment: {
              ...contribution.equipment,
              persistent_copy_uuid: null,
            },
          },
        ],
      },
      {
        ...witness,
        contributors: [{ ...contribution, source_file: "stale-artifact" }],
      },
    ]) {
      body = { ...response, latest_area_damage: invalid };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        /범위 피해/,
      );
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("nearby ally damage evidence validates employee count, contributor totals and source generation", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const contribution = {
    equipment: {
      definition_uuid: "accessory",
      runtime_item_instance_id: "runtime-copy",
      persistent_copy_uuid: "owned-copy",
    },
    count: 3,
    percent: 25,
    definition_id: "equipment:accessory",
    source_file: "artifact",
    field_path: "equipment_attacks[0].program.nearby_ally_damage.percent",
  };
  const witness = {
    time_ms: 11,
    actor: "wearer",
    wearer: "wearer",
    target: "enemy",
    percent: 25,
    battle_terminal: false,
    contributors: [contribution],
    origins: [
      { definition: "values[0].value.AttackProgram", calls: [], inputs: [] },
    ],
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_nearby_ally_damage: witness,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7)).latest_nearby_ally_damage,
    ).toEqual(witness);
    for (const invalid of [
      { ...witness, contributors: [{ ...contribution, count: -1 }] },
      { ...witness, contributors: [{ ...contribution, count: 1.5 }] },
      { ...witness, wearer: "enemy" },
      { ...witness, percent: 26 },
      { ...witness, origins: [] },
      { ...witness, contributors: [{ ...contribution, percent: 10001 }] },
      {
        ...witness,
        contributors: [
          {
            ...contribution,
            equipment: {
              ...contribution.equipment,
              persistent_copy_uuid: null,
            },
          },
        ],
      },
      {
        ...witness,
        contributors: [{ ...contribution, source_file: "stale-artifact" }],
      },
    ]) {
      body = { ...response, latest_nearby_ally_damage: invalid };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        /주변 아군 피해/,
      );
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("resonance gain evidence validates fraction, clamped gain, actual copy and source generation", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const contribution = {
    equipment: {
      definition_uuid: "accessory",
      runtime_item_instance_id: "runtime-copy",
      persistent_copy_uuid: "owned-copy",
    },
    percent: 25,
    definition_id: "equipment:accessory",
    source_file: "artifact",
    field_path: "equipment_supports[0].program.resonance_gain.percent",
  };
  const witness = {
    time_ms: 11,
    actor: "wearer",
    channel: "automatic",
    base_amount: 1,
    increased_amount: 2,
    actual_gain: 1,
    before: 9,
    after: 10,
    max: 10,
    remainder_before: 75,
    remainder_after: 0,
    percent: 25,
    battle_terminal: false,
    contributors: [contribution],
    origins: [
      { definition: "values[0].value.SupportProgram", calls: [], inputs: [] },
    ],
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_resonance_gain: witness,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7)).latest_resonance_gain,
    ).toEqual(witness);
    const unownedCopy = {
      ...witness,
      contributors: [
        {
          ...contribution,
          equipment: { ...contribution.equipment, persistent_copy_uuid: null },
        },
      ],
    };
    body = { ...response, latest_resonance_gain: unownedCopy };
    expect(
      (await runDefinition(definition, config, 7)).latest_resonance_gain,
    ).toEqual(unownedCopy);
    for (const invalid of [
      { ...witness, remainder_before: 100 },
      { ...witness, remainder_after: 1 },
      { ...witness, channel: "cost" },
      { ...witness, before: 10 },
      { ...witness, actual_gain: 2 },
      { ...witness, increased_amount: 0 },
      { ...witness, base_amount: 1.5 },
      { ...witness, percent: 26 },
      { ...witness, origins: [] },
      { ...witness, contributors: [{ ...contribution, percent: 10001 }] },
      {
        ...witness,
        contributors: [
          {
            ...contribution,
            equipment: {
              ...contribution.equipment,
              persistent_copy_uuid: "",
            },
          },
        ],
      },
      {
        ...witness,
        contributors: [{ ...contribution, source_file: "stale-artifact" }],
      },
    ]) {
      body = { ...response, latest_resonance_gain: invalid };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        /공명 획득/,
      );
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("actual heal reaction evidence validates both providers, finite outcomes and generation", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const shield = {
    actual_heal_percent: 25,
    max_hp_cap_percent: 20,
    before: 1,
    after: 3,
    cap: 20,
    remainder_before: 0,
    remainder_after: 0,
    expires_at_ms: 5010,
  };
  const haste = {
    attack_speed_points: 30,
    expires_at_ms: 3010,
    refreshed: true,
  };
  const contribution = {
    equipment: {
      definition_uuid: "armor",
      runtime_item_instance_id: "armor-copy",
      persistent_copy_uuid: "owned-copy",
    },
    definition_id: "armor",
    source_file: "artifact",
    field_path: "equipment_supports[0].program.actual_heal_reaction",
    self_shield: shield,
    target_haste: null,
  };
  const second = {
    ...contribution,
    equipment: {
      definition_uuid: "accessory",
      runtime_item_instance_id: "accessory-copy",
      persistent_copy_uuid: null,
    },
    self_shield: null,
    target_haste: haste,
  };
  const witness = {
    time_ms: 10,
    actor: "healer",
    target: "ally",
    actual_healed: 8,
    healing_source: { actor: "healer", action_ordinal: 1 },
    battle_terminal: false,
    contributors: [contribution, second],
    origins: [
      { definition: "values[0].value.SupportProgram", calls: [], inputs: [] },
      { definition: "values[1].value.SupportProgram", calls: [], inputs: [] },
    ],
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_actual_heal_reaction: witness,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7)).latest_actual_heal_reaction,
    ).toEqual(witness);
    for (const invalid of [
      { ...witness, actual_healed: 0 },
      { ...witness, target: "healer" },
      { ...witness, time_ms: -1 },
      { ...witness, origins: [] },
      { ...witness, contributors: [contribution, contribution] },
      { ...witness, healing_source: { actor: "healer", action_ordinal: -1 } },
      ...[
        { after: 21 },
        { after: 0 },
        { actual_heal_percent: 10001 },
        { max_hp_cap_percent: 0 },
        { remainder_after: 100 },
        { after: 20, remainder_after: 1 },
        { expires_at_ms: 10 },
      ].map((change) => ({
        ...witness,
        contributors: [
          { ...contribution, self_shield: { ...shield, ...change } },
          second,
        ],
      })),
      ...[
        { attack_speed_points: 0 },
        { expires_at_ms: 10 },
        { refreshed: "yes" },
      ].map((change) => ({
        ...witness,
        contributors: [
          contribution,
          { ...second, target_haste: { ...haste, ...change } },
        ],
      })),
      {
        ...witness,
        contributors: [{ ...contribution, self_shield: null }, second],
      },
      {
        ...witness,
        contributors: [
          { ...contribution, source_file: "old-generation" },
          second,
        ],
      },
    ]) {
      body = { ...response, latest_actual_heal_reaction: invalid };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        /실제 치유 반응/,
      );
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("threshold counter evidence distinguishes consumption, immunity and source generation", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const witness = {
    time_ms: 10,
    battle_terminal: false,
    definition_id: "frostbite",
    skill_id: null,
    cast_sequence: null,
    source_file: "artifact",
    field_path: "equipment_attacks[0].program.on_hit_status",
    target_unit: "enemy",
    equipment: {
      definition_uuid: "weapon",
      runtime_item_instance_id: "item",
      persistent_copy_uuid: "copy",
    },
    area_source: null,
    before: 4,
    added: 1,
    after: 0,
    threshold: 5,
    control: "Stun",
    triggered: true,
    control_applied: true,
    counter_expires_at_ms: null,
    control_expires_at_ms: 1010,
    origin: {
      definition: "values[0].value.AttackProgram",
      calls: [],
      inputs: [],
    },
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_hit_counter: witness,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7)).latest_hit_counter,
    ).toEqual(witness);
    const immune = {
      ...witness,
      control_applied: false,
      control_expires_at_ms: null,
    };
    body = { ...response, latest_hit_counter: immune };
    expect(
      (await runDefinition(definition, config, 7)).latest_hit_counter,
    ).toEqual(immune);
    for (const change of [
      { threshold: 0 },
      { threshold: 1001 },
      { before: 5 },
      { added: 0 },
      { added: 6 },
      { after: 5 },
      { control: "Freeze" },
      { triggered: false },
      { counter_expires_at_ms: 30 },
      { control_expires_at_ms: 10 },
      { control_applied: false },
      { source_file: "old-artifact" },
      { cast_sequence: -1 },
    ]) {
      body = { ...response, latest_hit_counter: { ...witness, ...change } };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow();
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("finite protection evidence validates authored properties and source generation", async () => {
  const source = "version: 2";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const properties = {
    damage_invulnerable: true,
    basic_attack_locked: true,
    active_skill_locked: true,
    received_heal_percent: 50,
  };
  const witness = {
    time_ms: 10,
    battle_terminal: false,
    definition_id: "armor",
    skill_id: null,
    cast_sequence: null,
    source_file: "artifact",
    field_path: "equipment_states[0].program.handlers[0].body",
    target_unit: "wearer",
    equipment: {
      definition_uuid: "armor",
      runtime_item_instance_id: "item",
      persistent_copy_uuid: "copy",
    },
    area_source: null,
    activation: 1,
    ended: null,
    expires_at_ms: 3010,
    before: { attack: 10, defense: 20 },
    after: { attack: 10, defense: 20 },
    target_state: properties,
    origin: {
      definition: "values[0].value.StateProgram",
      calls: [],
      inputs: [],
    },
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: witness,
    latest_area: null,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7)).latest_sustained
        ?.target_state,
    ).toEqual(properties);
    for (const invalid of [
      { ...properties, damage_invulnerable: 1 },
      { ...properties, basic_attack_locked: null },
      { ...properties, active_skill_locked: "true" },
      { ...properties, received_heal_percent: -1 },
      { ...properties, received_heal_percent: 10001 },
    ]) {
      body = {
        ...response,
        latest_sustained: { ...witness, target_state: invalid },
      };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "지속 효과 관측",
      );
    }
    body = {
      ...response,
      latest_sustained: { ...witness, source_file: "stale-artifact" },
    };
    await expect(runDefinition(definition, config, 7)).rejects.toThrow();
    body = {
      ...response,
      latest_sustained: { ...witness, target_state: undefined },
    };
    expect(
      (await runDefinition(definition, config, 7)).latest_sustained
        ?.target_state,
    ).toBeUndefined();
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("conditional damage validates sampled HP, condition result, totals and source generation", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const contribution = {
    equipment: {
      definition_uuid: "accessory",
      runtime_item_instance_id: "runtime-copy",
      persistent_copy_uuid: "owned-copy",
    },
    matched: true,
    percent: 25,
    definition_id: "equipment:accessory",
    source_file: "artifact",
    field_path: "equipment_attacks[0].program.conditional_damage.percent",
  };
  const witness = {
    time_ms: 11,
    actor: "wearer",
    wearer: "wearer",
    target: "enemy",
    wearer_hp: 80,
    wearer_max_hp: 100,
    target_hp: 200,
    target_max_hp: 300,
    target_movement_impaired: true,
    percent: 25,
    battle_terminal: false,
    contributors: [contribution],
    origins: [
      { definition: "values[0].value.AttackProgram", calls: [], inputs: [] },
    ],
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_conditional_damage: witness,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7)).latest_conditional_damage,
    ).toEqual(witness);
    for (const invalid of [
      { ...witness, contributors: [{ ...contribution, matched: "yes" }] },
      { ...witness, contributors: [{ ...contribution, matched: false }] },
      { ...witness, wearer_hp: 101 },
      { ...witness, target_hp: 0 },
      { ...witness, target_movement_impaired: "yes" },
      { ...witness, wearer: "enemy" },
      { ...witness, percent: 26 },
      { ...witness, origins: [] },
      { ...witness, contributors: [{ ...contribution, percent: 10001 }] },
      {
        ...witness,
        contributors: [
          {
            ...contribution,
            equipment: {
              ...contribution.equipment,
              persistent_copy_uuid: null,
            },
          },
        ],
      },
      {
        ...witness,
        contributors: [{ ...contribution, source_file: "stale-artifact" }],
      },
    ]) {
      body = { ...response, latest_conditional_damage: invalid };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        /조건부 피해/,
      );
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("basic slow evidence validates percentage and source generation", async () => {
  const source = "version: 2";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const properties = 20;
  const witness = {
    time_ms: 10,
    battle_terminal: false,
    definition_id: "weapon",
    skill_id: null,
    cast_sequence: null,
    source_file: "artifact",
    field_path: "equipment_attacks[0].program.on_hit_status",
    target_unit: "enemy",
    equipment: {
      definition_uuid: "weapon",
      runtime_item_instance_id: "item",
      persistent_copy_uuid: "copy",
    },
    area_source: null,
    activation: 1,
    ended: null,
    expires_at_ms: 3010,
    before: { attack: 10, defense: 20 },
    after: { attack: 10, defense: 20 },
    basic_speed_reduction_percent: properties,
    origin: {
      definition: "values[0].value.AttackProgram",
      calls: [],
      inputs: [],
    },
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: witness,
    latest_area: null,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7)).latest_sustained
        ?.basic_speed_reduction_percent,
    ).toEqual(properties);
    for (const invalid of [0, -1, 100, 20.5, "20", true]) {
      body = {
        ...response,
        latest_sustained: {
          ...witness,
          basic_speed_reduction_percent: invalid,
        },
      };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "지속 효과 관측",
      );
    }
    body = {
      ...response,
      latest_sustained: { ...witness, source_file: "stale-artifact" },
    };
    await expect(runDefinition(definition, config, 7)).rejects.toThrow();
    body = {
      ...response,
      latest_sustained: {
        ...witness,
        basic_speed_reduction_percent: undefined,
      },
    };
    expect(
      (await runDefinition(definition, config, 7)).latest_sustained
        ?.basic_speed_reduction_percent,
    ).toBeUndefined();
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("incoming flat evidence validates allowance conservation and source generation", async () => {
  const source = "version: 2";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const witness = {
    time_ms: 10,
    target: "wearer",
    equipment: {
      definition_uuid: "armor",
      runtime_item_instance_id: "item",
      persistent_copy_uuid: "copy",
    },
    strike: 2,
    defense: 20,
    allowance: 10,
    before: 10,
    used: 7,
    remaining: 3,
    definition_id: "armor",
    source_file: "artifact",
    field_path:
      "equipment_defenses[0].program.PhysicalBasicFlatReduction.amount",
    incoming: { actor: "enemy", action_ordinal: 1 },
    battle_terminal: false,
    origin: {
      definition: "values[0].value.DefenseProgram",
      calls: [],
      inputs: [],
    },
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_incoming_flat: witness,
    latest_area: null,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7)).latest_incoming_flat?.used,
    ).toEqual(7);
    for (const patch of [
      { used: 8 },
      { before: 11 },
      { remaining: -1 },
      { allowance: 2147483648 },
      { defense: 1.5 },
      { defense: -2147483649 },
      { incoming: { actor: "enemy", action_ordinal: -1 } },
      { source_file: "stale-artifact" },
    ]) {
      body = { ...response, latest_incoming_flat: { ...witness, ...patch } };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "물리 기본 타격 차감 관측",
      );
    }
    body = {
      ...response,
      latest_incoming_flat: {
        ...witness,
        defense: -20,
        allowance: 0,
        before: 0,
        used: 0,
        remaining: 0,
        incoming: { actor: "enemy", action_ordinal: null },
      },
    };
    expect(
      (await runDefinition(definition, config, 7)).latest_incoming_flat?.used,
    ).toBe(0);
    body = { ...response, latest_incoming_flat: undefined };
    expect(
      (await runDefinition(definition, config, 7)).latest_incoming_flat,
    ).toBeUndefined();
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("recipient support evidence checks heal composition and duration arithmetic", async () => {
  const source = "version: 2";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const common = {
    time_ms: 10,
    target: "wearer",
    equipment: {
      definition_uuid: "brooch",
      runtime_item_instance_id: "item",
      persistent_copy_uuid: "copy",
    },
    definition_id: "brooch",
    source_file: "artifact",
    incoming: { actor: "enemy", action_ordinal: null },
    battle_terminal: false,
    origin: {
      definition: "values[0].value.SupportProgram",
      calls: [],
      inputs: [],
    },
  };
  const healing = {
    ...common,
    actor: "healer",
    field_path: "equipment_supports[0].program.received_heal_percent",
    blocked_hostiles: 2,
    percent: 40,
    other_percent: 10,
    before: 20,
    after: 30,
  };
  const control = {
    ...common,
    caster: "enemy",
    field_path: "equipment_supports[0].program.control_duration",
    kind: "Freeze",
    reduction_percent: 50,
    original_ms: 1001,
    effective_ms: 501,
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_received_heal: healing,
    latest_control_duration: control,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    const result = await runDefinition(definition, config, 7);
    expect(result.latest_received_heal?.after).toBe(30);
    expect(result.latest_control_duration?.effective_ms).toBe(501);
    for (const patch of [
      { after: 31 },
      { percent: 10001 },
      { blocked_hostiles: -1 },
      { before: 2147483648 },
      { other_percent: 4294967296 },
      { source_file: "stale" },
      { incoming: { actor: null, action_ordinal: -1 } },
    ]) {
      body = { ...response, latest_received_heal: { ...healing, ...patch } };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        /관측/,
      );
    }
    for (const patch of [
      { effective_ms: 500 },
      { original_ms: 0 },
      { effective_ms: 0 },
      { kind: "Knockback" },
      { reduction_percent: 100 },
      { reduction_percent: 0 },
      { source_file: "stale" },
    ]) {
      body = { ...response, latest_control_duration: { ...control, ...patch } };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        /관측/,
      );
    }
    body = {
      ...response,
      latest_received_heal: {
        ...healing,
        actor: null,
        incoming: { actor: null, action_ordinal: null },
      },
    };
    expect(
      (await runDefinition(definition, config, 7)).latest_received_heal?.actor,
    ).toBeNull();
    body = {
      ...response,
      latest_received_heal: undefined,
      latest_control_duration: undefined,
    };
    expect(
      (await runDefinition(definition, config, 7)).latest_control_duration,
    ).toBeUndefined();
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("damage taken evidence validates percentage and source generation", async () => {
  const source = "version: 2";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const properties = 20;
  const witness = {
    time_ms: 10,
    battle_terminal: false,
    definition_id: "weapon",
    skill_id: null,
    cast_sequence: null,
    source_file: "artifact",
    field_path: "equipment_attacks[0].program.on_hit_status",
    target_unit: "enemy",
    equipment: {
      definition_uuid: "weapon",
      runtime_item_instance_id: "item",
      persistent_copy_uuid: "copy",
    },
    area_source: null,
    activation: 1,
    ended: null,
    expires_at_ms: 3010,
    before: { attack: 10, defense: 20 },
    after: { attack: 10, defense: 20 },
    damage_taken_increase_percent: properties,
    origin: {
      definition: "values[0].value.AttackProgram",
      calls: [],
      inputs: [],
    },
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: witness,
    latest_area: null,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    expect(
      (await runDefinition(definition, config, 7)).latest_sustained
        ?.damage_taken_increase_percent,
    ).toEqual(properties);
    for (const invalid of [0, -1, 2147483648, 20.5, "20", true]) {
      body = {
        ...response,
        latest_sustained: {
          ...witness,
          damage_taken_increase_percent: invalid,
        },
      };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        "지속 효과 관측",
      );
    }
    body = {
      ...response,
      latest_sustained: { ...witness, source_file: "stale-artifact" },
    };
    await expect(runDefinition(definition, config, 7)).rejects.toThrow();
    body = {
      ...response,
      latest_sustained: {
        ...witness,
        damage_taken_increase_percent: undefined,
      },
    };
    expect(
      (await runDefinition(definition, config, 7)).latest_sustained
        ?.damage_taken_increase_percent,
    ).toBeUndefined();
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("conditional reduction validates strongest copy, distance, incoming source and generation", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const contribution = {
    equipment: {
      definition_uuid: "accessory",
      runtime_item_instance_id: "runtime-copy",
      persistent_copy_uuid: "owned-copy",
    },
    matched: true,
    percent: 25,
    definition_id: "equipment:accessory",
    source_file: "artifact",
    field_path: "equipment_defenses[0].program.ConditionalReduction.percent",
  };
  const witness = {
    time_ms: 11,
    actor: "enemy",
    wearer: "wearer",
    target: "enemy",
    wearer_hp: 80,
    wearer_max_hp: 100,
    target_hp: 200,
    target_max_hp: 300,
    target_movement_impaired: null,
    distance_squared: 9,
    equipment: contribution.equipment,
    incoming: { actor: "enemy", action_ordinal: 1 },
    percent: 25,
    battle_terminal: false,
    contributors: [contribution],
    origins: [
      { definition: "values[0].value.DefenseProgram", calls: [], inputs: [] },
    ],
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_conditional_reduction: witness,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    const weaker = {
      ...contribution,
      percent: 10,
      equipment: {
        ...contribution.equipment,
        runtime_item_instance_id: "weaker",
        persistent_copy_uuid: "other",
      },
    };
    body = {
      ...response,
      latest_conditional_reduction: {
        ...witness,
        contributors: [contribution, weaker],
        origins: [witness.origins[0], witness.origins[0]],
      },
    };
    expect(
      (await runDefinition(definition, config, 7)).latest_conditional_reduction
        ?.percent,
    ).toBe(25);
    body = response;
    expect(
      (await runDefinition(definition, config, 7)).latest_conditional_reduction,
    ).toEqual(witness);
    for (const invalid of [
      { ...witness, distance_squared: -1 },
      { ...witness, distance_squared: "far" },
      { ...witness, incoming: { actor: "ally", action_ordinal: 1 } },
      { ...witness, incoming: { actor: "enemy", action_ordinal: -1 } },
      {
        ...witness,
        equipment: {
          ...contribution.equipment,
          runtime_item_instance_id: "unselected",
        },
      },
      { ...witness, contributors: [{ ...contribution, matched: "yes" }] },
      { ...witness, contributors: [{ ...contribution, matched: false }] },
      { ...witness, wearer_hp: 101 },
      { ...witness, target_hp: 0 },
      { ...witness, target_movement_impaired: "yes" },
      { ...witness, wearer: "enemy" },
      { ...witness, percent: 26 },
      { ...witness, origins: [] },
      { ...witness, contributors: [{ ...contribution, percent: 100 }] },
      {
        ...witness,
        contributors: [
          {
            ...contribution,
            equipment: {
              ...contribution.equipment,
              persistent_copy_uuid: null,
            },
          },
        ],
      },
      {
        ...witness,
        contributors: [{ ...contribution, source_file: "stale-artifact" }],
      },
    ]) {
      body = { ...response, latest_conditional_reduction: invalid };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        /조건부 (피해|감면)/,
      );
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("shield beneficiary validates provider, personal pool, bounds and generation", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const witness = {
    time_ms: 11,
    actor: "recipient",
    beneficiary: "recipient",
    target: "enemy",
    provider: "provider",
    percent: 20,
    shield_activation: 1,
    shield_remaining: 50,
    battle_terminal: false,
    definition_id: "equipment:ring",
    source_file: "artifact",
    field_path:
      "equipment_supports[0].program.shield_beneficiary_damage_percent",
    equipment: {
      definition_uuid: "ring",
      runtime_item_instance_id: "runtime-copy",
      persistent_copy_uuid: "owned-copy",
    },
    shield_source: { actor: "provider", action_ordinal: 1 },
    origin: {
      definition: "values[0].value.SupportProgram",
      calls: [],
      inputs: [],
    },
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_shield_beneficiary_damage: witness,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    body = response;
    expect(
      (await runDefinition(definition, config, 7))
        .latest_shield_beneficiary_damage,
    ).toEqual(witness);
    for (const invalid of [
      { ...witness, percent: 0 },
      { ...witness, percent: 10001 },
      { ...witness, shield_activation: 0 },
      { ...witness, shield_remaining: 0 },
      { ...witness, shield_remaining: 4294967296 },
      { ...witness, time_ms: -1 },
      { ...witness, provider: 42 },
      { ...witness, shield_source: { actor: "other", action_ordinal: 1 } },
      { ...witness, shield_source: { actor: "provider", action_ordinal: -1 } },
      {
        ...witness,
        equipment: { ...witness.equipment, persistent_copy_uuid: 42 },
      },
      { ...witness, source_file: "stale-artifact" },
      { ...witness, origin: {} },
    ]) {
      body = { ...response, latest_shield_beneficiary_damage: invalid };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        /보호막 수혜 피해 증가/,
      );
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("wearer reduction validates current HP percentage source and generation", async () => {
  const source = "version: 1";
  const definition: Definition = {
    ticket: { source, epoch: 1, revision: 2, request_id: 3 },
    sourceDigest: await digest(source),
    artifact: "definition",
    artifactDigest: await digest("definition"),
  };
  const config = parseTrialConfig(
    JSON.stringify({
      root: "/test-content",
      selection: { packs: ["official"], winners: [] },
      scenario: {},
    }),
  );
  const witness = {
    time_ms: 11,
    target: "wearer",
    strike: 1,
    current_hp: 1000,
    max_hp: 1000,
    percent: 0,
    battle_terminal: false,
    definition_id: "equipment:armor",
    source_file: "artifact",
    field_path: "equipment_defenses[0].program.WearerReduction.percent",
    equipment: {
      definition_uuid: "armor",
      runtime_item_instance_id: "runtime-copy",
      persistent_copy_uuid: "owned-copy",
    },
    incoming: { actor: null, action_ordinal: null },
    origin: {
      definition: "values[0].value.DefenseProgram.WearerReduction",
      calls: [],
      inputs: [],
    },
  };
  const response = {
    epoch: 1,
    revision: 2,
    request_id: 7,
    source_digest: definition.sourceDigest,
    artifact_digest: definition.artifactDigest,
    status: "completed",
    accepted_steps: 1,
    failure: null,
    latest_effect: null,
    latest_sustained: null,
    latest_area: null,
    latest_wearer_reduction: witness,
    identity: {
      run_id: "run",
      source_digest: definition.sourceDigest,
      artifact_digest: definition.artifactDigest,
      artifact_path: "artifact",
      content: { semantic_sha256: "content" },
      scenario_digest: await digest("{}"),
    },
  };
  const originalFetch = globalThis.fetch;
  try {
    let body: unknown = response;
    globalThis.fetch = async () =>
      new Response(JSON.stringify(body), {
        headers: { "Content-Type": "application/json" },
      });
    body = response;
    expect(
      (await runDefinition(definition, config, 7)).latest_wearer_reduction,
    ).toEqual(witness);
    for (const invalid of [
      { ...witness, percent: -1 },
      { ...witness, percent: 100 },
      { ...witness, current_hp: 0 },
      { ...witness, current_hp: 1001 },
      { ...witness, max_hp: 4294967296 },
      { ...witness, time_ms: -1 },
      { ...witness, incoming: { actor: 42, action_ordinal: null } },
      { ...witness, incoming: { actor: "enemy", action_ordinal: -1 } },
      {
        ...witness,
        equipment: { ...witness.equipment, persistent_copy_uuid: 42 },
      },
      { ...witness, source_file: "stale-artifact" },
      { ...witness, origin: {} },
    ]) {
      body = { ...response, latest_wearer_reduction: invalid };
      await expect(runDefinition(definition, config, 7)).rejects.toThrow(
        /현재 HP 기반 감면/,
      );
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});
