import { useRef, useState } from "react";
import { Button } from "../components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "../components/ui/dialog";
import type { Definition } from "./session";
import { CompositionRunSource } from "./CompositionRunSource";
import {
  parseTrialConfig,
  runDefinition,
  type RunResponse,
  type TrialConfig,
} from "./execution";

const sustainedEndLabels = {
  Expired: "지속 시간 만료",
  Stopped: "중지",
  Interrupted: "중단",
  HardCrowdControl: "행동 제어",
  SourceUnavailable: "시전자 퇴장",
  TargetUnavailable: "대상 퇴장",
  Terminal: "전투 종료",
  AttackReleased: "공격 방출",
  Depleted: "소진",
  Triggered: "발동",
  Reinput: "재입력",
  Replaced: "교체",
} as const;

function failureReason(value: unknown, depth = 0): string {
  if (depth >= 16) return "상세 진단을 확인하세요.";
  if (typeof value === "string") return value;
  if (value && typeof value === "object") {
    const record = value as Record<string, unknown>;
    for (const key of [
      "cause",
      "reason",
      "error",
      "diagnostic",
      "details",
      "fault",
      "code",
    ])
      if (record[key] !== undefined)
        return failureReason(record[key], depth + 1);
  }
  return "상세 진단을 확인하세요.";
}

export function CompositionTrial({
  definition,
  currentSource,
}: {
  definition: Definition | null;
  currentSource: string;
}) {
  const [open, setOpen] = useState(false);
  const [config, setConfig] = useState<TrialConfig | null>(null);
  const [fileName, setFileName] = useState("");
  const [pending, setPending] = useState<Definition | null>(null);
  const [completed, setCompleted] = useState<{
    definition: Definition;
    result: RunResponse;
  } | null>(null);
  const [error, setError] = useState("");
  const [failed, setFailed] = useState<RunResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const button = useRef<HTMLButtonElement>(null);
  const serial = useRef(0);
  const busy = useRef(false);
  const fileSerial = useRef(0);
  async function load(file: File) {
    const version = ++fileSerial.current;
    setLoading(true);
    try {
      if (file.size > 8 * 1024 * 1024)
        throw new Error("실행 설정 파일은 8MB 이하여야 합니다.");
      const next = parseTrialConfig(await file.text());
      if (fileSerial.current !== version) return;
      setConfig(next);
      setFileName(file.name);
      setError("");
    } catch (cause) {
      if (fileSerial.current === version) setError(String(cause));
    } finally {
      if (fileSerial.current === version) setLoading(false);
    }
  }
  async function run() {
    if (!definition || !config || busy.current || loading) return;
    const pinned = definition;
    busy.current = true;
    setPending(pinned);
    setError("");
    setFailed(null);
    try {
      const result = await runDefinition(pinned, config, ++serial.current);
      if (result.status === "completed")
        setCompleted({ definition: pinned, result });
      else setFailed(result);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      busy.current = false;
      setPending(null);
    }
  }
  const result = completed?.result;
  const sustained = result?.latest_sustained;
  const hitCounter = result?.latest_hit_counter;
  const area = result?.latest_area;
  const readyWait = result?.latest_ready_wait;
  const nearbyDefense = result?.latest_nearby_defense;
  const areaDamage = result?.latest_area_damage;
  const nearbyAllyDamage = result?.latest_nearby_ally_damage;
  const conditionalDamage = result?.latest_conditional_damage;
  const conditionalReduction = result?.latest_conditional_reduction;
  const resonanceGain = result?.latest_resonance_gain;
  const healReaction = result?.latest_actual_heal_reaction;
  const primaryDamage = result?.latest_primary_damage;
  const speed = result?.latest_equipment_speed;
  const support = result?.latest_equipment_support;
  const deferred = result?.latest_deferred_damage;
  const magazine = result?.latest_magazine;
  const weaponStage = result?.latest_weapon_stage;
  const cohort = result?.latest_shared_cohort;
  const bounty = result?.latest_bounty;
  const equipmentForm = result?.latest_equipment_form;
  const lastWill = result?.latest_last_will;
  const transformation = result?.latest_employee_transformation;
  const deathBurst = result?.latest_equipment_death_burst;
  const reserve = result?.latest_overheal_reserve;
  const echo = result?.latest_basic_heal_echo;
  const healBarrier = result?.latest_heal_barrier;
  const adaptive = result?.latest_adaptive_shield;
  const opening = result?.latest_channel_opening;
  const flat = result?.latest_incoming_flat;
  const wearerReduction = result?.latest_wearer_reduction;
  const shieldBeneficiary = result?.latest_shield_beneficiary_damage;
  const receivedHeal = result?.latest_received_heal;
  const controlDuration = result?.latest_control_duration;
  const barrier = result?.latest_skill_barrier;
  const evasion = result?.latest_basic_evasion;
  return (
    <>
      <Button
        ref={button}
        size="sm"
        variant="outline"
        onClick={() => setOpen(true)}
      >
        실행 시험
      </Button>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent
          className="composition-trial"
          onCloseAutoFocus={(event) => {
            event.preventDefault();
            button.current?.focus();
          }}
        >
          <DialogHeader>
            <DialogTitle>조합 실행 시험</DialogTitle>
            <DialogDescription>
              작성한 스킬과 자원 정의를 반영한 별도 시험 콘텐츠로 시나리오를
              실행합니다. 같은 ID가 있으면 현재 작성한 정의를 사용합니다.
            </DialogDescription>
          </DialogHeader>
          <label>
            실행 설정 파일
            <input
              type="file"
              accept=".json"
              aria-label="실행 설정 파일"
              onChange={(event) => {
                const file = event.target.files?.[0];
                if (file) void load(file);
              }}
            />
          </label>
          {config && (
            <>
              <p>{fileName}</p>
              <label>
                콘텐츠 폴더
                <input
                  aria-label="실행 콘텐츠 폴더"
                  value={config.root}
                  disabled={loading}
                  onChange={(event) =>
                    setConfig({ ...config, root: event.target.value })
                  }
                />
              </label>
            </>
          )}
          <p>
            {definition
              ? `실행할 원본 ${definition.ticket.epoch}:${definition.ticket.revision}`
              : "현재 원본을 컴파일해야 새 실행을 시작할 수 있습니다."}
          </p>
          <Button
            variant="outline"
            disabled={!definition || !config || !!pending || loading}
            onClick={() => void run()}
          >
            시나리오 실행
          </Button>
          {pending && (
            <p role="status">
              원본 {pending.ticket.epoch}:{pending.ticket.revision} 실행 중…
              편집·재컴파일은 이 실행을 바꾸지 않습니다.
            </p>
          )}
          {error && <p role="alert">{error}</p>}
          {loading && <p role="status">실행 설정을 읽는 중…</p>}
          {failed && (
            <div role="alert">
              <p>
                {failed.status === "rejected" ? "실행 준비 거절" : "실행 실패"}{" "}
                · 원본 {failed.epoch}:{failed.revision} · 처리 단계{" "}
                {failed.accepted_steps}
              </p>
              <p>{failureReason(failed.failure?.diagnostic)}</p>
              <details>
                <summary>실패 상세</summary>
                <pre>{JSON.stringify(failed, null, 2)}</pre>
              </details>
            </div>
          )}
          {result && completed && (
            <div
              className="composition-trial-result"
              aria-label="실행 결과"
              role="group"
            >
              <p role="status">
                {failed || pending
                  ? "이전 완료 실행"
                  : result.status === "completed"
                    ? "시나리오 실행 완료"
                    : result.status === "rejected"
                      ? "실행 준비 거절"
                      : "실행 실패"}{" "}
                · 원본 {completed.definition.ticket.epoch}:
                {completed.definition.ticket.revision} · 처리 단계{" "}
                {result.accepted_steps}
              </p>
              {result.identity && <p>실행 {result.identity.run_id}</p>}
              {evasion && (
                <div role="group" aria-label="기본 공격 회피 관측">
                  <p>
                    기본 공격 회피 · {evasion.evaded ? "성공" : "실패"} ·{" "}
                    {evasion.time_ms}ms
                  </p>
                  <p>
                    회피율 {evasion.chance_percent}% · 판정값{" "}
                    {evasion.roll_percent} (0~99)
                  </p>
                  <p>
                    {evasion.definition_id} · 대상 {evasion.target}
                  </p>
                  <p>
                    사본{" "}
                    {evasion.equipment.persistent_copy_uuid ??
                      evasion.equipment.runtime_item_instance_id}
                  </p>
                  <p>
                    공격자 {evasion.incoming.actor} · 타격 {evasion.strike}
                  </p>
                  <p>가장 최근 타격의 판정입니다.</p>
                  {evasion.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:evasion`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={evasion.origin}
                    />
                  ) : (
                    <p>이 장비 효과는 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {adaptive && (
                <div role="group" aria-label="적응형 보호막 관측">
                  <p>
                    적응형 보호막 ·{" "}
                    {adaptive.damage_type === "Physical" ? "물리" : "마법"} ·{" "}
                    {adaptive.capacity} · {adaptive.time_ms}ms
                  </p>
                  <p>
                    발동 당시 최대 HP {adaptive.max_hp} · 만료{" "}
                    {adaptive.expires_at_ms}ms · 재발동 가능{" "}
                    {adaptive.ready_at_ms}ms
                  </p>
                  <p>
                    {adaptive.definition_id} · 착용자 {adaptive.actor}
                  </p>
                  <p>
                    주 대상 {adaptive.primary} · 공격{" "}
                    {adaptive.attack_source.action_ordinal}
                  </p>
                  <p>
                    사본{" "}
                    {adaptive.equipment.persistent_copy_uuid ??
                      adaptive.equipment.runtime_item_instance_id}
                  </p>
                  <p>
                    가장 최근 발동 시점의 기록입니다. 생성량은 이후 피해로
                    줄어든 현재 잔량과 다를 수 있습니다.
                  </p>
                  {adaptive.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:adaptive-shield`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={adaptive.origin}
                    />
                  ) : (
                    <p>이 장비 효과는 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {shieldBeneficiary && (
                <div role="group" aria-label="보호막 수혜 피해 증가 관측">
                  <p>
                    보호막 수혜 피해 증가 · {shieldBeneficiary.time_ms}ms · +
                    {shieldBeneficiary.percent}%
                  </p>
                  <p>
                    제공자 {shieldBeneficiary.provider} · 수혜자{" "}
                    {shieldBeneficiary.beneficiary}
                  </p>
                  <p>
                    보호막 #{shieldBeneficiary.shield_activation} · 판정 당시
                    잔량 {shieldBeneficiary.shield_remaining}
                  </p>
                  <p>
                    마지막 피해 계산의 기록입니다. 현재 보호막 상태와 실제 HP
                    피해는 전투 결과에서 확인합니다.
                  </p>
                  <p>
                    {shieldBeneficiary.definition_id} · 공격자{" "}
                    {shieldBeneficiary.actor} · 대상 {shieldBeneficiary.target}
                  </p>
                  <p>
                    장비 사본{" "}
                    {shieldBeneficiary.equipment.persistent_copy_uuid ??
                      shieldBeneficiary.equipment.runtime_item_instance_id}
                  </p>
                  {shieldBeneficiary.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:shield-beneficiary`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={shieldBeneficiary.origin}
                    />
                  ) : (
                    <p>이 장비 효과는 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {receivedHeal && (
                <div role="group" aria-label="받는 회복 관측">
                  <p>
                    받는 회복 · {receivedHeal.time_ms}ms · 저지{" "}
                    {receivedHeal.blocked_hostiles}명 · 증가{" "}
                    {receivedHeal.percent}%
                  </p>
                  <p>
                    입력 {receivedHeal.before} → HP 상한 적용 전{" "}
                    {receivedHeal.after} · 다른 증가{" "}
                    {receivedHeal.other_percent}%
                  </p>
                  <p>실제 HP 회복량은 전투 결과에서 확인합니다.</p>
                  <p>
                    {receivedHeal.definition_id} · 대상 {receivedHeal.target}
                  </p>
                  <p>
                    장비 사본{" "}
                    {receivedHeal.equipment.persistent_copy_uuid ??
                      receivedHeal.equipment.runtime_item_instance_id}
                  </p>
                  {receivedHeal.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:received-heal`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={receivedHeal.origin}
                    />
                  ) : (
                    <p>이 장비 효과는 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {controlDuration && (
                <div role="group" aria-label="제어 시간 감소 관측">
                  <p>
                    제어 시간 감소 · {controlDuration.time_ms}ms ·{" "}
                    {controlDuration.kind}
                  </p>
                  <p>
                    {controlDuration.original_ms}ms →{" "}
                    {controlDuration.effective_ms}ms · 감소{" "}
                    {controlDuration.reduction_percent}%
                  </p>
                  <p>
                    {controlDuration.definition_id} · 대상{" "}
                    {controlDuration.target}
                  </p>
                  <p>
                    장비 사본{" "}
                    {controlDuration.equipment.persistent_copy_uuid ??
                      controlDuration.equipment.runtime_item_instance_id}
                  </p>
                  {controlDuration.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:control-duration`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={controlDuration.origin}
                    />
                  ) : (
                    <p>이 장비 효과는 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {wearerReduction && (
                <div role="group" aria-label="현재 HP 기반 감면 관측">
                  <p>
                    현재 HP 기반 감면 · {wearerReduction.time_ms}ms · 피해 감소{" "}
                    {wearerReduction.percent}%
                  </p>
                  <p>
                    HP {wearerReduction.current_hp} / {wearerReduction.max_hp} ·
                    타격 {wearerReduction.strike}
                  </p>
                  <p>
                    {wearerReduction.definition_id} · 대상{" "}
                    {wearerReduction.target}
                  </p>
                  <p>
                    장비 사본{" "}
                    {wearerReduction.equipment.persistent_copy_uuid ??
                      wearerReduction.equipment.runtime_item_instance_id}
                  </p>
                  <p>
                    해당 타격 시작의 계산값입니다. 실제 HP 변화는 전투 결과에서
                    확인합니다.
                  </p>
                  {wearerReduction.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:wearer-reduction`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={wearerReduction.origin}
                    />
                  ) : (
                    <p>이 장비 효과는 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {flat && (
                <div role="group" aria-label="물리 기본 타격 차감 관측">
                  <p>
                    물리 기본 타격 차감 · {flat.time_ms}ms · 차감 {flat.used}
                  </p>
                  <p>
                    방어력 {flat.defense} · 타격 한도 {flat.allowance} · 남은
                    한도 {flat.remaining}
                  </p>
                  <p>
                    {flat.definition_id} · 대상 {flat.target}
                  </p>
                  <p>
                    장비 사본{" "}
                    {flat.equipment.persistent_copy_uuid ??
                      flat.equipment.runtime_item_instance_id}
                  </p>
                  <p>
                    방어 계산 후, 비율 보정 전 차감한 양입니다. 실제 HP 변화는
                    전투 결과에서 확인합니다.
                  </p>
                  {flat.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:incoming-flat`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={flat.origin}
                    />
                  ) : (
                    <p>이 장비 효과는 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {opening && (
                <div role="group" aria-label="집중 초반 보호 관측">
                  <p>
                    집중 초반 보호 · {opening.time_ms}ms · 피해 감소{" "}
                    {opening.reduction_percent}%
                  </p>
                  <p>
                    집중 시작 {opening.started_at_ms}ms · 보호 한계{" "}
                    {opening.window_end_ms}ms
                  </p>
                  <p>
                    {opening.definition_id} · 대상 {opening.target}
                  </p>
                  <p>
                    장비 사본{" "}
                    {opening.equipment.persistent_copy_uuid ??
                      opening.equipment.runtime_item_instance_id}
                  </p>
                  <p>
                    최근 적 피해 계산의 기록입니다. 집중이 끝나면 보호도
                    끝납니다. 실제 HP 변화는 전투 결과에서 확인합니다.
                  </p>
                  {opening.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:opening`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={opening.origin}
                    />
                  ) : (
                    <p>이 장비 효과는 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {barrier && (
                <div role="group" aria-label="스킬 차단 관측">
                  <p>
                    스킬 차단 · {barrier.time_ms}ms · 재충전 가능{" "}
                    {barrier.ready_at_ms}ms
                  </p>
                  <p>
                    {barrier.definition_id} · 대상 {barrier.target}
                  </p>
                  <p>
                    사본{" "}
                    {barrier.equipment.persistent_copy_uuid ??
                      barrier.equipment.runtime_item_instance_id}
                  </p>
                  <p>
                    차단한 스킬 {barrier.incoming.cast.skill_id} · 시전{" "}
                    {barrier.incoming.cast.cast_seq}
                  </p>
                  <p>
                    최근 차단 시점의 기록입니다. 대기 중 피격은 재충전 시각을
                    바꾸지 않습니다.
                  </p>
                  {barrier.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:barrier`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={barrier.origin}
                    />
                  ) : (
                    <p>이 장비 효과는 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {cohort && (
                <div role="group" aria-label="공유 사냥감 관측">
                  <p>공유 사냥감 · {cohort.time_ms}ms 출현</p>
                  <p>{cohort.definition}</p>
                  <p>
                    출현 시 HP {cohort.initial_hp} · 제공 사본{" "}
                    {cohort.provider_count}개 · 공유 주기 1개
                  </p>
                  <p>다음 출현 예정 {cohort.next_spawn_ms}ms</p>
                  <p>직원 {cohort.employee}</p>
                  <p>실제 적 {cohort.unit}</p>
                  <p>
                    파편 보상 수신 사본: {cohort.fragment_recipient ?? "없음"}
                  </p>
                  {cohort.equipment_copy && (
                    <p>장비 사본 {cohort.equipment_copy}</p>
                  )}
                  <p>
                    가장 최근 출현의 기록입니다. 전투 종료 시 생성 주기는
                    종료되며, 표시한 HP는 현재 생존 상태를 뜻하지 않습니다.
                  </p>
                  {cohort.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:shared-cohort`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={cohort.origin}
                    />
                  ) : (
                    <p>
                      이 사냥감 제공자는 현재 조합 원본과 연결되지 않았습니다.
                    </p>
                  )}
                </div>
              )}
              {weaponStage && (
                <div role="group" aria-label="무기 단계 관측">
                  <p>
                    무기 단계 ·{" "}
                    {weaponStage.stage === "Base" ? "기본형" : "승급형"} ·{" "}
                    {weaponStage.time_ms}ms
                  </p>
                  <p>
                    설정 사격 간격 {weaponStage.interval_ms}ms · 준비{" "}
                    {weaponStage.windup_ms}ms
                  </p>
                  <p>
                    실제 HP 피해 {weaponStage.actual_hp_loss} · 밀치기 최대{" "}
                    {weaponStage.push_distance_units / 1_000_000}타일
                  </p>
                  <p>
                    대상 위치 (
                    {weaponStage.from_data_units
                      .map((n) => n / 1_000_000)
                      .join(", ")}
                    ) → (
                    {weaponStage.to_data_units
                      .map((n) => n / 1_000_000)
                      .join(", ")}
                    )
                  </p>
                  <p>
                    장비 사본{" "}
                    {weaponStage.equipment.persistent_copy_uuid ??
                      weaponStage.equipment.runtime_item_instance_id}
                  </p>
                  <p>
                    가장 최근 유효 적중의 기록입니다. 피해와 실제 이동 결과는
                    각각 표시합니다.
                  </p>
                  {weaponStage.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:weapon-stage`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={weaponStage.origin}
                    />
                  ) : (
                    <p>이 사격은 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {transformation && (
                <div role="group" aria-label="직원 변신 관측">
                  <p>직원 영구 사망 · 적대 변신 · {transformation.time_ms}ms</p>
                  <p>
                    변신 위치 ({transformation.position.x},{" "}
                    {transformation.position.y}) · 형태 {transformation.form}
                  </p>
                  <p>직원 {transformation.employee_uuid}</p>
                  <p>
                    새 적 {transformation.successor} · 승리 필수 · 조우
                    보상/도감 제외
                  </p>
                  <p>
                    장비 사본 {transformation.equipment.persistent_copy_uuid}
                  </p>
                  {transformation.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:employee-transformation`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={transformation.origin}
                    />
                  ) : (
                    <p>이 장비 효과는 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {lastWill && (
                <div role="group" aria-label="마지막 유언 관측">
                  <p>마지막 유언 · {lastWill.time_ms}ms · 전투불능</p>
                  <p>
                    HP {lastWill.hp_before} → {lastWill.hp_protected} 보호 →{" "}
                    {lastWill.hp_after} 퇴장
                  </p>
                  <p>
                    발동 위치 ({lastWill.center.x}, {lastWill.center.y}) · 방향{" "}
                    {
                      { up: "위", right: "오른쪽", down: "아래", left: "왼쪽" }[
                        lastWill.facing
                      ]
                    }
                  </p>
                  <p>
                    공격력 {lastWill.attack} · 기초 피해 {lastWill.base_damage}{" "}
                    · 폭발 대상 {lastWill.targets}명
                  </p>
                  <p>
                    장비 사본 {lastWill.equipment.persistent_copy_uuid} · 이번
                    전투 사용 완료
                  </p>
                  {lastWill.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:last-will`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={lastWill.origin}
                    />
                  ) : (
                    <p>이 장비 효과는 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {deathBurst && (
                <div role="group" aria-label="처치 연쇄 폭발 관측">
                  <p>
                    처치 연쇄 폭발 · {deathBurst.time_ms}ms ·{" "}
                    {
                      {
                        Reserved: "예약",
                        Detonated: "폭발",
                        Cancelled: "취소",
                      }[deathBurst.operation]
                    }
                  </p>
                  <p>
                    사망 위치 ({deathBurst.center.x}, {deathBurst.center.y}) ·
                    발효 {deathBurst.due_ms}ms
                  </p>
                  <p>
                    예약 공격력 {deathBurst.attack} · 기초 피해{" "}
                    {deathBurst.base_damage}
                  </p>
                  <p>
                    이번 폭발 대상 {deathBurst.targets}명 · 남은 예약{" "}
                    {deathBurst.pending}개
                  </p>
                  <p>장비 사본 {deathBurst.equipment_copy}</p>
                  {deathBurst.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:death-burst`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={deathBurst.origin}
                    />
                  ) : (
                    <p>이 폭발은 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {equipmentForm && (
                <div role="group" aria-label="장비 형태 관측">
                  <p>
                    장비 형태 · {equipmentForm.form.name} ·{" "}
                    {equipmentForm.time_ms}ms
                  </p>
                  <p>{equipmentForm.form.description}</p>
                  <p>외형 참조 {equipmentForm.form.appearance}</p>
                  <p>
                    현재 단계 진행 {equipmentForm.form.progress} /{" "}
                    {equipmentForm.form.wins_required || "최종 형태"} · 조건
                    충족 승리 {equipmentForm.confirmed}회
                  </p>
                  <p>
                    이번 전투 직접 처치 조건{" "}
                    {equipmentForm.pending === 1
                      ? "충족 · 승리 대기"
                      : "미확정 기록 없음"}
                  </p>
                  <p>
                    현재 효과 ·{" "}
                    {"MaxHealth" in equipmentForm.form.effect
                      ? `최대 HP +${equipmentForm.form.effect.MaxHealth.amount}`
                      : "Attack" in equipmentForm.form.effect
                        ? `공격력 +${equipmentForm.form.effect.Attack.amount}`
                        : `받는 피해 +${equipmentForm.form.effect.IncomingDamage.percent}%`}
                  </p>
                  <p>장비 사본 {equipmentForm.equipment_copy}</p>
                  <p>
                    {
                      {
                        Earned: "직접 처치 기록",
                        Confirmed: "승리 정산 완료",
                        Discarded: "미확정 기록 폐기",
                        Equipped: "장착 형태 적용",
                      }[equipmentForm.operation]
                    }{" "}
                    · 효과는 형태마다 교체되며 전투 중 처치만으로 바뀌지
                    않습니다.
                  </p>
                  {equipmentForm.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:equipment-form`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={equipmentForm.origin}
                    />
                  ) : (
                    <p>이 장비 형태는 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {bounty && (
                <div role="group" aria-label="수금과 공격력 성장 관측">
                  <p>
                    수금 ·{" "}
                    {
                      {
                        Earned: "직접 처치 적립",
                        Confirmed: "승리 확정",
                        Discarded: "미확정 수금 폐기",
                        Equipped: "장착 성장 적용",
                      }[bounty.operation]
                    }{" "}
                    · {bounty.time_ms}ms
                  </p>
                  <p>
                    이번 전투 미확정 {bounty.pending} · 원정 확정{" "}
                    {bounty.confirmed}
                  </p>
                  <p>
                    확정 단계 보너스 Attack +{bounty.tier_attack} · 다음 임계치{" "}
                    {bounty.next_threshold ?? "최대 단계"}
                  </p>
                  <p>
                    관측 시 실제 Attack{" "}
                    {bounty.actual_attack ?? "활성 착용자 없음"}
                  </p>
                  <p>장비 사본 {bounty.equipment_copy}</p>
                  <p>
                    가장 최근 수금·정산·장착 관측입니다. 미확정 수금은 이번
                    전투의 공격력을 바꾸지 않습니다.
                  </p>
                  {bounty.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:bounty`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={bounty.origin}
                    />
                  ) : (
                    <p>이 수금 효과는 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {magazine && (
                <div role="group" aria-label="탄창 관측">
                  <p>
                    탄창 · {magazine.definition_id} · {magazine.time_ms}ms
                  </p>
                  <p>
                    {magazine.operation === "Fired"
                      ? "발사"
                      : magazine.operation === "Reloaded"
                        ? "재장전 완료"
                        : "사냥 보상"}{" "}
                    · {magazine.enhanced ? "강화탄" : "일반탄"} · 잔탄{" "}
                    {magazine.remaining} / {magazine.capacity}
                  </p>
                  {magazine.reload_at_ms !== null && (
                    <p>재장전 완료 예정 {magazine.reload_at_ms}ms</p>
                  )}
                  <p>착용자 {magazine.actor}</p>
                  <p>
                    사본{" "}
                    {magazine.equipment.persistent_copy_uuid ??
                      magazine.equipment.runtime_item_instance_id}
                  </p>
                  <p>
                    가장 최근 발사·재장전·보상의 관측입니다. 이후 탄창 변화나
                    전체 사격 이력을 대신하지 않습니다.
                  </p>
                  {magazine.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:magazine`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={magazine.origin}
                    />
                  ) : (
                    <p>이 탄창은 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {deferred && (
                <div role="group" aria-label="지연 피해 관측">
                  <p>
                    지연 피해 · {deferred.definition_id} · {deferred.time_ms}ms
                  </p>
                  <p>
                    {deferred.operation === "Store"
                      ? "유예"
                      : deferred.operation === "Repay"
                        ? "상환"
                        : "처치 제거"}{" "}
                    {deferred.amount} · 남은 흉터 {deferred.remaining}
                  </p>
                  <p>
                    실제 HP 손실 {deferred.actual_hp_loss} · 착용자{" "}
                    {deferred.target}
                  </p>
                  <p>
                    사본{" "}
                    {deferred.equipment.persistent_copy_uuid ??
                      deferred.equipment.runtime_item_instance_id}
                  </p>
                  <p>
                    가장 최근 유예·상환·처치 제거의 관측입니다. 처치 제거는 HP를
                    회복하지 않습니다.
                  </p>
                  {deferred.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:deferred-damage`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={deferred.origin}
                    />
                  ) : (
                    <p>이 지연 피해는 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {reserve && (
                <div role="group" aria-label="초과 회복 저장 관측">
                  <p>
                    초과 회복 저장 · {reserve.definition_id} · {reserve.time_ms}
                    ms
                  </p>
                  <p>
                    {reserve.operation === "Store" ? "저장" : "방출"} · 저장량{" "}
                    {reserve.stored_before} → {reserve.stored_after} / 상한{" "}
                    {reserve.capacity}
                  </p>
                  <p>
                    받은 초과 회복 {reserve.incoming_overheal} · 실제 방출 회복{" "}
                    {reserve.actual_healed}
                  </p>
                  <p>착용자 {reserve.target}</p>
                  <p>
                    사본{" "}
                    {reserve.equipment.persistent_copy_uuid ??
                      reserve.equipment.runtime_item_instance_id}
                  </p>
                  <p>
                    가장 최근 저장 또는 방출의 관측입니다. 전투 중 저장량이며
                    자기 방출 회복은 다시 저장하지 않습니다.
                  </p>
                  {reserve.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:overheal-reserve`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={reserve.origin}
                    />
                  ) : (
                    <p>이 장비 회복은 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {echo && (
                <div role="group" aria-label="기본 치료 자기 회복 관측">
                  <p>
                    기본 치료 자기 회복 · {echo.definition_id} · {echo.time_ms}
                    ms
                  </p>
                  <p>
                    다른 아군 실제 회복 합계 {echo.actual_other_healed} · 자기
                    회복 비율 {echo.actual_heal_percent}%
                  </p>
                  <p>
                    행동당 상한: 발동 당시 최대 HP {echo.max_hp}의{" "}
                    {echo.max_hp_cap_percent}%
                  </p>
                  <p>
                    요청 회복 {echo.requested_self_heal} → 실제 자기 회복{" "}
                    {echo.actual_self_healed}
                  </p>
                  <p>
                    착용자 {echo.actor} · 기본 행동{" "}
                    {echo.healing_source.action_ordinal}
                  </p>
                  <p>
                    사본{" "}
                    {echo.equipment.persistent_copy_uuid ??
                      echo.equipment.runtime_item_instance_id}
                  </p>
                  <p>
                    가장 최근 기본 치료 행동의 관측입니다. 자기 치료분과 대상의
                    초과 회복은 합계에서 제외합니다.
                  </p>
                  {echo.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:basic-heal-echo`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={echo.origin}
                    />
                  ) : (
                    <p>이 장비 회복은 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {healBarrier && (
                <div role="group" aria-label="회복 결계 관측">
                  <p>
                    회복 결계 · {healBarrier.definition_id} ·{" "}
                    {healBarrier.time_ms}ms
                  </p>
                  <p>
                    회복 전 HP {healBarrier.hp_before} / {healBarrier.max_hp} ·
                    실제 회복 {healBarrier.actual_healed}
                  </p>
                  <p>
                    기본 결계 {healBarrier.base_capacity} → 증폭 후{" "}
                    {healBarrier.increased_capacity} · 남은 양{" "}
                    {healBarrier.remaining}
                  </p>
                  <p>
                    {healBarrier.kept_existing
                      ? "기존 결계 유지"
                      : "회복에 따른 결계량 반영"}{" "}
                    · 만료 {healBarrier.expires_at_ms}ms · 재발동 가능{" "}
                    {healBarrier.ready_at_ms}ms
                  </p>
                  <p>
                    제공자 {healBarrier.actor} · 대상 {healBarrier.target}
                  </p>
                  <p>
                    사본{" "}
                    {healBarrier.equipment.persistent_copy_uuid ??
                      healBarrier.equipment.runtime_item_instance_id}
                  </p>
                  <p>
                    가장 최근 결계 발동의 관측입니다. 같은 시각의 회복 성분은
                    합산하며 남은 양은 이후 피해에 따라 줄어듭니다.
                  </p>
                  {healBarrier.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:heal-barrier`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={healBarrier.origin}
                    />
                  ) : (
                    <p>이 장비 결계는 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {support && (
                <div role="group" aria-label="장비 지원 관측">
                  <p>
                    {support.channel === "heal" ? "회복" : "보호막"} 지원 ·{" "}
                    {support.time_ms}ms · +{support.percent}%
                  </p>
                  <p>
                    원량 {support.base_amount} → 증폭량{" "}
                    {support.increased_amount}
                  </p>
                  <p>
                    제공자 {support.actor} · 대상 {support.target}
                  </p>
                  <p>
                    가장 최근 지원량 보정입니다. 회복의 실제 적용량은 대상의 HP
                    상한을 따릅니다.
                  </p>
                  {support.contributors.map((item, index) => (
                    <div key={item.equipment.runtime_item_instance_id}>
                      <p>
                        {item.definition_id} · +{item.percent}%
                      </p>
                      <p>
                        사본{" "}
                        {item.equipment.persistent_copy_uuid ??
                          item.equipment.runtime_item_instance_id}
                      </p>
                      {support.origins[index] ? (
                        <CompositionRunSource
                          key={`${result.identity?.run_id}:support:${index}`}
                          source={completed.definition.ticket.source}
                          currentSource={currentSource}
                          origin={support.origins[index]}
                        />
                      ) : (
                        <p>
                          이 장비 기여는 현재 조합 원본과 연결되지 않았습니다.
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              )}
              {readyWait && (
                <div role="group" aria-label="시전 대기 피해 증가 관측">
                  <p>
                    마지막 시전 · {readyWait.time_ms}ms · {readyWait.skill_id}
                  </p>
                  <p>
                    가하는 피해 +{readyWait.percent}% · 착용자 {readyWait.actor}
                  </p>
                  <p>
                    시작 때 소비한 대기 기록입니다. 현재 활성 여부나 최종 HP
                    피해가 아닙니다.
                  </p>
                  {readyWait.contributors.map((item, index) => (
                    <div
                      key={`${item.equipment.runtime_item_instance_id}-${index}`}
                    >
                      <p>
                        {item.definition_id} · 대기 {item.waited_ms}ms · 초당{" "}
                        {item.per_second_percent}% · 상한 {item.max_percent}%
                      </p>
                      <p>
                        기여 +{item.percent}% · 만료 {item.until_ms}ms · 장비
                        사본 {item.equipment.persistent_copy_uuid}
                      </p>
                      {readyWait.origins[index] ? (
                        <CompositionRunSource
                          key={`${result.identity?.run_id}:ready-wait:${index}`}
                          source={completed.definition.ticket.source}
                          currentSource={currentSource}
                          origin={readyWait.origins[index]}
                        />
                      ) : (
                        <p>이 실행 세대에 연결된 원본 위치가 없습니다.</p>
                      )}
                    </div>
                  ))}
                </div>
              )}
              {nearbyDefense && (
                <div role="group" aria-label="주변 적 방어력 관측">
                  <p>
                    주변 적 방어력 · {nearbyDefense.time_ms}ms · 착용자{" "}
                    {nearbyDefense.actor}
                  </p>
                  <p>
                    고정 기여 +{nearbyDefense.bonus} · 최종 방어력{" "}
                    {nearbyDefense.actual_defense}
                  </p>
                  {nearbyDefense.contributors.map((item, index) => (
                    <div
                      key={`${item.equipment.runtime_item_instance_id}-${index}`}
                    >
                      <p>
                        {item.definition_id} · 생존 적 {item.count}명 ×{" "}
                        {item.per_enemy} · 상한 {item.max_bonus} · 기여 +
                        {item.bonus}
                      </p>
                      <p>장비 사본 {item.equipment.persistent_copy_uuid}</p>
                      {nearbyDefense.origins[index] ? (
                        <CompositionRunSource
                          key={`${result.identity?.run_id}:nearby-defense:${index}`}
                          source={completed.definition.ticket.source}
                          currentSource={currentSource}
                          origin={nearbyDefense.origins[index]}
                        />
                      ) : (
                        <p>이 실행 세대에 연결된 원본 위치가 없습니다.</p>
                      )}
                    </div>
                  ))}
                </div>
              )}
              {areaDamage && (
                <div role="group" aria-label="범위 피해 관측">
                  <p>
                    범위 피해 판정 · {areaDamage.time_ms}ms · 행동{" "}
                    {areaDamage.action ?? "독립 효과"}
                  </p>
                  <p>
                    고유 적 {areaDamage.target_count}명 · 범위 피해 +
                    {areaDamage.percent}%
                  </p>
                  <p>공격자 {areaDamage.actor}</p>
                  <p>
                    피해 적용 직전에 확정한 적 수로 계산합니다. 보호막·회피 등
                    방어 결과는 수에 영향을 주지 않습니다.
                  </p>
                  {areaDamage.contributors.map((item, index) => (
                    <div key={item.equipment.runtime_item_instance_id}>
                      <p>
                        {item.definition_id} · +{item.percent}%
                      </p>
                      <p>사본 {item.equipment.persistent_copy_uuid}</p>
                      {areaDamage.origins[index] ? (
                        <CompositionRunSource
                          key={`${result.identity?.run_id}:area-damage:${index}`}
                          source={completed.definition.ticket.source}
                          currentSource={currentSource}
                          origin={areaDamage.origins[index]}
                        />
                      ) : (
                        <p>
                          이 장비 기여는 현재 조합 원본과 연결되지 않았습니다.
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              )}
              {healReaction && (
                <div role="group" aria-label="실제 치유 반응 관측">
                  <p>
                    실제 치유 반응 · {healReaction.time_ms}ms · 이 대상의 실제
                    회복 {healReaction.actual_healed} HP
                  </p>
                  <p>
                    치유자 {healReaction.actor} · 대상 {healReaction.target}
                  </p>
                  {healReaction.healing_source.action_ordinal != null && (
                    <p>
                      회복 행동 {healReaction.healing_source.action_ordinal}
                    </p>
                  )}
                  {healReaction.contributors.map((item, index) => (
                    <div key={item.equipment.runtime_item_instance_id}>
                      <p>
                        {item.definition_id} ·{" "}
                        {item.equipment.persistent_copy_uuid
                          ? `사본 ${item.equipment.persistent_copy_uuid}`
                          : `전투 장비 ${item.equipment.runtime_item_instance_id}`}
                      </p>
                      {item.self_shield && (
                        <>
                          <p>
                            자신의 보호막 · 실제 회복의{" "}
                            {item.self_shield.actual_heal_percent}% · 관측 시
                            잔량 {item.self_shield.before} →{" "}
                            {item.self_shield.after} / 상한{" "}
                            {item.self_shield.cap} (최대 HP의{" "}
                            {item.self_shield.max_hp_cap_percent}%)
                          </p>
                          <p>
                            소수 잔여 {item.self_shield.remainder_before}/100 →{" "}
                            {item.self_shield.remainder_after}/100 · 만료{" "}
                            {item.self_shield.expires_at_ms}ms
                          </p>
                        </>
                      )}
                      {item.target_haste && (
                        <p>
                          치유 대상 공격 속도 +
                          {item.target_haste.attack_speed_points} ·{" "}
                          {item.target_haste.refreshed
                            ? "지속 시간 갱신"
                            : "부여"}{" "}
                          · 만료 {item.target_haste.expires_at_ms}ms (같은 계열
                          최댓값 적용)
                        </p>
                      )}
                      {healReaction.origins[index] ? (
                        <CompositionRunSource
                          key={`${result.identity?.run_id}:actual-heal-reaction:${index}`}
                          source={completed.definition.ticket.source}
                          currentSource={currentSource}
                          origin={healReaction.origins[index]}
                        />
                      ) : (
                        <p>
                          이 장비 기여는 현재 조합 원본과 연결되지 않았습니다.
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              )}
              {resonanceGain && (
                <div role="group" aria-label="공명 획득 관측">
                  <p>
                    공명 획득 · {resonanceGain.time_ms}ms · 획득량 +
                    {resonanceGain.percent}%
                  </p>
                  <p>
                    착용자 {resonanceGain.actor} ·{" "}
                    {resonanceGain.channel === "automatic"
                      ? "자동 충전"
                      : "작성된 추가 획득"}
                  </p>
                  <p>
                    기본 {resonanceGain.base_amount} → 증가 후{" "}
                    {resonanceGain.increased_amount} · 실제 획득{" "}
                    {resonanceGain.actual_gain}
                  </p>
                  <p>
                    공명 {resonanceGain.before} → {resonanceGain.after} /{" "}
                    {resonanceGain.max}
                  </p>
                  <p>
                    소수 잔여 {resonanceGain.remainder_before}/100 →{" "}
                    {resonanceGain.remainder_after}/100 · 만충 시 잔여분과
                    초과량을 버립니다.
                  </p>
                  {resonanceGain.contributors.map((item, index) => (
                    <div key={item.equipment.runtime_item_instance_id}>
                      <p>
                        {item.definition_id} · +{item.percent}%
                      </p>
                      <p>
                        {item.equipment.persistent_copy_uuid
                          ? `사본 ${item.equipment.persistent_copy_uuid}`
                          : `전투 장비 ${item.equipment.runtime_item_instance_id}`}
                      </p>
                      {resonanceGain.origins[index] ? (
                        <CompositionRunSource
                          key={`${result.identity?.run_id}:resonance-gain:${index}`}
                          source={completed.definition.ticket.source}
                          currentSource={currentSource}
                          origin={resonanceGain.origins[index]}
                        />
                      ) : (
                        <p>
                          이 장비 기여는 현재 조합 원본과 연결되지 않았습니다.
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              )}
              {nearbyAllyDamage && (
                <div role="group" aria-label="주변 아군 피해 관측">
                  <p>
                    주변 아군 피해 · {nearbyAllyDamage.time_ms}ms · 피해 +
                    {nearbyAllyDamage.percent}%
                  </p>
                  <p>착용자 {nearbyAllyDamage.wearer}</p>
                  <p>
                    공격자 {nearbyAllyDamage.actor} · 대상{" "}
                    {nearbyAllyDamage.target}
                  </p>
                  <p>
                    피해가 발생할 때 착용자 주변의 다른 배치 직원을 셉니다.
                    소환물·전장 물체는 인원에 포함하지 않습니다.
                  </p>
                  {nearbyAllyDamage.contributors.map((item, index) => (
                    <div key={item.equipment.runtime_item_instance_id}>
                      <p>
                        {item.definition_id} · 주변 직원 {item.count}명 · +
                        {item.percent}%
                      </p>
                      <p>사본 {item.equipment.persistent_copy_uuid}</p>
                      {nearbyAllyDamage.origins[index] ? (
                        <CompositionRunSource
                          key={`${result.identity?.run_id}:nearby-ally-damage:${index}`}
                          source={completed.definition.ticket.source}
                          currentSource={currentSource}
                          origin={nearbyAllyDamage.origins[index]}
                        />
                      ) : (
                        <p>
                          이 장비 기여는 현재 조합 원본과 연결되지 않았습니다.
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              )}
              {conditionalDamage && (
                <div role="group" aria-label="조건부 피해 관측">
                  <p>
                    조건부 피해 · {conditionalDamage.time_ms}ms · 피해 +
                    {conditionalDamage.percent}%
                  </p>
                  <p>
                    착용자 {conditionalDamage.wearer} · HP{" "}
                    {conditionalDamage.wearer_hp}/
                    {conditionalDamage.wearer_max_hp}
                  </p>
                  <p>
                    공격자 {conditionalDamage.actor} · 대상{" "}
                    {conditionalDamage.target} · HP{" "}
                    {conditionalDamage.target_hp}/
                    {conditionalDamage.target_max_hp}
                  </p>
                  <p>
                    대상 이동 방해 ·{" "}
                    {conditionalDamage.target_movement_impaired === null
                      ? "이번 조건에서 조회하지 않음"
                      : conditionalDamage.target_movement_impaired
                        ? "적용 중"
                        : "없음"}
                  </p>
                  <p>
                    타격이 시작될 때의 조건을 같은 타격의 피해에 적용합니다.
                  </p>
                  {conditionalDamage.distance_squared != null && (
                    <p>
                      판정 거리 ·{" "}
                      {Math.sqrt(conditionalDamage.distance_squared).toFixed(3)}
                      타일
                    </p>
                  )}
                  {conditionalDamage.contributors.map((item, index) => (
                    <div key={item.equipment.runtime_item_instance_id}>
                      <p>
                        {item.definition_id} · 조건{" "}
                        {item.matched ? "충족" : "미충족"} · +{item.percent}%
                      </p>
                      <p>사본 {item.equipment.persistent_copy_uuid}</p>
                      {conditionalDamage.origins[index] ? (
                        <CompositionRunSource
                          key={`${result.identity?.run_id}:conditional-damage:${index}`}
                          source={completed.definition.ticket.source}
                          currentSource={currentSource}
                          origin={conditionalDamage.origins[index]}
                        />
                      ) : (
                        <p>
                          이 장비 기여는 현재 조합 원본과 연결되지 않았습니다.
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              )}
              {conditionalReduction && (
                <div role="group" aria-label="조건부 감면 관측">
                  <p>
                    조건부 감면 · {conditionalReduction.time_ms}ms · 받는 피해 −
                    {conditionalReduction.percent}%
                  </p>
                  <p>
                    착용자 {conditionalReduction.wearer} · HP{" "}
                    {conditionalReduction.wearer_hp}/
                    {conditionalReduction.wearer_max_hp}
                  </p>
                  <p>
                    공격자 {conditionalReduction.actor} · 대상{" "}
                    {conditionalReduction.target} · HP{" "}
                    {conditionalReduction.target_hp}/
                    {conditionalReduction.target_max_hp}
                  </p>
                  <p>
                    대상 이동 방해 ·{" "}
                    {conditionalReduction.target_movement_impaired === null
                      ? "이번 조건에서 조회하지 않음"
                      : conditionalReduction.target_movement_impaired
                        ? "적용 중"
                        : "없음"}
                  </p>
                  <p>
                    현재 조건에서 가장 강한 장비 사본 하나를 적용합니다. 선택
                    사본 {conditionalReduction.equipment.persistent_copy_uuid}
                  </p>
                  <p>
                    판정 거리 ·{" "}
                    {conditionalReduction.distance_squared == null
                      ? "이번 조건에서 조회하지 않음"
                      : `${Math.sqrt(conditionalReduction.distance_squared).toFixed(3)}타일`}
                  </p>
                  {conditionalReduction.contributors.map((item, index) => (
                    <div key={item.equipment.runtime_item_instance_id}>
                      <p>
                        {item.definition_id} · 조건{" "}
                        {item.matched ? "충족" : "미충족"} · −{item.percent}%
                      </p>
                      <p>사본 {item.equipment.persistent_copy_uuid}</p>
                      {conditionalReduction.origins[index] ? (
                        <CompositionRunSource
                          key={`${result.identity?.run_id}:conditional-reduction:${index}`}
                          source={completed.definition.ticket.source}
                          currentSource={currentSource}
                          origin={conditionalReduction.origins[index]}
                        />
                      ) : (
                        <p>
                          이 장비 기여는 현재 조합 원본과 연결되지 않았습니다.
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              )}
              {primaryDamage && (
                <div role="group" aria-label="주 대상 피해 관측">
                  <p>
                    기본 공격 주 대상 · {primaryDamage.time_ms}ms · 행동{" "}
                    {primaryDamage.action}
                  </p>
                  <p>
                    적 HP {primaryDamage.target_current_hp} /{" "}
                    {primaryDamage.target_max_hp} · 직접 피해 +
                    {primaryDamage.percent}%
                  </p>
                  <p>
                    공격자 {primaryDamage.actor} · 주 대상{" "}
                    {primaryDamage.target}
                  </p>
                  <p>타격 직전 현재 HP로 계산한 장비 기여입니다.</p>
                  {primaryDamage.contributors.map((item, index) => (
                    <div key={item.equipment.runtime_item_instance_id}>
                      <p>
                        {item.definition_id} · +{item.percent}%
                      </p>
                      <p>사본 {item.equipment.persistent_copy_uuid}</p>
                      {primaryDamage.origins[index] ? (
                        <CompositionRunSource
                          key={`${result.identity?.run_id}:primary-damage:${index}`}
                          source={completed.definition.ticket.source}
                          currentSource={currentSource}
                          origin={primaryDamage.origins[index]}
                        />
                      ) : (
                        <p>
                          이 장비 기여는 현재 조합 원본과 연결되지 않았습니다.
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              )}
              {speed && (
                <div role="group" aria-label="장비 공속 관측">
                  <p>
                    기본 공격 시작 · {speed.time_ms}ms · HP {speed.current_hp} /{" "}
                    {speed.max_hp}
                  </p>
                  <p>
                    공격 주기 {speed.base_interval_ms} →{" "}
                    {speed.final_interval_ms}ms · 장비 공속 +
                    {speed.equipment_points}포인트
                  </p>
                  <p>
                    장비 계산식이 적용된 최근 기본 공격의 시작 시점 값입니다.
                  </p>
                  {speed.contributors.map((item, index) => (
                    <div key={item.equipment.runtime_item_instance_id}>
                      <p>
                        {item.definition_id} · +{item.points}포인트
                      </p>
                      <p>
                        사본{" "}
                        {item.equipment.persistent_copy_uuid ??
                          item.equipment.runtime_item_instance_id}
                      </p>
                      {speed.origins[index] ? (
                        <CompositionRunSource
                          key={`${result.identity?.run_id}:speed:${index}`}
                          source={completed.definition.ticket.source}
                          currentSource={currentSource}
                          origin={speed.origins[index]}
                        />
                      ) : (
                        <p>
                          이 장비 기여는 현재 조합 원본과 연결되지 않았습니다.
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              )}
              {area && (
                <div role="group" aria-label="장판 관측">
                  <p>
                    장판{" "}
                    {area.change.kind === "ended"
                      ? "종료"
                      : area.change.kind === "started"
                        ? "시작"
                        : "갱신"}
                    {" · "}
                    {area.time_ms}ms · {area.skill_id}
                  </p>
                  <p>
                    시작 {area.started_at_ms}ms · 예정 만료 {area.expires_at_ms}
                    ms
                  </p>
                  <p>
                    영역 {area.area_id} · 시전 {area.cast_sequence}
                  </p>
                  {area.change.kind === "ended" && (
                    <p>종료 사유 · {sustainedEndLabels[area.change.reason]}</p>
                  )}
                  {(area.before || area.after) && (
                    <p>
                      영역 안 대상 {area.before?.members ?? 0} →{" "}
                      {area.after?.members ?? 0}
                      {" · "}상태 기여 {area.before?.contributions ?? 0} →{" "}
                      {area.after?.contributions ?? 0}
                    </p>
                  )}
                  <p>
                    전투의 최신 장판 관측 한 건입니다. 전체 장판 목록이나 이력이
                    아닙니다.
                  </p>
                  {area.origin ? (
                    <CompositionRunSource
                      key={`${result.identity?.run_id}:area`}
                      source={completed.definition.ticket.source}
                      currentSource={currentSource}
                      origin={area.origin}
                    />
                  ) : (
                    <p>이 장판은 현재 조합 원본과 연결되지 않았습니다.</p>
                  )}
                </div>
              )}
              {result.latest_resonance?.origin && (
                <div role="group" aria-label="공명 효과 관측">
                  <p>
                    공명 변화{" "}
                    {result.latest_resonance.after -
                      result.latest_resonance.before >=
                    0
                      ? "+"
                      : ""}
                    {result.latest_resonance.after -
                      result.latest_resonance.before}{" "}
                    · {result.latest_resonance.time_ms}ms ·{" "}
                    {result.latest_resonance.definition_id}
                  </p>
                  <p>
                    {result.latest_resonance.before} →{" "}
                    {result.latest_resonance.after} /{" "}
                    {result.latest_resonance.max}
                    {result.latest_resonance.battle_terminal
                      ? " · 전투 종료 시 보존된 관측"
                      : ""}
                  </p>
                  <CompositionRunSource
                    key={`${result.identity?.run_id}:resonance`}
                    source={completed.definition.ticket.source}
                    currentSource={currentSource}
                    origin={result.latest_resonance.origin}
                  />
                </div>
              )}
              {result.latest_effect?.origin ? (
                <div role="group" aria-label="HP 효과 관측">
                  <p>
                    작성 효과 관측 · {result.latest_effect.time_ms}ms ·{" "}
                    {result.latest_effect.definition_id} ·{" "}
                    {result.latest_effect.hp_reason === "HealthCost"
                      ? "HP 비용"
                      : "HP 변화"}{" "}
                    {result.latest_effect.hp_delta}
                  </p>
                  {result.latest_effect.equipment && (
                    <p>
                      장비 사본{" "}
                      {result.latest_effect.equipment.persistent_copy_uuid ??
                        result.latest_effect.equipment.runtime_item_instance_id}
                    </p>
                  )}
                  <CompositionRunSource
                    key={result.identity?.run_id}
                    source={completed.definition.ticket.source}
                    currentSource={currentSource}
                    origin={result.latest_effect.origin}
                  />
                </div>
              ) : result.latest_resonance?.origin ||
                healReaction ||
                hitCounter ||
                sustained?.target_state ||
                sustained?.basic_speed_reduction_percent ||
                sustained?.damage_taken_increase_percent ||
                (weaponStage && weaponStage.actual_hp_loss > 0) ? null : (
                <p>
                  최근 관측에 이 조합의 HP 변화가 없습니다. 이 결과만으로 스킬
                  사용 성공을 판단하지 마세요.
                </p>
              )}
              {hitCounter?.origin && (
                <div role="group" aria-label="적중 누적 관측">
                  <p>
                    적중 누적 · {hitCounter.time_ms}ms ·{" "}
                    {hitCounter.definition_id}
                  </p>
                  <p>대상 {hitCounter.target_unit}</p>
                  <p>
                    관측 시점 누적 {hitCounter.before} + {hitCounter.added} →{" "}
                    {hitCounter.after} · 임계치 {hitCounter.threshold}
                  </p>
                  <p>
                    {hitCounter.triggered ? "임계치 소모" : "누적 유지"} ·{" "}
                    {hitCounter.control === "Stun" ? "기절" : "속박"}{" "}
                    {hitCounter.control_applied
                      ? "적용"
                      : hitCounter.triggered
                        ? "미적용"
                        : "미발동"}
                  </p>
                  {hitCounter.counter_expires_at_ms !== null && (
                    <p>누적 만료 시각 · {hitCounter.counter_expires_at_ms}ms</p>
                  )}
                  {hitCounter.control_expires_at_ms !== null && (
                    <p>제어 만료 시각 · {hitCounter.control_expires_at_ms}ms</p>
                  )}
                  {hitCounter.equipment && (
                    <p>
                      장비 사본{" "}
                      {hitCounter.equipment.persistent_copy_uuid ??
                        hitCounter.equipment.runtime_item_instance_id}
                    </p>
                  )}
                  <p>
                    {hitCounter.battle_terminal
                      ? "종료한 전투의 관측입니다."
                      : "이 적중에서 확정된 값입니다. 현재 상태 전체를 나타내지는 않습니다."}
                  </p>
                  <CompositionRunSource
                    key={result.identity?.run_id}
                    source={completed.definition.ticket.source}
                    currentSource={currentSource}
                    origin={hitCounter.origin}
                  />
                </div>
              )}
              {sustained?.origin && (
                <div role="group" aria-label="지속 효과 관측">
                  <p>
                    지속 효과 {sustained.ended ? "종료" : "시작"} ·{" "}
                    {sustained.time_ms}ms · {sustained.definition_id}
                  </p>
                  <p>
                    대상 {sustained.target_unit} · 활성 {sustained.activation}
                  </p>
                  <p>만료 시각 · {sustained.expires_at_ms}ms</p>
                  {sustained.damage_taken_increase_percent != null && (
                    <p>
                      부여한 받는 피해 증가 · +
                      {sustained.damage_taken_increase_percent}%
                    </p>
                  )}
                  {sustained.basic_speed_reduction_percent != null && (
                    <p>
                      부여한 기본 공격 속도 감소 · -
                      {sustained.basic_speed_reduction_percent}%
                    </p>
                  )}
                  {sustained.target_state &&
                    (sustained.target_state.damage_invulnerable ||
                      sustained.target_state.basic_attack_locked ||
                      sustained.target_state.active_skill_locked ||
                      sustained.target_state.received_heal_percent > 0) && (
                      <p>
                        부여한 상태
                        {sustained.target_state.damage_invulnerable &&
                          " · 피해 무효"}
                        {sustained.target_state.basic_attack_locked &&
                          " · 기본 공격 잠금"}
                        {sustained.target_state.active_skill_locked &&
                          " · active 시전 잠금"}
                        {sustained.target_state.received_heal_percent > 0 &&
                          ` · 받는 회복 +${sustained.target_state.received_heal_percent}%`}
                      </p>
                    )}
                  {sustained.equipment && (
                    <p>
                      장비 사본{" "}
                      {sustained.equipment.persistent_copy_uuid ??
                        sustained.equipment.runtime_item_instance_id}
                    </p>
                  )}
                  {sustained.area_source && (
                    <>
                      <p>
                        생성한 장판 · {sustained.area_source.skill_id} /{" "}
                        {sustained.area_source.step_id} · 상태{" "}
                        {sustained.area_source.status_index + 1}
                      </p>
                      {sustained.area_source.modifier_source && (
                        <p>
                          유물 ·{" "}
                          {sustained.area_source.modifier_source.definition} ·
                          사본 {sustained.area_source.modifier_source.copy_uuid}
                        </p>
                      )}
                    </>
                  )}
                  {sustained.ended && (
                    <p>종료 사유 · {sustainedEndLabels[sustained.ended]}</p>
                  )}
                  {sustained.before && sustained.after && (
                    <p>
                      공격력 {sustained.before.attack} →{" "}
                      {sustained.after.attack} · 방어력{" "}
                      {sustained.before.defense} → {sustained.after.defense}
                    </p>
                  )}
                  {sustained.before?.move_speed_units_per_ms !== undefined &&
                    sustained.after?.move_speed_units_per_ms !== undefined &&
                    sustained.before.move_speed_units_per_ms !==
                      sustained.after.move_speed_units_per_ms && (
                      <p>
                        이동 속도{" "}
                        {sustained.before.move_speed_units_per_ms / 1000} →{" "}
                        {sustained.after.move_speed_units_per_ms / 1000} 타일/초
                      </p>
                    )}
                  <p>
                    {sustained.battle_terminal
                      ? "종료한 전투의 최근 지속 효과 관측입니다."
                      : "최근 지속 효과 한 건을 표시합니다."}
                  </p>
                  <CompositionRunSource
                    key={result.identity?.run_id}
                    source={completed.definition.ticket.source}
                    currentSource={currentSource}
                    origin={sustained.origin}
                  />
                </div>
              )}
              {result.failure && (
                <pre>{JSON.stringify(result.failure, null, 2)}</pre>
              )}
              <details>
                <summary>버전과 원본 추적</summary>
                <pre>
                  {JSON.stringify(
                    {
                      identity: result.identity,
                      effect: result.latest_effect,
                      resonance: result.latest_resonance,
                      sustained,
                      hitCounter,
                    },
                    null,
                    2,
                  )}
                </pre>
              </details>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </>
  );
}
