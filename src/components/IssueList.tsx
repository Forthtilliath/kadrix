import type { Issue } from "../lib/api";

const LEVEL_STYLE = {
  error: "text-rose-600 dark:text-rose-400",
  warning: "text-amber-600 dark:text-amber-400",
  info: "text-zinc-500",
} as const;

const LEVEL_ICON = { error: "✕", warning: "!", info: "i" } as const;

export function IssueList({ issues }: { issues: Issue[] }) {
  if (issues.length === 0) return null;
  return (
    <ul className="flex flex-col gap-0.5 text-xs">
      {issues.map((issue, i) => (
        <li key={`${issue.code}-${i}`} className={`flex gap-1.5 ${LEVEL_STYLE[issue.level]}`}>
          <span className="w-3 shrink-0 text-center font-bold">{LEVEL_ICON[issue.level]}</span>
          <span>{issue.message}</span>
        </li>
      ))}
    </ul>
  );
}
