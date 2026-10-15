import { useCallback, useEffect, useState } from "react";
import { downloadModels, errorMessage, modelStatus, type ModelInfo } from "../lib/api";
import { Button } from "@forthtilliath/forth-ui/components/button";

const formatSize = (bytes: number) =>
  bytes >= 1e9 ? `${(bytes / 1e9).toFixed(2)} Go` : `${Math.max(1, Math.round(bytes / 1e6))} Mo`;

/** Affiché tant que les modèles ne sont pas téléchargés. */
export function ModelsBanner({ onReady }: { onReady: (ready: boolean) => void }) {
  const [models, setModels] = useState<ModelInfo[] | null>(null);
  const [progress, setProgress] = useState<Record<string, number>>({});
  const [downloading, setDownloading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const list = await modelStatus();
      setModels(list);
      onReady(list.every((m) => m.present));
    } catch (e) {
      setError(errorMessage(e));
    }
  }, [onReady]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  if (!models || models.every((m) => m.present)) return null;

  const missing = models.filter((m) => !m.present);
  const totalBytes = missing.reduce((sum, m) => sum + m.sizeBytes, 0);
  const receivedBytes = missing.reduce((sum, m) => sum + (progress[m.id] ?? 0), 0);

  const start = async () => {
    setDownloading(true);
    setError(null);
    try {
      await downloadModels((p) => setProgress((prev) => ({ ...prev, [p.model]: p.received })));
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setDownloading(false);
      void refresh();
    }
  };

  return (
    <div className="m-4 mb-0 rounded-lg border border-amber-300 bg-amber-50 p-4 text-sm dark:border-amber-800 dark:bg-amber-950/40">
      <p className="font-medium">Modèles à télécharger ({formatSize(totalBytes)}, une seule fois)</p>
      <ul className="mt-1 text-xs text-zinc-600 dark:text-zinc-400">
        {missing.map((m) => (
          <li key={m.id}>
            {m.label} — {formatSize(m.sizeBytes)}, licence {m.license}
          </li>
        ))}
      </ul>
      <p className="mt-1 text-xs text-zinc-500">Chaque fichier est vérifié (empreinte SHA-256) avant d'être utilisé.</p>
      <div className="mt-3 flex items-center gap-3">
        <Button size="sm" onClick={start} loading={downloading}>
          {downloading ? "Téléchargement…" : "Télécharger"}
        </Button>
        {downloading && (
          <div className="h-2 flex-1 overflow-hidden rounded bg-amber-200 dark:bg-amber-900">
            <div className="h-full bg-teal-600 transition-[width]" style={{ width: `${(receivedBytes / totalBytes) * 100}%` }} />
          </div>
        )}
      </div>
      {error && <p className="mt-2 text-xs text-rose-600">{error}</p>}
    </div>
  );
}
