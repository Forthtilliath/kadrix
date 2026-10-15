import { useEffect, useState, type ReactNode } from "react";

export function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="border-b border-zinc-200 px-5 py-4 dark:border-zinc-800">
      <h2 className="mb-3 text-xs font-semibold tracking-wide text-zinc-500 uppercase">{title}</h2>
      <div className="flex flex-col gap-3">{children}</div>
    </section>
  );
}

export function Button({
  children,
  onClick,
  disabled,
  variant = "secondary",
}: {
  children: ReactNode;
  onClick: () => void;
  disabled?: boolean;
  variant?: "primary" | "secondary" | "danger";
}) {
  const styles = {
    primary: "bg-teal-600 text-white hover:bg-teal-700",
    secondary:
      "border border-zinc-300 bg-white hover:bg-zinc-100 dark:border-zinc-700 dark:bg-zinc-900 dark:hover:bg-zinc-800",
    danger: "bg-rose-600 text-white hover:bg-rose-700",
  }[variant];
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      className={`rounded-md px-3 py-1.5 text-sm font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${styles}`}
    >
      {children}
    </button>
  );
}

/** Champ numérique : la saisie reste libre, la valeur est bornée à la sortie du champ. */
export function NumberField({
  label,
  value,
  onChange,
  min,
  max,
  step = 1,
  suffix,
}: {
  label: string;
  value: number;
  onChange: (v: number) => void;
  min: number;
  max: number;
  step?: number;
  suffix?: string;
}) {
  const [text, setText] = useState(String(value));
  useEffect(() => setText(String(value)), [value]);

  const commit = () => {
    const parsed = Number(text.replace(",", "."));
    const next = Number.isFinite(parsed) ? Math.min(max, Math.max(min, parsed)) : value;
    const rounded = step >= 1 ? Math.round(next) : next;
    setText(String(rounded));
    if (rounded !== value) onChange(rounded);
  };

  return (
    <label className="flex items-center justify-between gap-3 text-sm">
      <span>{label}</span>
      <span className="flex items-center gap-1.5">
        <input
          type="number"
          inputMode="decimal"
          value={text}
          min={min}
          max={max}
          step={step}
          onChange={(e) => setText(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => e.key === "Enter" && commit()}
          className="w-20 rounded-md border border-zinc-300 bg-white px-2 py-1 text-right tabular-nums dark:border-zinc-700 dark:bg-zinc-900"
        />
        {suffix && <span className="w-5 text-xs text-zinc-500">{suffix}</span>}
      </span>
    </label>
  );
}

export function Toggle({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="flex cursor-pointer items-center gap-2 text-sm">
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="size-4 accent-teal-600"
      />
      {label}
    </label>
  );
}

export function ColorField({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <label className="flex items-center justify-between gap-3 text-sm">
      <span>{label}</span>
      <span className="flex items-center gap-2">
        <span className="font-mono text-xs text-zinc-500">{value}</span>
        <input
          type="color"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className="h-7 w-10 cursor-pointer rounded border border-zinc-300 bg-transparent dark:border-zinc-700"
        />
      </span>
    </label>
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
        <Button onClick={onPick}>Choisir…</Button>
      </div>
    </div>
  );
}
