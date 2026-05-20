import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useState, type ReactNode } from "react";
import {
  describeDelivery,
  describeEffect,
  describeTarget,
  skillToRonPreview,
  validateSkillDraft,
  type AreaDraft,
  type AreaShapeDraft,
  type DamageModifiersDraft,
  type DeliveryDraft,
  type EffectDraft,
  type SkillDraft,
  type SkillStepDraft,
  type StepConditionDraft,
  type StepRepeatDraft,
  type TargetDraft,
  type ValidationMessage,
} from "./skillDraft";
import { sampleSkills } from "./sampleSkills";

const TILE_UNITS_PER_TILE = 1_000_000;
const DEFAULT_UNIT_HITBOX_RADIUS_TILES = 0.25;

function App() {
  const [skills, setSkills] = useState<SkillDraft[]>(sampleSkills);
  const [editingSkills, setEditingSkills] = useState<SkillDraft[] | null>(null);
  const [selectedSkillId, setSelectedSkillId] = useState(sampleSkills[0]?.id ?? "");
  const [selectedStepId, setSelectedStepId] = useState(sampleSkills[0]?.steps[0]?.id ?? "");
  const [query, setQuery] = useState("");
  const [dataSource, setDataSource] = useState<"sample" | "ron">("sample");
  const [loadStatus, setLoadStatus] = useState("Using bundled sample data");
  const [exportStatus, setExportStatus] = useState<ValidationMessage | null>(null);
  const [isExporting, setIsExporting] = useState(false);
  const [coreValidationMessages, setCoreValidationMessages] = useState<ValidationMessage[]>([]);
  const [isValidating, setIsValidating] = useState(false);
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);
  const activeSkills = editingSkills ?? skills;
  const isEditMode = editingSkills !== null;

  useEffect(() => {
    if (!isTauriRuntime()) {
      setLoadStatus("Browser preview: using sample data until the Tauri shell is running");
      return;
    }

    invoke<SkillDraft[]>("load_default_skills")
      .then((loadedSkills) => {
        if (loadedSkills.length === 0) {
          setLoadStatus("Loaded skill file, but it did not contain any skills");
          return;
        }

        setSkills(loadedSkills);
        setEditingSkills(null);
        setSelectedSkillId(loadedSkills[0].id);
        setSelectedStepId(loadedSkills[0].steps[0]?.id ?? "");
        setDataSource("ron");
        setHasUnsavedChanges(false);
        setCoreValidationMessages([]);
        setLoadStatus(`Loaded ${loadedSkills.length} skills from game_resources/data/skills/base.ron`);
      })
      .catch((error: unknown) => {
        setDataSource("sample");
        setLoadStatus(`Could not load RON through Tauri: ${String(error)}`);
      });
  }, []);

  const filteredSkills = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    if (!normalizedQuery) {
      return activeSkills;
    }

    return activeSkills.filter(
      (skill) =>
        skill.id.toLowerCase().includes(normalizedQuery) ||
        skill.name.toLowerCase().includes(normalizedQuery),
    );
  }, [activeSkills, query]);

  const selectedSkill =
    activeSkills.find((skill) => skill.id === selectedSkillId) ?? activeSkills[0] ?? sampleSkills[0];
  const selectedStep =
    selectedSkill.steps.find((step) => step.id === selectedStepId) ?? selectedSkill.steps[0];
  const selectedStepIndex = selectedSkill.steps.findIndex((step) => step.id === selectedStep.id);
  const uiValidationMessages = useMemo(
    () => [...validateSkillDraft(selectedSkill), ...validateSkillCollection(activeSkills)],
    [activeSkills, selectedSkill],
  );
  const validationMessages = useMemo(
    () => [...uiValidationMessages, ...coreValidationMessages],
    [coreValidationMessages, uiValidationMessages],
  );
  const hasValidationErrors = validationMessages.some((message) => message.severity === "error");
  const ronPreview = useMemo(() => skillToRonPreview(selectedSkill), [selectedSkill]);

  function selectSkill(skill: SkillDraft) {
    setSelectedSkillId(skill.id);
    setSelectedStepId(skill.steps[0]?.id ?? "");
  }

  function beginEdit() {
    setEditingSkills(cloneDraft(skills));
    setCoreValidationMessages([]);
    setExportStatus(null);
  }

  function commitEdit() {
    if (!editingSkills) {
      return;
    }

    setSkills(editingSkills);
    setEditingSkills(null);
    setHasUnsavedChanges(true);
    setCoreValidationMessages([]);
    setExportStatus({
      severity: "info",
      label: "Draft committed",
      detail: "Run validation before exporting the generated RON.",
    });
  }

  function cancelEdit() {
    const nextSkill =
      skills.find((skill) => skill.id === selectedSkillId) ?? skills[0] ?? sampleSkills[0];
    const nextStep =
      nextSkill.steps.find((step) => step.id === selectedStepId) ?? nextSkill.steps[0];

    setEditingSkills(null);
    setSelectedSkillId(nextSkill.id);
    setSelectedStepId(nextStep?.id ?? "");
    setCoreValidationMessages([]);
    setExportStatus(null);
  }

  function updateEditingSkills(updater: (currentSkills: SkillDraft[]) => SkillDraft[]) {
    setEditingSkills((currentSkills) => {
      const draftSkills = currentSkills ?? cloneDraft(skills);
      return updater(draftSkills);
    });
    setCoreValidationMessages([]);
    setExportStatus(null);
  }

  function createSkill() {
    const id = uniqueId("new_skill", activeSkills.map((skill) => skill.id));
    const step = createDefaultStep("step");
    const skill: SkillDraft = {
      id,
      name: "New Skill",
      kind: "Targeted",
      focusTimeMs: 300,
      focusPermissions: {
        allowsMove: false,
        allowsBasicAttack: false,
      },
      steps: [step],
    };

    updateEditingSkills((currentSkills) => [skill, ...currentSkills]);
    setSelectedSkillId(skill.id);
    setSelectedStepId(step.id);
    setQuery("");
  }

  function duplicateSelectedSkill() {
    const skill = cloneDraft(selectedSkill);
    const id = uniqueId(`${skill.id}_copy`, activeSkills.map((candidate) => candidate.id));
    skill.id = id;
    skill.name = `${skill.name} Copy`;

    updateEditingSkills((currentSkills) => [skill, ...currentSkills]);
    setSelectedSkillId(skill.id);
    setSelectedStepId(skill.steps[0]?.id ?? "");
    setQuery("");
  }

  function deleteSelectedSkill() {
    if (activeSkills.length <= 1) {
      return;
    }

    const nextSkills = activeSkills.filter((skill) => skill.id !== selectedSkill.id);
    const nextSelectedSkill = nextSkills[0];

    updateEditingSkills(() => nextSkills);
    setSelectedSkillId(nextSelectedSkill.id);
    setSelectedStepId(nextSelectedSkill.steps[0]?.id ?? "");
  }

  function addStep() {
    const step = createDefaultStep(uniqueId("step", selectedSkill.steps.map((candidate) => candidate.id)));

    updateEditingSkills((currentSkills) =>
      currentSkills.map((skill) =>
        skill.id === selectedSkill.id
          ? {
              ...skill,
              steps: [...skill.steps, step],
            }
          : skill,
      ),
    );
    setSelectedStepId(step.id);
  }

  function duplicateSelectedStep() {
    const step = cloneDraft(selectedStep);
    step.id = uniqueId(`${step.id}_copy`, selectedSkill.steps.map((candidate) => candidate.id));

    updateEditingSkills((currentSkills) =>
      currentSkills.map((skill) =>
        skill.id === selectedSkill.id
          ? {
              ...skill,
              steps: [...skill.steps, step],
            }
          : skill,
      ),
    );
    setSelectedStepId(step.id);
  }

  function deleteSelectedStep() {
    if (selectedSkill.steps.length <= 1) {
      return;
    }

    const nextSteps = selectedSkill.steps.filter((step) => step.id !== selectedStep.id);
    const nextStep = nextSteps[0];

    updateEditingSkills((currentSkills) =>
      currentSkills.map((skill) =>
        skill.id === selectedSkill.id
          ? {
              ...skill,
              steps: nextSteps,
            }
          : skill,
      ),
    );
    setSelectedStepId(nextStep.id);
  }

  function moveSelectedStep(direction: -1 | 1) {
    const nextIndex = selectedStepIndex + direction;
    if (selectedStepIndex < 0 || nextIndex < 0 || nextIndex >= selectedSkill.steps.length) {
      return;
    }

    updateEditingSkills((currentSkills) =>
      currentSkills.map((skill) =>
        skill.id === selectedSkill.id
          ? {
              ...skill,
              steps: moveArrayItem(skill.steps, selectedStepIndex, nextIndex),
            }
          : skill,
      ),
    );
  }

  function updateSelectedSkill(patch: Partial<SkillDraft>) {
    updateEditingSkills((currentSkills) =>
      currentSkills.map((skill) => (skill.id === selectedSkill.id ? { ...skill, ...patch } : skill)),
    );
  }

  function renameSelectedSkill(rawId: string) {
    const id = uniqueId(
      rawId,
      activeSkills.filter((skill) => skill.id !== selectedSkill.id).map((skill) => skill.id),
    );

    updateEditingSkills((currentSkills) =>
      currentSkills.map((skill) => (skill.id === selectedSkill.id ? { ...skill, id } : skill)),
    );
    setSelectedSkillId(id);
  }

  function updateSelectedStep(patch: Partial<SkillStepDraft>) {
    updateEditingSkills((currentSkills) =>
      currentSkills.map((skill) =>
        skill.id === selectedSkill.id
          ? {
              ...skill,
              steps: skill.steps.map((step) =>
                step.id === selectedStep.id ? { ...step, ...patch } : step,
              ),
            }
          : skill,
      ),
    );
  }

  function renameSelectedStep(rawId: string) {
    const id = uniqueId(
      rawId,
      selectedSkill.steps.filter((step) => step.id !== selectedStep.id).map((step) => step.id),
    );

    updateEditingSkills((currentSkills) =>
      currentSkills.map((skill) =>
        skill.id === selectedSkill.id
          ? {
              ...skill,
              steps: skill.steps.map((step) => (step.id === selectedStep.id ? { ...step, id } : step)),
            }
          : skill,
      ),
    );
    setSelectedStepId(id);
  }

  function updateEffect(index: number, patch: Partial<SkillStepDraft["effects"][number]>) {
    updateEditingSkills((currentSkills) =>
      currentSkills.map((skill) =>
        skill.id === selectedSkill.id
          ? {
              ...skill,
              steps: skill.steps.map((step) =>
                step.id === selectedStep.id
                  ? {
                      ...step,
                      effects: step.effects.map((effect, effectIndex) =>
                        effectIndex === index ? ({ ...effect, ...patch } as SkillStepDraft["effects"][number]) : effect,
                      ),
                    }
                  : step,
              ),
            }
          : skill,
      ),
    );
  }

  function addEffect() {
    updateEditingSkills((currentSkills) =>
      currentSkills.map((skill) =>
        skill.id === selectedSkill.id
          ? {
              ...skill,
              steps: skill.steps.map((step) =>
                step.id === selectedStep.id
                  ? {
                      ...step,
                      effects: [...step.effects, defaultEffect("Damage")],
                    }
                  : step,
              ),
            }
          : skill,
      ),
    );
  }

  function duplicateEffect(index: number) {
    updateEditingSkills((currentSkills) =>
      currentSkills.map((skill) =>
        skill.id === selectedSkill.id
          ? {
              ...skill,
              steps: skill.steps.map((step) =>
                step.id === selectedStep.id
                  ? {
                      ...step,
                      effects: [
                        ...step.effects.slice(0, index + 1),
                        cloneDraft(step.effects[index]),
                        ...step.effects.slice(index + 1),
                      ],
                    }
                  : step,
              ),
            }
          : skill,
      ),
    );
  }

  function deleteEffect(index: number) {
    updateEditingSkills((currentSkills) =>
      currentSkills.map((skill) =>
        skill.id === selectedSkill.id
          ? {
              ...skill,
              steps: skill.steps.map((step) =>
                step.id === selectedStep.id
                  ? {
                      ...step,
                      effects: step.effects.filter((_, effectIndex) => effectIndex !== index),
                    }
                  : step,
              ),
            }
          : skill,
      ),
    );
  }

  function replaceEffect(index: number, effect: EffectDraft) {
    updateEditingSkills((currentSkills) =>
      currentSkills.map((skill) =>
        skill.id === selectedSkill.id
          ? {
              ...skill,
              steps: skill.steps.map((step) =>
                step.id === selectedStep.id
                  ? {
                      ...step,
                      effects: step.effects.map((currentEffect, effectIndex) =>
                        effectIndex === index ? effect : currentEffect,
                      ),
                    }
                  : step,
              ),
            }
          : skill,
      ),
    );
  }

  function moveEffect(index: number, direction: -1 | 1) {
    const nextIndex = index + direction;
    if (nextIndex < 0 || nextIndex >= selectedStep.effects.length) {
      return;
    }

    updateEditingSkills((currentSkills) =>
      currentSkills.map((skill) =>
        skill.id === selectedSkill.id
          ? {
              ...skill,
              steps: skill.steps.map((step) =>
                step.id === selectedStep.id
                  ? {
                      ...step,
                      effects: moveArrayItem(step.effects, index, nextIndex),
                    }
                  : step,
              ),
            }
          : skill,
      ),
    );
  }

  async function validateWithCore() {
    if (isEditMode) {
      setCoreValidationMessages([
        {
          severity: "warning",
          label: "Commit required",
          detail: "Commit or cancel the current edit draft before running core validation.",
        },
      ]);
      return;
    }

    const blockingMessage = firstBlockingValidationMessage(uiValidationMessages);
    if (blockingMessage) {
      setCoreValidationMessages([
        {
          severity: "error",
          label: "Fix UI validation first",
          detail: `${blockingMessage.label}: ${blockingMessage.detail}`,
          stepId: blockingMessage.stepId,
        },
      ]);
      return;
    }

    if (!isTauriRuntime()) {
      setCoreValidationMessages([
        {
          severity: "warning",
          label: "Core validation unavailable",
          detail: "Run the Tauri desktop app to validate through game_core. Browser preview only runs UI checks.",
        },
      ]);
      return;
    }

    setIsValidating(true);
    setCoreValidationMessages([
      {
        severity: "info",
        label: "Core validation running",
        detail: "Checking the full skill database through game_core.",
      },
    ]);

    try {
      const result = await runCoreValidation(skills);
      setCoreValidationMessages([
        {
          severity: "info",
          label: "Core validation passed",
          detail: `game_core accepted ${result.skillCount} skills.`,
        },
      ]);
    } catch (error) {
      setCoreValidationMessages([
        {
          severity: "error",
          label: "Core validation failed",
          detail: formatCoreError(error),
        },
      ]);
    } finally {
      setIsValidating(false);
    }
  }

  async function exportGeneratedRon() {
    if (isEditMode) {
      setExportStatus({
        severity: "warning",
        label: "Commit required",
        detail: "Commit or cancel the current edit draft before exporting RON.",
      });
      return;
    }

    const blockingMessage = firstBlockingValidationMessage(validationMessages);
    if (blockingMessage) {
      setExportStatus({
        severity: "error",
        label: "Export blocked",
        detail: `${blockingMessage.label}: ${blockingMessage.detail}`,
        stepId: blockingMessage.stepId,
      });
      return;
    }

    if (!isTauriRuntime()) {
      setExportStatus({
        severity: "warning",
        label: "Desktop app required",
        detail: "Export requires the Tauri desktop app. Browser preview cannot write files.",
      });
      return;
    }

    setIsExporting(true);
    setExportStatus({
      severity: "info",
      label: "Export running",
      detail: "Validating through game_core before writing generated RON.",
    });
    try {
      const validationResult = await runCoreValidation(skills);
      setCoreValidationMessages([
        {
          severity: "info",
          label: "Core validation passed",
          detail: `game_core accepted ${validationResult.skillCount} skills.`,
        },
      ]);
      const result = await invoke<{ path: string; skillCount: number }>("export_generated_skills", {
        skills,
      });
      setHasUnsavedChanges(false);
      setExportStatus({
        severity: "info",
        label: "Export complete",
        detail: `Exported ${result.skillCount} skills to ${result.path}`,
      });
    } catch (error) {
      setExportStatus({
        severity: "error",
        label: "Export failed",
        detail: formatCoreError(error),
      });
    } finally {
      setIsExporting(false);
    }
  }

  return (
    <main className="h-screen overflow-hidden bg-slate-950 text-slate-100">
      <div className="flex h-full min-h-0 flex-col">
        <header className="flex h-16 shrink-0 items-center justify-between border-b border-slate-800 bg-slate-950 px-6">
          <div>
            <h1 className="text-lg font-semibold tracking-wide">Skill Builder</h1>
            <p className="text-xs text-slate-400">{loadStatus}</p>
          </div>
          <div className="flex items-center gap-2">
            <span
              className={`rounded px-2 py-1 text-xs ${
                dataSource === "ron"
                  ? "bg-emerald-400/10 text-emerald-200"
                  : "bg-amber-400/10 text-amber-200"
              }`}
            >
              {dataSource === "ron" ? "RON data" : "Sample data"}
            </span>
            {hasUnsavedChanges ? (
              <span className="rounded bg-cyan-400/10 px-2 py-1 text-xs text-cyan-200">
                Unsaved changes
              </span>
            ) : null}
            {isEditMode ? (
              <>
                <span className="rounded bg-amber-400/10 px-2 py-1 text-xs text-amber-200">
                  Editing draft
                </span>
                <button
                  onClick={commitEdit}
                  className="rounded-md bg-amber-400 px-3 py-2 text-sm font-semibold text-slate-950 hover:bg-amber-300"
                >
                  Commit
                </button>
                <button
                  onClick={cancelEdit}
                  className="rounded-md border border-slate-700 px-3 py-2 text-sm text-slate-300 hover:border-slate-500 hover:text-slate-100"
                >
                  Cancel
                </button>
              </>
            ) : (
              <button
                onClick={beginEdit}
                className="rounded-md border border-slate-700 px-3 py-2 text-sm font-semibold text-slate-300 hover:border-cyan-500 hover:text-cyan-200"
              >
                수정 모드
              </button>
            )}
            <button
              onClick={validateWithCore}
              disabled={isValidating || isEditMode}
              className="rounded-md border border-slate-700 px-3 py-2 text-sm text-slate-300 hover:border-cyan-500 hover:text-cyan-200 disabled:cursor-not-allowed disabled:border-slate-800 disabled:text-slate-600"
            >
              {isValidating ? "Validating..." : "Validate"}
            </button>
            <button
              onClick={exportGeneratedRon}
              disabled={isExporting || isEditMode || hasValidationErrors}
              className="rounded-md bg-cyan-500 px-3 py-2 text-sm font-semibold text-slate-950 hover:bg-cyan-400 disabled:cursor-not-allowed disabled:bg-slate-700 disabled:text-slate-400"
            >
              {isExporting ? "Exporting..." : "Export RON"}
            </button>
          </div>
        </header>

        <section className="grid min-h-0 flex-1 overflow-hidden grid-cols-[280px_minmax(780px,1fr)_360px]">
          <aside className="flex min-h-0 flex-col border-r border-slate-800 bg-slate-950">
            <div className="shrink-0 border-b border-slate-800 p-4">
              <div className="flex items-center justify-between gap-3">
                <label className="text-xs font-medium uppercase tracking-wide text-slate-500">
                  Skills
                </label>
                <span className="text-xs text-slate-600">{activeSkills.length} total</span>
              </div>
              <input
                value={query}
                onChange={(event) => setQuery(event.currentTarget.value)}
                className="mt-2 w-full rounded-md border border-slate-700 bg-slate-900 px-3 py-2 text-sm text-slate-100 outline-none placeholder:text-slate-600 focus:border-cyan-500"
                placeholder="Search by ID or name"
              />
              <div className="mt-3 grid grid-cols-3 gap-2">
                <ToolButton disabled={!isEditMode} onClick={createSkill}>
                  New
                </ToolButton>
                <ToolButton disabled={!isEditMode} onClick={duplicateSelectedSkill}>
                  Copy
                </ToolButton>
                <ToolButton disabled={!isEditMode || activeSkills.length <= 1} tone="danger" onClick={deleteSelectedSkill}>
                  Delete
                </ToolButton>
              </div>
            </div>
            <div className="min-h-0 flex-1 overflow-y-auto p-3">
              <div className="space-y-2">
                {filteredSkills.map((skill) => (
                  <button
                    key={skill.id}
                    onClick={() => selectSkill(skill)}
                    className={`w-full rounded-md border p-3 text-left transition ${
                      skill.id === selectedSkill.id
                        ? "border-cyan-500 bg-cyan-500/10"
                        : "border-slate-800 bg-slate-900 hover:border-slate-600"
                    }`}
                  >
                    <div className="flex items-start justify-between gap-3">
                      <div>
                        <p className="text-sm font-semibold text-slate-100">{skill.name}</p>
                        <p className="mt-1 break-all text-xs text-slate-500">{skill.id}</p>
                      </div>
                      <span className="rounded bg-slate-800 px-2 py-1 text-[11px] text-slate-300">
                        {skill.steps.length}
                      </span>
                    </div>
                  </button>
                ))}
              </div>
            </div>
          </aside>

          <section className="flex min-h-0 flex-col overflow-hidden bg-slate-950">
            <div className="shrink-0 border-b border-slate-800 p-5">
              <div className="flex items-start justify-between gap-6">
                <div>
                  <p className="text-xs font-medium uppercase tracking-wide text-cyan-300">
                    {selectedSkill.kind}
                  </p>
                  <h2 className="mt-1 text-2xl font-semibold text-white">{selectedSkill.name}</h2>
                  <p className="mt-1 text-sm text-slate-400">{selectedSkill.id}</p>
                </div>
                <div className="grid grid-cols-3 gap-2 text-center">
                  <Metric label="Focus" value={`${selectedSkill.focusTimeMs}ms`} />
                  <Metric label="Steps" value={String(selectedSkill.steps.length)} />
                  <Metric
                    label="Move"
                    value={selectedSkill.focusPermissions.allowsMove ? "Allowed" : "Locked"}
                  />
                </div>
              </div>
            </div>

            <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden p-4">
              {exportStatus ? (
                <StatusBanner message={exportStatus} />
              ) : null}
              <div className="grid min-h-0 flex-1 grid-cols-[minmax(440px,1fr)_360px] gap-4 overflow-hidden">
                <Timeline
                  skill={selectedSkill}
                  selectedStepId={selectedStep.id}
                  isEditMode={isEditMode}
                  onSelectStep={setSelectedStepId}
                  onAddStep={addStep}
                  onDuplicateStep={duplicateSelectedStep}
                  onDeleteStep={deleteSelectedStep}
                />
                <StepPreviewPanel step={selectedStep} />
              </div>
            </div>

            <div className="grid h-56 shrink-0 grid-cols-[380px_minmax(0,1fr)] border-t border-slate-800">
              <ValidationPanel messages={validationMessages} />
              <RonPreview ron={ronPreview} />
            </div>
          </section>

          <Inspector
            skill={selectedSkill}
            step={selectedStep}
            selectedStepIndex={selectedStepIndex}
            isEditMode={isEditMode}
            onUpdateSkill={updateSelectedSkill}
            onRenameSkill={renameSelectedSkill}
            onUpdateStep={updateSelectedStep}
            onRenameStep={renameSelectedStep}
            onMoveSelectedStep={moveSelectedStep}
            onUpdateEffect={updateEffect}
            onAddEffect={addEffect}
            onDuplicateEffect={duplicateEffect}
            onDeleteEffect={deleteEffect}
            onReplaceEffect={replaceEffect}
            onMoveEffect={moveEffect}
          />
        </section>
      </div>
    </main>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-24 rounded-md border border-slate-800 bg-slate-900 px-3 py-2">
      <p className="text-[11px] uppercase tracking-wide text-slate-500">{label}</p>
      <p className="mt-1 text-sm font-semibold text-slate-100">{value}</p>
    </div>
  );
}

function Timeline({
  skill,
  selectedStepId,
  isEditMode,
  onSelectStep,
  onAddStep,
  onDuplicateStep,
  onDeleteStep,
}: {
  skill: SkillDraft;
  selectedStepId: string;
  isEditMode: boolean;
  onSelectStep: (stepId: string) => void;
  onAddStep: () => void;
  onDuplicateStep: () => void;
  onDeleteStep: () => void;
}) {
  const maxDelay = Math.max(1, ...skill.steps.map((step) => step.delayMs));

  return (
    <section className="flex min-h-0 flex-col overflow-hidden">
      <div className="mb-3 flex shrink-0 items-center justify-between gap-3">
        <div>
          <h3 className="text-sm font-semibold uppercase tracking-wide text-slate-300">
            Step Timeline
          </h3>
          <span className="text-xs text-slate-500">Focus window: {skill.focusTimeMs}ms</span>
        </div>
        <div className="flex items-center gap-2">
          <ToolButton disabled={!isEditMode} onClick={onAddStep}>
            Add Step
          </ToolButton>
          <ToolButton disabled={!isEditMode} onClick={onDuplicateStep}>
            Copy
          </ToolButton>
          <ToolButton disabled={!isEditMode || skill.steps.length <= 1} tone="danger" onClick={onDeleteStep}>
            Delete
          </ToolButton>
        </div>
      </div>

      <div className="relative min-h-0 flex-1 overflow-y-auto rounded-md border border-slate-800 bg-slate-900 p-3">
        <div className="absolute left-6 right-6 top-9 h-px bg-slate-700" />
        <div className="relative grid gap-4">
          {skill.steps.map((step) => {
            const offset = Math.round((step.delayMs / maxDelay) * 100);
            return (
              <button
                key={step.id}
                onClick={() => onSelectStep(step.id)}
                className={`grid grid-cols-[88px_minmax(0,1fr)] gap-4 rounded-md border p-3 text-left transition ${
                  selectedStepId === step.id
                    ? "border-cyan-500 bg-cyan-500/10"
                    : "border-slate-800 bg-slate-950 hover:border-slate-600"
                }`}
              >
                <div>
                  <div className="relative h-7">
                    <span
                      className="absolute top-0 h-3 w-3 rounded-full bg-cyan-400"
                      style={{ left: `${Math.min(offset, 86)}%` }}
                    />
                  </div>
                  <p className="text-base font-semibold text-cyan-200">{step.delayMs}ms</p>
                  <p className="text-xs text-slate-500">range {step.rangeTiles}</p>
                </div>
                <div className="min-w-0">
                  <div className="flex items-start justify-between gap-3">
                    <div>
                      <p className="font-semibold text-slate-100">{step.id}</p>
                      <p className="mt-1 text-sm text-slate-400">{describeTarget(step.target)}</p>
                    </div>
                    <span className="rounded bg-slate-800 px-2 py-1 text-xs text-slate-300">
                      {step.condition.type}
                    </span>
                  </div>
                  <div className="mt-2 flex flex-wrap gap-2">
                    <Chip tone="blue">{describeDelivery(step.delivery)}</Chip>
                    {step.effects.map((effect, index) => (
                      <Chip key={`${step.id}-effect-${index}`} tone="green">
                        {describeEffect(effect)}
                      </Chip>
                    ))}
                    {step.presentation.impactVfxId ? (
                      <Chip tone="purple">VFX {step.presentation.impactVfxId}</Chip>
                    ) : null}
                  </div>
                </div>
              </button>
            );
          })}
        </div>
      </div>
    </section>
  );
}

function ToolButton({
  children,
  disabled,
  tone = "default",
  onClick,
}: {
  children: ReactNode;
  disabled?: boolean;
  tone?: "default" | "danger";
  onClick: () => void;
}) {
  const toneClass =
    tone === "danger"
      ? "border-red-400/30 text-red-200 hover:border-red-300 hover:bg-red-400/10"
      : "border-slate-700 text-slate-300 hover:border-cyan-500 hover:bg-cyan-400/10 hover:text-cyan-100";

  return (
    <button
      type="button"
      disabled={disabled}
      onClick={onClick}
      className={`rounded-md border px-2.5 py-1.5 text-xs font-semibold transition disabled:cursor-not-allowed disabled:border-slate-800 disabled:bg-slate-900 disabled:text-slate-600 ${toneClass}`}
    >
      {children}
    </button>
  );
}

function Chip({ children, tone }: { children: ReactNode; tone: "blue" | "green" | "purple" }) {
  const toneClass = {
    blue: "border-blue-400/30 bg-blue-400/10 text-blue-200",
    green: "border-emerald-400/30 bg-emerald-400/10 text-emerald-200",
    purple: "border-fuchsia-400/30 bg-fuchsia-400/10 text-fuchsia-200",
  }[tone];

  return <span className={`rounded border px-2 py-1 text-xs ${toneClass}`}>{children}</span>;
}

function StepPreviewPanel({ step }: { step: SkillStepDraft }) {
  return (
    <div className="min-h-0 overflow-y-auto rounded-md border border-slate-800 bg-slate-950 p-3">
      <div className="space-y-3">
        <CastTargetPreview step={step} />
        <DeliveryShapePreview delivery={step.delivery} />
      </div>
    </div>
  );
}

function CastTargetPreview({ step }: { step: SkillStepDraft }) {
  const radius =
    step.target.type === "Allies" || step.target.type === "Enemies"
      ? step.target.area.type === "RadiusChebyshev"
        ? step.target.area.radiusTiles
        : 0
      : step.rangeTiles;
  const visibleRadius = Math.min(Math.max(radius, 2), 8);
  const gridSize = visibleRadius * 2 + 1;
  const cellSize = gridSize <= 7 ? 30 : gridSize <= 11 ? 24 : 18;
  const isClipped = radius > visibleRadius;

  const cells = Array.from({ length: gridSize * gridSize }, (_, index) => {
    const x = index % gridSize;
    const y = Math.floor(index / gridSize);
    const dx = Math.abs(x - visibleRadius);
    const dy = Math.abs(y - visibleRadius);
    const active = Math.max(dx, dy) <= Math.min(radius, visibleRadius);
    const center = x === visibleRadius && y === visibleRadius;

    return (
      <div
        key={index}
        style={{ width: cellSize, height: cellSize }}
        className={`flex items-center justify-center rounded border text-[11px] ${
          center
            ? "border-cyan-300 bg-cyan-300 text-slate-950"
            : active
              ? "border-cyan-500/40 bg-cyan-500/15 text-cyan-100"
              : "border-slate-800 bg-slate-950 text-slate-700"
        }`}
      >
        {center ? "C" : ""}
      </div>
    );
  });

  return (
    <section className="overflow-hidden rounded-md border border-slate-800 bg-slate-900 p-4">
      <div className="mb-3">
        <h3 className="text-sm font-semibold uppercase tracking-wide text-slate-300">
          Cast Target
        </h3>
        <div className="mt-1 flex items-center justify-between gap-2">
          <span className="truncate text-xs text-slate-500">{describeTarget(step.target)}</span>
          <span className="shrink-0 rounded bg-cyan-400/10 px-2 py-0.5 text-xs text-cyan-200">
            {radius} tiles
          </span>
        </div>
      </div>
      <div className="overflow-auto rounded-md border border-slate-800 bg-slate-950 p-2">
        <div
          className="grid w-fit gap-1"
          style={{ gridTemplateColumns: `repeat(${gridSize}, ${cellSize}px)` }}
        >
          {cells}
        </div>
      </div>
      {isClipped ? (
        <p className="mt-2 text-xs text-amber-200">
          Showing inner {visibleRadius} tiles. Full range is {radius} tiles.
        </p>
      ) : null}
    </section>
  );
}

function DeliveryShapePreview({ delivery }: { delivery: DeliveryDraft }) {
  if (delivery.type === "Instant") {
    return (
      <section className="rounded-md border border-slate-800 bg-slate-900 p-4">
        <h3 className="text-sm font-semibold uppercase tracking-wide text-slate-300">
          Delivery Shape
        </h3>
        <div className="mt-3 rounded-md border border-slate-800 bg-slate-950 p-4 text-sm text-slate-400">
          Instant delivery has no spatial shape.
        </div>
      </section>
    );
  }

  if (delivery.type === "Projectile") {
    const radiusTiles = unitsToTiles(delivery.radiusUnits ?? 0);
    const viewRadius = Math.max(3, radiusTiles + 1);
    const viewBox = `${-viewRadius} ${-viewRadius} ${viewRadius * 2} ${viewRadius * 2}`;
    const markerId = "projectile-preview-arrow";

    return (
      <section className="rounded-md border border-slate-800 bg-slate-900 p-4">
        <PreviewHeader title="Delivery Shape" detail={`Projectile / ${formatTiles(radiusTiles)} collision`} />
        <svg className="mt-3 h-56 w-full rounded-md border border-slate-800 bg-slate-950" viewBox={viewBox}>
          <PreviewGrid radius={viewRadius} />
          <line x1={-2} y1={0} x2={2} y2={0} stroke="#22d3ee" strokeWidth={0.08} markerEnd={`url(#${markerId})`} />
          <defs>
            <marker id={markerId} markerWidth="4" markerHeight="4" refX="4" refY="2" orient="auto">
              <path d="M0,0 L4,2 L0,4 Z" fill="#22d3ee" />
            </marker>
          </defs>
          <circle cx={-2} cy={0} r={0.12} fill="#e2e8f0" />
          <circle cx={2} cy={0} r={Math.max(radiusTiles, 0.05)} fill="#22d3ee22" stroke="#22d3ee" strokeWidth={0.05} />
          <text x={-2} y={-0.35} textAnchor="middle" fontSize="0.35" fill="#94a3b8">caster</text>
          <text x={2} y={-0.35} textAnchor="middle" fontSize="0.35" fill="#94a3b8">impact</text>
        </svg>
        <PreviewMeta>
          speed {delivery.speedUnitsPerMs.toLocaleString()} units/ms · hit {delivery.hitTargets ?? "Enemies"}
        </PreviewMeta>
      </section>
    );
  }

  const shape = delivery.shape ?? defaultAreaShape("Circle");
  const bounds = deliveryShapeBounds(shape);
  const viewRadius = Math.max(3, bounds + 1);
  const viewBox = `${-viewRadius} ${-viewRadius} ${viewRadius * 2} ${viewRadius * 2}`;

  return (
    <section className="rounded-md border border-slate-800 bg-slate-900 p-4">
      <PreviewHeader title="Delivery Shape" detail={`${describeAreaShapeTiles(shape)} / ${delivery.anchor}`} />
      <svg className="mt-3 h-56 w-full rounded-md border border-slate-800 bg-slate-950" viewBox={viewBox}>
        <PreviewGrid radius={viewRadius} />
        {renderDeliveryShape(shape)}
        <circle cx={0} cy={0} r={0.12} fill="#e2e8f0" />
        <text x={0} y={-0.35} textAnchor="middle" fontSize="0.35" fill="#94a3b8">anchor</text>
      </svg>
      <PreviewMeta>
        hit {delivery.hitTargets ?? "Enemies"} · {delivery.includeCaster ? "includes caster" : "excludes caster"}
      </PreviewMeta>
    </section>
  );
}

function PreviewHeader({ title, detail }: { title: string; detail: string }) {
  return (
    <div>
      <h3 className="text-sm font-semibold uppercase tracking-wide text-slate-300">{title}</h3>
      <p className="mt-1 truncate text-xs text-slate-500">{detail}</p>
    </div>
  );
}

function PreviewMeta({ children }: { children: ReactNode }) {
  return <p className="mt-2 text-xs text-slate-500">{children}</p>;
}

function PreviewGrid({ radius }: { radius: number }) {
  const lines = [];
  const gridRadius = Math.ceil(radius);

  for (let value = -gridRadius; value <= gridRadius; value += 1) {
    lines.push(
      <line key={`x-${value}`} x1={value} y1={-gridRadius} x2={value} y2={gridRadius} stroke="#1e293b" strokeWidth={0.02} />,
      <line key={`y-${value}`} x1={-gridRadius} y1={value} x2={gridRadius} y2={value} stroke="#1e293b" strokeWidth={0.02} />,
    );
  }

  return <g>{lines}</g>;
}

function renderDeliveryShape(shape: AreaShapeDraft) {
  switch (shape.type) {
    case "Circle": {
      const radius = unitsToTiles(shape.radiusUnits);
      return <circle cx={0} cy={0} r={radius} fill="#22d3ee22" stroke="#22d3ee" strokeWidth={0.05} />;
    }
    case "Box": {
      const width = unitsToTiles(shape.widthUnits);
      const height = unitsToTiles(shape.heightUnits);
      return <rect x={-width / 2} y={-height / 2} width={width} height={height} fill="#22d3ee22" stroke="#22d3ee" strokeWidth={0.05} />;
    }
    case "Line": {
      const length = unitsToTiles(shape.lengthUnits);
      return (
        <line
          x1={0}
          y1={0}
          x2={length}
          y2={0}
          stroke="#22d3ee"
          strokeLinecap="round"
          strokeWidth={DEFAULT_UNIT_HITBOX_RADIUS_TILES * 2}
          opacity={0.75}
        />
      );
    }
    case "Rectangle": {
      const width = unitsToTiles(shape.widthUnits);
      const length = unitsToTiles(shape.lengthUnits);
      return <rect x={0} y={-width / 2} width={length} height={width} fill="#22d3ee22" stroke="#22d3ee" strokeWidth={0.05} />;
    }
    case "Cone": {
      const length = unitsToTiles(shape.lengthUnits);
      const halfAngle = (shape.angleDegrees / 2) * (Math.PI / 180);
      const x = length * Math.cos(halfAngle);
      const y = length * Math.sin(halfAngle);
      const largeArc = shape.angleDegrees > 180 ? 1 : 0;
      const path = `M 0 0 L ${x} ${-y} A ${length} ${length} 0 ${largeArc} 1 ${x} ${y} Z`;
      return <path d={path} fill="#22d3ee22" stroke="#22d3ee" strokeWidth={0.05} />;
    }
  }
}

function deliveryShapeBounds(shape: AreaShapeDraft) {
  switch (shape.type) {
    case "Circle":
      return unitsToTiles(shape.radiusUnits);
    case "Box":
      return Math.max(unitsToTiles(shape.widthUnits), unitsToTiles(shape.heightUnits)) / 2;
    case "Line":
      return unitsToTiles(shape.lengthUnits);
    case "Rectangle":
      return Math.max(unitsToTiles(shape.lengthUnits), unitsToTiles(shape.widthUnits));
    case "Cone":
      return unitsToTiles(shape.lengthUnits);
  }
}

function describeAreaShapeTiles(shape: AreaShapeDraft) {
  switch (shape.type) {
    case "Circle":
      return `Circle ${formatTiles(unitsToTiles(shape.radiusUnits))}`;
    case "Line":
      return `Line ${formatTiles(unitsToTiles(shape.lengthUnits))}`;
    case "Box":
      return `Box ${formatTiles(unitsToTiles(shape.widthUnits))} x ${formatTiles(unitsToTiles(shape.heightUnits))}`;
    case "Rectangle":
      return `Rectangle ${formatTiles(unitsToTiles(shape.widthUnits))} x ${formatTiles(unitsToTiles(shape.lengthUnits))}`;
    case "Cone":
      return `Cone ${shape.angleDegrees}deg / ${formatTiles(unitsToTiles(shape.lengthUnits))}`;
  }
}

function Inspector({
  skill,
  step,
  selectedStepIndex,
  isEditMode,
  onUpdateSkill,
  onRenameSkill,
  onUpdateStep,
  onRenameStep,
  onMoveSelectedStep,
  onUpdateEffect,
  onAddEffect,
  onDuplicateEffect,
  onDeleteEffect,
  onReplaceEffect,
  onMoveEffect,
}: {
  skill: SkillDraft;
  step: SkillStepDraft;
  selectedStepIndex: number;
  isEditMode: boolean;
  onUpdateSkill: (patch: Partial<SkillDraft>) => void;
  onRenameSkill: (id: string) => void;
  onUpdateStep: (patch: Partial<SkillStepDraft>) => void;
  onRenameStep: (id: string) => void;
  onMoveSelectedStep: (direction: -1 | 1) => void;
  onUpdateEffect: (index: number, patch: Partial<SkillStepDraft["effects"][number]>) => void;
  onAddEffect: () => void;
  onDuplicateEffect: (index: number) => void;
  onDeleteEffect: (index: number) => void;
  onReplaceEffect: (index: number, effect: EffectDraft) => void;
  onMoveEffect: (index: number, direction: -1 | 1) => void;
}) {
  return (
    <aside className="min-h-0 overflow-y-auto border-l border-slate-800 bg-slate-950 p-5">
      <div className="flex items-center justify-between gap-3">
        <h3 className="text-sm font-semibold uppercase tracking-wide text-slate-300">Inspector</h3>
        <span
          className={`rounded px-2 py-1 text-xs ${
            isEditMode ? "bg-amber-400/10 text-amber-200" : "bg-slate-800 text-slate-400"
          }`}
        >
          {isEditMode ? "Editing enabled" : "Read only"}
        </span>
      </div>

      <div className="mt-4 space-y-4">
        <InspectorGroup title="Skill">
          <TextField
            label="Name"
            value={skill.name}
            disabled={!isEditMode}
            onChange={(name) => onUpdateSkill({ name })}
          />
          <TextField
            label="ID"
            value={skill.id}
            disabled={!isEditMode}
            onChange={onRenameSkill}
          />
          <SelectField
            label="Kind"
            value={skill.kind}
            options={["Targeted", "Untargeted"]}
            disabled={!isEditMode}
            onChange={(kind) => onUpdateSkill({ kind })}
          />
          <NumberField
            label="Focus"
            value={skill.focusTimeMs}
            suffix="ms"
            min={0}
            disabled={!isEditMode}
            onChange={(focusTimeMs) => onUpdateSkill({ focusTimeMs })}
          />
          <BooleanField
            label="Allow Move"
            checked={skill.focusPermissions.allowsMove}
            disabled={!isEditMode}
            onChange={(allowsMove) =>
              onUpdateSkill({
                focusPermissions: { ...skill.focusPermissions, allowsMove },
              })
            }
          />
          <BooleanField
            label="Allow Basic Attack"
            checked={skill.focusPermissions.allowsBasicAttack}
            disabled={!isEditMode}
            onChange={(allowsBasicAttack) =>
              onUpdateSkill({
                focusPermissions: { ...skill.focusPermissions, allowsBasicAttack },
              })
            }
          />
        </InspectorGroup>

        <InspectorGroup title="Selected Step">
          <div className="grid grid-cols-2 gap-2">
            <ToolButton
              disabled={!isEditMode || selectedStepIndex <= 0}
              onClick={() => onMoveSelectedStep(-1)}
            >
              Move Up
            </ToolButton>
            <ToolButton
              disabled={!isEditMode || selectedStepIndex < 0 || selectedStepIndex >= skill.steps.length - 1}
              onClick={() => onMoveSelectedStep(1)}
            >
              Move Down
            </ToolButton>
          </div>
          <TextField
            label="Step ID"
            value={step.id}
            disabled={!isEditMode}
            onChange={onRenameStep}
          />
          <NumberField
            label="Delay"
            value={step.delayMs}
            suffix="ms"
            min={0}
            disabled={!isEditMode}
            onChange={(delayMs) => onUpdateStep({ delayMs })}
          />
          <NumberField
            label="Range"
            value={step.rangeTiles}
            suffix="tiles"
            min={0}
            disabled={!isEditMode}
            onChange={(rangeTiles) => onUpdateStep({ rangeTiles })}
          />
        </InspectorGroup>

        <InspectorGroup title="Target">
          <TargetFields
            target={step.target}
            disabled={!isEditMode}
            onChange={(target) => onUpdateStep({ target })}
          />
        </InspectorGroup>

        <InspectorGroup title="Delivery">
          <DeliveryFields
            delivery={step.delivery}
            disabled={!isEditMode}
            onChange={(delivery) => onUpdateStep({ delivery })}
          />
        </InspectorGroup>

        <InspectorGroup title="Condition / Repeat">
          <ConditionFields
            condition={step.condition}
            disabled={!isEditMode}
            onChange={(condition) => onUpdateStep({ condition })}
          />
          <RepeatFields
            repeat={step.repeat}
            disabled={!isEditMode}
            onChange={(repeat) => onUpdateStep({ repeat })}
          />
        </InspectorGroup>

        <InspectorGroup title="Effects">
          <div className="space-y-2">
            {step.effects.map((effect, index) => (
              <div key={index} className="rounded-md border border-slate-800 bg-slate-900 p-3">
                <div className="flex items-start justify-between gap-3">
                  <p className="min-w-0 text-sm text-slate-200">{describeEffect(effect)}</p>
                  <div className="flex shrink-0 items-center gap-1">
                    <ToolButton disabled={!isEditMode || index <= 0} onClick={() => onMoveEffect(index, -1)}>
                      Up
                    </ToolButton>
                    <ToolButton
                      disabled={!isEditMode || index >= step.effects.length - 1}
                      onClick={() => onMoveEffect(index, 1)}
                    >
                      Down
                    </ToolButton>
                    <ToolButton disabled={!isEditMode} onClick={() => onDuplicateEffect(index)}>
                      Copy
                    </ToolButton>
                    <ToolButton disabled={!isEditMode} tone="danger" onClick={() => onDeleteEffect(index)}>
                      Delete
                    </ToolButton>
                  </div>
                </div>
                <EditableEffectFields
                  effect={effect}
                  index={index}
                  disabled={!isEditMode}
                  onUpdateEffect={onUpdateEffect}
                  onReplaceEffect={onReplaceEffect}
                />
              </div>
            ))}
            <ToolButton disabled={!isEditMode} onClick={onAddEffect}>
              Add Effect
            </ToolButton>
          </div>
        </InspectorGroup>

        <InspectorGroup title="Presentation">
          <TextField
            label="Cast State"
            value={step.presentation.castState ?? ""}
            placeholder="None"
            disabled={!isEditMode}
            onChange={(castState) =>
              onUpdateStep({ presentation: { ...step.presentation, castState: optionalText(castState) } })
            }
          />
          <TextField
            label="Projectile VFX"
            value={step.presentation.projectileVfxId ?? ""}
            placeholder="None"
            disabled={!isEditMode}
            onChange={(projectileVfxId) =>
              onUpdateStep({
                presentation: { ...step.presentation, projectileVfxId: optionalText(projectileVfxId) },
              })
            }
          />
          <TextField
            label="Impact VFX"
            value={step.presentation.impactVfxId ?? ""}
            placeholder="None"
            disabled={!isEditMode}
            onChange={(impactVfxId) =>
              onUpdateStep({ presentation: { ...step.presentation, impactVfxId: optionalText(impactVfxId) } })
            }
          />
          <TextField
            label="Target Anchor"
            value={step.presentation.targetAnchor ?? ""}
            placeholder="None"
            disabled={!isEditMode}
            onChange={(targetAnchor) =>
              onUpdateStep({ presentation: { ...step.presentation, targetAnchor: optionalText(targetAnchor) } })
            }
          />
        </InspectorGroup>
      </div>
    </aside>
  );
}

function TargetFields({
  target,
  disabled,
  onChange,
}: {
  target: TargetDraft;
  disabled: boolean;
  onChange: (target: TargetDraft) => void;
}) {
  return (
    <div className="space-y-3">
      <SelectField
        label="Target Type"
        value={target.type}
        options={["SelfUnit", "EnemySingle", "Allies", "Enemies"]}
        disabled={disabled}
        onChange={(type) => onChange(defaultTarget(type))}
      />
      {target.type === "EnemySingle" ? (
        <SelectField
          label="Enemy Rule"
          value={target.rule}
          options={["Nearest", "CurrentTarget", "LowestHealthEnemy"]}
          disabled={disabled}
          onChange={(rule) => onChange({ ...target, rule })}
        />
      ) : null}
      {target.type === "Allies" || target.type === "Enemies" ? (
        <AreaFields
          area={target.area}
          disabled={disabled}
          onChange={(area) => onChange({ ...target, area })}
        />
      ) : null}
    </div>
  );
}

function AreaFields({
  area,
  disabled,
  onChange,
}: {
  area: AreaDraft;
  disabled: boolean;
  onChange: (area: AreaDraft) => void;
}) {
  return (
    <div className="space-y-3 rounded-md border border-slate-800 bg-slate-950 p-3">
      <SelectField
        label="Area Type"
        value={area.type}
        options={["All", "RadiusChebyshev", "Line"]}
        disabled={disabled}
        onChange={(type) => onChange(defaultArea(type))}
      />
      {area.type === "RadiusChebyshev" ? (
        <NumberField
          label="Radius"
          value={area.radiusTiles}
          suffix="tiles"
          min={0}
          disabled={disabled}
          onChange={(radiusTiles) => onChange({ ...area, radiusTiles })}
        />
      ) : null}
      {area.type === "Line" ? (
        <NumberField
          label="Length"
          value={area.lengthTiles}
          suffix="tiles"
          min={0}
          disabled={disabled}
          onChange={(lengthTiles) => onChange({ ...area, lengthTiles })}
        />
      ) : null}
    </div>
  );
}

function DeliveryFields({
  delivery,
  disabled,
  onChange,
}: {
  delivery: DeliveryDraft;
  disabled: boolean;
  onChange: (delivery: DeliveryDraft) => void;
}) {
  return (
    <div className="space-y-3">
      <SelectField
        label="Delivery Type"
        value={delivery.type}
        options={["Instant", "Projectile", "Area"]}
        disabled={disabled}
        onChange={(type) => onChange(defaultDelivery(type))}
      />
      {delivery.type === "Projectile" ? (
        <div className="space-y-3 rounded-md border border-slate-800 bg-slate-950 p-3">
          <NumberField
            label="Speed"
            value={delivery.speedUnitsPerMs}
            suffix="units/ms"
            min={1}
            disabled={disabled}
            onChange={(speedUnitsPerMs) => onChange({ ...delivery, speedUnitsPerMs })}
          />
          <NumberField
            label="Collision Radius"
            value={unitsToTiles(delivery.radiusUnits ?? 0)}
            suffix="tiles"
            min={0}
            disabled={disabled}
            step={0.125}
            onChange={(radiusTiles) => onChange({ ...delivery, radiusUnits: tilesToUnits(radiusTiles) })}
          />
          <SelectField
            label="Hit Targets"
            value={delivery.hitTargets ?? "Enemies"}
            options={["Enemies", "Allies", "Any"]}
            disabled={disabled}
            onChange={(hitTargets) => onChange({ ...delivery, hitTargets })}
          />
          <BooleanField
            label="Piercing"
            checked={Boolean(delivery.piercing)}
            disabled={disabled}
            onChange={(piercing) => onChange({ ...delivery, piercing })}
          />
          <BooleanField
            label="Despawn On Hit"
            checked={delivery.despawnOnHit ?? true}
            disabled={disabled}
            onChange={(despawnOnHit) => onChange({ ...delivery, despawnOnHit })}
          />
          <OptionalNumberField
            label="Max Hits"
            value={delivery.maxHits ?? undefined}
            min={1}
            disabled={disabled}
            onChange={(maxHits) => onChange({ ...delivery, maxHits })}
          />
        </div>
      ) : null}
      {delivery.type === "Area" ? (
        <div className="space-y-3 rounded-md border border-slate-800 bg-slate-950 p-3">
          <AreaShapeFields
            shape={delivery.shape ?? defaultAreaShape("Circle")}
            disabled={disabled}
            onChange={(shape) => onChange({ ...delivery, shape })}
          />
          <SelectField
            label="Anchor"
            value={delivery.anchor}
            options={["CastTarget", "ImpactContext", "Caster", "CastTargetStart", "ImpactContextStart"]}
            disabled={disabled}
            onChange={(anchor) => onChange({ ...delivery, anchor })}
          />
          <SelectField
            label="Hit Targets"
            value={delivery.hitTargets ?? "Enemies"}
            options={["Enemies", "Allies", "Any"]}
            disabled={disabled}
            onChange={(hitTargets) => onChange({ ...delivery, hitTargets })}
          />
          <BooleanField
            label="Include Caster"
            checked={Boolean(delivery.includeCaster)}
            disabled={disabled}
            onChange={(includeCaster) => onChange({ ...delivery, includeCaster })}
          />
          <SelectField
            label="Tick Policy"
            value={delivery.tickPolicy ?? "OncePerArea"}
            options={["OncePerArea", "EveryTick", "OnEnter"]}
            disabled={disabled}
            onChange={(tickPolicy) => onChange({ ...delivery, tickPolicy })}
          />
          <NumberField
            label="Duration"
            value={delivery.durationMs ?? 0}
            suffix="ms"
            min={0}
            disabled={disabled}
            onChange={(durationMs) => onChange({ ...delivery, durationMs })}
          />
          <OptionalNumberField
            label="Tick Interval"
            value={delivery.tickIntervalMs ?? undefined}
            suffix="ms"
            min={1}
            disabled={disabled}
            onChange={(tickIntervalMs) => onChange({ ...delivery, tickIntervalMs })}
          />
        </div>
      ) : null}
    </div>
  );
}

function AreaShapeFields({
  shape,
  disabled,
  onChange,
}: {
  shape: AreaShapeDraft;
  disabled: boolean;
  onChange: (shape: AreaShapeDraft) => void;
}) {
  return (
    <div className="space-y-3">
      <SelectField
        label="Shape"
        value={shape.type}
        options={["Circle", "Line", "Box", "Rectangle", "Cone"]}
        disabled={disabled}
        onChange={(type) => onChange(defaultAreaShape(type))}
      />
      {shape.type === "Circle" ? (
        <NumberField
          label="Radius"
          value={unitsToTiles(shape.radiusUnits)}
          suffix="tiles"
          min={0}
          disabled={disabled}
          step={0.25}
          onChange={(radiusTiles) => onChange({ ...shape, radiusUnits: tilesToUnits(radiusTiles) })}
        />
      ) : null}
      {shape.type === "Line" ? (
        <NumberField
          label="Length"
          value={unitsToTiles(shape.lengthUnits)}
          suffix="tiles"
          min={0}
          disabled={disabled}
          step={0.25}
          onChange={(lengthTiles) => onChange({ ...shape, lengthUnits: tilesToUnits(lengthTiles) })}
        />
      ) : null}
      {shape.type === "Box" ? (
        <>
          <NumberField
            label="Width"
            value={unitsToTiles(shape.widthUnits)}
            suffix="tiles"
            min={0}
            disabled={disabled}
            step={0.25}
            onChange={(widthTiles) => onChange({ ...shape, widthUnits: tilesToUnits(widthTiles) })}
          />
          <NumberField
            label="Height"
            value={unitsToTiles(shape.heightUnits)}
            suffix="tiles"
            min={0}
            disabled={disabled}
            step={0.25}
            onChange={(heightTiles) => onChange({ ...shape, heightUnits: tilesToUnits(heightTiles) })}
          />
        </>
      ) : null}
      {shape.type === "Rectangle" ? (
        <>
          <NumberField
            label="Width"
            value={unitsToTiles(shape.widthUnits)}
            suffix="tiles"
            min={0}
            disabled={disabled}
            step={0.25}
            onChange={(widthTiles) => onChange({ ...shape, widthUnits: tilesToUnits(widthTiles) })}
          />
          <NumberField
            label="Length"
            value={unitsToTiles(shape.lengthUnits)}
            suffix="tiles"
            min={0}
            disabled={disabled}
            step={0.25}
            onChange={(lengthTiles) => onChange({ ...shape, lengthUnits: tilesToUnits(lengthTiles) })}
          />
        </>
      ) : null}
      {shape.type === "Cone" ? (
        <>
          <NumberField
            label="Angle"
            value={shape.angleDegrees}
            suffix="deg"
            min={1}
            disabled={disabled}
            onChange={(angleDegrees) => onChange({ ...shape, angleDegrees })}
          />
          <NumberField
            label="Length"
            value={unitsToTiles(shape.lengthUnits)}
            suffix="tiles"
            min={0}
            disabled={disabled}
            step={0.25}
            onChange={(lengthTiles) => onChange({ ...shape, lengthUnits: tilesToUnits(lengthTiles) })}
          />
        </>
      ) : null}
    </div>
  );
}

function ConditionFields({
  condition,
  disabled,
  onChange,
}: {
  condition: StepConditionDraft;
  disabled: boolean;
  onChange: (condition: StepConditionDraft) => void;
}) {
  return (
    <div className="space-y-3">
      <SelectField
        label="Condition"
        value={condition.type}
        options={["Always", "IfPreviousStepDealtDamage", "IfCasterHasBuff"]}
        disabled={disabled}
        onChange={(type) => onChange(defaultCondition(type))}
      />
      {condition.type === "IfCasterHasBuff" ? (
        <div className="space-y-3 rounded-md border border-slate-800 bg-slate-950 p-3">
          <TextField
            label="Buff ID"
            value={condition.buffId}
            disabled={disabled}
            onChange={(buffId) => onChange({ ...condition, buffId })}
          />
          <NumberField
            label="Min Stacks"
            value={condition.minStacks}
            min={1}
            disabled={disabled}
            onChange={(minStacks) => onChange({ ...condition, minStacks })}
          />
        </div>
      ) : null}
    </div>
  );
}

function RepeatFields({
  repeat,
  disabled,
  onChange,
}: {
  repeat: StepRepeatDraft;
  disabled: boolean;
  onChange: (repeat: StepRepeatDraft) => void;
}) {
  return (
    <div className="space-y-3 border-t border-slate-800 pt-3">
      <SelectField
        label="Repeat"
        value={repeat.type}
        options={["None", "Times", "ByBuffStacks"]}
        disabled={disabled}
        onChange={(type) => onChange(defaultRepeat(type))}
      />
      {repeat.type === "Times" ? (
        <NumberField
          label="Count"
          value={repeat.count}
          min={1}
          disabled={disabled}
          onChange={(count) => onChange({ ...repeat, count })}
        />
      ) : null}
      {repeat.type === "ByBuffStacks" ? (
        <div className="space-y-3 rounded-md border border-slate-800 bg-slate-950 p-3">
          <SelectField
            label="Unit"
            value={repeat.unit ?? "StepTarget"}
            options={["StepTarget", "SelfUnit"]}
            disabled={disabled}
            onChange={(unit) => onChange({ ...repeat, unit })}
          />
          <TextField
            label="Buff ID"
            value={repeat.buffId}
            disabled={disabled}
            onChange={(buffId) => onChange({ ...repeat, buffId })}
          />
          <OptionalNumberField
            label="Max"
            value={repeat.max}
            min={1}
            disabled={disabled}
            onChange={(max) => onChange({ ...repeat, max })}
          />
        </div>
      ) : null}
    </div>
  );
}

function EditableEffectFields({
  effect,
  index,
  disabled,
  onUpdateEffect,
  onReplaceEffect,
}: {
  effect: SkillStepDraft["effects"][number];
  index: number;
  disabled: boolean;
  onUpdateEffect: (index: number, patch: Partial<SkillStepDraft["effects"][number]>) => void;
  onReplaceEffect: (index: number, effect: EffectDraft) => void;
}) {
  const typeSelector = (
    <SelectField
      label="Effect Type"
      value={effect.type}
      options={["Damage", "ModifyDamage", "Heal", "ModifyResonance", "ModifyStats", "ApplyBuff", "ExtraAttack"]}
      disabled={disabled}
      onChange={(type) => onReplaceEffect(index, defaultEffect(type))}
    />
  );

  switch (effect.type) {
    case "Damage":
      return (
        <div className="mt-3 space-y-3">
          {typeSelector}
          <SelectField
            label="Damage Type"
            value={effect.damageType}
            options={["Physical", "Magic", "True"]}
            disabled={disabled}
            onChange={(damageType) => onUpdateEffect(index, { damageType })}
          />
          <NumberField
            label="Amount"
            value={effect.amount}
            disabled={disabled}
            onChange={(amount) => onUpdateEffect(index, { amount })}
          />
        </div>
      );
    case "ModifyDamage":
      return (
        <div className="mt-3 space-y-3">
          {typeSelector}
          {damageModifierFields.map(({ key, label, suffix }) => (
            <NumberField
              key={key}
              label={label}
              value={effect.modifiers[key] ?? 0}
              suffix={suffix}
              disabled={disabled}
              onChange={(value) =>
                onUpdateEffect(index, {
                  modifiers: { ...effect.modifiers, [key]: value },
                })
              }
            />
          ))}
        </div>
      );
    case "Heal":
    case "ModifyResonance":
      return (
        <div className="mt-3 space-y-3">
          {typeSelector}
          <NumberField
            label="Amount"
            value={effect.amount}
            disabled={disabled}
            onChange={(amount) => onUpdateEffect(index, { amount })}
          />
        </div>
      );
    case "ApplyBuff":
      return (
        <div className="mt-3 space-y-3">
          {typeSelector}
          <TextField
            label="Buff ID"
            value={effect.buffId}
            disabled={disabled}
            onChange={(buffId) => onUpdateEffect(index, { buffId })}
          />
          <NumberField
            label="Duration"
            value={effect.durationMs}
            suffix="ms"
            min={0}
            disabled={disabled}
            onChange={(durationMs) => onUpdateEffect(index, { durationMs })}
          />
        </div>
      );
    case "ExtraAttack":
      return (
        <div className="mt-3 space-y-3">
          {typeSelector}
          <NumberField
            label="Count"
            value={effect.count}
            min={0}
            disabled={disabled}
            onChange={(count) => onUpdateEffect(index, { count })}
          />
        </div>
      );
    case "ModifyStats":
      return (
        <div className="mt-3 space-y-3">
          {typeSelector}
          <SelectField
            label="Stat"
            value={effect.modifier.stat}
            options={["MaxHealth", "Attack", "Defense", "MagicResist", "AttackIntervalMs", "MoveSpeedUnitsPerMs"]}
            disabled={disabled}
            onChange={(stat) =>
              onUpdateEffect(index, {
                modifier: { ...effect.modifier, stat },
              })
            }
          />
          <SelectField
            label="Kind"
            value={effect.modifier.kind}
            options={["Flat", "Percent"]}
            disabled={disabled}
            onChange={(kind) =>
              onUpdateEffect(index, {
                modifier: { ...effect.modifier, kind },
              })
            }
          />
          <NumberField
            label="Value"
            value={effect.modifier.value}
            disabled={disabled}
            onChange={(value) =>
              onUpdateEffect(index, {
                modifier: { ...effect.modifier, value },
              })
            }
          />
        </div>
      );
  }
}

const damageModifierFields: Array<{
  key: keyof DamageModifiersDraft;
  label: string;
  suffix?: string;
}> = [
  { key: "armorPenetrationFlat", label: "Armor Penetration" },
  { key: "magicResistPenetrationFlat", label: "Magic Resist Penetration" },
  { key: "armorPenetrationPercent", label: "Armor Penetration", suffix: "%" },
  { key: "magicResistPenetrationPercent", label: "Magic Resist Penetration", suffix: "%" },
  { key: "damageAmpPercent", label: "Damage Amp", suffix: "%" },
  { key: "damageReductionPercent", label: "Damage Reduction", suffix: "%" },
  { key: "physicalDamageAmpPercent", label: "Physical Amp", suffix: "%" },
  { key: "magicDamageAmpPercent", label: "Magic Amp", suffix: "%" },
  { key: "trueDamageAmpPercent", label: "True Amp", suffix: "%" },
  { key: "physicalDamageReductionPercent", label: "Physical Reduction", suffix: "%" },
  { key: "magicDamageReductionPercent", label: "Magic Reduction", suffix: "%" },
  { key: "trueDamageReductionPercent", label: "True Reduction", suffix: "%" },
  { key: "critChancePercent", label: "Crit Chance", suffix: "%" },
  { key: "critDamagePercent", label: "Crit Damage", suffix: "%" },
];

function InspectorGroup({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="rounded-md border border-slate-800 bg-slate-900 p-4">
      <h4 className="text-xs font-semibold uppercase tracking-wide text-slate-500">{title}</h4>
      <div className="mt-3 space-y-3">{children}</div>
    </section>
  );
}

function TextField({
  label,
  value,
  placeholder,
  disabled,
  onChange,
}: {
  label: string;
  value: string;
  placeholder?: string;
  disabled?: boolean;
  onChange: (value: string) => void;
}) {
  return (
    <label className="block">
      <span className="text-[11px] uppercase tracking-wide text-slate-500">{label}</span>
      <input
        value={value}
        placeholder={placeholder}
        disabled={disabled}
        onChange={(event) => onChange(event.currentTarget.value)}
        className="mt-1 w-full rounded-md border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 outline-none placeholder:text-slate-600 focus:border-cyan-500 disabled:cursor-not-allowed disabled:border-slate-800 disabled:bg-slate-900 disabled:text-slate-500"
      />
    </label>
  );
}

function NumberField({
  label,
  value,
  suffix,
  min,
  step,
  disabled,
  onChange,
}: {
  label: string;
  value: number;
  suffix?: string;
  min?: number;
  step?: number;
  disabled?: boolean;
  onChange: (value: number) => void;
}) {
  return (
    <label className="block">
      <span className="text-[11px] uppercase tracking-wide text-slate-500">{label}</span>
      <div className="mt-1 flex items-center gap-2">
        <input
          type="number"
          value={value}
          min={min}
          step={step}
          disabled={disabled}
          onChange={(event) => onChange(Number(event.currentTarget.value))}
          className="w-full rounded-md border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 outline-none focus:border-cyan-500 disabled:cursor-not-allowed disabled:border-slate-800 disabled:bg-slate-900 disabled:text-slate-500"
        />
        {suffix ? <span className="shrink-0 text-xs text-slate-500">{suffix}</span> : null}
      </div>
    </label>
  );
}

function SelectField({
  label,
  value,
  options,
  disabled,
  onChange,
}: {
  label: string;
  value: string;
  options: string[];
  disabled?: boolean;
  onChange: (value: string) => void;
}) {
  return (
    <label className="block">
      <span className="text-[11px] uppercase tracking-wide text-slate-500">{label}</span>
      <select
        value={value}
        disabled={disabled}
        onChange={(event) => onChange(event.currentTarget.value)}
        className="mt-1 w-full rounded-md border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 outline-none focus:border-cyan-500 disabled:cursor-not-allowed disabled:border-slate-800 disabled:bg-slate-900 disabled:text-slate-500"
      >
        {options.map((option) => (
          <option key={option} value={option}>
            {option}
          </option>
        ))}
      </select>
    </label>
  );
}

function BooleanField({
  label,
  checked,
  disabled,
  onChange,
}: {
  label: string;
  checked: boolean;
  disabled?: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="flex items-center justify-between gap-3 rounded-md border border-slate-800 bg-slate-950 px-3 py-2">
      <span className="text-[11px] uppercase tracking-wide text-slate-500">{label}</span>
      <input
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.currentTarget.checked)}
        className="h-4 w-4 accent-cyan-400 disabled:cursor-not-allowed"
      />
    </label>
  );
}

function OptionalNumberField({
  label,
  value,
  suffix,
  min,
  disabled,
  onChange,
}: {
  label: string;
  value?: number;
  suffix?: string;
  min?: number;
  disabled?: boolean;
  onChange: (value: number | undefined) => void;
}) {
  return (
    <label className="block">
      <span className="text-[11px] uppercase tracking-wide text-slate-500">{label}</span>
      <div className="mt-1 flex items-center gap-2">
        <input
          type="number"
          value={value ?? ""}
          min={min}
          disabled={disabled}
          placeholder="None"
          onChange={(event) => {
            const nextValue = event.currentTarget.value;
            onChange(nextValue === "" ? undefined : Number(nextValue));
          }}
          className="w-full rounded-md border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 outline-none placeholder:text-slate-600 focus:border-cyan-500 disabled:cursor-not-allowed disabled:border-slate-800 disabled:bg-slate-900 disabled:text-slate-500"
        />
        {suffix ? <span className="shrink-0 text-xs text-slate-500">{suffix}</span> : null}
      </div>
    </label>
  );
}

function optionalText(value: string) {
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : undefined;
}

function defaultTarget(type: string): TargetDraft {
  switch (type) {
    case "SelfUnit":
      return { type: "SelfUnit" };
    case "Allies":
      return { type: "Allies", area: { type: "RadiusChebyshev", radiusTiles: 2 } };
    case "Enemies":
      return { type: "Enemies", area: { type: "RadiusChebyshev", radiusTiles: 2 } };
    case "EnemySingle":
    default:
      return { type: "EnemySingle", rule: "Nearest" };
  }
}

function defaultArea(type: string): AreaDraft {
  switch (type) {
    case "All":
      return { type: "All" };
    case "Line":
      return { type: "Line", lengthTiles: 3 };
    case "RadiusChebyshev":
    default:
      return { type: "RadiusChebyshev", radiusTiles: 2 };
  }
}

function defaultDelivery(type: string): DeliveryDraft {
  switch (type) {
    case "Projectile":
      return {
        type: "Projectile",
        speedUnitsPerMs: 5,
        radiusUnits: tilesToUnits(0.125),
        hitTargets: "Enemies",
        piercing: false,
        despawnOnHit: true,
        maxHits: undefined,
      };
    case "Area":
      return {
        type: "Area",
        shape: { type: "Circle", radiusUnits: tilesToUnits(2) },
        anchor: "CastTarget",
        hitTargets: "Enemies",
        includeCaster: false,
        tickPolicy: "OncePerArea",
        durationMs: 0,
        tickIntervalMs: undefined,
      };
    case "Instant":
    default:
      return { type: "Instant" };
  }
}

function defaultAreaShape(type: string): AreaShapeDraft {
  switch (type) {
    case "Line":
      return { type: "Line", lengthUnits: tilesToUnits(4) };
    case "Box":
      return { type: "Box", widthUnits: tilesToUnits(2), heightUnits: tilesToUnits(2) };
    case "Rectangle":
      return { type: "Rectangle", widthUnits: tilesToUnits(2), lengthUnits: tilesToUnits(4) };
    case "Cone":
      return { type: "Cone", angleDegrees: 60, lengthUnits: tilesToUnits(4) };
    case "Circle":
    default:
      return { type: "Circle", radiusUnits: tilesToUnits(2) };
  }
}

function defaultCondition(type: string): StepConditionDraft {
  switch (type) {
    case "IfPreviousStepDealtDamage":
      return { type: "IfPreviousStepDealtDamage" };
    case "IfCasterHasBuff":
      return { type: "IfCasterHasBuff", buffId: "", minStacks: 1 };
    case "Always":
    default:
      return { type: "Always" };
  }
}

function defaultRepeat(type: string): StepRepeatDraft {
  switch (type) {
    case "Times":
      return { type: "Times", count: 2 };
    case "ByBuffStacks":
      return { type: "ByBuffStacks", unit: "StepTarget", buffId: "", max: undefined };
    case "None":
    default:
      return { type: "None" };
  }
}

function defaultEffect(type: string): EffectDraft {
  switch (type) {
    case "Heal":
      return { type: "Heal", amount: 10 };
    case "ModifyResonance":
      return { type: "ModifyResonance", amount: 10 };
    case "ModifyStats":
      return {
        type: "ModifyStats",
        modifier: {
          stat: "Attack",
          kind: "Percent",
          value: 10,
        },
      };
    case "ApplyBuff":
      return { type: "ApplyBuff", buffId: "", durationMs: 3000 };
    case "ExtraAttack":
      return { type: "ExtraAttack", count: 1 };
    case "ModifyDamage":
      return { type: "ModifyDamage", modifiers: { damageAmpPercent: 20 } };
    case "Damage":
    default:
      return { type: "Damage", amount: 10, damageType: "Magic" };
  }
}

function createDefaultStep(id: string): SkillStepDraft {
  return {
    id,
    delayMs: 0,
    rangeTiles: 2,
    condition: { type: "Always" },
    repeat: { type: "None" },
    target: { type: "EnemySingle", rule: "Nearest" },
    delivery: { type: "Instant" },
    effects: [defaultEffect("Damage")],
    presentation: {},
  };
}

function uniqueId(base: string, usedIds: string[]) {
  const used = new Set(usedIds);
  const normalizedBase = toIdentifier(base);

  if (!used.has(normalizedBase)) {
    return normalizedBase;
  }

  for (let index = 2; index < 10_000; index += 1) {
    const candidate = `${normalizedBase}_${index}`;
    if (!used.has(candidate)) {
      return candidate;
    }
  }

  return `${normalizedBase}_${Date.now()}`;
}

function toIdentifier(value: string) {
  const identifier = value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9_]+/g, "_")
    .replace(/^_+|_+$/g, "")
    .replace(/_{2,}/g, "_");

  return identifier || "new_skill";
}

function cloneDraft<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

function moveArrayItem<T>(items: T[], fromIndex: number, toIndex: number): T[] {
  const nextItems = [...items];
  const [item] = nextItems.splice(fromIndex, 1);
  nextItems.splice(toIndex, 0, item);
  return nextItems;
}

function unitsToTiles(units: number) {
  if (!Number.isFinite(units)) {
    return 0;
  }

  return Number((units / TILE_UNITS_PER_TILE).toFixed(3));
}

function tilesToUnits(tiles: number) {
  if (!Number.isFinite(tiles)) {
    return 0;
  }

  return Math.round(tiles * TILE_UNITS_PER_TILE);
}

function formatTiles(tiles: number) {
  if (!Number.isFinite(tiles)) {
    return "0 tiles";
  }

  return `${Number(tiles.toFixed(2)).toLocaleString()} tiles`;
}

async function runCoreValidation(skills: SkillDraft[]) {
  return invoke<{ skillCount: number }>("validate_skills", { skills });
}

function firstBlockingValidationMessage(messages: ValidationMessage[]) {
  return messages.find((message) => message.severity === "error");
}

function formatCoreError(error: unknown) {
  const message = String(error);
  const replacements: Array<[RegExp, string]> = [
    [/unsupported skill kind '([^']+)'/, "Unsupported skill kind '$1'. Use Targeted or Untargeted."],
    [/unsupported enemy target rule '([^']+)'/, "Unsupported enemy target rule '$1'."],
    [/unsupported repeat unit reference '([^']+)'/, "Unsupported repeat unit '$1'. Use SelfUnit or StepTarget."],
    [/unsupported hit target filter '([^']+)'/, "Unsupported hit target filter '$1'. Use Allies, Enemies, or Any."],
    [/unsupported area anchor '([^']+)'/, "Unsupported area anchor '$1'."],
    [/unsupported area tick policy '([^']+)'/, "Unsupported area tick policy '$1'."],
    [/unsupported stat id '([^']+)'/, "Unsupported stat '$1'."],
    [/unsupported stat modifier kind '([^']+)'/, "Unsupported stat modifier kind '$1'."],
  ];

  for (const [pattern, replacement] of replacements) {
    if (pattern.test(message)) {
      return message.replace(pattern, replacement);
    }
  }

  return message;
}

function validateSkillCollection(skills: SkillDraft[]): ValidationMessage[] {
  const messages: ValidationMessage[] = [];
  const seenIds = new Map<string, number>();

  for (const skill of skills) {
    seenIds.set(skill.id, (seenIds.get(skill.id) ?? 0) + 1);
  }

  for (const [id, count] of seenIds) {
    if (count > 1) {
      messages.push({
        severity: "error",
        label: "Duplicate skill ID",
        detail: `Skill ID '${id}' appears ${count} times in the database.`,
      });
    }
  }

  return messages;
}

function ValidationPanel({ messages }: { messages: ValidationMessage[] }) {
  return (
    <section className="min-h-0 overflow-y-auto border-r border-slate-800 bg-slate-950 p-4">
      <h3 className="text-sm font-semibold uppercase tracking-wide text-slate-300">Validation</h3>
      <div className="mt-3 space-y-2">
        {messages.map((message, index) => (
          <MessageCard key={index} message={message} />
        ))}
      </div>
    </section>
  );
}

function StatusBanner({ message }: { message: ValidationMessage }) {
  return (
    <div className="shrink-0">
      <MessageCard message={message} />
    </div>
  );
}

function MessageCard({ message }: { message: ValidationMessage }) {
  return (
    <div
      className={`rounded-md border p-3 ${
        message.severity === "error"
          ? "border-red-400/40 bg-red-400/10"
          : message.severity === "warning"
            ? "border-amber-400/40 bg-amber-400/10"
            : "border-emerald-400/40 bg-emerald-400/10"
      }`}
    >
      <p className="text-sm font-semibold text-slate-100">{message.label}</p>
      <p className="mt-1 text-xs text-slate-400">{message.detail}</p>
      {message.stepId ? <p className="mt-1 text-xs text-slate-500">Step: {message.stepId}</p> : null}
    </div>
  );
}

function RonPreview({ ron }: { ron: string }) {
  return (
    <section className="min-h-0 overflow-hidden bg-slate-950 p-4">
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-sm font-semibold uppercase tracking-wide text-slate-300">RON Preview</h3>
        <span className="text-xs text-slate-500">read-only draft output</span>
      </div>
      <pre className="h-[calc(100%-28px)] overflow-auto rounded-md border border-slate-800 bg-slate-900 p-3 text-xs leading-5 text-slate-300">
        {ron}
      </pre>
    </section>
  );
}

export default App;

function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
