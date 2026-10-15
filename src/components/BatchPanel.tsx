import { useState } from "react";
import type { BatchState } from "../hooks/useBatch";
import { errorMessage, openOutput, type ItemReport, type ItemStatus } from "../lib/api";
import { IssueList } from "./IssueList";
import { Button } from "@forthtilliath/forth-ui/components/button";
import { Segmented } from "./ui";

interface Props {
  state: BatchState;
  canStart: boolean;
  onStart: () => void;
  onCancel: () => void;
}

type Filter = "all" | "problems";

const STATUS_DOT: Record<ItemStatus, string> = {
  ok: "bg-emerald-500",
  warning: "bg-amber-500",
  error: "bg-rose-500",
};

export function BatchPanel({ state, canStart, onStart, onCancel }: Props) {
  const [filter, setFilter] = useState<Filter>("problems");
  const [openError, setOpenError] = useState<string | null>(null);
  const done = state.items.length;
  const count = (s: ItemStatus) => state.items.filter((i) => i.status === s).length;
  const shown = filter === "all" ? state.items : state.items.filter((i) => i.status !== "ok");

  return (
    <div className="flex max-h-[45%] min-h-48 flex-col gap-3 rounded-lg border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-900">
      <div className="flex flex-wrap items-center gap-3">
        {state.running ? (
          <Button variant="destructive" size="sm" onClick={onCancel}>
            Arrêter
          </Button>
        ) : (
          <Button size="sm" onClick={onStart} disabled={!canStart}>
            Traiter tout le dossier
          </Button>
        )}
        {state.total > 0 && (
          <>
            <div className="h-2 min-w-32 flex-1 overflow-hidden rounded bg-zinc-200 dark:bg-zinc-800">
              <div className="h-full bg-teal-600 transition-[width]" style={{ width: `${(done / state.total) * 100}%` }} />
            </div>
            <span className="text-sm tabular-nums">
              {done} / {state.total}
            </span>
            <span className="text-xs text-zinc-500">
              {count("ok")} ok · {count("warning")} à vérifier · {count("error")} non traitée(s)
            </span>
          </>
        )}
        {state.reportPath && !state.running && (
          <Button variant="outline" size="sm" onClick={() => openOutput().catch((e) => setOpenError(errorMessage(e)))}>Ouvrir le dossier de sortie</Button>
        )}
      </div>

      {state.error && <p className="text-sm text-rose-600">{state.error}</p>}
      {openError && <p className="text-sm text-rose-600">{openError}</p>}
      {state.cancelled && <p className="text-sm text-amber-600">Traitement arrêté : le rapport couvre les photos déjà traitées.</p>}
      {state.reportPath && <p className="truncate text-xs text-zinc-500">Rapport : {state.reportPath}</p>}

      {state.items.length > 0 && (
        <>
          <div className="w-72">
            <Segmented
              value={filter}
              options={[
                { value: "problems", label: "À vérifier" },
                { value: "all", label: "Toutes" },
              ]}
              onChange={setFilter}
            />
          </div>
          <ul className="grid min-h-0 grid-cols-[repeat(auto-fill,minmax(15rem,1fr))] gap-3 overflow-y-auto">
            {shown.map((item) => (
              <ResultCard key={item.file} item={item} />
            ))}
          </ul>
        </>
      )}
    </div>
  );
}

function ResultCard({ item }: { item: ItemReport }) {
  return (
    <li className="flex gap-3 rounded-md border border-zinc-200 p-2 dark:border-zinc-800">
      {item.thumbnail ? (
        <img src={item.thumbnail} alt="" className="h-20 w-16 shrink-0 rounded object-cover" />
      ) : (
        <div className="checker h-20 w-16 shrink-0 rounded" />
      )}
      <div className="flex min-w-0 flex-col gap-1">
        <p className="flex items-center gap-1.5 text-sm">
          <span className={`size-2 shrink-0 rounded-full ${STATUS_DOT[item.status]}`} />
          <span className="truncate" title={item.file}>
            {item.file}
          </span>
        </p>
        <IssueList issues={item.issues} />
        <p className="text-xs text-zinc-500">{(item.millis / 1000).toFixed(1)} s</p>
      </div>
    </li>
  );
}
