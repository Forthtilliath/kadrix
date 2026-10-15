import { useEffect, useState } from "react";
import { baseName, errorMessage, listImages, preview, type PreviewResult } from "../lib/api";
import type { Params } from "../lib/settings";
import { IssueList } from "./IssueList";
import { Button } from "./ui";

interface Props {
  inputDir: string;
  params: Params;
  modelsReady: boolean;
  busy: boolean;
}

/** Liste des photos du dossier + aperçu d'une photo avec les réglages courants. */
export function PreviewPane({ inputDir, params, modelsReady, busy }: Props) {
  const [files, setFiles] = useState<string[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [result, setResult] = useState<PreviewResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setFiles([]);
    setSelected(null);
    setResult(null);
    if (!inputDir) return;
    listImages(inputDir).then(setFiles, (e) => setError(errorMessage(e)));
  }, [inputDir]);

  const run = async (path: string) => {
    setSelected(path);
    setLoading(true);
    setError(null);
    try {
      setResult(await preview(path, params));
    } catch (e) {
      setResult(null);
      setError(errorMessage(e));
    } finally {
      setLoading(false);
    }
  };

  const disabled = !modelsReady || busy || loading;

  return (
    <div className="flex min-h-0 flex-1 gap-4">
      <div className="flex w-64 shrink-0 flex-col overflow-hidden rounded-lg border border-zinc-200 bg-white dark:border-zinc-800 dark:bg-zinc-900">
        <p className="border-b border-zinc-200 px-3 py-2 text-xs text-zinc-500 dark:border-zinc-800">
          {inputDir ? `${files.length} photo(s)` : "Choisissez le dossier des photos"}
        </p>
        <ul className="flex-1 overflow-y-auto text-sm">
          {files.map((f) => (
            <li key={f}>
              <button
                type="button"
                disabled={disabled}
                onClick={() => run(f)}
                className={`w-full truncate px-3 py-1.5 text-left hover:bg-zinc-100 disabled:cursor-not-allowed dark:hover:bg-zinc-800 ${
                  f === selected ? "bg-teal-50 font-medium dark:bg-teal-950" : ""
                }`}
              >
                {baseName(f)}
              </button>
            </li>
          ))}
        </ul>
      </div>

      <div className="flex min-w-0 flex-1 flex-col gap-3 rounded-lg border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-900">
        <div className="flex items-center justify-between gap-3">
          <p className="truncate text-sm font-medium">{selected ? baseName(selected) : "Aperçu"}</p>
          {selected && (
            <Button onClick={() => run(selected)} disabled={disabled}>
              Recalculer
            </Button>
          )}
        </div>
        <div className="checker flex min-h-0 flex-1 items-center justify-center overflow-hidden rounded-md">
          {loading ? (
            <p className="rounded bg-white/80 px-3 py-1 text-sm text-zinc-700">Traitement…</p>
          ) : result?.image ? (
            <img src={result.image} alt="Aperçu du portrait traité" className="max-h-full max-w-full object-contain shadow" />
          ) : (
            <p className="rounded bg-white/80 px-3 py-1 text-sm text-zinc-700">
              {modelsReady ? "Cliquez une photo pour tester les réglages." : "Téléchargez d'abord les modèles."}
            </p>
          )}
        </div>
        {result && <IssueList issues={result.issues} />}
        {error && <p className="text-xs text-rose-600">{error}</p>}
      </div>
    </div>
  );
}
