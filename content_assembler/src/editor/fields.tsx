import { useId, useState, type ReactNode } from "react";
import { Input } from "@/components/ui/input";
// Remount by document revision/selection in the owner. Blur/Enter commits; Escape discards.
export function Field({
  label,
  value,
  onCommit,
  numeric = false,
  scale = 1,
  disabled = false,
}: {
  label: string;
  value: string | number;
  onCommit: (v: string) => void;
  numeric?: boolean;
  scale?: number;
  disabled?: boolean;
}) {
  const id = useId();
  const shown = String(typeof value === "number" ? value / scale : value);
  const [buffer, setBuffer] = useState({ base: shown, draft: shown });
  const draft = buffer.base === shown ? buffer.draft : shown;
  const setDraft = (draft: string) => setBuffer({ base: shown, draft });
  const [error, setError] = useState(false);
  const commit = () => {
    if (numeric && (!draft.trim() || !Number.isFinite(Number(draft)))) {
      setDraft(shown);
      setError(true);
      return;
    }
    setError(false);
    setDraft(shown);
    if (draft !== shown)
      onCommit(numeric ? String(Number(draft) * scale) : draft);
  };
  return (
    <label className="field" htmlFor={id}>
      <span>{label}</span>
      <Input
        id={id}
        value={draft}
        disabled={disabled}
        inputMode={numeric ? "decimal" : undefined}
        onChange={(e) => {
          setDraft(e.target.value);
          setError(false);
        }}
        onBlur={commit}
        onKeyDown={(e) => {
          if (e.nativeEvent.isComposing) return;
          if (e.key === "Enter") {
            e.preventDefault();
            e.currentTarget.blur();
          }
          if (e.key === "Escape") {
            setDraft(shown);
            setError(false);
            e.stopPropagation();
          }
        }}
      />
      {error && (
        <small role="alert">숫자가 아니므로 이전 값으로 복원했습니다.</small>
      )}
    </label>
  );
}
export function Choice({
  label,
  value,
  onChange,
  children,
  disabled = false,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  children: ReactNode;
  disabled?: boolean;
}) {
  return (
    <label className="field">
      <span>{label}</span>
      <select
        aria-label={label}
        value={value}
        disabled={disabled}
        onChange={(e) => onChange(e.target.value)}
      >
        {children}
      </select>
    </label>
  );
}
export function Reference({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: { id: string; name: string }[];
  onChange: (v: string) => void;
}) {
  return (
    <Choice label={label} value={value} onChange={onChange}>
      <option value="">미지정</option>
      {value && !options.some((o) => o.id === value) && (
        <option value={value}>끊어진 참조: {value}</option>
      )}
      {options.map((o) => (
        <option key={o.id} value={o.id}>
          {o.name || "(이름 없음)"}
        </option>
      ))}
    </Choice>
  );
}
