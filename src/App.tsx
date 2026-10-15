import { useCallback, useEffect, useState } from "react";
import { BatchPanel } from "./components/BatchPanel";
import { ModelsBanner } from "./components/ModelsBanner";
import { PreviewPane } from "./components/PreviewPane";
import { SettingsPanel } from "./components/SettingsPanel";
import { useBatch } from "./hooks/useBatch";
import { loadSettings, saveSettings, type Settings } from "./lib/settings";

export default function App() {
  const [settings, setSettings] = useState<Settings>(loadSettings);
  const [modelsReady, setModelsReady] = useState(false);
  const batch = useBatch();

  useEffect(() => saveSettings(settings), [settings]);
  const onModelsReady = useCallback((ready: boolean) => setModelsReady(ready), []);

  const canStart = modelsReady && !!settings.inputDir && !!settings.outputDir;

  return (
    <div className="flex h-screen overflow-hidden">
      <SettingsPanel settings={settings} onChange={setSettings} />
      <main className="flex min-w-0 flex-1 flex-col">
        <header className="flex items-baseline gap-3 border-b border-zinc-200 px-5 py-3 dark:border-zinc-800">
          <h1 className="text-lg font-semibold">Kadrix</h1>
          <p className="text-sm text-zinc-500">Portraits uniformes pour trombinoscope</p>
        </header>
        <ModelsBanner onReady={onModelsReady} />
        <div className="flex min-h-0 flex-1 flex-col gap-4 p-4">
          <PreviewPane
            inputDir={settings.inputDir}
            params={settings.params}
            modelsReady={modelsReady}
            busy={batch.state.running}
          />
          <BatchPanel
            state={batch.state}
            canStart={canStart}
            onStart={() => batch.start(settings.inputDir, settings.outputDir, settings.params)}
            onCancel={batch.cancel}
          />
        </div>
      </main>
    </div>
  );
}
