import { useState } from "react";
import { cancelBatch, errorMessage, runBatch, type BatchEvent, type ItemReport } from "../lib/api";
import type { Params } from "../lib/settings";

export interface BatchState {
  running: boolean;
  total: number;
  items: ItemReport[];
  reportPath: string | null;
  cancelled: boolean;
  error: string | null;
}

const IDLE: BatchState = { running: false, total: 0, items: [], reportPath: null, cancelled: false, error: null };

export function useBatch() {
  const [state, setState] = useState<BatchState>(IDLE);

  const onEvent = (event: BatchEvent) => {
    setState((s) => {
      switch (event.type) {
        case "started":
          return { ...s, total: event.total };
        case "item":
          return { ...s, items: [...s.items, event.item] };
        case "finished":
          return { ...s, reportPath: event.reportPath, cancelled: event.cancelled };
      }
    });
  };

  const start = async (inputDir: string, outputDir: string, params: Params) => {
    setState({ ...IDLE, running: true });
    try {
      await runBatch(inputDir, outputDir, params, onEvent);
    } catch (e) {
      setState((s) => ({ ...s, error: errorMessage(e) }));
    } finally {
      setState((s) => ({ ...s, running: false }));
    }
  };

  const cancel = () => void cancelBatch();

  return { state, start, cancel };
}
