import { Channel, invoke } from "@tauri-apps/api/core";
import type { Params } from "./settings";

export type Level = "error" | "warning" | "info";

export interface Issue {
  level: Level;
  code: string;
  message: string;
}

export interface ModelInfo {
  id: string;
  label: string;
  license: string;
  sizeBytes: number;
  present: boolean;
}

export interface DownloadProgress {
  model: string;
  received: number;
  total: number;
}

export type ItemStatus = "ok" | "warning" | "error";

export interface ItemReport {
  file: string;
  status: ItemStatus;
  issues: Issue[];
  outputs: string[];
  thumbnail?: string;
  millis: number;
}

export type BatchEvent =
  | { type: "started"; total: number }
  | { type: "item"; index: number; item: ItemReport }
  | { type: "finished"; reportPath: string; cancelled: boolean };

export interface PreviewResult {
  image: string | null;
  issues: Issue[];
}

export const modelStatus = () => invoke<ModelInfo[]>("model_status");

export function downloadModels(onProgress: (p: DownloadProgress) => void) {
  const channel = new Channel<DownloadProgress>();
  channel.onmessage = onProgress;
  return invoke<void>("download_models", { onProgress: channel });
}

export const listImages = (dir: string) => invoke<string[]>("list_images", { dir });

export const preview = (path: string, params: Params) =>
  invoke<PreviewResult>("preview", { path, params });

export function runBatch(
  inputDir: string,
  outputDir: string,
  params: Params,
  onEvent: (e: BatchEvent) => void,
) {
  const channel = new Channel<BatchEvent>();
  channel.onmessage = onEvent;
  return invoke<string>("run_batch", { inputDir, outputDir, params, onEvent: channel });
}

export const cancelBatch = () => invoke<void>("cancel_batch");
export const openOutput = () => invoke<void>("open_output");

/** Message lisible pour une erreur renvoyée par le cœur Rust. */
export function errorMessage(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : "Erreur inconnue.";
}

export function baseName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}
