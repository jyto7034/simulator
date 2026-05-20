export type SkillKind = "Targeted" | "Untargeted" | string;

export type TargetDraft =
  | { type: "SelfUnit" }
  | { type: "EnemySingle"; rule: "Nearest" | "CurrentTarget" | "LowestHealthEnemy" | string }
  | { type: "Allies"; area: AreaDraft }
  | { type: "Enemies"; area: AreaDraft };

export type AreaDraft =
  | { type: "All" }
  | { type: "RadiusChebyshev"; radiusTiles: number }
  | { type: "Line"; lengthTiles: number };

export type DeliveryDraft =
  | { type: "Instant" }
  | {
      type: "Projectile";
      speedUnitsPerMs: number;
      radiusUnits?: number;
      hitTargets?: string;
      piercing?: boolean;
      despawnOnHit?: boolean | null;
      maxHits?: number | null;
    }
  | {
      type: "Area";
      shape: AreaShapeDraft;
      anchor: "CastTarget" | "ImpactContext" | "Caster" | "CastTargetStart" | "ImpactContextStart" | string;
      hitTargets?: string;
      includeCaster?: boolean;
      tickPolicy?: string;
      durationMs?: number;
      tickIntervalMs?: number | null;
    };

export type AreaShapeDraft =
  | { type: "Circle"; radiusUnits: number }
  | { type: "Line"; lengthUnits: number }
  | { type: "Box"; widthUnits: number; heightUnits: number }
  | { type: "Rectangle"; widthUnits: number; lengthUnits: number }
  | { type: "Cone"; angleDegrees: number; lengthUnits: number };

export type DamageTypeDraft = "Physical" | "Magic" | "True" | string;

export interface DamageModifiersDraft {
  armorPenetrationFlat?: number;
  magicResistPenetrationFlat?: number;
  armorPenetrationPercent?: number;
  magicResistPenetrationPercent?: number;
  damageAmpPercent?: number;
  damageReductionPercent?: number;
  physicalDamageAmpPercent?: number;
  magicDamageAmpPercent?: number;
  trueDamageAmpPercent?: number;
  physicalDamageReductionPercent?: number;
  magicDamageReductionPercent?: number;
  trueDamageReductionPercent?: number;
  critChancePercent?: number;
  critDamagePercent?: number;
}

export type EffectDraft =
  | { type: "Damage"; amount: number; damageType: DamageTypeDraft }
  | { type: "ModifyDamage"; modifiers: DamageModifiersDraft }
  | { type: "Heal"; amount: number }
  | { type: "ModifyResonance"; amount: number }
  | { type: "ModifyStats"; modifier: StatModifierDraft }
  | { type: "ApplyBuff"; buffId: string; durationMs: number }
  | { type: "ExtraAttack"; count: number };

export interface StatModifierDraft {
  stat: "MaxHealth" | "Attack" | "Defense" | "AttackIntervalMs" | "MoveSpeedUnitsPerMs" | string;
  kind: "Flat" | "Percent" | string;
  value: number;
}

export type StepConditionDraft =
  | { type: "Always" }
  | { type: "IfPreviousStepDealtDamage" }
  | { type: "IfCasterHasBuff"; buffId: string; minStacks: number };

export type StepRepeatDraft =
  | { type: "None" }
  | { type: "Times"; count: number }
  | { type: "ByBuffStacks"; unit?: string; buffId: string; max?: number };

export interface SkillPresentationDraft {
  castState?: string;
  projectileVfxId?: string;
  impactVfxId?: string;
  targetAnchor?: string;
}

export interface SkillStepDraft {
  id: string;
  delayMs: number;
  rangeTiles: number;
  condition: StepConditionDraft;
  repeat: StepRepeatDraft;
  target: TargetDraft;
  delivery: DeliveryDraft;
  effects: EffectDraft[];
  presentation: SkillPresentationDraft;
}

export interface SkillDraft {
  id: string;
  name: string;
  kind: SkillKind;
  focusTimeMs: number;
  focusPermissions: {
    allowsMove: boolean;
    allowsBasicAttack: boolean;
  };
  steps: SkillStepDraft[];
}

export interface ValidationMessage {
  severity: "error" | "warning" | "info";
  label: string;
  detail: string;
  stepId?: string;
}

export function describeTarget(target: TargetDraft): string {
  switch (target.type) {
    case "SelfUnit":
      return "Self";
    case "EnemySingle":
      return `Enemy single / ${target.rule}`;
    case "Allies":
      return `Allies / ${describeArea(target.area)}`;
    case "Enemies":
      return `Enemies / ${describeArea(target.area)}`;
  }
}

export function describeArea(area: AreaDraft): string {
  switch (area.type) {
    case "All":
      return "All";
    case "RadiusChebyshev":
      return `Radius ${area.radiusTiles} tiles`;
    case "Line":
      return `Line ${area.lengthTiles} tiles`;
  }
}

export function describeDelivery(delivery: DeliveryDraft): string {
  switch (delivery.type) {
    case "Instant":
      return "Instant";
    case "Projectile":
      return `Projectile / ${delivery.speedUnitsPerMs.toLocaleString()} units/ms`;
    case "Area":
      return `${describeAreaShape(delivery.shape ?? fallbackAreaShape())} / ${delivery.anchor}`;
  }
}

export function describeAreaShape(shape: AreaShapeDraft): string {
  switch (shape.type) {
    case "Circle":
      return `Circle ${shape.radiusUnits.toLocaleString()}u`;
    case "Line":
      return `Line ${shape.lengthUnits.toLocaleString()}u`;
    case "Box":
      return `Box ${shape.widthUnits.toLocaleString()}x${shape.heightUnits.toLocaleString()}u`;
    case "Rectangle":
      return `Rectangle ${shape.widthUnits.toLocaleString()}x${shape.lengthUnits.toLocaleString()}u`;
    case "Cone":
      return `Cone ${shape.angleDegrees}deg / ${shape.lengthUnits.toLocaleString()}u`;
  }
}

export function describeEffect(effect: EffectDraft): string {
  switch (effect.type) {
    case "Damage":
      return `${effect.damageType} damage ${effect.amount}`;
    case "ModifyDamage":
      return describeDamageModifiers(effect.modifiers);
    case "Heal":
      return `Heal ${effect.amount}`;
    case "ModifyResonance":
      return `Resonance ${effect.amount >= 0 ? "+" : ""}${effect.amount}`;
    case "ModifyStats":
      return `Modify ${effect.modifier.stat} / ${effect.modifier.kind} ${effect.modifier.value}`;
    case "ApplyBuff":
      return `Apply ${effect.buffId} for ${effect.durationMs}ms`;
    case "ExtraAttack":
      return `Extra attack x${effect.count}`;
  }
}

function describeDamageModifiers(modifiers: DamageModifiersDraft): string {
  const entries = Object.entries(modifiers).filter(([, value]) => value !== undefined && value !== 0);
  if (entries.length === 0) {
    return "Modify damage";
  }
  return entries
    .map(([key, value]) => `${key} ${Number(value) >= 0 ? "+" : ""}${value}`)
    .join(" / ");
}

export function validateSkillDraft(skill: SkillDraft): ValidationMessage[] {
  const messages: ValidationMessage[] = [];

  if (!skill.id.trim()) {
    messages.push({
      severity: "error",
      label: "Missing skill ID",
      detail: "A skill needs a stable ID before it can be exported.",
    });
  }

  if (skill.steps.length === 0) {
    messages.push({
      severity: "error",
      label: "No steps",
      detail: "Add at least one step so the skill has runtime behavior.",
    });
  }

  const seenStepIds = new Set<string>();
  for (const step of skill.steps) {
    if (!step.id.trim()) {
      messages.push({
        severity: "error",
        label: "Missing step ID",
        detail: "Every step needs a stable ID for timeline and presentation references.",
      });
    } else if (seenStepIds.has(step.id)) {
      messages.push({
        severity: "error",
        label: "Duplicate step ID",
        detail: `Step '${step.id}' appears more than once.`,
        stepId: step.id,
      });
    }
    seenStepIds.add(step.id);

    if (step.rangeTiles < 0) {
      messages.push({
        severity: "error",
        label: "Invalid range",
        detail: "Range cannot be negative.",
        stepId: step.id,
      });
    }

    if (step.effects.length === 0) {
      messages.push({
        severity: "warning",
        label: "No effects",
        detail: "This step resolves targets but does not apply any effects.",
        stepId: step.id,
      });
    }

    if (step.delivery.type === "Projectile" && step.delivery.speedUnitsPerMs <= 0) {
      messages.push({
        severity: "error",
        label: "Invalid projectile speed",
        detail: "Projectile speed must be greater than zero.",
        stepId: step.id,
      });
    }
  }

  if (messages.length === 0) {
    messages.push({
      severity: "info",
      label: "Ready for core validation",
      detail: "The draft passes local UI checks. Rust validation should run before export.",
    });
  }

  return messages;
}

export function skillToRonPreview(skill: SkillDraft): string {
  const steps = skill.steps
    .map((step) => {
      const effects = step.effects.map((effect) => `                        ${describeEffectRon(effect)},`).join("\n");
      return `                SkillStepDef(
                    id: "${step.id}",
                    delay_ms: ${step.delayMs},
                    range_tiles: ${step.rangeTiles},
                    target: ${describeTargetRon(step.target)},
                    when: ${describeConditionRon(step.condition)},
                    repeat: ${describeRepeatRon(step.repeat)},
                    delivery: ${describeDeliveryRon(step.delivery)},
                    effects: [
${effects}
                    ],
                ),`;
    })
    .join("\n");

  return `SkillDef(
    id: "${skill.id}",
    name: "${skill.name}",
    kind: ${skill.kind},
    focus_time_ms: ${skill.focusTimeMs},
    focus_permissions: FocusPermissions(
        allows_move: ${skill.focusPermissions.allowsMove},
        allows_basic_attack: ${skill.focusPermissions.allowsBasicAttack},
    ),
    steps: [
${steps}
    ],
)`;
}

function describeTargetRon(target: TargetDraft): string {
  switch (target.type) {
    case "SelfUnit":
      return "SelfUnit";
    case "EnemySingle":
      return `EnemySingle(rule: ${target.rule})`;
    case "Allies":
      return `Allies(area: ${describeAreaRon(target.area)})`;
    case "Enemies":
      return `Enemies(area: ${describeAreaRon(target.area)})`;
  }
}

function describeAreaRon(area: AreaDraft): string {
  switch (area.type) {
    case "All":
      return "All";
    case "RadiusChebyshev":
      return `RadiusChebyshev(radius_tiles: ${area.radiusTiles})`;
    case "Line":
      return `Line(length_tiles: ${area.lengthTiles})`;
  }
}

function describeDeliveryRon(delivery: DeliveryDraft): string {
  switch (delivery.type) {
    case "Instant":
      return "Instant";
    case "Projectile":
      return `Projectile(
                        speed_units_per_ms: ${delivery.speedUnitsPerMs},
                        collision: (
                            radius_units: ${delivery.radiusUnits ?? 0},
                            hit_targets: ${delivery.hitTargets ?? "Enemies"},
                            piercing: ${Boolean(delivery.piercing)},
                            despawn_on_hit: ${describeOptionalBoolRon(delivery.despawnOnHit)},
                            max_hits: ${describeOptionalNumberRon(delivery.maxHits)},
                        ),
                    )`;
    case "Area":
      return `Area(area: (
                        shape: ${describeAreaShapeRon(delivery.shape ?? fallbackAreaShape())},
                        anchor: ${delivery.anchor},
                        hit_targets: ${delivery.hitTargets ?? "Enemies"},
                        include_caster: ${Boolean(delivery.includeCaster)},
                        tick_policy: ${delivery.tickPolicy ?? "OncePerArea"},
                        duration_ms: ${delivery.durationMs ?? 0},
                        tick_interval_ms: ${describeOptionalNumberRon(delivery.tickIntervalMs)},
                    ))`;
  }
}

function describeAreaShapeRon(shape: AreaShapeDraft): string {
  switch (shape.type) {
    case "Circle":
      return `Circle(radius_units: ${shape.radiusUnits})`;
    case "Line":
      return `Line(length_units: ${shape.lengthUnits})`;
    case "Box":
      return `Box(width_units: ${shape.widthUnits}, height_units: ${shape.heightUnits})`;
    case "Rectangle":
      return `Rectangle(width_units: ${shape.widthUnits}, length_units: ${shape.lengthUnits})`;
    case "Cone":
      return `Cone(angle_degrees: ${shape.angleDegrees}, length_units: ${shape.lengthUnits})`;
  }
}

function fallbackAreaShape(): AreaShapeDraft {
  return { type: "Circle", radiusUnits: 2_000_000 };
}

function describeEffectRon(effect: EffectDraft): string {
  switch (effect.type) {
    case "Damage":
      return `Damage(amount: ${effect.amount}, damage_type: ${effect.damageType})`;
    case "ModifyDamage":
      return `ModifyDamage(modifiers:(${describeDamageModifiersRon(effect.modifiers)}))`;
    case "Heal":
      return `Heal(amount: ${effect.amount})`;
    case "ModifyResonance":
      return `ModifyResonance(amount: ${effect.amount})`;
    case "ModifyStats":
      return `ModifyStats(modifier: StatModifier(stat: ${effect.modifier.stat}, kind: ${effect.modifier.kind}, value: ${effect.modifier.value}))`;
    case "ApplyBuff":
      return `ApplyBuff(buff_id: "${effect.buffId}", duration_ms: ${effect.durationMs})`;
    case "ExtraAttack":
      return `ExtraAttack(count: ${effect.count})`;
  }
}

function describeDamageModifiersRon(modifiers: DamageModifiersDraft): string {
  const keyMap: Record<keyof DamageModifiersDraft, string> = {
    armorPenetrationFlat: "armor_penetration_flat",
    magicResistPenetrationFlat: "magic_resist_penetration_flat",
    armorPenetrationPercent: "armor_penetration_percent",
    magicResistPenetrationPercent: "magic_resist_penetration_percent",
    damageAmpPercent: "damage_amp_percent",
    damageReductionPercent: "damage_reduction_percent",
    physicalDamageAmpPercent: "physical_damage_amp_percent",
    magicDamageAmpPercent: "magic_damage_amp_percent",
    trueDamageAmpPercent: "true_damage_amp_percent",
    physicalDamageReductionPercent: "physical_damage_reduction_percent",
    magicDamageReductionPercent: "magic_damage_reduction_percent",
    trueDamageReductionPercent: "true_damage_reduction_percent",
    critChancePercent: "crit_chance_percent",
    critDamagePercent: "crit_damage_percent",
  };

  return Object.entries(modifiers)
    .filter(([, value]) => value !== undefined && value !== 0)
    .map(([key, value]) => `${keyMap[key as keyof DamageModifiersDraft]}: ${value}`)
    .join(", ");
}

function describeConditionRon(condition: StepConditionDraft): string {
  switch (condition.type) {
    case "Always":
      return "Always";
    case "IfPreviousStepDealtDamage":
      return "IfPreviousStepDealtDamage";
    case "IfCasterHasBuff":
      return `IfCasterHasBuff(buff_id: "${condition.buffId}", min_stacks: ${condition.minStacks})`;
  }
}

function describeRepeatRon(repeat: StepRepeatDraft): string {
  switch (repeat.type) {
    case "None":
      return "Once";
    case "Times":
      return `Times(count: ${repeat.count})`;
    case "ByBuffStacks":
      return `ByBuffStacks(unit: ${repeat.unit ?? "StepTarget"}, buff_id: "${repeat.buffId}", max: ${describeOptionalNumberRon(repeat.max)})`;
  }
}

function describeOptionalBoolRon(value?: boolean | null): string {
  return value === undefined || value === null ? "None" : `Some(${value})`;
}

function describeOptionalNumberRon(value?: number | null): string {
  return value === undefined || value === null ? "None" : `Some(${value})`;
}
