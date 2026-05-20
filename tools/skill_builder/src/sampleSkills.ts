import type { SkillDraft } from "./skillDraft";

export const sampleSkills: SkillDraft[] = [
  {
    id: "scorched_explosion",
    name: "Scorched Explosion",
    kind: "Targeted",
    focusTimeMs: 300,
    focusPermissions: {
      allowsMove: false,
      allowsBasicAttack: false,
    },
    steps: [
      {
        id: "explode",
        delayMs: 0,
        rangeTiles: 2,
        condition: { type: "Always" },
        repeat: { type: "None" },
        target: { type: "EnemySingle", rule: "Nearest" },
        delivery: { type: "Instant" },
        effects: [{ type: "Damage", amount: 50, damageType: "Magic" }],
        presentation: {
          impactVfxId: "scorched_explosion_hit",
          targetAnchor: "Center",
        },
      },
    ],
  },
  {
    id: "white_night_pale_benediction",
    name: "Pale Benediction",
    kind: "Untargeted",
    focusTimeMs: 600,
    focusPermissions: {
      allowsMove: false,
      allowsBasicAttack: false,
    },
    steps: [
      {
        id: "ally_salvation",
        delayMs: 0,
        rangeTiles: 3,
        condition: { type: "Always" },
        repeat: { type: "None" },
        target: { type: "Allies", area: { type: "RadiusChebyshev", radiusTiles: 3 } },
        delivery: {
          type: "Area",
          anchor: "CastTarget",
          includeCaster: true,
          shape: { type: "Box", widthUnits: 6_000_000, heightUnits: 6_000_000 },
        },
        effects: [{ type: "Heal", amount: 35 }],
        presentation: {
          impactVfxId: "white_night_pale_benediction_salvation",
        },
      },
      {
        id: "ally_blessing",
        delayMs: 180,
        rangeTiles: 3,
        condition: { type: "Always" },
        repeat: { type: "None" },
        target: { type: "Allies", area: { type: "RadiusChebyshev", radiusTiles: 3 } },
        delivery: { type: "Instant" },
        effects: [{ type: "ApplyBuff", buffId: "blessing", durationMs: 5000 }],
        presentation: {
          castState: "Cast",
          impactVfxId: "white_night_blessing",
        },
      },
      {
        id: "enemy_judgement",
        delayMs: 420,
        rangeTiles: 3,
        condition: { type: "IfPreviousStepDealtDamage" },
        repeat: { type: "None" },
        target: { type: "Enemies", area: { type: "RadiusChebyshev", radiusTiles: 3 } },
        delivery: {
          type: "Area",
          anchor: "CastTarget",
          includeCaster: false,
          shape: { type: "Box", widthUnits: 6_000_000, heightUnits: 6_000_000 },
        },
        effects: [{ type: "Damage", amount: 45, damageType: "Magic" }],
        presentation: {
          targetAnchor: "Head",
          impactVfxId: "white_night_pale_judgement",
        },
      },
    ],
  },
  {
    id: "fragment_universe_nova",
    name: "Fragment Nova",
    kind: "Untargeted",
    focusTimeMs: 600,
    focusPermissions: {
      allowsMove: false,
      allowsBasicAttack: false,
    },
    steps: [
      {
        id: "nova",
        delayMs: 0,
        rangeTiles: 2,
        condition: { type: "Always" },
        repeat: { type: "None" },
        target: { type: "Enemies", area: { type: "RadiusChebyshev", radiusTiles: 2 } },
        delivery: {
          type: "Area",
          anchor: "CastTarget",
          includeCaster: false,
          shape: { type: "Box", widthUnits: 4_000_000, heightUnits: 4_000_000 },
        },
        effects: [{ type: "Damage", amount: 35, damageType: "Magic" }],
        presentation: {
          impactVfxId: "fragment_universe_nova",
        },
      },
    ],
  },
];
