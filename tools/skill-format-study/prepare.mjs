import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';
import { formats, serialize } from './adapters.mjs';

export const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
export const inputs = resolve(root, 'tools/skill-format-study/inputs');
export function baseModule() {
  const module = JSON.parse(readFileSync(resolve(root, 'core_enhanced/tests/fixtures/skill_composition/woodsman_monk.json'), 'utf8'));
  // Deliberate broken reference, identical in all candidate inputs.
  module.skills[1].body.Call.arguments[0].value.Shared = 'burst-powre';
  return module;
}
export function representation() {
  return {
    schema: 'representation-study-only/v1', executable: false,
    note: '작성 표현 보존 시험. 이 문서는 실행 지원을 주장하지 않는다. 모든 변경값은 비교용 fixture이다.',
    values: {
      'PC.v.duration': { type: 'Milliseconds', Unresolved: '원문 지속시간 미정' },
      'PC.v.coefficient': { type: 'Scalar', Unresolved: '원문 보호량 계수 미정' },
      'PC.v.ringComposition': { Unresolved: 'PC.U5 생성량 합성 순서 미정' },
      'QH.v.heartCap': { type: 'Count', fixture: 6 },
      'QH.v.gainPeriod': { type: 'Milliseconds', fixture: 15000 },
    },
    states: [
      { id: 'PC.shield', owner: ['sourceEmployee', 'sourceDeployment', 'sourceCopy', 'targetDeployment'], fields: ['maxAmount', 'remaining', 'expiresAt', 'ended'], initial: { maxAmount: { Ref: 'PC.amount.final' }, remaining: { Ref: 'PC.amount.final' }, expiresAt: { Ref: 'PC.v.duration' }, ended: false }, ends: ['depleted', 'expired', 'sourceWithdrawal', 'targetWithdrawal', 'sourceDeath', 'targetDeath', 'battleEnd'], survives: ['movement', 'outOfRange', 'hardCC', 'silence'] },
      { id: 'HG.active', owner: ['employee', 'deployment', 'sourceCopy', 'activation'], fields: ['startedAt', 'expiresAt', 'contributionHandle', 'ended'], ends: ['expired', 'manualConsumption', 'withdrawal', 'actualDeath', 'battleEnd'], cleanup: ['ownContribution', 'reentryRight', 'resonanceGainLock'], releasedAttacks: 'keepReleaseSnapshot', resetAttackClock: false },
      { id: 'QH.owner', owner: ['sourceCopy', 'employeeDeployment'], initial: { hearts: 0, gainElapsed: 0, rememberedTile: null, mode: 'normal', pendingAuto: false, currentCast: null }, cap: { Ref: 'QH.v.heartCap' }, clock: { period: { Ref: 'QH.v.gainPeriod' }, runsDuring: ['regularCast', 'hardCC', 'silence'], stopsDuring: ['pendingAuto', 'hysteric', 'pause', 'undeployed'] }, resetOn: ['newBattle', 'redeployment'], preserveOn: ['tacticalMove'] },
    ],
    events: [
      { id: 'QH.reserve', when: 'hearts reaches shared cap', set: { pendingAuto: true }, waitFor: ['currentActionFinishedOrCleaned', 'actionAdmission'], forbidden: ['duplicateAuto', 'manualReserveBypass'] },
      { id: 'QH.auto.end', on: ['naturalCompletion', 'actualForcedCancel'], set: { hearts: 0, gainElapsed: 0, mode: 'normal' }, restartExcept: ['death', 'withdrawal', 'battleEnd'] },
    ],
    boundaries: [
      { id: 'A1.input', owner: ['battle', 'sourceCopy', 'inputSession'], tentative: ['layout', 'aim', 'previousSpeed', 'pauseOwnership'], commitTogether: ['resonance', 'allSpears', 'rememberedLayout', 'releaseIntents', 'releaseOwnPause'], requestKey: 'idempotent', disconnect: 'retainFrozenSession', cancel: 'discardTentativeAndRestorePreviousSpeed' },
      { id: 'A2.transfer', snapshot: ['bothMemberships', 'positions'], order: ['allBaseHealing', 'directionBudgets', 'bothDeficitSnapshots', 'allocate', 'transfer'], excess: 'maxHPOverflowOnly', retransmit: false, amplifyAgain: false, carry: false, rate: { Unresolved: '원문 전달률 미정' } },
      { id: 'A3.promotion', owner: 'selectedWeaponCopy', transaction: ['cost', 'unlock', 'permanentProfile', 'receipt'], duplicate: 'existingReceipt', futureCopies: 'unchanged', markStore: ['weaponCopy', 'battle'] },
      { id: 'A4.spread', trigger: 'confirmedHostDeath', dedupe: ['deathId', 'sourceCopy'], prevention: 'noConfirmedDeathNoSpread', newHosts: 'noRetransmitInSameTransaction', radius: { Unresolved: '원문 전파반경 미정' } },
      { id: 'A5.debt', owner: ['expedition', 'sourceCopy'], settle: 'oncePerTerminalBattleId', debt: 'max(0,-R)', borrowed: 'retainEvenAfterRepayment', retreat: 'restoreEntrySnapshot', positiveCarry: false },
    ],
  };
}

export function expectedModule() {
  const result = baseModule();
  result.values[1].value.Amount.root.Ratio.numerator.Mul.scalar.Const.value = 5;
  result.skills[1].prepare_ms = 150;
  result.skills[1].body.Call.arguments[0].value.Shared = 'burst-power';
  const sequence = result.skills[0].body;
  [sequence.Sequence.children[0], sequence.Sequence.children[1]] = [sequence.Sequence.children[1], sequence.Sequence.children[0]];
  sequence.Sequence.children[2].Call.arguments[2].value.Shared = 'stun';
  result.skills[0].body = { Repeat: { id: 'double-combo', count: { Literal: { Count: 2 } }, interval: { Literal: { Milliseconds: 50 } }, body: sequence } };
  return result;
}
export function expectedRepresentation() {
  const result = representation();
  result.values['QH.v.heartCap'].fixture = 7;
  result.values['QH.v.gainPeriod'].fixture = 12000;
  return result;
}

export function prepare() {
  mkdirSync(inputs, { recursive: true });
  for (const format of formats) {
    const comment = format === 'yaml' ? '#' : '//';
    const decorate = (value, marker) => {
      const lines = serialize(format, value).split('\n');
      lines.splice(Math.min(8, lines.length - 1), 0, `${comment} KEEP:${marker}:interior 원본 주석 보존`);
      return `${comment} KEEP:${marker}:header 비교 fixture, 출시값 아님\n` + lines.join('\n');
    };
    writeFileSync(resolve(inputs, `module.${format}`), decorate(baseModule(), 'module'));
    writeFileSync(resolve(inputs, `representation.${format}`), decorate(representation(), 'representation'));
    const syntax = {
      jsonc: 'JSON with // comments and optional trailing commas. Objects/arrays follow normal JSON.',
      yaml: 'YAML 1.2 restricted to JSON data: string keys, explicit null, no tags/anchors/aliases/merge. # comments.',
      kdl: 'KDL 2 JSON-in-KDL: root -, objects as named child nodes; arrays as children named -; scalar as one argument. Empty array (array)name; empty object (object)name. Strings quoted when needed; booleans/null #true/#false/#null. // comments. Keep the same logical JSON mapping.',
    }[format];
    writeFileSync(resolve(inputs, `task.${format}.txt`), `You are editing two ${format} documents. Syntax: ${syntax}\n\nOnly edit your assigned output copies of module.${format} and representation.${format}. Preserve all other values, IDs, references, ordering except the specified swap, and both KEEP comments in each file. Do not read prepare.mjs, study.test.mjs, other attempts, product source, or grader implementation. You may read the two input files and this guide, use local shell/text editing tools, and invoke the provided submit command. Do not browse. The representation document is NOT executable; do not convert it into a Core module. No authoring time limit.\n\nChanges to module:\n1. In shared value burst-power, change numerator Mul.scalar.Const.value from 3 to 5. Keep denominator=2 and Ceil.\n2. In monk-authored, prepare_ms=150. Fix its power argument burst-powre to the existing burst-power.\n3. In woodsman-authored three-strikes, swap gather and stagger (stagger,gather,split). Keep IDs and each call's arguments together. For split alone change after from Shared none to Shared stun.\n4. Wrap that entire Sequence in Repeat with id=double-combo, count={Literal:{Count:2}}, interval={Literal:{Milliseconds:50}}, body=the unchanged edited Sequence. This is repeated execution, not copying children.\n\nChanges to representation:\n5. Change QH.v.heartCap.fixture from 6 to 7 and QH.v.gainPeriod.fixture from 15000 to 12000. Keep all Ref objects; do not replace with inlined numbers. Keep PC, HG, QH ownership, events, all Unresolved values and A1-A5 boundaries exactly as supplied. These are test changes only.\n\nSubmit by node /mnt/f/work/simulator/tools/skill-format-study/submit.mjs <your-attempt-directory>. Maximum 3 submissions total: initial plus up to 2 corrections using its diagnostics. Stop after PASS or third submission and report. The submit tool saves each submitted version and validation evidence. No timeout is imposed on your editing.\n`);
  }
}
if (process.argv[1] === fileURLToPath(import.meta.url)) prepare();
