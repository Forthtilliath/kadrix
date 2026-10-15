import type { ReactNode } from "react";
import { Button } from "@forthtilliath/forth-ui/components/button";
import { ColorPicker } from "@forthtilliath/forth-ui/components/color-picker";
import { NumberInput } from "@forthtilliath/forth-ui/components/number-input";

export function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="border-b border-zinc-200 px-5 py-4 dark:border-zinc-800">
      <h2 className="mb-3 text-xs font-semibold tracking-wide text-zinc-500 uppercase">{title}</h2>
      <div className="flex flex-col gap-3">{children}</div>
    </section>
  );
}

/** Libellé à gauche, contrôle à droite. */
function FieldRow({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-3 text-sm">
      <span className="min-w-0">{label}</span>
      <span className="flex shrink-0 items-center gap-1.5">{children}</span>
    </div>
  );
}

/** Champ numérique borné : la saisie reste libre, la valeur est bornée à la sortie du champ. */
export function NumberField({
  label,
  value,
  onChange,
  min,
  max,
  suffix,
}: {
  label: string;
  value: number;
  onChange: (v: number) => void;
  min: number;
  max: number;
  suffix?: string;
}) {
  return (
    <FieldRow label={label}>
      <NumberInput aria-label={label} value={value} min={min} max={max} onValueChange={onChange} />
      {suffix && <span className="w-5 text-xs text-zinc-500">{suffix}</span>}
    </FieldRow>
  );
}

export function ColorField({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  return (
    <FieldRow label={label}>
      <ColorPicker ariaLabel={label} value={value} onValueChange={onChange} />
    </FieldRow>
  );
}

export function Segmented<T extends string>({
  value,
  options,
  onChange,
}: {
  value: T;
  options: { value: T; label: string }[];
  onChange: (v: T) => void;
}) {
  return (
    <div className="flex rounded-md border border-zinc-300 p-0.5 text-sm dark:border-zinc-700">
      {options.map((o) => (
        <button
          key={o.value}
          type="button"
          aria-pressed={o.value === value}
          onClick={() => onChange(o.value)}
          className={`flex-1 rounded px-2 py-1 transition-colors ${
            o.value === value
              ? "bg-teal-600 text-white"
              : "text-zinc-600 hover:bg-zinc-100 dark:text-zinc-400 dark:hover:bg-zinc-800"
          }`}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}

export function PathField({
  label,
  value,
  placeholder,
  onPick,
}: {
  label: string;
  value: string;
  placeholder: string;
  onPick: () => void;
}) {
  return (
    <div className="flex flex-col gap-1 text-sm">
      <span>{label}</span>
      <div className="flex items-center gap-2">
        <span
          title={value}
          className="min-w-0 flex-1 truncate rounded-md border border-zinc-300 bg-white px-2 py-1 text-xs text-zinc-600 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-400"
        >
          {value || placeholder}
        </span>
        <Button variant="outline" size="sm" onClick={onPick}>
          Choisir…
        </Button>
      </div>
    </div>
  );
}
